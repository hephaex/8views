// FFI entry point — uniffi scaffolding will be generated here in Sprint 14.
// For now this exports a smoke-test function to verify the static lib builds.

/// Returns the library version string.
#[no_mangle]
pub extern "C" fn sc_version() -> *const std::ffi::c_char {
    c"0.1.0".as_ptr()
}
