<div align="center">

<img src="./static/fzf-make-logo.png" />

`fzf-make` is the command line tool that execute make command using fzf.

![License:MIT](https://img.shields.io/static/v1?label=License&message=MIT&color=blue&style=flat-square)
[![Latest Release](https://img.shields.io/github/v/release/kyu08/fzf-make?style=flat-square)](https://github.com/kyu08/fzf-make/releases/latest)

![fzf-make-demo](https://user-images.githubusercontent.com/49891479/228574753-2e0e46b8-b446-4b7d-b866-2362f33c9e17.gif)

</div>

# 🔧 Installation
```sh
brew tap kyu08/tap
brew install kyu08/tap/fzf-make
```

## ✨ How to update
```sh
brew update
brew upgrade fzf-make
```

## 💻Develop

### nix

> You can create a development environment using nix!

```sh
nix develop
```

> You can also build a local derivation.

```sh
nix build .
```

> Or run it from your terminal

```sh
nix run github:kyu08/fzf-make
```

# ⚠️ Caution
- The following format targets are supported(contributions are welcome!)
  - `^[^.#\s\t].+:.*$`
- File name is only supported for `Makefile`. (File names in formats such as `xxx.mk` are not supported.)
- This command run only on a apple silicon machine.

# 💡 Usage
1. execute `fzf-make` in the directory include `Makefile`
1. select make command you want to execute

# 🗒 Related Article(s)
- [Makefileに定義されたtargetをfzfで選択して実行するCLIツールをRustでつくった](https://blog.kyu08.com/posts/fzf-make)
