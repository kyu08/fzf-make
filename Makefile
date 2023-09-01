.PHONY: test
test : # Run unit tests
	RUST_BACKTRACE=1 cargo nextest run

.PHONY: run
run:
	@cargo run

.PHONY: build
build:
	@cargo build

.PHONY: check
check:
	@cargo check

.PHONY: build-release
build-release:
	@cargo build --verbose --release

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
