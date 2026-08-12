Output docs/assets/organize-aliases.gif
Output docs/assets/organize-aliases.txt
Source docs/vhs/settings.tape

Hide
Wait+Line /❯$/
Type 'export PATH="$PWD/target/debug:$PATH"'
Enter
Wait+Line /❯$/
Type 'unset NO_COLOR'
Enter
Wait+Line /❯$/
Type 'export ALIASMGR_CATALOG_PATH=/tmp/aliasmgr-organize-aliases.toml'
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

Type 'aliasmgr add checks "cargo test" --tag rust --tag dev --description "Run tests" --disabled'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 5s
Type 'aliasmgr list --all'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 5s
Type 'aliasmgr edit checks "cargo test --all-targets" --remove-tag dev --add-tag ci --description "Run all tests"'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 5s
Type 'aliasmgr enable checks'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 5s
Type 'aliasmgr list --all'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 5s

Hide
Type 'rm -f "$ALIASMGR_CATALOG_PATH"'
Enter
