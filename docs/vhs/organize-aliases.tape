Output docs/assets/organize-aliases.gif
Output docs/assets/organize-aliases.txt
Source docs/vhs/settings.tape

Hide
Wait+Line /[$>]$/
Type 'export PATH="$PWD/target/debug:$PATH"'
Enter
Wait+Line /[$>]$/
Type 'export ALIASMGR_CATALOG_PATH=/tmp/aliasmgr-organize-aliases.toml'
Enter
Wait+Line /[$>]$/
Type ': > "$ALIASMGR_CATALOG_PATH"'
Enter
Wait+Line /[$>]$/
Type 'clear'
Enter
Wait+Line /[$>]$/
Show

Sleep 750ms
Type 'aliasmgr add checks "cargo test" --tag rust --tag dev --description "Run tests" --disabled'
Enter
Wait+Line /[$>]$/
Sleep 1s
Type 'aliasmgr list --all --columns status,name,tags,description'
Enter
Wait+Line /[$>]$/
Sleep 2s
Type 'aliasmgr edit checks "cargo test --all-targets" --remove-tag dev --add-tag ci --description "Run all tests"'
Enter
Wait+Line /[$>]$/
Sleep 1s
Type 'aliasmgr enable checks'
Enter
Wait+Line /[$>]$/
Sleep 1s
Type 'aliasmgr list --all --columns status,name,tags,description'
Enter
Wait+Line /[$>]$/
Sleep 2s

Hide
Type 'rm -f "$ALIASMGR_CATALOG_PATH"'
Enter
