use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use crate::error::Result;
use crate::paths::Paths;

/// Point `active` at versions/<tag> and rebuild bin/ symlinks.
pub fn activate(home: &Paths, tag: &str) -> Result<()> {
    let target = home.versions.join(tag);
    let rel = Path::new("versions").join(tag);

    let tmp = home.home.join(".active.tmp");
    let _ = fs::remove_file(&tmp);
    unix_fs::symlink(&rel, &tmp)?;
    fs::rename(&tmp, &home.active)?;

    rebuild_bin(home, &target)?;
    Ok(())
}

/// Remove all current symlinks in bin/ and re-create one per regular file in `target`.
fn rebuild_bin(home: &Paths, target: &Path) -> Result<()> {
    clear_bin(home)?;
    for entry in fs::read_dir(target)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_file() {
            continue;
        }
        let name = entry.file_name();
        let link = home.bin.join(&name);
        let _ = fs::remove_file(&link);
        // bin/<name> -> ../active/<name>
        let dest = Path::new("..").join("active").join(&name);
        unix_fs::symlink(&dest, &link)?;
    }
    Ok(())
}

/// Remove every symlink in bin/ that points into the iotaup home.
pub fn clear_bin(home: &Paths) -> Result<()> {
    if !home.bin.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(&home.bin)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_symlink() {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}
