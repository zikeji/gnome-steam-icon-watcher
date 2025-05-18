#!/usr/bin/env bash
set -e

INSTALL_DIR="$HOME/.local/share/linux-steam-icon-watcher"
BINARY_PATH="$INSTALL_DIR/linux-steam-icon-watcher"
SERVICE_PATH="$HOME/.config/systemd/user/linux-steam-icon-watcher.service"
REPO="zikeji/linux-steam-icon-watcher"

usage() {
  echo "Usage: $0 [uninstall]"
  exit 1
}

if [[ "$1" == "uninstall" ]]; then
  read -p "Disable and remove systemd service? [y/N] " confirm
  if [[ "$confirm" =~ ^[Yy]$ ]]; then
    systemctl --user disable --now linux-steam-icon-watcher.service || true
    rm -f "$SERVICE_PATH"
    echo "Service removed."
  else
    echo "Skipped service removal."
  fi
  read -p "Remove binary and install directory ($INSTALL_DIR)? [y/N] " confirm2
  if [[ "$confirm2" =~ ^[Yy]$ ]]; then
    rm -rf "$INSTALL_DIR"
    echo "Install directory removed."
  else
    echo "Skipped binary removal."
  fi
  echo "Uninstall complete."
  exit 0
fi

mkdir -p "$INSTALL_DIR"

if [[ -f "$BINARY_PATH" ]]; then
  # If the service is running, stop it before overwriting the binary
  if systemctl --user is-active --quiet linux-steam-icon-watcher.service; then
    echo "Service is running, stopping it before updating binary..."
    systemctl --user stop linux-steam-icon-watcher.service
  fi
fi

LATEST_URL=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep browser_download_url | grep linux-steam-icon-watcher | cut -d '"' -f 4)
if [[ -z "$LATEST_URL" ]]; then
  echo "Failed to find latest release binary URL." >&2
  exit 1
fi

curl -L "$LATEST_URL" -o "$BINARY_PATH"
chmod +x "$BINARY_PATH"

mkdir -p "$HOME/.config/systemd/user"

cat > "$SERVICE_PATH" <<EOF
[Unit]
Description=GNOME Steam Icon Watcher

[Service]
ExecStart=$BINARY_PATH
Restart=on-failure

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload
systemctl --user enable --now linux-steam-icon-watcher.service

echo "Install complete. Service is running."
