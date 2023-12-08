use crate::logging::PacketWriter;
use crate::packets;
use crate::{encoding, ZvtEnum, ZvtParser, ZvtSerializer};
use anyhow::Result;
use async_stream::try_stream;
use futures::Stream;
use std::boxed::Box;
use std::marker::Unpin;
use std::pin::Pin;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// The trait for converting a sequence into a stream.
///
/// What is written below? The [Self::Input] type must be a command as defined
/// under [packets]. The [Self::Output] must implement the [ZvtParser] trait -
/// an enum listing all possible packets the PT may send to the ECR after
/// receiving the [Self::Input] message.
///
/// Additionally we enforce that the [Self::Input] and [Self::Output] are marked
/// with [Send], so the stream can be shared between threads (or moved into
/// [tokio::spawn], e.x.).
///
/// The default implementation just waits for one message, acknowledges it and
/// returns.
pub trait Sequence
where
    Self::Input: ZvtSerializer + Send + Sync,
    encoding::Default: encoding::Encoding<Self::Input>,
{
    type Input;
    type Output: ZvtParser + Send;

    fn into_stream<'a, Source>(
        input: &'a Self::Input,
        src: &'a mut PacketWriter<Source>,
    ) -> Pin<Box<dyn futures::Stream<Item = Result<Self::Output>> + Send + 'a>>
    where
        Source: AsyncReadExt + AsyncWriteExt + Unpin + Send,
        Self: 'a,
    {
        let s = try_stream! {
            // This pin has nothing to do with the fact that we return a Stream
            // but is needed to access methods like `write_packet`.
            src.write_packet_with_ack(input).await?;
            let bytes = src.read_packet().await?;
            let packet = Self::Output::zvt_parse(&bytes)?;
            // Write the response.
            src.write_packet::<packets::Ack>(&packets::Ack {}).await?;
            yield packet;
        };
        Box::pin(s)
    }
}

/// Registration sequence as defined under 2.1.
///
/// Using the command Registration the ECR can set up different configurations
/// on the PT and also control the current status of the PT.
pub struct Registration {}

/// Response to [packets::Registration] as defined under 2.1.
#[derive(Debug, ZvtEnum)]
pub enum RegistrationResponse {
    CompletionData(packets::CompletionData),
}

impl Sequence for Registration {
    type Input = packets::Registration;
    type Output = RegistrationResponse;
}

/// Read-card sequence as defined under 2.21.
///
/// With this command the PT reads a chip-card/magnet-card and transmits the
/// card-data to the ECR.
pub struct ReadCard;

/// Response to [packets::ReadCard] message as defined in 2.21.
#[derive(Debug, ZvtEnum)]
#[allow(clippy::large_enum_variant)]
pub enum ReadCardResponse {
    IntermediateStatusInformation(packets::IntermediateStatusInformation),
    StatusInformation(packets::StatusInformation),
    Abort(packets::Abort),
}

impl Sequence for ReadCard {
    type Input = packets::ReadCard;
    type Output = ReadCardResponse;

    fn into_stream<'a, Source>(
        input: &'a Self::Input,
        src: &'a mut PacketWriter<Source>,
    ) -> Pin<Box<dyn futures::Stream<Item = Result<Self::Output>> + Send + 'a>>
    where
        Source: AsyncReadExt + AsyncWriteExt + Unpin + Send,
        Self: 'a,
    {
        let s = try_stream! {
            src.write_packet_with_ack(input).await?;
            loop {
                let bytes = src.read_packet().await?;
                let packet = ReadCardResponse::zvt_parse(&bytes)?;
                // Write the response.
                src.write_packet(&packets::Ack {}).await?;

                match packet {
                    ReadCardResponse::StatusInformation(_) | ReadCardResponse::Abort(_) => {
                        yield packet;
                        break;
                    }
                    _ => yield packet,
                }
            }
        };
        Box::pin(s)
    }
}

/// Initialization sequence as defined under 2.18.
///
/// The command forces the PT to send a initialization message to the ECR.
pub struct Initialization;

/// Response to [packets::Initialization] message as defined under 2.18.
#[derive(Debug, ZvtEnum)]
pub enum InitializationResponse {
    /// 2.18.3
    IntermediateStatusInformation(packets::IntermediateStatusInformation),
    /// 2.18.4
    PrintLine(packets::PrintLine),
    /// 2.18.4
    PrintTextBlock(packets::PrintTextBlock),
    /// 2.18.5, terminal.
    CompletionData(packets::CompletionData),
    /// 2.18.6, terminal.
    Abort(packets::Abort),
}

impl Sequence for Initialization {
    /// Defined under 2.18.1
    type Input = packets::Initialization;
    type Output = InitializationResponse;

    fn into_stream<'a, Source>(
        input: &'a Self::Input,
        src: &'a mut PacketWriter<Source>,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Output>> + Send + 'a>>
    where
        Source: AsyncReadExt + AsyncWriteExt + Unpin + Send,
        Self: 'a,
    {
        let s = try_stream! {
            // 2.18.1
            src.write_packet_with_ack(input).await?;
            loop {
                let bytes = src.read_packet().await?;
                let response = InitializationResponse::zvt_parse(&bytes)?;

                // Every message requires an Ack.
                src.write_packet(&packets::Ack {}).await?;

                match response {
                    InitializationResponse::CompletionData(_)
                    | InitializationResponse::Abort(_) => {
                        yield response;
                        break;
                    }
                    _ => yield response,
                }
            }
        };
        Box::pin(s)
    }
}

/// Set/Reset the terminal id as defined under 2.45.
///
/// Causes the PT to set or reset the terminal identifier. The command will only
/// be executed, if the turnover storage is empty e.g. after an [EndOfDay]
/// sequence.
pub struct SetTerminalId;

/// Response to [packets::SetTerminalId] message as defined in 2.45.2
#[derive(Debug, ZvtEnum)]
pub enum SetTerminalIdResponse {
    /// 2.45.2, terminal.
    CompletionData(packets::CompletionData),
    /// 2.45.2, terminal.
    Abort(packets::Abort),
}

impl Sequence for SetTerminalId {
    /// 2.45.1
    type Input = packets::SetTerminalId;
    type Output = SetTerminalIdResponse;
}

/// The Reset-terminal sequence, defined under 2.43.
///
/// With this command the ECR causes the PT to restart.
pub struct ResetTerminal;

/// Response to [packets::ResetTerminal] message, defined under 2.43.2.
#[derive(Debug, ZvtEnum)]
pub enum ResetTerminalResponse {
    /// 2.43.2, terminal.
    CompletionData(packets::CompletionData),
}

impl Sequence for ResetTerminal {
    /// 2.43.1
    type Input = packets::ResetTerminal;
    type Output = ResetTerminalResponse;
}

/// Diagnosis sequence, as defined under 2.17.
///
/// With this command the ECR forces the PT to send a diagnostic message to the
/// host.
pub struct Diagnosis;

/// Response to [packets::Diagnosis] message, as defined under 2.17.
#[derive(Debug, ZvtEnum)]
pub enum DiagnosisResponse {
    // 2.17.2 not implemented.
    /// 2.17.3
    IntermediateStatusInformation(packets::IntermediateStatusInformation),
    /// 2.17.4.
    ///
    /// The message definition can be found under 3.4:
    ///
    /// If the PT sends this command to the ECR, the ECR sets its system-time to
    /// the value sent in Data block.
    SetTimeAndDate(packets::SetTimeAndDate),
    /// 2.17.5
    PrintLine(packets::PrintLine),
    /// 2.17.5
    PrintTextBlock(packets::PrintTextBlock),
    /// 2.17.6, terminal.
    CompletionData(packets::CompletionData),
    /// 2.17.6, terminal.
    Abort(packets::Abort),
}

impl Sequence for Diagnosis {
    /// 2.17.1
    type Input = packets::Diagnosis;
    type Output = DiagnosisResponse;

    fn into_stream<'a, Source>(
        input: &'a Self::Input,
        src: &'a mut PacketWriter<Source>,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Output>> + Send + 'a>>
    where
        Source: AsyncReadExt + AsyncWriteExt + Unpin + Send,
        Self: 'a,
    {
        let s = try_stream! {
            // 2.18.1
            src.write_packet_with_ack(input).await?;
            loop {
                let bytes = src.read_packet().await?;
                let response = DiagnosisResponse::zvt_parse(&bytes)?;

                // Every message requires an Ack.
                src.write_packet(&packets::Ack {}).await?;

                match response {
                    DiagnosisResponse::CompletionData(_)
                    | DiagnosisResponse::Abort(_) => {
                        yield response;
                        break;
                    }
                    _ => yield response,
                }
            }
        };
        Box::pin(s)
    }
}

/// End-of-day sequence as defined under 2.16.
///
/// With this command the ECR induces the PT to transfer the stored turnover to
/// the host.
pub struct EndOfDay;

/// Response to [packets::EndOfDay] message as defined under 2.16.
#[derive(Debug, ZvtEnum)]
#[allow(clippy::large_enum_variant)]
pub enum EndOfDayResponse {
    // TODO(ddo) 2.16.2 not implemented
    /// 2.16.3
    IntermediateStatusInformation(packets::IntermediateStatusInformation),
    /// 2.16.4
    StatusInformation(packets::StatusInformation),
    /// 2.16.5
    PrintLine(packets::PrintLine),
    /// 2.16.5
    PrintTextBlock(packets::PrintTextBlock),
    /// 2.16.6
    CompletionData(packets::CompletionData),
    /// 2.16.6
    ///
    /// Our data shows that if there is a pending transaction, the PT rather
    /// returns [packets::PartialReversalAbort] over [packets::Abort].
    Abort(packets::PartialReversalAbort),
}

impl Sequence for EndOfDay {
    /// 2.16.1
    type Input = packets::EndOfDay;
    type Output = EndOfDayResponse;

    fn into_stream<'a, Source>(
        input: &'a Self::Input,
        src: &'a mut PacketWriter<Source>,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Output>> + Send + 'a>>
    where
        Source: AsyncReadExt + AsyncWriteExt + Unpin + Send,
        Self: 'a,
    {
        let s = try_stream! {
            // 2.16.1
            src.write_packet_with_ack(input).await?;

            loop {
                let bytes = src.read_packet().await?;
                let packet = EndOfDayResponse::zvt_parse(&bytes)?;

                // Write the response.
                src.write_packet(&packets::Ack {}).await?;
                match packet {
                    EndOfDayResponse::CompletionData(_) | EndOfDayResponse::Abort(_) => {
                        yield packet;
                        break;
                    }
                    _ => yield packet,
                }
            }
        };
        Box::pin(s)
    }
}

/// Reservation sequence as defined under 2.8.
///
/// The ECR requests PT to reserve a certain payment-amount. This is necessary
/// when the final payment-amount is only established after the authorization.
/// In this case the ECR firstly reserves an amount (= maximal Possible
/// payment-amount) and then, after the sales-process, releases the unused
/// amount via a [PartialReversal] or Book Total (06 24, not implemented).
pub struct Reservation;

/// Response to [packets::Reservation] message, as defined under 2.8.
///
/// The response is the same as for Authorization, defined in chapter 2.1.
#[derive(Debug, ZvtEnum)]
#[allow(clippy::large_enum_variant)]
pub enum AuthorizationResponse {
    /// 2.2.4
    IntermediateStatusInformation(packets::IntermediateStatusInformation),
    // 2.2.5 produces no message.
    /// 2.2.6
    StatusInformation(packets::StatusInformation),
    /// 2.2.7
    PrintLine(packets::PrintLine),
    /// 2.2.7
    PrintTextBlock(packets::PrintTextBlock),
    // 2.2.8 produces no message.
    /// 2.2.9
    CompletionData(packets::CompletionData),
    /// 2.2.9
    Abort(packets::Abort),
}

impl Sequence for Reservation {
    type Input = packets::Reservation;
    type Output = AuthorizationResponse;

    fn into_stream<'a, Source>(
        input: &'a Self::Input,
        src: &'a mut PacketWriter<Source>,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Output>> + Send + 'a>>
    where
        Source: AsyncReadExt + AsyncWriteExt + Unpin + Send,
        Self: 'a,
    {
        let s = try_stream! {
            // 2.8
            src.write_packet_with_ack(input).await?;

            loop {
                let bytes = src.read_packet().await?;
                let packet = AuthorizationResponse::zvt_parse(&bytes)?;
                src.write_packet(&packets::Ack {}).await?;
                match packet {
                    AuthorizationResponse::CompletionData(_) | AuthorizationResponse::Abort(_) => {
                        yield packet;
                        break;
                    }
                    _ => yield packet,
                }
            }
        };
        Box::pin(s)
    }
}

/// Partial reversal sequence as defined under 2.10.
///
/// This command executes a Partial-Reversal for a [Reservation] to release the
/// unused amount of the reservation. The Partial-Reversal is only carried-out
/// if a Pre-Authorization with the passed receipt number (returned in
/// [packets::StatusInformation::receipt_no] after running [Reservation]) is
/// found in the turnover-records.
pub struct PartialReversal;

/// Response to [packets::PartialReversal] message as defined in 2.10.
///
/// The output is identical to Reversal (06 30), which has the same output
/// as Authorization (06 01). The only difference is the 2.10.1 case, where
/// we return the currently active transactions.
#[derive(Debug, ZvtEnum)]
#[allow(clippy::large_enum_variant)]
pub enum PartialReversalResponse {
    /// 2.2.4
    IntermediateStatusInformation(packets::IntermediateStatusInformation),
    // 2.2.5 produces no message.
    /// 2.2.6
    StatusInformation(packets::StatusInformation),
    /// 2.2.7
    PrintLine(packets::PrintLine),
    /// 2.2.7
    PrintTextBlock(packets::PrintTextBlock),
    // 2.2.8 produces no message.
    /// 2.2.9
    CompletionData(packets::CompletionData),
    /// 2.2.9 and 2.10.1 Abort messages.
    ///
    /// Note: The [packets::Abort] message is a valid subset of
    /// [packets::PartialReversalAbort].
    PartialReversalAbort(packets::PartialReversalAbort),
}

impl Sequence for PartialReversal {
    /// 2.10
    type Input = packets::PartialReversal;
    type Output = PartialReversalResponse;

    fn into_stream<'a, Source>(
        input: &'a Self::Input,
        src: &'a mut PacketWriter<Source>,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Output>> + Send + 'a>>
    where
        Source: AsyncReadExt + AsyncWriteExt + Unpin + Send,
        Self: 'a,
    {
        let s = try_stream! {
            src.write_packet_with_ack(input).await?;

            loop {
                let bytes = src.read_packet().await?;
                let packet = PartialReversalResponse::zvt_parse(&bytes)?;
                src.write_packet(&packets::Ack {}).await?;
                match packet {
                    PartialReversalResponse::CompletionData(_)
                    | PartialReversalResponse::PartialReversalAbort(_) => {
                        yield packet;
                        break;
                    }
                    _ => yield packet,
                }
            }
        };
        Box::pin(s)
    }
}

/// Pre-Auth-Reversal sequence as defined in 2.14
///
/// This command executes a reversal of a [Reservation] in the case of a
/// null-filling. The sequence is identical to the [PartialReversal].
pub struct PreAuthReversal;

impl Sequence for PreAuthReversal {
    type Input = packets::PreAuthReversal;
    type Output = PartialReversalResponse;

    fn into_stream<'a, Source>(
        input: &'a Self::Input,
        src: &'a mut PacketWriter<Source>,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Output>> + Send + 'a>>
    where
        Source: AsyncReadExt + AsyncWriteExt + Unpin + Send,
        Self: 'a,
    {
        let s = try_stream! {
            src.write_packet_with_ack(input).await?;

            loop {
                let bytes = src.read_packet().await?;
                let packet = PartialReversalResponse::zvt_parse(&bytes)?;
                src.write_packet(&packets::Ack {}).await?;
                match packet {
                    PartialReversalResponse::CompletionData(_)
                    | PartialReversalResponse::PartialReversalAbort(_) => {
                        yield packet;
                        break;
                    }
                    _ => yield packet,
                }
            }
        };
        Box::pin(s)
    }
}

/// Prints the system information as defined in 2.44.
///
/// With this command the ECR causes the PT to print its system information to
/// the print target defined in [Registration].
pub struct PrintSystemConfiguration;

/// Response to [packets::PrintSystemConfiguration] message, as defined in 2.44.
#[derive(Debug, ZvtEnum)]
pub enum PrintSystemConfigurationResponse {
    /// 2.44.2
    PrintLine(packets::PrintLine),
    /// 2.44.2
    PrintTextBlock(packets::PrintTextBlock),
    /// 2.44.3
    CompletionData(packets::CompletionData),
}

impl Sequence for PrintSystemConfiguration {
    type Input = packets::PrintSystemConfiguration;
    type Output = PrintSystemConfigurationResponse;

    fn into_stream<'a, Source>(
        input: &'a Self::Input,
        src: &'a mut PacketWriter<Source>,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Output>> + Send + 'a>>
    where
        Source: AsyncReadExt + AsyncWriteExt + Unpin + Send,
        Self: 'a,
    {
        let s = try_stream! {
            src.write_packet_with_ack(input).await?;

            loop {
                let bytes = src.read_packet().await?;
                let packet = PrintSystemConfigurationResponse::zvt_parse(&bytes)?;
                src.write_packet(&packets::Ack {}).await?;
                match packet {
                    PrintSystemConfigurationResponse::CompletionData(_) => {
                        yield packet;
                        break;
                    }
                    _ => yield packet,
                }
            }
        };
        Box::pin(s)
    }
}

/// Sets the language of the PT as defined in 2.36.
///
/// With this command the ECR selects the language in the PT.
pub struct SelectLanguage;

/// Response to [packets::SelectLanguage] message as defined in 2.36.
#[derive(Debug, ZvtEnum)]
pub enum SelectLanguageResponse {
    CompletionData(packets::CompletionData),
}

impl Sequence for SelectLanguage {
    type Input = packets::SelectLanguage;
    type Output = SelectLanguageResponse;
}

/// Status enquiry sequence as defined in 2.55.
///
/// With this command the ECR can request the Status of the PT allow the PT to
/// carry out time-controlled events (e.g. OPT-actions or End-of-Day). To allow
/// time-controlled events on the PT to be executed punctually the ECR should
/// send Status-Enquiries as often as possible (every minute or more frequently).
pub struct StatusEnquiry;

/// Response to [packets::StatusEnquiry] message as defined in 2.55.
#[derive(Debug, ZvtEnum)]
pub enum StatusEnquiryResponse {
    // 2.55.2 not supported
    /// 2.55.3
    IntermediateStatusInformation(packets::IntermediateStatusInformation),
    /// 2.55.4
    PrintLine(packets::PrintLine),
    /// 2.55.4
    PrintTextBlock(packets::PrintTextBlock),
    /// 2.55.5
    CompletionData(packets::CompletionData),
}

impl Sequence for StatusEnquiry {
    type Input = packets::StatusEnquiry;
    type Output = StatusEnquiryResponse;

    fn into_stream<'a, Source>(
        input: &'a Self::Input,
        src: &'a mut PacketWriter<Source>,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Output>> + Send + 'a>>
    where
        Source: AsyncReadExt + AsyncWriteExt + Unpin + Send,
        Self: 'a,
    {
        let s = try_stream! {
            src.write_packet_with_ack(input).await?;

            loop {
                let bytes = src.read_packet().await?;
                let packet = StatusEnquiryResponse::zvt_parse(&bytes)?;
                src.write_packet(&packets::Ack {}).await?;
                match packet {
                    StatusEnquiryResponse::CompletionData(_) => {
                        yield packet;
                        break;
                    }
                    _ => yield packet,
                }
            }
        };
        Box::pin(s)
    }
}
