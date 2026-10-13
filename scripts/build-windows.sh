#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export NVM_DIR="$HOME/.nvm"
source "$NVM_DIR/nvm.sh"
nvm use 24 >/dev/null
npm run check
npm test
npm run build
cargo test -p airvault-core --release
cargo build -p airvault --release --target x86_64-pc-windows-gnu
mkdir -p artifacts/windows
cp target/x86_64-pc-windows-gnu/release/airvault.exe artifacts/windows/AirVault.exe
cp target/x86_64-pc-windows-gnu/release/WebView2Loader.dll artifacts/windows/WebView2Loader.dll
node scripts/check-production.mjs artifacts/windows/AirVault.exe
bash scripts/bundle-windows.sh
cp target/x86_64-pc-windows-gnu/release/bundle/nsis/AirVault_0.1.0_x64-setup.exe artifacts/windows/
cargo build -p airvault --release --target x86_64-pc-windows-gnu --no-default-features --features custom-protocol
cp target/x86_64-pc-windows-gnu/release/airvault.exe artifacts/windows/AirVault-offline.exe
node scripts/check-production.mjs artifacts/windows/AirVault-offline.exe
python3 scripts/package-windows.py
