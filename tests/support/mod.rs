#[cfg(unix)]
pub fn checksums() -> String {
    [
        "macos-arm64",
        "macos-x64",
        "linux-arm64",
        "linux-x64",
        "windows-x64.exe",
    ]
    .iter()
    .map(|p| {
        format!(
            "{}  sdev-v{}-{p}\n",
            "0".repeat(64),
            env!("CARGO_PKG_VERSION")
        )
    })
    .collect()
}
