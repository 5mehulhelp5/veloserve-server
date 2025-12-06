#!/bin/bash
# WordPress Setup Script for VeloServe Demo on Ona.com
# This script downloads and configures WordPress to run with VeloServe

set -e

# Get the repo root directory FIRST (before any cd commands)
# When called from Makefile in repo root, Cargo.toml exists in PWD
# When called directly, find it from script location
if [ -f "Cargo.toml" ]; then
    REPO_DIR="$(pwd)"
elif [ -f "../Cargo.toml" ]; then
    REPO_DIR="$(cd .. && pwd)"
else
    # Fallback: assume script is in .devcontainer/
    SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
    REPO_DIR="$(dirname "$SCRIPT_DIR")"
fi

WORDPRESS_DIR="/var/www/wordpress"
WORDPRESS_VERSION="latest"

echo "🚀 Setting up WordPress for VeloServe..."
echo "📍 Repo directory: $REPO_DIR"

# Create WordPress directory
sudo mkdir -p "$WORDPRESS_DIR"
sudo chown -R $(whoami):$(whoami) "$WORDPRESS_DIR"

# Download WordPress
echo "📦 Downloading WordPress..."
cd /tmp
curl -sO https://wordpress.org/latest.tar.gz
tar -xzf latest.tar.gz
cp -r wordpress/* "$WORDPRESS_DIR/"
rm -rf wordpress latest.tar.gz

# Create wp-config.php for SQLite (no MySQL needed for demo!)
echo "⚙️ Configuring WordPress..."

# Download SQLite integration plugin (allows WP to run without MySQL)
echo "📦 Installing SQLite database driver..."
cd "$WORDPRESS_DIR/wp-content"
mkdir -p mu-plugins database
curl -sL https://github.com/aaemnnosttv/wp-sqlite-db/archive/refs/heads/master.zip -o sqlite.zip
unzip -q sqlite.zip
# db.php must be in wp-content/ directly for WordPress to find it
cp wp-sqlite-db-master/src/db.php db.php
rm -rf sqlite.zip wp-sqlite-db-master

# Create wp-config.php
cat > "$WORDPRESS_DIR/wp-config.php" << 'WPCONFIG'
<?php
/**
 * WordPress Configuration for VeloServe Demo
 * Using SQLite - no MySQL required!
 */

// SQLite database file location
define('DB_DIR', __DIR__ . '/wp-content/database/');
define('DB_FILE', 'wordpress.db');

// Fake MySQL settings (required by WP but not used with SQLite)
define('DB_NAME', 'wordpress');
define('DB_USER', 'root');
define('DB_PASSWORD', '');
define('DB_HOST', 'localhost');
define('DB_CHARSET', 'utf8');
define('DB_COLLATE', '');

// Authentication keys - generate unique ones for production!
define('AUTH_KEY',         'veloserve-demo-key-1');
define('SECURE_AUTH_KEY',  'veloserve-demo-key-2');
define('LOGGED_IN_KEY',    'veloserve-demo-key-3');
define('NONCE_KEY',        'veloserve-demo-key-4');
define('AUTH_SALT',        'veloserve-demo-salt-1');
define('SECURE_AUTH_SALT', 'veloserve-demo-salt-2');
define('LOGGED_IN_SALT',   'veloserve-demo-salt-3');
define('NONCE_SALT',       'veloserve-demo-salt-4');

// Table prefix
$table_prefix = 'wp_';

// Debug mode - enable for development
define('WP_DEBUG', true);
define('WP_DEBUG_LOG', true);
define('WP_DEBUG_DISPLAY', false);

// Auto-detect site URL (works with Ona.com port forwarding)
if (isset($_SERVER['HTTP_X_FORWARDED_PROTO'])) {
    $_SERVER['HTTPS'] = $_SERVER['HTTP_X_FORWARDED_PROTO'] === 'https' ? 'on' : 'off';
}
if (isset($_SERVER['HTTP_X_FORWARDED_HOST'])) {
    $_SERVER['HTTP_HOST'] = $_SERVER['HTTP_X_FORWARDED_HOST'];
}

$protocol = (!empty($_SERVER['HTTPS']) && $_SERVER['HTTPS'] !== 'off') ? 'https://' : 'http://';
$host = $_SERVER['HTTP_HOST'] ?? 'localhost:8080';
define('WP_HOME', $protocol . $host);
define('WP_SITEURL', $protocol . $host);

// Disable automatic updates for demo
define('AUTOMATIC_UPDATER_DISABLED', true);
define('WP_AUTO_UPDATE_CORE', false);

// File permissions
define('FS_METHOD', 'direct');

// Load WordPress
if (!defined('ABSPATH')) {
    define('ABSPATH', __DIR__ . '/');
}
require_once ABSPATH . 'wp-settings.php';
WPCONFIG

# Set permissions
chmod -R 755 "$WORDPRESS_DIR"
chmod -R 777 "$WORDPRESS_DIR/wp-content"

# Create VeloServe config for WordPress
cat > "$REPO_DIR/wordpress.toml" << 'VELOCONFIG'
# VeloServe Configuration for WordPress Demo
# Start with: cargo run -- --config wordpress.toml

[server]
listen = "0.0.0.0:8080"
workers = "auto"
max_connections = 1000

[php]
enable = true
version = "8.3"
binary_path = "/usr/bin/php"
workers = 4
memory_limit = "256M"
max_execution_time = 300
ini_settings = [
    "opcache.enable=1",
    "opcache.memory_consumption=128",
    "upload_max_filesize=64M",
    "post_max_size=64M"
]

[cache]
enable = true
storage = "memory"
memory_limit = "256M"
default_ttl = 3600

# WordPress virtual host
[[virtualhost]]
domain = "*"
root = "/var/www/wordpress"
platform = "wordpress"
index = ["index.php", "index.html"]

[virtualhost.cache]
enable = false  # Disable cache for demo/development
VELOCONFIG

echo ""
echo "✅ WordPress installation complete!"
echo ""
echo "📁 WordPress location: $WORDPRESS_DIR"
echo "⚙️ Config file: $REPO_DIR/wordpress.toml"
echo ""
echo "🚀 To start WordPress with VeloServe:"
echo "   cd $REPO_DIR"
echo "   cargo run -- --config wordpress.toml"
echo ""
echo "   Or simply run: make wordpress"
echo ""
echo "🌐 Then open the forwarded port 8080 in your browser!"
echo ""

