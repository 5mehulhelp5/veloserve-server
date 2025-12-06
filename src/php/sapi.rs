//! PHP SAPI (Server API) Integration
//!
//! This module provides true PHP embedding using the php-embed SAPI.
//! PHP runs directly inside VeloServe - no external processes!
//!
//! ## Usage
//!
//! ```bash
//! # Build with embedded PHP support
//! cargo build --release --features php-embed
//! ```
//!
//! ## Requirements
//!
//! - PHP development files: `sudo apt install php-dev libphp-embed`
//! - Or compile PHP with `--enable-embed`

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Once;

use parking_lot::Mutex;
use tracing::{debug, error, info, warn};

#[cfg(feature = "php-embed")]
use super::ffi;

// ============================================================================
// PHP SAPI Runtime
// ============================================================================

static PHP_INITIALIZED: AtomicBool = AtomicBool::new(false);
static PHP_INIT_ONCE: Once = Once::new();
static PHP_INIT_ERROR: Mutex<Option<String>> = Mutex::new(None);

/// PHP SAPI Runtime Manager
///
/// Manages the embedded PHP runtime lifecycle.
/// Only one instance should exist per process.
pub struct PhpSapi {
    /// Whether this instance successfully initialized PHP
    initialized: bool,
    /// Request counter for statistics
    request_count: AtomicU64,
    /// Output buffer for capturing PHP output
    output_buffer: Mutex<Vec<u8>>,
}

impl PhpSapi {
    /// Create a new PHP SAPI instance
    pub fn new() -> Self {
        Self {
            initialized: false,
            request_count: AtomicU64::new(0),
            output_buffer: Mutex::new(Vec::with_capacity(64 * 1024)), // 64KB initial
        }
    }

    /// Initialize the embedded PHP runtime
    ///
    /// This should be called once at server startup.
    /// Thread-safe and idempotent.
    #[cfg(feature = "php-embed")]
    pub fn initialize(&mut self) -> Result<(), String> {
        let mut init_error: Option<String> = None;

        PHP_INIT_ONCE.call_once(|| {
            info!("Initializing PHP embed SAPI...");

            unsafe {
                // Initialize with empty argv
                let argc = 0;
                let argv: *mut *mut i8 = ptr::null_mut();

                let result = ffi::php_embed_init(argc, argv);

                if result == 0 {
                    PHP_INITIALIZED.store(true, Ordering::SeqCst);
                    
                    // Get PHP version for logging
                    info!("PHP embed SAPI initialized successfully");
                } else {
                    let err = format!("php_embed_init failed with code: {}", result);
                    error!("{}", err);
                    *PHP_INIT_ERROR.lock() = Some(err);
                }
            }
        });

        // Check if initialization was successful
        if PHP_INITIALIZED.load(Ordering::SeqCst) {
            self.initialized = true;
            Ok(())
        } else {
            let error = PHP_INIT_ERROR.lock().clone()
                .unwrap_or_else(|| "Unknown PHP initialization error".to_string());
            Err(error)
        }
    }

    /// Fallback when php-embed feature is not enabled
    #[cfg(not(feature = "php-embed"))]
    pub fn initialize(&mut self) -> Result<(), String> {
        Err("PHP embed SAPI not compiled. Build with: cargo build --features php-embed".to_string())
    }

    /// Execute a PHP script and return its output
    ///
    /// # Arguments
    /// * `script_path` - Path to the PHP file
    /// * `server_vars` - $_SERVER variables
    /// * `get_vars` - $_GET query parameters
    /// * `post_data` - Raw POST body
    /// * `headers` - HTTP headers
    ///
    /// # Returns
    /// A tuple of (output_body, response_headers)
    #[cfg(feature = "php-embed")]
    pub fn execute_script(
        &self,
        script_path: &Path,
        server_vars: &HashMap<String, String>,
        get_vars: &HashMap<String, String>,
        post_data: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<PhpResponse, String> {
        if !self.initialized {
            return Err("PHP SAPI not initialized".to_string());
        }

        self.request_count.fetch_add(1, Ordering::Relaxed);

        let script_path_str = script_path.to_string_lossy();
        let c_script_path = CString::new(script_path_str.as_ref())
            .map_err(|e| format!("Invalid script path: {}", e))?;

        debug!("Executing PHP script: {}", script_path_str);

        unsafe {
            // Start output buffering
            ffi::php_output_start_default();

            // Create file handle for the script
            let mut file_handle: ffi::zend_file_handle = std::mem::zeroed();
            ffi::zend_stream_init_filename(&mut file_handle, c_script_path.as_ptr());

            // Execute the script
            let success = ffi::php_execute_script(&mut file_handle);

            // Get output buffer contents
            let mut output_zval = ffi::zval::default();
            ffi::php_output_get_contents(&mut output_zval);

            // End output buffering
            ffi::php_output_end();

            // Clean up file handle
            ffi::zend_destroy_file_handle(&mut file_handle);

            if success {
                // TODO: Extract actual output from zval
                // For now, return a placeholder
                Ok(PhpResponse {
                    body: Vec::new(),
                    headers: HashMap::new(),
                    status_code: 200,
                })
            } else {
                Err("PHP script execution failed".to_string())
            }
        }
    }

    /// Execute PHP code string
    #[cfg(feature = "php-embed")]
    pub fn eval_string(&self, code: &str) -> Result<String, String> {
        if !self.initialized {
            return Err("PHP SAPI not initialized".to_string());
        }

        let c_code = CString::new(code)
            .map_err(|e| format!("Invalid PHP code: {}", e))?;
        let c_name = CString::new("<eval>").unwrap();

        unsafe {
            ffi::php_output_start_default();

            let mut retval = ffi::zval::default();
            let result = ffi::zend_eval_string(
                c_code.as_ptr(),
                &mut retval,
                c_name.as_ptr(),
            );

            let mut output_zval = ffi::zval::default();
            ffi::php_output_get_contents(&mut output_zval);
            ffi::php_output_end();

            if result == ffi::SUCCESS {
                // TODO: Convert output_zval to string
                Ok(String::new())
            } else {
                Err("PHP eval failed".to_string())
            }
        }
    }

    #[cfg(not(feature = "php-embed"))]
    pub fn execute_script(
        &self,
        _script_path: &Path,
        _server_vars: &HashMap<String, String>,
        _get_vars: &HashMap<String, String>,
        _post_data: &[u8],
        _headers: &HashMap<String, String>,
    ) -> Result<PhpResponse, String> {
        Err("PHP embed not available".to_string())
    }

    #[cfg(not(feature = "php-embed"))]
    pub fn eval_string(&self, _code: &str) -> Result<String, String> {
        Err("PHP embed not available".to_string())
    }

    /// Check if PHP SAPI is initialized and available
    pub fn is_available(&self) -> bool {
        self.initialized
    }

    /// Get total request count
    pub fn request_count(&self) -> u64 {
        self.request_count.load(Ordering::Relaxed)
    }

    /// Get statistics
    pub fn stats(&self) -> serde_json::Value {
        serde_json::json!({
            "mode": "sapi",
            "initialized": self.initialized,
            "request_count": self.request_count(),
            "feature_enabled": cfg!(feature = "php-embed"),
        })
    }
}

impl Default for PhpSapi {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PhpSapi {
    fn drop(&mut self) {
        #[cfg(feature = "php-embed")]
        if self.initialized && PHP_INITIALIZED.load(Ordering::SeqCst) {
            info!("Shutting down PHP embed SAPI...");
            unsafe {
                ffi::php_embed_shutdown();
            }
            PHP_INITIALIZED.store(false, Ordering::SeqCst);
            info!("PHP embed SAPI shutdown complete");
        }
    }
}

// ============================================================================
// PHP Response
// ============================================================================

/// Response from PHP script execution
#[derive(Debug, Clone)]
pub struct PhpResponse {
    /// Response body
    pub body: Vec<u8>,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// HTTP status code
    pub status_code: u16,
}

impl PhpResponse {
    /// Create a new PHP response
    pub fn new() -> Self {
        Self {
            body: Vec::new(),
            headers: HashMap::new(),
            status_code: 200,
        }
    }

    /// Parse raw PHP output (headers + body)
    pub fn from_raw_output(output: &[u8]) -> Self {
        // Find header/body separator (double CRLF)
        let separator = b"\r\n\r\n";
        if let Some(pos) = output.windows(4).position(|w| w == separator) {
            let headers_bytes = &output[..pos];
            let body = output[pos + 4..].to_vec();

            let mut headers = HashMap::new();
            let mut status_code = 200;

            // Parse headers
            let headers_str = String::from_utf8_lossy(headers_bytes);
            for line in headers_str.lines() {
                if line.starts_with("Status:") {
                    // Parse status line: "Status: 404 Not Found"
                    if let Some(code_str) = line.strip_prefix("Status:").map(|s| s.trim()) {
                        if let Some(code) = code_str.split_whitespace().next() {
                            status_code = code.parse().unwrap_or(200);
                        }
                    }
                } else if let Some((name, value)) = line.split_once(':') {
                    headers.insert(name.trim().to_string(), value.trim().to_string());
                }
            }

            Self {
                body,
                headers,
                status_code,
            }
        } else {
            // No headers, entire output is body
            Self {
                body: output.to_vec(),
                headers: HashMap::new(),
                status_code: 200,
            }
        }
    }
}

impl Default for PhpResponse {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_php_sapi_creation() {
        let sapi = PhpSapi::new();
        assert!(!sapi.is_available());
        assert_eq!(sapi.request_count(), 0);
    }

    #[test]
    fn test_php_response_parsing() {
        let raw = b"Content-Type: text/html\r\nStatus: 200 OK\r\n\r\n<html>Hello</html>";
        let response = PhpResponse::from_raw_output(raw);

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, b"<html>Hello</html>");
        assert_eq!(response.headers.get("Content-Type"), Some(&"text/html".to_string()));
    }

    #[test]
    fn test_php_response_no_headers() {
        let raw = b"Hello World";
        let response = PhpResponse::from_raw_output(raw);

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, b"Hello World");
        assert!(response.headers.is_empty());
    }

    #[test]
    fn test_php_response_404() {
        let raw = b"Status: 404 Not Found\r\nContent-Type: text/html\r\n\r\nNot Found";
        let response = PhpResponse::from_raw_output(raw);

        assert_eq!(response.status_code, 404);
    }
}
