use std::ffi::c_char;
// src/ffi.rs (used in no-bindgen or for hand-written mirrors)
use std::os::raw::c_int;

#[repr(C)]
pub struct RE2Wrapper { _private: [u8; 0] }

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct re2_span_t {
    pub start: usize,
    pub len: usize,
}


unsafe extern "C" {
    pub fn re2_new(p: *const c_char, p_len: usize, err_ptr: *mut *const c_char, err_len: *mut usize) -> *mut RE2Wrapper;
    pub fn re2_delete(r: *mut RE2Wrapper);
    pub fn re2_ok(r: *const RE2Wrapper) -> c_int;
    pub fn re2_error(r: *const RE2Wrapper, err_ptr: *mut *const c_char, err_len: *mut usize);
    pub fn re2_full_match(r: *const RE2Wrapper, t: *const c_char, t_len: usize) -> c_int;

    pub fn re2_partial_match(r: *const RE2Wrapper, t: *const c_char, t_len: usize) -> c_int;
    pub fn re2_partial_match_captures(
        r: *const RE2Wrapper,
        t: *const c_char, t_len: usize,
        out_spans: *mut re2_span_t, out_spans_len: usize,
        written: *mut usize
    ) -> c_int;

    pub fn re2_full_match_captures(
        r: *const RE2Wrapper,
        t: *const c_char, t_len: usize,
        out_spans: *mut re2_span_t, out_spans_len: usize,
        written: *mut usize
    ) -> c_int;

    pub fn re2_group_count(r: *const RE2Wrapper) -> c_int;
}
