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

// ── Session state ─────────────────────────────────────────────────────────────

/// Session state for one archive. Mirrors the Rust SessionStateRecord.
typedef struct {
    uint32_t page_index;        ///< 0-based page (matches sc_archive_read_page index)
    double   zoom_level;        ///< 1.0 = 100%
    int32_t  rotation_degrees;  ///< 0, 90, 180, or 270
    bool     two_page_spread;
    bool     right_to_left;     ///< true = manga (right-to-left) mode
    int32_t  scale_mode;        ///< 0=fit-window, 1=fit-width, 2=actual-size
    double   scroll_x;
    double   scroll_y;
} ScSessionState;

/// Load persisted session state for archive_path.
///
/// On success writes state to *out_state and returns true.
/// On failure (no record or DB error) writes default state and returns false.
/// archive_path must be a valid NUL-terminated UTF-8 C string.
/// out_state must be a valid non-null pointer.
bool sc_session_load(
    const char * _Nonnull archive_path,
    ScSessionState * _Nonnull out_state
);

/// Persist session state for archive_path. Returns true on success.
///
/// archive_path and state must be valid non-null pointers.
bool sc_session_save(
    const char * _Nonnull archive_path,
    const ScSessionState * _Nonnull state
);

/// Delete persisted session state for archive_path. No-op if not found. NULL is safe.
void sc_session_delete_c(const char * _Nullable archive_path);

// ── Thumbnail ─────────────────────────────────────────────────────────────────

/// Generate a thumbnail from compressed image bytes (JPEG, PNG, WebP, …).
///
/// thumb_size = max dimension; output is at most thumb_size × thumb_size pixels.
///
/// On success: returns heap-allocated RGBA buffer (4 bytes/pixel, row-major);
/// writes pixel dimensions to *out_width/*out_height, byte count to *out_len,
/// sets *error_code_out to 0.  Caller must release with sc_free_bytes(ptr, *out_len).
/// On failure: returns NULL; sets *error_code_out to 1.
///
/// image_bytes must point to image_len valid bytes.
/// out_len, out_width, out_height must be valid non-null pointers.
/// error_code_out may be NULL.
uint8_t * _Nullable sc_thumbnail_from_bytes(
    const uint8_t * _Nonnull image_bytes,
    size_t image_len,
    uint32_t thumb_size,
    size_t * _Nonnull out_len,
    uint32_t * _Nonnull out_width,
    uint32_t * _Nonnull out_height,
    int32_t * _Nullable error_code_out
);

#ifdef __cplusplus
}
#endif
