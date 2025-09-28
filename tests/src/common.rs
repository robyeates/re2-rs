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

#[test]
fn unicode_property_matching() {
    // Match any Greek letter
    let re = Regex::new(r"^\p{Greek}+$").unwrap();
    assert!(re.full_match("Ωμέγα"));
    assert!(!re.full_match("Omega")); // Latin letters should fail
}

#[test]
fn emoji_sequence_match() {
    // Emoji family sequence 👩‍👩‍👧‍👦
    let re = Regex::new(r"👩‍👩‍👧‍👦").unwrap();
    assert!(re.partial_match("hello 👩‍👩‍👧‍👦 world"));
    assert!(!re.full_match("👩 👩 👧 👦")); // must be grapheme cluster sequence
}

#[test]
fn capture_groups_with_unicode() {
    let re = Regex::new(r"^(\p{Han}+)-(\p{Hiragana}+)$").unwrap();
    let caps = re.full_captures("漢字-ひらがな").unwrap();
    assert_eq!(caps[1].unwrap(), "漢字");
    assert_eq!(caps[2].unwrap(), "ひらがな");
}

#[test]
fn bulgarian_cyrillic_script_property() {
    // Match only Cyrillic letters (Bulgarian: "Здравей")
    let re = Regex::new(r"^\p{Cyrillic}+$").unwrap();

    assert!(re.full_match("Здравей")); // Bulgarian "Hello"
    assert!(!re.full_match("Hello"));  // Latin should fail
}

#[test]
fn full_captures_returns_all_groups() {
    let re = Regex::new(r"(\w+)=(\d+)").unwrap();
    let text = "foo=42";

    let captures = re.full_captures(text).unwrap();
    assert_eq!(re.num_captures(), 2);

    // The first element in the vector is the entire match
    assert_eq!(captures[0], Some("foo=42"));
    // Group 1
    assert_eq!(captures[1], Some("foo"));
    // Group 2
    assert_eq!(captures[2], Some("42"));
}

#[test]
fn full_captures_handles_optional_groups() {
    let re = Regex::new(r"(a)?(b)").unwrap();

    let caps1 = re.full_captures("b").unwrap();
    assert_eq!(caps1[0], Some("b"));      // whole match
    assert_eq!(caps1[1], None);           // (a)? not matched
    assert_eq!(caps1[2], Some("b"));      // (b) matched

    let caps2 = re.full_captures("ab").unwrap();
    assert_eq!(caps2[0], Some("ab"));
    assert_eq!(caps2[1], Some("a"));
    assert_eq!(caps2[2], Some("b"));
}

#[test]
fn full_captures_returns_none_if_no_match() {
    let re = Regex::new(r"(\w+)=(\d+)").unwrap();
    assert!(re.full_captures("not a pair").is_none());
}

#[test]
fn wrapper_replace_one_and_all() {
    let re = Regex::new(r"\d+").unwrap();

    assert_eq!(
        re.replace_one("a1b22c333", "X").unwrap(),
        "aXb22c333"
    );

    assert_eq!(
        re.replace_all("a1b22c333", "X").unwrap(),
        "aXbXcX"
    );
}

#[test]
fn wrapper_find_iter_collects_matches() {
    let re = Regex::new(r"\d+").unwrap();
    let matches: Vec<_> = re.find_iter("a1b22c333").collect();
    assert_eq!(matches, vec!["1", "22", "333"]);
}

#[test]
fn wrapper_captures_iter_collects_groups() {
    let re = Regex::new(r"(a)(\d+)").unwrap();
    let caps: Vec<_> = re.captures_iter("a1a22").collect();

    assert_eq!(
        caps,
        vec![
            vec![Some("a"), Some("1")],
            vec![Some("a"), Some("22")]
        ]
    );
}

#[test]
fn wrapper_num_captures_reports_correct_count() {
    let re = Regex::new(r"(foo)(bar)?").unwrap();
    assert_eq!(re.num_captures(), 2);
}
