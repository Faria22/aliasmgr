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
- A missing custom catalog is treated as empty by read-only commands and shell synchronization. Commands that can save ask before using a missing custom catalog; use `--yes` to accept, `--no` to decline, or `--no-input` to fail without prompting.
- Use `--no-auto-sync` to load aliases initially without installing the prompt hook. Catalog changes then require an explicit `aliasmgr sync`.

## Commands

- `aliasmgr add` — Add an alias to the catalog.
- `aliasmgr edit` — Change an existing alias and its metadata.
- `aliasmgr import` — Import aliases from Bash or Zsh files.
- `aliasmgr list` — List aliases in the catalog.
- `aliasmgr remove` — Remove aliases or tags.
- `aliasmgr rename` — Rename an alias or tag.
- `aliasmgr enable` — Enable aliases by name, tag, or filter.
- `aliasmgr disable` — Disable aliases by name, tag, or filter.
- `aliasmgr sync` — Reconcile the current shell with the catalog.
- `aliasmgr doctor` — Validate the catalog and report potential problems.

Use `-h` or `--help` with aliasmgr or any command for more information.

Notes:

- `list` shows enabled aliases by default. Use `--disabled` for disabled aliases or `--all` for both.
- Tags are case-sensitive. Repeated tag filters use AND semantics.
- Global aliases only work on Zsh and are skipped for other shells.
- Adding or editing an alias warns when its name conflicts with a shell builtin or executable on `PATH`.

## Examples

![Terminal recording of adding and using am and ll aliases](docs/assets/quick-start.gif)

## Sync Behavior

- Each initialized terminal tracks the alias names and effective catalog revision that it last applied.
- Before each prompt, aliasmgr compares that terminal's revision with the current catalog. It emits no shell changes when they match.
- When the effective catalog changes, aliases tracked by that terminal are removed with targeted, quiet `unalias` commands before all current active aliases are added back.
- This avoids `unalias -a`, so aliases maintained outside aliasmgr are not cleared.
- Disabled aliases, invalid alias names, and Zsh global aliases in non-Zsh shells are skipped when generating shell commands.
- Changes made in another terminal or by manually editing the catalog are applied when the next prompt is displayed.
- `aliasmgr sync` forces immediate reconciliation even when the stored revision matches.

## Alias Catalog File

- Default path: `~/.config/aliasmgr/aliases.toml` (XDG config home).
- Aliases and tags are saved in case-sensitive alphabetical order.
- Catalog saves use atomic replacement, preserve existing permissions, and update symlink targets without replacing the symlink.
- Concurrent modifying commands wait for one another, then load the latest catalog before applying their changes. Read-only commands remain available while a modifying command runs.
- A persistent `<catalog filename>.lock` file beside the resolved catalog target coordinates modifying commands. Leave this file in place, including when the catalog is in a dotfiles repository; add it to your ignore rules if needed. This advisory lock coordinates aliasmgr commands, so external editors must avoid writing during a command.

```toml
ll = "ls -la"
glob = { command = "*.rs", enabled = true, global = true }
test = { command = "cargo test", enabled = true, global = false, description = "Run the test suite", tags = ["dev", "rust"] }
```

## User Configuration

Presentation preferences live in `~/.config/aliasmgr/config.toml` (XDG config home). A missing configuration file uses built-in defaults. To use a custom file, pass it when initializing the shell, for example `eval "$(aliasmgr init zsh --config ~/.config/aliasmgr/work.toml)"`.

```toml
[color]
mode = "auto" # auto, always, or never

[symbols]
enabled = "✔"
disabled = "✘"
global = "⦾"

[styles]
header = { bold = true }
enabled = { foreground = "green", bold = true }
disabled = { foreground = "red", bold = true }
global = { foreground = "blue", bold = true }

[list]
columns = ["status", "name", "command", "global", "tags", "description"]
status = "auto"
```

`list.columns` is ordered. Valid names are `status`, `name`, `command`, `global`, `tags`, and `description`. With the default `status = "auto"`, the Status column is hidden when listing only enabled or disabled aliases and shown by `list --all`. Use `always` or `never` to override that behavior. An explicit `list --columns name,command,tags` is exhaustive and overrides the status policy for that command. The human-readable Global column is hidden under Bash even when configured or explicitly requested; JSON output still includes `global`. Interactive tables truncate wide cells with an ellipsis to fit the terminal; selected columns are never dropped.

Table headers are bold by default when styling is enabled. Set `styles.header.bold = false` to use plain headers.

`auto` color applies only to terminal output and respects `NO_COLOR`. The global `--color <auto|always|never>` option overrides the configured mode. Invalid known settings fail clearly; unknown settings warn and are ignored.

## Development

Requires Rust 1.99.1 or newer, Bash, and Zsh. CI tests the latest stable Rust on Linux and macOS and builds with the minimum version declared in `Cargo.toml`. On macOS, shell integration tests use `/bin/bash` (the system Bash 3.2) and `/bin/zsh`.

- Build: `cargo build --locked`
- Run tests: `cargo test --locked`
- Format: `cargo fmt`
- Lint: `cargo clippy --locked --all-targets --all-features -- -D warnings`
- Regenerate the README recording: `just render-vhs` (requires VHS v0.11.0).

The VHS recording remains manual because the previous automated workflow was unreliable. Regenerate `docs/assets/quick-start.gif` whenever commands, output, formatting, or shell behavior shown in `docs/vhs/quick-start.tape` change, and whenever the tape or its settings change. Review the GIF and commit it with the corresponding change.
