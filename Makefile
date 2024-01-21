export
RUST_BACKTRACE=full

.PHONY: ci
ci: # Checks same as CI
	@make test-ci; \
	make check; \
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

.PHONY: test
test: tool-test
	RUST_BACKTRACE=full FZF_MAKE_IS_TESTING=true cargo nextest run

.PHONY: bump-fzf-make-version
bump-fzf-make-version: tool-bump-version
	@git checkout main; \
	git pull; \
	cargo set-version --bump minor; \
	export CURRENT_VERSION=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'); \
	git add .; \
	git commit -m "Bump fzf-make's version to v$${CURRENT_VERSION}"; \
	git push origin HEAD; \
	gh release create "v$${CURRENT_VERSION}" --generate-notes --draft | sed 's@releases/tag@releases/edit@' | xargs open

.PHONY: spell-check
spell-check: tool-spell-check
	typos

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

# Targets for test
include ./makefiles/test.mk

.PHONY: toooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo-long-target2
toooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo-long-target2:
	@echo "this is toooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo-long-target."


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
