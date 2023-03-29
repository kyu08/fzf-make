.PHONY: test build run check echo-test build-release echo-greeting

echo-test:
	@echo good

test :
	echo "test"

run:
		@cargo run

build:
		@cargo build

check:
		@cargo check

build-release:
		@cargo build --verbose --release

echo-greeting:
	@echo hello fzf-make!
