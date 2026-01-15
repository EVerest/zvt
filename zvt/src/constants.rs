use num_derive::FromPrimitive;
use thiserror::Error;

/// Messages as defined under chapter 10.
#[derive(Debug, PartialEq, FromPrimitive, Clone, Copy, Error)]
#[repr(u8)]
pub enum ErrorMessages {
    #[cfg(feature = "with_lavego_error_codes")]
    #[error("declined, referred voice authorization possible")]
    Declined = 0x02,
    #[cfg(feature = "with_lavego_error_codes")]
    #[error("declined")]
    Declined = 0x05,
    #[cfg(feature = "with_lavego_error_codes")]
    #[error("contactless transaction count exceeded")]
    ContactlessTransactionCountExceeded = 0x13,
    #[cfg(feature = "with_lavego_error_codes")]
    #[error("card expired")]
    CardExpiredLavego = 0x21,
    #[cfg(feature = "with_lavego_error_codes")]
    #[error("PIN entry required")]
    PinEntryRequiredx33 = 0x33,
    #[cfg(feature = "with_lavego_error_codes")]
    #[error("TID not activated")]
    TidNotActivated = 0x3a,
    #[cfg(feature = "with_lavego_error_codes")]
    #[error("PIN entry required")]
    PinEntryRequiredx3d = 0x3d,
    #[cfg(feature = "with_lavego_error_codes")]
    #[error("PIN entry required")]
    PinEntryRequiredx41 = 0x41,
    #[error("card not readable (LRC-/parity-error)")]
    CardNotReadable = 0x64,
    #[error("card-data not present (neither track-data nor chip found)")]
    CardDataNotPresent = 0x65,
    #[error("processing-error (also for problems with card-reader mechanism)")]
    ProcessingError = 0x66,
    #[error("function not permitted for ec- and Maestro-cards")]
    FunctionNotPermittedForEcAndMaestroCards = 0x67,
    #[error("function not permitted for credit- and tank-cards")]
    FunctionNotPermittedForCreditAndTankCards = 0x68,
    #[error("turnover-file full")]
    TurnoverFileFull = 0x6a,
    #[error("function deactivated (PT not registered)")]
    FunctionDeactivated = 0x6b,
    #[error("abort via timeout or abort-key")]
    AbortViaTimeoutOrAbortKey = 0x6c,
    #[error("card in blocked-list (response to command 06 E4)")]
    CardInBlockedList = 0x6e,
    #[error("wrong currency")]
    WrongCurrency = 0x6f,
    #[error("credit not sufficient (chip-card)")]
    CreditNotSufficient = 0x71,
    #[error("chip error")]
    ChipError = 0x72,
    #[error("card-data incorrect (e.g. country-key check, checksum-error)")]
    CardDataIncorrect = 0x73,
    #[error("DUKPT engine exhausted")]
    DukptEngineExhausted = 0x74,
    #[error("text not authentic")]
    TextNotAuthentic = 0x75,
    #[error("PAN not in white list")]
    PanNotInWhiteList = 0x76,
    #[error("end-of-day batch not possible")]
    EndOfDayBatchNotPossible = 0x77,
    #[error("card expired")]
    CardExpired = 0x78,
    #[error("card not yet valid")]
    CardNotYetValid = 0x79,
    #[error("card unknown")]
    CardUnknown = 0x7a,
    #[error("fallback to magnetic stripe for girocard not possible")]
    FallbackToMagneticStripeNotPossibleForGiroCard1 = 0x7b,
    #[error("fallback to magnetic stripe not possible (used for non girocard cards)")]
    FallbackToMagneticStripeNotPossibleForNonGiroCard = 0x7c,
    #[error("communication error (communication module does not answer or is not present)")]
    CommunicationError = 0x7d,
    #[error(
        "fallback to magnetic stripe not possible, debit advice possible (used only for giro-card)"
    )]
    FallbackToMagneticStripeNotPossibleForGiroCard2 = 0x7e,
    #[error("function not possible")]
    FunctionNotPossible = 0x83,
    #[error("key missing")]
    KeyMissing = 0x85,
    #[error("PIN-pad defective")]
    PinPadDefective1 = 0x89,
    #[error("ZVT protocol error. e. g. parsing error, mandatory message element missing")]
    ZvtProtocolError = 0x9a,
    #[error("error from dial-up/communication fault")]
    ErrorFromDialUp = 0x9b,
    #[error("please wait")]
    PleaseWait = 0x9c,
    #[error("receiver not ready")]
    ReceiverNotReady = 0xa0,
    #[error("remote station does not respond")]
    RemoteStationDoesNotRespond = 0xa1,
    #[error("no connection")]
    NoConnection = 0xa3,
    #[error("submission of Geldkarte not possible")]
    SubmissionOfGeldkarteNotPossible = 0xa4,
    #[error("function not allowed due to PCI-DSS/P2PE rules")]
    FunctionNotAllowedDueToPciDss = 0xa5,
    #[error("memory full")]
    MemoryFull = 0xb1,
    #[error("merchant-journal full")]
    MerchantJournalFull = 0xb2,
    #[error("already reversed")]
    AlreadyReversed = 0xb4,
    #[error("reversal not possible")]
    ReversalNotPossible = 0xb5,
    #[error("pre-authorization incorrect (amount too high) or amount wrong")]
    PreAuthorizationIncorrect = 0xb7,
    #[error("error pre-authorization")]
    ErrorPreAuthorization = 0xb8,
    #[error("voltage supply to low (external power supply)")]
    VoltageSupplyToLow = 0xbf,
    #[error("card locking mechanism defective")]
    CardLockingMechanismDefective = 0xc0,
    #[error("merchant-card locked")]
    MerchantCardLocked = 0xc1,
    #[error("diagnosis required")]
    DiagnosisRequired = 0xc2,
    #[error("maximum amount exceeded")]
    MaximumAmountExceeded = 0xc3,
    #[error("card-profile invalid. New card-profiles must be loaded.")]
    CardProfileInvalid = 0xc4,
    #[error("payment method not supported")]
    PaymentMethodNotSupported = 0xc5,
    #[error("currency not applicable")]
    CurrencyNotApplicable = 0xc6,
    #[error("amount too small")]
    AmountTooSmall = 0xc8,
    #[error("max. transaction-amount too small")]
    MaxTransactionAmountTooSmall = 0xc9,
    #[error("function only allowed in EURO")]
    FunctionOnlyAllowedInEuro = 0xcb,
    #[error("printer not ready")]
    PrinterNotReady = 0xcc,
    #[error("Cashback not possible")]
    CashbackNotPossible = 0xcd,
    #[error("function not permitted for service-cards/bank-customer-cards")]
    FunctionNotPermittedForServiceCards = 0xd2,
    #[error("card inserted")]
    CardInserted = 0xdc,
    #[error("error during card-eject (for motor-insertion reader)")]
    ErrorDuringCardEject = 0xdd,
    #[error("error during card-insertion (for motor-insertion reader)")]
    ErrorDuringCardInsertion = 0xde,
    #[error("remote-maintenance activated")]
    RemoteMaintenanceActivated = 0xe0,
    #[error("card-reader does not answer / card-reader defective")]
    CardReaderDoesNotAnswer = 0xe2,
    #[error("shutter closed")]
    ShutterClosed = 0xe3,
    #[error("Terminal activation required")]
    TerminalActivationRequired = 0xe4,
    #[error("min. one goods-group not found")]
    MinOneGoodsGroupNotFound = 0xe7,
    #[error("no goods-groups-table loaded")]
    NoGoodsGroupsTableLoaded = 0xe8,
    #[error("restriction-code not permitted")]
    RestrictionCodeNotPermitted = 0xe9,
    #[error("card-code not permitted (e.g. card not activated via Diagnosis)")]
    CardCodeNotPermitted = 0xea,
    #[error("function not executable (PIN-algorithm unknown)")]
    FunctionNotExecutable = 0xeb,
    #[error("PIN-processing not possible")]
    PinProcessingNotPossible = 0xec,
    #[error("PIN-pad defective")]
    PinPadDefective2 = 0xed,
    #[error("open end-of-day batch present")]
    OpenEndOfDayBatchPresent = 0xf0,
    #[error("ec-cash/Maestro offline error")]
    EcCashOrMaestroOfflineError = 0xf1,
    #[error("OPT-error")]
    OptError = 0xf5,
    #[error("OPT-data not available (= OPT personalization required)")]
    OptDataNotAvailable = 0xf6,
    #[error("error transmitting offline-transactions (clearing error)")]
    ErrorTransmittingOfflineTransactions = 0xfa,
    #[error("turnover data-set defective")]
    TurnoverDataSetDefective = 0xfb,
    #[error("necessary device not present or defective")]
    NecessaryDeviceNotPresentOrDefective = 0xfc,
    #[error("baudrate not supported")]
    BaudRateNotSupported = 0xfd,
    #[error("register unknown")]
    RegisterUnknown = 0xfe,
    #[error("system error (= other/unknown error), See TLV tags 1F16 and 1F17")]
    SystemError = 0xff,
}
