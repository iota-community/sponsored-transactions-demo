use iota_sdk::{
    iota_client_config::{IotaClientConfig, IotaEnv},
    rpc_types::IotaTransactionBlockResponseOptions,
    types::{
        base_types::{IotaAddress, ObjectID},
        crypto::SignatureScheme::ED25519,
        digests::TransactionDigest,
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        quorum_driver_types::ExecuteTransactionRequestType,
        transaction::{self, Argument, Command, Transaction, TransactionData},
    },
    wallet_context::WalletContext,
    IotaClient, IotaClientBuilder,
};

use iota_config::{
    Config, IOTA_CLIENT_CONFIG, IOTA_KEYSTORE_FILENAME, PersistedConfig, iota_config_dir,
};
use iota_keys::keystore::{AccountKeystore, FileBasedKeystore};

use anyhow::bail;
use iota_sdk::rpc_types::IotaObjectDataOptions;
use serde_json::json;
use std::{str::FromStr, time::Duration};

use reqwest::Client;
use shared_crypto::intent::Intent;

pub const IOTA_FAUCET_BASE_URL: &str = "https://faucet.testnet.iota.cafe"; // testnet faucet

#[derive(serde::Deserialize)]
struct FaucetResponse {
    task: String,
    error: Option<String>,
}


/// Request tokens from the Faucet for the given address
pub async fn request_tokens_from_faucet(
    address: IotaAddress,
    client: &IotaClient,
) -> Result<(), anyhow::Error> {
    let address_str = address.to_string();
    let json_body = json![{
        "FixedAmountRequest": {
            "recipient": &address_str
        }
    }];

    // make the request to the faucet JSON RPC API for coin
    let reqwest_client = Client::new();
    let resp = reqwest_client
        .post(format!("{IOTA_FAUCET_BASE_URL}/v1/gas"))
        .header("Content-Type", "application/json")
        .json(&json_body)
        .send()
        .await?;
    println!(
        "Faucet request for address {address_str} has status: {}",
        resp.status()
    );
    println!("Waiting for the faucet to complete the gas request...");
    let faucet_resp: FaucetResponse = resp.json().await?;

    let task_id = if let Some(err) = faucet_resp.error {
        bail!("Faucet request was unsuccessful. Error is {err:?}")
    } else {
        faucet_resp.task
    };

    println!("Faucet request task id: {task_id}");

    // wait for the faucet to finish the batch of token requests
    let coin_id = loop {
        let resp = reqwest_client
            .get(format!("{IOTA_FAUCET_BASE_URL}/v1/status/{task_id}"))
            .send()
            .await?;
        let text = resp.text().await?;
        if text.contains("SUCCEEDED") {
            let resp_json: serde_json::Value = serde_json::from_str(&text).unwrap();

            break <&str>::clone(
                &resp_json
                    .pointer("/status/transferred_gas_objects/sent/0/id")
                    .unwrap()
                    .as_str()
                    .unwrap(),
            )
            .to_string();
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    };

    // wait until the fullnode has the coin object, and check if it has the same
    // owner
    loop {
        let owner = client
            .read_api()
            .get_object_with_options(
                ObjectID::from_str(&coin_id)?,
                IotaObjectDataOptions::new().with_owner(),
            )
            .await?;

        if owner.owner().is_some() {
            let owner_address = owner.owner().unwrap().get_owner_address()?;
            if owner_address == address {
                break;
            }
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    println!("Faucet request for address {address_str} has completed successfully");
    Ok(())
}


/// Create signed and funded transaction
pub async fn sign_and_fund_transaction(
    client: &IotaClient,
    sender: &IotaAddress,
    tx_data: TransactionData,
) -> Result<(), anyhow::Error> {
    let keystore = FileBasedKeystore::new(&iota_config_dir()?.join(IOTA_KEYSTORE_FILENAME))?;
    let signature = keystore.sign_secure(sender, &tx_data, Intent::iota_transaction())?;


    // TODO: Build a transaction with the signature
    //let transaction = TransactionData::new_programmable_allow_sponsor(sender, gas_payment, pt, gas_budget, gas_price, sponsor);

   



    Ok(())
}
