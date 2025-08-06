# Systemd Configuration for Rings

This directory contains systemd service configuration files for the Rings application.

## Files

- `rings.service` - Main systemd service unit file
- `install.sh` - Installation script for setting up the service
- `README.md` - This documentation file

## Installation

1. Build the release binary:
   ```bash
   cargo build --release
   ```

2. Run the installation script:
   ```bash
   cd systemd
   chmod +x install.sh
   sudo ./install.sh
   ```

## Service Management

### Start the service
```bash
sudo systemctl start rings
```

### Stop the service
```bash
sudo systemctl stop rings
```

### Enable auto-start on boot
```bash
sudo systemctl enable rings
```

### Check service status
```bash
sudo systemctl status rings
```

### View logs
```bash
# Follow logs in real-time
sudo journalctl -u rings -f

# View recent logs
sudo journalctl -u rings -n 100
```

## Configuration

The service expects:
- Binary location: `/opt/rings/rings`
- Config file: `/opt/rings/config.yml`
- Log directory: `/var/log/rings`
- Service user: `rings`

## Security Features

The service includes several security hardening features:
- Runs as dedicated `rings` user
- No new privileges
- Private temporary directory
- Protected system directories
- Limited file system access
- Memory limits

## Dependencies

The service is configured to start after:
- Network is available
- PostgreSQL service (if running locally)
- Redis service (if running locally)

## Customization

Edit `rings.service` to customize:
- Environment variables
- Resource limits
- Security settings
- Dependencies