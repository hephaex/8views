#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/// Error codes returned via the error_code_out parameter of sc_archive_read_page.
typedef enum SCArchiveError {
    SCArchiveErrorNone          = 0, ///< Success
    SCArchiveErrorIO            = 1, ///< I/O error or archive open failure
    SCArchiveErrorNotFound      = 2, ///< Entry not found (index out of range)
    SCArchiveErrorUnsupported   = 3, ///< Unsupported archive format
} SCArchiveError;

/// NSError domain for errors originating from the Rust sc-archive layer.
#define SCArchiveErrorDomain @"SCArchiveErrorDomain"

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

// ── ScPageList — open-once archive enumeration ────────────────────────────────

/// Opaque handle returned by sc_archive_open_pages.
/// Opens the archive once and caches all page metadata.
/// Must be released with sc_archive_pages_free.
typedef struct ScPageList ScPageList;

/// Open archive_path and collect all page metadata into an opaque ScPageList.
///
/// Returns a non-null ScPageList on success (release with sc_archive_pages_free).
/// Returns NULL on error; sets *error_code_out to SCArchiveErrorIO or
/// SCArchiveErrorUnsupported.  error_code_out may be NULL.
///
/// archive_path must be a valid NUL-terminated UTF-8 C string.
ScPageList * _Nullable sc_archive_open_pages(
    const char * _Nonnull archive_path,
    int32_t * _Nullable error_code_out
);

/// Return the total number of image pages in list.
uint32_t sc_page_list_count(const ScPageList * _Nonnull list);

/// Return the NUL-terminated filename for page at index.
/// The pointer is valid until sc_archive_pages_free is called on list.
/// Returns NULL if index is out of range. Do NOT free the returned pointer.
const char * _Nullable sc_page_list_name(const ScPageList * _Nonnull list, uint32_t index);

/// Return the uncompressed byte size of page at index. Returns 0 if out of range.
uint64_t sc_page_list_size(const ScPageList * _Nonnull list, uint32_t index);

/// Free a ScPageList returned by sc_archive_open_pages. NULL is safe.
void sc_archive_pages_free(ScPageList * _Nullable list);

#ifdef __cplusplus
}
#endif
