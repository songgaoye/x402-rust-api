use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequirements {
    #[serde(rename = "x402Version")]
    pub x402_version: u8,
    pub scheme: &'static str,
    pub network: &'static str,
    #[serde(rename = "payTo")]
    pub pay_to: String,
    pub asset: &'static str,
    #[serde(rename = "maxAmountRequired")]
    pub max_amount_required: &'static str,
    #[serde(rename = "maxTimeoutSeconds")]
    pub max_timeout_seconds: u64,
    pub description: &'static str,
    #[serde(rename = "mimeType")]
    pub mime_type: &'static str,
}

impl PaymentRequirements {
    pub fn new(wallet: String) -> Self {
        Self {
            x402_version: 1,
            scheme: "exact",
            network: "cronos-testnet",
            pay_to: wallet,
            asset: "0xc01efAaF7C5C61bEbFAeb358E1161b537b8bC0e0",  // USDC.e testnet
            max_amount_required: "1000000",   // 1 USDC.e
            max_timeout_seconds: 300,
            description: "Premium Rust Weather API",
            mime_type: "application/json",
        }
    }
}
