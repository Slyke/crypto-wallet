use serde::Serialize;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub network: &'static str,
    pub derivation: &'static str,
    pub watch_only: bool,
    pub signing: &'static str,
    pub messages: &'static str,
    pub staking: &'static str,
    pub nfts: &'static str,
    pub note: &'static str,
}
pub fn registry() -> Vec<Capability> {
    let mut items = vec![Capability { network: "Bitcoin", derivation: "experimental: BIP44/49/84/86", watch_only: true, signing: "experimental: P2WPKH PSBT / SIGHASH_ALL", messages: "unsupported", staking: "unsupported", nfts: "unsupported", note: "Full previous transactions required. Mainnet, testnet3, testnet4, signet and regtest address encodings. No multisig or Taproot signing." }];
    for network in [
        "Ethereum",
        "SHIB / ERC-20",
        "Dogecoin",
        "Solana / SPL",
        "Polkadot",
        "Litecoin",
        "Bitcoin Cash",
        "BNB Smart Chain",
        "Polygon",
        "Avalanche C-Chain",
        "Arbitrum",
        "Optimism",
        "Base",
        "Cardano",
        "Cosmos Hub",
        "XRP Ledger",
        "Tron",
    ] {
        items.push(Capability { network, derivation: "unsupported", watch_only: false, signing: "unsupported", messages: "unsupported", staking: "unsupported", nfts: "unsupported", note: if network == "Polkadot" { "Common sr25519 recovery is not verified. No fallback to ed25519." } else { "Requires a tested chain-specific adapter. Wallet Core coverage alone does not enable signing." } });
    }
    items
}
