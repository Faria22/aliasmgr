# Changelog

## 0.3.0 - 2025-11-2025
- Added `rename` command functionality.
- Stopped using `GroupId` in favor of `Option<String>`.

## 0.2.1 - 2025-11-25
- Updated bash init script to use `type -P` instead of `command -v` to bet the binary path.

## 0.2.0 - 2025-11-24
- Prevent the `list` command from showing global aliases when running under Bash.
- Improve `list` options with pattern matching support.
- Guard reassigning aliases when removing a group to avoid touching ungrouped aliases unnecessarily.

## 0.1.1 - 2025-11-24
- Fixed the Bash init command and aligned tests with the new behavior.
- Removed incorrect package manager installation instructions.

## 0.1.0 - 2025-11-23
- Initial release.
