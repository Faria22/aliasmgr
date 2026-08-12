Output docs/assets/quick-start.gif
Output docs/assets/quick-start.txt
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
Type 'source "$ALIASMGR_REPO_ROOT/docs/vhs/git-demo.zsh"'
Enter
Wait+Line /❯$/
Type 'clear'
Enter
Wait+Line /❯$/
Show

Type 'aliasmgr add g git --description "Git shorthand"'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 5s
Type 'aliasmgr add gp "git push" --description "Push the current branch"'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 5s
Type 'aliasmgr list'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 5s
Type 'g status --short'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 5s
Type 'gp --set-upstream origin main'
Sleep 500ms
Enter
Wait+Line /❯$/
Sleep 5s

Hide
Type 'cd "$ALIASMGR_REPO_ROOT"'
Enter
Type 'rm -f "$ALIASMGR_CATALOG_PATH"'
Enter
Type 'rm -rf -- /tmp/aliasmgr-vhs-demo-repo /tmp/aliasmgr-vhs-demo-remote.git'
Enter
