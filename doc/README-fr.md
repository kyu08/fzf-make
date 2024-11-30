<div align="center">

<img src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/logo.png" />

`fzf-make` is a command line tool that executes commands using fuzzy finder with preview window. Currently supporting **make**, **pnpm**.

![License:MIT](https://img.shields.io/static/v1?label=License&message=MIT&color=blue&style=flat-square)
[![Latest Release](https://img.shields.io/github/v/release/kyu08/fzf-make?style=flat-square)](https://github.com/kyu08/fzf-make/releases/latest)
[![crates.io](https://img.shields.io/crates/v/fzf-make?style=flatflat-square)](https://crates.io/crates/fzf-make)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/fzf-make)](https://crates.io/crates/fzf-make)

<p align="center">
    [<a href="../README.md">English</a>]
    [<a href="../doc/README-de.md">Français</a>]
    [French]
</p>

<img src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/demo.gif" />

</div>

# 🛠️ Fonctionnalitées
- Select and execute a make target or pnpm scripts using fuzzy-finder with a preview window by running `fzf-make`!
- Execute the last executed command(By running `fzf-make --repeat`.)
- Command history
- [make] Support `include` directive
-Support make, pnpm. **Scheduled to be developed: yarn, npm.** 
- **(Scheduled to be developed)** Support config file

# 👓 Pré-requis
- **(If you install fzf-make via a package manager other than Homebrew)** [bat](https://github.com/sharkdp/bat)
    - Dans le futuer, nous prévoyons de le faire aussi marcher avec `cat`, mais pour l'instant, il ne marche qu'avec `bat`.

# 📦 Installation
## macOS
### Homebrew
Vous n'avez pas besoin d'installer `bat` car `fzf-make` l'installera automatiquement avec Homebrew.

```sh
# installer
brew install kyu08/tap/fzf-make
```

```sh
# Mise à jour
brew update && brew upgrade fzf-make
```

## Arch Linux

`fzf-make` peut être installé depuis le [AUR](https://aur.archlinux.org/packages/fzf-make) en utilisant un [assistant AUR](https://wiki.archlinux.org/title/AUR_helpers_(Fran%C3%A7ais)). Par exemple:

```sh
paru -S fzf-make
```

## NixOS / Nix (gestionnaire de paquets)
`fzf-make` peut être lancé depuis le référentiel (dernière version)
```sh
nix run github:kyu08/fzf-make
```

Ou depuis les nixpkgs (channel >= 23.05)
```sh
nix run nixpkgs#fzf-make
```

> **Note**
> Vous devrez possblement activer les fonctionnalitées expérimentales. Dans ce cas, éxecutez la command ci-dessous pour les activer
> `echo "experimental-features = nix-command flakes" | tee  ~/.config/nix/nix.conf`

## Méthode indépendante du système d'exploitation
### Cargo
```sh
cargo install --locked fzf-make
```

# 💡 Usage
## Run target using fuzzy finder
1. Execute `fzf-make` in the directory you want to run make target, or pnpm scripts.
1. Selectionnez la commande à éxecuter. If you type some characters, the list will be filtered.
    <img width="752" alt="demo" src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/usage-main.png"> 
    <img width="752" alt="demo" src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/usage-type-characters.png"> 

## Run target from history
1. Execute `fzf-make` in the directory you want to run make target, or pnpm scripts.
1. Press `Tab` to move to the history list
1. Select make command you want to execute.
    <img width="752" alt="demo" src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/usage-history.png"> 

## How fzf-make judges which command runner can be used
### make
Whether makefile(file name should be one of `GNUmakefile`, `makefile`, `Makefile`) is in the current directory.

### pnpm
Whether `package.json` and `pnpm-lock.yaml` are in the current directory.

## Commands Supported
| Command                                                   | Description                                   |
| --------                                                  | --------                                      |
| `fzf-make`                                                | Launch fzf-make                               |
| `fzf-make --repeat` / `fzf-make -r` / `fzf-make repeat`   | Execute last executed target                  |
| `fzf-make --history` / `fzf-make -h` / `fzf-make history` | Launch fzf-make with the history pane focused |
| `fzf-make --help` / `fzf-make help`                       | Show help                                     |
| `fzf-make --version` / `fzf-make -v` / `fzf-make version` | Show version                                  |

# 💻 Dévelopment
1. Clonez ce dépôt
2. Changez le code
3. Lancez `make run`

Pour éxecuter les tests, lancez `make test`(requiert `nextest`).

## nix
Ou vous pouvez utiliser `nix` pour créer un interpreteur de commande avec les dépendances du projet.

À la racine le dépôt, éxecutez la commande ci-dessous:
```nix
nix develop
```

# 👥 Contribution
- Les contributions sont bienvenues!
- Si vous avez une demande de fonctionnalité, merci d'ouvrir une issue en premier.
- Si vous ajoutez fzf-make à un gestionnaire de paquets, merci de me le faire savoir. (ou envoyez une demande de tirage (pr) pour ajoutez les instructions d'installation avec le gestionnaire dans `README.md`)
- Si vous avez des questions, n'hésitez pas à les demander à travers une issue.

# 🗒 Article(s) liés
- (Japanese)[Makefileに定義されたtargetをfzfで選択して実行するCLIツールをRustでつくった](https://blog.kyu08.com/posts/fzf-make)
