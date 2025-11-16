# OPHANION Deployment Guide

Complete guide for deploying OPHANION on a Tor Hidden Service.

## Prerequisites

- **Operating System**: Ubuntu 22.04 LTS or Debian 11+
- **Root Access**: Required for Tor configuration
- **Hardware**: Minimum 2 CPU cores, 4 GB RAM, 10 GB disk
- **Network**: Stable internet connection

## Step-by-Step Deployment

### Step 1: Install Tor

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Tor
sudo apt install tor -y

# Verify installation
tor --version
# Should output: Tor version 0.4.7.x or higher
```

### Step 2: Configure Tor Hidden Service

Edit Tor configuration:

```bash
sudo nano /etc/tor/torrc
```

Add the following lines:

```
# Enable control port for OPHANION
ControlPort 9051
CookieAuthentication 1

# Hidden Service configuration
HiddenServiceDir /var/lib/tor/hidden_service/
HiddenServicePort 80 127.0.0.1:8080

# Optional: Additional security
HiddenServiceMaxStreams 100
HiddenServiceMaxStreamsCloseCircuit 1
```

Save and exit (Ctrl+O, Enter, Ctrl+X).

Restart Tor:

```bash
sudo systemctl restart tor
sudo systemctl status tor
```

Get your onion address:

```bash
sudo cat /var/lib/tor/hidden_service/hostname
```

Save this address - it's your hidden service URL!

### Step 3: Install Rust

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow prompts, choose option 1 (default)

# Load Rust environment
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Step 4: Build OPHANION

```bash
# Clone repository (or extract from release)
cd /opt
sudo git clone https://github.com/ophanion/ophanion.git
cd ophanion

# Build release binary
cargo build --release

# Verify binary
ls -lh target/release/ophanion
```

### Step 5: Configure OPHANION

```bash
# Create configuration directory
sudo mkdir -p /etc/ophanion

# Copy example config
sudo cp config.toml /etc/ophanion/config.toml

# Edit configuration
sudo nano /etc/ophanion/config.toml
```

Verify these settings:

```toml
[tor]
control_port = 9051
cookie_path = "/var/run/tor/control.authcookie"

[service]
listen_port = 8080    # Matches HiddenServicePort in torrc
backend_port = 8081   # Your actual service will run here
```

### Step 6: Set Up Your Hidden Service Backend

Your actual hidden service (website, marketplace, forum) should listen on port 8081.

Example with Nginx:

```bash
sudo apt install nginx -y

# Configure Nginx to listen on 8081
sudo nano /etc/nginx/sites-available/hidden-service
```

Add:

```nginx
server {
    listen 127.0.0.1:8081;
    server_name _;
    
    root /var/www/hidden-service;
    index index.html;
    
    location / {
        try_files $uri $uri/ =404;
    }
}
```

Enable and restart:

```bash
sudo ln -s /etc/nginx/sites-available/hidden-service /etc/nginx/sites-enabled/
sudo systemctl restart nginx
```

### Step 7: Create Log Directory

```bash
sudo mkdir -p /var/log/ophanion
sudo chown $USER:$USER /var/log/ophanion
```

### Step 8: Start OPHANION

```bash
cd /opt/ophanion

# Test run (foreground)
sudo ./target/release/ophanion --config /etc/ophanion/config.toml --verbose
```

You should see:

```
┌────────────────────────────────────────┐
│     OPHANION v1.0                     │
│  Resonant Monolith DDoS Protection     │
└────────────────────────────────────────┘

Loading configuration from: /etc/ophanion/config.toml
✓ Configuration loaded and validated
Initializing OPHANION components...
✓ Spectral Engine initialized
✓ Resonance Engine initialized (64 Gabriel Cells)
✓ Adaptive Threshold initialized (θ₀ = 0.500)
✓ Delta-Kernel Optimizer initialized
✓ Decision Engine initialized

All components initialized successfully!
Target absorption rate: 95.0%
```

### Step 9: Create Systemd Service

```bash
sudo nano /etc/systemd/system/ophanion.service
```

Add:

```ini
[Unit]
Description=OPHANION - Resonant Monolith DDoS Protection
After=network.target tor.service
Requires=tor.service

[Service]
Type=simple
User=root
WorkingDirectory=/opt/ophanion
ExecStart=/opt/ophanion/target/release/ophanion --config /etc/ophanion/config.toml
Restart=always
RestartSec=10

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/ophanion

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable ophanion
sudo systemctl start ophanion
sudo systemctl status ophanion
```

### Step 10: Verify Everything Works

#### Check OPHANION Status

```bash
sudo systemctl status ophanion
sudo journalctl -u ophanion -f
```

#### Check Tor Status

```bash
sudo systemctl status tor
```

#### Test Access

From another computer with Tor Browser:

1. Open Tor Browser
2. Navigate to your `.onion` address
3. You should see your hidden service

#### Monitor Metrics

```bash
# In another terminal
curl http://localhost:9090/metrics
```

You should see metrics like:

```
ophanion_circuits_total 0
ophanion_resonance_coherence 0.0
ophanion_adaptive_threshold 0.5
```

## Monitoring & Maintenance

### View Logs

```bash
# Live logs
sudo journalctl -u ophanion -f

# Last 100 lines
sudo journalctl -u ophanion -n 100

# Errors only
sudo journalctl -u ophanion -p err
```

### Check Metrics

```bash
watch -n 5 'curl -s http://localhost:9090/metrics | grep ophanion'
```

### Restart OPHANION

```bash
sudo systemctl restart ophanion
```

### Update OPHANION

```bash
cd /opt/ophanion
git pull
cargo build --release
sudo systemctl restart ophanion
```

## Troubleshooting

### Problem: OPHANION won't start

**Solution 1**: Check Tor control port

```bash
netstat -tlnp | grep 9051
```

If not listening:

```bash
sudo nano /etc/tor/torrc
# Ensure: ControlPort 9051
sudo systemctl restart tor
```

**Solution 2**: Check cookie file permissions

```bash
ls -l /var/run/tor/control.authcookie
sudo chmod 644 /var/run/tor/control.authcookie
```

### Problem: Hidden service not accessible

**Check 1**: Tor is running

```bash
sudo systemctl status tor
```

**Check 2**: Hidden service directory exists

```bash
ls -l /var/lib/tor/hidden_service/
```

**Check 3**: Backend service is running

```bash
netstat -tlnp | grep 8081
```

### Problem: High CPU usage

**Solution**: Reduce learning rate

```bash
sudo nano /etc/ophanion/config.toml
# Change:
learning_rate_alpha = 0.005  # Lower from 0.01
sudo systemctl restart ophanion
```

### Problem: Too many false positives

**Solution**: Lower threshold

```bash
sudo nano /etc/ophanion/config.toml
# Change:
initial_threshold = 0.3  # Lower from 0.5
sudo systemctl restart ophanion
```

## Advanced Configuration

### High-Traffic Marketplace

```toml
[ophanion]
num_gabriel_cells = 128
max_tracked_circuits = 20000
worker_threads = 4

[performance]
metadata_retention = 7200  # 2 hours
```

### Maximum Security

```toml
[ophanion]
initial_threshold = 0.3
learning_rate_alpha = 0.005
target_absorption_rate = 0.90  # Lower to avoid blocking users
```

### Under Heavy Attack

```toml
[ophanion]
initial_threshold = 0.7  # Aggressive
target_absorption_rate = 0.98
```

## Integration with Monitoring

### Prometheus + Grafana

```bash
# Install Prometheus
sudo apt install prometheus -y

# Configure scrape target
sudo nano /etc/prometheus/prometheus.yml
```

Add:

```yaml
scrape_configs:
  - job_name: 'ophanion'
    static_configs:
      - targets: ['localhost:9090']
```

Restart Prometheus:

```bash
sudo systemctl restart prometheus
```

### Simple Monitoring Script

```bash
#!/bin/bash
# /usr/local/bin/ophanion-monitor.sh

while true; do
    COHERENCE=$(curl -s localhost:9090/metrics | grep coherence | awk '{print $2}')
    ABSORPTION=$(curl -s localhost:9090/metrics | grep absorption | awk '{print $2}')
    
    echo "$(date) | Coherence: $COHERENCE | Absorption: $ABSORPTION" >> /var/log/ophanion/monitor.log
    
    sleep 60
done
```

## Security Checklist

- [ ] Tor control port only accessible locally
- [ ] Cookie authentication enabled
- [ ] OPHANION runs as non-root user (if possible)
- [ ] Firewall configured (UFW recommended)
- [ ] Logs rotated regularly
- [ ] Regular updates applied
- [ ] Monitoring configured
- [ ] Backup configuration stored securely

## Next Steps

1. Monitor system for 24 hours under normal load
2. Simulate attack to verify protection
3. Tune parameters based on traffic patterns
4. Set up automated alerts
5. Document baseline metrics

## Support

- Documentation: https://docs.ophanion.org
- Issues: https://github.com/ophanion/ophanion/issues
- Community: https://forum.ophanion.org

---

**Deployment complete! Your hidden service is now protected by OPHANION.**
