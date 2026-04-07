# iotaup

A tiny, rustup-like version manager for [IOTA](https://github.com/iotaledger/iota). Unofficial and not affiliated with the IOTA Foundation. Download, switch between, and manage multiple IOTA releases (including release candidates) without touching your shell config.

> **Note:** iotaup is aimed at developers who need to jump between IOTA toolchain versions locally or in CI. It is **not** intended for validator operators or other production node deployments, where you should follow the official installation and upgrade guidance instead.

## Setup

```sh
cargo install --git https://github.com/marc2332/iotaup
export PATH="$HOME/.iotaup/bin:$PATH"
```

Add that `export` line to your shell config (`~/.bashrc`, `~/.zshrc`, etc.) so it persists across sessions. Everything lives under `~/.iotaup` (override with `IOTAUP_HOME`). Run `iotaup self path` to print the bin directory.

## Install and activate a release

Install a stable release:

```sh
iotaup install 1.19.1
```

Install a release candidate (any `-rc.N` tag works):

```sh
iotaup install 1.20.0-rc.1
```

The first version you install is activated automatically. Switch between installed versions at any time:

```sh
iotaup use 1.20.0-rc.1
iota --version
```

List what you have installed (the active one is marked with `*`):

```sh
iotaup list
#   v1.19.1
# * v1.20.0-rc.1
```

Print the path of the active version directory:

```sh
iotaup which
```

Remove a version (use `-f` / `--force` to remove the active one):

```sh
iotaup uninstall 1.19.1
```

## CI usage

Example GitHub Actions step that installs a pinned IOTA release and puts it on `PATH` for the rest of the job:

```yaml
- name: Install IOTA
  run: |
    # Pin to a specific commit (--rev <sha>) or tag (--tag vX.Y.Z) for reproducible, auditable CI builds.
    cargo install --git https://github.com/marc2332/iotaup
    iotaup install 1.19.1
    echo "$HOME/.iotaup/bin" >> "$GITHUB_PATH"

- name: Use it
  run: iota --version
```

The version string can be any tag from `iotaledger/iota`, including release candidates (`1.20.0-rc.1`). Pin it explicitly so CI is reproducible.

## How it works

```
~/.iotaup/
├── bin/                       # on your PATH (stable symlinks)
│   ├── iota -> ../active/iota
│   └── ...
├── versions/
│   ├── v1.19.1/               # extracted release tarballs
│   └── v1.20.0-rc.1/
└── active -> versions/v1.20.0-rc.1
```

`iotaup use` retargets the `active` symlink and refreshes `bin/`, so switching is instant and shell-independent.

## License

MIT
