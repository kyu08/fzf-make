include ./makefiles/test.mk

.PHONY: echo-test
echo-test:
	@echo good

.PHONY: test
test : # run test
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

.PHONY: echo-greeting
echo-greeting:
	@echo hello fzf-make!

.PHONY: cmd
cmd:
	@read -p "Do something? y/n:" ans; \
	if [ "$$ans" = y ]; then  \
		echo "Doing something..."; \
	fi
