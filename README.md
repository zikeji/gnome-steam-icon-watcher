# linux-steam-icon-watcher

Monitor Steam game launches on Linux and dynamically create and remove temporary `.desktop` launcher files in `~/.local/share/applications` for them. This is useful for integrating Steam games with GNOME Shell and other desktop environments without adding permanent shortcuts.

## Features
- Detects Steam game launches and exits by monitoring running processes.
- Creates a `.desktop` file for each running game, using the correct Steam icon if available.
- Removes the `.desktop` file when the game exits.
- If the icon doesn't already exist in `~/.local/share/icons/hicolor`, sources it from the Steam installation, and, if unavailable, downloads from the Steam CDN.

## Usage

Build and run manually:
```sh
cargo run --release
```

Or install the binary and run it directly.

## Quick Install / Update / Uninstall

> ⚠️ **Warning:** Always review scripts before running code from the internet! This script will download and install a binary, create a user systemd service, and can also uninstall everything it sets up.

Install or update to the latest release:

```sh
curl -fsSL https://raw.githubusercontent.com/zikeji/linux-steam-icon-watcher/main/installer.sh | bash
```

Uninstall and clean up:

```sh
curl -fsSL https://raw.githubusercontent.com/zikeji/linux-steam-icon-watcher/main/installer.sh | bash -s -- uninstall
```

## Running as a systemd user service

1. Create a systemd unit file at `~/.config/systemd/user/linux-steam-icon-watcher.service` with the following contents:

```
[Unit]
Description=Steam Icon Watcher

[Service]
ExecStart=%h/.cargo/bin/linux-steam-icon-watcher
Restart=on-failure

[Install]
WantedBy=default.target
```

- Adjust the `ExecStart` path if your binary is elsewhere (e.g., use `%h/Projects/linux-steam-icon-watcher/target/release/linux-steam-icon-watcher` if running from source).

2. Reload systemd user units:
```sh
systemctl --user daemon-reload
```

3. Enable and start the service:
```sh
systemctl --user enable --now linux-steam-icon-watcher.service
```

4. To check logs:
```sh
journalctl --user -u linux-steam-icon-watcher.service -f
```

---

**Note:** This tool is intended for Linux desktop environments that use `.desktop` files, such as GNOME.

## Attribution

The implementation in `appinfo_vdf.rs` is adapted from the below sources. I sold my soul to Copilot to save my sanity. I'm not cut out for writing raw binary handling like this.

- [SteamDatabase/SteamAppInfo](https://github.com/SteamDatabase/SteamAppInfo)
- [ValveResourceFormat/ValveKeyValue](https://github.com/ValveResourceFormat/ValveKeyValue)
