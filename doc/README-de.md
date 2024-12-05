<div align="center">

<img src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/logo.png" />

`fzf-make` is a command line tool that executes commands using fuzzy finder with preview window. Currently supporting **make**, **pnpm**.

![License:MIT](https://img.shields.io/static/v1?label=License&message=MIT&color=blue&style=flat-square)
[![Latest Release](https://img.shields.io/github/v/release/kyu08/fzf-make?style=flat-square)](https://github.com/kyu08/fzf-make/releases/latest)
[![crates.io](https://img.shields.io/crates/v/fzf-make?style=flatflat-square)](https://crates.io/crates/fzf-make)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/fzf-make)](https://crates.io/crates/fzf-make)

<p align="center">
    [<a href="../README.md">English</a>]
    [Deutsch]
    [<a href="../doc/README-fr.md">Français</a>]
</p>

<img src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/demo.gif" />

</div>

# 🛠️ Eigenschaften
- Select and execute a make target or pnpm scripts using fuzzy-finder with a preview window by running `fzf-make`!
- Execute the last executed command(By running `fzf-make --repeat`.)
- Command history
- Support make, pnpm. **Scheduled to be developed: yarn, npm.** 
- [make] Support `include` directive
- [pnpm] Support workspace(collect scripts all of `package.json` in the directory where fzf-make is launched.)
- **(Scheduled to be developed)** Support config file

# 👓 Voraussetzungen
- **(If you install fzf-make via a package manager other than Homebrew)** [bat](https://github.com/sharkdp/bat)
    - Für die Zukunft ist geplant, dass es auch mit `cat` funktioniert, aber derzeit funktioniert es nur mit `bat`.

# 📦 Installation
## macOS
### Homebrew
Man braucht `bat` nicht zu installieren, da `fzf-make` es automatisch über Homebrew installiert.

```sh
# install
brew install kyu08/tap/fzf-make
```

```sh
# update 
brew update && brew upgrade fzf-make
```

## Arch Linux

`fzf-make` kann aus dem [AUR](https://aur.archlinux.org/packages/fzf-make) mit Hilfe eines [AUR-Helpers](https://wiki.archlinux.org/title/AUR_helpers) installiert werden. Zum Beispiel:

```sh
paru -S fzf-make
```

## NixOS / Nix (package manager)
`fzf-make` kann aus dem Repository ausgeführt werden (neueste Version)
```sh
nix run github:kyu08/fzf-make
```

Oder nixpkgs (channel >= 23.05)
```sh
nix run nixpkgs#fzf-make
```

> **Note**
> Möglicherweise müssen die experimentellen Funktionen aktiviert werden. Folgender Befehl muss ausgeführt werden, um sie zu aktivieren:
> `echo "experimental-features = nix-command flakes" | tee  ~/.config/nix/nix.conf`

## OS-unabhängige Methode
### Cargo
```sh
cargo install --locked fzf-make
```

# 💡 Nutzung
## Run target using fuzzy finder
1. Execute `fzf-make` in the directory you want to run make target, or pnpm scripts.
1. Command auswählen, welches ausgeführt werden soll. If you type some characters, the list will be filtered.
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

# 💻 Entwicklung
1. Dieses repository klonen
2. Codes ändern
3. `make run` ausführen

Um den Test auszuführen, führe `make test` (benötigt `nextest`) aus.

## nix
Oder man kann `nix` verwenden, um eine Entwicklungsshell mit den Dependencies zu erstellen.

Führe im Stammverzeichnis des Repo den folgenden Befehl aus:
```nix
nix develop
```

# 👥 Contribution
- Contributions sind willkommen!
- Wenn du eine Funktionsanfrage hast, erstelle bitte zuerst ein Issue.
- Wenn du fzf-make zu einem Paketmanager hinzugefügt hast, lass es mich bitte wissen. (oder sende bitte einen PR, um die Installation über den Paketmanager in die `README.md` aufzunehmen)
- Wenn Fragen bestehen, gerne einfach ein Issue erstellen und fragen.

# 🗒 Verwandte Artikel
- [fzf-make - A command runner with fuzzy finder and preview window for make, pnpm - reddit](https://www.reddit.com/r/commandline/comments/1h7btkl/fzfmake_a_command_runner_with_fuzzy_finder_and/)
- (Japanese)[[make, pnpmに対応]タスクランナーのコマンドをfuzzy finder形式で選択できるCLIツール fzf-makeの紹介](https://zenn.dev/kyu08/articles/974fd8bc25c303)
- (Japanese)[Makefileに定義されたtargetをfzfで選択して実行するCLIツールをRustでつくった](https://blog.kyu08.com/posts/fzf-make)
