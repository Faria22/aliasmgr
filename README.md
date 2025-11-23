# aliasmgr

CLI tool to manage shell aliases from a single, versionable TOML file. It keeps aliases grouped, toggled, and synchronized with your shell so you can avoid hand-editing scattered `alias` definitions.

## Features
- Store aliases in `~/.config/aliasmgr/aliases.toml` (or a custom path) with optional groups.
- Add, move, list, and remove aliases; mark groups or aliases as disabled.
- Generate shell-ready alias definitions on demand with `aliasmgr sync`.
- Zsh-only global aliases (`alias -g`) support.

## Installation

### Cargo
`cargo install aliasmgr`

### Homebrew
`brew install aliasmgr`

### Debian/Ubuntu
`sudo apt-get update && sudo apt-get install -y aliasmgr`

### Arch/Manjaro
`sudo pacman -S aliasmgr`

## Shell Setup
- Initialize in your shell rc file so aliasmgr can sync on startup and know which shell you use:
  - Bash: `eval "$(aliasmgr init bash)"`
  - Zsh: `eval "$(aliasmgr init zsh)"`
- Custom config location: `aliasmgr init zsh --config ~/.aliases.toml`
  - This sets `ALIASMGR_CONFIG_PATH` so subsequent commands use that file.
- The init script also exports `ALIASMGR_SHELL` and defines a wrapper function that applies alias deltas returned on file descriptor 3.

## Configuration File
- Default path: `~/.config/aliasmgr/aliases.toml` (XDG config home).
- Format supports top-level aliases and grouped aliases. Disabled or global aliases use the detailed form.
- Order of groups and aliases matches the config file; new items are appended to the bottom.
- When aliasmgr rewrites the config, extra whitespace (including blank lines) is removed.

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
- `aliasmgr move <name> [<new_group>]`
- `aliasmgr list [--group <group> | --ungrouped | --enabled | --disabled | --global]`
- `aliasmgr remove alias <name>`
- `aliasmgr remove group <name> [--reassign]`
- `aliasmgr remove all`
- `aliasmgr rename`
- `aliasmgr edit`
- `aliasmgr sync`
- `aliasmgr enable`
- `aliasmgr disable`
- `aliasmgr convert`

Notes:
- Alias names cannot contain whitespace or `=`.
- Global aliases (`--global`) only work on zsh; they are skipped on other shells.

## Sync Behavior
> [!WARNING]
> `aliasmgr sync` (and the `init` snippet that runs it) starts by executing `unalias -a`, which clears **all** aliases in the current shell.
- If you maintain aliases outside aliasmgr, define them **after** running `eval "$(aliasmgr init {shell})"` in your rc file. Re-running `sync` after sourcing your rc file will remove any aliases not managed by aliasmgr.
- Run `aliasmgr sync` again after manually editing the config file without using aliasmgr commands, and after changes made in one terminal when you have multiple shells open.
- Recommended:
  1. Move all aliases into aliasmgr, **or**
  2. Avoid `sync` and re-source your rc file whenever you update aliases.

## Development
- Run tests: `cargo test`
- Format: `cargo fmt`
- Lint: `cargo clippy`
