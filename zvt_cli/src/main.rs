use anyhow::{bail, Result};
use argh::FromArgs;
use env_logger::{Builder, Env};
use std::io::Write;
use std::net::Ipv4Addr;
use std::str::FromStr;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use zvt::sequences::Sequence;
use zvt::{feig, packets, sequences};

type PacketTransport = zvt::io::PacketTransport<TcpStream>;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommands {
    Status(StatusArgs),
    FactoryReset(FactoryResetArgs),
    Registration(RegistrationArgs),
    SetTerminalId(SetTerminalIdArgs),
    Initialization(InitializationArgs),
    Diagnosis(DiagnosisArgs),
    PrintSystemConfiguration(PrintSystemConfigurationArgs),
    EndOfDay(EndOfDayArgs),
    ReadCard(ReadCardArgs),
    Reservation(ReservationArgs),
    PartialReversal(PartialReversalArgs),
    ChangeHostConfiguration(ChangeHostConfigurationArgs),
}

#[derive(Debug, PartialEq)]
enum StatusType {
    Feig,
    Zvt,
}

impl FromStr for StatusType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "feig" => Ok(StatusType::Feig),
            "zvt" => Ok(StatusType::Zvt),
            _ => Err(anyhow::anyhow!(
                "'{s}' is not a valid StatusType (feig | zvt)"
            )),
        }
    }
}

#[derive(FromArgs, PartialEq, Debug)]
/// Query the status.
#[argh(subcommand, name = "status")]
struct StatusArgs {
    /// which type of status to use (feig | zvt)
    #[argh(option, default = "StatusType::Zvt")]
    r#type: StatusType,

    /// in case of zvt - which service byte to use. See section 2.55.1 for more details.
    #[argh(option)]
    service_byte: Option<u8>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Factory resets the terminal.
#[argh(subcommand, name = "factory_reset")]
struct FactoryResetArgs {}

#[derive(FromArgs, PartialEq, Debug)]
/// Runs registration.
#[argh(subcommand, name = "registration")]
struct RegistrationArgs {
    /// currency code. Defauls to 978 (= EUR).
    #[argh(option, default = "978")]
    currency_code: usize,

    /// config byte. Defaults to 0xDE (= 222).
    #[argh(option, default = "222")]
    config_byte: u8,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Sets the terminal id.
#[argh(subcommand, name = "set_terminal_id")]
struct SetTerminalIdArgs {
    /// terminal id to be set.
    #[argh(option)]
    terminal_id: usize,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Runs initialization.
#[argh(subcommand, name = "init")]
struct InitializationArgs {}

fn parse_diagnosis(s: &str) -> std::result::Result<packets::DiagnosisType, String> {
    match s {
        "1" | "line" => Ok(packets::DiagnosisType::Line),
        "2" | "extended" => Ok(packets::DiagnosisType::Extended),
        "3" | "configuration" => Ok(packets::DiagnosisType::Configuration),
        "4" | "emv_configuration" => Ok(packets::DiagnosisType::EmvConfiguration),
        "5" | "ep2_configuration" => Ok(packets::DiagnosisType::Ep2Configuration),
        _ => Err(format!("Invalid argument: '{s}'. Valid is line, extended, configuration, emv_configuration or ep2_configuration.")),
    }
}

#[derive(FromArgs, PartialEq, Debug)]
/// Runs diagnosis
#[argh(subcommand, name = "diagnosis")]
struct DiagnosisArgs {
    /// the type of diagnosis to run. See DiagnosisType for options.
    #[argh(positional, from_str_fn(parse_diagnosis))]
    diagnosis: packets::DiagnosisType,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Prints the terminals configuration.
#[argh(subcommand, name = "print_system_diagnosis")]
struct PrintSystemConfigurationArgs {}

#[derive(FromArgs, PartialEq, Debug)]
/// Runs an end of day job.
#[argh(subcommand, name = "end_of_day")]
struct EndOfDayArgs {}

#[derive(FromArgs, PartialEq, Debug)]
/// Waits for a card to be presented and prints information about the card.
#[argh(subcommand, name = "read_card")]
struct ReadCardArgs {
    /// the timeout to wait for cards in seconds.
    #[argh(option, default = "15")]
    timeout: u8,

    /// card type as defined in Table 6. Default is chip-card.
    #[argh(option, default = "16")]
    card_type: u8,

    /// reading control. See Tlv tag 0x1f15 for the documentation. The default detects bank cards
    /// and RFID cards, but does not send commands for payment cards.
    // TODO(hrapp): If we only want to read a card once, we probably have to do something here.
    #[argh(option, default = "208")]
    short_card_reading_control: u8,

    /// dialog control as defined in Table 7. Which only mention the  choice between 1 and 0.
    /// However, some terminals accept 2 and this silences their beeps..
    #[argh(option, default = "2")]
    dialog_control: u8,

    /// allowed cards ot read. The default reads all possible cards. See Tlv tag 0x1f60 for
    /// documentation.
    #[argh(option, default = "7")]
    allowed_cards: u8,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Reserve money on a card.
#[argh(subcommand, name = "reservation")]
struct ReservationArgs {
    /// currency code. The default is EUR.
    #[argh(option, default = "978")]
    currency_code: usize,

    /// amount in the fractional monetary unit (see
    /// https://www.thefreedictionary.com/fractional+monetary+unit) which should be pre-authorized.
    #[argh(option, default = "5")]
    amount: usize,

    /// the payment type, defined in table 4. The default is "Payment according to PTs decision
    /// excluding `GeldKarte`.
    // TODO(hrapp): When bit 2 is set here, the PT should execute the payment using the data from
    // the previous ReadCard command. Which might mean we do not need to reread.
    #[argh(option, default = "64")]
    payment_type: u8,

    /// track 2 data to identify past read card.
    #[argh(option)]
    track_2_data: Option<String>,

    /// bmp_prefix. If this is set, bmp_data must be set too.
    #[argh(option)]
    bmp_prefix: Option<String>,

    /// bmp_data. If this is set, bmp_prefix must be set too.
    #[argh(option)]
    bmp_data: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Returns some money of a prior Reservation.
#[argh(subcommand, name = "partial_reversal")]
struct PartialReversalArgs {
    /// the receipt number to partially reverse.
    #[argh(option)]
    receipt: usize,

    /// currency code. The default is EUR.
    #[argh(option, default = "978")]
    currency_code: usize,

    /// the amount in the fractional monetary unit that should be reversed  pre-authorized. So if
    /// the original reservation was for 500 and a reversal is executed with 50, the booked amount
    /// becomes 450.
    #[argh(option, default = "5")]
    amount: usize,

    /// see reservation.
    #[argh(option, default = "64")]
    payment_type: u8,

    /// see reservation.
    #[argh(option)]
    bmp_prefix: Option<String>,

    /// see reservation.
    #[argh(option)]
    bmp_data: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Changes the Host the payment terminal connects to.
#[argh(subcommand, name = "change_host_config")]
struct ChangeHostConfigurationArgs {
    /// the IP the terminal should connect to.
    #[argh(option)]
    ip: Ipv4Addr,

    /// the port the terminal should connect to.
    #[argh(option, default = "30401")]
    port: u16,

    /// see reservation.
    #[argh(option, default = "1")]
    configuration_byte: u8,
}

#[derive(FromArgs, Debug)]
/// Example tool to interact with the payment terminal.
struct Args {
    /// ip and port of the payment terminal.
    #[argh(option, default = "\"localhost:22000\".to_string()")]
    ip: String,

    /// password of the payment terminal.
    #[argh(option, default = "123456")]
    password: usize,

    #[argh(subcommand)]
    command: SubCommands,
}

fn init_logger() {
    let env = Env::default().filter_or("ZVT_LOGGER_LEVEL", "info");

    Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{}: {}",
                match record.level() {
                    log::Level::Error => 3,
                    log::Level::Warn => 4,
                    log::Level::Info => 6,
                    log::Level::Debug => 7,
                    log::Level::Trace => 7,
                },
                record.args()
            )
        })
        .init();
}

async fn status(
    socket: &mut PacketTransport,
    password: usize,
    status_type: StatusType,
    service_byte: Option<u8>,
) -> Result<()> {
    match status_type {
        StatusType::Feig => {
            // Check the current version of the software
            let request = feig::packets::CVendFunctions {
                password: None,
                instr: feig::constants::CVendFunctions::SystemsInfo as u16,
            };
            let mut stream = feig::sequences::GetSystemInfo::into_stream(&request, socket);
            while let Some(response) = stream.next().await {
                use feig::sequences::GetSystemInfoResponse::*;
                match response? {
                    CVendFunctionsEnhancedSystemInformationCompletion(data) => {
                        log::info!("{data:#?}")
                    }
                    Abort(_) => bail!("Failed to get system info. Received Abort."),
                }
            }
        }
        StatusType::Zvt => {
            let request = packets::StatusEnquiry {
                password: Some(password),
                service_byte: service_byte,
                tlv: None,
            };

            // See table 12 in the definition. We cannot parse this reqeust
            // correctly.
            if let Some(sb) = service_byte {
                if (sb & 0x02) == 0 {
                    log::warn!("The 'Do send SW-Version' is not supported. The output will be not correctly parsed.");
                }
            }

            let mut stream = sequences::StatusEnquiry::into_stream(&request, socket);
            while let Some(response) = stream.next().await {
                log::info!("{response:#?}");
            }
        }
    }
    Ok(())
}

async fn factory_reset(socket: &mut PacketTransport, password: usize) -> Result<()> {
    let request = feig::packets::CVendFunctions {
        password: Some(password),
        instr: feig::constants::CVendFunctions::FactoryReset as u16,
    };
    let mut stream = feig::sequences::FactoryReset::into_stream(&request, socket);
    while let Some(response) = stream.next().await {
        use feig::sequences::FactoryResetResponse::*;
        match response? {
            CompletionData(data) => log::info!("{data:#?}"),
        }
    }
    Ok(())
}

async fn registration(
    socket: &mut PacketTransport,
    password: usize,
    args: &RegistrationArgs,
) -> Result<()> {
    let request = packets::Registration {
        password,
        config_byte: args.config_byte,
        currency: Some(args.currency_code),
        tlv: None,
    };

    let mut stream = sequences::Registration::into_stream(&request, socket);
    while let Some(response) = stream.next().await {
        use sequences::RegistrationResponse::*;
        match response? {
            CompletionData(data) => log::info!("{data:#?}"),
        }
    }
    Ok(())
}

async fn set_terminal_id(
    socket: &mut PacketTransport,
    password: usize,
    args: &SetTerminalIdArgs,
) -> Result<()> {
    let request = packets::SetTerminalId {
        password,
        terminal_id: Some(args.terminal_id),
    };

    let mut stream = sequences::SetTerminalId::into_stream(&request, socket);
    while let Some(response) = stream.next().await {
        use sequences::SetTerminalIdResponse::*;
        match response? {
            CompletionData(data) => log::info!("{data:#?}"),
            Abort(_) => bail!("Failed to get system info. Received Abort."),
        }
    }
    Ok(())
}

async fn initialization(socket: &mut PacketTransport, password: usize) -> Result<()> {
    let request = packets::Initialization { password };

    let mut stream = sequences::Initialization::into_stream(&request, socket);
    while let Some(response) = stream.next().await {
        use sequences::InitializationResponse::*;
        match response? {
            IntermediateStatusInformation(data) => log::info!("{data:#?}"),
            PrintLine(data) => log::info!("{}", data.text),
            PrintTextBlock(data) => log::info!("{data:#?}"),
            CompletionData(data) => log::info!("{data:#?}"),
            Abort(_) => bail!("Received Abort."),
        }
    }
    Ok(())
}

async fn diagnosis(socket: &mut PacketTransport, args: &DiagnosisArgs) -> Result<()> {
    let request = packets::Diagnosis {
        tlv: Some(packets::tlv::Diagnosis {
            diagnosis_type: Some(args.diagnosis as u8),
        }),
    };

    let mut stream = sequences::Diagnosis::into_stream(&request, socket);
    while let Some(response) = stream.next().await {
        use sequences::DiagnosisResponse::*;
        match response? {
            SetTimeAndDate(data) => log::info!("{data:#?}"),
            PrintLine(data) => log::info!("{}", data.text),
            PrintTextBlock(data) => log::info!("{data:#?}"),
            IntermediateStatusInformation(_) | CompletionData(_) => (),
            Abort(_) => bail!("Received Abort."),
        }
    }
    Ok(())
}

async fn print_system_diagnosis(socket: &mut PacketTransport) -> Result<()> {
    let request = packets::PrintSystemConfiguration {};
    let mut stream = sequences::PrintSystemConfiguration::into_stream(&request, socket);
    while let Some(response) = stream.next().await {
        use sequences::PrintSystemConfigurationResponse::*;
        match response? {
            PrintLine(data) => log::info!("{}", data.text),
            PrintTextBlock(data) => log::info!("{data:#?}"),
            CompletionData(_) => (),
        }
    }
    Ok(())
}

async fn end_of_day(socket: &mut PacketTransport, password: usize) -> Result<()> {
    let request = packets::EndOfDay { password };
    let mut stream = sequences::EndOfDay::into_stream(&request, socket);
    while let Some(response) = stream.next().await {
        use sequences::EndOfDayResponse::*;
        match response? {
            StatusInformation(data) => log::info!("{:?}", data),
            PrintLine(data) => log::info!("{}", data.text),
            PrintTextBlock(data) => log::info!("{data:#?}"),
            IntermediateStatusInformation(_) | CompletionData(_) => (),
            Abort(data) => bail!("Received Abort: {:?}", data),
        }
    }
    Ok(())
}

async fn read_card(socket: &mut PacketTransport, args: &ReadCardArgs) -> Result<()> {
    let request = packets::ReadCard {
        timeout_sec: args.timeout,
        card_type: Some(args.card_type),
        dialog_control: Some(args.dialog_control),
        tlv: Some(packets::tlv::ReadCard {
            card_reading_control: Some(args.short_card_reading_control),
            card_type: Some(args.allowed_cards),
        }),
    };

    let mut stream = sequences::ReadCard::into_stream(&request, socket);
    while let Some(response) = stream.next().await {
        use sequences::ReadCardResponse::*;
        match response? {
            IntermediateStatusInformation(_) => (),
            Abort(data) => {
                if data.error == zvt::constants::ErrorMessages::AbortViaTimeoutOrAbortKey as u8 {
                    log::info!("No card presented before timeout.");
                } else {
                    bail!("Received Abort: {:?}", data);
                }
            }
            StatusInformation(data) => {
                log::info!("StatusInformation: {:#?}", data);
                // TODO(hrapp): This is taken from internal code, but should really be in the ZVT
                // library.
                // Retrieve the card information. The code is mostly taken
                // from python.
                let tlv = data.tlv.ok_or(zvt::ZVTError::IncompleteData)?;
                if !tlv.subs.is_empty() {
                    let subs = &tlv.subs[0];
                    if subs.application_id.is_some() {
                        log::info!("Bank Card");
                    } else {
                        bail!("Unknown card type")
                    }
                } else if let Some(mut uuid) = tlv.uuid {
                    uuid = uuid.to_uppercase();
                    if uuid.len() > 14 {
                        uuid = uuid[uuid.len() - 14..].to_string();
                        uuid = uuid.strip_prefix("000000").unwrap_or(&uuid).to_string();
                    }
                    log::info!("RFID: {}", uuid);
                } else {
                    bail!(zvt::ZVTError::IncompleteData)
                }
            }
        }
    }
    Ok(())
}

fn prep_bmp_data(
    bmp_prefix: Option<String>,
    bmp_data: Option<String>,
) -> Result<Option<packets::tlv::PreAuthData>> {
    match (bmp_prefix, bmp_data) {
        (Some(bmp_prefix), Some(bmp_data)) => Ok(Some(packets::tlv::PreAuthData {
            bmp_data: Some(packets::tlv::Bmp60 {
                bmp_prefix,
                bmp_data,
            }),
        })),
        (None, None) => Ok(None),
        _ => bail!("Either none or both of bmp_data and bmp_prefix must be given."),
    }
}

async fn reservation(socket: &mut PacketTransport, args: ReservationArgs) -> Result<()> {
    let tlv = prep_bmp_data(args.bmp_prefix, args.bmp_data)?;
    let request = packets::Reservation {
        currency: Some(args.currency_code),
        amount: Some(args.amount),
        payment_type: Some(args.payment_type),
        track_2_data: args.track_2_data,
        tlv,
        ..packets::Reservation::default()
    };

    let mut stream = sequences::Reservation::into_stream(&request, socket);
    use sequences::AuthorizationResponse::*;
    while let Some(response) = stream.next().await {
        match response? {
            IntermediateStatusInformation(_) | CompletionData(_) => (),
            PrintLine(data) => log::info!("{}", data.text),
            PrintTextBlock(data) => log::info!("{data:#?}"),
            Abort(data) => bail!("Received Abort: {:?}", data),
            StatusInformation(data) => log::info!("StatusInformation: {:#?}", data),
        }
    }
    Ok(())
}

async fn partial_reversal(socket: &mut PacketTransport, args: PartialReversalArgs) -> Result<()> {
    let tlv = prep_bmp_data(args.bmp_prefix, args.bmp_data)?;

    let request = packets::PartialReversal {
        receipt_no: Some(args.receipt),
        amount: Some(args.amount),
        payment_type: Some(args.payment_type),
        currency: Some(args.currency_code),
        tlv,
    };

    let mut stream = sequences::PartialReversal::into_stream(&request, socket);
    use sequences::PartialReversalResponse::*;
    while let Some(response) = stream.next().await {
        match response? {
            IntermediateStatusInformation(_) | CompletionData(_) => (),
            PrintLine(data) => log::info!("{}", data.text),
            PrintTextBlock(data) => log::info!("{data:#?}"),
            PartialReversalAbort(data) => bail!("Received Abort: {:?}", data),
            StatusInformation(data) => log::info!("StatusInformation: {:#?}", data),
        }
    }
    Ok(())
}

async fn change_host_config(
    socket: &mut PacketTransport,
    password: usize,
    args: ChangeHostConfigurationArgs,
) -> Result<()> {
    let request = feig::packets::ChangeConfiguration {
        tlv: feig::packets::tlv::ChangeConfiguration {
            system_information: feig::packets::tlv::SystemInformation {
                password,
                host_configuration_data: Some(feig::packets::tlv::HostConfigurationData {
                    ip: args.ip.into(),
                    port: args.port,
                    config_byte: args.configuration_byte,
                }),
            },
        },
    };

    let mut stream = feig::sequences::ChangeHostConfiguration::into_stream(&request, socket);
    use feig::sequences::ChangeHostConfigurationResponse::*;
    while let Some(response) = stream.next().await {
        match response? {
            CompletionData(_) => (),
            Abort(data) => bail!("Received Abort: {:?}", data),
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    let args: Args = argh::from_env();

    let mut socket = {
        let source = TcpStream::connect(args.ip).await?;
        PacketTransport { source }
    };

    match args.command {
        SubCommands::Status(a) => {
            status(&mut socket, args.password, a.r#type, a.service_byte).await?
        }
        SubCommands::FactoryReset(_) => factory_reset(&mut socket, args.password).await?,
        SubCommands::Registration(a) => registration(&mut socket, args.password, &a).await?,
        SubCommands::SetTerminalId(a) => set_terminal_id(&mut socket, args.password, &a).await?,
        SubCommands::Initialization(_) => initialization(&mut socket, args.password).await?,
        SubCommands::Diagnosis(a) => diagnosis(&mut socket, &a).await?,
        SubCommands::PrintSystemConfiguration(_) => print_system_diagnosis(&mut socket).await?,
        SubCommands::EndOfDay(_) => end_of_day(&mut socket, args.password).await?,
        SubCommands::ReadCard(a) => read_card(&mut socket, &a).await?,
        SubCommands::Reservation(a) => reservation(&mut socket, a).await?,
        SubCommands::PartialReversal(a) => partial_reversal(&mut socket, a).await?,
        SubCommands::ChangeHostConfiguration(a) => {
            change_host_config(&mut socket, args.password, a).await?
        }
    }

    Ok(())
}
