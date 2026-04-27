//! Caller-tunable options for the unified `*_with_options` entry points.
//!
//! Construct via [`NormalizeOptions::new`] + chainable `with_*` methods so
//! new fields can land without breaking existing call sites.

/// Default sentence-mode sliding-window cap.
pub const DEFAULT_MAX_SPAN_TOKENS: usize = 16;

/// Options for [`crate::normalize_with_options`] and
/// [`crate::normalize_sentence_with_options`]. Defaults match plain
/// [`crate::normalize`] / [`crate::normalize_sentence`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NormalizeOptions {
    /// Concatenate consecutive small-number chunks instead of summing them.
    /// `"seven eighty eight"` → `"788"` (issue #14), `"thirty five sixty
    /// two"` → `"3562"` (issue #23). Default `false`.
    pub concat_compound_numbers: bool,

    /// Sentence-mode sliding-window cap (in tokens). `None` uses
    /// [`DEFAULT_MAX_SPAN_TOKENS`]. Ignored in single-expression mode.
    pub max_span_tokens: Option<usize>,

    /// Skip the ordinal tagger for the bare word `"second"` so it is not
    /// rewritten to `"2nd"` in phrases like `"give me a second"` (issue #22).
    /// Compound ordinals (`"twenty second"` → `"22nd"`) and date contexts
    /// (`"January second twenty twenty five"`) still convert. Default `false`.
    pub disable_bare_second: bool,
}

impl NormalizeOptions {
    /// `const` constructor with library defaults.
    pub const fn new() -> Self {
        Self {
            concat_compound_numbers: false,
            max_span_tokens: None,
            disable_bare_second: false,
        }
    }

    /// Set [`Self::concat_compound_numbers`].
    pub const fn with_concat_compound_numbers(mut self, enabled: bool) -> Self {
        self.concat_compound_numbers = enabled;
        self
    }

    /// Set [`Self::max_span_tokens`].
    pub const fn with_max_span_tokens(mut self, max_span_tokens: usize) -> Self {
        self.max_span_tokens = Some(max_span_tokens);
        self
    }

    /// Set [`Self::disable_bare_second`].
    pub const fn with_disable_bare_second(mut self, enabled: bool) -> Self {
        self.disable_bare_second = enabled;
        self
    }
}
