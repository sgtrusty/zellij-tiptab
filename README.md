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

## Background Plugin

In order to listen to `TabUpdate` events and rename all opened tabs while the plugin is unfocused, it must be loaded on startup to be typed as a background plugin. Zellij's event broadcaster ([`targeted_plugin_ids`](https://github.com/zellij-org/zellij/blob/main/zellij-server/src/screen.rs#L4334-L4351)) only includes background plugins that are explicitly subscribed to the event type. As specified by the [plugin loading wiki](https://zellij.dev/documentation/plugin-loading.html#on-startup), it can be loaded into a KDL file. See `@dev-docker.template.kdl` for an example.
