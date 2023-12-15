use crate::{encoding, Zvt};
use chrono::NaiveDateTime;

#[derive(Debug, Default, PartialEq, Zvt)]
pub struct Subs {
    #[zvt_tlv(tag = 0x41, encoding = encoding::Hex)]
    pub card_type: Option<String>,

    #[zvt_tlv(tag = 0x43, encoding = encoding::Hex)]
    pub application_id: Option<String>,
}

#[derive(Debug, Default, PartialEq, Zvt)]
pub struct SubsOnCard {
    #[zvt_tlv(tag = 0x60)]
    pub subs: Vec<Subs>,
}

#[derive(Debug, Default, PartialEq, Zvt)]
pub struct StatusInformation {
    #[zvt_tlv(tag = 0x4c, encoding = encoding::Hex)]
    pub uuid: Option<String>,

    #[zvt_tlv(tag = 0x1f0b,  encoding = encoding::Bcd)]
    pub maximum_pre_autorisation: Option<usize>,

    #[zvt_tlv(tag = 0x1f14, encoding = encoding::Hex)]
    pub card_identification_item: Option<String>,

    #[zvt_tlv(tag = 0x1f45, encoding = encoding::Hex)]
    pub ats: Option<String>,

    #[zvt_tlv(tag = 0x1f4c)]
    pub card_type: Option<u8>,

    #[zvt_tlv(tag = 0x1f4d,  encoding = encoding::Hex)]
    pub sub_type: Option<String>,

    #[zvt_tlv(tag = 0x1f4f,  encoding = encoding::Hex)]
    pub atqa: Option<String>,

    #[zvt_tlv(tag = 0x1f50)]
    pub sak: Option<u8>,

    // The documentation just says that this tag may be present but in reality
    // this is a vector.
    #[zvt_tlv(tag = 0x60)]
    pub subs: Vec<Subs>,

    #[zvt_tlv(tag = 0x62)]
    pub subs_on_card: Option<SubsOnCard>,
}

#[derive(Debug, Default, PartialEq, Zvt)]
pub struct StatusEnquiry {
    #[zvt_tlv(tag = 0x1Ff2)]
    enable_extended_contactless_card_detection: Option<u8>,
}

#[derive(Debug, PartialEq, Zvt)]
pub struct DeviceInformation {
    #[zvt_tlv(tag = 0x1f40)]
    pub device_name: Option<String>,

    #[zvt_tlv(tag = 0x1f41)]
    pub software_version: Option<String>,

    #[zvt_tlv(tag = 0x1f42, encoding = encoding::Bcd)]
    pub serial_number: Option<usize>,

    #[zvt_tlv(tag = 0x1f43)]
    pub device_state: Option<u8>,
}

#[derive(Debug, PartialEq, Zvt)]
pub struct ReceiptPrintoutCompletion {
    #[zvt_tlv(tag = 0x1f44, encoding = encoding::Bcd)]
    pub terminal_id: Option<usize>,

    #[zvt_tlv(tag = 0xe4)]
    pub device_information: Option<DeviceInformation>,

    #[zvt_tlv(tag = 0x34)]
    pub date_time: Option<NaiveDateTime>,
}

#[derive(Debug, PartialEq, Zvt)]
pub struct ReservationAbort {
    // Spec says a "variable length binary". Not sure what this means regarding
    // the encoding.
    #[zvt_tlv(tag = 0x1f16, encoding = encoding::Bcd)]
    pub extended_error_code: Option<usize>,

    #[zvt_tlv(tag = 0x1f17)]
    pub extended_error_text: Option<String>,
}

#[derive(Debug, PartialEq, Zvt)]
pub struct Bmp60 {
    #[zvt_tlv(tag = 0x1f62)]
    pub bmp_prefix: String,

    #[zvt_tlv(tag = 0x1f63)]
    pub bmp_data: String,
}

#[derive(Debug, PartialEq, Zvt)]
pub struct PreAuthData {
    #[zvt_tlv(tag = 0xe9)]
    pub bmp_data: Option<Bmp60>,
}

#[derive(Debug, PartialEq, Zvt)]
pub struct Diagnosis {
    #[zvt_tlv(tag = 0x1b)]
    pub diagnosis_type: Option<u8>,
}

#[derive(Debug, PartialEq, Zvt)]
pub struct ReadCard {
    #[zvt_tlv(tag = 0x1f15)]
    pub card_reading_control: Option<u8>,

    #[zvt_tlv(tag = 0x1f60)]
    pub card_type: Option<u8>,
}

#[derive(Debug, PartialEq, Zvt)]
pub struct ZvtString {
    #[zvt_tlv(tag = 0x07)]
    pub line: String,
}

#[derive(Debug, PartialEq, Zvt)]
pub struct TextLines {
    #[zvt_tlv(tag = 0x07)]
    pub lines: Vec<String>,

    #[zvt_tlv(tag = 0x09)]
    pub eol: Option<u8>,
}

#[derive(Debug, PartialEq, Zvt)]
pub struct PrintTextBlock {
    #[zvt_tlv(tag = 0x1f07)]
    pub receipt_type: Option<u8>,

    #[zvt_tlv(tag = 0x25)]
    pub lines: Option<TextLines>,
    // TODO(ddo) missing 0x14 (ISO character set) and 0x1f37 (Receipt information)
}

#[derive(Debug, PartialEq, Zvt)]
pub struct Registration {
    // Or what means (hi-byte sent before lo-byte)
    #[zvt_tlv(tag = 0x1a, encoding = encoding::BigEndian)]
    pub max_len_adpu: Option<u16>,
}
