#include "c-bindings.h"
#include "re2/re2.h"
#include <string>

struct RE2Wrapper {
    RE2* re;
    std::string error;
};

RE2Wrapper* re2_new(const char* pattern) {
    auto wrapper = new RE2Wrapper;
    wrapper->re = new RE2(pattern);
    if (!wrapper->re->ok()) {
        wrapper->error = wrapper->re->error();
    }
    return wrapper;
}

void re2_delete(RE2Wrapper* wrapper) {
    if (wrapper) {
        delete wrapper->re;
        delete wrapper;
    }
}

bool re2_ok(const RE2Wrapper* wrapper) {
    return wrapper && wrapper->re && wrapper->re->ok();
}

const char* re2_error(const RE2Wrapper* wrapper) {
    if (!wrapper) return "null wrapper";
    return wrapper->error.empty() ? nullptr : wrapper->error.c_str();
}

bool re2_full_match(const RE2Wrapper* wrapper, const char* text) {
    if (!wrapper || !wrapper->re) return false;
    return RE2::FullMatch(text, *(wrapper->re));
}
