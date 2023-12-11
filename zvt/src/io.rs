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

pub struct PacketTransport<Source> {
    pub source: Source,
}

impl<S> PacketTransport<S>
where
    S: AsyncReadExt + Unpin + Send,
{
    /// Reads an ADPU packet from the PT.
    pub async fn read_packet<T>(&mut self) -> Result<T>
    where
        T: ZvtParser + Send,
    {
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

        // NOCOM(#sirver): add pretty hex here
        log::debug!("Read {:?}", buf);

        Ok(T::zvt_parse(&buf)?)
    }
}

impl<S> PacketTransport<S>
where
    S: AsyncWriteExt + Unpin + Send,
{
    /// Writes an ADPU packet to the PT.
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

impl<S> PacketTransport<S>
where
    S: AsyncWriteExt + AsyncReadExt + Unpin + Send,
{
    /// Reads an ADPU packet from the PT and send an [packets::Ack].
    pub async fn read_packet_with_ack<'a, T>(&mut self) -> Result<T>
    where
        T: ZvtParser + Send,
    {
        let packet = self.read_packet::<T>().await?;
        self.write_packet(&packets::Ack {}).await?;

        Ok(packet)
    }

    /// Writes an ADPU packet to the PT and awaits its [packets::Ack].
    pub async fn write_packet_with_ack<'a, T>(&mut self, msg: &T) -> Result<()>
    where
        T: ZvtSerializer + Sync + Send,
        encoding::Default: encoding::Encoding<T>,
    {
        self.write_packet(msg).await?;

        let _ = self.read_packet::<Ack>().await?;

        Ok(())
    }
}
