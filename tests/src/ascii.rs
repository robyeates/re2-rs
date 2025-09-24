use re2_rs_wrapper::{Regex};

#[test]
fn ascii_digit_matching_only() {
    let re = Regex::new(r"^\d+$").unwrap();
    assert!(re.full_match("123"));
    assert!(!re.full_match("٣٤٥")); // Arabic-Indic should fail
}

#[test]
fn ascii_word_boundaries() {
    // ASCII \b works on [A-Za-z0-9_], but not on Greek
    let re = Regex::new(r"\bword\b").unwrap();
    assert!(re.partial_match("some word here"));
    assert!(!re.partial_match("somewordhere"));

    // Greek should fail: RE2 without ICU doesn't treat it as a word
    let re_greek = Regex::new(r"\bκόσμος\b").unwrap();
    assert!(!re_greek.partial_match("γειά σου κόσμος!"));
}

#[test]
fn no_unicode_digit_support() {
    let re = Regex::new(r"^\d+$").unwrap();
    assert!(re.full_match("123"));    // ASCII works
    assert!(!re.full_match("٣٤٥"));  // Arabic-Indic digits should fail
}

#[test]
fn no_unicode_word_boundaries() {
    let re = Regex::new(r"\bκόσμος\b").unwrap();
    assert!(!re.partial_match("γειά σου κόσμος!")); // would work with ICU
}

#[test]
fn no_unicode_case_folding() {
    let re = Regex::new(r"^straße$").unwrap();
    assert!(re.full_match("straße"));
    assert!(!re.full_match("STRASSE")); // ß→SS folding missing
}

