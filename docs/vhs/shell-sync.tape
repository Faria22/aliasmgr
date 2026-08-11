Output docs/assets/shell-sync.gif
Source docs/vhs/settings.tape

Hide
Type 'export PATH="$PWD/target/debug:$PATH"'
Enter
Type 'export ALIASMGR_CATALOG_PATH=/tmp/aliasmgr-shell-sync.toml'
Enter
Type ': > "$ALIASMGR_CATALOG_PATH"'
Enter
Type 'export PS1="$ "'
Enter
Type 'clear'
Enter
Show

Sleep 750ms
Type 'eval "$(aliasmgr init bash)"'
Enter
Sleep 1s
Type 'aliasmgr add greet "echo Hello from the synced alias"'
Enter
Sleep 1s
Type 'greet'
Enter
Sleep 2s
Type 'aliasmgr edit greet "echo Updated at the next prompt"'
Enter
Sleep 1s
Type 'greet'
Enter
Sleep 2s

Hide
Type 'rm -f "$ALIASMGR_CATALOG_PATH"'
Enter
