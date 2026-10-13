# Security boundaries and preview limitations

AirVault is experimental wallet software. The current build is intended for disposable test wallets and test networks. It has not had an independent security audit.

## Storage and trust boundaries

The native Rust process owns key derivation, secret encryption, generation, signing and verification. The webview performs UI work and transparent dice-to-word arithmetic; it does not derive seeds or sign. Newly generated phrases necessarily cross IPC once for their creation display. Ordinary snapshot/export commands do not return secrets.

A random 256-bit master key is wrapped with Argon2id (64 MiB, three iterations, one lane, a random 16-byte salt) and AES-256-GCM. Every record uses a random 256-bit key. Custom-password roots wrap that key only with the custom password; unlocking the outer vault does not decrypt those records. Passphrase records inherit their root's protection. Metadata, including encrypted secret records, is itself authenticated and encrypted. GCM nonces are independently generated 96-bit values. Associated data binds roles, version and identifiers. Master-password rotation rewraps the master key.

The entire authenticated file is written via a same-directory temporary file and atomic replacement. Backups contain encrypted secrets and require password reauthentication plus explicit UI confirmation. Restore never overwrites an existing profile. Unknown versions and unreasonable KDF/file bounds fail closed. This format is not yet a stable production interchange promise.

Native secret buffers use zeroization where supported. Rust crypto internals, webview strings, allocator copies, crash dumps, swap, hibernation and the OS may retain copies. This is not a defense against a compromised OS, malicious hardware, keyloggers or an attacker reading unlocked process memory. Windows ACLs are inherited from the selected profile directory; use a private local directory. Custom network locations need their own access-control review.

## Signing

Supported signing: Bitcoin P2WPKH single-key PSBT v0, SIGHASH_ALL. Every input requires its complete previous transaction; txid, vout, amount, script and witness UTXO must agree. Duplicate inputs, unknown/non-P2WPKH scripts, finalized/partially signed PSBTs and permissive sighash modes are rejected.

Review derives amounts, destinations and fees from transaction bytes. Descriptions are explicitly untrusted. Known addresses are labeled for the user to verify change, not automatically trusted as intent. An offline signer cannot independently know whether an output is unspent or whether a supplied blockchain history is current.

Each approval is bound to a request digest, expires after two minutes and is consumed on the first attempt, including failure. The vault password and any necessary custom root passwords are required. No-match and ambiguous-match inputs fail closed. Wallet Core produces signatures; a separate rust-bitcoin/libsecp256k1 path verifies every input and exact transaction semantics. Imported responses are matched to their original request and reverified on review and broadcast.

The current limits reject fees over 1,000,000 satoshis or 500 sat/vB, transactions over 100 inputs/outputs, exchange files over 2 MiB and payloads over 512 KiB. These conservative limits are preview policy, not universal fee advice. A signed request cannot be signed again with the same request ID. Bitcoin's consensus rules remain the final authority on validity and double-spending.

## Modes, IPC and lifecycle

Proxy mode refuses secret import and signing, and a vault containing secret records cannot switch to Proxy. Offline mode checks precede network construction; the hardened executable also excludes the provider dependency and commands at compile time. A physical air gap requires physical/OS isolation; a webview runtime may have its own OS-managed behavior.

Windows suspend/resume callbacks request immediate invalidation before the next native command. Inactivity and optional focus loss also lock the session. UI polling clears visible secrets after native locking. The five-minute phrase ceremony is single-use; cancellation and leaving the wallet page clear it. Actual dice histories live only in component memory and never enter the vault, clipboard, diagnostics, exports or MCP.

The production application contains no test bridge, automation bypass or remote-debugging route. Test-only code resides under `tests/ui` and the `ui_test_bridge` Cargo example, which are separate entry points. There is no runtime switch to enable them in a release binary. Production artifact scans enforce their absence.

MCP is not implemented in this preview. No tool can unlock/sign/export secrets via an MCP interface. Implementing scoped, authenticated, disabled-by-default stdio MCP with human approval queues remains a later milestone.

## Dependency review

Trust Wallet Core 4.8.4 is pinned at `d40d24a63d92619167903369308bf0e2f7eb3a59`. Its internal Rust Bitcoin API is used because the upstream C++ desktop build does not officially support Windows. The integration must be retested when the pin changes.

At the initial test date, npm audit reported no vulnerabilities. Cargo audit reported no entries under known vulnerabilities, but did report six unmaintained dependencies and two unsoundness warnings. The `atty` warning is in Wallet Core protobuf build tooling; `glib` is a Linux desktop dependency and absent from the Windows target graph. These warnings are not silently ignored and must be reviewed before a production release. This report is a point-in-time check, not a security guarantee.
