use re2_rs_wrapper::{Regex, Options, has_icu};

#[test]
fn unicode_property_matching() {
    // Match any Greek letter
    let re = Regex::new(r"^\p{Greek}+$").unwrap();
    assert!(re.full_match("Ωμέγα"));
    assert!(!re.full_match("Omega")); // Latin letters should fail
}

#[test]
fn unicode_digit_matching() {
    // Match Unicode decimal digits, not just ASCII 0-9
    let re = Regex::new(r"^\d+$").unwrap();
    assert!(re.full_match("٣٤٥")); // Arabic-Indic digits
    assert!(re.full_match("123")); // ASCII digits
}

#[test]
fn emoji_sequence_match() {
    // Emoji family sequence 👩‍👩‍👧‍👦
    let re = Regex::new(r"👩‍👩‍👧‍👦").unwrap();
    assert!(re.partial_match("hello 👩‍👩‍👧‍👦 world"));
    assert!(!re.full_match("👩 👩 👧 👦")); // must be grapheme cluster sequence
}

#[test]
fn emoji_property_match() {
    assert!(has_icu(), "Failing emoji_property_match: ICU not enabled");

    let opts = Options::new().perl_classes(true);
    let re = Regex::with_options(r"^\p{Emoji}+$", &opts).unwrap();

    assert!(re.full_match("😀"));         // Single emoji
    assert!(re.full_match("😀👍🚀"));      // Multiple emojis
    assert!(!re.full_match("Hello 😀"));  // Mixed text
}

#[test]
fn emoji_in_text() {
    let opts = Options::new().perl_classes(true);
    let re = Regex::with_options(r"\p{Emoji}", &opts).unwrap();

   // let re = Regex::new(r"\p{Emoji}+").unwrap();
    let text = "Let's go 🚀 to the stars!";
    let caps = re.partial_captures(text).unwrap();

    assert_eq!(caps[0], Some("🚀"));
}

#[test]
fn capture_groups_with_unicode() {
    let re = Regex::new(r"^(\p{Han}+)-(\p{Hiragana}+)$").unwrap();
    let caps = re.full_captures("漢字-ひらがな").unwrap();
    assert_eq!(caps[1].unwrap(), "漢字");
    assert_eq!(caps[2].unwrap(), "ひらがな");
}

#[test]
fn unicode_word_boundaries() {
    let opts = Options::new().unicode_word_boundaries(true);
    // Word boundary should work across scripts
    let re = Regex::with_options(r"\bκόσμος\b", &opts).unwrap();
    assert!(re.partial_match("γειά σου κόσμος!"));
    assert!(!re.partial_match("γειάσουκόσμος")); // no boundary
}

#[test]
fn ascii_case_insensitive() {
    let opts = Options::new().case_insensitive(true);
    let re = Regex::with_options(r"^hello$", &opts).unwrap();

    assert!(re.full_match("hello"));
    assert!(re.full_match("HELLO"));
    assert!(re.full_match("HeLlO"));

    // Only ASCII letters are folded; ß won't fold to SS here
    let opts = Options::new().case_insensitive(true);
    let re = Regex::with_options(r"^straße$", &opts).unwrap();

    assert!(re.full_match("straße"));
    assert_eq!(re.full_match("STRASSE"), false); // no ICU folding
}


#[test]
fn greek_case_insensitive() {
    let opts = Options::new().case_insensitive(true);
    let re = Regex::with_options(r"^κόσμος$", &opts).unwrap();

    assert!(re.full_match("κόσμος"));
    assert!(re.full_match("ΚΌΣΜΟΣ")); // uppercase Greek, should match with ICU
}

#[test]
fn bulgarian_cyrillic_script_property() {
    // Match only Cyrillic letters (Bulgarian: "Здравей")
    let re = Regex::new(r"^\p{Cyrillic}+$").unwrap();

    assert!(re.full_match("Здравей")); // Bulgarian "Hello"
    assert!(!re.full_match("Hello"));  // Latin should fail
}

#[test]
fn bulgarian_case_insensitive() {
    let opts = Options::new().case_insensitive(true);
    let re = Regex::with_options(r"^София$", &opts).unwrap();

    assert!(re.full_match("София")); // Sofia
    assert!(re.full_match("СОФИЯ")); // uppercase
    assert!(re.full_match("сОфИя")); // mixed case
}