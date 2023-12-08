use async_trait::async_trait;

/// Trait which will log the byte payload after receiving.
#[async_trait]
pub trait AsyncReadExt: tokio::io::AsyncReadExt + Unpin + Send {
    async fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> tokio::io::Result<usize> {
        let res = <Self as tokio::io::AsyncReadExt>::read_exact(self, buf).await;
        log::debug!("The length is {} and the result is {:?}", buf.len(), res);
        log::debug!("Read {:?}", buf);
        res
    }
}

impl<R: tokio::io::AsyncReadExt + ?Sized + Unpin + Send> AsyncReadExt for R {}

/// Trait which will log the byte payload before transmitting.
#[async_trait]
pub trait AsyncWriteExt: tokio::io::AsyncWriteExt + Unpin + Send {
    async fn write_all<'a>(&'a mut self, src: &'a [u8]) -> tokio::io::Result<()> {
        log::debug!("Write {src:?}");
        <Self as tokio::io::AsyncWriteExt>::write_all(self, src).await
    }
}

impl<W: tokio::io::AsyncWriteExt + ?Sized + Unpin + Send> AsyncWriteExt for W {}
