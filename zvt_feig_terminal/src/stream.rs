//! Reconnecting TcpStream.
//!
//! We observe that we need to reset the connection on errors resulting either
//! from transmission or parsing. The module implements a [TcpStream], which
//! it self contains all information to reset a tcp connection and a wrapper
//! around [sequence::Sequence] - which is responsible for restarting a
//! dropped connection and terminating it on errors.s
use crate::config::Config;

use anyhow::{bail, Result};
use async_stream::stream;
use futures::stream::BoxStream;
use log::{debug, info, warn};
use std::io::{Error, ErrorKind};
use std::net::SocketAddrV4;
use std::time::Duration;
use tokio_stream::StreamExt;
use zvt::{encoding, feig, io, packets, sequences, sequences::Sequence};

const TIMEOUT: Duration = Duration::from_secs(60);

/// The implementation of our I/O.
///
/// We're using a custom switch very similar to what [mockall_double::double]
/// is doing.
#[cfg(not(test))]
type InnerTcpStream = tokio::net::TcpStream;

/// Mocked I/O for unit tests.
#[cfg(test)]
type InnerTcpStream = test::MockTcpStream;

/// The modules just to use the mocking. Really cumbersome...
mod outer {
    use super::*;
    #[mockall::automock()]
    pub(super) mod inner {
        use super::*;

        /// Reconnection
        ///
        /// Tries to open a new [InnerTcpStream] to the PT defined in the
        /// `config` and performs basic registration to the terminal.
        ///
        /// We mock this function in the test configuration.
        #[cfg_attr(test, allow(dead_code))]
        pub async fn connect(config: &Config) -> Result<io::PacketTransport<InnerTcpStream>> {
            // Configuration byte.
            pub const CONFIG_BYTE: u8 = 0xde;

            let source =
                InnerTcpStream::connect(SocketAddrV4::new(config.ip_address, 22000)).await?;
            let mut socket = io::PacketTransport { source };

            let request = packets::Registration {
                password: config.feig_config.password,
                config_byte: CONFIG_BYTE,
                currency: Some(config.feig_config.currency),
                tlv: None,
            };

            // Register to the terminal.
            let mut stream =
                <sequences::Registration as Sequence>::into_stream(&request, &mut socket);
            while let Some(response) = stream.next().await {
                let completion = response?;
                info!("Registered to the terminal {:?}", completion);
            }
            drop(stream);

            // Verify that we're connected to the right terminal.
            let request = feig::packets::CVendFunctions {
                password: None,
                instr: 1,
            };
            let mut stream =
                <feig::sequences::GetSystemInfo as Sequence>::into_stream(&request, &mut socket);
            let Some(packet) = stream.next().await else {
                bail!(zvt::ZVTError::IncompleteData)
            };
            debug!("Received {packet:?}");
            match packet? {
                    feig::sequences::GetSystemInfoResponse::CVendFunctionsEnhancedSystemInformationCompletion(packet) => {
                        let expected_serial: String = config.feig_serial.to_lowercase();
                        let actual_serial: String = packet.device_id.to_lowercase();
                        if actual_serial == expected_serial {
                            drop(stream);
                            return Ok(socket);
                        }
                        bail!(Error::new(ErrorKind::NotConnected, format!("Wrong device. Expected {}, got {}", expected_serial, actual_serial)))
                    },
                    feig::sequences::GetSystemInfoResponse::Abort(packet) => bail!(zvt::ZVTError::Aborted(packet.error))
                }
        }
    }
}

#[mockall_double::double]
use outer::inner;

/// Stream which can reconnect.
#[derive(Default)]
pub struct TcpStream {
    /// Configuration, needed to regain connection.
    config: Config,
    /// Underlying I/O.
    inner: Option<io::PacketTransport<InnerTcpStream>>,
}

impl TcpStream {
    /// Creates a new [TcpStream].
    ///
    /// # Arguments
    /// * `pole_config` - The configuration from backend.
    pub fn new(mut config: Config) -> Result<Self> {
        // Check the terminal_id
        if config.terminal_id.is_empty() {
            warn!("No terminal-id provided. Fix this if it's production");
            config.terminal_id = "00000000".to_string();
        }

        Ok(Self {
            config,
            inner: None,
        })
    }

    /// Returns the config used to construct the stream.
    pub fn config(&self) -> &Config {
        &self.config
    }
}

/// One of our most important.
///
/// The wrapper around the [sequence::Sequence], which also manages the tcp
/// connection. Before starting the stream, it will setup a connection (if
/// necessary) and tear it down on errors.
pub trait ResetSequence: Sequence
where
    encoding::Default: encoding::Encoding<Self::Input>,
{
    fn into_stream<'a>(
        input: Self::Input,
        src: &'a mut TcpStream,
    ) -> BoxStream<'a, Result<Self::Output>>
    where
        Self: 'a,
        Self::Input: std::fmt::Debug,
        Self::Output: std::fmt::Debug,
    {
        let repeater = futures::stream::repeat(())
            .throttle(std::time::Duration::from_secs(2))
            .take(20);

        Self::into_stream_with_retry(input, src, repeater, TIMEOUT)
    }

    fn into_stream_with_retry<'a, RetryStream>(
        input: Self::Input,
        src: &'a mut TcpStream,
        retry: RetryStream,
        timeout: std::time::Duration,
    ) -> BoxStream<'a, Result<Self::Output>>
    where
        Self: 'a,
        Self::Input: std::fmt::Debug,
        Self::Output: std::fmt::Debug,
        RetryStream: futures::Stream<Item = ()> + Send + 'a,
    {
        debug!("Sending packet {input:?}");
        let s = stream! {

            tokio::pin!(retry);
            while let Some(()) = retry.next().await {
                // We don't have a valid connection - we must reconnect.
                if src.inner.is_none() {
                    warn!("Reconnecting");
                    match inner::connect(&src.config).await {
                        Ok(inner) => src.inner = Some(inner),
                        Err(err) => {
                            warn!("Failed to reconnect: {err:?}");
                            yield Err(err);
                            continue;
                        }
                    }
                }

                // Start the underlying stream.
                let mut stream =
                    <Self as Sequence>::into_stream(&input, src.inner.as_mut().unwrap());
                let mut is_err = false;
                // We are awaiting packets with a timeout. In case of a timeout
                // we convert the error into None, to break out of the loop. The
                // timeout is needed since we may not finish the preceding
                // sequence and hang.
                while let Some(packet) = match tokio::time::timeout(timeout, stream.next()).await {
                    Ok(packet) => packet,
                    Err(_) => {
                        warn!("Timeout");
                        is_err = true;
                        None
                    }
                } {
                    debug!("Received packet {packet:?}");
                    is_err = packet.is_err();
                    yield packet;
                    if is_err {
                        break;
                    }
                }
                drop(stream);
                if is_err {
                    debug!("Dropping the connection");
                    src.inner = None;
                    continue;
                }
                return;
            }
        };

        Box::pin(s)
    }
}

impl<St: ?Sized> ResetSequence for St
where
    St: Sequence,
    encoding::Default: encoding::Encoding<<St as Sequence>::Input>,
{
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::FeigConfig;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
    use tokio::net::ToSocketAddrs;

    /// Mocked TcpStream.
    pub struct MockTcpStream;

    impl MockTcpStream {
        /// Interface for connecting - the same signature as
        /// [tokio::net::TcpStream::connect].
        ///
        /// The function is never called but just there to compile the code.
        #[allow(dead_code)]
        pub(super) async fn connect<A>(_: A) -> std::io::Result<Self>
        where
            A: ToSocketAddrs,
        {
            unimplemented!("just a mock")
        }
    }

    /// Impl for AsyncRead - which is required for [tokio::AsyncReadExt], which
    /// is used in the [sequence::Sequence] signature.
    impl AsyncRead for MockTcpStream {
        fn poll_read(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
            _: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            unimplemented!("just a mock")
        }
    }

    /// Impl for AsyncRead - which is required for [tokio::AsyncWriteExt], which
    /// is used in the [sequence::Sequence] signature.s
    impl AsyncWrite for MockTcpStream {
        fn poll_write(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
            _: &[u8],
        ) -> Poll<std::result::Result<usize, std::io::Error>> {
            unimplemented!("just a mock")
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), std::io::Error>> {
            unimplemented!("just a mock")
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), std::io::Error>> {
            unimplemented!("just a mock")
        }
    }

    /// Fake sequence which always fails.
    struct FailSequence {}

    impl sequences::Sequence for FailSequence {
        type Input = feig::packets::CVendFunctions;
        type Output = feig::sequences::GetSystemInfoResponse;

        /// Fake Stream
        ///
        /// Returns just an error, without calling any I/O functions.s
        fn into_stream<'a, Source>(
            _: &'a Self::Input,
            _: &'a mut io::PacketTransport<Source>,
        ) -> std::pin::Pin<Box<dyn futures::Stream<Item = Result<Self::Output>> + Send + 'a>>
        where
            Source: AsyncReadExt + AsyncWriteExt + Unpin + Send,
            Self: 'a,
        {
            let res = vec![Err(zvt::ZVTError::NonImplemented.into())];
            Box::pin(futures::stream::iter(res))
        }
    }

    /// Fake sequence which always succeeds.
    struct SuccessSequence {}

    impl sequences::Sequence for SuccessSequence {
        type Input = feig::packets::CVendFunctions;
        type Output = feig::sequences::GetSystemInfoResponse;

        /// Fake Stream
        ///
        /// Returns just a successful message without calling any I/O.
        fn into_stream<'a, Source>(
            _: &'a Self::Input,
            _: &'a mut io::PacketTransport<Source>,
        ) -> std::pin::Pin<Box<dyn futures::Stream<Item = Result<Self::Output>> + Send + 'a>>
        where
            Source: AsyncReadExt + AsyncWriteExt + Unpin + Send,
            Self: 'a,
        {
            let res = vec![Ok(Self::Output::Abort(zvt::packets::Abort { error: 0 }))];
            Box::pin(futures::stream::iter(res))
        }
    }

    /// Fake sequence which takes some time to return.
    struct SlowSequence {}

    impl sequences::Sequence for SlowSequence {
        type Input = feig::packets::CVendFunctions;
        type Output = feig::sequences::GetSystemInfoResponse;
        /// Slow stream
        ///
        /// Returns a message and waits for a long time before the next one.
        fn into_stream<'a, Source>(
            _: &'a Self::Input,
            _: &'a mut io::PacketTransport<Source>,
        ) -> std::pin::Pin<Box<dyn futures::Stream<Item = Result<Self::Output>> + Send + 'a>>
        where
            Source: AsyncReadExt + AsyncWriteExt + Unpin + Send,
            Self: 'a,
        {
            Box::pin(
                futures::stream::repeat_with(|| {
                    Ok(Self::Output::Abort(zvt::packets::Abort { error: 0 }))
                })
                .throttle(Duration::from_secs(60)),
            )
        }
    }

    fn get_config() -> Config {
        Config {
            feig_config: FeigConfig {
                currency: 0,
                pre_authorization_amount: 0,
                read_card_timeout: 15,
                password: 123456,
            },
            ..Config::default()
        }
    }

    #[tokio::test]
    async fn test_running_once() {
        // Test the connection failure at beginning.
        let mut socket = TcpStream {
            config: get_config(),
            inner: None,
        };

        let ctx = inner::connect_context();
        ctx.expect().times(1).returning(|_| bail!("not connected"));

        let repeater = futures::stream::repeat(()).take(1);
        let request = feig::packets::CVendFunctions {
            password: None,
            instr: 0,
        };
        let mut stream =
            FailSequence::into_stream_with_retry(request, &mut socket, repeater, TIMEOUT);

        assert!(stream.next().await.unwrap().is_err());
        assert!(stream.next().await.is_none());
        ctx.checkpoint();
        drop(stream);
        // The inner should still be none.
        assert!(socket.inner.is_none());

        // Now pretend that the connection was successful.
        ctx.expect().returning(|_| {
            Ok(io::PacketTransport {
                source: InnerTcpStream {},
            })
        });
        let repeater = futures::stream::repeat(()).take(1);
        let request = feig::packets::CVendFunctions {
            password: None,
            instr: 0,
        };
        let mut stream =
            FailSequence::into_stream_with_retry(request, &mut socket, repeater, TIMEOUT);

        // Verify the results.
        assert!(stream.next().await.unwrap().is_err());
        assert!(stream.next().await.is_none());
        drop(stream);
        // The inner connection shall be resetted.
        assert!(socket.inner.is_none());

        // Now call the successful sequence.
        let repeater = futures::stream::repeat(()).take(1);
        let request = feig::packets::CVendFunctions {
            password: None,
            instr: 0,
        };
        let mut stream =
            SuccessSequence::into_stream_with_retry(request, &mut socket, repeater, TIMEOUT);
        assert!(stream.next().await.unwrap().is_ok());
        assert!(stream.next().await.is_none());
        drop(stream);
        assert!(socket.inner.is_some());
        ctx.checkpoint();

        // Now try with retrying the fail sequence.
        socket.inner = None;
        let attempts = 2;
        ctx.expect().times(attempts).returning(|_| {
            Ok(io::PacketTransport {
                source: InnerTcpStream {},
            })
        });

        let repeater = futures::stream::repeat(()).take(attempts);
        let request = feig::packets::CVendFunctions {
            password: None,
            instr: 0,
        };
        let mut stream =
            FailSequence::into_stream_with_retry(request, &mut socket, repeater, TIMEOUT);

        assert!(stream.next().await.unwrap().is_err());
        assert!(stream.next().await.unwrap().is_err());
        assert!(stream.next().await.is_none());
        ctx.checkpoint();
        drop(stream);
        // The inner should still be none.
        assert!(socket.inner.is_none());

        // Now try with a timeout.
        ctx.expect().returning(|_| {
            Ok(io::PacketTransport {
                source: InnerTcpStream {},
            })
        });
        let repeater = futures::stream::repeat(()).take(1);
        let request = feig::packets::CVendFunctions {
            password: None,
            instr: 0,
        };
        let mut stream = SlowSequence::into_stream_with_retry(
            request,
            &mut socket,
            repeater,
            Duration::from_millis(10),
        );
        assert!(stream.next().await.unwrap().is_ok());
        assert!(stream.next().await.is_none());
        drop(stream);
        // The inner should still be none.
        assert!(socket.inner.is_none());
    }
}
