use iota_sdk::{
    types::{
        base_types::{IotaAddress, ObjectID},
        crypto::Signature,
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        transaction::{SenderSignedData, Transaction, TransactionData},
        Identifier,
    },
    IotaClient,
};

use iota_config::{iota_config_dir, IOTA_KEYSTORE_FILENAME};
use iota_keys::keystore::{AccountKeystore, FileBasedKeystore};

use iota_sdk::types;

use iota_sdk::types::{
    crypto::EmptySignInfo, message_envelope::Envelope, signature::GenericSignature,
};

use anyhow::{anyhow, bail};
use iota_sdk::rpc_types::IotaObjectDataOptions;
use serde_json::json;
use std::{str::FromStr, time::Duration};

use reqwest::Client;
use shared_crypto::intent::{Intent, IntentMessage};

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
/// Supposed to return a transaction that calls:
///│ Published Objects:                                                                              │
// │  ┌──                                                                                             │
// │  │ PackageID: 0xedd3cabbd8ebd2575a22b3752bcbb5d289a2c883bf520fdf9b0c1d50ed0ddb7a                 │
// │  │ Version: 1                                                                                    │
// │  │ Digest: 5jYzCtTT4G2KD5JgvB84yErVM6KVdgmzKmxkrkVadim1                                          │
// │  │ Modules: sponsored_transactions_packages                                                      │
// │  └──                                                                                             │
// ╰──────────────────────────────────────────────────────────────────────────────────────────────────╯
pub async fn sign_and_fund_transaction(
    client: &IotaClient,
    sender: &IotaAddress,
    sponsor: &IotaAddress,
) -> Result<Envelope<SenderSignedData, EmptySignInfo>, anyhow::Error> {
    let keystore = FileBasedKeystore::new(&iota_config_dir()?.join(IOTA_KEYSTORE_FILENAME))?;

    let path = iota_config_dir()?.join(IOTA_KEYSTORE_FILENAME);
    println!("Keystore path: {:?}", path);

    // TODO: Consruct a subscribe transaction
    let pt = {
        let mut builder = ProgrammableTransactionBuilder::new();
        let package = ObjectID::from_str(
            "0xedd3cabbd8ebd2575a22b3752bcbb5d289a2c883bf520fdf9b0c1d50ed0ddb7a",
        )?;
        let module = Identifier::from_str("sponsored_transactions_packages")?;
        let function = Identifier::from_str("subscribe")?;
        builder
            .move_call(package, module, function, vec![], vec![])
            .unwrap();
        builder.finish()
    };

    let gas_coin = client
        .coin_read_api()
        .get_coins(*sponsor, None, None, None)
        .await?
        .data
        .into_iter()
        .next()
        .ok_or(anyhow!("No coins found for sender"))?;

    let gas_budget = 5_000_000;
    let gas_price = client.read_api().get_reference_gas_price().await?;

    let tx = TransactionData::new_programmable_allow_sponsor(
        *sender,
        vec![gas_coin.object_ref()],
        pt,
        gas_budget,
        gas_price,
        *sponsor,
    );

    // This should be done by the sender when the tx is recieved
    //let signature = keystore.sign_secure(sender, &tx, Intent::iota_transaction())?;

    let sponsor_signature = keystore.sign_secure(sponsor, &tx, Intent::iota_transaction())?;

    let intent_msg = IntentMessage::new(Intent::iota_transaction(), tx);

    let signed_tx = types::transaction::Transaction::from_generic_sig_data(
        intent_msg.value,
        vec![
            //GenericSignature::Signature(signature),
            GenericSignature::Signature(sponsor_signature),
        ],
    );

    Ok(signed_tx)
}
