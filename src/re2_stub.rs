// Minimal stub used when RE2 cannot be built (docs.rs, etc.)
#[allow(non_camel_case_types)]
pub enum RE2Wrapper {}

#[no_mangle]
pub extern "C" fn re2_new(_pattern: *const ::std::os::raw::c_char) -> *mut RE2Wrapper {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn re2_delete(_re: *mut RE2Wrapper) {}

#[no_mangle]
pub extern "C" fn re2_ok(_re: *const RE2Wrapper) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn re2_error(_re: *const RE2Wrapper) -> *const ::std::os::raw::c_char {
    std::ptr::null()
}

#[no_mangle]
pub extern "C" fn re2_full_match(_re: *const RE2Wrapper, _text: *const ::std::os::raw::c_char) -> bool {
    false
}
