use re2_rs_wrapper::{Regex, Options};

#[test]
fn options_posix_longest() {

    let opts = Options::new().posix_syntax(true).longest_match(true);
    let re = Regex::with_options(r"(ab|a)b", &opts).unwrap();
    // POSIX longest: prefers the longest-leftmost alternative
    assert!(re.full_match("ab"));
}

#[test]
fn posix_syntax_behavior() {
    let opts = Options::new().posix_syntax(true);

    // In POSIX mode, `\d` is not recognized — only `[0-9]`
    let re1 = Regex::with_options(r"\d+", &opts);
    assert!(re1.is_err());

    let re2 = Regex::with_options(r"[0-9]+", &opts).unwrap();
    assert!(re2.full_match("12345"));
}

#[test]
fn emoji_case_insensitive_no_effect() {
    // Emojis don’t have case — so case_insensitive should not change behavior
    let opts = Options::new().case_insensitive(true);
    let re = Regex::with_options("\\p{Emoji}", &opts).unwrap();

    assert!(re.full_match("😀"));
    assert!(re.full_match("😀👍🚀"));
    assert!(!re.full_match("Hello 😀")); // still fails
}