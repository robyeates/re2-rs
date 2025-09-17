// src/c-bindings.h
#pragma once
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RE2Wrapper RE2Wrapper;

// already present
RE2Wrapper* re2_new(const char* pattern, size_t pattern_len, const char** err_ptr, size_t* err_len);
void        re2_delete(RE2Wrapper* re2);
int         re2_ok(const RE2Wrapper* re2);
void        re2_error(const RE2Wrapper* re2, const char** err_ptr, size_t* err_len);
int         re2_full_match(const RE2Wrapper* re2, const char* text, size_t text_len);

// NEW: byte-range span into the input text. start is byte offset.
// If a group didn't participate, start == SIZE_MAX and len == 0.
typedef struct {
    size_t start;
    size_t len;
} re2_span_t;

// NEW: partial (UNANCHORED) boolean match
int re2_partial_match(const RE2Wrapper* re2, const char* text, size_t text_len);

// NEW: partial match with captures. `out_spans_len` is the capacity of `out_spans`.
// Fills up to `out_spans_len` spans, index 0 is the whole match, then capture 1..N.
// Returns 1 if matched, 0 if not. On success, `*written` is set to number of spans written.
int re2_partial_match_captures(
    const RE2Wrapper* re2,
    const char* text, size_t text_len,
    re2_span_t* out_spans, size_t out_spans_len,
    size_t* written
);

// NEW: full (ANCHOR_BOTH) variant with captures
int re2_full_match_captures(
    const RE2Wrapper* re2,
    const char* text, size_t text_len,
    re2_span_t* out_spans, size_t out_spans_len,
    size_t* written
);

int re2_group_count(const RE2Wrapper* re2);

#ifdef __cplusplus
} // extern "C"
#endif
