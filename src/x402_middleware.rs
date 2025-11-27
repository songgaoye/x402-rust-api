use std::env;

use axum::{
    Json,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use reqwest::Client;
use serde_json::{Value, json};

use crate::payment_requirements::PaymentRequirements;

const DEFAULT_FACILITATOR: &str = "https://facilitator.cronoslabs.org/v2/x402";

pub async fn create_purchase(
    payment_header: &str,
    resource_id: &str,
    requirements: PaymentRequirements,
) -> Result<Value, Response> {
    let settlement = process_payment(payment_header, &requirements).await?;

    let tx_hash = match settlement.get("txHash").and_then(Value::as_str) {
        Some(hash) => hash.to_string(),
        None => {
            return Err((
                StatusCode::PAYMENT_REQUIRED,
                Json(json!({
                    "error": "Payment not settled",
                    "settlement": settlement
                })),
            )
                .into_response());
        }
    };

    let resource = unlock_resource(resource_id);

    Ok(json!({
        "status": "ok",
        "tx": tx_hash,
        "resource": resource,
    }))
}

pub async fn x402_guard(
    headers: HeaderMap,
    requirements: PaymentRequirements,
) -> Result<String, Response> {
    let Some(payment_header) = headers.get("x-payment") else {
        return Err(payment_required_response(
            &requirements,
            "Payment Required",
            None,
        ));
    };

    let header_str = match payment_header.to_str() {
        Ok(value) => value.to_string(),
        Err(_) => {
            return Err(payment_required_response(
                &requirements,
                "Invalid X-PAYMENT header",
                Some(json!({ "reason": "Header must be valid UTF-8" })),
            ));
        }
    };

    let purchase =
        create_purchase(&header_str, requirements.description, requirements.clone()).await?;

    let tx_hash = purchase
        .get("tx")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": "Purchase response missing tx",
                })),
            )
                .into_response()
        })?
        .to_string();

    Ok(tx_hash)
}

async fn process_payment(
    payment_header: &str,
    requirements: &PaymentRequirements,
) -> Result<Value, Response> {
    let settle_body = json!({
        "x402Version": 1,
        "paymentHeader": payment_header,
        "paymentRequirements": requirements
    });

    let client = Client::new();
    let facilitator = facilitator_base_url();
    let verify_url = format!("{}/verify", facilitator);
    let settle_url = format!("{}/settle", facilitator);

    // Verify payment header first
    let verify = client
        .post(&verify_url)
        .json(&settle_body)
        .header("X402-Version", "1")
        .send()
        .await
        .map_err(|err| facilitator_unavailable("verify", err))?
        .json::<Value>()
        .await
        .map_err(|err| facilitator_unavailable("verify (json)", err))?;

    let is_valid = verify
        .get("isValid")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !is_valid {
        let reason = verify
            .get("invalidReason")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown reason");
        return Err(payment_required_response(
            requirements,
            "Invalid Payment",
            Some(json!({ "reason": reason })),
        ));
    }

    let settlement = client
        .post(&settle_url)
        .json(&settle_body)
        .header("X402-Version", "1")
        .send()
        .await
        .map_err(|err| facilitator_unavailable("settle", err))?
        .json::<Value>()
        .await
        .map_err(|err| facilitator_unavailable("settle (json)", err))?;

    if settlement
        .get("event")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        != "payment.settled"
    {
        let detail = settlement
            .get("error")
            .cloned()
            .unwrap_or_else(|| json!(null));
        return Err((
            StatusCode::PAYMENT_REQUIRED,
            Json(json!({
                "error": "Settlement Failed",
                "detail": detail
            })),
        )
            .into_response());
    }

    Ok(settlement)
}

fn unlock_resource(resource_id: &str) -> String {
    format!("Access granted -> {}", resource_id)
}

fn payment_required_response(
    requirements: &PaymentRequirements,
    error: &str,
    extra: Option<Value>,
) -> Response {
    let mut body = json!({
        "error": error,
        "x402Version": 1,
        "paymentRequirements": requirements
    });

    if let Some(extra_value) = extra {
        if let Some(map) = body.as_object_mut() {
            if let Some(extra_map) = extra_value.as_object() {
                for (key, value) in extra_map {
                    map.insert(key.clone(), value.clone());
                }
            } else {
                map.insert("detail".to_string(), extra_value);
            }
        }
    }

    (StatusCode::PAYMENT_REQUIRED, Json(body)).into_response()
}

fn facilitator_unavailable(context: &str, err: reqwest::Error) -> Response {
    (
        StatusCode::BAD_GATEWAY,
        Json(json!({
            "error": format!("Facilitator {} request failed", context),
            "detail": err.to_string(),
        })),
    )
        .into_response()
}

fn facilitator_base_url() -> String {
    env::var("FACILITATOR_URL").unwrap_or_else(|_| DEFAULT_FACILITATOR.to_string())
}
