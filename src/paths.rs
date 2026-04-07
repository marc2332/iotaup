use std::fs;
use std::path::PathBuf;

use crate::error::{Error, Result};

pub struct Paths {
    pub home: PathBuf,
    pub bin: PathBuf,
    pub versions: PathBuf,
    pub active: PathBuf,
}

impl Paths {
    pub fn resolve() -> Result<Self> {
        let home = if let Some(custom) = std::env::var_os("IOTAUP_HOME") {
            PathBuf::from(custom)
        } else {
            let base = std::env::var_os("HOME")
                .map(PathBuf::from)
                .ok_or_else(|| Error::Msg("HOME is not set".into()))?;
            base.join(".iotaup")
        };
        Ok(Self {
            bin: home.join("bin"),
            versions: home.join("versions"),
            active: home.join("active"),
            home,
        })
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.bin)?;
        fs::create_dir_all(&self.versions)?;
        Ok(())
    }
}
