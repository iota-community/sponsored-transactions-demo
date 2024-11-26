use axum::{
    routing::get,
    Router,
};


use iota_sdk::IotaClientBuilder;



#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // build our application with a single route
    let iota_testnet = IotaClientBuilder::default().build_testnet().await?;
    let msg = format!("welcome to IOTA Testnet {:?}", iota_testnet.api_version()) ;
    let app = Router::new().route("/", get( move | | async { msg}));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3001".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}