use crate::packets;
use crate::ZvtEnum;
use crate::ZvtParser;
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zvt_builder::encoding;
use zvt_builder::ZvtSerializer;

#[derive(ZvtEnum)]
pub enum Ack {
    Ack(packets::Ack),
}

pub struct PacketWriter<Source> {
    pub source: Source,
}

impl<S> PacketWriter<S>
where
    S: AsyncReadExt + Unpin + Send,
{
    pub async fn read_packet(&mut self) -> Result<Vec<u8>> {
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

impl<S> PacketWriter<S>
where
    S: AsyncWriteExt + Unpin + Send,
{
    pub async fn write_packet<'a, T>(&mut self, msg: &T) -> Result<()>
    where
        T: ZvtSerializer + Sync + Send,
        encoding::Default: encoding::Encoding<T>,
    {
        let bytes = msg.zvt_serialize();
        log::debug!("Write {:?}", bytes);
        self.source
            .write_all(&bytes)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to write {:?}", e))
    }
}

impl<S> PacketWriter<S>
where
    S: AsyncWriteExt + AsyncReadExt + Unpin + Send,
{
    pub async fn write_packet_with_ack<'a, T>(&mut self, msg: &T) -> Result<()>
    where
        T: ZvtSerializer + Sync + Send,
        encoding::Default: encoding::Encoding<T>,
    {
        self.write_packet(msg).await?;

        let bytes = self.read_packet().await?;
        let _ = Ack::zvt_parse(&bytes)?;

        Ok(())
    }
}
