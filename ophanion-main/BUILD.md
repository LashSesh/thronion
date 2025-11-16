# Build Instructions for OPHANION

## Prerequisites

- **Rust**: 1.70 or higher
- **Operating System**: Linux (Ubuntu 22.04+, Debian 11+)
- **Dependencies**: All dependencies are managed by Cargo

## Quick Build

```bash
# Clone repository
git clone https://github.com/ophanion/ophanion.git
cd ophanion

# Build release binary
cargo build --release

# Binary will be at: target/release/ophanion
```

## Development Build

```bash
# Debug build (faster compilation, includes debug symbols)
cargo build

# Run in debug mode
cargo run -- --config config.toml --verbose
```

## Testing

### Run All Tests

```bash
# Run all tests (unit + integration)
cargo test --all

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_test

# Run tests with output
cargo test -- --nocapture
```

### Test Results Summary

The test suite includes:
- **20 unit tests** covering all core modules
- **5 integration tests** for end-to-end functionality

Expected output:
```
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured
```

## Benchmarks

```bash
# Run performance benchmarks
cargo bench

# Results will be in target/criterion/
```

Benchmarks cover:
- Spectral fingerprint computation
- Signature creation
- Resonance scoring (full and k-NN)
- Learning operations
- Full decision pipeline

## Code Quality

### Check for Warnings

```bash
cargo clippy --all-targets --all-features
```

### Format Code

```bash
cargo fmt --all
```

### Check Code

```bash
cargo check --all-targets
```

## Build Profiles

### Release Profile (Production)

```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = true             # Link-time optimization
codegen-units = 1      # Single codegen unit for better optimization
strip = true           # Strip symbols for smaller binary
```

Binary size: ~2.4 MB (stripped, optimized)

### Debug Profile

Faster compilation, includes debug symbols, no optimizations.

## Installation

### Using Install Script (Recommended)

```bash
sudo chmod +x install.sh
sudo ./install.sh
```

The script will:
1. Build the release binary
2. Install to `/opt/ophanion/`
3. Create configuration directory
4. Set up systemd service
5. Configure Tor

### Manual Installation

```bash
# Build release
cargo build --release

# Create directories
sudo mkdir -p /opt/ophanion
sudo mkdir -p /etc/ophanion
sudo mkdir -p /var/log/ophanion

# Copy binary
sudo cp target/release/ophanion /opt/ophanion/
sudo chmod +x /opt/ophanion/ophanion

# Copy configuration
sudo cp config.toml /etc/ophanion/

# Install systemd service
sudo cp ophanion.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable ophanion
```

## Cross-Compilation

### For ARM64 (Raspberry Pi, etc.)

```bash
# Install cross-compilation toolchain
rustup target add aarch64-unknown-linux-gnu

# Install cross-compiler
sudo apt install gcc-aarch64-linux-gnu

# Build
cargo build --release --target aarch64-unknown-linux-gnu
```

### For Different Architectures

```bash
# List available targets
rustup target list

# Add target
rustup target add <target-triple>

# Build
cargo build --release --target <target-triple>
```

## Troubleshooting

### Build Fails with "linker not found"

```bash
sudo apt install build-essential
```

### Build Fails with Missing Dependencies

```bash
sudo apt update
sudo apt install pkg-config libssl-dev
```

### Out of Memory During Compilation

Reduce optimization or use fewer codegen units:

```toml
[profile.release]
codegen-units = 4  # Increase from 1
```

### Tests Fail

Ensure you have sufficient resources:
- At least 1 GB RAM
- Stable filesystem

## Build Artifacts

After a successful build:

```
ophanion/
├── target/
│   ├── debug/
│   │   └── ophanion          # Debug binary
│   └── release/
│       └── ophanion          # Release binary (optimized)
├── Cargo.lock                 # Dependency lock file
└── ...
```

## Verification

### Verify Binary

```bash
# Check binary
file target/release/ophanion
# Output: ELF 64-bit LSB executable, x86-64

# Check size
ls -lh target/release/ophanion
# Output: ~2.4M

# Run version check
./target/release/ophanion --help
```

### Run Quick Test

```bash
# Test with default config
./target/release/ophanion --config config.toml --verbose
```

Expected output:
```
┌────────────────────────────────────────┐
│     OPHANION v1.0                     │
│  Resonant Monolith DDoS Protection     │
└────────────────────────────────────────┘

Loading configuration from: config.toml
✓ Configuration loaded and validated
...
```

## Performance Optimization Tips

1. **Use Release Profile**: Always use `--release` for production
2. **Enable LTO**: Link-time optimization improves performance
3. **Profile-Guided Optimization** (Advanced):
   ```bash
   RUSTFLAGS="-C profile-generate=/tmp/pgo-data" cargo build --release
   # Run workload
   RUSTFLAGS="-C profile-use=/tmp/pgo-data/merged.profdata" cargo build --release
   ```

## Documentation

Generate documentation:

```bash
cargo doc --open
```

## Clean Build

Remove build artifacts:

```bash
cargo clean
```

## Continuous Integration

For CI/CD pipelines:

```bash
# Check formatting
cargo fmt --all -- --check

# Check code
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all

# Build release
cargo build --release

# Strip binary (optional)
strip target/release/ophanion
```

## Support

- **Build Issues**: https://github.com/ophanion/ophanion/issues
- **Documentation**: https://docs.ophanion.org
- **Community**: https://forum.ophanion.org

---

**Successfully built OPHANION? See DEPLOYMENT.md for deployment instructions.**
