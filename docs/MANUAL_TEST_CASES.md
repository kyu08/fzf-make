## Test cases
- [ ] `fzf-make`
    - [ ] Execution
        - [ ] A command name executing is shown
        - [ ] Selected command is executed
    - [ ] Narrowing down
        - [ ] Placeholder is shown when no characters are typed.
        - [ ] Narrowing down works properly when some characters are typed.
        - [ ] No command is shown when no command is matched.
        - [ ] Focus is reset when backspace is pressed.
        - [ ] Focus is reset when a character is typed.
    - [ ] Preview window
        - [ ] Preview window works properly.
    - [ ] History
        - [ ] A executed command is stored in histories after execution
        - [ ] Histories is shown properly
        - [ ] Any command can be executed from history pane
    - [ ] Quitting
        - [ ] `esc` quits fzf-make
    - [ ] Additional arguments popup
        - [ ] common
            - [ ] `esc` closes the popup
        - [ ] make
            - [ ] no additional argument
                - [ ] can be executed properly
                - [ ] can be saved properly
            - [ ] one additional arguments
                - [ ] can be executed properly
                - [ ] can be saved properly
            - [ ] multiple additional arguments
                - [ ] can be executed properly
                - [ ] can be saved properly
        - [ ] just
            - [ ] no additional argument
                - [ ] can be executed properly
                - [ ] can be saved properly
            - [ ] one additional arguments
                - [ ] can be executed properly
                - [ ] can be saved properly
            - [ ] multiple additional arguments
                - [ ] can be executed properly
                - [ ] can be saved properly
        - [ ] pnpm
            - [ ] no additional argument
                - [ ] can be executed properly
                - [ ] can be saved properly
            - [ ] one additional arguments
                - [ ] can be executed properly
                - [ ] can be saved properly
            - [ ] multiple additional arguments
                - [ ] can be executed properly
                - [ ] can be saved properly
        - [ ] yarn
            - [ ] no additional argument
                - [ ] can be executed properly
                - [ ] can be saved properly
            - [ ] one additional arguments
                - [ ] can be executed properly
                - [ ] can be saved properly
            - [ ] multiple additional arguments
                - [ ] can be executed properly
                - [ ] can be saved properly
- [ ] `fzf-make --repeat` executes the command the last one.
- [ ] `fzf-make --history` launches fzf-make focusing history pane.
