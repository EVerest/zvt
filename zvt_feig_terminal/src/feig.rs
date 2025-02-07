use crate::config::Config;
use crate::stream::{ResetSequence, TcpStream};
use anyhow::{anyhow, bail, Result};
use log::{info, warn};
use num_traits::FromPrimitive;
use std::collections::HashMap;
use std::time::Duration;
use tokio_stream::StreamExt;
use zvt::{constants, feig, packets, sequences};

/// The card information returned from read-card.
pub enum CardInfo {
    /// Indicatates if we've received a bank card.
    Bank,

    /// Indicates if we've received a member ship card. The stirng is our tag-id.
    MembershipCard(String),
}

/// Summary after a transaction.
pub struct TransactionSummary {
    /// The terminal-id of the payment terminal.
    pub terminal_id: Option<String>,

    /// The amount billed in the transaction, in Cents.
    pub amount: Option<u64>,

    /// The trace number identifying the transaction.
    pub trace_number: Option<u64>,

    /// The date of the transaction, 4 numerical letters, e.g. -> '0517' means
    /// the payment was made on May 17th.
    pub date: Option<String>,

    /// The time of the payment - 6 numerical letters, e.g. '134530' means the
    /// payment was made at 13:45:30.
    pub time: Option<String>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("Unexpected packet")]
    UnexpectedPacket,

    #[error("Active transaction: {0}")]
    ActiveTransaction(String),

    #[error("No Card presented")]
    NoCardPresented,

    #[error("Unknown token: {0}")]
    UnknownToken(String),

    #[error("The presented card requires a PIN entry.")]
    NeedsPinEntry,
}

/// Default card type, which is chip-card, as defined in Table 6.
const CARD_TYPE: Option<u8> = Some(0x10);

/// Default value for reading control.
///
/// See Tlv tag 0x1f15 for the documentation.
const SHORT_CARD_READING_CONTROL: Option<u8> = Some(0xd0);

/// Default value for allowed card types.
///
/// See Tlv tag 0x1f60 for documentation.
const ALLOWED_CARDS: Option<u8> = Some(0x07);

/// Default dialog control for reading the card.
///
/// The values are defined in Table 7 - having only the choice between 1 and 0;
/// However, the value 2 can silence the beeps.
const DIALOG_CONTROL: Option<u8> = Some(0x02);

/// The default payment type, defined in table 4.
///
/// Payment type according to PTs decision excluding `GeldKarte`.
const PAYMENT_TYPE: Option<u8> = Some(0x40);

/// Identifier for the individual reference number.
///
/// This identifier is used in BMP60 when transmitting the individual reference
/// number to the host. It allows us to tack payments at Lavego.
const BMP_PREFIX: &str = "AC";

#[derive(Default)]
pub struct Feig {
    socket: TcpStream,
    /// Map of active transactions to their receipt-number.
    transactions: HashMap<String, usize>,

    /// Maximum number of concurrent transactions.
    transactions_max_num: usize,
}

impl Feig {
    pub async fn new(config: Config) -> Result<Self> {
        let transactions_max_num = config.transactions_max_num;
        let mut this = Self {
            socket: TcpStream::new(config)?,
            transactions: HashMap::new(),
            transactions_max_num,
        };

        // Ignore the errors from configure (call fails if e.x. the terminal id is
        // invalid)
        let _ = this.configure().await;
        Ok(this)
    }

    /// Returns the system information of the feig-terminal.
    async fn get_system_info(
        &mut self,
    ) -> Result<feig::packets::CVendFunctionsEnhancedSystemInformationCompletion> {
        let request = feig::packets::CVendFunctions {
            password: None,
            instr: 1,
        };
        let mut stream = feig::sequences::GetSystemInfo::into_stream(request, &mut self.socket);
        while let Some(response) = stream.next().await {
            let Ok(response) = response else {
                continue;
            };
            match response {
                feig::sequences::GetSystemInfoResponse::CVendFunctionsEnhancedSystemInformationCompletion(packet) => {
                    return Ok(packet)
                },
                feig::sequences::GetSystemInfoResponse::Abort(packet) => bail!(zvt::ZVTError::Aborted(packet.error))
            }
        }
        bail!(zvt::ZVTError::IncompleteData)
    }

    /// Sets the terminal id.
    ///
    /// Function does nothing if the feig-terminal has already the desired
    /// terminal-id.
    ///
    /// Returns true if a new TID was set, and false if the requested TID is
    /// already set to the terminal
    async fn set_terminal_id(&mut self) -> Result<bool> {
        let system_info = self.get_system_info().await?;
        let config = self.socket.config();

        // Set the terminal id if required.
        if config.terminal_id == system_info.terminal_id {
            info!("Terminal id already up-to-date");
            return Ok(false);
        }

        // Sadly the terminal_id is a int, but we communicate it as a string...
        let terminal_id = config.terminal_id.parse::<usize>()?;
        let request = packets::SetTerminalId {
            password: config.feig_config.password,
            terminal_id: Some(terminal_id),
        };

        info!("Updating the terminal_id to {terminal_id}");

        let mut stream = sequences::SetTerminalId::into_stream(request, &mut self.socket);
        while let Some(response) = stream.next().await {
            let Ok(response) = response else {
                continue;
            };
            match response {
                sequences::SetTerminalIdResponse::CompletionData(_) => return Ok(true),
                sequences::SetTerminalIdResponse::Abort(data) => {
                    bail!(zvt::ZVTError::Aborted(data.error))
                }
            }
        }

        bail!(zvt::ZVTError::IncompleteData)
    }

    async fn run_diagnosis(&mut self, diagnosis: packets::DiagnosisType) -> Result<()> {
        let request = packets::Diagnosis {
            tlv: Some(packets::tlv::Diagnosis {
                diagnosis_type: Some(diagnosis as u8),
            }),
        };

        let mut stream = sequences::Diagnosis::into_stream(request, &mut self.socket);
        while let Some(response) = stream.next().await {
            use sequences::DiagnosisResponse::*;
            match response? {
                SetTimeAndDate(data) => log::debug!("{data:#?}"),
                PrintLine(data) => log::debug!("{}", data.text),
                PrintTextBlock(data) => log::debug!("{data:#?}"),
                IntermediateStatusInformation(_) | CompletionData(_) => (),
                Abort(_) => bail!("Received Abort."),
            }
        }
        Ok(())
    }

    /// Initializes the feig-terminal.
    async fn initialize(&mut self) -> Result<()> {
        let password = self.socket.config().feig_config.password;
        let request = packets::Initialization { password };
        let mut stream = sequences::Initialization::into_stream(request, &mut self.socket);
        while let Some(response) = stream.next().await {
            use sequences::InitializationResponse::*;
            let Ok(response) = response else {
                continue;
            };
            match response {
                IntermediateStatusInformation(_) => (),
                PrintLine(data) => log::info!("{}", data.text),
                PrintTextBlock(data) => log::info!("{data:#?}"),
                CompletionData(_) => return Ok(()),
                Abort(data) => {
                    bail!(zvt::ZVTError::Aborted(data.error))
                }
            }
        }

        bail!(zvt::ZVTError::IncompleteData)
    }

    /// Returns the pending transaction.
    ///
    /// We return the first possible pending transactions. Right now we just
    /// check for one.
    async fn get_pending(&mut self) -> Result<Option<usize>> {
        let request = packets::PartialReversal {
            receipt_no: Some(0xFFFF),
            ..packets::PartialReversal::default()
        };

        let mut stream = sequences::PartialReversal::into_stream(request, &mut self.socket);
        while let Some(response) = stream.next().await {
            let Ok(response) = response else {
                continue;
            };
            match response {
                sequences::PartialReversalResponse::PartialReversalAbort(data) => {
                    // The 0xFFFF means no pending transactions.
                    let Some(receipt_no) = data.receipt_no else {
                        return Ok(None);
                    };

                    if receipt_no == 0xFFFF {
                        return Ok(None);
                    }
                    return Ok(Some(receipt_no));
                }
                _ => bail!(Error::UnexpectedPacket),
            }
        }

        bail!(zvt::ZVTError::IncompleteData)
    }

    /// Cancels all pending transactions.
    async fn cancel_pending(&mut self) -> Result<()> {
        self.transactions.clear();

        while let Some(receipt_no) = self.get_pending().await? {
            self.cancel_transaction_by_receipt_no(receipt_no).await?;
        }
        Ok(())
    }

    /// Runs an end-of-day job.
    ///
    /// Will first cancel all currently pending transactions and then run an
    /// end of day job. Caution: Calling this will wipe all ongoing
    /// transactions.
    async fn end_of_day(&mut self) -> Result<()> {
        // Cancel all possible pending transactions.
        self.cancel_pending().await?;

        let password = self.socket.config().feig_config.password;
        let request = packets::EndOfDay { password };
        let mut stream = sequences::EndOfDay::into_stream(request, &mut self.socket);
        // Note: The timeout might be too little as this needs a call to the
        // PT's host.
        while let Some(response) = stream.next().await {
            let Ok(response) = response else {
                continue;
            };
            match response {
                sequences::EndOfDayResponse::CompletionData(_) => return Ok(()),
                sequences::EndOfDayResponse::Abort(data) => {
                    // If the payment terminal was not configured it may return
                    // 'receiver not ready' - in this case we'll ignore the
                    // error.
                    if data.error == constants::ErrorMessages::ReceiverNotReady as u8 {
                        warn!("End-of-Day: Terminal not ready");
                        return Ok(());
                    }

                    bail!(zvt::ZVTError::Aborted(data.error))
                }
                _ => {}
            }
        }

        bail!(zvt::ZVTError::IncompleteData)
    }

    /// Initializes the connection.
    ///
    /// We're doing the following
    /// * Set the terminal id if required.
    /// * Initialize the terminal.
    /// * Run end-of-day job.
    pub async fn configure(&mut self) -> Result<()> {
        let tid_changed = self.set_terminal_id().await?;
        if tid_changed {
            self.run_diagnosis(packets::DiagnosisType::EmvConfiguration)
                .await?;
        }
        self.initialize().await?;
        self.end_of_day().await?;

        Ok(())
    }

    /// Reads the card data
    ///
    /// The call will either return some [CardInfo] or [None] - if there is no
    /// card presented during the specified [config.read_card_timeout].
    pub async fn read_card(&mut self) -> Result<CardInfo> {
        let timeout_sec = self.socket.config().feig_config.read_card_timeout;
        let request = packets::ReadCard {
            timeout_sec,
            card_type: CARD_TYPE,
            dialog_control: DIALOG_CONTROL,
            tlv: Some(packets::tlv::ReadCard {
                card_reading_control: SHORT_CARD_READING_CONTROL,
                card_type: ALLOWED_CARDS,
            }),
        };

        let retry = futures::stream::repeat(())
            .throttle(Duration::from_secs(2))
            .take(20);
        let mut stream = sequences::ReadCard::into_stream_with_retry(
            request,
            &mut self.socket,
            retry,
            Duration::from_secs((timeout_sec + 2) as u64),
        );
        let mut card_info = None;
        while let Some(response) = stream.next().await {
            let Ok(response) = response else {
                continue;
            };
            match response {
                sequences::ReadCardResponse::Abort(data) => {
                    use zvt::constants::ErrorMessages::*;

                    let err = zvt::constants::ErrorMessages::from_u8(data.error)
                        .ok_or(anyhow!("Unknown error code: 0x{:X}", data.error))?;
                    match err {
                        AbortViaTimeoutOrAbortKey => {
                            // If there is no card to read, we will receive a timeout
                            // error.
                            bail!(Error::NoCardPresented)
                        }
                        other => {
                            log::info!("Unhandled error: {other}");
                            bail!(other)
                        }
                    }
                }
                sequences::ReadCardResponse::StatusInformation(data) => {
                    // Retrieve the card information.
                    let tlv = data.tlv.ok_or(zvt::ZVTError::IncompleteData)?;
                    if !tlv.subs.is_empty() {
                        let subs = &tlv.subs[0];
                        if subs.application_id.is_some() {
                            card_info = Some(CardInfo::Bank);
                        } else {
                            bail!("Unknown card type")
                        }
                    } else if let Some(mut uuid) = tlv.uuid {
                        uuid = uuid.to_uppercase();
                        if uuid.len() > 14 {
                            uuid = uuid[uuid.len() - 14..].to_string();
                            uuid = uuid.strip_prefix("000000").unwrap_or(&uuid).to_string();
                        }

                        card_info = Some(CardInfo::MembershipCard(uuid));
                    } else {
                        bail!(zvt::ZVTError::IncompleteData)
                    }
                }
                _ => {}
            }
        }

        card_info.ok_or(zvt::ZVTError::IncompleteData.into())
    }

    /// Begins a transaction.
    ///
    /// The transaction must be finished with either [Feig::cancel_transaction]
    /// or [Feig::commit_transaction]. The given `transaction` must be used
    /// for both follow up functions. The method returns an error if the
    /// maximum number of currently active transactions has been reached or if
    /// the [Transaction::token] is already in use.
    ///
    /// Under the hood the method maps to [sequences::Reservation].
    ///
    /// # Arguments
    /// * `token` - The token to identify the transaction with.
    pub async fn begin_transaction(&mut self, token: &str) -> Result<()> {
        if self.transactions.len() == self.transactions_max_num {
            bail!(Error::ActiveTransaction(format!(
                "Maximum number of transactions reached: {}",
                self.transactions_max_num
            )))
        }

        if self.transactions.contains_key(token) {
            bail!(Error::ActiveTransaction("Token already in use".to_string()))
        }

        let config = self.socket.config();
        let request = packets::Reservation {
            currency: Some(config.feig_config.currency),
            amount: Some(config.feig_config.pre_authorization_amount),
            payment_type: PAYMENT_TYPE,
            tlv: Some(packets::tlv::PreAuthData {
                bmp_data: Some(packets::tlv::Bmp60 {
                    bmp_prefix: BMP_PREFIX.to_string(),
                    bmp_data: token.to_string(),
                }),
            }),
            ..packets::Reservation::default()
        };

        let mut stream = sequences::Reservation::into_stream(request, &mut self.socket);
        let mut receipt_no = None;
        while let Some(response) = stream.next().await {
            let Ok(response) = response else {
                continue;
            };
            match response {
                sequences::AuthorizationResponse::Abort(data) => {
                    use zvt::constants::ErrorMessages::*;

                    let err = zvt::constants::ErrorMessages::from_u8(data.error)
                        .ok_or(anyhow!("Unknown error code: 0x{:X}", data.error))?;
                    match err {
                        NecessaryDeviceNotPresentOrDefective => {
                            bail!(Error::NeedsPinEntry)
                        }
                        _ => bail!(zvt::ZVTError::Aborted(data.error)),
                    }
                }
                sequences::AuthorizationResponse::StatusInformation(data) => {
                    // Only overwrite the receipt_no if it is contained in the
                    // message.
                    if let Some(inner) = data.receipt_no {
                        receipt_no = Some(inner);
                    }
                }
                _ => {}
            }
        }

        match receipt_no {
            None => bail!(zvt::ZVTError::IncompleteData),
            Some(receipt_no) => {
                self.transactions.insert(token.to_string(), receipt_no);
                Ok(())
            }
        }
    }

    async fn cancel_transaction_by_receipt_no(&mut self, receipt_no: usize) -> Result<()> {
        let config = self.socket.config();
        let request = packets::PreAuthReversal {
            payment_type: PAYMENT_TYPE,
            currency: Some(config.feig_config.currency),
            receipt_no: Some(receipt_no),
        };

        let mut stream = sequences::PreAuthReversal::into_stream(request, &mut self.socket);
        while let Some(response) = stream.next().await {
            let Ok(response) = response else {
                continue;
            };
            match response {
                sequences::PartialReversalResponse::CompletionData(_) => return Ok(()),
                sequences::PartialReversalResponse::PartialReversalAbort(data) => {
                    bail!(zvt::ZVTError::Aborted(data.error))
                }
                _ => {}
            }
        }

        bail!(zvt::ZVTError::IncompleteData)
    }

    /// Cancels a transaction.
    ///
    /// The transaction must be started with [Feig::begin_transaction] and the
    /// argument must contain a [Transaction::token] matching the token from
    /// [Feig::begin_transaction]. The method fails if the `token` is unknown.
    ///
    /// # Arguments
    /// * `token` - The token the transaction is identified with.
    pub async fn cancel_transaction(&mut self, token: &str) -> Result<()> {
        // Check if the transaction is known to us.
        let removed = self.transactions.remove(token);
        match removed {
            None => bail!(Error::UnknownToken(token.to_string())),
            Some(receipt_no) => {
                self.cancel_transaction_by_receipt_no(receipt_no).await?;

                // Run end of day if we don't have any pending transactions
                if self.transactions.is_empty() {
                    self.end_of_day().await?;
                }
                Ok(())
            }
        }
    }

    /// Commits a transaction.
    ///
    /// The transaction must be started with [Feig::begin_transaction] and the
    /// argument must contain a [Transaction::token] matching the token from
    /// [Feig::begin_transaction]. The method fails if the `token` is unknown.
    ///
    /// # Arguments
    /// * `token` - The token under which the transaction is known.
    /// * `amount` - The amount in fractional monetary unit.
    ///
    /// # Returns
    /// The summary of the transaction.
    pub async fn commit_transaction(
        &mut self,
        token: &str,
        amount: u64,
    ) -> Result<TransactionSummary> {
        let removed = self.transactions.remove(token);
        if removed.is_none() {
            bail!(Error::UnknownToken(token.to_string()));
        }

        let config = self.socket.config();
        let reversal_amount = config
            .feig_config
            .pre_authorization_amount
            .saturating_sub(amount as usize);

        let request = packets::PartialReversal {
            receipt_no: Some(removed.unwrap()),
            currency: Some(config.feig_config.currency),
            amount: Some(reversal_amount),
            payment_type: PAYMENT_TYPE,
            tlv: Some(packets::tlv::PreAuthData {
                bmp_data: Some(packets::tlv::Bmp60 {
                    bmp_prefix: BMP_PREFIX.to_string(),
                    bmp_data: token.to_string(),
                }),
            }),
        };

        let mut stream = sequences::PartialReversal::into_stream(request, &mut self.socket);
        let mut status_information = None;
        while let Some(response) = stream.next().await {
            use sequences::PartialReversalResponse::*;
            let Ok(response) = response else {
                continue;
            };
            match response {
                IntermediateStatusInformation(_) | CompletionData(_) => (),
                PrintLine(data) => log::info!("{}", data.text),
                PrintTextBlock(data) => log::info!("{data:#?}"),
                StatusInformation(data) => status_information = Some(data),
                PartialReversalAbort(data) => bail!(zvt::ZVTError::Aborted(data.error)),
            }
        }
        drop(stream);

        if self.transactions.is_empty() {
            self.end_of_day().await?;
        }

        let status_information = status_information.ok_or(zvt::ZVTError::IncompleteData)?;
        Ok(TransactionSummary {
            terminal_id: status_information
                .terminal_id
                .map(|inner| inner.to_string()),
            date: status_information.date.map(|n| format!("{:04}", n)),
            time: status_information.time.map(|n| format!("{:06}", n)),
            amount: status_information.amount.map(|inner| inner as u64),
            trace_number: status_information.trace_number.map(|inner| inner as u64),
        })
    }
}
