use re2_rs_wrapper::{Options, Regex};

#[test]
fn wrapper_partial_match() {
    let re = Regex::new(r"(?s)hello\..*world").unwrap();
    assert!(re.partial_match("xxx hello.\nnew world yyy"));
    assert!(!re.partial_match("nope"));
}

#[test]
fn wrapper_captures_email() {
    let re = Regex::new(r"(\w+)@([A-Za-z0-9.-]+\.[A-Za-z]{2,})").unwrap();
    let text = "Contact: foo@example.com";
    let caps = re.partial_captures(text).unwrap();
    assert_eq!(caps[0], Some("foo@example.com"));
    assert_eq!(caps[1], Some("foo"));
    assert_eq!(caps[2], Some("example.com"));
}

#[test]
fn wrapper_full_match_with_date() {
    let re = Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
    assert!(re.full_captures("2024-12-31").is_some());
    assert!(re.full_captures("x 2024-12-31 y").is_none());
}

#[test]
fn wrapper_unicode_greek() {
    let re = Regex::new(r"(\p{Greek}+)").unwrap();
    let s = "ASCII και Ελληνικά Ωραία";
    let caps = re.partial_captures(s).unwrap();
    assert!(caps[0].unwrap().chars().all(|c| c >= '\u{0370}'));
}

#[test]
fn wrapper_optional_group_none() {
    let re = Regex::new(r"(foo)?bar").unwrap();
    let caps = re.partial_captures("bar").unwrap();
    assert_eq!(caps[0], Some("bar"));
    assert!(caps[1].is_none());
}

#[test]
fn wrapper_passwordish_without_lookahead() {
    // 1) length >= 6
    let re_len = Regex::new(r"^.{6,}$").unwrap();

    // 2) contains at least one digit and at least one ASCII letter
    // We use alternation: either digit before letter, or letter before digit.
    let re_mix = Regex::new(r"^(?:.*\d.*[A-Za-z].*|.*[A-Za-z].*\d.*)$").unwrap();

    // OK: has letters + digits, length >= 6
    assert!(re_len.full_match("a1b2c3"));
    assert!(re_mix.full_match("a1b2c3"));

    // Too short
    assert!(!re_len.full_match("a1b2c"));

    // No digit
    assert!(!re_mix.full_match("abcdef"));

    // No letter
    assert!(!re_mix.full_match("123456"));
}

#[test]
fn wrapper_lookahead_is_not_supported() {
    assert!(Regex::new(r"(?=.{8,})(?=.*\d)(?=.*[A-Z])[A-Za-z0-9]+").is_err());
    assert!(Regex::new(r"(foo)\1").is_err());       // backref
    assert!(Regex::new(r"(?<=foo)bar").is_err());   // lookbehind
}

//Unicode
#[test]
fn unicode_greek_letters_match() {
    // Matches one or more Greek letters
    let re = Regex::new(r"^\p{Greek}+$").unwrap();
    assert!(re.full_match("αλφα"));   // "alpha" in Greek
    assert!(re.full_match("Ωμέγα"));  // "omega" in Greek
    assert!(!re.full_match("abc"));   // Latin letters, not Greek
}

#[test]
fn unicode_any_letter_property() {
    // \p{L} means "any letter" (from any script)
    let re = Regex::new(r"^\p{L}+$").unwrap();
    assert!(re.full_match("abc"));
    assert!(re.full_match("αλφα"));   // Greek
    assert!(re.full_match("漢字"));     // Han characters
    assert!(!re.full_match("123"));   // digits
}

#[test]
fn unicode_mixed_word_with_digits() {
    // Ensure we can mix letters and digits
    let re = Regex::new(r"^(?:.*\d.*\p{L}.*|.*\p{L}.*\d.*)$").unwrap();
    assert!(re.full_match("abc123"));
    assert!(re.full_match("123漢字"));
    assert!(re.full_match("Ω123"));
    assert!(!re.full_match("123456")); // only digits
}
