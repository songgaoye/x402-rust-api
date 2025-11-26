use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequirements {
    pub x402Version: u8,
    pub scheme: &'static str,
    pub network: &'static str,
    pub payTo: String,
    pub asset: &'static str,
    pub maxAmountRequired: &'static str,
    pub maxTimeoutSeconds: u64,
    pub description: &'static str,
    pub mimeType: &'static str,
}

impl PaymentRequirements {
    pub fn new(wallet: String) -> Self {
        Self {
            x402Version: 1,
            scheme: "exact",
            network: "cronos-testnet",
            payTo: wallet,
            asset: "0xc01efAaF7C5C61bEbFAeb358E1161b537b8bC0e0",  // USDC.e testnet
            maxAmountRequired: "1000000",   // 1 USDC.e
            maxTimeoutSeconds: 300,
            description: "Premium Rust Weather API",
            mimeType: "application/json",
        }
    }
}
