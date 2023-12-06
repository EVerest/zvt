use clap::Parser;
use env_logger::{Builder, Env};
use log::info;
use std::io::Write;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use zvt::{packets, sequences, sequences::Sequence};

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

use std::fs::File;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct LoggingTcpStream {
    stream: TcpStream,
    file: File,
}

impl LoggingTcpStream {
    pub async fn new(stream: TcpStream, file_path: &str) -> io::Result<Self> {
        let file = File::create(file_path)?;
        Ok(LoggingTcpStream { stream, file })
    }
}

impl AsyncRead for LoggingTcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let self_mut = self.get_mut();
        let read = buf.filled().len();
        let poll = Pin::new(&mut self_mut.stream).poll_read(cx, buf);
        if let Poll::Ready(Ok(_)) = poll {
            let buf = &buf.filled()[read..];
            self_mut.file.write_all(buf)?;
        }
        poll
    }
}

impl AsyncWrite for LoggingTcpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let self_mut = self.get_mut();
        let poll = Pin::new(&mut self_mut.stream).poll_write(cx, buf);
        if let Poll::Ready(Ok(size)) = poll {
            // If write is successful, write to file
            let data = &buf[..size];
            self_mut.file.write_all(data)?;
        }
        poll
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logger();
    let args = Args::parse();

    info!("Using the args {:?}", args);
    let mut socket = LoggingTcpStream {
        stream: TcpStream::connect(args.ip).await?,
        file: File::create("/tmp/dump.txt")?,
    };

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
