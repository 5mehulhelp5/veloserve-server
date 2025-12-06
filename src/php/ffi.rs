//! PHP FFI Bindings
//!
//! Low-level FFI bindings to PHP's embed SAPI.
//! These are the raw C function declarations.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::os::raw::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong, c_void};

// ============================================================================
// PHP Type Definitions
// ============================================================================

/// PHP's zval structure - the core PHP value type
/// This is an opaque type for safety
#[repr(C)]
pub struct zval {
    _data: [u8; 16], // zval is typically 16 bytes on 64-bit
}

impl Default for zval {
    fn default() -> Self {
        Self { _data: [0; 16] }
    }
}

/// PHP file handle for script execution
#[repr(C)]
pub struct zend_file_handle {
    pub handle: zend_file_handle_union,
    pub filename: *const c_char,
    pub opened_path: *mut zend_string,
    pub type_: zend_stream_type,
    pub primary_script: bool,
    pub in_list: bool,
    pub buf: *mut c_char,
    pub len: usize,
}

#[repr(C)]
pub union zend_file_handle_union {
    pub fp: *mut c_void,       // FILE*
    pub stream: zend_stream,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct zend_stream {
    pub handle: *mut c_void,
    pub isatty: c_int,
    pub mmap: zend_mmap,
    pub reader: *mut c_void,
    pub fsizer: *mut c_void,
    pub closer: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct zend_mmap {
    pub len: usize,
    pub pos: usize,
    pub map: *mut c_char,
    pub buf: *mut c_char,
    pub old_handle: *mut c_void,
    pub old_closer: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum zend_stream_type {
    ZEND_HANDLE_FILENAME = 0,
    ZEND_HANDLE_FP = 1,
    ZEND_HANDLE_STREAM = 2,
}

/// Opaque zend_string type
#[repr(C)]
pub struct zend_string {
    _data: [u8; 0],
}

/// SAPI module structure
#[repr(C)]
pub struct sapi_module_struct {
    pub name: *mut c_char,
    pub pretty_name: *mut c_char,
    // ... many more fields, but we don't need to access them
    _rest: [u8; 512], // Padding for the rest of the structure
}

/// PHP result type
pub type zend_result = c_int;
pub const SUCCESS: zend_result = 0;
pub const FAILURE: zend_result = -1;

// ============================================================================
// PHP Embed SAPI Functions
// ============================================================================

#[cfg(feature = "php-embed")]
#[link(name = "php8.3")]
extern "C" {
    /// Initialize PHP embed SAPI
    /// Returns 0 on success, -1 on failure
    pub fn php_embed_init(argc: c_int, argv: *mut *mut c_char) -> c_int;

    /// Shutdown PHP embed SAPI
    pub fn php_embed_shutdown();

    /// The embed SAPI module struct
    pub static mut php_embed_module: sapi_module_struct;
}

// ============================================================================
// Zend Engine Functions
// ============================================================================

#[cfg(feature = "php-embed")]
#[link(name = "php8.3")]
extern "C" {
    /// Execute a PHP file
    /// Returns true on success
    pub fn php_execute_script(primary_file: *mut zend_file_handle) -> bool;

    /// Evaluate a PHP string
    pub fn zend_eval_string(
        str: *const c_char,
        retval_ptr: *mut zval,
        string_name: *const c_char,
    ) -> zend_result;

    /// Evaluate a PHP string with length
    pub fn zend_eval_stringl(
        str: *const c_char,
        str_len: usize,
        retval_ptr: *mut zval,
        string_name: *const c_char,
    ) -> zend_result;

    /// Execute multiple scripts
    pub fn zend_execute_scripts(
        type_: c_int,
        retval: *mut zval,
        file_count: c_int,
        ...
    ) -> zend_result;

    /// Initialize a file handle for a filename
    pub fn zend_stream_init_filename(
        handle: *mut zend_file_handle,
        filename: *const c_char,
    );

    /// Destroy a file handle
    pub fn zend_destroy_file_handle(handle: *mut zend_file_handle);
}

// ============================================================================
// PHP Output Functions
// ============================================================================

#[cfg(feature = "php-embed")]
#[link(name = "php8.3")]
extern "C" {
    /// Start default output buffering
    pub fn php_output_start_default() -> c_int;

    /// Get output buffer contents
    pub fn php_output_get_contents(contents: *mut zval) -> c_int;

    /// Discard output buffer
    pub fn php_output_discard() -> c_int;

    /// End and flush output buffer
    pub fn php_output_end() -> c_int;

    /// Get length of output buffer
    pub fn php_output_get_length(len: *mut usize) -> c_int;
}

// ============================================================================
// PHP Variable Registration
// ============================================================================

#[cfg(feature = "php-embed")]
#[link(name = "php8.3")]
extern "C" {
    /// Register a variable in a track_vars array
    pub fn php_register_variable(
        var: *const c_char,
        val: *const c_char,
        track_vars_array: *mut zval,
    );

    /// Register a variable with length
    pub fn php_register_variable_ex(
        var_name: *const c_char,
        val: *mut zval,
        track_vars_array: *mut zval,
    );
}

// ============================================================================
// PHP Request Lifecycle
// ============================================================================

#[cfg(feature = "php-embed")]
#[link(name = "php8.3")]
extern "C" {
    /// Start a request
    pub fn php_request_startup() -> c_int;

    /// Shutdown a request
    pub fn php_request_shutdown(dummy: *mut c_void);

    /// Module startup
    pub fn php_module_startup(
        sf: *mut sapi_module_struct,
        additional_modules: *mut c_void,
        num_additional_modules: c_uint,
    ) -> c_int;

    /// Module shutdown
    pub fn php_module_shutdown();
}

// ============================================================================
// SAPI Functions
// ============================================================================

#[cfg(feature = "php-embed")]
#[link(name = "php8.3")]
extern "C" {
    /// Set SAPI header
    pub fn sapi_add_header(
        header_line: *mut c_char,
        header_line_len: usize,
        duplicate: c_int,
    ) -> c_int;

    /// Get SAPI request info
    pub fn SG(what: c_int) -> *mut c_void;
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a null-terminated C string from a Rust string
pub fn to_cstring(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).unwrap_or_else(|_| std::ffi::CString::new("").unwrap())
}

/// Check if PHP embed is available (compile-time check)
pub fn is_embed_available() -> bool {
    cfg!(feature = "php-embed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zval_size() {
        // Verify our zval is at least the size of a pointer
        assert!(std::mem::size_of::<zval>() >= 16);
    }

    #[test]
    fn test_embed_available() {
        let available = is_embed_available();
        #[cfg(feature = "php-embed")]
        assert!(available);
        #[cfg(not(feature = "php-embed"))]
        assert!(!available);
    }
}

