demo_repo=/tmp/aliasmgr-vhs-demo-repo
demo_remote=/tmp/aliasmgr-vhs-demo-remote.git

rm -rf -- "$demo_repo" "$demo_remote"
git init --quiet --initial-branch=main "$demo_repo"
git -C "$demo_repo" config user.name "aliasmgr demo"
git -C "$demo_repo" config user.email "demo@example.com"
print '# aliasmgr demo' > "$demo_repo/README.md"
git -C "$demo_repo" add README.md
git -C "$demo_repo" commit --quiet --message "Initial commit"
print 'Uncommitted demo notes' > "$demo_repo/notes.txt"
git init --quiet --bare "$demo_remote"
git -C "$demo_repo" remote add origin "$demo_remote"
cd "$demo_repo"
