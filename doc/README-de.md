<div align="center">

<img src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/logo.png" />

`fzf-make` ist ein Kommandozeilenwerkzeug, das make target unter Verwendung des Fuzzy Finders mit Vorschaufenster ausführt.

![License:MIT](https://img.shields.io/static/v1?label=License&message=MIT&color=blue&style=flat-square)
[![Latest Release](https://img.shields.io/github/v/release/kyu08/fzf-make?style=flat-square)](https://github.com/kyu08/fzf-make/releases/latest)
[![crates.io](https://img.shields.io/crates/v/fzf-make?style=flatflat-square)](https://crates.io/crates/fzf-make)

<p align="center">
    [<a href="../README.md">English</a>]
    [Deutsch]
    [<a href="../doc/README-fr.md">Français</a>]
</p>

<img src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/demo.gif" />

</div>

# 🛠️ Eigenschaften
- Auswählen und Ausführen eines Make-Targets mit fuzzy-finder
- Execute last executed target(By running `fzf-make --repeat` without any arguments.)
- Command-Verlauf / History
- Unterstützt `include` directive
- **(In Entwicklung)** Unterstützt Konfigurations-Dateien

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
brew update
brew upgrade fzf-make
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
1. Führe `fzf-make` in dem Verzeichnis aus, das makefile enthält (der Dateiname sollte einer von `GNUmakefile`, `makefile`, `Makefile` sein)
1. Make-Command auswählen, welches ausgeführt werden soll. If you type some characters, the list will be filtered.
    <img width="752" alt="demo" src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/usage-main.png"> 
    <img width="752" alt="demo" src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/usage-type-characters.png"> 

## Run target from history
1. Execute `fzf-make` in the directory include makefile(file name should be one of `GNUmakefile`, `makefile`, `Makefile`)
1. Press `Tab` to move to the history list
1. Select make command you want to execute.
    <img width="752" alt="demo" src="https://raw.githubusercontent.com/kyu08/fzf-make/main/static/usage-history.png"> 

## Commands Supported
| Command | Description |
|--------|--------|
| `fzf-make` | Launch fzf-make |
| `fzf-make --repeat` / `fzf-make -r` / `fzf-make repeat` |  Execute last executed target |
| `fzf-make --help` / `fzf-make -h` / `fzf-make help` | Show help |
| `fzf-make --version` / `fzf-make -v` / `fzf-make version` | Show version |

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
- (Japanese)[Makefileに定義されたtargetをfzfで選択して実行するCLIツールをRustでつくった](https://blog.kyu08.com/posts/fzf-make)
