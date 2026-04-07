use crate::error::{Error, Result};

/// Build the GitHub release tarball URL for the current platform.
pub fn download_url(tag: &str) -> Result<String> {
    let os = match std::env::consts::OS {
        "linux" => "linux",
        "macos" => "macos",
        "windows" => "windows",
        other => return Err(Error::UnsupportedPlatform(other.into())),
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "arm64",
        other => return Err(Error::UnsupportedPlatform(other.into())),
    };
    Ok(format!(
        "https://github.com/iotaledger/iota/releases/download/{tag}/iota-{tag}-{os}-{arch}.tgz"
    ))
}
