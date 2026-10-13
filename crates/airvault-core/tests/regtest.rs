#![cfg(feature = "network")]
// Run explicitly with BITCOIND pointing to a verified local Bitcoin Core binary.
use airvault_core::{
    bitcoin as adapter,
    model::{Mode, Network},
    session::Session,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    process::{Child, Command, Stdio},
    time::Duration,
};
const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const PASSWORD: &str = "public-regtest-password";
struct Node {
    child: Child,
    url: String,
    cookie: String,
}
impl Drop for Node {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Node {
    fn rpc(&self, method: &str, params: Value) -> Value {
        let result: Value = ureq::post(&self.url)
            .set(
                "Authorization",
                &format!("Basic {}", STANDARD.encode(&self.cookie)),
            )
            .send_json(json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}))
            .unwrap()
            .into_json()
            .unwrap();
        assert!(
            result.get("error").is_none_or(Value::is_null),
            "{method}: {:?}",
            result["error"]
        );
        result["result"].clone()
    }
}
#[test]
#[ignore = "requires a verified Bitcoin Core binary; no external peers are enabled"]
fn wallet_core_transaction_is_accepted_and_confirmed_by_bitcoin_core() {
    let binary = std::env::var("BITCOIND").expect("set BITCOIND");
    let node_dir = tempfile::tempdir().unwrap();
    let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = socket.local_addr().unwrap().port();
    drop(socket);
    let child = Command::new(binary)
        .args([
            "-regtest",
            "-server=1",
            "-listen=0",
            "-connect=0",
            "-dnsseed=0",
            "-discover=0",
            "-rpcbind=127.0.0.1",
            "-rpcallowip=127.0.0.1",
        ])
        .arg(format!("-rpcport={port}"))
        .arg(format!("-datadir={}", node_dir.path().display()))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut node = Node {
        child,
        url: format!("http://127.0.0.1:{port}"),
        cookie: String::new(),
    };
    for _ in 0..100 {
        if let Ok(cookie) = std::fs::read_to_string(node_dir.path().join("regtest/.cookie")) {
            node.cookie = cookie;
            if ureq::post(&node.url)
                .set(
                    "Authorization",
                    &format!("Basic {}", STANDARD.encode(&node.cookie)),
                )
                .send_json(json!({"jsonrpc":"2.0","id":1,"method":"getblockchaininfo","params":[]}))
                .is_ok()
            {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(!node.cookie.is_empty(), "node did not start");
    let signer_dir = tempfile::tempdir().unwrap();
    let proxy_dir = tempfile::tempdir().unwrap();
    let mut signer = Session::new(signer_dir.path().join("signer.avvault"));
    signer.create(PASSWORD, Mode::Offline).unwrap();
    signer
        .mutate(|v| {
            let identity = v.import_phrase("Public regtest vector", PHRASE, None)?;
            v.derive_account(identity, "Input", Network::Regtest, "m/84'/1'/0'/0/0", None)?;
            v.derive_account(
                identity,
                "Recipient",
                Network::Regtest,
                "m/84'/1'/0'/0/1",
                None,
            )?;
            Ok(())
        })
        .unwrap();
    let accounts = signer.snapshot().accounts;
    let blocks = node.rpc("generatetoaddress", json!([101, accounts[0].address]));
    let block = node.rpc("getblock", json!([blocks[0], 2]));
    let previous = node
        .rpc(
            "getrawtransaction",
            json!([block["tx"][0]["txid"], false, blocks[0]]),
        )
        .as_str()
        .unwrap()
        .to_owned();
    let request = adapter::construct(
        Network::Regtest,
        vec![adapter::Utxo {
            transaction_hex: previous,
            vout: 0,
        }],
        vec![adapter::Recipient {
            address: accounts[1].address.clone(),
            satoshis: 100_000,
        }],
        &accounts[0].address,
        1000,
        &accounts,
    )
    .unwrap();
    let mut proxy = Session::new(proxy_dir.path().join("proxy.avvault"));
    proxy.create(PASSWORD, Mode::Proxy).unwrap();
    proxy
        .import_public(&signer.public_bytes().unwrap())
        .unwrap();
    let avreq = serde_json::to_vec(&request).unwrap();
    proxy.import_request(&avreq).unwrap();
    signer.import_request(&avreq).unwrap();
    let review = signer.review(request.request_id).unwrap();
    let response = signer
        .sign(review.token, true, PASSWORD, &HashMap::new(), &[])
        .unwrap();
    proxy
        .import_response(&serde_json::to_vec(&response).unwrap())
        .unwrap();
    let raw = hex::encode(response.payload.decode().unwrap());
    let accepted = node.rpc("testmempoolaccept", json!([[raw]]));
    assert_eq!(accepted[0]["allowed"], true, "{accepted}");
    let txid = node.rpc("sendrawtransaction", json!([raw]));
    assert_eq!(txid.as_str().unwrap(), response.txid);
    let mined = node.rpc("generatetoaddress", json!([1, accounts[0].address]));
    let confirmed = node.rpc("getrawtransaction", json!([txid, true, mined[0]]));
    assert_eq!(confirmed["confirmations"], 1);
    println!(
        "Bitcoin Core accepted and confirmed Wallet Core transaction: {}",
        response.txid
    );
}
