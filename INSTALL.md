# Thronion Installation Guide

This guide provides detailed instructions for installing and configuring Thronion for various deployment scenarios.

## Table of Contents

- [System Requirements](#system-requirements)
- [Prerequisites](#prerequisites)
- [Installation Methods](#installation-methods)
- [Configuration](#configuration)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)

---

## System Requirements

### Minimum Requirements
- **CPU**: 2 cores (x86_64)
- **RAM**: 4GB
- **Disk**: 1GB free space
- **OS**: Linux (kernel 4.4+)
- **Network**: Tor 0.4.7+ (for production)

### Recommended Requirements
- **CPU**: 4+ cores (x86_64)
- **RAM**: 8GB
- **Disk**: 5GB free space (for logs, metrics)
- **OS**: Ubuntu 22.04 LTS, Debian 11+, or RHEL 9+
- **Network**: Dedicated network interface, Tor 0.4.8+

### Tested Platforms
- ✅ Ubuntu 22.04 LTS (primary development platform)
- ✅ Debian 11 (Bullseye)
- ✅ Debian 12 (Bookworm)
- ⚠️  Other Linux distributions (should work, but not extensively tested)
- ❌ Windows (not supported)
- ❌ macOS (not supported)

---

## Prerequisites

### 1. Install Rust

Thronion requires Rust 1.91 or higher.

```bash
# Install rustup (Rust toolchain installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts to complete installation
# Typically: select option 1 (default installation)

# Configure current shell
source "$HOME/.cargo/env"

# Verify installation
rustc --version  # Should show 1.91 or higher
cargo --version
```

### 2. Install System Dependencies

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git
```

#### RHEL/CentOS/Rocky Linux
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install -y \
    pkg-config \
    openssl-devel \
    git
```

### 3. Install Tor (for production deployment)

#### Ubuntu/Debian
```bash
# Add Tor repository
sudo apt-get install -y apt-transport-https

# Add Tor GPG key and repository
# See https://support.torproject.org/apt/ for latest instructions

# Install Tor
sudo apt-get update
sudo apt-get install -y tor
```

#### Configure Tor
Edit `/etc/tor/torrc`:

```
# Enable control port
ControlPort 9051

# Enable cookie authentication
CookieAuthentication 1
CookieAuthFile /var/run/tor/control.authcookie

# Optional: Set bandwidth limits
RelayBandwidthRate 10 MB
RelayBandwidthBurst 20 MB

# Restart Tor to apply changes
sudo systemctl restart tor
sudo systemctl enable tor
```

---

## Installation Methods

### Method 1: Build from Source (Recommended)

```bash
# 1. Clone the repository
git clone https://github.com/LashSesh/thronion.git
cd thronion

# 2. Navigate to the Rust project
cd thronion

# 3. Build in release mode (optimized)
cargo build --release

# 4. Run tests to verify build
cargo test --release

# 5. Install binary to system (optional)
sudo cp target/release/thronion /usr/local/bin/
sudo chmod +x /usr/local/bin/thronion

# 6. Verify installation
thronion --version
```

**Build time**: Approximately 60 seconds on a modern 4-core system.

### Method 2: Development Build

For active development or debugging:

```bash
cd thronion/thronion

# Build with debug symbols (faster compilation, larger binary)
cargo build

# Binary location: target/debug/thronion
```

### Method 3: Using Cargo Install (Future)

Once published to crates.io:

```bash
# Not yet available - future release
cargo install thronion
```

---

## Configuration

### 1. Create Configuration Directory

```bash
# System-wide configuration
sudo mkdir -p /etc/thronion
sudo mkdir -p /var/log/thronion
sudo mkdir -p /var/lib/thronion

# Set permissions (adjust user as needed)
sudo chown -R $USER:$USER /etc/thronion
sudo chown -R $USER:$USER /var/log/thronion
sudo chown -R $USER:$USER /var/lib/thronion
```

### 2. Create Configuration File

Create `/etc/thronion/thronion.toml`:

```toml
# Thronion Configuration File
# Version: 1.0.0

# ===== Thronion Core Settings =====
[thronion]
# Maximum number of learned regions (traffic patterns)
# Higher = more memory usage, better pattern diversity
max_regions = 100

# Learning rate for adaptive updates (0.0 - 1.0)
# Higher = faster adaptation, more sensitive to recent data
learning_rate = 0.1

# Attack classification threshold (0.0 - 1.0)
# Higher = more conservative (fewer false positives)
attack_threshold = 0.5

# Resonance threshold for region matching (0.0 - 1.0)
# Higher = stricter matching, more new regions created
resonance_threshold = 0.3

# How often to run Delta Kernel optimization (number of classifications)
optimization_interval = 100

# Delta Kernel coherence threshold for stability
coherence_threshold = 0.05

# Fidelity threshold for region merging (0.0 - 1.0)
merge_threshold = 0.9

# ===== Tor Integration Settings =====
[tor]
# Tor control port (default: 9051)
control_port = 9051

# Path to Tor authentication cookie
cookie_path = "/var/run/tor/control.authcookie"

# Optional: Control port password (if not using cookie auth)
# control_password = "your_password_here"

# ===== Service Runtime Settings =====
[service]
# Bind address for internal service
bind_address = "127.0.0.1"

# Number of worker threads (0 = auto-detect CPU cores)
worker_threads = 4

# ===== Monitoring & Metrics Settings =====
[monitoring]
# Enable Prometheus metrics endpoint
enable_metrics = true

# Metrics HTTP server port
metrics_port = 9090

# Enable verbose logging (debug level)
verbose_logging = false

# Log file path (None = stdout only)
log_file = "/var/log/thronion/thronion.log"

# ===== Performance Settings =====
[performance]
# Maximum number of circuits to track simultaneously
max_tracked_circuits = 10000

# How long to retain circuit metadata after closure (seconds)
metadata_retention_secs = 3600
```

### 3. Adjust Configuration for Your Environment

**For high-traffic hidden services:**
```toml
[thronion]
max_regions = 200
learning_rate = 0.05  # More stable
attack_threshold = 0.4  # More sensitive

[performance]
max_tracked_circuits = 50000
```

**For resource-constrained systems:**
```toml
[thronion]
max_regions = 50

[service]
worker_threads = 2

[performance]
max_tracked_circuits = 5000
```

**For development/testing:**
```toml
[monitoring]
verbose_logging = true
log_file = "./thronion.log"  # Current directory
```

### 4. Set Up Log Rotation (Optional but Recommended)

Create `/etc/logrotate.d/thronion`:

```
/var/log/thronion/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0640 thronion thronion
    sharedscripts
    postrotate
        systemctl reload thronion > /dev/null 2>&1 || true
    endscript
}
```

---

## Verification

### 1. Run Tests

```bash
cd thronion/thronion

# Run all 156 tests
cargo test --release

# Expected output:
# test result: ok. 156 passed; 0 failed; 1 ignored
```

### 2. Check Binary

```bash
# Verify binary exists
ls -lh target/release/thronion

# Expected: ~2.8 MB optimized binary

# Check dependencies
ldd target/release/thronion
```

### 3. Test Configuration

```bash
# Validate configuration file syntax
cargo run --release -- --config /etc/thronion/thronion.toml --validate

# Expected: "Configuration valid"
```

### 4. Verify Tor Connection (if applicable)

```bash
# Check Tor is running
sudo systemctl status tor

# Test control port connectivity
nc -zv 127.0.0.1 9051

# Expected: "Connection to 127.0.0.1 9051 port [tcp/*] succeeded!"

# Verify cookie file exists and is readable
ls -l /var/run/tor/control.authcookie
```

### 5. Run Library Tests

```bash
# Test as library
cd thronion/thronion
cargo doc --no-deps --open

# This will build docs and open in browser
# Verify all modules are documented
```

---

## Troubleshooting

### Build Errors

**Problem**: `error: linker 'cc' not found`
```bash
# Solution: Install build tools
sudo apt-get install build-essential  # Ubuntu/Debian
sudo dnf groupinstall "Development Tools"  # RHEL/CentOS
```

**Problem**: `could not find OpenSSL`
```bash
# Solution: Install OpenSSL development headers
sudo apt-get install libssl-dev  # Ubuntu/Debian
sudo dnf install openssl-devel  # RHEL/CentOS
```

**Problem**: `error: failed to download dependencies`
```bash
# Solution: Check internet connection, try again
cargo clean
cargo build --release
```

### Runtime Errors

**Problem**: `Failed to connect to Tor control port`
```bash
# Check Tor is running
sudo systemctl status tor

# Check control port configuration
grep ControlPort /etc/tor/torrc

# Verify port is listening
sudo netstat -tlnp | grep 9051
```

**Problem**: `Permission denied: /var/run/tor/control.authcookie`
```bash
# Add user to debian-tor group (Ubuntu/Debian)
sudo usermod -a -G debian-tor $USER

# Re-login for group membership to take effect
# Or use: newgrp debian-tor
```

**Problem**: `Configuration validation failed`
```bash
# Check TOML syntax
cat /etc/thronion/thronion.toml | toml-lint  # if available

# Verify all required fields are present
cargo run --release -- --config /etc/thronion/thronion.toml --validate
```

### Performance Issues

**Problem**: High memory usage
```bash
# Reduce max_regions and max_tracked_circuits in config
[thronion]
max_regions = 50

[performance]
max_tracked_circuits = 5000
```

**Problem**: High CPU usage
```bash
# Reduce worker threads
[service]
worker_threads = 2

# Increase optimization interval (less frequent optimization)
[thronion]
optimization_interval = 500
```

### Getting Help

- **Documentation**: See [USAGE.md](USAGE.md) for usage examples
- **Issues**: https://github.com/LashSesh/thronion/issues
- **Architecture**: See THRONION_ARCHITECTURE.md for technical details
- **Logs**: Check `/var/log/thronion/thronion.log` for errors

---

## Next Steps

After successful installation:

1. Read [USAGE.md](USAGE.md) for usage examples
2. Review [THRONION_ARCHITECTURE.md](THRONION_ARCHITECTURE.md) for system design
3. Configure Prometheus scraping for metrics (if using monitoring)
4. Set up alerting for attack detection events
5. Run the system in monitoring mode before enabling blocking

---

**Installation complete!** You're ready to protect your Tor Hidden Service with Thronion.
