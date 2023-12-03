export
RUST_BACKTRACE=full

.PHONY: toooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo-long-target2
toooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo-long-target2:
	@echo "this is toooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo-long-target."

.PHONY: test
test : # Run unit tests
	RUST_BACKTRACE=full cargo nextest run

# Install tools if not installed.
.PHONY: tools
tools:
	@if ! which cargo-nextest > /dev/null; then \
		cargo install cargo-nextest; \
	fi
	@if ! which cargo-set-version > /dev/null; then \
		cargo install cargo-edit; \
	fi

.PHONY: run
run:
	@cargo run

.PHONY: run-ratatui
run-ratatui:
	@RUST_BACKTRACE=full cargo run -- -r

.PHONY: build
build:
	@cargo build

.PHONY: check
 check:
	@cargo clippy -- -D warnings

.PHONY: build-release
build-release:
	@cargo build --verbose --release

.PHONY: bump-fzf-make-version
bump-fzf-make-version: tools
	cargo set-version --bump minor; \

# Targets for test
include ./makefiles/test.mk

.PHONY: toooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo-long-target
toooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo-long-target:
	@echo "this is toooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo-long-target."

.PHONY: echo-greeting
echo-greeting:
	@echo hello fzf-make!

.PHONY: cmd
cmd:
	@read -p "Do something? y/n:" ans; \
	if [ "$$ans" = y ]; then  \
		echo "Doing something..."; \
	fi
