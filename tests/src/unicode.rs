use re2_rs_wrapper::{Regex, Options, has_icu};

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
fn bulgarian_case_insensitive() {
    let opts = Options::new().case_insensitive(true);
    let re = Regex::with_options(r"^София$", &opts).unwrap();

    assert!(re.full_match("София")); // Sofia
    assert!(re.full_match("СОФИЯ")); // uppercase
    assert!(re.full_match("сОфИя")); // mixed case
}

#[test]
fn greek_case_insensitive() {
    let opts = Options::new().case_insensitive(true);
    let re = Regex::with_options(r"^κόσμος$", &opts).unwrap();

    assert!(re.full_match("κόσμος"));
    assert!(re.full_match("ΚΌΣΜΟΣ")); // uppercase Greek, should match with ICU
}

#[test]
fn ascii_case_insensitive() {
    let opts = Options::new().case_insensitive(true).unicode_word_boundaries(true);
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
fn unicode_full_captures_with_icu() {
    // Match: one or more letters = one or more digits
    // Using Unicode properties \p{L} (letters), \p{Nd} (decimal digits)
    let re = Regex::new(r"(\p{L}+)=([\p{Nd}]+)").unwrap();

    // Greek letters with Arabic-Indic digits
    let text = "κόσμος=٣٤٥";

    let captures = re.full_captures(text).unwrap();
    assert_eq!(re.num_captures(), 2);

    // The first element is always the full match
    assert_eq!(captures[0], Some("κόσμος=٣٤٥"));
    // Group 1 = Greek letters
    assert_eq!(captures[1], Some("κόσμος"));
    // Group 2 = Arabic-Indic digits
    assert_eq!(captures[2], Some("٣٤٥"));
}
#[test]
fn unicode_digit_matching() {
    // Match Unicode decimal digits, not just ASCII 0-9
    let re = Regex::new(r"^\p{Nd}+$").unwrap();
    assert!(re.full_match("٣٤٥")); // Arabic-Indic digits
    assert!(re.full_match("123")); // ASCII digits
}
#[test]
fn replace_emoji_with_placeholder() {
    // Match base emoji + optional skin-tone modifier
    let re = Regex::new(r"\p{Emoji_Modifier_Base}\p{Emoji_Modifier}?").unwrap();
    let text = "👍🏽 😀";

    let replaced = re.replace_all(text, "[EMOJI]").unwrap();
    assert_eq!(replaced, "[EMOJI] 😀"); // thumbs-up replaced, smiley left alone
}

#[test]
fn case_fold_and_replace() {
    let re = Regex::with_options(r"κόσμος", &re2_rs_icu::Options::new().case_insensitive(true)).unwrap();

    let text = "ΚΌΣΜΟΣ";
    let replaced = re.replace_all(text, "world").unwrap();
    assert_eq!(replaced, "world");
}

#[ignore]
#[test]
fn shortest_vs_longest_match() {
    let opts_short = Options::new().longest_match(false);
    let re_short = Regex::with_options(r".+ς", &opts_short).unwrap();

    let opts_long = Options::new().longest_match(true);
    let re_long  = Regex::with_options(r".+ς", &opts_long).unwrap();

    let text = "ςςςςςςςςςςςςςςςς";

    let caps_short = re_short.full_captures(text).unwrap();
    let caps_long = re_long.full_captures(text).unwrap();

    assert_eq!(caps_short[0], Some("ς"));            // shortest
    assert_eq!(caps_long[0], Some("ςςςςςςςςςςςςςςςς"));      // longest
}

///should work. Need to check options are being respected
#[test]
#[ignore]
fn shortest_vs_longest_match_latin() {
    let opts_short = Options::new().longest_match(false);
    let opts_long = Options::new().longest_match(true);

    let text = "aaaa";

    let re_short = Regex::with_options(r"(a+?)(a+)", &opts_short).unwrap();
    let re_long  = Regex::with_options(r"(a+?)(a+)", &opts_long).unwrap();

    let caps_short = re_short.partial_captures(text).unwrap();
    let caps_long  = re_long.partial_captures(text).unwrap();

    assert_eq!(caps_short[0], Some("a"));      // shortest match
    assert_eq!(caps_long[0], Some("aaaa"));    // longest match
}

#[test]
fn multiline_anchors() {
    let re = Regex::new(r"(?m)^κόσμος$").unwrap();
    let text = "γειά\nκόσμος\nκαληνύχτα";

    assert!(re.partial_match(text));
}

#[test]
fn dotall_mode() {
    let re = Regex::new(r"(?s)κόσμος.*καληνύχτα").unwrap();
    let text = "κόσμος\n…\nκαληνύχτα";

    assert!(re.full_match(text));
}

///(?m) is limited to ASCII newlines in RE2. ICU doesn't change this
#[test]
fn unicode_newline_multiline() {
    let re = Regex::new(r"(?m)^κόσμος$").unwrap();

    // Unicode line separator U+2028
    let text = "γειά\u{2028}κόσμος\u{2028}καληνύχτα";

    assert!(!re.partial_match(text));

    let text = text.replace('\u{2028}', "\n");

    assert!(re.partial_match(&*text));
}

#[ignore]
#[test]
fn ascii_vs_unicode_word_boundaries() {
    let re = Regex::new(r"\b123\b").unwrap();
    // ASCII \b sees \u{0661}\u{0662}\u{0663} (Arabic-Indic digits) as non-\w
    assert!(!re.partial_match("١٢٣")); // U+0661 U+0662 U+0663

    let opts = Options::new().unicode_word_boundaries(true);
    let re_unicode = Regex::with_options(r"\b123\b", &opts).unwrap();

    // With ICU, digits are word characters, so this can now match.
    assert!(re_unicode.partial_match("١٢٣"));
}