use axum::{
    extract::Json,
    routing::{get, post},
    Router,
};

mod utils;


use iota_sdk::types::base_types::IotaAddress;
use iota_sdk::IotaClientBuilder;
use serde::Deserialize;

use iota_sdk::types::transaction::{self, TransactionDataV1};




#[derive(Debug, Deserialize)]
pub struct GaslessTransactionRequest {
    pub sender: IotaAddress,
}


async fn handle_gasless_transaction(Json(payload): Json<GaslessTransactionRequest>) {
    println!("Received gasless transaction request: {:?}", payload);

    // lets try to send fund this address with some gas

    let iota_testnet = IotaClientBuilder::default().build_testnet().await.unwrap();

    let address = payload.sender;

    let request_from_faucet = utils::request_tokens_from_faucet(address, &iota_testnet).await.unwrap();

    


}





#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // build our application with a single route
    let iota_testnet = IotaClientBuilder::default().build_testnet().await?;
    let msg = format!("welcome to IOTA Testnet {:?}", iota_testnet.api_version()) ;

    //let app = Router::new().route("/", get( move | | async { msg}));

    let app = Router::new()
        .route("/", get(move || async { msg }))
        .route("/gasless-transaction", post(handle_gasless_transaction));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3001".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}