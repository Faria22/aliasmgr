# Changelog

## 1.3.0 - 2026-08-09
### Added
- Add `list --format json` for machine-readable alias output using the same pattern, group, status, global, and shell-compatibility filters as the human-readable view.
- Add `enable all` and `disable all` to update every alias and group in one operation.
- Add `doctor`, with `validate` as an alias, for non-mutating catalog and shell-compatibility diagnostics.
- Warn when alias names conflict with Bash or Zsh builtins or executables on `PATH`; warnings remain non-blocking and can be suppressed with `--quiet`.
- Add global `--force` and `--no-input` controls for scripted commands that would otherwise prompt. `--force` accepts prompts, while `--no-input` exits with status 2 without changing the catalog when input is required.
- Add `--pattern <glob>` and `--group [group]` selectors to explicit alias enable, disable, and remove commands, including combined filters and single-confirmation bulk removal.

### Changed
- Omit group headers from `list` output when no aliases remain after filtering.

## 1.2.0 - 2026-08-08
### Added
- Allow aliases to be managed without spelling out the `alias` resource for `add`, `remove`, `rename`, `enable`, and `disable`, while retaining explicit alias and group forms.
- Synchronize aliases automatically before each Bash or Zsh prompt when the effective catalog changes.
- Track managed alias names and the applied catalog revision independently in each terminal.
- Add `aliasmgr init --no-auto-sync` for manual synchronization workflows.
- Prompt before individually enabled aliases are activated when removing a disabled group with `--reassign`, with flags for non-interactive control.

### Changed
- Allow an ungrouped alias and a group to share a name, prompting to disambiguate shorthand commands when necessary.
- Generate reconciliation code through a dedicated stdout command instead of sending alias deltas over file descriptor 3.
- Make `aliasmgr sync` force a complete reconciliation of the current terminal.
- Reject adding, moving, or editing aliases into groups that do not exist.

### Fixed
- Avoid prompting about reassigned aliases when removing an empty group.

### Removed
- Remove the shared last-synced catalog snapshot, `ALIASMGR_LAST_SYNCED_CATALOG_PATH`, and hidden startup sync mode.

## 1.1.1 - 2026-05-05
### Fixed
- Use `sync --startup` from generated shell init scripts so a new shell only adds active aliases and does not remove aliases from the last synced catalog snapshot before they are active.
- Avoid emitting extra blank lines when generated sync output has no alias commands.

### Added
- Add tests covering startup sync behavior and active-alias command generation.

## 1.1.0 - 2026-04-26
### Added
- Track a last synced catalog snapshot so `sync` can remove stale managed aliases without clearing unrelated shell aliases.
- Add `ALIASMGR_LAST_SYNCED_CATALOG_PATH` support for customizing the last synced catalog snapshot location.
- Add test coverage for missing last synced catalog snapshots.

### Changed
- `sync` now emits targeted `unalias '<name>'` commands from the last synced catalog instead of using `unalias -a`.

## 1.0.0 - 2025-11-29
### Added
- Added `enable` command
- Added `disable` command

### Changed
- Using `Option<&T>` instead of `&Option<T>`

## 0.6.0 - 2025-11-26
### Added
- `sort` command for sorting aliases and groups.

## 0.5.0 - 2025-11-26
### Added
- `edit` command now has flags to change the alias group, toggle enable/disable status, and global status.

### Removed
- `Failure::UnexpectedBehaviour` in favor of `unreachable`.

## 0.4.0 - 2025-11-26
### Added
- `edit` command to edit aliases.

## 0.3.1 - 2025-11-25
### Added
- Added `UnexpectedBehaviour` variant to the `Failure` enum to better handle errors in the future.

### Fixed
- `rename alias` command.

## 0.3.0 - 2025-11-25
### Added
- Added `rename` command functionality.
- Stopped using `GroupId` in favor of `Option<String>`.

## 0.2.1 - 2025-11-25
### Fixed
- Updated bash init script to use `type -P` instead of `command -v` to bet the binary path.

## 0.2.0 - 2025-11-24
### Added
- Improve `list` options with pattern matching support.

### Fixed
- Prevent the `list` command from showing global aliases when running under Bash.
- Guard reassigning aliases when removing a group to avoid touching ungrouped aliases unnecessarily.

## 0.1.1 - 2025-11-24
### Fixed
- Fixed the Bash init command and aligned tests with the new behavior.
- Removed incorrect package manager installation instructions.

## 0.1.0 - 2025-11-23
Initial release.
