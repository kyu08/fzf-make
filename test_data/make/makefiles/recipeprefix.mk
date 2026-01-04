.RECIPEPREFIX = >

.PHONY: test build clean

test:
>@echo "Running tests with > prefix"
>@echo "This should work now!"

build:
>@echo "Building with custom recipe prefix"
>cargo build

clean:
>@echo "Cleaning up"
>rm -rf target
