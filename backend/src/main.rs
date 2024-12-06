use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde_json::json;

use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

mod utils;

use iota_sdk::types::base_types::IotaAddress;
use iota_sdk::IotaClientBuilder;
use serde::Deserialize;
use std::str::FromStr;


#[derive(Debug, Deserialize)]
pub struct GaslessTransactionRequest {
    pub sender: IotaAddress,
}

// Shared state to keep track of addresses
type SharedState = Arc<RwLock<HashSet<IotaAddress>>>;
//type SharedState = HashSet<IotaAddress>;

async fn faucet(
    State(state): State<Arc<RwLock<HashSet<IotaAddress>>>>,
    Json(payload): Json<GaslessTransactionRequest>,
) -> impl axum::response::IntoResponse {
    let mut addresses = state.write().await;

    if addresses.contains(&payload.sender) {
        // Return a conflict status if the address has already requested funds
        return (
            StatusCode::CONFLICT,
            Json(json!({"error": "Address already funded"})),
        );
    }

    // Add the address to the set
    addresses.insert(payload.sender);

    println!("Received gasless transaction request: {:?}", payload);

    // Simulate funding the address (replace this with your IOTA faucet logic)
    let iota_testnet = IotaClientBuilder::default().build_testnet().await.unwrap();
    utils::request_tokens_from_faucet(payload.sender, &iota_testnet)
        .await
        .unwrap();

    // Respond with success
    (
        StatusCode::OK,
        Json(json!({"message": "Funds requested successfully"})),
    )
}

/// Get an address from the user, and send back a signed sponsored transaction
async fn sign_and_fund_transaction(
    State(state): State<Arc<RwLock<HashSet<IotaAddress>>>>,
    Json(payload): Json<GaslessTransactionRequest>) -> impl axum::response::IntoResponse {
    println!("Received gasless transaction request: {:?}", payload);

    let mut addresses = state.write().await;

    if addresses.contains(&payload.sender) {
        // Return a conflict status if the address has already requested funds
        return (
            StatusCode::CONFLICT,
            Json(json!({"error": "Address already funded"})),
        );
    }

    // Add the address to the set
    addresses.insert(payload.sender);

    println!("Received gasless transaction request: {:?}", payload);

    // now lets create a signed and funded transaction
    let iota_testnet = IotaClientBuilder::default().build_testnet().await.unwrap();
    let sponsor = IotaAddress::from_str("0xbf293ced2593118cd231f107f341bb1ad9db39cd0497bff29d355730cf4e2bc2").unwrap();
    let signed_tx = utils::sign_and_fund_transaction(&iota_testnet, &payload.sender, &sponsor)
        .await
        .unwrap();

    println!("Signed and funded transaction: {:?}", signed_tx);


    // Respond with success
    (
        StatusCode::OK,
        Json(json!({"message": "Funds requested successfully"})),
    )
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create shared state
    let state: SharedState = Arc::new(RwLock::new(HashSet::new()));

    // Initialize IOTA client
    let iota_testnet = IotaClientBuilder::default().build_testnet().await?;
    let msg = format!("Welcome to IOTA Testnet {:?}", iota_testnet.api_version());

    // Build the router
    let app = Router::new()
        .route("/", get(move || async { msg }))
        .route("/faucet", post(faucet))
        .route("/sign_and_fund_transaction", post(sign_and_fund_transaction))
        .with_state(state);

    // Run the server
    axum::Server::bind(&"0.0.0.0:3001".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
