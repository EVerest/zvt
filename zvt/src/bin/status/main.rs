use clap::Parser;
use env_logger::{Builder, Env};
use log::info;
use std::io::Write;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use zvt::{logging, packets, sequences, sequences::Sequence};

#[derive(Parser, Debug)]
struct Args {
    /// The ip and port of the payment terminal.
    #[clap(long, default_value = "localhost:22000")]
    ip: String,

    /// The password of the payment terminal.
    #[clap(long, default_value = "123456")]
    password: usize,

    /// The config byte for the registration. Defaults to 0xDE (= 222).
    #[clap(long, default_value = "222")]
    config_byte: u8,

    /// The currency code
    #[clap(long, default_value = "978")]
    currency_code: usize,

    /// The terminal id to be set.
    #[clap(long, default_value = "123456")]
    terminal_id: usize,
}

fn init_logger() {
    let env = Env::default().filter_or("ZVT_LOGGER_LEVEL", "info");

    Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "<{}>{}: {}",
                match record.level() {
                    log::Level::Error => 3,
                    log::Level::Warn => 4,
                    log::Level::Info => 6,
                    log::Level::Debug => 7,
                    log::Level::Trace => 7,
                },
                record.target(),
                record.args()
            )
        })
        .init();
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logger();
    let args = Args::parse();

    info!("Using the args {:?}", args);
    let source = TcpStream::connect(args.ip).await?;
    let mut socket = logging::PacketWriter { source };

    let request = packets::Registration {
        password: args.password,
        config_byte: args.config_byte,
        currency: Some(args.currency_code),
        tlv: None,
    };

    info!("Running Registration with {request:?}");

    let mut stream = sequences::Registration::into_stream(&request, &mut socket);
    while let Some(response) = stream.next().await {
        info!("Response to Registration: {:?}", response);
    }
    drop(stream);

    let request = packets::SetTerminalId {
        password: args.password,
        terminal_id: Some(args.terminal_id),
    };
    info!("Running SetTerminalId with {request:?}");
    let mut stream = sequences::SetTerminalId::into_stream(&request, &mut socket);
    while let Some(response) = stream.next().await {
        info!("Response to SetTerminalId: {:?}", response);
    }
    drop(stream);

    let request = packets::Initialization {
        password: args.password,
    };
    info!("Running Initialization with {request:?}");
    let mut stream = sequences::Initialization::into_stream(&request, &mut socket);
    while let Some(response) = stream.next().await {
        info!("Response to Initialization: {:?}", response);
    }
    drop(stream);

    let request = packets::Diagnosis {
        tlv: Some(packets::tlv::Diagnosis {
            diagnosis_type: Some(1),
        }),
    };
    info!("Running Diagnosing with {request:?}");
    let mut stream = sequences::Diagnosis::into_stream(&request, &mut socket);
    while let Some(response) = stream.next().await {
        info!("Response to Diagnosis: {:?}", response);
    }
    drop(stream);

    let request = packets::PrintSystemConfiguration {};
    info!("Running PrintSystemConfiguration");
    let mut stream = sequences::PrintSystemConfiguration::into_stream(&request, &mut socket);
    while let Some(response) = stream.next().await {
        info!("Response to PrintSystemConfiguration: {:?}", response);
    }
    drop(stream);

    let request = packets::EndOfDay {
        password: args.password,
    };

    info!("Running EndOfDay with {request:?}");
    let mut stream = sequences::EndOfDay::into_stream(&request, &mut socket);
    while let Some(response) = stream.next().await {
        info!("Response to EndOfDay: {:?}", response);
    }
    drop(stream);

    let request = packets::StatusEnquiry {
        password: None,
        service_byte: None,
        tlv: None,
    };
    info!("Running StatusEnquiry with {request:?}");
    let mut stream = sequences::StatusEnquiry::into_stream(&request, &mut socket);
    while let Some(response) = stream.next().await {
        info!("Response to StatusEnquiry: {:?}", response);
    }
    drop(stream);

    let request = packets::PartialReversal {
        receipt_no: Some(0xffff),
        amount: None,
        payment_type: None,
        currency: None,
        tlv: None,
    };

    info!("Running PartialReversalData with {request:?}");
    let mut stream = sequences::PartialReversal::into_stream(&request, &mut socket);
    while let Some(response) = stream.next().await {
        info!("Response to PartialReversalData: {:?}", response);
    }
    drop(stream);

    Ok(())
}
