fn main() {
    let hash = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_else(|| "uncommitted".into());
    println!("cargo:rustc-env=AIRVAULT_BUILD_HASH={hash}");
    tauri_build::build();
}
