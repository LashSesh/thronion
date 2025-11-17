#!/bin/bash
# OPHANION Installation Script

set -e

echo "┌────────────────────────────────────────┐"
echo "│   OPHANION Installation Script        │"
echo "│   Resonant Monolith DDoS Protection    │"
echo "└────────────────────────────────────────┘"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "⚠ This script must be run as root (use sudo)"
    exit 1
fi

# Check prerequisites
echo "[1/8] Checking prerequisites..."

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "✗ Rust is not installed. Please install Rust first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi
echo "✓ Rust found: $(rustc --version)"

# Check for Tor
if ! command -v tor &> /dev/null; then
    echo "✗ Tor is not installed. Installing Tor..."
    apt update
    apt install -y tor
fi
echo "✓ Tor found: $(tor --version | head -1)"

# Build OPHANION
echo ""
echo "[2/8] Building OPHANION..."
cargo build --release
echo "✓ Build complete"

# Create directories
echo ""
echo "[3/8] Creating directories..."
mkdir -p /opt/ophanion
mkdir -p /etc/ophanion
mkdir -p /var/log/ophanion
chown -R $SUDO_USER:$SUDO_USER /var/log/ophanion
echo "✓ Directories created"

# Copy binary
echo ""
echo "[4/8] Installing binary..."
cp target/release/ophanion /opt/ophanion/ophanion
chmod +x /opt/ophanion/ophanion
echo "✓ Binary installed to /opt/ophanion/ophanion"

# Copy configuration
echo ""
echo "[5/8] Installing configuration..."
if [ ! -f /etc/ophanion/config.toml ]; then
    cp config.toml /etc/ophanion/config.toml
    echo "✓ Configuration installed to /etc/ophanion/config.toml"
else
    echo "⚠ Configuration already exists at /etc/ophanion/config.toml (skipping)"
fi

# Configure Tor
echo ""
echo "[6/8] Configuring Tor..."

TORRC="/etc/tor/torrc"
if ! grep -q "ControlPort 9051" "$TORRC"; then
    echo "" >> "$TORRC"
    echo "# OPHANION Configuration" >> "$TORRC"
    echo "ControlPort 9051" >> "$TORRC"
    echo "CookieAuthentication 1" >> "$TORRC"
    systemctl restart tor
    echo "✓ Tor configured and restarted"
else
    echo "⚠ Tor already configured (skipping)"
fi

# Install systemd service
echo ""
echo "[7/8] Installing systemd service..."
cp ophanion.service /etc/systemd/system/ophanion.service
systemctl daemon-reload
systemctl enable ophanion
echo "✓ Systemd service installed and enabled"

# Display next steps
echo ""
echo "[8/8] Installation complete!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  OPHANION has been installed!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Next steps:"
echo ""
echo "1. Edit configuration:"
echo "   sudo nano /etc/ophanion/config.toml"
echo ""
echo "2. Start OPHANION:"
echo "   sudo systemctl start ophanion"
echo ""
echo "3. Check status:"
echo "   sudo systemctl status ophanion"
echo ""
echo "4. View logs:"
echo "   sudo journalctl -u ophanion -f"
echo ""
echo "5. Check metrics:"
echo "   curl http://localhost:9090/metrics"
echo ""
echo "For more information, see:"
echo "  - README.md"
echo "  - DEPLOYMENT.md"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
