use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

mod error;
mod paths;
mod platform;
mod progress;
mod symlinks;
mod version;

use error::{Error, Result};

#[derive(Parser)]
#[command(
    name = "iotaup",
    version,
    about = "rustup-like manager for IOTA releases"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Download and install a release (e.g. 1.19.1, 1.20.0-rc.1)
    Install {
        version: String,
        /// Do not activate the release after installing
        #[arg(long)]
        no_default: bool,
    },
    /// Activate an installed version
    Use { version: String },
    /// List installed versions
    List,
    /// Remove an installed version
    Uninstall {
        version: String,
        #[arg(short, long)]
        force: bool,
    },
    /// Print the path of the active version directory
    Which,
    /// List the tools available in the active release
    Ls,
    /// Self-management commands
    #[command(subcommand)]
    Self_(SelfCmd),
}

#[derive(Subcommand)]
enum SelfCmd {
    /// Print the bin directory to add to PATH
    Path,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let home = paths::Paths::resolve()?;
    home.ensure_dirs()?;

    match cli.cmd {
        Cmd::Install {
            version,
            no_default,
        } => install(&home, &version, !no_default),
        Cmd::Use { version } => use_version(&home, &version),
        Cmd::List => list(&home),
        Cmd::Uninstall { version, force } => uninstall(&home, &version, force),
        Cmd::Which => which(&home),
        Cmd::Ls => ls_tools(&home),
        Cmd::Self_(SelfCmd::Path) => {
            println!("{}", home.bin.display());
            Ok(())
        }
    }
}

fn install(home: &paths::Paths, version: &str, activate: bool) -> Result<()> {
    let tag = version::normalize(version)?;
    let dest = home.versions.join(&tag);
    if dest.exists() {
        println!("{tag} is already installed");
    } else {
        let url = platform::download_url(&tag)?;
        println!("downloading {url}");
        download_and_extract(&url, &dest, &tag)?;
        println!("installed {tag}");
    }
    if activate {
        symlinks::activate(home, &tag)?;
        println!("activated {tag}");
    }
    Ok(())
}

fn use_version(home: &paths::Paths, version: &str) -> Result<()> {
    let tag = version::normalize(version)?;
    if !home.versions.join(&tag).is_dir() {
        return Err(Error::NotInstalled(tag));
    }
    symlinks::activate(home, &tag)?;
    println!("activated {tag}");
    Ok(())
}

fn list(home: &paths::Paths) -> Result<()> {
    let active = active_tag(home);
    let mut entries: Vec<String> = match fs::read_dir(&home.versions) {
        Ok(read_dir) => read_dir
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|file_type| file_type.is_dir()).unwrap_or(false))
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect(),
        Err(_) => Vec::new(),
    };
    entries.sort();
    if entries.is_empty() {
        println!("no versions installed. try: iotaup install <version>");
        return Ok(());
    }
    for name in entries {
        let marker = if Some(&name) == active.as_ref() {
            "*"
        } else {
            " "
        };
        println!("{marker} {name}");
    }
    Ok(())
}

fn uninstall(home: &paths::Paths, version: &str, force: bool) -> Result<()> {
    let tag = version::normalize(version)?;
    let dir = home.versions.join(&tag);
    if !dir.is_dir() {
        return Err(Error::NotInstalled(tag));
    }
    if active_tag(home).as_deref() == Some(tag.as_str()) && !force {
        return Err(Error::Msg(format!(
            "{tag} is currently active; pass --force to remove it"
        )));
    }
    fs::remove_dir_all(&dir)?;
    if active_tag(home).as_deref() == Some(tag.as_str()) {
        let _ = fs::remove_file(&home.active);
        symlinks::clear_bin(home)?;
    }
    println!("removed {tag}");
    Ok(())
}

fn ls_tools(home: &paths::Paths) -> Result<()> {
    if !home.active.exists() {
        return Err(Error::Msg("no version is active".into()));
    }
    let dir = fs::canonicalize(&home.active)?;
    let mut tools: Vec<String> = fs::read_dir(&dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|file_type| file_type.is_file()).unwrap_or(false))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    tools.sort();
    if tools.is_empty() {
        println!("active release contains no tools");
        return Ok(());
    }
    for tool in tools {
        println!("{tool}");
    }
    Ok(())
}

fn which(home: &paths::Paths) -> Result<()> {
    if !home.active.exists() {
        return Err(Error::Msg("no version is active".into()));
    }
    let resolved = fs::canonicalize(&home.active)?;
    println!("{}", resolved.display());
    Ok(())
}

fn active_tag(home: &paths::Paths) -> Option<String> {
    let target = fs::read_link(&home.active).ok()?;
    Some(target.file_name()?.to_string_lossy().into_owned())
}

fn download_and_extract(url: &str, dest: &Path, tag: &str) -> Result<()> {
    let response = match ureq::get(url).call() {
        Ok(response) => response,
        Err(ureq::Error::Status(404, _)) => return Err(Error::VersionNotFound(tag.to_string())),
        Err(ureq::Error::Status(code, _)) => {
            return Err(Error::Msg(format!("HTTP {code} downloading {url}")));
        }
        Err(err) => return Err(Error::Msg(format!("network error: {err}"))),
    };

    let tmp: PathBuf = dest.with_file_name(format!(".{tag}.tmp"));
    if tmp.exists() {
        fs::remove_dir_all(&tmp)?;
    }
    fs::create_dir_all(&tmp)?;

    let total = response
        .header("Content-Length")
        .and_then(|header| header.parse::<u64>().ok());
    let progressed = progress::ProgressReader::new(response.into_reader(), total);
    let gzip = flate2::read::GzDecoder::new(progressed);
    let mut archive = tar::Archive::new(gzip);
    if let Err(err) = archive.unpack(&tmp) {
        let _ = fs::remove_dir_all(&tmp);
        return Err(Error::Msg(format!("failed to extract archive: {err}")));
    }

    // Flatten if the archive wraps everything in a single top-level dir.
    let flattened = maybe_flatten(&tmp)?;
    fs::rename(&flattened, dest).map_err(|err| {
        let _ = fs::remove_dir_all(&tmp);
        Error::Io(err)
    })?;
    if tmp.exists() && tmp != *dest {
        let _ = fs::remove_dir_all(&tmp);
    }
    Ok(())
}

fn maybe_flatten(tmp: &Path) -> io::Result<PathBuf> {
    let mut entries: Vec<_> = fs::read_dir(tmp)?.filter_map(|entry| entry.ok()).collect();
    if entries.len() == 1 {
        let only = entries.remove(0);
        if only.file_type()?.is_dir() {
            return Ok(only.path());
        }
    }
    Ok(tmp.to_path_buf())
}
