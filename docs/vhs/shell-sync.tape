Output docs/assets/shell-sync.gif
Output docs/assets/shell-sync.txt
Source docs/vhs/settings.tape

Hide
Wait+Line /❯$/
Type 'export PATH="$PWD/target/debug:$PATH"'
Enter
Wait+Line /❯$/
Type 'unset NO_COLOR'
Enter
Wait+Line /❯$/
Type 'export ALIASMGR_CATALOG_PATH=/tmp/aliasmgr-shell-sync.toml'
Enter
Wait+Line /❯$/
Type ': > "$ALIASMGR_CATALOG_PATH"'
Enter
Wait+Line /❯$/
Type 'eval "$(aliasmgr init zsh)"'
Enter
Wait+Line /❯$/
Type 'source "$PWD/docs/vhs/terminal.zsh"'
Enter
Wait+Line /❯$/
Type 'clear'
Enter
Wait+Line /❯$/
Show

Type 'aliasmgr add greet "echo Hello from the synced alias"'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 5s
Type 'greet'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 5s
Type 'aliasmgr edit greet "echo Updated at the next prompt"'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 5s
Type 'greet'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 5s

Hide
Type 'rm -f "$ALIASMGR_CATALOG_PATH"'
Enter
