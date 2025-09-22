use std::ffi::CString;
use re2_rs_sys::{re2_delete, re2_full_match, re2_new, re2_ok};

#[test]
fn ffi_simple_match() {
    unsafe {
        let pat = CString::new("hello.*world").unwrap();
        let txt = CString::new("hello brave new world").unwrap();

        let re = re2_new(pat.as_ptr(), pat.as_bytes().len(), std::ptr::null_mut(), std::ptr::null_mut());
        assert!(!re.is_null());
        assert_eq!(re2_ok(re), 1);
        assert_eq!(re2_full_match(re, txt.as_ptr(), txt.as_bytes().len()), 1);
        re2_delete(re);
    }
}

#[test]
fn ffi_unsupported_backreference() {
    unsafe {
        let pat = CString::new(r"^(a)\1$").unwrap();
        let re = re2_new(pat.as_ptr(), pat.as_bytes().len(), std::ptr::null_mut(), std::ptr::null_mut());
        assert!(!re.is_null());
        assert_eq!(re2_ok(re), 0);
        re2_delete(re);
    }
}

#[test]
fn ffi_unsupported_lookbehind() {
    unsafe {
        let pat = CString::new(r"(?<=foo)bar").unwrap();
        let re = re2_new(pat.as_ptr(), pat.as_bytes().len(), std::ptr::null_mut(), std::ptr::null_mut());
        assert!(!re.is_null());
        assert_eq!(re2_ok(re), 0);
        re2_delete(re);
    }
}