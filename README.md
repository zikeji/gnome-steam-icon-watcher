# gnome-steam-icon-watcher

Monitor Steam game launches on Linux and dynamically create and remove `.desktop` launcher files in `~/.local/share/applications` for each running game. This is useful for integrating Steam games with GNOME Shell and other desktop environments without adding permanent shortcuts.

## Features
- Detects Steam game launches and exits by monitoring running processes.
- Creates a `.desktop` file for each running game, using the correct Steam icon if available.
- Removes the `.desktop` file when the game exits.

## Usage

Build and run manually:
```sh
cargo run --release
```

Or install the binary and run it directly.

## Running as a systemd user service

1. Create a systemd unit file at `~/.config/systemd/user/gnome-steam-icon-watcher.service` with the following contents:

```
[Unit]
Description=GNOME Steam Icon Watcher

[Service]
ExecStart=%h/.cargo/bin/gnome-steam-icon-watcher
Restart=on-failure

[Install]
WantedBy=default.target
```

- Adjust the `ExecStart` path if your binary is elsewhere (e.g., use `%h/Projects/gnome-steam-icon-watcher/target/release/gnome-steam-icon-watcher` if running from source).

2. Reload systemd user units:
```sh
systemctl --user daemon-reload
```

3. Enable and start the service:
```sh
systemctl --user enable --now gnome-steam-icon-watcher.service
```

4. To check logs:
```sh
journalctl --user -u gnome-steam-icon-watcher.service -f
```

---

**Note:** This tool is intended for Linux desktop environments that use `.desktop` files, such as GNOME.
