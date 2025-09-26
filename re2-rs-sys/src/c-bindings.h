// src/c-bindings.h
#pragma once
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RE2Wrapper RE2Wrapper;

// NEW: byte-range span into the input text. start is byte offset.
// If a group didn't participate, start == SIZE_MAX and len == 0.
typedef struct {
    size_t start;
    size_t len;
} re2_span_t;

// already present
RE2Wrapper* re2_new(const char* pattern, size_t pattern_len, const char** err_ptr, size_t* err_len);

// Options handle
typedef struct RE2Options RE2Options;

RE2Options* re2_options_new(void);
void        re2_options_delete(RE2Options*);
void        re2_options_set_case_sensitive(RE2Options* o, int sensitive);
void        re2_options_set_posix_syntax(RE2Options* o, int posix);
void        re2_options_set_longest_match(RE2Options* o, int longest);
void        re2_options_set_word_boundary(RE2Options* o, int yes);
void        re2_options_set_perl_classes(RE2Options* o, int yes);

// Construct with options
RE2Wrapper* re2_new_with_options(const char* pattern, size_t pattern_len,
                                 const RE2Options* opts,
                                 const char** err_ptr, size_t* err_len);

void        re2_delete(RE2Wrapper* re2);
int         re2_ok(const RE2Wrapper* re2);
void        re2_error(const RE2Wrapper* re2, const char** err_ptr, size_t* err_len);
int         re2_full_match(const RE2Wrapper* re2, const char* text, size_t text_len);



int re2_replace_one(const RE2Wrapper* re,
                    const char* text, size_t text_len,
                    const char* rewrite, size_t rewrite_len,
                    char* out_buf, size_t out_len,
                    size_t* written);

// Replace all matches
int re2_replace_all(const RE2Wrapper* re,
                    const char* text, size_t text_len,
                    const char* rewrite, size_t rewrite_len,
                    char* out_buf, size_t out_len,
                    size_t* written);

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

//Regex::captures (anchored full match only) vs. captures_iter (streaming unanchored matches)
// Iterator over matches of a compiled RE2 on a given input.
typedef struct RE2Iter RE2Iter;

RE2Iter* re2_iter_new(const RE2Wrapper* re2, const char* text, size_t len);

// Single-match iterator (whole match only). Returns 1 if a match was produced, 0 when done.
int re2_iter_next(RE2Iter* it, re2_span_t* out_span);

// Capturing iterator. Fills up to out_spans_len spans (0..=n groups). `*written` is set to
// the number of spans written. Returns 1 if a match was produced, 0 when done.
int re2_iter_next_captures(RE2Iter* it,
                           re2_span_t* out_spans,
                           size_t out_spans_len,
                           size_t* written);

void re2_iter_delete(RE2Iter* it);


// Return 1 if this build of RE2 has ICU enabled, else 0.
int re2_has_icu();

#ifdef __cplusplus
} // extern "C"
#endif
