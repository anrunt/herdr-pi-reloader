# Herdr Pi Reloader

A small [Herdr](https://herdr.dev/) plugin for safely reloading or restarting Pi agent sessions from an overlay TUI.

The plugin only operates on Pi agents that Herdr reports as `idle` or `done`. Busy agents are skipped.

## Features

- **Reload all Pi** sends `/reload` to every eligible Pi pane.
- **Reset all Pi** exits each eligible Pi process and resumes its recorded session with `pi --session`.
- Provides an overlay TUI with a summary of completed, skipped, and failed operations.

## Requirements

- Herdr 0.7.0 or newer
- The [Pi coding agent](https://github.com/badlogic/pi-mono) available on `PATH`
- The Herdr Pi integration
- Rust 1.85 or newer with Cargo (required to build the plugin during installation)
- Linux or macOS

## Installation

Install the Herdr integration for Pi first:

```sh
herdr integration install pi
```

Then install the plugin from GitHub:

```sh
herdr plugin install anrunt/herdr-pi-reloader
```

Herdr shows the repository and build commands for review before installation. The plugin is compiled locally with Cargo and stored in Herdr's managed plugin directory.

Confirm that it is installed and enabled:

```sh
herdr plugin list
herdr plugin action list --plugin herdr-pi-reloader.pi-reloader
```

## Usage

Open the overlay from the command line:

```sh
herdr plugin action invoke herdr-pi-reloader.pi-reloader.open
```

Inside the overlay:

- `j` / `Down` — move down
- `k` / `Up` — move up
- `Enter` — run the selected operation
- `q`, `Esc`, or `Ctrl+C` — close (disabled while a reset is running)

### Optional keybinding

Add a plugin action binding to `~/.config/herdr/config.toml`:

```toml
[[keys.command]]
key = "prefix+alt+r"
type = "plugin_action"
command = "herdr-pi-reloader.pi-reloader.open"
description = "open Pi Reloader"
```

Apply the configuration without restarting panes:

```sh
herdr server reload-config
```

No additional plugin-specific configuration is required.

## Updating

Herdr does not currently provide a separate plugin update command. Reinstall the plugin to replace the managed checkout with the latest version:

```sh
herdr plugin install anrunt/herdr-pi-reloader
```

To install a reproducible release instead, pin a Git tag:

```sh
herdr plugin install anrunt/herdr-pi-reloader --ref v0.1.0
```

## Uninstalling

```sh
herdr plugin uninstall anrunt/herdr-pi-reloader
```

## Local development

Herdr does not run manifest build commands for locally linked plugins, so build the binary first:

```sh
git clone https://github.com/anrunt/herdr-pi-reloader.git
cd herdr-pi-reloader
cargo build --release --locked
herdr plugin link .
```

Open the linked plugin:

```sh
herdr plugin action invoke herdr-pi-reloader.pi-reloader.open
```

Inspect plugin logs or remove the local link with:

```sh
herdr plugin log list --plugin herdr-pi-reloader.pi-reloader
herdr plugin unlink herdr-pi-reloader.pi-reloader
```

## Trust and security

Herdr plugins run as the current user and are not sandboxed. Review [`herdr-plugin.toml`](./herdr-plugin.toml) and the source before installing.
