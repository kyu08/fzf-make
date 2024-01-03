export
RUST_BACKTRACE=full

.PHONY: toooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo-long-target2
toooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo-long-target2:
	@echo "this is toooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo-long-target."

.PHONY: ci
ci: # Checks same as CI
	RUST_BACKTRACE=full make test; \
	make check; \
	make spell-check

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
	@if ! which typos > /dev/null; then \
		cargo install typos-cli; \
	fi

.PHONY: run
run:
	@cargo run

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
	@cargo set-version --bump minor; \
	export CURRENT_VERSION=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'); \
	git checkout -b "release_v$${CURRENT_VERSION}"; \
	git add .; \
	git commit -m "Bump fzf-make's version to v$${CURRENT_VERSION}"; \
	git push origin "release_v$${CURRENT_VERSION}"; \
	gh pr create --fill --web

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

.PHONY: spell-check
spell-check:
	typos
