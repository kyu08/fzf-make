.PHONY: test run build check 

test :
	echo "test"

run:
		@cargo run

build:
		@cargo build

check:
		@cargo check

echo:
	@echo good
