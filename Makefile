.PHONY: test build run check echo-test build-release echo-greeting cmd

echo-test:
	@echo good

test : # run test
	cargo nextest run

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

cmd:
	@read -p "Do something? y/n:" ans; \
	if [ "$$ans" = y ]; then  \
		echo "Doing something..."; \
	fi
