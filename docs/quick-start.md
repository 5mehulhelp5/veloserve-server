# Quick Start

Get VeloServe running in under 2 minutes!

## One-Line Install

```bash
curl -sSL https://veloserve.io/install.sh | bash
```

This automatically:
- Detects your OS (Linux, macOS, Windows)
- Downloads the correct binary
- Installs to `/usr/local/bin`
- Creates default config at `/etc/veloserve/veloserve.toml`

## Start Serving

### Simple Test

```bash
# Create a test directory
mkdir -p /tmp/mysite
echo '<?php phpinfo();' > /tmp/mysite/index.php
echo '<h1>Hello VeloServe!</h1>' > /tmp/mysite/index.html

# Start server
veloserve start --root /tmp/mysite --listen 0.0.0.0:8080
```

Visit http://localhost:8080 🎉

### Using Config File

```bash
# Start with config file
veloserve --config /etc/veloserve/veloserve.toml

# Or specify custom config
veloserve --config ./mysite.toml
```

## Minimal Config Example

Create `veloserve.toml`:

```toml
[server]
listen = "0.0.0.0:8080"

[php]
enable = true

[[virtualhost]]
domain = "*"
root = "/var/www/html"
```

## What's Next?

- [Full Configuration Reference](configuration.md)
- [PHP Setup (CGI vs SAPI)](php.md)
- [WordPress Setup](wordpress.md)
- [Performance Tuning](performance.md)

