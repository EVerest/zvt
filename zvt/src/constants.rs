use num_derive::FromPrimitive;

/// Messages as defined under chapter 10.
#[derive(Debug, PartialEq, FromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum ErrorMessages {
    CardNotReadable = 0x64,
    CardDataNotPresent = 0x65,
    ProcessingError = 0x66,
    FunctionNotPermittedForEcAndMaestroCards = 0x67,
    FunctionNotPermittedForCreditAndTankCards = 0x68,
    TurnoverFileFull = 0x6a,
    FunctionDeactivated = 0x6b,
    AbortViaTimeoutOrAbortKey = 0x6c,
    CardInBlockedList = 0x6e,
    WrongCurrency = 0x6f,
    CreditNotSufficient = 0x71,
    ChipError = 0x72,
    CardDataIncorrect = 0x73,
    DukptEngineExhausted = 0x74,
    TextNotAuthentic = 0x75,
    PanNotInWhiteList = 0x76,
    EndOfDayBatchNotPossible = 0x77,
    CardExpired = 0x78,
    CardNotYetValid = 0x79,
    CardUnknown = 0x7a,
    FallbackToMagneticStripeNotPossibleForGiroCard1 = 0x7b,
    FallbackToMagneticStripeNotPossibleForNonGiroCard = 0x7c,
    CommunicationError = 0x7d,
    FallbackToMagneticStripeNotPossibleForGiroCard2 = 0x7e,
    FunctionNotPossible = 0x83,
    KeyMissing = 0x85,
    PinPadDefective1 = 0x89,
    ZvtProtocolError = 0x9a,
    ErrorFromDialUp = 0x9b,
    PleaseWait = 0x9c,
    ReceiverNotReady = 0xa0,
    RemoteStationDoesNotRespond = 0xa1,
    NoConnection = 0xa3,
    SubmissionOfGeldkarteNotPossible = 0xa4,
    FunctionNotAllowedDueToPciDss = 0xa5,
    MemoryFull = 0xb1,
    MerchantJournalFull = 0xb2,
    AlreadyReversed = 0xb4,
    ReversalNotPossible = 0xb5,
    PreAuthorizationIncorrect = 0xb7,
    ErrorPreAuthorization = 0xb8,
    VoltageSupplyToLow = 0xbf,
    CardLockingMechanismDefective = 0xc0,
    MerchantCardLocked = 0xc1,
    DiagnosisRequired = 0xc2,
    MaximumAmountExceeded = 0xc3,
    CardProfileInvalid = 0xc4,
    PaymentMethodNotSupported = 0xc5,
    CurrencyNotApplicable = 0xc6,
    AmountTooSmall = 0xc8,
    MaxTransactionAmountTooSmall = 0xc9,
    FunctionOnlyAllowedInEuro = 0xcb,
    PrinterNotReady = 0xcc,
    CashbackNotPossible = 0xcd,
    FunctionNotPermittedForServiceCards = 0xd2,
    CardInserted = 0xdc,
    ErrorDuringCardEject = 0xdd,
    ErrorDuringCardInsertion = 0xde,
    RemoteMaintenanceActivated = 0xe0,
    CardReaderDoesNotAnswer = 0xe2,
    ShutterClosed = 0xe3,
    TerminalActivationRequired = 0xe4,
    MinOneGoodsGroupNotFound = 0xe7,
    NoGoodsGroupsTableLoaded = 0xe8,
    RestrictionCodeNotPermitted = 0xe9,
    CardCodeNotPermitted = 0xea,
    FunctionNotExecutable = 0xeb,
    PinProcessingNotPossible = 0xec,
    PinPadDefective2 = 0xed,
    OpenEndOfDayBatchPresent = 0xf0,
    EcCashOrMaestroOfflineError = 0xf1,
    OptError = 0xf5,
    OptDataNotAvailable = 0xf6,
    ErrorTransmittingOfflineTransactions = 0xfa,
    TurnoverDataSetDefective = 0xfb,
    NecessaryDeviceNotPresentOrDefective = 0xfc,
    BaudRateNotSupported = 0xfd,
    RegisterUnknown = 0xfe,
    SystemError = 0xff,
}

impl std::fmt::Display for ErrorMessages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CardNotReadable => write!(f, "card not readable (LRC-/parity-error)"),
            Self::CardDataNotPresent => write!(f, "card-data not present (neither track-data nor chip found)"),
            Self::ProcessingError => write!(f, "processing-error (also for problems with card-reader mechanism)"),
            Self::FunctionNotPermittedForEcAndMaestroCards => write!(f, "function not permitted for ec- and Maestro-cards"),
            Self::FunctionNotPermittedForCreditAndTankCards => write!(f, "function not permitted for credit- and tank-cards"),
            Self::TurnoverFileFull => write!(f, "turnover-file full"),
            Self::FunctionDeactivated => write!(f, "function deactivated (PT not registered)"),
            Self::AbortViaTimeoutOrAbortKey => write!(f, "abort via timeout or abort-key"),
            Self::CardInBlockedList => write!(f, "card in blocked-list (response to command 06 E4)"),
            Self::WrongCurrency => write!(f, "wrong currency"),
            Self::CreditNotSufficient => write!(f, "credit not sufficient (chip-card)"),
            Self::ChipError => write!(f, "chip error"),
            Self::CardDataIncorrect => write!(f, "card-data incorrect (e.g. country-key check, checksum-error)"),
            Self::DukptEngineExhausted => write!(f, "DUKPT engine exhausted"),
            Self::TextNotAuthentic => write!(f, "text not authentic"),
            Self::PanNotInWhiteList => write!(f, "PAN not in white list"),
            Self::EndOfDayBatchNotPossible => write!(f, "end-of-day batch not possible"),
            Self::CardExpired => write!(f, "card expired"),
            Self::CardNotYetValid => write!(f, "card not yet valid"),
            Self::CardUnknown => write!(f, "card unknown"),
            Self::FallbackToMagneticStripeNotPossibleForGiroCard1 => write!(f, "fallback to magnetic stripe for girocard not possible"),
            Self::FallbackToMagneticStripeNotPossibleForNonGiroCard => write!(f, "fallback to magnetic stripe not possible (used for non girocard cards)"),
            Self::CommunicationError => write!(f, "communication error (communication module does not answer or is not present)"),
            Self::FallbackToMagneticStripeNotPossibleForGiroCard2 => write!(f, "fallback to magnetic stripe not possible, debit advice possible (used only for giro-card)"),
            Self::FunctionNotPossible => write!(f, "function not possible"),
            Self::KeyMissing => write!(f, "key missing"),
            Self::PinPadDefective1 => write!(f, "PIN-pad defective"),
            Self::ZvtProtocolError => write!(f, "ZVT protocol error. e. g. parsing error, mandatory message element missing"),
            Self::ErrorFromDialUp => write!(f, "error from dial-up/communication fault"),
            Self::PleaseWait => write!(f, "please wait"),
            Self::ReceiverNotReady => write!(f, "receiver not ready"),
            Self::RemoteStationDoesNotRespond => write!(f, "remote station does not respond"),
            Self::NoConnection => write!(f, "no connection"),
            Self::SubmissionOfGeldkarteNotPossible => write!(f, "submission of Geldkarte not possible"),
            Self::FunctionNotAllowedDueToPciDss => write!(f, "function not allowed due to PCI-DSS/P2PE rules"),
            Self::MemoryFull => write!(f, "memory full"),
            Self::MerchantJournalFull => write!(f, "merchant-journal full"),
            Self::AlreadyReversed => write!(f, "already reversed"),
            Self::ReversalNotPossible => write!(f, "reversal not possible"),
            Self::PreAuthorizationIncorrect => write!(f, "pre-authorization incorrect (amount too high) or amount wrong"),
            Self::ErrorPreAuthorization => write!(f, "error pre-authorization"),
            Self::VoltageSupplyToLow => write!(f, "voltage supply to low (external power supply)"),
            Self::CardLockingMechanismDefective => write!(f, "card locking mechanism defective"),
            Self::MerchantCardLocked => write!(f, "merchant-card locked"),
            Self::DiagnosisRequired => write!(f, "diagnosis required"),
            Self::MaximumAmountExceeded => write!(f, "maximum amount exceeded"),
            Self::CardProfileInvalid => write!(f, "card-profile invalid. New card-profiles must be loaded."),
            Self::PaymentMethodNotSupported => write!(f, "payment method not supported"),
            Self::CurrencyNotApplicable => write!(f, "currency not applicable"),
            Self::AmountTooSmall => write!(f, "amount too small"),
            Self::MaxTransactionAmountTooSmall => write!(f, "max. transaction-amount too small"),
            Self::FunctionOnlyAllowedInEuro => write!(f, "function only allowed in EURO"),
            Self::PrinterNotReady => write!(f, "printer not ready"),
            Self::CashbackNotPossible => write!(f, "Cashback not possible"),
            Self::FunctionNotPermittedForServiceCards => write!(f, "function not permitted for service-cards/bank-customer-cards"),
            Self::CardInserted => write!(f, "card inserted"),
            Self::ErrorDuringCardEject => write!(f, "error during card-eject (for motor-insertion reader)"),
            Self::ErrorDuringCardInsertion => write!(f, "error during card-insertion (for motor-insertion reader)"),
            Self::RemoteMaintenanceActivated => write!(f, "remote-maintenance activated"),
            Self::CardReaderDoesNotAnswer => write!(f, "card-reader does not answer / card-reader defective"),
            Self::ShutterClosed => write!(f, "shutter closed"),
            Self::TerminalActivationRequired => write!(f, "Terminal activation required"),
            Self::MinOneGoodsGroupNotFound => write!(f, "min. one goods-group not found"),
            Self::NoGoodsGroupsTableLoaded => write!(f, "no goods-groups-table loaded"),
            Self::RestrictionCodeNotPermitted => write!(f, "restriction-code not permitted"),
            Self::CardCodeNotPermitted => write!(f, "card-code not permitted (e.g. card not activated via Diagnosis)"),
            Self::FunctionNotExecutable => write!(f, "function not executable (PIN-algorithm unknown)"),
            Self::PinProcessingNotPossible => write!(f, "PIN-processing not possible"),
            Self::PinPadDefective2 => write!(f, "PIN-pad defective"),
            Self::OpenEndOfDayBatchPresent => write!(f, "open end-of-day batch present"),
            Self::EcCashOrMaestroOfflineError => write!(f, "ec-cash/Maestro offline error"),
            Self::OptError => write!(f, "OPT-error"),
            Self::OptDataNotAvailable => write!(f, "OPT-data not available (= OPT personalization required)"),
            Self::ErrorTransmittingOfflineTransactions => write!(f, "error transmitting offline-transactions (clearing error)"),
            Self::TurnoverDataSetDefective => write!(f, "turnover data-set defective"),
            Self::NecessaryDeviceNotPresentOrDefective => write!(f, "necessary device not present or defective"),
            Self::BaudRateNotSupported => write!(f, "baudrate not supported"),
            Self::RegisterUnknown => write!(f, "register unknown"),
            Self::SystemError => write!(f, "system error (= other/unknown error), See TLV tags 1F16 and 1F17"),
        }
    }
}
