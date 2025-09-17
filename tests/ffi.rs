use std::ffi::CString;
use re2_sys::*;

//TODO (e.g., partial match, captures),
#[test]
fn simple_match() {
    unsafe {
        let pat = CString::new("hello.*world").unwrap();
        let txt = CString::new("hello brave new world").unwrap();

        let re = re2_new(pat.as_ptr());
        assert!(!re.is_null(), "regex creation failed");

        assert!(re2_ok(re));
        assert!(re2_full_match(re, txt.as_ptr()));

        re2_delete(re);
    }
}

#[test]
fn case_insensitive_with_anchors() {
    unsafe {
        let pat = CString::new(r"(?i)^hello.*world$").unwrap();
        let txt = CString::new("HeLLo brave new WORLD").unwrap();

        let re = re2_new(pat.as_ptr());
        assert!(!re.is_null());
        assert!(re2_ok(re));
        assert!(re2_full_match(re, txt.as_ptr()));
        re2_delete(re);
    }
}

#[test]
fn unicode_property_greek_letters() {
    unsafe {
        // Match one or more Greek letters (RE2 supports Unicode properties)
        let pat = CString::new(r"^\p{Greek}+$").unwrap();
        let txt = CString::new("αλφα").unwrap(); // "alpha" in Greek

        let re = re2_new(pat.as_ptr());
        assert!(!re.is_null());
        assert!(re2_ok(re));
        assert!(re2_full_match(re, txt.as_ptr()));
        re2_delete(re);
    }
}

#[test]
fn dotall_multiline_like() {
    unsafe {
        // (?s) makes '.' match newlines; entire string must match
        let pat = CString::new(r"(?s)^BEGIN:.*END:$").unwrap();
        let txt = CString::new("BEGIN:\nline1\nline2\nEND:").unwrap();

        let re = re2_new(pat.as_ptr());
        assert!(!re.is_null());
        assert!(re2_ok(re));
        assert!(re2_full_match(re, txt.as_ptr()));
        re2_delete(re);
    }
}

#[test]
fn alternation_and_counts() {
    unsafe {
        // Either "cat" or "dog" followed by 2–4 exclamation marks
        let pat = CString::new(r"^(cat|dog)!{2,4}$").unwrap();
        let good = CString::new("dog!!!").unwrap();
        let bad  = CString::new("cat!").unwrap();

        let re = re2_new(pat.as_ptr());
        assert!(!re.is_null());
        assert!(re2_ok(re));
        assert!(re2_full_match(re, good.as_ptr()));
        assert!(!re2_full_match(re, bad.as_ptr()));
        re2_delete(re);
    }
}

#[test]
fn word_boundary_in_full_match() {
    unsafe {
        // Use ^.* … .*$/\b to let full_match succeed while checking word boundaries
        let pat = CString::new(r"^.*\bfoo\b.*$").unwrap();
        let yes = CString::new("xx foo yy").unwrap();
        let no  = CString::new("xx foobar yy").unwrap();

        let re = re2_new(pat.as_ptr());
        assert!(!re.is_null());
        assert!(re2_ok(re));
        assert!(re2_full_match(re, yes.as_ptr()));
        assert!(!re2_full_match(re, no.as_ptr()));
        re2_delete(re);
    }
}

#[test]
fn complex_passwordish() {
    use std::ffi::CString;
    unsafe {
        // 1) length ≥ 6  (no newlines)
        let pat_len = CString::new(r"^.{6,}$").unwrap();
        // 2) contains at least one letter and one digit (either order)
        let pat_mix = CString::new(r"^(?:.*[A-Za-z].*\d.*|.*\d.*[A-Za-z].*)$").unwrap();

        let ok    = CString::new("a1b2c3").unwrap();
        let no_len = CString::new("a1b2c").unwrap();   // too short
        let no_mix = CString::new("abcdef").unwrap();  // no digit

        let re_len = re2_new(pat_len.as_ptr());
        let re_mix = re2_new(pat_mix.as_ptr());
        assert!(re2_ok(re_len) && re2_ok(re_mix));

        // both conditions must hold
        assert!(re2_full_match(re_len, ok.as_ptr()));
        assert!(re2_full_match(re_mix, ok.as_ptr()));

        // fails length
        assert!(!re2_full_match(re_len, no_len.as_ptr()));
        // fails composition
        assert!(!re2_full_match(re_mix, no_mix.as_ptr()));

        re2_delete(re_len);
        re2_delete(re_mix);
    }
}

#[test]
fn unsupported_lookahead_fails_to_compile() {
    use std::ffi::CString;
    unsafe {
        let pat = CString::new(r"^(?=.{6,}$).*$").unwrap();
        let re = re2_new(pat.as_ptr());
        assert!(!re2_ok(re), "RE2 should reject lookahead");
        assert!(!re2_error(re).is_null());
        re2_delete(re);
    }
}

#[test]
fn unsupported_backreference_fails_to_compile() {
    unsafe {
        // RE2 does NOT support backreferences like \1
        let pat = CString::new(r"^(a)\1$").unwrap();
        let re = re2_new(pat.as_ptr());
        assert!(!re.is_null());
        assert!(!re2_ok(re));

        let err = re2_error(re);
        assert!(!err.is_null(), "expected an error message");
        re2_delete(re);
    }
}

#[test]
fn unsupported_lookbehind_fails_to_compile() {
    unsafe {
        // RE2 does NOT support lookbehind
        let pat = CString::new(r"(?<=foo)bar").unwrap();
        let re = re2_new(pat.as_ptr());
        assert!(!re.is_null());
        assert!(!re2_ok(re));

        let err = re2_error(re);
        assert!(!err.is_null(), "expected an error message");
        re2_delete(re);
    }
}