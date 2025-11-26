mod payment_requirements;
mod x402_middleware;

use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{Json, Router, extract::State, http::HeaderMap, response::IntoResponse, routing::get};
use dotenv::dotenv;
use rand::{Rng, seq::SliceRandom};
use serde_json::{Value, json};

use payment_requirements::PaymentRequirements;
use x402_middleware::x402_guard;

#[derive(Clone)]
struct AppState {
    requirements: PaymentRequirements,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let wallet = env::var("SELLER_WALLET").expect("SELLER_WALLET required");
    let state = AppState {
        requirements: PaymentRequirements::new(wallet),
    };

    let app = Router::new()
        .route("/weather", get(weather_handler))
        .route("/poem", get(poem_handler))
        .route("/crypto-price", get(crypto_price_handler))
        .with_state(state);

    println!(
        "\nRust X402 API running → \
         http://localhost:3000/{{weather,poem,crypto-price}}\n"
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind TCP listener");
    axum::serve(listener, app).await.unwrap();
}

async fn weather_handler(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let tx_hash = match x402_guard(headers, state.requirements.clone()).await {
        Ok(hash) => hash,
        Err(res) => return res,
    };

    Json(json!({
        "txHash": tx_hash,
        "weather": random_weather_report()
    }))
    .into_response()
}

async fn poem_handler(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let tx_hash = match x402_guard(headers, state.requirements.clone()).await {
        Ok(hash) => hash,
        Err(res) => return res,
    };

    Json(json!({
        "txHash": tx_hash,
        "poem": generate_poem()
    }))
    .into_response()
}

async fn crypto_price_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let tx_hash = match x402_guard(headers, state.requirements.clone()).await {
        Ok(hash) => hash,
        Err(res) => return res,
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Json(json!({
        "txHash": tx_hash,
        "asOf": now,
        "assets": mock_price_feed()
    }))
    .into_response()
}

fn random_weather_report() -> Value {
    let mut rng = rand::thread_rng();
    let conditions = [
        "crystal sky",
        "meteor shower",
        "aurora flare",
        "ion storm",
        "ocean breeze",
    ];
    let locales = [
        "over Cronos City",
        "above Web3 Harbor",
        "across validator valleys",
        "near the Gravity Bridge",
        "around the zk frontier",
    ];
    let summary = format!(
        "{} {}",
        conditions.choose(&mut rng).unwrap(),
        locales.choose(&mut rng).unwrap()
    );

    json!({
        "tempC": rng.gen_range(-5..35),
        "feelsLikeC": rng.gen_range(-7..33),
        "humidity": rng.gen_range(20..90),
        "windKmh": rng.gen_range(0..60),
        "summary": summary,
    })
}

fn generate_poem() -> Value {
    let mut rng = rand::thread_rng();
    let titles = [
        "Ballad of the On-Chain Voyager",
        "Ode to the Validator Winds",
        "Verses from the Cronos Observatory",
        "Canticle for the Sovereign Wallet",
    ];
    let voices = [
        "neural bard",
        "cosmic druid",
        "liquid metal skald",
        "prismatic oracle",
    ];
    let realms = [
        "copper dunes",
        "astral reef",
        "sky mirror",
        "quantum ravine",
    ];
    let quests = [
        "seeking proof of warmth",
        "chasing unmatched blockspace",
        "sowing keys of resonance",
        "braiding sunlit ledgers",
    ];
    let waypoints = [
        "validator valleys",
        "stellar orchards",
        "saffron circuits",
        "chrono canyons",
        "midnight staking pools",
    ];

    let title = titles.choose(&mut rng).unwrap().to_string();
    let lines = vec![
        format!(
            "I am a {} singing above the {}",
            voices.choose(&mut rng).unwrap(),
            realms.choose(&mut rng).unwrap()
        ),
        format!(
            "My syllables spark {} beats per rune",
            rng.gen_range(40..90)
        ),
        format!(
            "I drift through {}, {}",
            waypoints.choose(&mut rng).unwrap(),
            quests.choose(&mut rng).unwrap()
        ),
        "Until your wallet blooms open with aurora dust.".to_string(),
    ];

    json!({
        "title": title,
        "style": "AI fantasy free verse",
        "lines": lines,
    })
}

fn mock_price_feed() -> Vec<Value> {
    let mut rng = rand::thread_rng();
    let assets = vec![
        ("CRO", "Cronos", 0.085),
        ("TCRO", "Testnet CRO", 0.085),
        ("USDC.e", "USD Coin", 1.0),
        ("x402", "Access Token", 4.2),
    ];

    assets
        .into_iter()
        .map(|(symbol, name, base_price)| {
            let price =
                ((base_price * rng.gen_range(0.95_f64..1.05_f64)) * 100.0_f64).round() / 100.0_f64;
            let change =
                ((rng.gen_range(-0.05_f64..0.05_f64) * 100.0_f64)).round() / 100.0_f64;
            json!({
                "symbol": symbol,
                "name": name,
                "priceUsd": price,
                "change24hPercent": change,
                "volume24h": ((rng.gen_range(1_000_000.0_f64..9_000_000.0_f64) * 100.0_f64)).round() / 100.0_f64,
            })
        })
        .collect()
}
