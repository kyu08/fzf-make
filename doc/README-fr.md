<div align="center">

<img src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/logo.png" />

`fzf-make` est un outil en ligne de commmandes qui éxecute des cibles make en utilisant un Fuzzy Finder avec une fenêtre de prévisualisation

![License:MIT](https://img.shields.io/static/v1?label=License&message=MIT&color=blue&style=flat-square)
[![Latest Release](https://img.shields.io/github/v/release/kyu08/fzf-make?style=flat-square)](https://github.com/kyu08/fzf-make/releases/latest)
[![crates.io](https://img.shields.io/crates/v/fzf-make?style=flatflat-square)](https://crates.io/crates/fzf-make)

<p align="center">
    [<a href="../README.md">English</a>]
    [<a href="../doc/README-de.md">Français</a>]
    [French]
</p>

<img src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/demo.gif" />

</div>

# 🛠️ Fonctionnalitées
- Selectionner et éxecuter une cible make avec fzf
- Supporte les instructions `include`
- **(dévelopement planifié)** Supporte un fichier de configuration
- **(dévelopement planifié)** Historique des commandes

# 👓 Pré-requis
- [bat](https://github.com/sharkdp/bat) (Dans le futuer, nous prévoyons de le
faire aussi marcher avec `cat`, mais pour l'instant, il ne marche qu'avec `bat`.)

# 📦 Installation
## macOS
### Homebrew
Vous n'avez pas besoin d'installer `bat` car `fzf-make` l'installera automatiquement avec Homebrew.

```sh
# installer
brew tap kyu08/tap
brew install kyu08/tap/fzf-make
```

```sh
# Mise à jour
brew upgrade fzf-make
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
## Run `fzf-make`
1. Exectuez `fzf-make` dans le dossier qui possède un fichier make (le noms doit être l'un des suivant: `GNUmakefile`, `makefile`, `Makefile`)
2. Selectionnez la commande à éxecuter

## Commandes
| Commande | Sortie |
|--------|--------|
| `fzf-make` |  <img width="752" alt="help.png" src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/demo.png">|
| `fzf-make --help` / `fzf-make -h` / `fzf-make help` |  <img width="752" alt="help.png" src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/help.png">|
| `fzf-make --version` / `fzf-make -v` / `fzf-make version` | <img width="752" alt="version.png" src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/version.png"> |
| `fzf-make ${some_invalid_command}` | <img width="752" alt="invalid-arg.png" src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/invalid-arg.png"> |

# 💻 Dévelopment
1. Clonez ce dépôt
2. Changez le code
3. Lancez `make run`

Pour éxecuter les tests, lancez `make test`(requiert `nextest`). Ou juste lancez `cargo test`.

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
