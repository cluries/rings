#!/bin/bash

# Rings systemd service installation script

set -e

SERVICE_NAME="rings"
SERVICE_FILE="rings.service"
INSTALL_DIR="/opt/rings"
SERVICE_USER="rings"
SERVICE_GROUP="rings"

echo "Installing Rings systemd service..."

# Create service user and group
if ! id "$SERVICE_USER" &>/dev/null; then
    echo "Creating service user: $SERVICE_USER"
    sudo useradd --system --no-create-home --shell /bin/false "$SERVICE_USER"
fi

# Create installation directory
echo "Creating installation directory: $INSTALL_DIR"
sudo mkdir -p "$INSTALL_DIR"
sudo mkdir -p "$INSTALL_DIR/logs"
sudo mkdir -p "/var/log/rings"

# Copy service files
echo "Copying service files..."
sudo cp target/release/rings "$INSTALL_DIR/"
sudo cp config.sample.yml "$INSTALL_DIR/config.yml"

# Set permissions
echo "Setting permissions..."
sudo chown -R "$SERVICE_USER:$SERVICE_GROUP" "$INSTALL_DIR"
sudo chown -R "$SERVICE_USER:$SERVICE_GROUP" "/var/log/rings"
sudo chmod +x "$INSTALL_DIR/rings"

# Install systemd service
echo "Installing systemd service..."
sudo cp "$SERVICE_FILE" "/etc/systemd/system/"
sudo systemctl daemon-reload

echo "Installation complete!"
echo ""
echo "To start the service:"
echo "  sudo systemctl start $SERVICE_NAME"
echo ""
echo "To enable auto-start on boot:"
echo "  sudo systemctl enable $SERVICE_NAME"
echo ""
echo "To check service status:"
echo "  sudo systemctl status $SERVICE_NAME"
echo ""
echo "To view logs:"
echo "  sudo journalctl -u $SERVICE_NAME -f"