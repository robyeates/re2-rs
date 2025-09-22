use crate::wrapper::{self, RE2WrapperHandle};

/// Safe Rust wrapper around RE2
pub struct Regex {
    raw: RE2WrapperHandle,
}

impl Regex {
    /// Always available (no ICU required)
    pub fn new(pattern: &str) -> Result<Self, String> {
        wrapper::compile_regex(pattern, None).map(|raw| Self { raw })
    }

    /// Only available when built with ICU
    #[cfg(feature = "icu")]
    pub fn with_options(pattern: &str, opts: &wrapper::Options) -> Result<Self, String> {
        wrapper::compile_regex(pattern, Some(opts)).map(|raw| Self { raw })
    }

    pub fn full_match(&self, text: &str) -> bool {
        wrapper::full_match(self.raw, text)
    }

    pub fn partial_match(&self, text: &str) -> bool {
        wrapper::partial_match(self.raw, text)
    }

    pub fn partial_captures<'t>(&self, text: &'t str) -> Option<Vec<Option<&'t str>>> {
        wrapper::captures(self.raw, text, false)
    }

    pub fn full_captures<'t>(&self, text: &'t str) -> Option<Vec<Option<&'t str>>> {
        wrapper::captures(self.raw, text, true)
    }

    pub fn num_captures(&self) -> usize {
        wrapper::group_count(self.raw)
    }

    pub fn replace_one(&self, text: &str, rewrite: &str) -> Option<String> {
        wrapper::replace(self.raw, text, rewrite, true)
    }

    pub fn replace_all(&self, text: &str, rewrite: &str) -> Option<String> {
        wrapper::replace(self.raw, text, rewrite, false)
    }
}

impl Drop for Regex {
    fn drop(&mut self) {
        wrapper::delete_regex(self.raw)
    }
}

