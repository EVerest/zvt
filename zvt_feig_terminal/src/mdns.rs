//! Looks up the Feig terminal via mDNS.
//!
//! The terminal announces itself as `cvend-<feig_serial>.local`. We send a
//! one-shot query to the mDNS group from an ephemeral port, so the terminal
//! answers directly to us - see
//! [RFC 6762, section 6.7](https://www.rfc-editor.org/rfc/rfc6762#section-6.7).
//!
//! The answer comes from the terminal's own address, but hickory's UDP client
//! drops answers which don't come from the queried address. The
//! [MdnsUdpSocket] works around that. We can't use hickory's multicast client
//! instead: in its one-shot mode it spins in a busy loop and never times out
//! (hickory-proto 0.25.2).
use anyhow::Result;
use hickory_client::client::{Client, ClientHandle};
use hickory_client::proto::rr::{DNSClass, Name, RData, RecordType};
use hickory_client::proto::runtime::{RuntimeProvider, TokioRuntimeProvider, TokioTime};
use hickory_client::proto::udp::{DnsUdpSocket, UdpClientStream};
use log::{debug, info, warn};
use std::future::Future;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::pin::Pin;
use std::str::FromStr;
use std::task::{ready, Context, Poll};
use std::time::{Duration, Instant};

/// Address for the mDns as defined [here](https://www.rfc-editor.org/rfc/rfc6762).
const MDNS_IPV4: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 251)), 5353);

/// How long we wait for the answer to one query. Responders on the local link
/// answer within a fraction of a second - no answer by then means a lost
/// datagram.
const QUERY_TIMEOUT: Duration = Duration::from_secs(1);

/// How often we send the query before the lookup fails.
const QUERY_ATTEMPTS: usize = 3;

/// The minimum time between two lookups while they fail.
const MIN_BACKOFF: Duration = Duration::from_secs(2);

/// The maximum time between two lookups while they fail. It bounds how long
/// we keep using a stale ip-address once the terminal is back under a new
/// one, and limits the lookups while the terminal is gone. The value itself is
/// arbitrary.
const MAX_BACKOFF: Duration = Duration::from_secs(60);

/// UDP socket which reports every answer as coming from the name server.
struct MdnsUdpSocket {
    socket: tokio::net::UdpSocket,
    name_server: SocketAddr,
}

impl DnsUdpSocket for MdnsUdpSocket {
    type Time = TokioTime;

    fn poll_recv_from(
        &self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<(usize, SocketAddr)>> {
        // Receive, don't peek: hickory polls until it gets an answer it
        // accepts, and a peeked datagram would come back again and again and
        // hide every datagram behind it.
        let (len, address) = ready!(DnsUdpSocket::poll_recv_from(&self.socket, cx, buf))?;
        debug!("Received an answer from {address:?}");
        Poll::Ready(Ok((len, self.name_server)))
    }

    fn poll_send_to(
        &self,
        cx: &mut Context<'_>,
        buf: &[u8],
        target: SocketAddr,
    ) -> Poll<io::Result<usize>> {
        DnsUdpSocket::poll_send_to(&self.socket, cx, buf, target)
    }
}

/// The tokio runtime of hickory, but with the [MdnsUdpSocket].
#[derive(Clone, Default)]
struct MdnsRuntimeProvider(TokioRuntimeProvider);

impl RuntimeProvider for MdnsRuntimeProvider {
    type Handle = <TokioRuntimeProvider as RuntimeProvider>::Handle;
    type Timer = TokioTime;
    type Udp = MdnsUdpSocket;
    type Tcp = <TokioRuntimeProvider as RuntimeProvider>::Tcp;

    fn create_handle(&self) -> Self::Handle {
        self.0.create_handle()
    }

    fn connect_tcp(
        &self,
        server_addr: SocketAddr,
        bind_addr: Option<SocketAddr>,
        timeout: Option<Duration>,
    ) -> Pin<Box<dyn Send + Future<Output = io::Result<Self::Tcp>>>> {
        self.0.connect_tcp(server_addr, bind_addr, timeout)
    }

    fn bind_udp(
        &self,
        local_addr: SocketAddr,
        server_addr: SocketAddr,
    ) -> Pin<Box<dyn Send + Future<Output = io::Result<Self::Udp>>>> {
        // The stock provider only binds and doesn't connect the socket - we
        // don't know the address which will answer.
        let socket = self.0.bind_udp(local_addr, server_addr);
        Box::pin(async move {
            Ok(MdnsUdpSocket {
                socket: socket.await?,
                name_server: server_addr,
            })
        })
    }
}

/// Returns the ip-address of the Feig terminal with the given serial.
pub(crate) async fn resolve(feig_serial: &str) -> Result<Ipv4Addr> {
    let res = lookup(
        MDNS_IPV4,
        &format!("cvend-{feig_serial}.local"),
        QUERY_ATTEMPTS,
        QUERY_TIMEOUT,
    )
    .await;
    match &res {
        Ok(ip_address) => info!("Found the feig_serial {feig_serial} at {ip_address}"),
        Err(err) => warn!("Failed to look up the feig_serial {feig_serial}: {err:#}"),
    }
    res
}

/// Queries the A record of `name` at the `name_server` - up to `attempts`
/// times, as long as the query fails.
async fn lookup(
    name_server: SocketAddr,
    name: &str,
    attempts: usize,
    timeout: Duration,
) -> Result<Ipv4Addr> {
    let mut result = query_a(name_server, name, timeout).await;
    for _ in 1..attempts {
        let Err(err) = &result else {
            break;
        };
        debug!("Failed to look up {name}: {err:#}. Retrying");
        result = query_a(name_server, name, timeout).await;
    }
    result
}

/// Queries the A record of `name` at the `name_server` once and returns the
/// first ip-address in the response.
async fn query_a(name_server: SocketAddr, name: &str, timeout: Duration) -> Result<Ipv4Addr> {
    let stream = UdpClientStream::builder(name_server, MdnsRuntimeProvider::default())
        .with_timeout(Some(timeout))
        .with_os_port_selection(true)
        .build();
    let (mut client, bg) = Client::connect(stream).await?;
    tokio::spawn(bg);

    let name = Name::from_str(name)?;
    let response = client
        .query(name.clone(), DNSClass::IN, RecordType::A)
        .await?;

    response
        .answers()
        .iter()
        .find_map(|answer| match answer.data() {
            RData::A(ip) => Some(ip.0),
            _ => None,
        })
        .ok_or_else(|| anyhow::anyhow!("No address for {name}"))
}

/// Throttles the lookups while they fail: the time between two lookups
/// doubles from [MIN_BACKOFF] up to [MAX_BACKOFF], and a successful lookup
/// resets it.
#[derive(Debug)]
pub(crate) struct Backoff {
    next: Instant,
    delay: Duration,
}

impl Default for Backoff {
    fn default() -> Self {
        Self {
            next: Instant::now(),
            delay: Duration::ZERO,
        }
    }
}

impl Backoff {
    pub(crate) fn is_ready(&self) -> bool {
        Instant::now() >= self.next
    }

    pub(crate) fn record_failure(&mut self) {
        self.delay = (self.delay * 2).clamp(MIN_BACKOFF, MAX_BACKOFF);
        self.next = Instant::now() + self.delay;
    }

    pub(crate) fn record_success(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hickory_client::proto::op::{Message, MessageType};
    use hickory_client::proto::rr::rdata::A;
    use hickory_client::proto::rr::Record;

    const IP: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 23);
    const NAME: &str = "cvend-1234.local";

    /// Fake name server which answers an A query with `ip` - but sends the
    /// answer from a second socket, like the terminal does. It ignores the
    /// first `ignored_queries` queries, like lost datagrams. With `junk_first`
    /// a datagram which is not a dns message precedes the answer.
    async fn spawn_responder(ignored_queries: usize, junk_first: bool) -> SocketAddr {
        let server = tokio::net::UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let replier = tokio::net::UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let name_server = server.local_addr().unwrap();
        tokio::spawn(async move {
            let mut buf = [0u8; 512];
            for _ in 0..ignored_queries {
                server.recv_from(&mut buf).await.unwrap();
            }
            let (len, client) = server.recv_from(&mut buf).await.unwrap();
            let request = Message::from_vec(&buf[..len]).unwrap();
            let mut response = Message::new();
            response
                .set_id(request.id())
                .set_message_type(MessageType::Response);
            for query in request.queries() {
                response.add_query(query.clone());
                response.add_answer(Record::from_rdata(
                    query.name().clone(),
                    60,
                    RData::A(A(IP)),
                ));
            }
            if junk_first {
                replier.send_to(b"not a dns message", client).await.unwrap();
            }
            replier
                .send_to(&response.to_vec().unwrap(), client)
                .await
                .unwrap();
        });
        name_server
    }

    #[tokio::test]
    async fn test_query_a_accepts_answer_from_another_address_than_the_name_server() {
        let name_server = spawn_responder(0, false).await;

        let ip = query_a(name_server, NAME, Duration::from_secs(5)).await;

        assert_eq!(ip.unwrap(), IP);
    }

    /// Guards against peeking instead of receiving: the junk datagram must be
    /// consumed so that the answer behind it can be read.
    #[tokio::test]
    async fn test_query_a_skips_junk_datagram_before_the_answer() {
        let name_server = spawn_responder(0, true).await;

        let ip = query_a(name_server, NAME, Duration::from_secs(5)).await;

        assert_eq!(ip.unwrap(), IP);
    }

    #[tokio::test]
    async fn test_lookup_sends_the_query_again_after_a_lost_answer() {
        let name_server = spawn_responder(1, false).await;

        let ip = lookup(name_server, NAME, 2, Duration::from_millis(200)).await;

        assert_eq!(ip.unwrap(), IP);
    }

    #[tokio::test]
    async fn test_lookup_fails_after_all_attempts_without_answer() {
        let silent = tokio::net::UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let name_server = silent.local_addr().unwrap();

        let ip = lookup(name_server, NAME, 2, Duration::from_millis(100)).await;

        assert!(ip.is_err());
    }

    #[test]
    fn test_backoff_doubles_the_delay_up_to_the_maximum() {
        let mut backoff = Backoff::default();

        let delays: Vec<_> = (0..7)
            .map(|_| {
                backoff.record_failure();
                backoff.delay.as_secs()
            })
            .collect();

        assert_eq!(delays, [2, 4, 8, 16, 32, 60, 60]);
        assert!(!backoff.is_ready());
    }

    #[test]
    fn test_backoff_is_ready_again_after_success() {
        let mut backoff = Backoff::default();
        backoff.record_failure();

        backoff.record_success();

        assert!(backoff.is_ready());
    }
}
