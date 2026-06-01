#pragma once
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/// Returns the library version string (process-lifetime pointer; do not free).
const char * _Nonnull sc_version(void);

/// Returns true if the file at the NUL-terminated UTF-8 path is a supported archive.
/// Checks by extension first; falls back to magic-byte detection.
/// Directories always return true. NULL path returns false.
bool sc_archive_is_supported(const char * _Nullable path);

#ifdef __cplusplus
}
#endif
