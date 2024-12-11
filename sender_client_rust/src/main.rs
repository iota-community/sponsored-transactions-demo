use fastcrypto::encoding::Encoding;
use iota_sdk::{
    types::{
        base_types::{IotaAddress, ObjectID},
        crypto::{SignableBytes, Signature, ToFromBytes},
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        transaction::{self, SenderSignedData, Transaction, TransactionData},
        Identifier,
    },
    IotaClient, IotaClientBuilder,
};

use iota_sdk::{
    rpc_types::IotaTransactionBlockResponseOptions,
    types::quorum_driver_types::ExecuteTransactionRequestType,
};

use std::{path::PathBuf, str::FromStr};

use iota_config::{iota_config_dir, IOTA_KEYSTORE_FILENAME};
use iota_keys::keystore::{AccountKeystore, FileBasedKeystore};

use iota_sdk::types;

use iota_sdk::types::{
    crypto::EmptySignInfo, message_envelope::Envelope, signature::GenericSignature,
};

use anyhow::{anyhow, bail};
use iota_sdk::rpc_types::IotaObjectDataOptions;
use serde_json::json;
use std::time::Duration;

use reqwest::Client;
use shared_crypto::intent::{Intent, IntentMessage};

pub const IOTA_FAUCET_BASE_URL: &str = "https://faucet.testnet.iota.cafe"; // testnet faucet

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    // Call the async function
    construct_tx().await;
}

async fn construct_tx() {
    let keystore = FileBasedKeystore::new(&PathBuf::from(
        "/home/vboxuser/.iota/iota_config/iota.keystore",
    ))
    .unwrap();

    //let tx_data = TransactionData::from_signable_bytes()

    let encoded_tx = "AAAAAQDt08q72OvSV1ois3Ury7XSiaLIg79SD9+bDB1Q7Q3beh9zcG9uc29yZWRfdHJhbnNhY3Rpb25zX3BhY2thZ2VzCXN1YnNjcmliZQAAx9FYqbBcbf0HwjNknJ4PeDINBm5awOj1EB3lAK2ehOgBOeqaP5iJwfw9edY9l64pOnSFnQzguHbAkCEUceNwZ7sIFAAAAAAAACAPLENBM7cfi5V6c/f5ot9UVn/Pj6i3t7UYrZsXRibTO78pPO0lkxGM0jHxB/NBuxrZ2znNBJe/8p01VzDPTivC6AMAAAAAAABAS0wAAAAAAAA=";
    let encoded_sponsor_sig = "AJjkrqCXfk6jxt8AlsuLvEp2PoWIJv6dpRv+QGZZrafZHkCJLRiw10rW14mn4JJm80fXgtaFbrYODVAvIQMcLgLp1vxEC3McO9IvqEI8BXaHBgCUFx3DXTO+QWmQKwj2uA==";

    let decoded = fastcrypto::encoding::Base64::decode(&encoded_tx).unwrap();
    let decoded_sig = fastcrypto::encoding::Base64::decode(&encoded_sponsor_sig).unwrap();

    let tx: TransactionData = bcs::from_bytes(&decoded).unwrap();

    //let sig_bcs : Signature = bcs::from_bytes(&decoded_sig).unwrap();

    let sig_bcs = Signature::from_bytes(&decoded_sig).unwrap();

    let intent_msg = IntentMessage::new(Intent::iota_transaction(), tx.clone());

    let sender_addr =
        IotaAddress::from_str("0xc7d158a9b05c6dfd07c233649c9e0f78320d066e5ac0e8f5101de500ad9e84e8")
            .unwrap();

    let sender_sig = keystore
        .sign_secure(&sender_addr, &tx, intent_msg.intent)
        .unwrap();

    let signed_tx = types::transaction::Transaction::from_generic_sig_data(
        intent_msg.value,
        vec![
            GenericSignature::Signature(sender_sig),
            GenericSignature::Signature(sig_bcs),
        ],
    );

    let iota_testnet = IotaClientBuilder::default().build_testnet().await.unwrap();

    let transaction_block_response = iota_testnet
        .quorum_driver_api()
        .execute_transaction_block(
            signed_tx.clone(),
            IotaTransactionBlockResponseOptions::full_content().with_raw_input(),
            ExecuteTransactionRequestType::WaitForLocalExecution,
        )
        .await
        .unwrap();

    println!("{:?}", transaction_block_response);
}
