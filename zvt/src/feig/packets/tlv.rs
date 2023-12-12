use crate::{
    encoding, encoding::Encoding, length, length::Length, Tag, ZVTError, ZVTResult, Zvt,
    ZvtSerializerImpl,
};

/// Custom encoder for a place vector string.
pub struct Custom;

impl encoding::Encoding<Vec<u8>> for Custom {
    fn encode(input: &Vec<u8>) -> Vec<u8> {
        input.clone()
    }

    fn decode(bytes: &[u8]) -> ZVTResult<(Vec<u8>, &[u8])> {
        Ok((bytes.to_vec(), &[]))
    }
}

/// Custom [ZvtSerializerImpl] which will just copy the data.
impl<TE: encoding::Encoding<Tag>> ZvtSerializerImpl<length::Tlv, Custom, TE> for Vec<u8> {
    fn deserialize_tagged(mut bytes: &[u8], tag: Option<Tag>) -> ZVTResult<(Self, &[u8])> {
        if let Some(desired_tag) = tag {
            let actual_tag;
            (actual_tag, bytes) = TE::decode(bytes)?;
            if actual_tag != desired_tag {
                return Err(ZVTError::WrongTag(actual_tag));
            }
        }
        let (length, payload) = length::Tlv::deserialize(bytes)?;
        if length > payload.len() {
            return Err(ZVTError::IncompleteData);
        }
        let (data, remainder) = Custom::decode(&payload[..length])?;

        Ok((data, &payload[length - remainder.len()..]))
    }

    fn serialize_tagged(&self, tag: Option<Tag>) -> Vec<u8> {
        let mut output = Vec::new();
        if self.is_empty() {
            return output;
        }
        if let Some(tag) = tag {
            output = TE::encode(&tag);
        }
        let mut length = length::Tlv::serialize(self.len());
        output.append(&mut length);
        output.append(&mut Custom::encode(self));
        output
    }
}

#[derive(Debug, PartialEq, Zvt, Default)]
pub struct File {
    #[zvt_tlv(tag = 0x1d)]
    pub file_id: Option<u8>,

    #[zvt_tlv(tag = 0x1e, encoding = encoding::BigEndian)]
    pub file_offset: Option<u32>,

    #[zvt_tlv(tag = 0x1f00, encoding = encoding::BigEndian)]
    pub file_size: Option<u32>,

    #[zvt_tlv(tag = 0x1c, encoding = Custom)]
    pub payload: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, Zvt)]
pub struct WriteData {
    #[zvt_tlv(tag = 0x2d)]
    pub file: Option<File>,
}

#[derive(Debug, PartialEq, Zvt)]
pub struct WriteFile {
    #[zvt_tlv(tag = 0x2d)]
    pub files: Vec<File>,
}

#[derive(Debug, PartialEq, Zvt, Default)]
pub struct HostConfigurationData {
    #[zvt_bmp(encoding = encoding::BigEndian)]
    pub ip: u32,

    #[zvt_bmp(encoding = encoding::BigEndian)]
    pub port: u16,

    #[zvt_bmp(encoding = encoding::BigEndian)]
    pub config_byte: u8,
}

#[derive(Debug, PartialEq, Zvt, Default)]
pub struct SystemInformation {
    #[zvt_tlv(encoding = encoding::Bcd, tag = 0xff40)]
    pub password: usize,

    #[zvt_tlv(tag = 0xff41)]
    pub host_configuration_data: Option<HostConfigurationData>,
}

#[derive(Debug, PartialEq, Zvt, Default)]
pub struct ChangeConfiguration {
    #[zvt_tlv(tag = 0xe4)]
    pub system_information: SystemInformation,
}
