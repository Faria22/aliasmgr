Output docs/assets/shell-sync.gif
Output docs/assets/shell-sync.txt
Source docs/vhs/settings.tape

Hide
Wait+Line /[$>]$/
Type 'export PATH="$PWD/target/debug:$PATH"'
Enter
Wait+Line /[$>]$/
Type 'export ALIASMGR_CATALOG_PATH=/tmp/aliasmgr-shell-sync.toml'
Enter
Wait+Line /[$>]$/
Type ': > "$ALIASMGR_CATALOG_PATH"'
Enter
Wait+Line /[$>]$/
Type 'eval "$(aliasmgr init bash)"'
Enter
Wait+Line /[$>]$/
Type 'clear'
Enter
Wait+Line /[$>]$/
Show

Sleep 750ms
Type 'aliasmgr add greet "echo Hello from the synced alias"'
Enter
Wait+Line /[$>]$/
Sleep 1s
Type 'greet'
Enter
Wait+Line /[$>]$/
Sleep 2s
Type 'aliasmgr edit greet "echo Updated at the next prompt"'
Enter
Wait+Line /[$>]$/
Sleep 1s
Type 'greet'
Enter
Wait+Line /[$>]$/
Sleep 2s

Hide
Type 'rm -f "$ALIASMGR_CATALOG_PATH"'
Enter
