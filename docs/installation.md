# Installation

## Quick Install (Recommended)

```bash
curl -sSL https://veloserve.io/install.sh | bash
```

Or with wget:
```bash
wget -qO- https://veloserve.io/install.sh | bash
```

## Manual Download

Download pre-built binaries from [GitHub Releases](https://github.com/veloserve/veloserve/releases):

| Platform | Download |
|----------|----------|
| Linux x86_64 | `veloserve-vX.X.X-linux-x86_64.tar.gz` |
| Linux ARM64 | `veloserve-vX.X.X-linux-aarch64.tar.gz` |
| macOS x86_64 | `veloserve-vX.X.X-darwin-x86_64.tar.gz` |
| macOS ARM64 (Apple Silicon) | `veloserve-vX.X.X-darwin-aarch64.tar.gz` |
| Windows x86_64 | `veloserve-vX.X.X-windows-x86_64.zip` |

### Linux/macOS

```bash
# Download
curl -LO https://github.com/veloserve/veloserve/releases/latest/download/veloserve-linux-x86_64.tar.gz

# Extract
tar -xzf veloserve-linux-x86_64.tar.gz

# Install
sudo mv veloserve /usr/local/bin/
sudo chmod +x /usr/local/bin/veloserve

# Verify
veloserve --version
```

### Windows

1. Download the `.zip` file
2. Extract to `C:\Program Files\VeloServe\`
3. Add to PATH or run directly

```powershell
.\veloserve.exe --version
```

## Build from Source

### Requirements

- Rust 1.70+ ([install](https://rustup.rs))
- PHP 8.x (for PHP support)

### CGI Mode (Default)

```bash
git clone https://github.com/veloserve/veloserve.git
cd veloserve
cargo build --release
sudo cp target/release/veloserve /usr/local/bin/
```

### SAPI Mode (Embedded PHP)

```bash
# Install PHP embed library
# Ubuntu/Debian
sudo apt install php-dev libphp-embed libxml2-dev libsodium-dev libargon2-dev

# Fedora/RHEL
sudo dnf install php-devel php-embedded

# Build with embedded PHP
cargo build --release --features php-embed
sudo cp target/release/veloserve /usr/local/bin/
```

## Docker

```dockerfile
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y php-cgi && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/veloserve /usr/local/bin/
EXPOSE 8080
CMD ["veloserve", "--config", "/etc/veloserve/veloserve.toml"]
```

## Verify Installation

```bash
# Check version
veloserve --version

# Check help
veloserve --help

# Test configuration
veloserve config test
```

## Uninstall

```bash
# Remove binary
sudo rm /usr/local/bin/veloserve

# Remove config (optional)
sudo rm -rf /etc/veloserve
```

