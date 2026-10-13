# Signing engine decision

Pinned source: Trust Wallet Core **4.8.4**, commit `d40d24a63d92619167903369308bf0e2f7eb3a59`.

Inspected upstream TWHDWallet.h, TWAnySigner.h, registry.json, BitcoinV2.proto, Bitcoin's Rust entry.rs, modules/signer.rs, modules/psbt_request, and Polkadot's Rust entry.rs before writing adapters.

The official C++ desktop build does not officially support Windows. This build uses the upstream native Rust Bitcoin implementation directly, pinned by commit. BitcoinEntry::sign invokes the same BitcoinV2 signer behind AnySigner. This is an internal upstream API; the adapter isolates it and the commit must only change with vector and interoperability testing.

Wallet Core's native Rust crates do not expose the C++ HDWallet mnemonic/HD derivation interface. The bip39 crate supplies standard mnemonic validation, NFKD passphrase normalization and seed generation. rust-bitcoin supplies BIP32, address encodings, PSBT construction/decoding and independent libsecp256k1 verification. Signing remains in Wallet Core (which uses k256 for ECDSA). No alternate multi-chain SDK is included. This avoids the outdated community Windows C++ fork. These additions require dependency review before a production release.

Upstream support does not imply AirVault support. Initial capability assessment:

| Network | Upstream evidence | AirVault integration target |
|---|---|---|
| Bitcoin | BitcoinV2 PSBT and transaction builder | Native SegWit single-key inputs first; independently verify every input |
| Ethereum / BSC / Polygon / Avalanche C / Arbitrum / Optimism / Base | Ethereum protobuf and Rust entry | Disabled until typed decoding and verification tests exist |
| SHIB / ERC-20 | Ethereum transfer operation | Disabled with EVM adapter |
| Dogecoin / Litecoin / BCH | UTXO registry and chain-specific entries | Disabled until chain-specific vectors exist |
| Solana / SPL | Solana protobuf and Rust entry | Disabled until instruction decoding and recovery vectors exist |
| Polkadot | ed25519 in inspected entry; not sr25519 | Disabled; do not produce incompatible common-wallet accounts |
| Cardano / Cosmos / XRP / Tron | Registry entries; chain-specific definitions | Disabled until verified integration |
| NFTs / staking / typed messages | Operation-specific support varies | No blind signing fallback |

Sources: https://github.com/trustwallet/wallet-core/tree/4.8.4 and https://developer.trustwallet.com/developer/wallet-core/faq.
