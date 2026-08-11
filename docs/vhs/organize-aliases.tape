Output docs/assets/organize-aliases.gif
Source docs/vhs/settings.tape

Hide
Type 'export PATH="$PWD/target/debug:$PATH"'
Enter
Type 'export ALIASMGR_CATALOG_PATH=/tmp/aliasmgr-organize-aliases.toml'
Enter
Type ': > "$ALIASMGR_CATALOG_PATH"'
Enter
Type 'export PS1="$ "'
Enter
Type 'clear'
Enter
Show

Sleep 750ms
Type 'aliasmgr add checks "cargo test" --tag rust --tag dev --description "Run tests" --disabled'
Enter
Sleep 1s
Type 'aliasmgr list --all --columns status,name,tags,description'
Enter
Sleep 2s
Type 'aliasmgr edit checks "cargo test --all-targets" --remove-tag dev --add-tag ci --description "Run all tests"'
Enter
Sleep 1s
Type 'aliasmgr enable checks'
Enter
Sleep 1s
Type 'aliasmgr list --all --columns status,name,tags,description'
Enter
Sleep 2s

Hide
Type 'rm -f "$ALIASMGR_CATALOG_PATH"'
Enter
