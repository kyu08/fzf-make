.PHONY: run
run:
	echo "run"

.PHONY: run-with-args
run-with-args:
	echo $(ARG1); echo $(ARG2)

.PHONY: run-with-arg
run-with-arg:
	echo $(ARG1)

include ./makefiles/test.mk

.PHONY: cmd
cmd:
	@read -p "Do something? y/n:" ans; \
	if [ "$$ans" = y ]; then  \
		echo "Doing something..."; \
	fi
