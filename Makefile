export
RUST_BACKTRACE=full

.PHONY: curl-test
curl-test:
	curl -sSL https://example.com | jq

# https://example.com
.PHONY: ci
ci: # Checks same as CI
	@make test-ci; \
	make check; \
	make fmt; \
	make spell-check

.PHONY: tools
tools: tool-test tool-bump-version tool-spell-check

.PHONY: tool-test
tool-test:
	@if ! which cargo-nextest > /dev/null; then \
		cargo install cargo-nextest; \
	fi

.PHONY: tool-bump-version
tool-bump-version:
	@if ! which cargo-set-version > /dev/null; then \
		cargo install cargo-edit; \
	fi

.PHONY: tool-spell-check
tool-spell-check:
	@if ! which typos > /dev/null; then \
		cargo install typos-cli; \
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
		git add .; \
		git commit -m "chore(release): bump to v$${CURRENT_VERSION}"; \
		git push origin HEAD; \
		gh release create "v$${CURRENT_VERSION}" --generate-notes --draft | sed 's@releases/tag@releases/edit@' | xargs open; \
	fi; \

.PHONY: spell-check
spell-check: tool-spell-check
	typos

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
	@cargo fmt -- --check

.PHONY: check
 check:
	@cargo clippy -- -D warnings

.PHONY: build-release
build-release:
	@cargo build --verbose --release
