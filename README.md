<div align="center">

<img src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/logo.png" />

`fzf-make` is a command line tool that executes make target using fuzzy finder with preview window.

![License:MIT](https://img.shields.io/static/v1?label=License&message=MIT&color=blue&style=flat-square)
[![Latest Release](https://img.shields.io/github/v/release/kyu08/fzf-make?style=flat-square)](https://github.com/kyu08/fzf-make/releases/latest)
[![crates.io](https://img.shields.io/crates/v/fzf-make?style=flatflat-square)](https://crates.io/crates/fzf-make)

<p align="center">
    [English]
    [<a href="doc/README-de.md">Deutsch</a>]
    [<a href="doc/README-fr.md">Français</a>]
</p>

<img src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/demo.gif" />

</div>

# 🛠️ Features
- Select and execute a make target using fzf
- Support `include` directive
- **(Scheduled to be developed)** Support config file
- **(Scheduled to be developed)** Command history

# 👓 Prerequisites
- [bat](https://github.com/sharkdp/bat) (In the future, we intend to make it work with `cat` as well, but currently it only works with `bat`.)

# 📦 Installation
## macOS
### Homebrew
You don't need to install `bat` because `fzf-make` will install it automatically via Homebrew.

```sh
# install
brew tap kyu08/tap
brew install kyu08/tap/fzf-make
```

```sh
# update 
brew upgrade fzf-make
```

## Arch Linux

`fzf-make` can be installed from the [AUR](https://aur.archlinux.org/packages/fzf-make) using an [AUR helper](https://wiki.archlinux.org/title/AUR_helpers). For example:

```sh
paru -S fzf-make
```

## NixOS / Nix (package manager)
`fzf-make` can be run from the repository (latest version)
```sh
nix run github:kyu08/fzf-make
```

Or from the nixpkgs (channel >= 23.05)
```sh
nix run nixpkgs#fzf-make
```

> **Note**
> You may need to enable experimental feature. In that case, execute the following command to enable them
> `echo "experimental-features = nix-command flakes" | tee  ~/.config/nix/nix.conf`

## OS-independent method
### Cargo
```sh
cargo install fzf-make
```

# 💡 Usage
## Run `fzf-make`
1. Execute `fzf-make` in the directory include makefile(file name should be one of `GNUmakefile`, `makefile`, `Makefile`)
1. Select make command you want to execute

## Commands
| Command | Output |
|--------|--------|
| `fzf-make` |  <img width="752" alt="help.png" src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/demo.png">|
| `fzf-make --help` / `fzf-make -h` / `fzf-make help` |  <img width="752" alt="help.png" src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/help.png">|
| `fzf-make --version` / `fzf-make -v` / `fzf-make version` | <img width="752" alt="version.png" src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/version.png"> |
| `fzf-make --old` / `fzf-make -o` / `fzf-make old` | <img width="752" alt="version.png" src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/old.png"> |
| `fzf-make ${some_invalid_command}` | <img width="752" alt="invalid-arg.png" src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/invalid-arg.png"> |

# 💻 Development
1. Clone this repository
1. Change the codes
1. Run `make run`

To execute test, run `make test`(needs `nextest`). Or just run `cargo test`.

## nix
Or you can use `nix` to create a development shell with the project dependencies.

Within the repo root, execute the following command:
```nix
nix develop
```

# 👥 Contribution
- Contributions are welcome!
- If you have a Feature request, please create an issue first.
- If you have added fzf-make to some package manager, please let me know. (or please send a PR to add how to install via the package manager in the `README.md`)
- If you have any questions, feel free to create an issue and ask.

# 🗒 Related Article(s)
- (Japanese)[Makefileに定義されたtargetをfzfで選択して実行するCLIツールをRustでつくった](https://blog.kyu08.com/posts/fzf-make)
