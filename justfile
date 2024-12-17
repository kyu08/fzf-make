#!/usr/bin/env -S just --justfile

test:
  cargo test --all

[group: 'misc']
run:
  cargo run

[group: 'misc']
build:
  cargo build

[group: 'misc']
fmt:
  cargo fmt --all

[group: 'misc']
[private]
fmt-private:
  cargo fmt --all

# everyone's favorite animate paper clip
[group: 'check']
clippy:
  cargo clippy --all --all-targets --all-features -- --deny warnings
