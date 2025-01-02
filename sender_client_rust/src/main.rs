use fastcrypto::encoding::Encoding;
use iota_keys::keystore::AccountKeystore;
use iota_sdk::{
    types::{
        base_types::IotaAddress,
        crypto::{Signature, ToFromBytes},
        transaction::TransactionData,
    },
    IotaClientBuilder,
};

use anyhow::{anyhow, bail};
use clap::{App, Arg};
use iota_keys::keystore::FileBasedKeystore;
use shared_crypto::intent::{Intent, IntentMessage};
use std::{path::PathBuf, str::FromStr};

#[tokio::main]
async fn main() {
    let matches = App::new("IOTA Transaction")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Constructs and sends an IOTA transaction")
        .arg(
            Arg::with_name("keystore_path")
                .long("keystore")
                .value_name("KEYSTORE_PATH")
                .help("Path to the keystore file")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("encoded_tx")
                .long("encoded-tx")
                .value_name("ENCODED_TX")
                .help("Base64-encoded transaction data")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("encoded_sig")
                .long("encoded-sig")
                .value_name("ENCODED_SIG")
                .help("Base64-encoded sponsor's signature")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("sender_addr")
                .long("sender-addr")
                .value_name("SENDER_ADDR")
                .help("Sender's IOTA address")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let keystore_path = matches.value_of("keystore_path").unwrap();
    let encoded_tx = matches.value_of("encoded_tx").unwrap();
    let encoded_sig = matches.value_of("encoded_sig").unwrap();
    let sender_addr = matches.value_of("sender_addr").unwrap();

    if let Err(e) = construct_tx(keystore_path, encoded_tx, encoded_sig, sender_addr).await {
        eprintln!("Error: {:?}", e);
    }
}

async fn construct_tx(
    keystore_path: &str,
    encoded_tx: &str,
    encoded_sponsor_sig: &str,
    sender_addr: &str,
) -> Result<(), anyhow::Error> {
    let keystore = FileBasedKeystore::new(&PathBuf::from(keystore_path))
        .map_err(|e| anyhow::anyhow!("Failed to load keystore: {:?}", e))?;

    let decoded_tx = fastcrypto::encoding::Base64::decode(encoded_tx)
        .map_err(|_| anyhow::anyhow!("Failed to decode transaction"))?;
    let decoded_sig = fastcrypto::encoding::Base64::decode(encoded_sponsor_sig)
        .map_err(|_| anyhow::anyhow!("Failed to decode sponsor's signature"))?;

    let tx: TransactionData = bcs::from_bytes(&decoded_tx)
        .map_err(|_| anyhow::anyhow!("Failed to deserialize transaction data"))?;
    let sig_bcs = Signature::from_bytes(&decoded_sig)
        .map_err(|_| anyhow::anyhow!("Failed to deserialize sponsor's signature"))?;

    let intent_msg = IntentMessage::new(Intent::iota_transaction(), tx.clone());

    let sender_addr = IotaAddress::from_str(sender_addr)
        .map_err(|_| anyhow::anyhow!("Failed to parse sender address"))?;

    let sender_sig = keystore
        .sign_secure(&sender_addr, &tx, intent_msg.intent)
        .map_err(|_| anyhow::anyhow!("Failed to sign transaction with sender's keystore"))?;

    let signed_tx = iota_sdk::types::transaction::Transaction::from_generic_sig_data(
        intent_msg.value,
        vec![
            iota_sdk::types::signature::GenericSignature::Signature(sender_sig),
            iota_sdk::types::signature::GenericSignature::Signature(sig_bcs),
        ],
    );

    let iota_testnet = IotaClientBuilder::default()
        .build_testnet()
        .await
        .map_err(|_| anyhow::anyhow!("Failed to connect to IOTA testnet"))?;

    let transaction_block_response = iota_testnet
        .quorum_driver_api()
        .execute_transaction_block(
            signed_tx.clone(),
            iota_sdk::rpc_types::IotaTransactionBlockResponseOptions::full_content()
                .with_raw_input(),
            iota_sdk::types::quorum_driver_types::ExecuteTransactionRequestType::WaitForLocalExecution,
        )
        .await
        .map_err(|e| anyhow!("Failed to execute transaction: {:?}", e))?;

    println!("{:?}", transaction_block_response);
    Ok(())
}
