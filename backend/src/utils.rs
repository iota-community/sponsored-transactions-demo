use iota_sdk::{
    types::{
        base_types::{IotaAddress, ObjectID},
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        transaction::{CallArg, ObjectArg, SenderSignedData, TransactionData},
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

use anyhow::anyhow;
use std::str::FromStr;

use shared_crypto::intent::{Intent, IntentMessage};

/// Create signed and funded transaction
/// Supposed to return a transaction that calls the `free_trial` function of the `sponsored_transactions_packages` package.
// │ Published Objects:                                                                                                                       │
// │  ┌──                                                                                                                                     │
// │  │ PackageID: 0x2069e91c8333350bdf6bbd2991266ad33992757db7af48291adb58e7a5b0e1aa                                                         │
// │  │ Version: 1                                                                                                                            │
// │  │ Digest: 2EXnq389M6Vjazdv7NDetC75ahJZWdpDuUKtzNEYqjfc                                                                                  │
// │  │ Modules: sponsored_transactions_packages                                                                                              │
// │  └──                                                                                                                                     │
// ╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯
pub async fn sign_and_fund_transaction(
    client: &IotaClient,
    sender: &IotaAddress,
    sponsor: &IotaAddress,
    content_type: &str,
) -> Result<Envelope<SenderSignedData, EmptySignInfo>, anyhow::Error> {
    let keystore = FileBasedKeystore::new(&iota_config_dir()?.join(IOTA_KEYSTORE_FILENAME))?;

    let pt = {
        let mut builder = ProgrammableTransactionBuilder::new();

        // Call the `free_trial` function of the `sponsored_transactions_packages` package
        let package = ObjectID::from_str(
            "0x2069e91c8333350bdf6bbd2991266ad33992757db7af48291adb58e7a5b0e1aa",
        )?;
        let module = Identifier::from_str("sponsored_transactions_packages")?;
        let function = Identifier::from_str("free_trial")?;

        // The content type is passed as a parameter to the `free_trial` function. options are "Music", "News" or "Movies"
        let content_type_arg = CallArg::Pure(bcs::to_bytes(content_type)?);

        // This is the SubscriptionManager object that is used to manage subscriptions, it is a shared object.
        let object_arg = ObjectArg::SharedObject {
            id: ObjectID::from_str(
                "0xb05236e6ca067e3fa114bab1558f35e44097e0078aa681761faa62220858a176",
            )?,
            initial_shared_version: 5134.into(),
            mutable: true,
        };
        let manager_object = CallArg::Object(object_arg);
        builder
            .move_call(
                package,
                module,
                function,
                vec![],
                vec![content_type_arg, manager_object],
            )
            .unwrap();
        builder.finish()
    };

    // Get the gas coin of the sponsor
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

    // Sign the transaction by the sponsor
    let sponsor_signature = keystore.sign_secure(sponsor, &tx, Intent::iota_transaction())?;

    let intent_msg = IntentMessage::new(Intent::iota_transaction(), tx);

    // Now we have a signed tx, which will be sent back to the user (with tx bytes separated from the sponsor singnature)
    let signed_tx = types::transaction::Transaction::from_generic_sig_data(
        intent_msg.value,
        vec![GenericSignature::Signature(sponsor_signature)],
    );

    Ok(signed_tx)
}
