Set Shell zsh
Set Theme "TokyoNightStorm"
Set Framerate 30
Set FontSize 15
Set Width 1800
Set Height 900
Output demo.gif

Type "fzf-make"
Sleep 3s

# Build and run
Hide
Enter # To delete "fzf-make"
Type "cd ../"
Enter
Type "cargo run"
Enter
Sleep 5s # Wait for the build to finish.
Show

# Main demo
Sleep 3s
Type@1s "test"
Sleep 3s
Down
Sleep 3s
Enter
Sleep 5s
