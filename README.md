<div align="center">

# 🏭 fzf-make

`fzf-make` is the command line tool that execute make command using fzf.

![License:MIT](https://img.shields.io/static/v1?label=License&message=MIT&color=blue&style=flat-square)
[![Latest Release](https://img.shields.io/github/v/release/kyu08/fzf-make?style=flat-square)](https://github.com/kyu08/fzf-make/releases/latest)

![fzf-make-demo](https://user-images.githubusercontent.com/49891479/228574753-2e0e46b8-b446-4b7d-b866-2362f33c9e17.gif)

</div>

# 🔧 Installation
🚨 This command run only on a apple silicon machine.

```sh
brew tap kyu08/tap
brew install kyu08/tap/fzf-make
```

# ⚠️ Caution
- The following format targets are supported(contributions are welcome!)
  - `^[^.#\s].+:$`
- File name is only supported for `Makefile`. (File names in formats such as `xxx.mk` are not supported.)

# 💡 Usage
1. execute `fzf-make` in the directory include `Makefile`
1. select make command you want to execute
