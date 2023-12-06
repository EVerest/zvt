use log::debug;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::Poll;
use tokio::io::{AsyncRead, AsyncWrite};

/// Wrapper around an io interface which will log on successful read and write.
///
/// The logging happens every time the async methods return [Poll::Ready],
/// assuming the runtime will not poll them after this point.
#[pin_project]
pub struct LoggingSource<Source> {
    #[pin]
    pub source: Source,
}

impl<Source> AsyncRead for LoggingSource<Source>
where
    Source: AsyncRead + Send,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let res = self.project().source.poll_read(cx, buf);
        if let Poll::Ready(_) = res {
            debug!("Read the bytes {:?}", buf)
        }

        res
    }
}

impl<Source> AsyncWrite for LoggingSource<Source>
where
    Source: AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::prelude::v1::Result<usize, std::io::Error>> {
        let res = self.project().source.poll_write(cx, buf);
        if let Poll::Ready(_) = res {
            debug!("Write the bytes {:?}", buf);
        }

        res
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::prelude::v1::Result<(), std::io::Error>> {
        // No logging here.
        self.project().source.poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::prelude::v1::Result<(), std::io::Error>> {
        // No logging here.
        self.project().source.poll_shutdown(cx)
    }
}
