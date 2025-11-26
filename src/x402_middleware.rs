use axum::{
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use reqwest::Client;

use crate::payment_requirements::PaymentRequirements;

const FACILITATOR: &str = "https://facilitator.cronoslabs.org/v2/x402";

pub async fn x402_guard(headers: HeaderMap, requirements: PaymentRequirements)
    -> Result<(), impl IntoResponse>
{
    let Some(payment_header) = headers.get("x-payment") else {
        return Err((
            StatusCode::PAYMENT_REQUIRED,
            Json(json!({
                "error": "Payment Required",
                "x402Version": 1,
                "paymentRequirements": requirements
            }))
        ));
    };

    let header_str = payment_header.to_str().unwrap().to_string();

    let verify_body = json!({
        "x402Version": 1,
        "paymentHeader": header_str,
        "paymentRequirements": requirements
    });

    let client = Client::new();

    // 1. Verify
    let verify = client.post(format!("{}/verify", FACILITATOR))
        .json(&verify_body)
        .header("X402-Version", "1")
        .send().await.unwrap()
        .json::<serde_json::Value>().await.unwrap();

    if verify["isValid"] != true {
        return Err((StatusCode::PAYMENT_REQUIRED, Json(json!({
            "error": "Invalid Payment",
            "reason": verify["invalidReason"]
        }))));
    }

    // 2. Settle
    let settle = client.post(format!("{}/settle", FACILITATOR))
        .json(&verify_body)
        .header("X402-Version", "1")
        .send().await.unwrap()
        .json::<serde_json::Value>().await.unwrap();

    if settle["event"] != "payment.settled" {
        return Err((StatusCode::PAYMENT_REQUIRED, Json(json!({
            "error": "Settlement Failed",
            "detail": settle["error"]
        }))));
    }

    Ok(())
}
