# Specification status — Windows Bitcoin milestone

The full supplied specification remains the product target. This checkout implements a Windows testing milestone, with a complete narrow Bitcoin transfer path and the requested dice addendum. Unsupported capabilities are visibly disabled; no blind-signing fallback exists.

| Area | Status |
| --- | --- |
| Tauri 2, Svelte 5, TypeScript, Rust; Windows x64 | Implemented and built |
| Linux desktop package and VM validation | Pending user VM; core runs in WSL |
| Three operating modes and network-free build | Implemented |
| Multi-root BIP39, passphrase identities, WIF | Implemented; only Bitcoin compressed WIF |
| AES-GCM / Argon2id / custom root passwords | Implemented; fixed documented KDF parameters |
| Vault password change | Implemented |
| Custom-root password rotation/removal UI | Pending; storage rewrapping primitive exists |
| Root/account rename, delete, descriptions and per-key policies | Pending |
| Startup/inactivity/focus lock | Implemented and core tested |
| Windows sleep/resume lock | Native callback compiled; real device sleep cycle needs manual check |
| Dice addendum | Implemented: guided entry, full maths, review, local list, basic/advanced help, 12/24-word checks |
| Public accounts and request/response files | Implemented and tested |
| Public xpub/descriptors and account discovery | Pending |
| PSBT transaction pipeline | Implemented for P2WPKH single-key inputs / SIGHASH_ALL |
| P2PKH, nested SegWit, Taproot, multisig signing | Pending; address derivation alone is not signing support |
| Provider UTXOs, address state, broadcast | Implemented through Esplora |
| Fee estimation, transaction/confirmation tracking UI | Pending |
| Coin control | Manual previous-transaction JSON; polished selection UI pending |
| Other chains, tokens, NFTs, staking | Pending chain adapters, decoding and vectors |
| Message signing / EIP-712 | Pending |
| MCP server / scoped grants / approval queue | Pending; no production MCP endpoint |
| Backup and fresh-profile restore | Implemented and tested |
| Reproducible signed releases / security audit | Pending; current binary and installer unsigned |

The native Bitcoin signer is the pinned Trust Wallet Core implementation. See [engine investigation](./engine-investigation.md) for exact supplementary libraries and why upstream chain coverage does not imply enabled AirVault features.

The test harness is an entirely separate entry point, not a release flag. It allows clicking creation and approval controls against disposable real vaults. The production graph does not import it. Actual recovery words and entropy are not emitted into test logs; public examples and deterministic vectors are marked as such.

Next product milestones are completing Bitcoin account management/provider UX, additional Bitcoin script families, a scoped stdio MCP bridge that cannot approve actions, then chain-specific EVM, UTXO and Solana adapters. Common Polkadot sr25519 compatibility needs explicit engine validation; there is no ed25519 fallback.
