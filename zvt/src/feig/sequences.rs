use crate::logging::PacketWriter;
use crate::sequences::Sequence;
use crate::{packets, ZvtEnum, ZvtParser};
use anyhow::Result;
use async_stream::try_stream;
use std::boxed::Box;
use std::collections::HashMap;
use std::io::Seek;
use std::io::{Error, ErrorKind};
use std::marker::Unpin;
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_stream::Stream;
use zvt_builder::ZVTError;

pub struct GetSystemInfo;

#[derive(Debug, ZvtEnum)]
pub enum GetSystemInfoResponse {
    CVendFunctionsEnhancedSystemInformationCompletion(
        super::packets::CVendFunctionsEnhancedSystemInformationCompletion,
    ),
    Abort(packets::Abort),
}

impl Sequence for GetSystemInfo {
    type Input = super::packets::CVendFunctions;
    type Output = GetSystemInfoResponse;
}

pub struct WriteFile;

pub struct File {
    /// The id as in 6.13, Table 2.
    pub file_id: u8,

    /// The file path.
    pub path: String,
}

fn convert_dir(dir: &Path) -> Result<HashMap<u8, String>> {
    let valid_paths = [
        (Path::new("firmware/kernel.gz"), 0x10),
        (Path::new("firmware/rootfs.gz"), 0x11),
        (Path::new("firmware/components.tar.gz"), 0x12),
        (Path::new("firmware/update.spec"), 0x13),
        (Path::new("firmware/update_extended.spec"), 0x14),
        (Path::new("app0/update.spec"), 0x20),
        (Path::new("app0/update.tar.gz"), 0x21),
        (Path::new("app1/update.spec"), 0x22),
        (Path::new("app1/update.tar.gz"), 0x23),
        (Path::new("app2/update.spec"), 0x24),
        (Path::new("app2/update.tar.gz"), 0x25),
        (Path::new("app3/update.spec"), 0x26),
        (Path::new("app3/update.tar.gz"), 0x27),
        (Path::new("app4/update.spec"), 0x28),
        (Path::new("app4/update.tar.gz"), 0x29),
        (Path::new("app5/update.spec"), 0x30),
        (Path::new("app5/update.tar.gz"), 0x31),
        (Path::new("app6/update.spec"), 0x32),
        (Path::new("app6/update.tar.gz"), 0x33),
        (Path::new("app7/update.spec"), 0x34),
        (Path::new("app7/update.tar.gz"), 0x35),
    ];
    let mut out = HashMap::new();

    for (p, i) in valid_paths.iter() {
        let full_path = dir.join(p);
        if full_path.exists() {
            out.insert(*i, full_path.into_os_string().into_string().unwrap());
        }
    }

    if out.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "The directory contained no valid data",
        )
        .into());
    }
    Ok(out)
}

#[derive(Debug, ZvtEnum)]
pub enum WriteFileResponse {
    CompletionData(packets::CompletionData),
    RequestForData(super::packets::RequestForData),
    Abort(packets::Abort),
}

impl WriteFile {
    pub fn into_stream<Source>(
        path: PathBuf,
        password: usize,
        adpu_size: u32,
        src: &mut PacketWriter<Source>,
    ) -> Pin<Box<impl Stream<Item = Result<WriteFileResponse>> + '_>>
    where
        Source: AsyncReadExt + AsyncWriteExt + Unpin + Send,
    {
        // Protocol from the handbook (the numbering is not part of the handbook)
        // 1.1 ECR->PT: Send over the list of all files with their sizes.
        // 1.2 PT->ECR: Ack
        // 2.1 PT->ERC: Send a Request with file id and offset
        // 2.2.ERC->PT: Send over the file
        // The steps 2.1 and 2.2. may be repeated
        // 3.0 PT->ERC replies with Completion.

        let s = try_stream! {
            use super::packets::tlv::File as TlvFile;
            let files = convert_dir(&path)?;
            let mut packets = Vec::with_capacity(files.len());
            for f in files.iter() {
                // Get the size.
                let size = std::fs::File::open(f.1)?.seek(std::io::SeekFrom::End(0))?;
                println!("The file {} has the size {}", f.1, size);

                // Convert to packet.
                packets.push(TlvFile {
                    file_id: Some(*f.0),
                    file_size: Some(size as u32),
                    file_offset: None,
                    payload: None,
                });
            }

            let packet = super::packets::WriteFile {
                password,
                tlv: Some(super::packets::tlv::WriteFile { files: packets }),
            };

            // 1.1. and 1.2
            src.write_packet_with_ack(&packet).await?;
            let mut buf = vec![0; adpu_size as usize];
            println!("the length is {}", buf.len());

            loop {
                // Get the data.
                let bytes = src.read_packet().await?;
                println!("The packet is {:?}", bytes);

                let response = WriteFileResponse::zvt_parse(&bytes)?;

                match response {
                    WriteFileResponse::CompletionData(_) => {
                        src.write_packet(&packets::Ack {}).await?;

                        yield response;
                        break;
                    }
                    WriteFileResponse::Abort(_) => {
                        src.write_packet(&packets::Ack {}).await?;

                        yield response;
                        break;
                    }
                    WriteFileResponse::RequestForData(ref data) => {
                        // Unwrap the request.
                        let request = data
                            .tlv
                            .as_ref()
                            .ok_or(ZVTError::IncompleteData)?
                            .file
                            .as_ref()
                            .ok_or(ZVTError::IncompleteData)?;

                        let file_id = request.file_id.as_ref().ok_or(ZVTError::IncompleteData)?;
                        let file_offset = request
                            .file_offset
                            .as_ref()
                            .ok_or(ZVTError::IncompleteData)?;

                        // Get the file from the hashmap
                        let file_path = files.get(file_id).ok_or(ZVTError::IncompleteData)?;

                        // Read into the buffer.
                        let file = std::fs::File::open(file_path)?;
                        let read_bytes = file.read_at(&mut buf, *file_offset as u64)?;

                        println!(
                            "Sending file {} at position {} with length {}",
                            file_path, file_offset, read_bytes
                        );

                        let packet = super::packets::WriteData {
                            tlv: Some(super::packets::tlv::WriteData {
                                file: Some(super::packets::tlv::File {
                                    file_id: request.file_id,
                                    file_offset: request.file_offset,
                                    file_size: None,
                                    payload: Some(buf[..read_bytes].to_vec()),
                                }),
                            }),
                        };
                        src.write_packet(&packet).await?;

                        yield response;
                    }
                }
            }
        };
        Box::pin(s)
    }
}
