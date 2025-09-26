use crate::wrapper::{self, CapturesIter, FindIter, RE2WrapperHandle};

/// Safe Rust wrapper around RE2
///
/// # Examples
///
/// Basic usage without ICU:
/// ```
/// use re2_rs::Regex;
///
/// let re = Regex::new(r"\d+").unwrap();
/// assert!(re.full_match("123"));
/// assert!(!re.full_match("abc"));
/// ```
///
/// ICU-specific examples (only compiled if `icu` feature is enabled):
/// ```ignore
/// #[cfg(feature = "icu")]
/// use re2_rs::Regex;
///
/// # #[cfg(feature = "icu")]
/// {
///     let re = Regex::new(r"\p{Emoji}").unwrap();
///     assert!(re.partial_match("😀"));
/// }
/// ```
pub struct Regex {
    raw: RE2WrapperHandle,
}

impl Regex {
    /// Compile a new regex pattern.
    ///
    /// # Examples
    /// ```
    /// use re2_rs::Regex;
    /// let re = Regex::new(r"\d+").unwrap();
    /// assert!(re.full_match("123"));
    /// ```
    pub fn new(pattern: &str) -> Result<Self, String> {
        wrapper::compile_regex(pattern, None).map(|raw| Self { raw })
    }

    /// Compile a new regex with options.
    ///
    /// # Examples
    /// ```
    /// use re2_rs::{Regex, wrapper::Options};
    /// let mut opts = Options::default();
    /// opts.set_case_sensitive(false);
    /// let re = Regex::with_options("abc", &opts).unwrap();
    /// assert!(re.full_match("ABC"));
    /// ```
    pub fn with_options(pattern: &str, opts: &wrapper::Options) -> Result<Self, String> {
        wrapper::compile_regex(pattern, Some(opts)).map(|raw| Self { raw })
    }

    /// Check if the regex fully matches the input.
    ///
    /// # Examples
    /// ```
    /// use re2_rs::Regex;
    /// let re = Regex::new(r"\d+").unwrap();
    /// assert!(re.full_match("123"));
    /// assert!(!re.full_match("12a"));
    /// ```
    pub fn full_match(&self, text: &str) -> bool {
        wrapper::full_match(self.raw, text)
    }

    /// Check if the regex partially matches the input.
    ///
    /// # Examples
    /// ```
    /// use re2_rs::Regex;
    /// let re = Regex::new(r"\d+").unwrap();
    /// assert!(re.partial_match("xx123yy"));
    /// assert!(!re.partial_match("no digits"));
    /// ```
    ///
    /// ICU-specific:
    /// ```
    /// # #[cfg(feature = "icu")]
    /// use re2_rs::Regex;
    /// # #[cfg(feature = "icu")]
    /// {
    ///     let re = Regex::new(r"\p{Emoji}").unwrap();
    ///     assert!(re.partial_match("😀"));
    /// }
    /// ```
    pub fn partial_match(&self, text: &str) -> bool {
        wrapper::partial_match(self.raw, text)
    }

    /// Get partial captures from the input.
    ///
    /// # Examples
    /// ```
    /// use re2_rs::Regex;
    /// let re = Regex::new(r"(foo)(bar)?").unwrap();
    /// let caps = re.partial_captures("xxfoo");
    /// assert_eq!(caps, Some(vec![Some("foo"), None]));
    /// ```
    pub fn partial_captures<'t>(&self, text: &'t str) -> Option<Vec<Option<&'t str>>> {
        wrapper::captures(self.raw, text, false)
    }

    /// Get full captures from the input.
    ///
    /// # Examples
    /// ```
    /// use re2_rs::Regex;
    /// let re = Regex::new(r"(foo)(bar)?").unwrap();
    /// let caps = re.full_captures("foobar");
    /// assert_eq!(caps, Some(vec![Some("foo"), Some("bar")]));
    /// ```
    pub fn full_captures<'t>(&self, text: &'t str) -> Option<Vec<Option<&'t str>>> {
        wrapper::captures(self.raw, text, true)
    }

    /// Return the number of capturing groups.
    ///
    /// # Examples
    /// ```
    /// use re2_rs::Regex;
    /// let re = Regex::new(r"(foo)(bar)?").unwrap();
    /// assert_eq!(re.num_captures(), 2);
    /// ```
    pub fn num_captures(&self) -> usize {
        wrapper::group_count(self.raw)
    }

    /// Replace the first match in the input string.
    ///
    /// # Examples
    /// ```
    /// use re2_rs::Regex;
    /// let re = Regex::new(r"\d+").unwrap();
    /// assert_eq!(re.replace_one("a1b2c3", "X"), Some("aXb2c3".to_string()));
    /// ```
    pub fn replace_one(&self, text: &str, rewrite: &str) -> Option<String> {
        wrapper::replace(self.raw, text, rewrite, true)
    }

    /// Replace all matches in the input string.
    ///
    /// # Examples
    /// ```
    /// use re2_rs::Regex;
    /// let re = Regex::new(r"\d+").unwrap();
    /// assert_eq!(re.replace_all("a1b2c3", "X"), Some("aXbXcX".to_string()));
    /// ```
    pub fn replace_all(&self, text: &str, rewrite: &str) -> Option<String> {
        wrapper::replace(self.raw, text, rewrite, false)
    }

    /// Iterate over all matches in the input string.
    ///
    /// # Examples
    /// ```
    /// use re2_rs::Regex;
    /// let re = Regex::new(r"\d+").unwrap();
    /// let matches: Vec<_> = re.find_iter("a1b22c333").collect();
    /// assert_eq!(matches, vec!["1", "22", "333"]);
    /// ```
    pub fn find_iter<'t>(&self, text: &'t str) -> FindIter<'t> {
        wrapper::find_iter(self.raw, text)
    }

    /// Iterate over all captures in the input string.
    ///
    /// # Examples
    /// ```
    /// use re2_rs::Regex;
    /// let re = Regex::new(r"(a)(\\d+)").unwrap();
    /// let caps: Vec<_> = re.captures_iter("a1a22").collect();
    /// assert_eq!(
    ///     caps,
    ///     vec![
    ///         vec![Some("a"), Some("1")],
    ///         vec![Some("a"), Some("22")]
    ///     ]
    /// );
    /// ```
    pub fn captures_iter<'t>(&self, text: &'t str) -> CapturesIter<'t> {
        wrapper::captures_iter(self.raw, text)
    }
}

