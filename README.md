# aliasmgr

CLI tool to manage shell aliases from a single, versionable TOML file, written in Rust 🦀. It keeps aliases grouped, toggled, and synchronized with your shell so you can avoid hand-editing scattered `alias` definitions.

## Features
- Store aliases in `~/.config/aliasmgr/aliases.toml` (or a custom path) with optional groups.
- Add, move, list, and remove aliases; mark groups or aliases as disabled.
- Keep open terminals synchronized automatically before each prompt.
- Track managed aliases per terminal so stale aliases can be removed without clearing unrelated shell aliases.
- Zsh-only global aliases (`alias -g`) support.

## Installation

### Cargo
`cargo install aliasmgr`

### Homebrew
`brew install faria22/homebrew-tap/aliasmgr`

## Shell Setup
- Initialize in your shell rc file so aliasmgr can load aliases, synchronize before each prompt, and know which shell you use:
  - Bash: `eval "$(aliasmgr init bash)"`
  - Zsh: `eval "$(aliasmgr init zsh)"`
- Custom catalog location: `eval "$(aliasmgr init zsh --catalog ~/.aliases.toml)"`
  - This sets `ALIASMGR_CATALOG_PATH` so subsequent commands use that file.
- Use `--no-auto-sync` to load aliases initially without installing the prompt hook. Catalog changes then require an explicit `aliasmgr sync`.
- The init script exports `ALIASMGR_SHELL`, defines the `aliasmgr` wrapper, and keeps the applied revision and managed alias names local to each terminal.

## Alias Catalog File
- Default path: `~/.config/aliasmgr/aliases.toml` (XDG config home).
- Format supports top-level aliases and grouped aliases. Disabled or global aliases use the detailed form.
- An ungrouped alias and a group may share a name; aliasmgr preserves both when writing the catalog.
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
- `aliasmgr add <name> <command> [--group <group>] [--disabled] [--global]`
- `aliasmgr add alias <name> <command> [--group <group>] [--disabled] [--global]` (explicit form)
- `aliasmgr add group <name> [--disabled]`
- `aliasmgr move <name> [group]`
- `aliasmgr list [<pattern>] [--group [group]] [--enabled] [--disabled] [--global] [--format <human|json>]`
- `aliasmgr remove <name>` (removes the matching alias or group; prompts if both exist)
- `aliasmgr remove alias <name>` (explicit form)
- `aliasmgr remove alias [--pattern <glob>] [--group [group]]` (bulk filter; prompts once)
- `aliasmgr remove group <name> [--reassign [--enable-reassigned | --disable-reassigned]]`
- `aliasmgr remove all`
- `aliasmgr rename <old_name> <new_name>` (renames the matching alias or group; prompts if both exist)
- `aliasmgr rename alias <old_name> <new_name>` (explicit form)
- `aliasmgr rename group <old_name> <new_name>`
- `aliasmgr edit <name> <new_command> [--group [group]] [--toggle_enabled] [--toggle_global]`
- `aliasmgr sync`
- `aliasmgr sort aliases [--group [group]]`
- `aliasmgr sort groups`
- `aliasmgr enable <name>` (enables the matching alias or group; prompts if both exist)
- `aliasmgr enable alias <name>` (explicit form)
- `aliasmgr enable alias [--pattern <glob>] [--group [group]]` (bulk filter)
- `aliasmgr enable group <name>`
- `aliasmgr enable all`
- `aliasmgr disable <name>` (disables the matching alias or group; prompts if both exist)
- `aliasmgr disable alias <name>` (explicit form)
- `aliasmgr disable alias [--pattern <glob>] [--group [group]]` (bulk filter)
- `aliasmgr disable group <name>`
- `aliasmgr disable all`
- `aliasmgr doctor` (also available as `aliasmgr validate`)

For more details, use the `-h` or `--help` flags.

Notes:
- `--force` and `--no-input` are mutually exclusive global automation flags.
  `--force` accepts every prompt and selects aliases when an alias and group have
  the same name. `--no-input` exits with status 2 if a command would prompt.
  Without either flag, prompt behavior remains interactive.
- Alias names cannot be empty or contain whitespace or `=`.
- Global aliases (`--global`) only work on zsh; they are skipped on other shells.
- Adding or editing an alias warns without blocking when its name conflicts with a
  builtin in the active Bash/Zsh shell or an executable found on `PATH`.
- `aliasmgr doctor` checks the catalog without modifying it. Invalid alias names,
  missing group references, and malformed structures are errors; shell-incompatible
  global aliases and command conflicts are warnings. Errors produce a non-zero exit
  status for scripts.
- `list --format json` emits an array of aliases with `name`, `command`, `group`, `enabled`, and `global` fields. Ungrouped aliases have a `null` group.
- Alias filter operations match alias names with glob syntax. `--group <group>`
  selects aliases in that exact group, while a bare `--group` selects ungrouped
  aliases. Combining `--pattern` and `--group` selects their intersection and
  never changes the group itself.

## Sync Behavior
- Each initialized terminal tracks the alias names and effective catalog revision that it last applied.
- Before each prompt, aliasmgr compares that terminal's revision with the current catalog. It emits no shell changes when they match.
- When the effective catalog changes, aliases tracked by that terminal are removed with targeted, quiet `unalias` commands before all current active aliases are added back.
- This avoids `unalias -a`, so aliases maintained outside aliasmgr are not cleared.
- Disabled groups, disabled aliases, invalid alias names, and zsh global aliases in non-zsh shells are skipped when generating shell commands.
- Changes made in another terminal or by manually editing the catalog are applied when the next prompt is displayed.
- `aliasmgr sync` forces immediate reconciliation even when the stored revision matches, which repairs managed aliases that were manually removed or overwritten.
- Reassigning aliases from a disabled group prompts once before activating its individually enabled aliases. The prompt defaults to keeping them disabled; use `--enable-reassigned` or `--disable-reassigned` with `--reassign` for non-interactive use.
- Shell changes are generated by an internal command on standard output and evaluated by the wrapper; normal command output remains untouched.

## Development
- Run tests: `cargo test`
- Format: `cargo fmt`
- Lint: `cargo clippy`

## Releasing
1. Add a dated `## <version> - <date>` entry to `CHANGELOG.md` with non-empty
   release notes.
2. Run `cargo release <major|minor|patch> --execute`.

`cargo release` bumps the package version, verifies and publishes the crate to
crates.io, creates the release commit and `v<version>` tag, and pushes them to
GitHub. Before committing, its release hook runs the build, tests, linter,
formatter check, and the same changelog validation used by the GitHub Release
workflow. Pushing the tag creates a GitHub Release from the matching changelog
entry and opens a formula update pull request in the Homebrew tap. The formula
update merges automatically after the tap's test suite passes. If the matching
GitHub milestone has no open issues or pull requests, the tag workflow closes
it automatically.
