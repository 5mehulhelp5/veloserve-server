//! Build script for VeloServe
//!
//! When compiled with `--features php-embed`, this script:
//! 1. Finds PHP installation using php-config
//! 2. Configures linking against libphp
//! 3. Sets up include paths for FFI

use std::env;
use std::process::Command;

fn main() {
    // Only run PHP detection if the php-embed feature is enabled
    if env::var("CARGO_FEATURE_PHP_EMBED").is_ok() {
        println!("cargo:rerun-if-changed=build.rs");
        
        setup_php_embed();
    }
}

fn setup_php_embed() {
    println!("cargo:warning=Building with PHP embed SAPI support");

    // Get PHP library path
    let lib_dir = get_php_config("--prefix")
        .map(|p| format!("{}/lib", p.trim()))
        .unwrap_or_else(|| "/usr/lib".to_string());
    
    // Also check common locations
    let lib_paths = [
        &lib_dir,
        "/usr/lib",
        "/usr/lib/x86_64-linux-gnu",
        "/usr/local/lib",
    ];

    for path in &lib_paths {
        println!("cargo:rustc-link-search=native={}", path);
    }

    // Link against PHP library
    // Try different library names in order of preference
    let php_version = get_php_config("--version")
        .map(|v| v.trim().split('.').take(2).collect::<Vec<_>>().join("."))
        .unwrap_or_else(|| "8.3".to_string());
    
    let major_minor = php_version.replace('.', "");
    
    // Try version-specific first, then generic
    // The library is typically named libphp8.3.so or libphp.so
    println!("cargo:rustc-link-lib=php{}", php_version);
    
    // Get additional libraries PHP depends on
    if let Some(libs) = get_php_config("--libs") {
        for lib in libs.split_whitespace() {
            if lib.starts_with("-l") {
                let lib_name = &lib[2..];
                println!("cargo:rustc-link-lib={}", lib_name);
            }
        }
    }
    
    // Get PHP include paths (for potential bindgen use)
    if let Some(includes) = get_php_config("--includes") {
        for inc in includes.split_whitespace() {
            if inc.starts_with("-I") {
                let path = &inc[2..];
                println!("cargo:include={}", path);
            }
        }
    }

    // Set environment variable for the crate to know PHP version
    println!("cargo:rustc-env=PHP_VERSION={}", php_version);
    
    println!("cargo:warning=PHP {} embed SAPI configured successfully", php_version);
}

fn get_php_config(arg: &str) -> Option<String> {
    Command::new("php-config")
        .arg(arg)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

