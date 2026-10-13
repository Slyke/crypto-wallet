from pathlib import Path
import hashlib, zipfile, shutil
root=Path(__file__).resolve().parents[1]
out=root/'artifacts/windows'
loader=root/'target/x86_64-pc-windows-gnu/release/WebView2Loader.dll'
if not loader.is_file(): raise SystemExit('WebView2Loader.dll missing: refusing incomplete Windows package')
shutil.copy2(loader, out/'WebView2Loader.dll')
for source in ['README.md','SECURITY.md','docs/testing.md','docs/implementation-status.md','THIRD_PARTY_NOTICES.md']:
    (out/source).parent.mkdir(parents=True,exist_ok=True)
    shutil.copy2(root/source, out/source)
with zipfile.ZipFile(out/'AirVault_0.1.0_windows-x64-portable.zip','w',zipfile.ZIP_DEFLATED) as archive:
    for name in ['AirVault.exe','AirVault-offline.exe','WebView2Loader.dll','README.md','SECURITY.md','docs/testing.md','docs/implementation-status.md','THIRD_PARTY_NOTICES.md']:
        archive.write(out/name, 'AirVault/'+name)
files=sorted([*out.glob('*.exe'), *out.glob('*.dll'), *out.glob('*.zip')])
(out/'SHA256SUMS.txt').write_text(''.join(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.name+'\n' for p in files))
print('Complete Windows portable ZIP and SHA256SUMS created.')
