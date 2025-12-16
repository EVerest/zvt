use crate::packets;
use crate::ZvtEnum;


/// Enum that wraps all ZVT command structs from packets.rs
#[derive(Debug, PartialEq, ZvtEnum)]
pub enum Command {
    /// PT to ECR - Chapter 3.4
    /// class = 0x04, instr = 0x01
    SetTimeAndDate(packets::SetTimeAndDate),
    /// PT to ECR - Chapter 2.2.6
    /// class = 0x04, instr = 0x0f
    StatusInformation(packets::StatusInformation),
    /// PT to ECR - Chapter 3.7
    /// class = 0x04, instr = 0xff
    IntermediateStatusInformation(packets::IntermediateStatusInformation),
    /// ECR to PT - Chapter 2.58
    /// class = 0x05, instr = 0x01
    StatusEnquiry(packets::StatusEnquiry),
    /// ECR to PT - Chapter 2.1
    /// class = 0x06, instr = 0x00
    Registration(packets::Registration),
    /// ECR to PT - Chapter 2.2
    /// class = 0x06, instr = 0x01
    Authorization(packets::Authorization),
    /// PT to ECR - Chapter 2.1
    /// class = 0x06, instr = 0x0f
    CompletionData(packets::CompletionData),
    /// PT to ECR
    /// class = 0x06, instr = 0x0f
    ReceiptPrintoutCompletion(packets::ReceiptPrintoutCompletion),
    /// ECR to PT - Chapter 2.46
    /// class = 0x06, instr = 0x18
    ResetTerminal(packets::ResetTerminal),
    /// ECR to PT - Chapter 2.47
    /// class = 0x06, instr = 0x1a
    PrintSystemConfiguration(packets::PrintSystemConfiguration),
    /// ECR to PT - Chapter 2.48
    /// class = 0x06, instr = 0x1b
    SetTerminalId(packets::SetTerminalId),
    /// PT to ECR - Chapter 2.2.9
    /// class = 0x06, instr = 0x1e
    Abort(packets::Abort),
    /// PT to ECR - Chapter 2.2.9
    /// class = 0x06, instr = 0x1e
    ReservationAbort(packets::ReservationAbort),
    /// PT to ECR - Chapter 2.10.1
    /// class = 0x06, instr = 0x1e
    PartialReversalAbort(packets::PartialReversalAbort),
    /// ECR to PT - Chapter 2.8
    /// class = 0x06, instr = 0x22
    Reservation(packets::Reservation),
    /// ECR to PT - Chapter 2.10
    /// class = 0x06, instr = 0x23
    PartialReversal(packets::PartialReversal),
    /// ECR to PT
    /// class = 0x06, instr = 0x25
    PreAuthReversal(packets::PreAuthReversal),
    /// ECR to PT - Chapter 2.16
    /// class = 0x06, instr = 0x50
    EndOfDay(packets::EndOfDay),
    /// ECR to PT - Chapter 2.18
    /// class = 0x06, instr = 0x70
    Diagnosis(packets::Diagnosis),
    /// ECR to PT - Chapter 2.19
    /// class = 0x06, instr = 0x93
    Initialization(packets::Initialization),
    /// ECR to PT - Chapter 2.22
    /// class = 0x06, instr = 0xc0
    ReadCard(packets::ReadCard),
    /// PT to ECR - Chapter 3.5
    /// class = 0x06, instr = 0xd1
    PrintLine(packets::PrintLine),
    /// PT to ECR
    /// class = 0x06, instr = 0xd3
    PrintTextBlock(packets::PrintTextBlock),
    /// ECR to PT - Chapter 2.38
    /// class = 0x08, instr = 0x30
    SelectLanguage(packets::SelectLanguage),

    /// class = 0x80, instr = 0x00
    Ack(packets::Ack),
    /// class = 0x84, instr = *
    #[zvt_instr_any]
    Nack(packets::Nack),
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::SetTimeAndDate(_) => write!(f, "SetTimeAndDate (04 01)"),
            Command::StatusInformation(_) => write!(f, "StatusInformation (04 0F)"),
            Command::IntermediateStatusInformation(_) => write!(f, "IntermediateStatusInformation (04 FF)"),
            Command::StatusEnquiry(_) => write!(f, "StatusEnquiry (05 01)"),
            Command::Registration(_) => write!(f, "Registration (06 00)"),
            Command::Authorization(_) => write!(f, "Authorization (06 01)"),
            Command::CompletionData(_) => write!(f, "CompletionData (06 0F)"),
            Command::ReceiptPrintoutCompletion(_) => write!(f, "ReceiptPrintoutCompletion (06 0F)"),
            Command::ResetTerminal(_) => write!(f, "ResetTerminal (06 18)"),
            Command::PrintSystemConfiguration(_) => write!(f, "PrintSystemConfiguration (06 1A)"),
            Command::SetTerminalId(_) => write!(f, "SetTerminalId (06 1B)"),
            Command::Abort(_) => write!(f, "Abort (06 1E)"),
            Command::ReservationAbort(_) => write!(f, "ReservationAbort (06 1E)"),
            Command::PartialReversalAbort(_) => write!(f, "PartialReversalAbort (06 1E)"),
            Command::Reservation(_) => write!(f, "Reservation (06 22)"),
            Command::PartialReversal(_) => write!(f, "PartialReversal (06 23)"),
            Command::PreAuthReversal(_) => write!(f, "PreAuthReversal (06 25)"),
            Command::EndOfDay(_) => write!(f, "EndOfDay (06 50)"),
            Command::Diagnosis(_) => write!(f, "Diagnosis (06 70)"),
            Command::Initialization(_) => write!(f, "Initialization (06 93)"),
            Command::ReadCard(_) => write!(f, "ReadCard (06 C0)"),
            Command::PrintLine(_) => write!(f, "PrintLine (06 D1)"),
            Command::PrintTextBlock(_) => write!(f, "PrintTextBlock (06 D3)"),
            Command::SelectLanguage(_) => write!(f, "SelectLanguage (08 30)"),
            Command::Ack(_) => write!(f, "Ack (80 00)"),
            Command::Nack(_) => write!(f, "Nack (84 XX)"),
        }
    }
}
