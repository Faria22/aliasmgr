Output docs/assets/quick-start.gif
Source docs/vhs/settings.tape

Hide
Wait+Line /❯$/
Type 'export ALIASMGR_REPO_ROOT="$PWD"'
Enter
Wait+Line /❯$/
Type 'export PATH="$ALIASMGR_REPO_ROOT/target/debug:$PATH"'
Enter
Wait+Line /❯$/
Type 'unset NO_COLOR'
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
Type 'source "$ALIASMGR_REPO_ROOT/docs/vhs/terminal.zsh"'
Enter
Wait+Line /❯$/
Type 'rm -rf -- /tmp/aliasmgr-vhs-demo'
Enter
Wait+Line /❯$/
Type 'mkdir -p /tmp/aliasmgr-vhs-demo/documents'
Enter
Wait+Line /❯$/
Type 'touch /tmp/aliasmgr-vhs-demo/notes.txt'
Enter
Wait+Line /❯$/
Type 'touch -t 202601010000 /tmp/aliasmgr-vhs-demo/notes.txt /tmp/aliasmgr-vhs-demo/documents /tmp/aliasmgr-vhs-demo'
Enter
Wait+Line /❯$/
Type 'cd /tmp/aliasmgr-vhs-demo'
Enter
Wait+Line /❯$/
Type 'clear'
Enter
Wait+Line /❯$/
Show

Type 'aliasmgr add am aliasmgr'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 4s
Type 'am add ll "ls -la"'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 4s
Type 'll'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 4s
Type 'am ls'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 4s

Hide
Type 'cd "$ALIASMGR_REPO_ROOT"'
Enter
Type 'rm -f "$ALIASMGR_CATALOG_PATH"'
Enter
Type 'rm -rf -- /tmp/aliasmgr-vhs-demo'
Enter
