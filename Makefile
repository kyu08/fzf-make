.PHONY: test build run check echo build-release

echo:
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
