# aliasmgr

CLI tool to manage shell aliases from a single, versionable TOML file, written in Rust 🦀. It keeps aliases documented, tagged, toggled, and synchronized with your shell so you can avoid hand-editing scattered `alias` definitions.

## Features

- Store aliases in `~/.config/aliasmgr/aliases.toml` (or a custom path).
- Add optional descriptions and multiple searchable tags to aliases.
- Add, edit, list, rename, enable, disable, and remove aliases or tag selections.
- Render human-readable listings as configurable tables, or emit JSON for scripts.
- Keep open terminals synchronized automatically before each prompt.
- Track managed aliases per terminal so stale aliases can be removed without clearing unrelated shell aliases.
- Support Zsh-only global aliases (`alias -g`).

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
- Enabled aliases without metadata use the simple string form.
- Disabled, global, described, or tagged aliases use the detailed inline form.
- Aliases and tags are saved in case-sensitive alphabetical order; duplicate tags are removed.
- Legacy group tables are not accepted. Their migration is handled separately.

```toml
ll = "ls -la"
glob = { command = "*.rs", enabled = true, global = true }
test = { command = "cargo test", enabled = true, global = false, description = "Run the test suite", tags = ["dev", "rust"] }
```

## User Configuration

Presentation preferences live in `~/.config/aliasmgr/config.toml` (XDG config home). A missing default file uses built-in defaults without creating a file. Set `ALIASMGR_CONFIG_PATH` to require and use an explicit file.

```toml
[color]
mode = "auto" # auto, always, or never

[symbols]
enabled = "✔"
disabled = "✘"
global = "⦾"

[styles]
enabled = { foreground = "green", bold = true }
disabled = { foreground = "red", bold = true }
global = { foreground = "blue", bold = true }

[list]
columns = ["status", "name", "command", "global", "tags", "description"]
status = "auto"
```

`list.columns` is ordered. Valid names are `status`, `name`, `command`, `global`, `tags`, and `description`. With the default `status = "auto"`, the Status column is hidden when listing only enabled or disabled aliases and shown by `list --all`. Use `always` or `never` to override that behavior. An explicit `list --columns name,command,tags` is exhaustive and overrides the status policy for that command. The human-readable Global column is hidden under Bash even when configured or explicitly requested; JSON output still includes `global`. Interactive tables truncate wide cells with an ellipsis to fit the terminal; selected columns are never dropped.

`auto` color applies only to terminal output and respects `NO_COLOR`. The global `--color <auto|always|never>` option overrides the configured mode. Invalid known settings fail clearly; unknown settings warn and are ignored.

## Commands

- `aliasmgr add <name> <command> [--description <text>] [--tag <tag>]... [--disabled] [--global]`
- `aliasmgr edit <name> [command] [--description <text> | --clear-description] [--add-tag <tag>]... [--remove-tag <tag>]... [--global | --no-global]`
- `aliasmgr list [pattern] [--tag <tag>]... [--disabled | --all] [--global] [--columns <columns>] [--format <human|json>]`
- `aliasmgr remove <name>`
- `aliasmgr remove alias <name>`
- `aliasmgr remove alias [--pattern <glob>] [--tag <tag>]...` (bulk filter; prompts once)
- `aliasmgr remove tag <tag>` (detaches the tag without removing aliases)
- `aliasmgr remove tag <tag> --aliases` (removes every tagged alias after one prompt)
- `aliasmgr remove all`
- `aliasmgr rename <old-name> <new-name>`
- `aliasmgr rename alias <old-name> <new-name>`
- `aliasmgr rename tag <old-tag> <new-tag>`
- `aliasmgr enable <name>`
- `aliasmgr enable alias <name>`
- `aliasmgr enable alias [--pattern <glob>] [--tag <tag>]...`
- `aliasmgr enable tag <tag>`
- `aliasmgr enable all`
- `aliasmgr disable <name>`
- `aliasmgr disable alias <name>`
- `aliasmgr disable alias [--pattern <glob>] [--tag <tag>]...`
- `aliasmgr disable tag <tag>`
- `aliasmgr disable all`
- `aliasmgr sync`
- `aliasmgr doctor` (also available as `aliasmgr validate`)

For more details, use `-h` or `--help`.

Notes:

- Repeat `--tag` when creating an alias or filtering by multiple tags. Filters use AND semantics: every supplied tag must be present.
- `list` shows enabled aliases by default. Use `--disabled` for disabled aliases or `--all` for both.
- Tags are case-sensitive and cannot be empty or contain whitespace.
- `--force` and `--no-input` are mutually exclusive global automation flags. `--force` accepts overwrite and removal prompts; `--no-input` exits with status 2 if input would be required.
- Alias names cannot be empty or contain whitespace or `=`.
- Global aliases only work on Zsh and are skipped for other shells.
- Adding or editing an alias warns without blocking when its name conflicts with a builtin in the active Bash/Zsh shell or an executable found on `PATH`.
- `aliasmgr doctor` checks the catalog without modifying it. Invalid alias names or tags and malformed structures are errors; shell-incompatible global aliases and command conflicts are warnings.
- `list --format json` always emits `name`, `command`, `enabled`, `global`, `tags`, and `description`. Missing descriptions are `null` and untagged aliases have an empty tag array.

## Sync Behavior

- Each initialized terminal tracks the alias names and effective catalog revision that it last applied.
- Before each prompt, aliasmgr compares that terminal's revision with the current catalog. It emits no shell changes when they match.
- When the effective catalog changes, aliases tracked by that terminal are removed with targeted, quiet `unalias` commands before all current active aliases are added back.
- This avoids `unalias -a`, so aliases maintained outside aliasmgr are not cleared.
- Disabled aliases, invalid alias names, and Zsh global aliases in non-Zsh shells are skipped when generating shell commands.
- Changes made in another terminal or by manually editing the catalog are applied when the next prompt is displayed.
- `aliasmgr sync` forces immediate reconciliation even when the stored revision matches.

## Development

- Run tests: `cargo test`
- Format: `cargo fmt`
- Lint: `cargo clippy`

## Releasing

1. Add a dated `## <version> - <date>` entry to `CHANGELOG.md` with non-empty release notes.
2. Run `cargo release <major|minor|patch> --execute`.

`cargo release` bumps the package version, verifies and publishes the crate to crates.io, creates the release commit and `v<version>` tag, and pushes them to GitHub. Before committing, its release hook runs the build, tests, linter, formatter check, and changelog validation. Pushing the tag creates a GitHub Release and opens a formula update pull request in the Homebrew tap.
