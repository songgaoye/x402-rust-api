mod payment_requirements;
mod x402_middleware;

use axum::{routing::get, Router, Json};
use serde_json::json;
use dotenv::dotenv;
use std::env;

use payment_requirements::PaymentRequirements;
use x402_middleware::x402_guard;

use axum::response::IntoResponse;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let wallet = env::var("SELLER_WALLET").expect("SELLER_WALLET required");
    let requirements = PaymentRequirements::new(wallet);

    let app = Router::new()
        .route("/rust-weather", get(move |headers| {
            let reqs = requirements.clone();
            async move {
                // Payment guard
                if let Err(res) = x402_guard(headers, reqs.clone()).await {
                    return res.into_response();
                }

                // Premium content returned now
                Json(json!({
                    "weather": {
                        "tempC": 22,
                        "wind_kmh": 4,
                        "summary": "Clear sky on chain 🌤️"
                    },
                    "access": "Payment Verified + Settled",
                }))
                .into_response()
            }
        }));

    println!("\n Rust X402 API running → http://localhost:3000/rust-weather\n");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind TCP listener");
    axum::serve(listener, app)
        .await
        .unwrap();
}
