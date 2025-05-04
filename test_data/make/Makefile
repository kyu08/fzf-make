.PHONY: run
run:
	echo "run"

# just echo args
.PHONY: run-with-args
run-with-args:
	echo $(ARG1); echo $(ARG2)

include ./makefiles/test.mk

.PHONY: cmd
cmd:
	@read -p "Do something? y/n:" ans; \
	if [ "$$ans" = y ]; then  \
		echo "Doing something..."; \
	fi
