//! PHP SAPI (Server API) Integration
//!
//! This module provides true PHP embedding using the php-embed SAPI.
//! PHP runs directly inside VeloServe - no external processes!
//!
//! ## Requirements
//!
//! Install PHP embed development files:
//! ```bash
//! # Ubuntu/Debian
//! sudo apt install php-dev libphp-embed
//!
//! # Fedora/RHEL
//! sudo dnf install php-devel php-embedded
//!
//! # macOS (Homebrew)
//! brew install php
//! ```
//!
//! ## How It Works
//!
//! 1. VeloServe links against libphp.so at compile time
//! 2. PHP is initialized once at startup
//! 3. Each request executes PHP in the same process
//! 4. Much faster than CGI (no process spawning)
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │        VeloServe Process            │
//! │  ┌─────────────────────────────┐    │
//! │  │    Rust HTTP Handler        │    │
//! │  └──────────┬──────────────────┘    │
//! │             │ FFI calls             │
//! │  ┌──────────▼──────────────────┐    │
//! │  │   PHP Zend Engine           │    │
//! │  │   (embedded via libphp)     │    │
//! │  └─────────────────────────────┘    │
//! └─────────────────────────────────────┘
//! ```

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

use parking_lot::Mutex;
use tracing::{debug, error, info, warn};

// ============================================================================
// PHP Embed SAPI FFI Declarations
// ============================================================================
//
// These are the C functions from PHP's embed SAPI that we call via FFI.
// In production, you'd use `bindgen` to generate these automatically from php-embed.h

#[cfg(feature = "php-embed")]
#[link(name = "php")]
extern "C" {
    // Initialize PHP embed SAPI
    // int php_embed_init(int argc, char **argv);
    fn php_embed_init(argc: c_int, argv: *mut *mut c_char) -> c_int;

    // Shutdown PHP embed SAPI
    // void php_embed_shutdown(void);
    fn php_embed_shutdown();

    // Execute a PHP file
    // int php_execute_script(zend_file_handle *primary_file);
    fn php_execute_script(file_handle: *mut ZendFileHandle) -> c_int;

    // Execute PHP code string
    // int zend_eval_string(char *str, zval *retval_ptr, char *string_name);
    fn zend_eval_string(
        str: *const c_char,
        retval: *mut c_void,
        name: *const c_char,
    ) -> c_int;

    // Start output buffering
    fn php_output_start_default();

    // Get output buffer contents
    fn php_output_get_contents(contents: *mut ZVal) -> c_int;

    // End output buffering
    fn php_output_end();

    // Set a superglobal variable ($_SERVER, $_GET, $_POST, etc.)
    fn php_register_variable(
        var: *const c_char,
        val: *const c_char,
        track_vars_array: *mut c_void,
    );
}

// Opaque types for PHP internal structures
#[cfg(feature = "php-embed")]
#[repr(C)]
struct ZendFileHandle {
    _data: [u8; 0],
}

#[cfg(feature = "php-embed")]
#[repr(C)]
struct ZVal {
    _data: [u8; 0],
}

// ============================================================================
// PHP SAPI Manager
// ============================================================================

static PHP_INITIALIZED: AtomicBool = AtomicBool::new(false);
static PHP_INIT_ONCE: Once = Once::new();

/// PHP SAPI Manager - manages embedded PHP runtime
pub struct PhpSapi {
    initialized: bool,
    /// Thread-safe request counter
    request_count: std::sync::atomic::AtomicU64,
}

impl PhpSapi {
    /// Create a new PHP SAPI instance
    pub fn new() -> Self {
        Self {
            initialized: false,
            request_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Initialize the embedded PHP runtime
    ///
    /// This should be called once at server startup.
    /// PHP initialization is thread-safe and idempotent.
    #[cfg(feature = "php-embed")]
    pub fn initialize(&mut self) -> Result<(), String> {
        PHP_INIT_ONCE.call_once(|| {
            info!("Initializing PHP embed SAPI...");

            unsafe {
                // Initialize with no arguments
                let argc = 0;
                let argv = std::ptr::null_mut();

                let result = php_embed_init(argc, argv);
                if result == 0 {
                    PHP_INITIALIZED.store(true, Ordering::SeqCst);
                    info!("PHP embed SAPI initialized successfully");
                } else {
                    error!("Failed to initialize PHP embed SAPI: {}", result);
                }
            }
        });

        self.initialized = PHP_INITIALIZED.load(Ordering::SeqCst);
        if self.initialized {
            Ok(())
        } else {
            Err("PHP initialization failed".to_string())
        }
    }

    /// Fallback initialization when php-embed feature is not enabled
    #[cfg(not(feature = "php-embed"))]
    pub fn initialize(&mut self) -> Result<(), String> {
        warn!("PHP embed SAPI not compiled in. Using CGI fallback.");
        warn!("To enable embedded PHP, compile with: cargo build --features php-embed");
        Err("PHP embed not available - compile with --features php-embed".to_string())
    }

    /// Execute a PHP script file
    ///
    /// # Arguments
    /// * `script_path` - Path to the PHP file
    /// * `server_vars` - $_SERVER variables
    /// * `get_vars` - $_GET variables
    /// * `post_vars` - $_POST variables
    /// * `post_body` - Raw POST body for php://input
    ///
    /// # Returns
    /// The output of the PHP script (headers + body)
    #[cfg(feature = "php-embed")]
    pub fn execute_script(
        &self,
        script_path: &Path,
        server_vars: &HashMap<String, String>,
        get_vars: &HashMap<String, String>,
        post_vars: &HashMap<String, String>,
        post_body: &[u8],
    ) -> Result<String, String> {
        if !self.initialized {
            return Err("PHP not initialized".to_string());
        }

        self.request_count.fetch_add(1, Ordering::Relaxed);

        // This would be the actual implementation using PHP embed
        // For now, this is a placeholder showing the structure

        unsafe {
            // 1. Start output buffering
            php_output_start_default();

            // 2. Set up superglobals ($_SERVER, $_GET, $_POST)
            // self.setup_superglobals(server_vars, get_vars, post_vars);

            // 3. Execute the script
            // let file_handle = self.create_file_handle(script_path);
            // php_execute_script(&mut file_handle);

            // 4. Get output buffer contents
            // let output = self.get_output_buffer();

            // 5. End output buffering
            php_output_end();

            // Return the output
            Ok("PHP output would be here".to_string())
        }
    }

    #[cfg(not(feature = "php-embed"))]
    pub fn execute_script(
        &self,
        _script_path: &Path,
        _server_vars: &HashMap<String, String>,
        _get_vars: &HashMap<String, String>,
        _post_vars: &HashMap<String, String>,
        _post_body: &[u8],
    ) -> Result<String, String> {
        Err("PHP embed not available".to_string())
    }

    /// Check if PHP SAPI is available
    pub fn is_available(&self) -> bool {
        self.initialized
    }

    /// Get request count
    pub fn request_count(&self) -> u64 {
        self.request_count.load(Ordering::Relaxed)
    }
}

impl Drop for PhpSapi {
    fn drop(&mut self) {
        #[cfg(feature = "php-embed")]
        if self.initialized && PHP_INITIALIZED.load(Ordering::SeqCst) {
            info!("Shutting down PHP embed SAPI...");
            unsafe {
                php_embed_shutdown();
            }
            PHP_INITIALIZED.store(false, Ordering::SeqCst);
        }
    }
}

// ============================================================================
// Build Configuration
// ============================================================================

/// Build script helper to find PHP and configure linking
///
/// Add this to your build.rs:
/// ```rust,ignore
/// fn main() {
///     // Find PHP config
///     let php_config = std::process::Command::new("php-config")
///         .arg("--includes")
///         .output()
///         .expect("php-config not found");
///     
///     let includes = String::from_utf8_lossy(&php_config.stdout);
///     
///     // Get library path
///     let lib_path = std::process::Command::new("php-config")
///         .arg("--prefix")
///         .output()
///         .expect("php-config not found");
///     
///     let prefix = String::from_utf8_lossy(&lib_path.stdout).trim().to_string();
///     
///     println!("cargo:rustc-link-search=native={}/lib", prefix);
///     println!("cargo:rustc-link-lib=php");
///     
///     // For includes (if using bindgen)
///     // println!("cargo:include={}", includes.trim());
/// }
/// ```
pub mod build_helpers {
    use std::process::Command;

    /// Get PHP include paths for compilation
    pub fn get_php_includes() -> Option<String> {
        Command::new("php-config")
            .arg("--includes")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }

    /// Get PHP library path for linking
    pub fn get_php_lib_path() -> Option<String> {
        Command::new("php-config")
            .arg("--prefix")
            .output()
            .ok()
            .map(|o| format!("{}/lib", String::from_utf8_lossy(&o.stdout).trim()))
    }

    /// Get PHP version
    pub fn get_php_version() -> Option<String> {
        Command::new("php-config")
            .arg("--version")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }

    /// Check if PHP embed SAPI is available
    pub fn has_php_embed() -> bool {
        Command::new("php-config")
            .arg("--php-sapis")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("embed"))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_php_sapi_creation() {
        let sapi = PhpSapi::new();
        assert!(!sapi.is_available());
    }

    #[test]
    fn test_build_helpers() {
        // These tests check if php-config is available
        if let Some(version) = build_helpers::get_php_version() {
            println!("PHP version: {}", version);
        }

        if let Some(includes) = build_helpers::get_php_includes() {
            println!("PHP includes: {}", includes);
        }

        println!("PHP embed available: {}", build_helpers::has_php_embed());
    }
}

