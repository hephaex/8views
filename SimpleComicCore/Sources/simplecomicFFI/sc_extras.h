#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/// Returns the library version string (process-lifetime pointer; do not free).
const char * _Nonnull sc_version(void);

/// Returns true if the file at the NUL-terminated UTF-8 path is a supported archive.
/// Checks by extension first; falls back to magic-byte detection.
/// Directories always return true. NULL path returns false.
bool sc_archive_is_supported(const char * _Nullable path);

/// Read raw image bytes for page `index` from the archive at `archive_path`.
///
/// On success: returns a heap-allocated buffer; writes byte count to `*out_len`;
/// sets `*error_code_out` to 0.  The caller must release the buffer with sc_free_bytes().
/// On failure: returns NULL and sets `*error_code_out`:
///   1 = I/O or archive open error
///   2 = entry not found (index out of range)
///   3 = unsupported format
///
/// archive_path must be a valid NUL-terminated UTF-8 C string.
/// out_len must be a valid non-null pointer.
/// error_code_out may be NULL (error code is discarded in that case).
uint8_t * _Nullable sc_archive_read_page(
    const char * _Nonnull archive_path,
    uint32_t index,
    size_t * _Nonnull out_len,
    int32_t * _Nullable error_code_out
);

/// Release a buffer returned by sc_archive_read_page.
/// Passing NULL is safe (no-op).
void sc_free_bytes(uint8_t * _Nullable ptr, size_t len);

#ifdef __cplusplus
}
#endif
