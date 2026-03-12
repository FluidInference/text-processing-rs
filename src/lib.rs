//! # text-processing-rs
//!
//! Inverse Text Normalization (ITN) — convert spoken-form ASR output to written form.
//!
//! Converts spoken-form text to written form:
//! - "two hundred thirty two" → "232"
//! - "five dollars and fifty cents" → "$5.50"
//! - "january fifth twenty twenty five" → "January 5, 2025"
//!
//! ## Usage
//!
//! ```
//! use text_processing_rs::normalize;
//!
//! let result = normalize("two hundred");
//! assert_eq!(result, "200");
//! ```

pub mod asr;
pub mod custom_rules;
pub mod tts;

#[cfg(feature = "ffi")]
pub mod ffi;

use asr::{
    cardinal, date, decimal, electronic, measure, money, ordinal, punctuation, telephone, time,
    whitelist, word,
};

/// Normalize spoken-form text to written form.
///
/// Tries taggers in order of specificity (most specific first).
/// Returns original text if no tagger matches.
pub fn normalize(input: &str) -> String {
    let input = input.trim();

    // Apply custom user rules first (highest priority)
    if let Some(result) = custom_rules::parse(input) {
        return result;
    }

    // Apply whitelist replacements (abbreviations, special terms)
    if let Some(result) = whitelist::parse(input) {
        return result;
    }

    // Try punctuation ("period" → ".", "comma" → ",")
    if let Some(result) = punctuation::parse(input) {
        return result;
    }

    // Try word patterns (spelled letters + numbers, numbers with punctuation)
    if let Some(result) = word::parse(input) {
        return result;
    }

    // Try time expressions (before telephone to avoid "two thirty" → alphanumeric)
    if let Some(result) = time::parse(input) {
        return result;
    }

    // Try date expressions (before telephone to avoid "nineteen ninety four" → alphanumeric)
    if let Some(result) = date::parse(input) {
        return result;
    }

    // Try money (contains number + currency) - before telephone
    if let Some(result) = money::parse(input) {
        return result;
    }

    // Try measurements (contains number + unit) - before telephone
    if let Some(result) = measure::parse(input) {
        return result;
    }

    // Try decimal numbers (before telephone to catch "sixty point two")
    if let Some(result) = decimal::parse(input) {
        return result;
    }

    // Try telephone/IP numbers (before electronic to catch IP addresses)
    if let Some(result) = telephone::parse(input) {
        return result;
    }

    // Try electronic addresses (emails, URLs)
    if let Some(result) = electronic::parse(input) {
        return result;
    }

    // Try decimal numbers
    if let Some(result) = decimal::parse(input) {
        return result;
    }

    // Try ordinal numbers
    if let Some(result) = ordinal::parse(input) {
        return result;
    }

    // Try cardinal number
    if let Some(num) = cardinal::parse(input) {
        return num;
    }

    // No match - return original
    input.to_string()
}

/// Normalize with language selection (future use).
pub fn normalize_with_lang(input: &str, _lang: &str) -> String {
    // TODO: Language-specific taggers
    normalize(input)
}

/// Default maximum token span to consider when scanning a sentence.
const DEFAULT_MAX_SPAN_TOKENS: usize = 16;

/// Try to parse a span of text using sentence-safe taggers.
///
/// Returns `(replacement, priority_score)` if a tagger matches.
/// Taggers are ordered by precision: high-confidence patterns first,
/// broad patterns (cardinal) last and limited to short spans.
///
/// Excluded in sentence mode: `word` and `telephone` (over-fire on natural language).
fn parse_span(span: &str) -> Option<(String, u8)> {
    let token_count = span.split_whitespace().count();
    if token_count == 0 {
        return None;
    }

    if let Some(result) = custom_rules::parse(span) {
        return Some((result, 110));
    }
    if let Some(result) = whitelist::parse(span) {
        return Some((result, 100));
    }
    if let Some(result) = punctuation::parse(span) {
        return Some((result, 98));
    }
    if let Some(result) = money::parse(span) {
        return Some((result, 95));
    }
    if let Some(result) = measure::parse(span) {
        return Some((result, 90));
    }
    if let Some(result) = date::parse(span) {
        return Some((result, 88));
    }
    if let Some(result) = time::parse(span) {
        return Some((result, 85));
    }
    if let Some(result) = electronic::parse(span) {
        return Some((result, 82));
    }
    if let Some(result) = decimal::parse(span) {
        return Some((result, 80));
    }
    if let Some(result) = ordinal::parse(span) {
        return Some((result, 75));
    }

    // Cardinal only for short spans to avoid over-matching on natural language.
    if token_count <= 4 {
        if let Some(result) = cardinal::parse(span) {
            return Some((result, 70));
        }
    }

    None
}

/// Normalize a full sentence, replacing spoken-form spans with written form.
///
/// Unlike [`normalize`] which expects the entire input to be a single expression,
/// this function scans for normalizable spans within a larger sentence.
/// Uses a default max span of 16 tokens.
///
/// ```
/// use text_processing_rs::normalize_sentence;
///
/// assert_eq!(normalize_sentence("I have twenty one apples"), "I have 21 apples");
/// assert_eq!(normalize_sentence("hello world"), "hello world");
/// ```
pub fn normalize_sentence(input: &str) -> String {
    normalize_sentence_with_max_span(input, DEFAULT_MAX_SPAN_TOKENS)
}

/// Normalize a full sentence with a configurable max span size.
///
/// `max_span_tokens` controls the maximum number of consecutive tokens
/// that will be considered as a single normalizable expression.
/// Smaller values are faster but may miss multi-word expressions.
/// Larger values catch more patterns but do more work per token.
///
/// ```
/// use text_processing_rs::normalize_sentence_with_max_span;
///
/// // Short span: only catches small expressions
/// assert_eq!(normalize_sentence_with_max_span("I have twenty one apples", 4), "I have 21 apples");
/// ```
pub fn normalize_sentence_with_max_span(input: &str, max_span_tokens: usize) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return trimmed.to_string();
    }

    let max_span = if max_span_tokens == 0 {
        1
    } else {
        max_span_tokens
    };
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    let mut out: Vec<String> = Vec::with_capacity(tokens.len());
    let mut i = 0usize;

    while i < tokens.len() {
        let max_end = usize::min(tokens.len(), i + max_span);
        let mut best: Option<(usize, String, u8)> = None;

        // Longest-span-first search keeps replacements stable and non-overlapping.
        for end in (i + 1..=max_end).rev() {
            let span = tokens[i..end].join(" ");
            let Some((candidate, score)) = parse_span(&span) else {
                continue;
            };

            // Reject no-op results (tagger returned same text).
            let candidate_trimmed = candidate.trim();
            if candidate_trimmed.is_empty() || candidate_trimmed == span {
                continue;
            }

            let candidate_len = end - i;
            match &best {
                None => {
                    best = Some((end, candidate, score));
                }
                Some((best_end, _, best_score)) => {
                    let best_len = *best_end - i;
                    if candidate_len > best_len
                        || (candidate_len == best_len && score > *best_score)
                    {
                        best = Some((end, candidate, score));
                    }
                }
            }
        }

        if let Some((end, replacement, _)) = best {
            out.push(replacement);
            i = end;
        } else {
            out.push(tokens[i].to_string());
            i += 1;
        }
    }

    out.join(" ")
}

// ── Text Normalization (written → spoken) ─────────────────────────────

/// Normalize written-form text to spoken form (Text Normalization).
///
/// Tries TN taggers in priority order (most specific first).
/// Returns original text if no tagger matches.
///
/// ```
/// use text_processing_rs::tn_normalize;
///
/// let result = tn_normalize("$5.50");
/// assert_eq!(result, "five dollars and fifty cents");
/// ```
pub fn tn_normalize(input: &str) -> String {
    let input = input.trim();

    if let Some(result) = tts::whitelist::parse(input) {
        return result;
    }
    if let Some(result) = tts::money::parse(input) {
        return result;
    }
    if let Some(result) = tts::measure::parse(input) {
        return result;
    }
    if let Some(result) = tts::date::parse(input) {
        return result;
    }
    if let Some(result) = tts::time::parse(input) {
        return result;
    }
    if let Some(result) = tts::electronic::parse(input) {
        return result;
    }
    if let Some(result) = tts::telephone::parse(input) {
        return result;
    }
    if let Some(result) = tts::ordinal::parse(input) {
        return result;
    }
    if let Some(result) = tts::decimal::parse(input) {
        return result;
    }
    if let Some(result) = tts::cardinal::parse(input) {
        return result;
    }

    input.to_string()
}

/// Try to parse a span of text using TN taggers.
///
/// Returns `(replacement, priority_score)` if a tagger matches.
fn tn_parse_span(span: &str) -> Option<(String, u8)> {
    if span.is_empty() {
        return None;
    }

    if let Some(result) = tts::whitelist::parse(span) {
        return Some((result, 100));
    }
    if let Some(result) = tts::money::parse(span) {
        return Some((result, 95));
    }
    if let Some(result) = tts::measure::parse(span) {
        return Some((result, 90));
    }
    if let Some(result) = tts::date::parse(span) {
        return Some((result, 88));
    }
    if let Some(result) = tts::time::parse(span) {
        return Some((result, 85));
    }
    if let Some(result) = tts::electronic::parse(span) {
        return Some((result, 82));
    }
    if let Some(result) = tts::telephone::parse(span) {
        return Some((result, 78));
    }
    if let Some(result) = tts::ordinal::parse(span) {
        return Some((result, 75));
    }
    if let Some(result) = tts::decimal::parse(span) {
        return Some((result, 73));
    }
    if let Some(result) = tts::cardinal::parse(span) {
        return Some((result, 70));
    }

    None
}

/// Normalize a full sentence, replacing written-form spans with spoken form.
///
/// Unlike [`tn_normalize`] which expects the entire input to be a single expression,
/// this function scans for normalizable spans within a larger sentence.
///
/// ```
/// use text_processing_rs::tn_normalize_sentence;
///
/// assert_eq!(tn_normalize_sentence("I paid $5 for 23 items"), "I paid five dollars for twenty three items");
/// ```
pub fn tn_normalize_sentence(input: &str) -> String {
    tn_normalize_sentence_with_max_span(input, DEFAULT_MAX_SPAN_TOKENS)
}

/// Normalize a full sentence (TN) with a configurable max span size.
pub fn tn_normalize_sentence_with_max_span(input: &str, max_span_tokens: usize) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return trimmed.to_string();
    }

    let max_span = if max_span_tokens == 0 {
        1
    } else {
        max_span_tokens
    };
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    let mut out: Vec<String> = Vec::with_capacity(tokens.len());
    let mut i = 0usize;

    while i < tokens.len() {
        let max_end = usize::min(tokens.len(), i + max_span);
        let mut best: Option<(usize, String, u8)> = None;

        for end in (i + 1..=max_end).rev() {
            let span = tokens[i..end].join(" ");
            let Some((candidate, score)) = tn_parse_span(&span) else {
                continue;
            };

            let candidate_trimmed = candidate.trim();
            if candidate_trimmed.is_empty() || candidate_trimmed == span {
                continue;
            }

            let candidate_len = end - i;
            match &best {
                None => {
                    best = Some((end, candidate, score));
                }
                Some((best_end, _, best_score)) => {
                    let best_len = *best_end - i;
                    if candidate_len > best_len
                        || (candidate_len == best_len && score > *best_score)
                    {
                        best = Some((end, candidate, score));
                    }
                }
            }
        }

        if let Some((end, replacement, _)) = best {
            out.push(replacement);
            i = end;
        } else {
            out.push(tokens[i].to_string());
            i += 1;
        }
    }

    out.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cardinal() {
        assert_eq!(normalize("one"), "1");
        assert_eq!(normalize("twenty one"), "21");
        assert_eq!(normalize("one hundred"), "100");
    }

    #[test]
    fn test_basic_money() {
        assert_eq!(normalize("five dollars"), "$5");
    }

    #[test]
    fn test_passthrough() {
        assert_eq!(normalize("hello world"), "hello world");
    }

    #[test]
    fn test_sentence_cardinal() {
        assert_eq!(
            normalize_sentence("I have twenty one apples"),
            "I have 21 apples"
        );
    }

    #[test]
    fn test_sentence_money() {
        assert_eq!(
            normalize_sentence("five dollars and fifty cents for the coffee"),
            "$5.50 for the coffee"
        );
    }

    #[test]
    fn test_sentence_passthrough() {
        assert_eq!(normalize_sentence("hello world"), "hello world");
        assert_eq!(
            normalize_sentence("the quick brown fox"),
            "the quick brown fox"
        );
    }

    #[test]
    fn test_sentence_mixed() {
        assert_eq!(
            normalize_sentence("I paid five dollars for twenty three items"),
            "I paid $5 for 23 items"
        );
    }

    #[test]
    fn test_sentence_empty() {
        assert_eq!(normalize_sentence(""), "");
        assert_eq!(normalize_sentence("   "), "");
    }

    #[test]
    fn test_sentence_single_number() {
        assert_eq!(normalize_sentence("forty two"), "42");
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(normalize("period"), ".");
        assert_eq!(normalize("comma"), ",");
        assert_eq!(normalize("question mark"), "?");
        assert_eq!(normalize("exclamation point"), "!");
    }

    #[test]
    fn test_sentence_punctuation() {
        assert_eq!(normalize_sentence("hello period"), "hello .");
        assert_eq!(normalize_sentence("yes comma I agree"), "yes , I agree");
        assert_eq!(normalize_sentence("really question mark"), "really ?");
    }

    // ── TN Tests ──

    #[test]
    fn test_tn_cardinal() {
        assert_eq!(tn_normalize("123"), "one hundred twenty three");
        assert_eq!(tn_normalize("0"), "zero");
        assert_eq!(tn_normalize("1000"), "one thousand");
    }

    #[test]
    fn test_tn_money() {
        assert_eq!(tn_normalize("$5.50"), "five dollars and fifty cents");
        assert_eq!(tn_normalize("$1"), "one dollar");
        assert_eq!(tn_normalize("$0.01"), "one cent");
    }

    #[test]
    fn test_tn_ordinal() {
        assert_eq!(tn_normalize("1st"), "first");
        assert_eq!(tn_normalize("21st"), "twenty first");
        assert_eq!(tn_normalize("100th"), "one hundredth");
    }

    #[test]
    fn test_tn_time() {
        assert_eq!(tn_normalize("2:30"), "two thirty");
        assert_eq!(tn_normalize("2:05"), "two oh five");
        assert_eq!(tn_normalize("2:00 PM"), "two p m");
    }

    #[test]
    fn test_tn_date() {
        assert_eq!(
            tn_normalize("January 5, 2025"),
            "january fifth twenty twenty five"
        );
        assert_eq!(tn_normalize("1980s"), "nineteen eighties");
    }

    #[test]
    fn test_tn_passthrough() {
        assert_eq!(tn_normalize("hello world"), "hello world");
    }

    #[test]
    fn test_tn_sentence() {
        assert_eq!(
            tn_normalize_sentence("I paid $5 for 23 items"),
            "I paid five dollars for twenty three items"
        );
    }

    #[test]
    fn test_tn_sentence_passthrough() {
        assert_eq!(tn_normalize_sentence("hello world"), "hello world");
        assert_eq!(tn_normalize_sentence(""), "");
    }
}
