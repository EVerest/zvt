use anyhow::Result;
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct PacketWriter<Source> {
    pub source: Source,
}

#[async_trait]
pub trait AsyncReadPacket {
    async fn read_packet(&mut self) -> Result<Vec<u8>>;
}

#[async_trait]
impl<S> AsyncReadPacket for PacketWriter<S>
where
    S: AsyncReadExt + Unpin + Send,
{
    async fn read_packet(&mut self) -> Result<Vec<u8>> {
        let mut buf = vec![0; 3];
        self.source.read_exact(&mut buf).await?;

        // Get the len.
        let len = if buf[2] == 0xff {
            buf.resize(5, 0);
            self.source.read_exact(&mut buf[3..5]).await?;
            u16::from_le_bytes(buf[3..5].try_into().unwrap()) as usize
        } else {
            buf[2] as usize
        };

        let start = buf.len();
        buf.resize(start + len, 0);
        self.source.read_exact(&mut buf[start..]).await?;

        log::debug!("Read {:?}", buf);

        Ok(buf.to_vec())
    }
}

#[async_trait]
pub trait AsyncWritePacket {
    async fn write_packet<'a>(&mut self, buf: &'a [u8]) -> Result<()>;
}

#[async_trait]
impl<S> AsyncWritePacket for PacketWriter<S>
where
    S: AsyncWriteExt + Unpin + Send,
{
    async fn write_packet<'a>(&mut self, src: &'a [u8]) -> Result<()> {
        log::debug!("Write {:?}", src);
        self.source
            .write_all(src)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to write {:?}", e))
    }
}
