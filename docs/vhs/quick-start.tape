Output docs/assets/quick-start.gif
Output docs/assets/quick-start.txt
Source docs/vhs/settings.tape

Hide
Wait+Line /❯$/
Type 'export PATH="$PWD/target/debug:$PATH"'
Enter
Wait+Line /❯$/
Type 'export ALIASMGR_CATALOG_PATH=/tmp/aliasmgr-quick-start.toml'
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

Sleep 750ms
Type "aliasmgr add greet 'echo Hello from aliasmgr!' --description 'Friendly greeting'"
Enter
Wait+Line /❯$/
Sleep 1s
Type 'aliasmgr list'
Enter
Wait+Line /❯$/
Sleep 2s
Type 'greet'
Enter
Wait+Line /❯$/
Sleep 2s

Hide
Type 'rm -f "$ALIASMGR_CATALOG_PATH"'
Enter
