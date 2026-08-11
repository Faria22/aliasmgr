Output docs/assets/quick-start.gif
Source docs/vhs/settings.tape

Hide
Type 'export PATH="$PWD/target/debug:$PATH"'
Enter
Type 'export ALIASMGR_CATALOG_PATH=/tmp/aliasmgr-quick-start.toml'
Enter
Type ': > "$ALIASMGR_CATALOG_PATH"'
Enter
Type 'export PS1="$ "'
Enter
Type 'clear'
Enter
Show

Sleep 750ms
Type 'aliasmgr add greet "echo Hello from aliasmgr!" --description "Friendly greeting"'
Enter
Sleep 1s
Type 'aliasmgr list --columns name,command,description'
Enter
Sleep 2s
Type 'eval "$(aliasmgr init bash --no-auto-sync)"'
Enter
Sleep 750ms
Type 'greet'
Enter
Sleep 2s

Hide
Type 'rm -f "$ALIASMGR_CATALOG_PATH"'
Enter
