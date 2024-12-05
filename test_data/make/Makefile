.PHONY: run
run:
	echo "run"

include ./makefiles/test.mk

.PHONY: cmd
cmd:
	@read -p "Do something? y/n:" ans; \
	if [ "$$ans" = y ]; then  \
		echo "Doing something..."; \
	fi
