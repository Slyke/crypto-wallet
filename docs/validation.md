# Validation record

Initial Windows preview tested on 2026-10-12/13.

- TypeScript/Svelte: zero errors and warnings.
- JavaScript: 10 unit tests, including all 2,048 BIP39 dice indices, invalid rolls/counts, undo/reset and the official wordlist SHA-256.
- Native core: encryption/password/backup tests; published address vectors; malformed PSBT, unsafe sighash, input substitution and response tampering rejection; independent signature verification; no-secret Proxy mode; cancellation, expiry and one-time display lifecycle.
- Independent entropy reconstruction verifies every allowed prefix count, leading zero bits, 12/24-word checksums and all-dice entropy.
- Browser UI: four end-to-end flows against the actual Rust core, using a separate test-only transport and temporary profiles.
- Bitcoin Core 31.1 regtest: generated transaction accepted by testmempoolaccept, broadcast and mined into a block with confirmation.
- Testnet4: a new disposable random wallet's provider/genesis/address query passed. It has no funds; public-network broadcast has not been exercised.
- Native Windows: the application renders, and a disposable vault was created through actual WebView2/Tauri IPC using the computer-use MCP. Packaged loader was verified with a valid Microsoft Authenticode signature.
- Release check: compiled normal and networking-excluded Windows binaries. Test-bridge markers and mock transport hooks are absent from release assets and binaries.
- Dependency review: npm audit zero vulnerabilities. Cargo audit zero vulnerability entries, six unmaintained warnings and two unsoundness warnings; see SECURITY.md.

A packaging smoke test found the portable EXE initially lacked its sibling WebView2Loader.dll. The portable packager now requires and copies it, supplies a complete ZIP, and hashes it. The NSIS installer already included the loader through Tauri's GNU target packaging.

Windows sleep/resume is handled by the native power API but a real machine sleep cycle was not triggered during automated testing. Installer upgrade/uninstall and Linux desktop VM testing remain pending.
