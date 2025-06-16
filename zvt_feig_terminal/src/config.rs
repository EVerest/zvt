use anyhow::{bail, Result};
use serde::Deserialize;
use std::net::Ipv4Addr;

/// The config for the Feig terminal included in the
/// [PoleConfiguration::configuration].
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct FeigConfig {
    /// The currency code as defined by ISO 4217. See
    /// https://en.wikipedia.org/wiki/ISO_4217.
    ///
    /// The input is the string representation of the currency as defined by
    /// ISO 4217, e.x. `EUR` or `GBP`.
    #[serde(default = "currency")]
    #[serde(deserialize_with = "deserialize_iso_4217")]
    pub currency: usize,

    /// The pre-authorization amount in the smallest currency unit (e.x. Cent).
    #[serde(default = "pre_authorization_amount")]
    pub pre_authorization_amount: usize,

    /// The default time to wait for reading a card in seconds. While a card is being read, the
    /// payment terminal cannot do anything else (like refunding a transaction for example).
    #[serde(default = "read_card_timeout")]
    pub read_card_timeout: u8,

    /// The password to the payment terminal.
    #[serde(default)]
    pub password: usize,

    /// The maximum time (in sec) between end of day jobs. Payment backends might
    /// force the payment terminals to run them regularly.
    #[serde(default = "end_of_day_max_interval")]
    pub end_of_day_max_interval: u64,
}

/// Deserializer which consumes a string code and returns the numerical code.
fn deserialize_iso_4217<'de, D>(deserializer: D) -> std::result::Result<usize, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let code = String::deserialize(deserializer)?;
    iso_4217(&code).map_err(serde::de::Error::custom)
}

/// The default currency (returns the EUR code).
const fn currency() -> usize {
    978
}

/// The default read card timeout in seconds
const fn read_card_timeout() -> u8 {
    15
}

/// The default pre-authorization amount in Cent (returns 25 EUR).
const fn pre_authorization_amount() -> usize {
    2500
}

/// The default duration between end of day jobs.
const fn end_of_day_max_interval() -> u64 {
    24 * 60 * 60
}

impl Default for FeigConfig {
    fn default() -> Self {
        Self {
            currency: currency(),
            pre_authorization_amount: pre_authorization_amount(),
            read_card_timeout: read_card_timeout(),
            password: 0,
            end_of_day_max_interval: end_of_day_max_interval(),
        }
    }
}

/// Maps the currency code (three letters) to a numeric value.
///
/// The mapping is defined under the ISO 4217. See
/// https://en.wikipedia.org/wiki/ISO_4217
fn iso_4217(code: &str) -> Result<usize> {
    match code.to_uppercase().as_str() {
        // Keep the list sorted by the numeric value.
        "SEK" => Ok(752),
        "GBP" => Ok(826),
        "EUR" => Ok(978),
        "PLN" => Ok(985),
        _ => bail!("Unknown currency code {code}"),
    }
}

/// The configuration needed for the entire payment terminal, which contains
/// parsed data.
#[derive(Clone, Debug)]
pub struct Config {
    pub terminal_id: String,
    /// We only use feig_serial to make sure we are connected to the proper
    /// terminal.
    pub feig_serial: String,
    pub ip_address: Ipv4Addr,
    /// Parsed from [PoleConfiguration::configuration].
    pub feig_config: FeigConfig,
    /// Maximum number of concurrent transactions.
    pub transactions_max_num: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            terminal_id: String::default(),
            feig_serial: String::default(),
            ip_address: Ipv4Addr::new(0, 0, 0, 0),
            feig_config: FeigConfig::default(),
            transactions_max_num: 1,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_feig_config() {
        // Valid inputs.
        let empty = serde_json::from_str::<FeigConfig>("{}").unwrap();
        assert_eq!(empty, FeigConfig::default());

        let with_currency = serde_json::from_str::<FeigConfig>("{\"currency\": \"GBP\"}").unwrap();
        assert_eq!(with_currency.currency, 826);
        assert_eq!(with_currency.pre_authorization_amount, 2500);

        let with_all = serde_json::from_str::<FeigConfig>(
            "{\"currency\": \"GBP\", \"pre_authorization_amount\": 10, \"end_of_day_max_interval\": 1234}",
        )
        .unwrap();
        assert_eq!(with_all.currency, 826);
        assert_eq!(with_all.pre_authorization_amount, 10);
        assert_eq!(with_all.end_of_day_max_interval, 1234);

        // Invalid inputs.
        assert!(serde_json::from_str::<FeigConfig>("{\"currency\": \"ABC\"}").is_err());
        assert!(serde_json::from_str::<FeigConfig>("{\"currency\": 123}").is_err());
        assert!(
            serde_json::from_str::<FeigConfig>("{\"pre_authorization_amount\": \"AB\"}").is_err()
        );
    }
}
