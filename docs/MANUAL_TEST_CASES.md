# Manual test cases

Everything listed here still needs a human. Anything not listed is covered by the scenario tests in
`src/usecase/tui/scenario_test.rs`, which drive the TUI through the same key handling the real event
loop uses and snapshot the rendered screen. Run them with `make test-ci`, and accept an intentional
UI change with `make review-snapshots`.

The scenario tests currently cover the make runner only. Extending them to the other runners is
tracked in [#111](https://github.com/kyu08/fzf-make/issues/111); until that is done, the runners
below have to be checked by hand.

## Not covered by the scenario tests

- [ ] Running a command
    - [ ] The command being executed is printed after the TUI is shut down
    - [ ] The selected command actually runs
- [ ] History
    - [ ] An executed command is stored in the history file
    - [ ] Additional arguments are stored together with the command
- [ ] Copy command to clipboard
    - [ ] main pane
        - [ ] `ctrl-y` copies the command to the clipboard
        - [ ] A notification is shown after copying
    - [ ] history pane
        - [ ] `ctrl-y` copies the command to the clipboard
        - [ ] A notification is shown after copying
- [ ] `fzf-make --repeat` executes the last command

## Runners other than make

The scenario tests only build a make runner, so these are checked by hand for now. For each of
`just`, `pnpm`, `npm`, `yarn` and `task`, in the matching directory under `test_data`:

- [ ] just
    - [ ] Commands are listed, including the ones a `mod` directive brings in
    - [ ] A command with no, one and multiple additional arguments runs and is stored
- [ ] pnpm
    - [ ] Commands are listed, including the ones of the other workspace packages
    - [ ] A command with no, one and multiple additional arguments runs and is stored
- [ ] npm
    - [ ] Commands are listed, including the ones of the other workspace packages
    - [ ] A command with no, one and multiple additional arguments runs and is stored
- [ ] yarn
    - [ ] Commands are listed, including the ones of the other workspace packages
    - [ ] A command with no, one and multiple additional arguments runs and is stored
- [ ] task
    - [ ] Commands are listed
    - [ ] A command with no, one and multiple additional arguments runs and is stored
