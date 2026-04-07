# iotaup

A tiny, rustup-like version manager for [IOTA](https://github.com/iotaledger/iota). Download, switch between, and manage multiple IOTA releases (including release candidates) without touching your shell config.

## Install the tool

```sh
cargo install --git https://github.com/marc2332/iotaup
```

This drops the `iotaup` binary into `~/.cargo/bin` (which should already be on your `PATH` if you use Rust).

## Configure your PATH (one-time)

iotaup keeps everything under `~/.iotaup` (override with `IOTAUP_HOME`). Activated IOTA binaries are exposed as stable symlinks in `~/.iotaup/bin`. Add that directory to your `PATH` once and you never need to touch your shell config again when switching versions.

```sh
# bash / zsh
echo 'export PATH="$HOME/.iotaup/bin:$PATH"' >> ~/.bashrc

# fish
fish_add_path "$HOME/.iotaup/bin"
```

You can ask iotaup where that directory is:

```sh
iotaup self path
# /home/you/.iotaup/bin
```

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
