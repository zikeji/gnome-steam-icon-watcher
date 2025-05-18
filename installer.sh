#!/usr/bin/env bash
set -e

RESET="\033[0m"
BOLD="\033[1m"
RED="\033[31m"
GREEN="\033[32m"
YELLOW="\033[33m"
BLUE="\033[34m"
CYAN="\033[36m"

log_info() {
  echo -e "${BLUE}[INFO]${RESET} $1"
}

log_success() {
  echo -e "${GREEN}[SUCCESS]${RESET} $1"
}

log_warn() {
  echo -e "${YELLOW}[WARNING]${RESET} $1"
}

log_error() {
  echo -e "${RED}[ERROR]${RESET} $1" >&2
}

log_step() {
  echo -e "${CYAN}[STEP]${RESET} $1"
}

# Script variables
INSTALL_DIR="$HOME/.local/share/linux-steam-icon-watcher"
BINARY_PATH="$INSTALL_DIR/linux-steam-icon-watcher"
SERVICE="linux-steam-icon-watcher.service"
SERVICE_PATH="$HOME/.config/systemd/user/$SERVICE"
REPO="zikeji/linux-steam-icon-watcher"
ARCH=`uname -m`

uninstall() {
  local found_service=0
  local found_install_dir=0
  if [[ $(systemctl list-units --user --all -t service --full --no-legend "$SERVICE" | sed 's/^\s*//g' | cut -f1 -d' ') == $SERVICE ]]; then
    found_service=1
    log_step "Disabling and removing service..."
    systemctl --user disable --now $SERVICE || true
    rm -f "$SERVICE_PATH"
    log_success "Service '$SERVICE' disabled & removed."
  else
    log_warn "Service '$SERVICE' not found, skipping."
  fi
  if [[ -d "$INSTALL_DIR" ]]; then
    found_install_dir=1
    log_step "Removing installation directory..."
    rm -rf "$INSTALL_DIR"
    log_success "Installation directory '$INSTALL_DIR' removed."
  else
    log_warn "Installation directory not found, skipping."
  fi
  if [[ $found_service -eq 0 && $found_install_dir -eq 0 ]]; then
    log_warn "Nothing to uninstall."
    exit 0
  fi
  log_success "Uninstall complete."
  exit 0
}

install() {
  log_step "Creating installation directory..."
  mkdir -p "$INSTALL_DIR"

  if [[ -f "$BINARY_PATH" ]]; then
    # If the service is running, stop it before overwriting the binary
    if systemctl --user is-active --quiet $SERVICE; then
      log_info "Service is running, stopping it before updating binary..."
      systemctl --user stop $SERVICE
    fi
  fi

  log_step "Fetching latest release from GitHub..."
  LATEST_URL=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep browser_download_url | grep linux-steam-icon-watcher-$ARCH | cut -d '"' -f 4)
  if [[ -z "$LATEST_URL" ]]; then
    log_error "Failed to find latest release binary URL."
    exit 1
  fi

  log_step "Downloading binary..."
  curl -sL "$LATEST_URL" -o "$BINARY_PATH"
  chmod +x "$BINARY_PATH"

  log_step "Creating systemd user service directory..."
  mkdir -p "$HOME/.config/systemd/user"

  log_step "Writing systemd service file..."
  cat > "$SERVICE_PATH" <<EOF
[Unit]
Description=Steam Icon Watcher

[Service]
ExecStart=$BINARY_PATH
Restart=on-failure

[Install]
WantedBy=default.target
EOF

  log_step "Reloading systemd daemon..."
  systemctl --user daemon-reload

  log_step "Enabling and starting service..."
  systemctl --user enable --now $SERVICE

  log_success "Install complete. Service is running."
  exit 0
}

if [[ "$1" == "install" ]]; then
  install
fi

if [[ "$1" == "uninstall" ]]; then
  uninstall
fi

echo -e "${BOLD}Usage:${RESET} $0 [install|uninstall]"
echo -e "${BOLD}Options:${RESET}"
echo -e "  ${BOLD}install${RESET} Install the service"
echo -e "  ${BOLD}uninstall${RESET} Uninstall the service"
