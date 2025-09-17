use crate::ffi::{RE2Wrapper, re2_new, re2_delete, re2_ok, re2_error,
                 re2_full_match, re2_partial_match,
                 re2_partial_match_captures, re2_full_match_captures,
                 re2_span_t, re2_group_count};
use std::ffi::{c_char, CString};

pub struct Regex {
    raw: *mut RE2Wrapper,
}

impl Regex {
    pub fn new(pattern: &str) -> Result<Self, String> {
        let cpat = CString::new(pattern).map_err(|_| "pattern contains NUL byte".to_string())?;
        let mut err_ptr: *const c_char = std::ptr::null();
        let mut err_len: usize = 0;
        let raw = unsafe { re2_new(cpat.as_ptr(), cpat.as_bytes().len(), &mut err_ptr, &mut err_len) };
        if raw.is_null() {
            let msg = if !err_ptr.is_null() && err_len > 0 {
                unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(err_ptr as *const u8, err_len)) }.to_string()
            } else {
                "re2_new returned null".to_string()
            };
            return Err(msg);
        }
        let ok = unsafe { re2_ok(raw) } == 1;
        if !ok {
            let mut e_ptr: *const c_char = std::ptr::null();
            let mut e_len: usize = 0;
            unsafe { re2_error(raw, &mut e_ptr, &mut e_len) };
            let msg = if !e_ptr.is_null() && e_len > 0 {
                unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(e_ptr as *const u8, e_len)) }.to_string()
            } else { "unknown RE2 error".into() };
            unsafe { re2_delete(raw) };
            return Err(msg);
        }
        Ok(Self { raw })
    }

    pub fn full_match(&self, text: &str) -> bool {
        unsafe { re2_full_match(self.raw, text.as_ptr() as *const c_char, text.len()) == 1 }
    }

    pub fn partial_match(&self, text: &str) -> bool {
        unsafe { re2_partial_match(self.raw, text.as_ptr() as *const c_char, text.len()) == 1 }
    }

    /// Returns (whole_match, captures...), each as Option<&str>.
    pub fn partial_captures<'t>(&self, text: &'t str) -> Option<Vec<Option<&'t str>>> {
        let cap_count = 1 + self.num_captures(); // including group 0
        let mut spans = vec![re2_span_t { start: usize::MAX, len: 0 }; cap_count];
        let mut written: usize = 0;
        let ok = unsafe {
            re2_partial_match_captures(
                self.raw,
                text.as_ptr() as *const c_char, text.len(),
                spans.as_mut_ptr(), spans.len(),
                &mut written
            ) == 1
        };
        if !ok { return None; }
        let written = written.min(spans.len());
        let mut out = Vec::with_capacity(written);
        for i in 0..written {
            let s = spans[i];
            if s.start == usize::MAX {
                out.push(None);
            } else {
                let start = s.start;
                let end = start + s.len;
                // Safety: RE2 returns UTF-8 ranges; they are byte offsets into `text`.
                out.push(Some(&text[start..end]));
            }
        }
        Some(out)
    }

    pub fn full_captures<'t>(&self, text: &'t str) -> Option<Vec<Option<&'t str>>> {
        let cap_count = 1 + self.num_captures();
        let mut spans = vec![re2_span_t { start: usize::MAX, len: 0 }; cap_count];
        let mut written: usize = 0;
        let ok = unsafe {
            re2_full_match_captures(
                self.raw,
                text.as_ptr() as *const c_char, text.len(),
                spans.as_mut_ptr(), spans.len(),
                &mut written
            ) == 1
        };
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

    /// Cache this if you like; this calls into RE2 metadata cheaply.
    pub fn num_captures(&self) -> usize {
        // For bindgen path you can expose NumberOfCapturingGroups via a tiny C shim
        // For now, derive from a trial match: allocate large, read `written-1`.
        // Better: add re2_group_count(re2) in C++ shim; included here:
        unsafe { re2_group_count(self.raw) as usize }
    }
}

impl Drop for Regex {
    fn drop(&mut self) {
        unsafe { re2_delete(self.raw) }
    }
}