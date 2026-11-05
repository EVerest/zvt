//! Reconnecting TcpStream.
//!
//! We observe that we need to reset the connection on errors resulting either
//! from transmission or parsing. The module implements a [TcpStream], which
//! it self contains all information to reset a tcp connection and a wrapper
//! around [sequence::Sequence] - which is responsible for restarting a
//! dropped connection and terminating it on errors.s
use crate::config::{Config, TerminalAddress};
use crate::feig::Error as FeigError;
use crate::mdns::Backoff;
use anyhow::{anyhow, bail, Result};
use async_stream::stream;
use futures::stream::BoxStream;
use log::{debug, info, warn};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
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
        /// Tries to open a new [InnerTcpStream] to the PT under the
        /// `ip_address` and performs basic registration to the terminal.
        ///
        /// We mock this function in the test configuration.
        #[cfg_attr(test, allow(dead_code))]
        pub async fn connect(
            config: &Config,
            ip_address: Ipv4Addr,
        ) -> Result<io::PacketTransport<InnerTcpStream>> {
            // Configuration byte.
            pub const CONFIG_BYTE: u8 = 0xde;
            let address = SocketAddr::V4(SocketAddrV4::new(ip_address, 22000));
            #[cfg(not(test))]
            let source: InnerTcpStream = {
                let socket = tokio::net::TcpSocket::new_v4()?;
                socket.set_linger(Some(Duration::from_secs(0)))?;
                let socket = socket.connect(address).await?;
                // Keepalive must be configured *after* the connect otherwise
                // it might be ignored by the kernel.
                let keepalive = socket2::TcpKeepalive::new()
                    .with_time(Duration::from_secs(10))
                    .with_interval(Duration::from_secs(5));
                socket2::SockRef::from(&socket).set_tcp_keepalive(&keepalive)?;
                socket
            };
            #[cfg(test)]
            let source = InnerTcpStream::connect(address).await?;

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
                        bail!(FeigError::IncorrectDeviceId{expected: expected_serial, received: actual_serial})
                    },
                    feig::sequences::GetSystemInfoResponse::Abort(packet) => bail!(zvt::ZVTError::Aborted(packet.error))
                }
        }

        /// Looks up the ip-address of the PT with the `feig_serial` via mDNS.
        ///
        /// We mock this function in the test configuration.
        #[cfg_attr(test, allow(dead_code))]
        pub async fn lookup(feig_serial: &str) -> Result<Ipv4Addr> {
            crate::mdns::resolve(feig_serial).await
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
    /// The address we connect to: the fixed one, or the one found via mDNS.
    ip_address: Option<Ipv4Addr>,
    /// Set if the last connect failed. With [TerminalAddress::Mdns] the
    /// terminal may have received a new address, so we look it up again.
    connect_failed: bool,
    /// Throttles the mDNS lookups while they fail.
    mdns_backoff: Backoff,
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

        let ip_address = match config.address {
            TerminalAddress::Ip(ip_address) => Some(ip_address),
            TerminalAddress::Mdns => None,
        };
        Ok(Self {
            config,
            ip_address,
            ..Self::default()
        })
    }

    /// Returns the config used to construct the stream.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Connects to the terminal under [Self::ip_address].
    async fn connect(&mut self) -> Result<io::PacketTransport<InnerTcpStream>> {
        if self.config.address == TerminalAddress::Mdns {
            self.mdns_lookup().await;
        }
        let ip_address = self.ip_address.ok_or_else(|| {
            anyhow!(
                "No ip-address for the feig_serial {}",
                self.config.feig_serial
            )
        })?;
        let res = inner::connect(&self.config, ip_address).await;
        self.connect_failed = res.is_err();
        res
    }

    /// Looks up the terminal via mDNS if we have no address yet or the last
    /// connect failed - but not more often than the backoff allows. Until a
    /// lookup succeeds, we keep the last address: the terminal may come back
    /// under it.
    async fn mdns_lookup(&mut self) {
        let needs_lookup = self.ip_address.is_none() || self.connect_failed;
        if !needs_lookup || !self.mdns_backoff.is_ready() {
            return;
        }
        let feig_serial = &self.config.feig_serial;
        match inner::lookup(feig_serial).await {
            Ok(ip_address) => {
                info!("Found the feig_serial {feig_serial} at {ip_address}");
                self.mdns_backoff.record_success();
                self.ip_address = Some(ip_address);
            }
            Err(err) => {
                warn!("Failed to look up the feig_serial {feig_serial}: {err:#}");
                self.mdns_backoff.record_failure();
            }
        }
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
        let max_retry_attempts = src.config.feig_config.max_retry_attempts;
        let repeater = futures::stream::repeat(())
            .throttle(std::time::Duration::from_secs(2))
            .take(1 + max_retry_attempts);

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
                    match src.connect().await {
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

    /// Serializes the tests which use the static mocks of [inner].
    async fn lock_mocks() -> tokio::sync::MutexGuard<'static, ()> {
        static MOCKS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
        MOCKS.lock().await
    }

    fn get_config() -> Config {
        Config {
            feig_config: FeigConfig {
                currency: 0,
                read_card_timeout: 15,
                password: 123456,
                end_of_day_max_interval: 1000,
                max_retry_attempts: 20,
            },
            ..Config::default()
        }
    }

    #[tokio::test]
    async fn test_running_once() {
        // Test the connection failure at beginning.
        let _lock = lock_mocks().await;
        let mut socket = TcpStream::new(get_config()).unwrap();

        let ctx = inner::connect_context();
        ctx.expect()
            .times(1)
            .returning(|_, _| bail!("not connected"));

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
        ctx.expect().returning(|_, _| {
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
        ctx.expect().times(attempts).returning(|_, _| {
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
        ctx.expect().returning(|_, _| {
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
    const IP_1: Ipv4Addr = Ipv4Addr::new(192, 168, 23, 50);
    const IP_2: Ipv4Addr = Ipv4Addr::new(192, 168, 23, 60);

    fn mdns_socket() -> TcpStream {
        TcpStream::new(Config {
            feig_serial: "serial".to_string(),
            address: TerminalAddress::Mdns,
            ..get_config()
        })
        .unwrap()
    }

    fn connected() -> Result<io::PacketTransport<InnerTcpStream>> {
        Ok(io::PacketTransport {
            source: InnerTcpStream {},
        })
    }

    #[tokio::test]
    async fn test_connect_uses_the_fixed_ip_without_lookup() {
        let _lock = lock_mocks().await;
        let lookup = inner::lookup_context();
        lookup.expect().never();
        let connect = inner::connect_context();
        connect
            .expect()
            .withf(|_, ip| *ip == IP_1)
            .times(1)
            .returning(|_, _| connected());
        let mut socket = TcpStream::new(Config {
            address: TerminalAddress::Ip(IP_1),
            ..get_config()
        })
        .unwrap();

        let res = socket.connect().await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_connect_reuses_the_found_address_while_connecting_works() {
        let _lock = lock_mocks().await;
        let lookup = inner::lookup_context();
        lookup
            .expect()
            .withf(|feig_serial| feig_serial == "serial")
            .times(1)
            .returning(|_| Ok(IP_1));
        let connect = inner::connect_context();
        connect
            .expect()
            .withf(|_, ip| *ip == IP_1)
            .times(2)
            .returning(|_, _| connected());
        let mut socket = mdns_socket();
        socket.connect().await.unwrap();

        let res = socket.connect().await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_connect_looks_up_the_address_again_after_a_failed_connect() {
        let _lock = lock_mocks().await;
        let lookup = inner::lookup_context();
        let mut ips = vec![IP_2, IP_1];
        lookup
            .expect()
            .times(2)
            .returning(move |_| Ok(ips.pop().unwrap()));
        let connect = inner::connect_context();
        connect
            .expect()
            .withf(|_, ip| *ip == IP_1)
            .times(1)
            .returning(|_, _| bail!("unreachable"));
        connect
            .expect()
            .withf(|_, ip| *ip == IP_2)
            .times(1)
            .returning(|_, _| connected());
        let mut socket = mdns_socket();
        assert!(socket.connect().await.is_err());

        let res = socket.connect().await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_connect_keeps_the_last_address_while_the_lookup_fails() {
        let _lock = lock_mocks().await;
        let lookup = inner::lookup_context();
        let mut results = vec![Err(anyhow!("no answer")), Ok(IP_1)];
        lookup
            .expect()
            .times(2)
            .returning(move |_| results.pop().unwrap());
        let connect = inner::connect_context();
        let mut results = vec![connected(), Err(anyhow!("unreachable"))];
        connect
            .expect()
            .withf(|_, ip| *ip == IP_1)
            .times(2)
            .returning(move |_, _| results.pop().unwrap());
        let mut socket = mdns_socket();
        assert!(socket.connect().await.is_err());

        let res = socket.connect().await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_connect_fails_without_connecting_if_no_address_was_found() {
        let _lock = lock_mocks().await;
        let lookup = inner::lookup_context();
        lookup.expect().times(1).returning(|_| bail!("no answer"));
        let connect = inner::connect_context();
        connect.expect().never();
        let mut socket = mdns_socket();

        let res = socket.connect().await;

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_connect_skips_the_lookup_while_the_backoff_is_not_ready() {
        let _lock = lock_mocks().await;
        let lookup = inner::lookup_context();
        lookup.expect().times(1).returning(|_| bail!("no answer"));
        let connect = inner::connect_context();
        connect.expect().never();
        let mut socket = mdns_socket();
        assert!(socket.connect().await.is_err());

        let res = socket.connect().await;

        assert!(res.is_err());
    }
}
