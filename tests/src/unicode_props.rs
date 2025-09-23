use re2_rs_wrapper::Regex as BasicRegex;
use re2_rs_wrapper::Regex as IcuRegex;
// These tests are [FIXME SUPPOSED TO BE /FIXME]meant as documentation: how to use Unicode properties with RE2+ICU.
// Run them against both `re2-rs` (ASCII/limited Unicode) and `re2-rs-icu` (full ICU).
//
// RE2 alone supports only a limited set of \p{} properties (e.g. Script=Greek).
// With ICU enabled, RE2 delegates to ICU's UnicodeSet parser, unlocking the full
// set of Unicode properties (see https://unicode-org.github.io/icu/userguide/strings/regexp.html).
#[test]
fn emoji_modifier_base() {
    let icu = IcuRegex::new(r"^\p{Emoji_Modifier_Base}$").unwrap();

    assert!(icu.full_match("👍")); // thumbs up can take skin-tone
    assert!(!icu.full_match("😀")); // plain emoji, no modifier support

    // Basic RE2: does not know about Emoji_Modifier_Base
    //let basic = BasicRegex::new(r"^\p{Emoji_Modifier_Base}$").unwrap();
    //assert!(!basic.full_match("👍"));
}

#[test]
fn alphabetic_property() {
    let icu = IcuRegex::new(r"^\p{Alphabetic}+$").unwrap();
    assert!(icu.full_match("κόσμος")); // Greek
    assert!(icu.full_match("hello"));  // ASCII


    // Basic RE2 only matches ASCII letters with \p{Alpha}
    //let basic = BasicRegex::new(r"^\p{Alpha}+$").unwrap();
    //assert!(basic.full_match("hello"));
    //assert!(!basic.full_match("κόσμος"));
}

#[test]
fn lowercase_and_uppercase() {
    let icu_lower = IcuRegex::new(r"^\p{Lowercase}+$").unwrap();
    let icu_upper = IcuRegex::new(r"^\p{Uppercase}+$").unwrap();

    assert!(icu_lower.full_match("κόσμος")); // Greek lowercase
    assert!(icu_upper.full_match("ΚΌΣΜΟΣ")); // Greek uppercase

    // Basic RE2: only knows ASCII a–z / A–Z
    //let basic_lower = BasicRegex::new(r"^\p{Lower}$").unwrap();
    //let basic_upper = BasicRegex::new(r"^\p{Upper}$").unwrap();

    //assert!(basic_lower.full_match("hello"));
    //assert!(!basic_lower.full_match("κόσμος"));

    //assert!(basic_upper.full_match("WORLD"));
    //assert!(!basic_upper.full_match("ΚΌΣΜΟΣ"));
}

#[test]
fn math_symbols() {
    let icu = IcuRegex::new(r"^\p{Math}+$").unwrap();
    assert!(icu.full_match("∑∞≈"));

    // Basic RE2: no Math property
    //let basic = BasicRegex::new(r"^\p{Math}+$").unwrap();
    //assert!(!basic.full_match("∑∞≈"));
}

#[test]
fn whitespace_property() {
    let icu = IcuRegex::new(r"^\p{White_Space}+$").unwrap();
    assert!(icu.full_match(" \t\n"));
    assert!(icu.full_match("\u{2003}")); // em space

    // Basic RE2: \s = [ \f\n\r\t\v]
    let basic = BasicRegex::new(r"^\s+$").unwrap();
    assert!(basic.full_match(" \t\n"));
    assert!(!basic.full_match("\u{2003}"));
}

#[test]
fn dash_characters() {
    let icu = IcuRegex::new(r"^\p{Dash}+$").unwrap();
    assert!(icu.full_match("–—")); // en-dash, em-dash
    assert!(icu.full_match("-"));  // ASCII hyphen

    // Basic RE2: "-" is just a literal
    let basic = BasicRegex::new(r"^-+$").unwrap();
    assert!(basic.full_match("-"));
    assert!(!basic.full_match("–")); // en-dash fails
}

#[test]
fn quotation_marks() {
    let icu = IcuRegex::new(r"^\p{Quotation_Mark}+$").unwrap();
    assert!(icu.full_match("\"«»“”")); // ASCII quote + guillemets + curly quotes

    // Basic RE2: only sees ASCII "
    let basic = BasicRegex::new(r#"^"+$"#).unwrap();
    assert!(basic.full_match("\""));
    assert!(!basic.full_match("“”"));
}