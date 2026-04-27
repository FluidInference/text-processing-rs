//! Public configuration for the unified `*_with_options` entry points.
//!
//! [`NormalizeOptions`] is the single extension point for caller-tunable
//! normalization behavior. Each field is an **orthogonal behavior flag**, not
//! a domain label — aviation flight numbers, sports scores, dispatch IDs and
//! similar code-style readings all reuse the same toggles, and new knobs are
//! added as additional fields rather than as new enum variants or new
//! function names.
//!
//! See issues
//! [#14](https://github.com/FluidInference/text-processing-rs/issues/14),
//! [#15](https://github.com/FluidInference/text-processing-rs/issues/15) and
//! [#23](https://github.com/FluidInference/text-processing-rs/issues/23) for
//! the motivating discussion on why this is a struct rather than a `Domain`
//! enum.
//!
//! # Stability
//!
//! New fields may be added in minor releases. Always construct
//! [`NormalizeOptions`] via [`NormalizeOptions::new`] (or `default()`) and
//! the chainable `with_*` methods — direct struct literals will break when
//! new fields are introduced.
//!
//! # Examples
//!
//! ```
//! use text_processing_rs::{normalize_sentence_with_options, NormalizeOptions};
//!
//! // Aviation / flight-number style: consecutive 0-99 chunks concatenate.
//! let opts = NormalizeOptions::new()
//!     .with_concat_compound_numbers(true)
//!     .with_max_span_tokens(8);
//!
//! assert_eq!(
//!     normalize_sentence_with_options("United seven eighty eight", opts),
//!     "United 788"
//! );
//! ```

/// Default maximum token span to consider when scanning a sentence.
///
/// Used by [`crate::normalize_sentence`] and by
/// [`crate::normalize_sentence_with_options`] when
/// [`NormalizeOptions::max_span_tokens`] is `None`.
pub const DEFAULT_MAX_SPAN_TOKENS: usize = 16;

/// Caller-tunable knobs for the unified
/// [`crate::normalize_with_options`] /
/// [`crate::normalize_sentence_with_options`] entry points.
///
/// Construct via [`NormalizeOptions::new`] or [`Default::default`] and
/// configure with the chainable `with_*` methods so future fields don't
/// break existing call sites.
///
/// All fields default to behavior matching plain
/// [`crate::normalize`] / [`crate::normalize_sentence`] — opt-in is the
/// only way to change semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NormalizeOptions {
    /// Read consecutive small-number compounds as concatenated digit groups
    /// instead of summing them.
    ///
    /// **Default:** `false` (preserve upstream NeMo grammatical reading).
    ///
    /// When `true`, the priority-89 aviation cardinal pass runs ahead of the
    /// time/date taggers and uses [`peel_compound_chunks`] semantics:
    /// - `"seven eighty eight"` → `"788"` (was `"95"` = 7 + 88) — issue
    ///   [#14](https://github.com/FluidInference/text-processing-rs/issues/14)
    /// - `"thirty five sixty two"` → `"3562"` (was `"97"` = 35 + 62) —
    ///   issue
    ///   [#23](https://github.com/FluidInference/text-processing-rs/issues/23)
    /// - `"two thirty five sixty two"` → `"23562"`
    /// - `"two thousand seventeen"` → `"2017"` (scale words still anchor
    ///   grammatical addition)
    /// - `"twenty one"` → `"21"` (single chunks never concatenate)
    ///
    /// Use cases: aviation flight numbers / call-signs, sports scores,
    /// jersey/room numbers, dispatch IDs, any code-style reading where
    /// consecutive small numbers should remain distinct.
    ///
    /// Money, measure, decimal and ordinal taggers retain their normal
    /// priorities and continue to win where they apply (e.g.
    /// `"five dollars"` → `"$5"` regardless of this flag).
    ///
    /// [`peel_compound_chunks`]: ../itn/en/cardinal/fn.peel_compound_chunks.html
    pub concat_compound_numbers: bool,

    /// Maximum span size (in whitespace-separated tokens) considered by the
    /// sliding-window sentence scanner.
    ///
    /// **Default:** `None`, which resolves to [`DEFAULT_MAX_SPAN_TOKENS`]
    /// (currently `16`).
    ///
    /// Lower values trade recall for speed and false-positive resistance —
    /// a span of `2` will catch `"twenty one"` → `"21"` but not the
    /// 5-token `"five dollars and fifty cents"` → `"$5.50"`. A span of `1`
    /// disables multi-token matching entirely.
    ///
    /// Ignored by [`crate::normalize_with_options`] — single-expression
    /// mode does not slide.
    pub max_span_tokens: Option<usize>,
}

impl NormalizeOptions {
    /// Construct an options bag with all fields at their library defaults.
    ///
    /// Equivalent to [`Default::default`] but `const`, so it can be used
    /// in `const` contexts.
    pub const fn new() -> Self {
        Self {
            concat_compound_numbers: false,
            max_span_tokens: None,
        }
    }

    /// Toggle [`Self::concat_compound_numbers`] (concatenate consecutive
    /// small-number chunks instead of summing them).
    pub const fn with_concat_compound_numbers(mut self, enabled: bool) -> Self {
        self.concat_compound_numbers = enabled;
        self
    }

    /// Set [`Self::max_span_tokens`] (sentence-mode sliding-window cap).
    ///
    /// Pass [`DEFAULT_MAX_SPAN_TOKENS`] explicitly to lock in the current
    /// default; pass `0` for single-token-only matching (rarely useful
    /// outside tests).
    pub const fn with_max_span_tokens(mut self, max_span_tokens: usize) -> Self {
        self.max_span_tokens = Some(max_span_tokens);
        self
    }
}
