# AirVault — Windows testing preview

A Tauri 2 / Svelte 5 / Rust desktop vault with a working Bitcoin native SegWit file-signing workflow. This is the first Bitcoin testing milestone of the supplied specification, **not the completed multi-chain product**.

## Run on Windows

Build outputs are in `artifacts/windows/`:

- `AirVault_0.1.0_windows-x64-portable.zip`: recommended portable package; extract the entire ZIP.
- `WebView2Loader.dll`: must stay beside both executables.
- `AirVault.exe`: portable Windows x64 application, with Offline, Proxy and Combined modes.
- `AirVault-offline.exe`: portable signer build with the blockchain networking dependency and commands excluded at compile time.
- `AirVault_0.1.0_x64-setup.exe`: current-user installer for the normal application.
- `SHA256SUMS.txt`: hashes of these files.

Windows 10/11 x64 with Microsoft Edge WebView2 Runtime is required. The installer deliberately does not download a runtime: prepare WebView2 before moving the app to an offline machine. These initial binaries are unsigned.

Start with a disposable wallet. The application has **Basic instructions** and **Advanced instructions**, available before unlocking and from the Instructions page. Appearance offers D4rk, Terminal, Paper and Ocean, each in dark or light mode.

By default, the vault is stored under the Tauri application data directory for `app.airvault.desktop` (normally `%APPDATA%\\app.airvault.desktop`). Supply a separate profile for each test role:

```powershell
.\\AirVault.exe --data-dir C:\\AirVaultTests\\signer
.\\AirVault.exe --data-dir C:\\AirVaultTests\\proxy
.\\AirVault-offline.exe --data-dir C:\\AirVaultTests\\offline
```

An exclusive profile lock prevents two app instances from modifying the same vault. The vault password is required on every startup. Backups restore into a fresh profile and support a read-only integrity check first.

## Current functionality

- Encrypted multiple BIP39 roots, BIP39 passphrase identities, compressed Bitcoin WIF import and independently password-protected roots.
- AES-256-GCM records and metadata, per-record random encryption keys, Argon2id master/password wrapping, atomic saves, portable encrypted backups.
- 12/24-word generation with exact manual-prefix bit preservation, OS randomness, or 128/256 raw entropy bits.
- Guided physical dice entry, undo/reset, complete selectable calculations, per-word verification, a bundled 2,048-word reference and independent verification instructions.
- BIP44/49/84/86 address derivation. Signing currently supports **single-key P2WPKH PSBTs with SIGHASH_ALL**.
- Bitcoin mainnet, Testnet3, Testnet4, signet and regtest network identities. Testnet4 is the default derivation choice.
- `.avpub` public accounts, `.avreq` requests, `.avsig` verified signed responses and `.avvault` encrypted backups.
- Payload-derived transaction review, key/address matching, expiring single-use approval, independent signature verification and explicit broadcast authorization.
- Explicit Esplora queries in Proxy/Combined mode. Provider genesis identity, full previous transactions, returned txids and UTXO ownership are checked.
- Startup/inactivity/focus locking, native Windows suspend/resume notifications, and a separate network-free build.

See [implementation status](./docs/implementation-status.md) for incomplete specification items, [security boundaries](./SECURITY.md), [architecture](./docs/architecture.md), and [Windows test instructions](./docs/testing.md).

## Development

Requirements: Node 24, Rust, a C/C++ toolchain and the Tauri platform prerequisites. Dependencies are locked in `Cargo.lock` and `package-lock.json`; Trust Wallet Core is pinned to a commit.

```bash
npm ci
npm run check
npm test
cargo test -p airvault-core
npm run test:ui
npm run tauri -- dev
```

The browser UI tests use a separate Vite entry and a Rust example process. They exercise actual vault encryption and signing in disposable directories. No debug route, mock IPC, test command or automation entry is imported into the production app. `npm run check:production` scans the built assets; the Windows build script also scans both executables.

Cross-building Windows from this WSL checkout uses `bash scripts/build-windows.sh`, the Rust Windows GNU target, MinGW and a local NSIS installation. On a configured native Windows build host, `npm run tauri -- build` uses the normal Tauri build workflow. See [testing/build notes](./docs/testing.md) for offline and cross-build details.

Linux desktop packaging and VM testing remain pending, as requested. Core tests and the isolated browser tests run in WSL; that is not a claim of Linux desktop qualification.
