# aliasmgr

CLI tool to manage shell aliases from a single, versionable TOML file, written in Rust 🦀. It keeps aliases grouped, toggled, and synchronized with your shell so you can avoid hand-editing scattered `alias` definitions.

## Features
- Store aliases in `~/.config/aliasmgr/aliases.toml` (or a custom path) with optional groups.
- Add, move, list, and remove aliases; mark groups or aliases as disabled.
- Generate shell-ready alias changes on demand with `aliasmgr sync`.
- Track the last synced catalog so aliasmgr removes stale managed aliases without clearing unrelated shell aliases.
- Zsh-only global aliases (`alias -g`) support.

## Installation

### Cargo
`cargo install aliasmgr`

### Homebrew
`brew install faria22/homebrew-tap/aliasmgr`

## Shell Setup
- Initialize in your shell rc file so aliasmgr can sync on startup and know which shell you use:
  - Bash: `eval "$(aliasmgr init bash)"`
  - Zsh: `eval "$(aliasmgr init zsh)"`
- Custom catalog location: `aliasmgr init zsh --catalog ~/.aliases.toml`
  - This sets `ALIASMGR_CATALOG_PATH` so subsequent commands use that file.
- The init script also exports `ALIASMGR_SHELL` and defines a wrapper function that applies alias deltas returned on file descriptor 3.
- Advanced: set `ALIASMGR_LAST_SYNCED_CATALOG_PATH` to customize where aliasmgr stores the last synced catalog snapshot.

## Alias Catalog File
- Default path: `~/.config/aliasmgr/aliases.toml` (XDG config home).
- Last synced snapshot path: `~/.local/state/aliasmgr/last_synced_catalog.toml` (XDG state home).
- Format supports top-level aliases and grouped aliases. Disabled or global aliases use the detailed form.
- Order of groups and aliases matches the catalog file; new items are appended to the bottom.
- When aliasmgr rewrites the catalog, extra whitespace (including blank lines) is removed.

```toml
py = "python3"                                 # enabled by default
js = { command = "node", enabled = false }     # disabled
x = { command = "xargs", global = true }       # global alias (zsh only)

[git]                                          # group name
ga = "git add"
gc = { command = "git commit", enabled = true }

[misc]
enabled = false                                # disable entire group
ll = { command = "ls -la", enabled = true }
```

## Commands
- `aliasmgr add alias <name> <command> [--group <group>] [--disabled] [--global]`
- `aliasmgr add group <name> [--disabled]`
- `aliasmgr move <name> [group]`
- `aliasmgr list [<pattern>] [--group [group]] [--enabled] [--disabled] [--global]`
- `aliasmgr remove alias <name>`
- `aliasmgr remove group <name> [--reassign]`
- `aliasmgr remove all`
- `aliasmgr rename alias <old_name> <new_name>`
- `aliasmgr rename group <old_name> <new_name>`
- `aliasmgr edit <name> <new_command> [--group [group]] [--toggle_enabled] [--toggle_global]`
- `aliasmgr sync`
- `aliasmgr sort aliases [--group [group]]`
- `aliasmgr sort groups`
- `aliasmgr enable alias <name>`
- `aliasmgr enable group <name>`
- `aliasmgr disable alias <name>`
- `aliasmgr disable group <name>`

For more details, use the `-h` or `--help` flags.

Notes:
- Alias names cannot contain whitespace or `=`.
- Global aliases (`--global`) only work on zsh; they are skipped on other shells.

## Sync Behavior
- `aliasmgr sync` compares the current catalog with the last synced catalog snapshot.
- Aliases that existed in the last synced snapshot are removed with targeted `unalias '<name>'` commands before the current enabled aliases are added back.
- This avoids `unalias -a`, so aliases you maintain outside aliasmgr are not cleared by sync.
- Disabled groups, disabled aliases, invalid alias names, and zsh global aliases in non-zsh shells are skipped when generating shell commands.
- The catalog and last synced snapshot are saved after commands that change aliases or groups, so new terminals can remove aliases that were renamed, disabled, moved out of scope, or deleted in another shell.
- Run `aliasmgr sync` after manually editing the catalog file without using aliasmgr commands, and after changes made in one terminal when you have multiple shells open.

## Development
- Run tests: `cargo test`
- Format: `cargo fmt`
- Lint: `cargo clippy`
- Release: use `cargo publish` to bump the crate version and publish to crates.io.
