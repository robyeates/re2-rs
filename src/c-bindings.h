#pragma once

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque wrapper around RE2 object
typedef struct RE2Wrapper RE2Wrapper;

// Create a new regex (pattern is UTF-8 string)
RE2Wrapper* re2_new(const char* pattern);

// Free regex
void re2_delete(RE2Wrapper* re);

// Return true if regex is valid
bool re2_ok(const RE2Wrapper* re);

// Return error message (nullptr if ok)
const char* re2_error(const RE2Wrapper* re);

// Simple full match (returns true if pattern matches whole string)
bool re2_full_match(const RE2Wrapper* re, const char* text);

#ifdef __cplusplus
}  // extern "C"
#endif
