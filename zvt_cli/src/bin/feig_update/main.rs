use anyhow::Result;
use argh::FromArgs;
use serde::Deserialize;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use zvt::{feig, packets, sequences, sequences::Sequence};

#[derive(FromArgs, Debug)]
/// Updates a feig terminal.
struct Args {
    /// ip and port of the payment terminal.
    #[argh(option, default = "\"localhost:22000\".to_string()")]
    ip_address: String,

    /// password of the payment terminal. The password is a 6-digits code,
    /// e.x. 123456.
    #[argh(option)]
    password: usize,

    /// config byte for the registration.
    #[argh(option, default = "222")]
    config_byte: u8,

    /// forces the update. The update will otherwise be skipped if the returned
    /// software version corresponds to the version stored in app1/update.spec.
    #[argh(switch)]
    force: bool,

    /// folder containing the payload, e.x. firmware and app1 folders.
    #[argh(positional)]
    payload_dir: PathBuf,
}

#[derive(Deserialize)]
struct UpdateSpec {
    version: String,
}

/// Returns the desired version of the App.
///
/// We're using the app1/update.spec as a proxy for the version of the entire
/// firmware update. Returns an error if the desired version cannot be read.
fn get_desired_version(payload_dir: &Path) -> Result<String> {
    let path = payload_dir.join("app1/update.spec");
    let update_spec_str = read_to_string(path)?;
    let update_spec: UpdateSpec = serde_json::from_str(&update_spec_str)?;
    Ok(update_spec.version)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = argh::from_env();

    // Connect to the payment terminal.
    let mut socket = TcpStream::connect(&args.ip_address).await?;
    const MAX_LEN_ADPU: u16 = 1u16 << 15;
    let registration = packets::Registration {
        password: args.password,
        config_byte: args.config_byte,
        currency: None,
        tlv: Some(packets::tlv::Registration {
            max_len_adpu: Some(MAX_LEN_ADPU),
        }),
    };

    {
        // Register to the terminal.
        let mut stream = sequences::Registration::into_stream(&registration, &mut socket);
        while let Some(response) = stream.next().await {
            match response {
                Err(_) => panic!("Failed to register to the terminal"),
                Ok(completion) => println!("Registered to the terminal {:?}", completion),
            }
        }
    }

    {
        // Check the current version of the software
        let request = feig::packets::CVendFunctions {
            password: None,
            instr: 1,
        };
        let mut stream = feig::sequences::GetSystemInfo::into_stream(&request, &mut socket);
        let mut current_version = "unknown".to_string();
        while let Some(response) = stream.next().await {
            match response {
                Err(_) => panic!("Failed to get the system info"),
                Ok(completion) => {
                    println!("The system info returned {:?}", completion);
                    if let feig::sequences::GetSystemInfoResponse::CVendFunctionsEnhancedSystemInformationCompletion(packet) = completion {
                        current_version = packet.sw_version;
                    }
                }
            }
        }

        // Check if we have to run the update.
        if !args.force {
            match get_desired_version(&args.payload_dir) {
                Ok(desired_version) => {
                    // We can't go for strict equality since the desired version
                    // contains just a semantic version e.x. `2.0.12` and the
                    // actual also contains the language e.x. `GER-APP-v2.0.12`.
                    if current_version.contains(&desired_version) {
                        println!("Skipping update");
                        return Ok(());
                    }
                }
                Err(err) => println!("Failed to get the current version {}", err),
            }
        }
    }

    // If the terminal has a pending EOD job the update will fail. Therefore
    // we precautionary run the EOD job here. However, if the payment terminal
    // is not setup yet, the EOD will fail. We therefore ignore all errors
    // during the EOD job.
    {
        let request = packets::EndOfDay {
            password: args.password,
        };
        let mut stream = sequences::EndOfDay::into_stream(&request, &mut socket);
        while let Some(response) = stream.next().await {
            println!("The EndOfDay returned {response:?}");
        }
    }

    {
        // Update the app.
        let mut stream = feig::sequences::WriteFile::into_stream(
            args.payload_dir,
            args.password,
            MAX_LEN_ADPU.into(),
            &mut socket,
        );
        while let Some(response) = stream.next().await {
            match response {
                Err(_) => panic!("Failed to update the terminal"),
                Ok(inner) => {
                    println!("Updating the terminal {:?}", inner);
                    match inner {
                        feig::sequences::WriteFileResponse::Abort(abort) => {
                            panic!("Failed to update the terminal {abort:?}")
                        }
                        _ => {}
                    }
                }
            }
        }
        println!("Finished the update");
    }

    Ok(())
}
