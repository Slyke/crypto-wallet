//! Test process only. Not linked by either desktop executable. Public/disposable fixtures only.
use airvault_core::{
    bitcoin as adapter, capabilities,
    model::{Mode, Network},
    protocol::Request,
    session::Session,
    Error, Result,
};
use bitcoin::{
    absolute, consensus, transaction, Address, Amount, OutPoint, ScriptBuf, Sequence, Transaction,
    TxIn, TxOut, Witness,
};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    io::{self, BufRead, Write},
    str::FromStr,
};
use uuid::Uuid;
fn text<'a>(a: &'a Value, name: &str) -> &'a str {
    a[name].as_str().unwrap_or("")
}
fn id(a: &Value, name: &str) -> Result<Uuid> {
    Uuid::parse_str(text(a, name)).map_err(|_| Error::Invalid("test UUID"))
}
fn handle(s: &mut Session, request: &Value) -> Result<Value> {
    let a = &request["args"];
    s.tick();
    match text(request, "command") {
        "snapshot" => Ok(serde_json::to_value(s.snapshot())?),
        "capabilities" => Ok(serde_json::to_value(capabilities::registry())?),
        "cancel_ceremony" => {
            s.cancel_ceremony();
            Ok(Value::Null)
        }
        "activity" => {
            s.touch();
            Ok(Value::Null)
        }
        "create_vault" => {
            s.create(
                text(a, "password"),
                serde_json::from_value(a["mode"].clone())?,
            )?;
            Ok(Value::Null)
        }
        "unlock_vault" => {
            s.unlock(text(a, "password"))?;
            Ok(Value::Null)
        }
        "lock_vault" => {
            s.lock();
            Ok(Value::Null)
        }
        "generate_phrase" => Ok(serde_json::to_value(s.generate(
            a["words"].as_u64().unwrap_or(24) as u8,
            text(a, "prefix"),
            a["bits"].as_str(),
        )?)?),
        "confirm_phrase" => Ok(json!(s.confirm_phrase(
            id(a, "token")?,
            text(a, "confirmation"),
            text(a, "label"),
            a["keyPassword"].as_str()
        )?)),
        "import_phrase" => Ok(json!(s.mutate(|v| v.import_phrase(
            text(a, "label"),
            text(a, "phrase"),
            a["keyPassword"].as_str()
        ))?)),
        "derive_account" => Ok(json!(s.mutate(|v| v.derive_account(
            id(a, "identityId")?,
            text(a, "label"),
            serde_json::from_value(a["network"].clone())?,
            text(a, "path"),
            a["keyPassword"].as_str()
        ))?)),
        "save_settings" => {
            s.settings(
                serde_json::from_value(a["settings"].clone())?,
                text(a, "password"),
                a["confirmed"].as_bool().unwrap_or(false),
            )?;
            Ok(Value::Null)
        }
        "review_request" => Ok(serde_json::to_value(s.review(id(a, "id")?)?)?),
        "approve_signing" => {
            let passwords: HashMap<Uuid, String> =
                serde_json::from_value(a["keyPasswords"].clone())?;
            let keys = passwords
                .into_iter()
                .map(|(k, v)| (k, zeroize::Zeroizing::new(v)))
                .collect();
            Ok(serde_json::to_value(s.sign(
                id(a, "token")?,
                a["confirmed"].as_bool().unwrap_or(false),
                text(a, "password"),
                &keys,
                &serde_json::from_value::<Vec<Uuid>>(a["selectedAccounts"].clone())?,
            )?)?)
        }
        "test_fund" => {
            let accounts = s.snapshot().accounts;
            let account = accounts.first().ok_or(Error::NotFound)?;
            let address = Address::from_str(&account.address)
                .unwrap()
                .require_network(account.network.native())
                .unwrap();
            let previous = Transaction {
                version: transaction::Version::TWO,
                lock_time: absolute::LockTime::ZERO,
                input: vec![TxIn {
                    previous_output: OutPoint::null(),
                    script_sig: ScriptBuf::from_bytes(vec![1, 1]),
                    sequence: Sequence::MAX,
                    witness: Witness::new(),
                }],
                output: vec![TxOut {
                    value: Amount::from_sat(100_000),
                    script_pubkey: address.script_pubkey(),
                }],
            };
            let r: Request = adapter::construct(
                account.network,
                vec![adapter::Utxo {
                    transaction_hex: hex::encode(consensus::serialize(&previous)),
                    vout: 0,
                }],
                vec![adapter::Recipient {
                    address: account.address.clone(),
                    satoshis: 40_000,
                }],
                &account.address,
                1000,
                &accounts,
            )?;
            s.import_request(&serde_json::to_vec(&r)?)?;
            Ok(json!(r.request_id))
        }
        "test_seed" => {
            s.create("public-test-password", Mode::Offline)?;
            s.mutate(|v| {
                let identity = v.import_phrase("Disposable public fixture", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about", None)?;
                v.derive_account(identity, "Public testnet account", Network::Testnet, "m/84'/1'/0'/0/0", None)?;
                Ok(())
            })?;
            Ok(Value::Null)
        }
        _ => Err(Error::Unsupported("test bridge command")),
    }
}
fn main() {
    let mut directory = tempfile::tempdir().unwrap();
    let mut session = Session::new(directory.path().join("test.avvault"));
    for line in io::stdin().lock().lines() {
        let request: Value = match serde_json::from_str(&line.unwrap()) {
            Ok(v) => v,
            Err(_) => {
                println!(
                    "{}",
                    r#"{"ok":false,"error":{"message":"Invalid test request"}}"#
                );
                continue;
            }
        };
        let value = if request["command"] == "test_reset" {
            session.lock();
            directory = tempfile::tempdir().unwrap();
            session = Session::new(directory.path().join("test.avvault"));
            Ok(Value::Null)
        } else {
            handle(&mut session, &request)
        };
        let reply = match value {
            Ok(value) => json!({"ok":true,"value":value}),
            Err(error) => json!({"ok":false,"error":error}),
        };
        println!("{}", reply);
        io::stdout().flush().unwrap();
    }
}
