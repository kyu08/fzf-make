## What
<!-- Please describe what you have done in this pull request. -->

## Why
<!--
    Please describe what is the motivation for this PR.
    If there is an related issue, it's fine to just write the issue number if it describes the motivation of this PR well.
-->

## Testing
<!-- Please describe what you have done to test this PR. -->

<!--

If you add some feature, make sure to follow the checklist below for preventing regression.

## Test cases
- [ ] Regression
	- [ ] Existing feature works properly.
		- [ ] History
			- [ ] Read histories
			- [ ] Write histories
		- [ ] Execution
			- [ ] Any command can be executed.
			- [ ] Narrow downed command can be executed.
		- [ ] Preview
			- [ ] Preview is shown properly.

-->

## Pre-submission Checklist
<!-- Please check the items below before submitting a PR. -->
- [ ] I have read and followed the guidelines in [`CONTRIBUTING.md`](https://github.com/kyu08/fzf-make/blob/main/CONTRIBUTING.md).
- [ ] I have double-checked my code for mistakes.
- [ ] I have added comments to help maintainers understand my code if needed.

<!--

If you want to add a new runner, please follow the checklist below.

## TODO(add_runner)
- [ ] Add a runner implementation
    - [ ] Before you implement functionality that finds or parses commands by yourself, check if the useful command which outputs you want like `task --list-all --json` exist to reduce unnecessary implementation and maintenance costs.
- [ ] Test to parse history file
- [ ] Test to write to history file
- [ ] Update README.md
- [ ] Update repository description
- [ ] Update the output of `fzf-make --help`
- [ ] Update `CREDITS` if needed
- [ ] Add a test directory to `test_data`
- [ ] Update `docs/MANUAL_TEST_CASES.md`

-->
