# zellij-tiptab

A [Zellij](https://zellij.dev) plugin that renames tabs based on their foreground binary and working directory.

## How it works

Source `lib/tiptab-hook.sh` in your shell rc. On every prompt and command launch, the hook pipes the current tab position, working directory, and foreground binary to the plugin via `zellij action pipe --name tiptab`.

**Naming precedence per tab:**

1. **Pipe binary** — foreground binary reported by the hook (capped at 12 chars).
2. **Git repo** — `<root>/../<sub>` from the hook's working directory (each component capped at 12).
3. **Folder** — `basename(cwd)` (capped at 12, `~` for `$HOME`).
4. **Terminal command** — fallback from `PaneInfo.terminal_command` for command panes (e.g. `htop`).

Components are truncated at 12 chars; final labels at 60.
