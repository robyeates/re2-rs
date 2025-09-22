// src/c-bindings.cc
#include "c-bindings.h"
#include <re2/re2.h>
#include <vector>
#include <cstring>

struct RE2Wrapper {
    re2::RE2 re;
    explicit RE2Wrapper(const re2::StringPiece& pat, re2::RE2::Options opts = re2::RE2::Options())
        : re(pat, opts) {}
};

struct RE2Options {
    re2::RE2::Options opts;
};

extern "C" {

// ----- options -----
RE2Options* re2_options_new() {
    return new (std::nothrow) RE2Options();
}
void re2_options_delete(RE2Options* o) {
    delete o;
}
void re2_options_set_case_sensitive(RE2Options* o, int sensitive) {
    if (o) o->opts.set_case_sensitive(sensitive != 0);
}
void re2_options_set_posix_syntax(RE2Options* o, int posix) {
    if (o) o->opts.set_posix_syntax(posix != 0);
}
void re2_options_set_longest_match(RE2Options* o, int longest) {
    if (o) o->opts.set_longest_match(longest != 0);
}
// ICU-related toggles
void re2_options_set_word_boundary(RE2Options* o, int yes) {
    if (!o) o->opts.set_word_boundary(yes != 0);
}
void re2_options_set_perl_classes(RE2Options* o, int yes) {
    if (!o) o->opts.set_perl_classes(yes != 0);
}


RE2Wrapper* re2_new(const char* pattern, size_t pattern_len, const char** err_ptr, size_t* err_len) {
    re2::StringPiece pat(pattern, pattern_len);
    auto* w = new (std::nothrow) RE2Wrapper(pat);
    if (!w) {
        static const char kOOM[] = "re2_new: out of memory";
        if (err_ptr) *err_ptr = kOOM;
        if (err_len) *err_len = sizeof(kOOM) - 1;
        return nullptr;
    }
    if (!w->re.ok()) {
        if (err_ptr && err_len) {
            const std::string& e = w->re.error();
            *err_ptr = e.c_str();
            *err_len = e.size();
        }
    } else {
        if (err_ptr) *err_ptr = nullptr;
        if (err_len) *err_len = 0;
    }
    return w;
}

// ----- construct with options -----
RE2Wrapper* re2_new_with_options(const char* pattern, size_t pattern_len,
                                 const RE2Options* opts,
                                 const char** err_ptr, size_t* err_len) {
    re2::StringPiece pat(pattern, pattern_len);
    re2::RE2::Options o = opts ? opts->opts : re2::RE2::Options();
    auto* w = new (std::nothrow) RE2Wrapper(pat, o);
    if (!w) {
        static const char kOOM[] = "re2_new_with_options: out of memory";
        if (err_ptr) *err_ptr = kOOM;
        if (err_len) *err_len = sizeof(kOOM) - 1;
        return nullptr;
    }
    if (!w->re.ok()) {
        if (err_ptr && err_len) {
            const std::string& e = w->re.error();
            *err_ptr = e.c_str();
            *err_len = e.size();
        }
    } else {
        if (err_ptr) *err_ptr = nullptr;
        if (err_len) *err_len = 0;
    }
    return w;
}


void re2_delete(RE2Wrapper* re2) { delete re2; }

int re2_ok(const RE2Wrapper* re2) { return (re2 && re2->re.ok()) ? 1 : 0; }

void re2_error(const RE2Wrapper* re2, const char** err_ptr, size_t* err_len) {
    if (!re2 || !err_ptr || !err_len) return;
    const std::string& e = re2->re.error();
    *err_ptr = e.empty() ? nullptr : e.c_str();
    *err_len = e.size();
}

int re2_full_match(const RE2Wrapper* re2, const char* text, size_t text_len) {
    if (!re2) return 0;
    re2::StringPiece t(text, text_len);
    return re2::RE2::FullMatch(t, re2->re) ? 1 : 0;
}

static int do_match_with_captures(
    const RE2Wrapper* w,
    const char* text, size_t text_len,
    re2::RE2::Anchor anchor,
    re2_span_t* out_spans, size_t out_spans_len,
    size_t* written
) {
    if (written) *written = 0;
    if (!w || !out_spans || out_spans_len == 0) return 0;

    // Number of capturing groups + whole match
    int ncap = 1 + w->re.NumberOfCapturingGroups();
    size_t to_write = static_cast<size_t>(ncap);
    if (to_write > out_spans_len) to_write = out_spans_len;

    // Prepare submatch array (RE2 expects pointers you own)
    std::vector<re2::StringPiece> subs(static_cast<size_t>(ncap));

    re2::StringPiece t(text, text_len);
    bool ok = w->re.Match(
        t,
        0, static_cast<int>(t.size()),
        anchor,
        subs.data(), ncap
    );
    if (!ok) return 0;

    // Convert to byte offsets
    for (size_t i = 0; i < to_write; ++i) {
        if (subs[i].data() == nullptr) { // group didn't participate
            out_spans[i].start = (size_t)-1;
            out_spans[i].len   = 0;
        } else {
            const char* p = subs[i].data();
            size_t start = static_cast<size_t>(p - text);
            size_t len   = static_cast<size_t>(subs[i].size());
            out_spans[i].start = start;
            out_spans[i].len   = len;
        }
    }
    if (written) *written = to_write;
    return 1;
}

int re2_partial_match(const RE2Wrapper* re2, const char* text, size_t text_len) {
    if (!re2) return 0;
    re2::StringPiece t(text, text_len);
    return re2->re.Match(t, 0, static_cast<int>(t.size()), re2::RE2::UNANCHORED, nullptr, 0) ? 1 : 0;
}

int re2_partial_match_captures(
    const RE2Wrapper* re2,
    const char* text, size_t text_len,
    re2_span_t* out_spans, size_t out_spans_len,
    size_t* written
) {
    return do_match_with_captures(re2, text, text_len, re2::RE2::UNANCHORED, out_spans, out_spans_len, written);
}

int re2_full_match_captures(
    const RE2Wrapper* re2,
    const char* text, size_t text_len,
    re2_span_t* out_spans, size_t out_spans_len,
    size_t* written
) {
    return do_match_with_captures(re2, text, text_len, re2::RE2::ANCHOR_BOTH, out_spans, out_spans_len, written);
}

int re2_group_count(const RE2Wrapper* re2) {
    if (!re2) return 0;
    return re2->re.NumberOfCapturingGroups();
}

static int copy_out(const std::string& s, char* out_buf, size_t out_len, size_t* written) {
    if (written) *written = s.size();
    if (!out_buf || out_len == 0) return 0;
    if (s.size() + 1 > out_len) return 0;
    std::memcpy(out_buf, s.data(), s.size());
    out_buf[s.size()] = '\0';
    return 1;
}

int re2_replace_one(const RE2Wrapper* w,
                    const char* text, size_t text_len,
                    const char* rewrite, size_t rewrite_len,
                    char* out_buf, size_t out_len,
                    size_t* written) {
    if (!w) return 0;
    std::string result(text, text_len);
    std::string rew(rewrite, rewrite_len);
    bool ok = re2::RE2::Replace(&result, w->re, rew);
    if (!ok) return 0;
    return copy_out(result, out_buf, out_len, written);
}

int re2_replace_all(const RE2Wrapper* w,
                    const char* text, size_t text_len,
                    const char* rewrite, size_t rewrite_len,
                    char* out_buf, size_t out_len,
                    size_t* written) {
    if (!w) return 0;
    std::string result(text, text_len);
    std::string rew(rewrite, rewrite_len);
    int n = re2::RE2::GlobalReplace(&result, w->re, rew);
    if (n <= 0) return 0;
    return copy_out(result, out_buf, out_len, written);
}


// Return 1 if this build of RE2 has ICU enabled, else 0.
int re2_has_icu() {
#ifdef RE2_WITH_ICU
    return 1;
#else
    return 0;
#endif
}

} // extern "C"
