Output docs/assets/global-alias.gif
Output docs/assets/global-alias.txt
Source docs/vhs/settings.tape

Hide
Wait+Line /❯$/
Type 'export ALIASMGR_REPO_ROOT="$PWD"'
Enter
Wait+Line /❯$/
Type 'export PATH="$ALIASMGR_REPO_ROOT/target/debug:$PATH"'
Enter
Wait+Line /❯$/
Type 'export ALIASMGR_CATALOG_PATH=/tmp/aliasmgr-global-alias.toml'
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
Type 'source "$ALIASMGR_REPO_ROOT/docs/vhs/git-demo.zsh"'
Enter
Wait+Line /❯$/
Type 'clear'
Enter
Wait+Line /❯$/
Show

Sleep 3s
Type 'aliasmgr add g git --description "Git shorthand"'
Enter
Wait+Line /❯$/
Sleep 3s
Type 'aliasmgr add G "| grep" --global --description "Filter command output"'
Enter
Wait+Line /❯$/
Sleep 3s
Type 'g branch G main'
Enter
Wait+Line /❯$/
Sleep 3s

Hide
Type 'cd "$ALIASMGR_REPO_ROOT"'
Enter
Type 'rm -f "$ALIASMGR_CATALOG_PATH"'
Enter
Type 'rm -rf -- /tmp/aliasmgr-vhs-demo-repo /tmp/aliasmgr-vhs-demo-remote.git'
Enter
