# Checks same as CI
.PHONY: ci
ci: test-ci check fmt-check detect-unused-dependencies check-licenses update-license-file spell-check

.PHONY: run
run:
	@cargo run

.PHONY: tools
tools: tool-test tool-bump-version tool-spell-check

.PHONY: tool-test
tool-test:
	@if ! which cargo-nextest > /dev/null; then \
		cargo install --locked cargo-nextest; \
	fi

.PHONY: tool-bump-version
tool-bump-version:
	@if ! which cargo-set-version > /dev/null; then \
		cargo install --locked cargo-edit; \
	fi

.PHONY: tool-spell-check
tool-spell-check:
	@if ! which typos > /dev/null; then \
		cargo install --locked typos-cli; \
	fi

.PHONY: tool-detect-unused-dependencies
tool-detect-unused-dependencies:
	@if ! which cargo-machete > /dev/null; then \
		cargo install --locked cargo-machete; \
	fi

.PHONY: tool-check-licenses
tool-check-licenses:
	@if ! which cargo-deny > /dev/null; then \
		cargo install --locked cargo-deny; \
	fi

.PHONY: tool-update-license-file
tool-update-license-file:
	@if ! which cargo-about > /dev/null; then \
		cargo install --locked cargo-about; \
	fi

.PHONY: test-ci # for CI
test-ci:
	RUST_BACKTRACE=full FZF_MAKE_IS_TESTING=true cargo test

TEST_HISTORY_DIR = ./test_data/history
.PHONY: test
test: tool-test
	rm -rf $(TEST_HISTORY_DIR)
	RUST_BACKTRACE=full FZF_MAKE_IS_TESTING=true cargo nextest run

.PHONY: test-watch
test-watch: tool-test
	rm -rf $(TEST_HISTORY_DIR)
	RUST_BACKTRACE=full FZF_MAKE_IS_TESTING=true cargo watch -x "nextest run"

.PHONY: run-watch
run-watch:
	rm -rf $(TEST_HISTORY_DIR)
	RUST_BACKTRACE=full FZF_MAKE_IS_TESTING=true cargo watch -x "run"

.PHONY: bump-fzf-make-version
bump-fzf-make-version: tool-bump-version
	@read -p "Really bump fzf-make version? y/n:" ans; \
	if [ "$$ans" = y ]; then  \
		git checkout main; \
		git pull; \
		cargo set-version --bump minor; \
		export CURRENT_VERSION=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'); \
		make update-license-file; \
		git add .; \
		git commit -m "chore(release): bump to v$${CURRENT_VERSION}"; \
		git push origin HEAD; \
		gh release create "v$${CURRENT_VERSION}" --generate-notes --draft | sed 's@releases/tag@releases/edit@' | xargs open; \
	fi; \

.PHONY: spell-check
spell-check: tool-spell-check
	typos

.PHONY: detect-unused-dependencies
detect-unused-dependencies: tool-detect-unused-dependencies
	cargo machete

.PHONY: update-license-file
update-license-file: tool-update-license-file
	cargo about generate about.hbs > CREDITS.html

.PHONY: check-licenses
check-licenses: tool-check-licenses
	cargo deny check licenses

DEBUG_EXECUTABLE = ./target/debug/fzf-make
TEST_DIR = ./test_data
.PHONY: run-in-test-data
run-in-test-data: build
	@TARGET_DIR=$$(find $(TEST_DIR) -type d -maxdepth 1 | fzf) && \
	if [ -n "$$TARGET_DIR" ]; then \
		mv $(DEBUG_EXECUTABLE) "$${TARGET_DIR}" && cd "$${TARGET_DIR}" && ./fzf-make; \
	else \
	    echo "No directory selected. Staying in the current directory."; \
	fi

.PHONY: build
build:
	@cargo build

.PHONY: fmt
 fmt:
	@cargo +nightly fmt

.PHONY: fmt-check
 fmt-check:
	@cargo +nightly fmt -- --check

.PHONY: check
 check:
	@cargo clippy -- -D warnings

.PHONY: build-release
build-release:
	@cargo build --verbose --release
