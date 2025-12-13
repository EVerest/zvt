use crate::{encoding, length, Zvt};

pub mod tlv;

/// Implements the SetTimeAndDate packet. See chapter 3.4
#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x04, instr = 0x01)]
pub struct SetTimeAndDate {
    #[zvt_bmp(number = 0xaa, length = length::Fixed<3>, encoding = encoding::Bcd)]
    pub date: usize,

    #[zvt_bmp(number = 0x0c, length = length::Fixed<3>, encoding = encoding::Bcd)]
    pub time: usize,
}

#[derive(Debug, Default, PartialEq, Zvt)]
pub struct NumAndTotal {
    pub num: u8,

    #[zvt_bmp(length = length::Fixed<6>, encoding = encoding::Bcd)]
    pub total: usize,
}

#[derive(Debug, Default, PartialEq, Zvt)]
pub struct SingleAmounts {
    #[zvt_bmp(length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub receipt_no_start: usize,

    #[zvt_bmp(length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub receipt_no_end: usize,

    pub girocard: NumAndTotal,
    pub jcb: NumAndTotal,
    pub eurocard: NumAndTotal,
    pub amex: NumAndTotal,
    pub visa: NumAndTotal,
    pub diners: NumAndTotal,
    pub others: NumAndTotal,
}

#[derive(Debug, Default, PartialEq, Zvt)]
#[zvt_control_field(class = 0x04, instr = 0x0f)]
pub struct StatusInformation {
    #[zvt_bmp(number = 0x04, length = length::Fixed<6>, encoding = encoding::Bcd)]
    pub amount: Option<usize>,

    #[zvt_bmp(number = 0x0b, length = length::Fixed<3>, encoding = encoding::Bcd)]
    pub trace_number: Option<usize>,

    #[zvt_bmp(number = 0x0c, length = length::Fixed<3>, encoding = encoding::Bcd)]
    pub time: Option<usize>,

    #[zvt_bmp(number = 0x0d, length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub date: Option<usize>,

    #[zvt_bmp(number = 0x0e, length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub expiry_date: Option<usize>,

    #[zvt_bmp(number = 0x17, length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub card_sequence_number: Option<usize>,

    #[zvt_bmp(number = 0x19)]
    pub card_type: Option<u8>,

    #[zvt_bmp(number = 0x22, length = length::Llv, encoding = encoding::Bcd)]
    pub card_number: Option<usize>,

    #[zvt_bmp(number = 0x23, length = length::Llv, encoding= encoding::Hex)]
    pub track_2_data: Option<String>,

    #[zvt_bmp(number = 0x27, length = length::Fixed<1>)]
    pub result_code: Option<u8>,

    #[zvt_bmp(number = 0x29, length = length::Fixed<4>, encoding = encoding::Bcd)]
    pub terminal_id: Option<usize>,

    #[zvt_bmp(number = 0x2a, length = length::Fixed<15>)]
    pub vu_number: Option<String>,

    #[zvt_bmp(number = 0x3b, length  = length::Fixed<8>)]
    pub aid_authorization_attribute: Option<String>,

    #[zvt_bmp(number = 0x3c, length = length::Lllv)]
    pub additional_text: Option<String>,

    #[zvt_bmp(number = 0x60, length = length::Lllv)]
    pub single_amounts: Option<SingleAmounts>,

    #[zvt_bmp(number = 0x87, length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub receipt_no: Option<usize>,

    #[zvt_bmp(number = 0x49, length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub currency: Option<usize>,

    #[zvt_bmp(number = 0x8a)]
    pub zvt_card_type: Option<u8>,

    #[zvt_bmp(number = 0x8b, length = length::Llv)]
    pub card_name: Option<String>,

    #[zvt_bmp(number = 0x8c)]
    pub zvt_card_type_id: Option<u8>,

    #[zvt_bmp(number = 0x06, length = length::Tlv)]
    pub tlv: Option<tlv::StatusInformation>,
}

#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x04, instr = 0xff)]
pub struct IntermediateStatusInformation {
    pub status: u8,

    #[zvt_bmp(encoding = encoding::Bcd)]
    pub timeout: Option<u8>,
}

/// Chapter 2.55
#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x05, instr = 0x01)]
pub struct StatusEnquiry {
    // TODO(ddo) the password must only be set if service byte is also set.
    #[zvt_bmp(length = length::Fixed<3>, encoding = encoding::Bcd)]
    pub password: Option<usize>,

    #[zvt_bmp(number = 0x03)]
    pub service_byte: Option<u8>,

    #[zvt_bmp(number = 0x06, length = length::Tlv)]
    pub tlv: Option<tlv::StatusEnquiry>,
}

#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x00)]
pub struct Registration {
    #[zvt_bmp(length = length::Fixed<3>, encoding = encoding::Bcd)]
    pub password: usize,

    pub config_byte: u8,

    #[zvt_bmp(length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub currency: Option<usize>,

    #[zvt_bmp(number = 0x06, length = length::Tlv)]
    pub tlv: Option<tlv::Registration>,
}

#[derive(Debug, Default, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x01)]
pub struct Authorization {
    #[zvt_bmp(number = 0x04, length = length::Fixed<6>, encoding = encoding::Bcd)]
    pub amount: Option<usize>,

    #[zvt_bmp(number = 0x49, length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub currency: Option<usize>,

    #[zvt_bmp(number = 0x19)]
    pub payment_type: Option<u8>,

    #[zvt_bmp(number = 0x0e, length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub expiry_date: Option<usize>,

    #[zvt_bmp(number = 0x22, length = length::Llv, encoding = encoding::Bcd)]
    pub card_number: Option<usize>,

    #[zvt_bmp(number = 0x23, length = length::Llv, encoding= encoding::Hex)]
    pub track_2_data: Option<String>,

    // Unclear how to interpret this.
    #[zvt_bmp(number = 0x01)]
    pub timeout: Option<u8>,

    #[zvt_bmp(number = 0x02)]
    pub maximum_no_of_status_info: Option<u8>,

    #[zvt_bmp(number = 0x05)]
    pub pump_no: Option<u8>,

    #[zvt_bmp(number = 0x3c, length = length::Lllv)]
    pub additional_text: Option<String>,

    #[zvt_bmp(number = 0x8a)]
    pub zvt_card_type: Option<u8>,

    #[zvt_bmp(number = 0x06, length = length::Tlv)]
    pub tlv: Option<tlv::AuthorizationData>,
}

#[derive(Debug, Default, PartialEq, Eq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x0f)]
pub struct CompletionData {
    #[zvt_bmp(number = 0x27)]
    pub result_code: Option<u8>,

    #[zvt_bmp(number = 0x19)]
    pub status_byte: Option<u8>,

    #[zvt_bmp(number = 0x29, length = length::Fixed<4>, encoding = encoding::Bcd)]
    pub terminal_id: Option<usize>,

    #[zvt_bmp(number = 0x49, length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub currency: Option<usize>,
}

#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x0f)]
pub struct ReceiptPrintoutCompletion {
    #[zvt_bmp(length = length::Lllv, encoding = encoding::Utf8)]
    pub sw_version: String,

    pub terminal_status_code: u8,

    #[zvt_bmp(number = 0x06, length = length::Tlv)]
    pub tlv: Option<tlv::ReceiptPrintoutCompletion>,
}

/// Resets the terminal.
///
/// See chapter 2.43.
#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x18)]
pub struct ResetTerminal {}

/// See chapter 2.44
#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x1a)]
pub struct PrintSystemConfiguration {}

/// Set/Reset the terminal id.
///
/// See chapter 2.45.
#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x1b)]
pub struct SetTerminalId {
    #[zvt_bmp(length = length::Fixed<3>, encoding = encoding::Bcd)]
    pub password: usize,

    #[zvt_bmp(number = 0x29, length = length::Fixed<4>, encoding = encoding::Bcd)]
    pub terminal_id: Option<usize>,
}

#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x1e)]
pub struct Abort {
    pub error: u8,
}

// Defined in 2.2.9
#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x1e)]
pub struct ReservationAbort {
    pub error: u8,

    // The currency code is here untagged and will be only included if the
    // error code above evaluates to 0x6f.
    #[zvt_bmp(length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub currency: Option<usize>,

    #[zvt_bmp(number = 0x06, length = length::Tlv)]
    pub tlv: Option<tlv::ReservationAbort>,
}

/// Defined in 2.10.1.
#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x1e)]
pub struct PartialReversalAbort {
    // Must be [ErrorMessage::ErrorPreAuthorization].
    pub error: u8,

    #[zvt_bmp(number = 0x87, length = length::Fixed<2>, encoding = PartialReversalReceiptNo)]
    pub receipt_no: Option<usize>,
    // TODO(ddo) There is also an tlv field which may contain further receipt
    // numbers. Produce the message to understand how it looks like.
}

/// Pre-Authorization/Reservation.
///
/// See chapter 2.8.
#[derive(Debug, Default, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x22)]
pub struct Reservation {
    #[zvt_bmp(number = 0x04, length = length::Fixed<6>, encoding = encoding::Bcd)]
    pub amount: Option<usize>,

    #[zvt_bmp(number = 0x49, length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub currency: Option<usize>,

    #[zvt_bmp(number = 0x19)]
    pub payment_type: Option<u8>,

    #[zvt_bmp(number = 0x0e, length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub expiry_date: Option<usize>,

    #[zvt_bmp(number = 0x22, length = length::Llv, encoding = encoding::Bcd)]
    pub card_number: Option<usize>,

    #[zvt_bmp(number = 0x23, length = length::Llv, encoding= encoding::Hex)]
    pub track_2_data: Option<String>,

    // Unclear how to interpret this.
    #[zvt_bmp(number = 0x01)]
    pub timeout: Option<u8>,

    #[zvt_bmp(number = 0x02)]
    pub maximum_no_of_status_info: Option<u8>,

    #[zvt_bmp(number = 0x05)]
    pub pump_no: Option<u8>,

    #[zvt_bmp(number = 0x0b, length = length::Fixed<3>, encoding = encoding::Bcd)]
    pub trace_number: Option<usize>,

    #[zvt_bmp(number = 0x3b, length = length::Fixed<8>)]
    pub aid_authorization_attribute: Option<String>,

    #[zvt_bmp(number = 0x3c, length = length::Lllv)]
    pub additional_text: Option<String>,

    #[zvt_bmp(number = 0x8a)]
    pub zvt_card_type: Option<u8>,

    #[zvt_bmp(number = 0x06, length = length::Tlv)]
    pub tlv: Option<tlv::PreAuthData>,
}

/// Encoding for receipt-no field in the PartialReversal struct.
///
/// The field may have a special value - 0xffff - which is not representable in
/// a 2 byte Bcd fashion. See 2.10.1 for details.
pub struct PartialReversalReceiptNo;

impl encoding::Encoding<usize> for PartialReversalReceiptNo {
    fn decode(bytes: &[u8]) -> zvt_builder::ZVTResult<(usize, &[u8])> {
        if bytes.len() < 2 {
            return Err(zvt_builder::ZVTError::IncompleteData);
        }
        if bytes[0..2] == [0xff, 0xff] {
            let tmp: u16 = encoding::Default::decode(&bytes[0..2])?.0;
            Ok((tmp as usize, &bytes[2..]))
        } else {
            Ok((encoding::Bcd::decode(&bytes[0..2])?.0, &bytes[2..]))
        }
    }

    fn encode(input: &usize) -> Vec<u8> {
        if *input == 0xffff {
            encoding::Default::encode(&(*input as u16))
        } else {
            encoding::Bcd::encode(input)
        }
    }
}

#[derive(Debug, Default, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x23)]
pub struct PartialReversal {
    #[zvt_bmp(number = 0x87, length = length::Fixed<2>, encoding = PartialReversalReceiptNo)]
    pub receipt_no: Option<usize>,

    #[zvt_bmp(number = 0x04, length = length::Fixed<6>, encoding = encoding::Bcd)]
    pub amount: Option<usize>,

    #[zvt_bmp(number = 0x19)]
    pub payment_type: Option<u8>,

    #[zvt_bmp(number = 0x49, length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub currency: Option<usize>,

    #[zvt_bmp(number = 0x06, length = length::Tlv)]
    pub tlv: Option<tlv::PreAuthData>,
}

#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x25)]
pub struct PreAuthReversal {
    #[zvt_bmp(number = 0x19)]
    pub payment_type: Option<u8>,

    #[zvt_bmp(number = 0x49, length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub currency: Option<usize>,

    #[zvt_bmp(number = 0x87, length = length::Fixed<2>, encoding = encoding::Bcd)]
    pub receipt_no: Option<usize>,
}

/// See chapter 2.16
#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x50)]
pub struct EndOfDay {
    #[zvt_bmp(length = length::Fixed<3>, encoding = encoding::Bcd)]
    pub password: usize,
}

/// Defined in 2.17.
#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x70)]
pub struct Diagnosis {
    #[zvt_bmp(number = 0x06, length = length::Tlv)]
    pub tlv: Option<tlv::Diagnosis>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DiagnosisType {
    Line = 1,
    Extended = 2,
    Configuration = 3,
    EmvConfiguration = 4,
    Ep2Configuration = 5,
}

#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x93)]
pub struct Initialization {
    #[zvt_bmp(length = length::Fixed<3>, encoding = encoding::Bcd)]
    pub password: usize,
}

#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0xc0)]
pub struct ReadCard {
    pub timeout_sec: u8,

    #[zvt_bmp(number = 0x19)]
    pub card_type: Option<u8>,

    #[zvt_bmp(number = 0xfc)]
    pub dialog_control: Option<u8>,

    #[zvt_bmp(number = 0x06, length = length::Tlv)]
    pub tlv: Option<tlv::ReadCard>,
}

/// PrintLine message as defined in 3.5
///
/// With this command a printer integrated in or attached to the ECR can be used
/// to print a line from the transferred data. The text contains no CR LF. Empty
/// lines are transferred as print-commands with an empty text-field.
/// The command is only sent from the PT if function ECR-receipt is active on
/// the PT (see command Registration).
///
/// The ECR shall either respond with [Ack] or [Nack] with (84 cc).
#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0xd1)]
pub struct PrintLine {
    pub attribute: u8,

    pub text: String,
}

#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0xd3)]
pub struct PrintTextBlock {
    #[zvt_bmp(number = 0x06, length = length::Tlv)]
    pub tlv: Option<tlv::PrintTextBlock>,
}

/// See chapter 2.36
#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x08, instr = 0x30)]
pub struct SelectLanguage {
    language: u8,
}

#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x80, instr = 0x00)]
pub struct Ack {}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::ZvtSerializer;
    use chrono::NaiveDate;
    use std::fs;

    #[rstest::fixture]
    pub fn common_setup() {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .is_test(true)
            .init();
    }

    pub fn get_bytes(name: &str) -> Vec<u8> {
        let path_from_root = "zvt/data/".to_string();
        let base_dir = match fs::metadata(&path_from_root) {
            Ok(_) => path_from_root,
            Err(_) => format!("data/"),
        };
        fs::read(&format!("{base_dir}/{name}")).unwrap()
    }

    #[rstest::rstest]
    fn test_read_card() {
        let bytes = get_bytes("1680722649.972316000_ecr_pt.blob");
        let expected = ReadCard {
            timeout_sec: 15,
            card_type: Some(16),
            dialog_control: Some(2),
            tlv: Some(tlv::ReadCard {
                card_reading_control: Some(0xd0),
                card_type: Some(0x07),
            }),
        };
        let automatic = expected.zvt_serialize();
        assert_eq!(automatic, bytes);
        let output = ReadCard::zvt_deserialize(&bytes).unwrap();
        assert_eq!(expected, output.0);
    }

    #[rstest::rstest]
    fn test_status_information() {
        let bytes = get_bytes("1680728161.963129000_pt_ecr.blob");
        let expected = StatusInformation {
            result_code: Some(0),
            tlv: Some(tlv::StatusInformation {
                uuid: Some("000000000000081ca72f".to_string()),
                ats: Some("0578807002".to_string()),
                card_type: Some(1),
                maximum_pre_autorisation: None,
                card_identification_item: None,
                subs_on_card: None,
                sub_type: Some("fe04".to_string()),
                atqa: Some("0400".to_string()),
                sak: Some(0x20),
                subs: vec![tlv::Subs {
                    application_id: Some("a0000000041010".to_string()),
                    card_type: None,
                }],
            }),
            ..StatusInformation::default()
        };
        assert_eq!(expected.zvt_serialize(), bytes);
        assert_eq!(
            expected,
            StatusInformation::zvt_deserialize(&bytes).unwrap().0
        );

        let bytes = get_bytes("1680728165.675509000_pt_ecr.blob");
        let expected = StatusInformation {
            result_code: Some(0),
            amount: Some(2500),
            card_type: Some(0x60),
            currency: Some(978),
            time: Some(225558),
            date: Some(405),
            card_number: Some(5598845555548074),
            receipt_no: Some(231),
            aid_authorization_attribute: Some("750071".to_string()),
            trace_number: Some(975),
            terminal_id: Some(52523535),
            expiry_date: Some(2405),
            zvt_card_type: Some(6),
            zvt_card_type_id: Some(1),
            card_name: Some("MasterCard".to_string()),
            vu_number: Some("804011926      ".to_string()),
            ..StatusInformation::default()
        };
        assert_eq!(
            expected,
            StatusInformation::zvt_deserialize(&bytes).unwrap().0
        );

        let bytes = get_bytes("1680728215.659492000_pt_ecr.blob");
        let expected = StatusInformation {
            result_code: Some(0),
            card_type: Some(0x60),
            amount: Some(0),
            currency: Some(978),
            time: Some(225558),
            date: Some(405),
            card_number: Some(5598845555548074),
            receipt_no: Some(232),
            aid_authorization_attribute: Some("750071".to_string()),
            trace_number: Some(977),
            terminal_id: Some(52523535),
            expiry_date: Some(2405),
            zvt_card_type: Some(6),
            zvt_card_type_id: Some(1),
            card_name: Some("MasterCard".to_string()),
            vu_number: Some("804011926      ".to_string()),
            additional_text: Some(
                "AS-Proc-Code= 00 076 06\rCapt.-Ref.= 0099\rAID59= 081520\r DAUER   7 TAGE"
                    .to_string(),
            ),
            ..StatusInformation::default()
        };
        assert_eq!(
            expected,
            StatusInformation::zvt_deserialize(&bytes).unwrap().0
        );

        let bytes = get_bytes("1680761828.489701000_pt_ecr.blob");
        let expected = StatusInformation {
            result_code: Some(0),
            amount: Some(958),
            trace_number: Some(982),
            date: Some(406),
            time: Some(81706),
            single_amounts: Some(SingleAmounts {
                receipt_no_start: 233,
                receipt_no_end: 234,
                eurocard: NumAndTotal { num: 2, total: 958 },
                ..SingleAmounts::default()
            }),
            ..StatusInformation::default()
        };
        assert_eq!(
            expected,
            StatusInformation::zvt_deserialize(&bytes).unwrap().0
        );

        // After pre-auth.
        let bytes = get_bytes("1682066249.409078000_pt_ecr.blob");
        let expected = StatusInformation {
            result_code: Some(0),
            amount: Some(2500),
            currency: Some(978),
            date: Some(421),
            time: Some(103720),
            card_number: Some(4711008005757038004),
            card_sequence_number: Some(2),
            receipt_no: Some(249),
            aid_authorization_attribute: Some("018372".to_string()),
            trace_number: Some(1012),
            card_type: Some(0x60),
            terminal_id: Some(52523535),
            expiry_date: Some(2612),
            zvt_card_type: Some(5),
            zvt_card_type_id: Some(0),
            card_name: Some("girocard".to_string()),
            vu_number: Some("16004008       ".to_string()),
            ..StatusInformation::default()
        };
        assert_eq!(
            expected,
            StatusInformation::zvt_deserialize(&bytes).unwrap().0
        );

        let bytes = [
            4, 15, 100, 39, 0, 6, 96, 76, 10, 0, 0, 0, 0, 0, 0, 8, 255, 105, 20, 31, 69, 12, 12,
            120, 128, 116, 3, 128, 49, 192, 115, 214, 49, 192, 31, 76, 1, 1, 31, 77, 2, 254, 4, 31,
            79, 2, 4, 0, 31, 80, 1, 32, 96, 11, 67, 9, 160, 0, 0, 0, 89, 69, 67, 1, 0, 96, 12, 67,
            10, 160, 0, 0, 3, 89, 16, 16, 2, 128, 1, 96, 11, 67, 9, 210, 118, 0, 0, 37, 71, 65, 1,
            0, 96, 9, 67, 7, 160, 0, 0, 0, 4, 16, 16,
        ];
        let expected = StatusInformation::zvt_deserialize(&bytes).unwrap().0;
        assert_eq!(expected.tlv.as_ref().unwrap().subs.len(), 4);
        assert_eq!(bytes, expected.zvt_serialize().as_slice());

        let bytes = [
            4, 15, 34, 39, 0, 6, 30, 76, 10, 0, 0, 0, 4, 99, 200, 178, 174, 79, 128, 31, 76, 1, 1,
            31, 77, 2, 0, 3, 31, 79, 2, 68, 0, 31, 80, 1, 0,
        ];
        let expected = StatusInformation::zvt_deserialize(&bytes).unwrap().0;
        assert!(expected.tlv.as_ref().unwrap().subs.is_empty());
    }

    #[rstest::rstest]
    fn test_status_information_read_card() {
        let bytes = get_bytes("status_information_read_card.blob");
        let expected = StatusInformation {
            result_code: Some(0),
            track_2_data: Some("6725904411001000142d24122012386013860f".to_string()),
            tlv: Some(tlv::StatusInformation {
                maximum_pre_autorisation: Some(10000),
                card_identification_item: Some("3f56a32065cc4dbe8330c37609f91996".to_string()),
                uuid: Some("00000000000008b3c880".to_string()),
                ats: Some("0c788074038031c073d631c0".to_string()),
                card_type: Some(1),
                sub_type: Some("fe04".to_string()),
                atqa: Some("0400".to_string()),
                sak: Some(32),
                subs: Vec::new(),
                subs_on_card: Some(tlv::SubsOnCard {
                    subs: vec![
                        tlv::Subs {
                            application_id: Some("a0000003591010028001".to_string()),
                            card_type: Some("0005".to_string()),
                        },
                        tlv::Subs {
                            application_id: Some("a0000000043060".to_string()),
                            card_type: Some("002e".to_string()),
                        },
                    ],
                }),
            }),
            ..StatusInformation::default()
        };
        assert_eq!(
            expected,
            StatusInformation::zvt_deserialize(&bytes).unwrap().0
        );
    }

    #[rstest::rstest]
    fn test_receipt_printout_completion() {
        let bytes = get_bytes("1680728219.054216000_pt_ecr.blob");
        let expected = ReceiptPrintoutCompletion {
            sw_version: "GER-APP-v2.0.9;cS02.01.01-10.10-2-2;CC26".to_string(),
            terminal_status_code: 0,
            tlv: Some(tlv::ReceiptPrintoutCompletion {
                terminal_id: Some(52523535),
                device_information: Some(tlv::DeviceInformation {
                    device_name: Some("cVEND plug".to_string()),
                    software_version: Some("GER-APP-v2.0.9;cS02.01.01-10.10-2-2;CC26".to_string()),
                    serial_number: Some(18632442),
                    device_state: Some(0),
                }),
                date_time: Some(
                    NaiveDate::from_ymd_opt(2023, 4, 5)
                        .unwrap()
                        .and_hms_opt(22, 56, 55)
                        .unwrap(),
                ),
            }),
        };
        assert_eq!(
            ReceiptPrintoutCompletion::zvt_deserialize(&bytes)
                .unwrap()
                .0,
            expected
        );
    }

    #[rstest::rstest]
    fn test_pre_auth_data() {
        let bytes = get_bytes("1680728162.033575000_ecr_pt.blob");
        let expected = Reservation {
            payment_type: Some(0x40),
            currency: Some(978),
            amount: Some(2500),
            tlv: Some(tlv::PreAuthData {
                bmp_data: Some(tlv::Bmp60 {
                    bmp_prefix: "AC".to_string(),
                    bmp_data: "384HH2".to_string(),
                }),
            }),
            ..Reservation::default()
        };
        assert_eq!(Reservation::zvt_deserialize(&bytes).unwrap().0, expected,);
    }

    #[rstest::rstest]
    fn test_registration() {
        let bytes = get_bytes("1681273860.511128000_ecr_pt.blob");
        let expected = Registration {
            password: 123456,
            config_byte: 0xde,
            currency: Some(978),
            tlv: None,
        };
        assert_eq!(bytes, expected.zvt_serialize());
        assert_eq!(Registration::zvt_deserialize(&bytes).unwrap().0, expected);
    }

    #[rstest::rstest]
    fn test_pre_auth_reversal() {
        let bytes = get_bytes("1680728213.562478000_ecr_pt.blob");
        let expected = PreAuthReversal {
            payment_type: Some(0x40),
            receipt_no: Some(231),
            currency: Some(978),
        };
        assert_eq!(
            PreAuthReversal::zvt_deserialize(&bytes).unwrap().0,
            expected
        );
    }

    #[rstest::rstest]
    fn test_completion_data() {
        let bytes = get_bytes("1680761818.641601000_pt_ecr.blob");
        let golden = CompletionData {
            result_code: None,
            status_byte: Some(0x10),
            terminal_id: Some(52523535),
            currency: Some(978),
        };
        let out = golden.zvt_serialize();
        assert_eq!(bytes, out);
        assert_eq!(CompletionData::zvt_deserialize(&bytes).unwrap().0, golden);
    }

    #[rstest::rstest]
    fn test_end_of_day() {
        let bytes = get_bytes("1681282621.302434000_ecr_pt.blob");
        let expected = EndOfDay { password: 123456 };

        assert_eq!(EndOfDay::zvt_deserialize(&bytes).unwrap().0, expected);
        assert_eq!(bytes, expected.zvt_serialize());
    }

    #[rstest::rstest]
    fn test_partial_reversal() {
        let bytes = get_bytes("1681455683.221609000_ecr_pt.blob");
        let expected = PartialReversal {
            receipt_no: Some(491),
            payment_type: Some(0x40),
            currency: Some(978),
            amount: Some(1295),
            tlv: Some(tlv::PreAuthData {
                bmp_data: Some(tlv::Bmp60 {
                    bmp_prefix: "AC".to_string(),
                    bmp_data: "MF2246".to_string(),
                }),
            }),
        };
        assert_eq!(
            PartialReversal::zvt_deserialize(&bytes).unwrap().0,
            expected
        );
    }

    #[rstest::rstest]
    fn test_intermediate_status() {
        let bytes = get_bytes("1680728162.647465000_pt_ecr.blob");
        let expected = IntermediateStatusInformation {
            status: 0x17,
            timeout: Some(0),
        };
        assert_eq!(
            IntermediateStatusInformation::zvt_deserialize(&bytes)
                .unwrap()
                .0,
            expected
        );
        assert_eq!(bytes, expected.zvt_serialize());
    }

    #[rstest::rstest]
    fn test_print_text_block() {
        let bytes = get_bytes("1680728215.585561000_pt_ecr.blob");
        let actual = PrintTextBlock::zvt_deserialize(&bytes).unwrap().0;
        let tlv = actual.tlv.as_ref().unwrap();
        let lines = tlv.lines.as_ref().unwrap();
        assert_eq!(lines.lines.len(), 33);
        assert_eq!(lines.eol, Some(255));
        assert_eq!(tlv.receipt_type, Some(2));
        assert_eq!(bytes, actual.zvt_serialize());
    }

    #[rstest::rstest]
    fn test_text_block_system_information() {
        let bytes = get_bytes("print_system_configuration_reply.blob");
        let actual = PrintTextBlock::zvt_deserialize(&bytes).unwrap().0;
        let tlv = actual.tlv.as_ref().unwrap();
        let lines = tlv.lines.as_ref().unwrap();
        assert_eq!(lines.lines.len(), 118);
        assert_eq!(lines.eol, None);
        assert_eq!(tlv.receipt_type, Some(3));
        assert_eq!(bytes, actual.zvt_serialize());
    }

    #[rstest::rstest]
    fn test_partial_reversal_abort() {
        let bytes = get_bytes("partial_reversal.blob");
        let actual = PartialReversalAbort::zvt_deserialize(&bytes).unwrap().0;
        let expected = PartialReversalAbort {
            error: 184,
            receipt_no: Some(0xffff),
        };

        assert_eq!(actual, expected);
        assert_eq!(expected.zvt_serialize(), bytes);
    }
}
