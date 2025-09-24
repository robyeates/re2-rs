use re2_rs_sys::*;
use std::{ffi::c_char, ptr, slice, str};

/// Raw pointer type alias for readability
pub type RE2WrapperHandle = *mut RE2Wrapper;
pub type OptionsHandle = *mut RE2Options;

/// Safe-ish wrapper around RE2 options (opaque to Rust)
/// https://github.com/google/re2/blob/main/re2/re2.h#L678
pub struct Options(pub(crate) OptionsHandle);

impl Options {
    pub fn new() -> Self {
        unsafe { Options(re2_options_new()) }
    }
    pub fn case_insensitive(self, yes: bool) -> Self {
        unsafe { re2_options_set_case_sensitive(self.0, if yes { 0 } else { 1 }); }
        self
    }
    pub fn posix_syntax(self, yes: bool) -> Self {
        unsafe { re2_options_set_posix_syntax(self.0, yes as i32); }
        self
    }
    pub fn longest_match(self, yes: bool) -> Self {
        unsafe { re2_options_set_longest_match(self.0, yes as i32); }
        self
    }

    pub fn unicode_word_boundaries(self, yes: bool) -> Self {
        unsafe { re2_options_set_word_boundary(self.0, yes as i32); }
        self
    }

    pub fn perl_classes(self, yes: bool) -> Self {
        unsafe { re2_options_set_perl_classes(self.0, yes as i32); }
        self
    }
}

impl Drop for Options {
    fn drop(&mut self) {
        unsafe { re2_options_delete(self.0) }
    }
}

/// Unified constructor that works with or without ICU
pub fn compile_regex(pattern: &str, opts: Option<&Options>) -> Result<RE2WrapperHandle, String> {
    let cpat = pattern.as_bytes();
    let mut err_ptr: *const c_char = ptr::null();
    let mut err_len: usize = 0;

    let raw = unsafe {
        #[cfg(feature = "icu")]
        {
            let opt_ptr = opts.map(|o| o.0).unwrap_or(ptr::null_mut());
            re2_new_with_options(
                cpat.as_ptr() as *const i8,
                cpat.len(),
                opt_ptr,
                &mut err_ptr,
                &mut err_len,
            )
        }
        #[cfg(not(feature = "icu"))]
        {
            re2_new(
                cpat.as_ptr() as *const i8,
                cpat.len(),
                &mut err_ptr,
                &mut err_len,
            )
        }
    };

    if raw.is_null() || err_len != 0 || unsafe { re2_ok(raw) } != 1 {
        let msg = if !err_ptr.is_null() && err_len > 0 {
            unsafe { str::from_utf8_unchecked(slice::from_raw_parts(err_ptr as *const u8, err_len)) }
                .to_string()
        } else {
            "RE2 compile error".to_string()
        };
        if !raw.is_null() {
            unsafe { re2_delete(raw) }
        }
        return Err(msg);
    }
    Ok(raw)
}

pub fn delete_regex(raw: RE2WrapperHandle) {
    unsafe { re2_delete(raw) }
}

pub fn full_match(raw: RE2WrapperHandle, text: &str) -> bool {
    unsafe { re2_full_match(raw, text.as_ptr() as *const c_char, text.len()) == 1 }
}

pub fn partial_match(raw: RE2WrapperHandle, text: &str) -> bool {
    unsafe { re2_partial_match(raw, text.as_ptr() as *const c_char, text.len()) == 1 }
}

pub fn captures(raw: RE2WrapperHandle, text: &str, full: bool) -> Option<Vec<Option<&str>>> {
    let cap_count = 1 + group_count(raw);
    let mut spans = vec![re2_span_t { start: usize::MAX, len: 0 }; cap_count];
    let mut written: usize = 0;
    let ok = unsafe {
        if full {
            re2_full_match_captures(
                raw,
                text.as_ptr() as *const c_char,
                text.len(),
                spans.as_mut_ptr(),
                spans.len(),
                &mut written,
            )
        } else {
            re2_partial_match_captures(
                raw,
                text.as_ptr() as *const c_char,
                text.len(),
                spans.as_mut_ptr(),
                spans.len(),
                &mut written,
            )
        }
    } == 1;
    if !ok { return None; }
    let written = written.min(spans.len());
    let mut out = Vec::with_capacity(written);
    for i in 0..written {
        let s = spans[i];
        if s.start == usize::MAX {
            out.push(None);
        } else {
            out.push(Some(&text[s.start..s.start + s.len]));
        }
    }
    Some(out)
}

pub fn group_count(raw: RE2WrapperHandle) -> usize {
    unsafe { re2_group_count(raw) as usize }
}

pub fn replace(raw: RE2WrapperHandle, text: &str, rewrite: &str, one: bool) -> Option<String> {
    const MAX: usize = 1 << 20;
    let mut buf = vec![0u8; MAX];
    let mut written: usize = 0;
    let ok = unsafe {
        if one {
            re2_replace_one(raw, text.as_ptr() as *const i8, text.len(),
                            rewrite.as_ptr() as *const i8, rewrite.len(),
                            buf.as_mut_ptr() as *mut i8, buf.len(), &mut written)
        } else {
            re2_replace_all(raw, text.as_ptr() as *const i8, text.len(),
                            rewrite.as_ptr() as *const i8, rewrite.len(),
                            buf.as_mut_ptr() as *mut i8, buf.len(), &mut written)
        }
    };
    if ok == 1 {
        Some(String::from_utf8_lossy(&buf[..written]).into_owned())
    } else {
        None
    }
}

pub fn has_icu() -> bool {
    unsafe { re2_has_icu() == 1 }
}
