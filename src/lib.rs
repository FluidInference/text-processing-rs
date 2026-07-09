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

pub mod custom_rules;
pub mod itn;
pub mod options;
pub mod tn;

#[cfg(feature = "ffi")]
pub mod ffi;
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub mod wasm;

pub use options::{NormalizeOptions, DEFAULT_MAX_SPAN_TOKENS};

use itn::en::{
    cardinal, date, decimal, electronic, measure, money, ordinal, punctuation, telephone, time,
    whitelist, word,
};

/// Normalize spoken-form text to written form.
///
/// Tries taggers in order of specificity (most specific first).
/// Returns original text if no tagger matches.
pub fn normalize(input: &str) -> String {
    normalize_inner(input, false)
}

/// Single-expression dispatch with the `disable_bare_second` flag plumbed in.
/// Issue #22: when the flag is set and the trimmed input is exactly the bare
/// word `"second"`, the ordinal tagger is skipped so it stays literal.
fn normalize_inner(input: &str, disable_bare_second: bool) -> String {
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

    // Try ordinal numbers (issue #22: skip the bare "second" case when opted out).
    let skip_ordinal = disable_bare_second && input.eq_ignore_ascii_case("second");
    if !skip_ordinal {
        if let Some(result) = ordinal::parse(input) {
            return result;
        }
    }

    // Try cardinal number
    if let Some(num) = cardinal::parse(input) {
        return num;
    }

    // No match - return original
    input.to_string()
}

/// Single-expression normalize with caller-specified [`NormalizeOptions`].
///
/// When `options.concat_compound_numbers` is `true`, `cardinal::parse_aviation`
/// is tried *after* the high-confidence taggers (`custom_rules`, `whitelist`,
/// `punctuation`, `word`) but *before* `time` and `date`. The result: when
/// the whole input is a number-only phrase like `"two thirty five"` or
/// `"seven eighty eight"`, concat-compound reading wins (`"235"`, `"788"`)
/// instead of being eaten as a time (`"02:35"`) or an old-year via the date
/// tagger. Non-number phrases still flow through the rest of the pipeline
/// (`"five dollars"` → `"$5"` via the money tagger).
///
/// ```
/// use text_processing_rs::{normalize_with_options, NormalizeOptions};
///
/// let opts = NormalizeOptions::new().with_concat_compound_numbers(true);
/// assert_eq!(normalize_with_options("seven eighty eight", opts), "788");
/// assert_eq!(normalize_with_options("two thirty five", opts), "235");
/// // Non-number phrases are unaffected.
/// assert_eq!(normalize_with_options("hello world", opts), "hello world");
/// ```
pub fn normalize_with_options(input: &str, options: NormalizeOptions) -> String {
    if !options.concat_compound_numbers {
        return normalize_inner(input, options.disable_bare_second);
    }

    let input = input.trim();

    // High-confidence rules still win.
    if let Some(result) = custom_rules::parse(input) {
        return result;
    }
    if let Some(result) = whitelist::parse(input) {
        return result;
    }
    if let Some(result) = punctuation::parse(input) {
        return result;
    }
    if let Some(result) = word::parse(input) {
        return result;
    }

    // Concat-compound cardinal beats time/date.
    if let Some(num) = cardinal::parse_aviation(input) {
        return num;
    }

    // Fall back to the standard pipeline for anything not recognised
    // (money, measure, decimal, ordinal, telephone, etc.).
    normalize_inner(input, options.disable_bare_second)
}

/// Normalize with language selection.
///
/// Supports language-specific ITN taggers for converting spoken-form
/// ASR output to written form in different languages.
///
/// Supported languages: "en" (default), "fr" (French), "de" (German),
/// "es" (Spanish), "hi" (Hindi), "ja" (Japanese), "zh" (Chinese).
pub fn normalize_with_lang(input: &str, lang: &str) -> String {
    let input = input.trim();

    match lang {
        "en" => normalize(input),
        "fr" => normalize_lang_fr(input),
        "de" => normalize_lang_de(input),
        "es" => normalize_lang_es(input),
        "hi" => normalize_lang_hi(input),
        "ja" => normalize_lang_ja(input),
        "zh" => normalize_lang_zh(input),
        _ => normalize(input), // Default to English
    }
}

/// Strip trailing punctuation from input: "vingt!" → ("vingt", "!")
fn strip_trailing_punctuation(input: &str) -> Option<(&str, &str)> {
    let punct_chars = ['!', '?', '.', ',', ';', ':', '…'];
    let trimmed = input.trim();
    for &p in &punct_chars {
        if trimmed.ends_with(p) {
            let text = trimmed[..trimmed.len() - p.len_utf8()].trim();
            let punct = &trimmed[trimmed.len() - p.len_utf8()..];
            if !text.is_empty() {
                return Some((text, punct));
            }
        }
    }
    None
}

// ── French ITN ──────────────────────────────────────────────────────────

/// ITN for French
fn normalize_lang_fr(input: &str) -> String {
    // Try full input first
    if let Some(result) = try_fr_taggers(input) {
        return result;
    }

    // Try stripping trailing punctuation: "vingt!" → try "vingt" then append " !"
    if let Some((text, punct)) = strip_trailing_punctuation(input) {
        if let Some(result) = try_fr_taggers(text) {
            return format!("{} {}", result, punct);
        }
    }

    // Try partial number normalization: "quarante trois" → "40 trois"
    // Only when input has exactly 2 space-separated tokens
    if let Some(result) = try_fr_partial_cardinal(input) {
        return result;
    }

    // No match - return original
    input.to_string()
}

/// Try all French ITN taggers on the input
fn try_fr_taggers(input: &str) -> Option<String> {
    if let Some(result) = custom_rules::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::fr::whitelist::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::fr::punctuation::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::fr::word::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::fr::time::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::fr::date::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::fr::money::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::fr::measure::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::fr::electronic::parse(input) {
        return Some(result);
    }
    // Fraction before ordinal: `un cinquième` is a fraction (1/5), but the
    // ordinal tagger would otherwise greedily read the `-ième` word. Bare
    // ordinals (`cinquième`) return None from fraction and fall through.
    if let Some(result) = itn::fr::fraction::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::fr::ordinal::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::fr::decimal::parse(input) {
        return Some(result);
    }
    if let Some(num) = itn::fr::cardinal::parse(input) {
        return Some(num);
    }
    // Telephone last since it can match numbers
    if let Some(result) = itn::fr::telephone::parse(input) {
        return Some(result);
    }
    None
}

/// Try partial cardinal normalization for French.
/// "quarante trois" → "40 trois" (normalize first word if it's a tens/hundreds number)
fn try_fr_partial_cardinal(input: &str) -> Option<String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.len() != 2 {
        return None;
    }

    // Only convert the first token if it's a standalone number ≥ 10
    let first = tokens[0];
    let first_lower = first.to_lowercase();
    if let Some(num) = itn::fr::cardinal::words_to_number(&first_lower) {
        if num >= 10 {
            return Some(format!("{} {}", num, tokens[1]));
        }
    }

    None
}

// ── German ITN ──────────────────────────────────────────────────────────

/// ITN for German
fn normalize_lang_de(input: &str) -> String {
    // Try full input first
    if let Some(result) = try_de_taggers(input) {
        return result;
    }

    // Try stripping trailing punctuation: "zwanzig!" → try "zwanzig" then append " !"
    if let Some((text, punct)) = strip_trailing_punctuation(input) {
        if let Some(result) = try_de_taggers(text) {
            return format!("{} {}", result, punct);
        }
    }

    // No match - return original
    input.to_string()
}

/// Try all German ITN taggers on the input
fn try_de_taggers(input: &str) -> Option<String> {
    if let Some(result) = custom_rules::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::de::whitelist::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::de::punctuation::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::de::time::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::de::date::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::de::money::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::de::measure::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::de::electronic::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::de::ordinal::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::de::fraction::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::de::decimal::parse(input) {
        return Some(result);
    }
    if let Some(num) = itn::de::cardinal::parse(input) {
        return Some(num);
    }
    // Telephone last since it can match digit sequences
    if let Some(result) = itn::de::telephone::parse(input) {
        return Some(result);
    }
    None
}

// ── Spanish ITN ─────────────────────────────────────────────────────────

/// ITN for Spanish
fn normalize_lang_es(input: &str) -> String {
    // Try full input first
    if let Some(result) = try_es_taggers(input) {
        return result;
    }

    // Try stripping trailing punctuation: "veinte!" → try "veinte" then append " !"
    if let Some((text, punct)) = strip_trailing_punctuation(input) {
        if let Some(result) = try_es_taggers(text) {
            return format!("{} {}", result, punct);
        }
    }

    // No match - return original
    input.to_string()
}

/// Try all Spanish ITN taggers on the input
fn try_es_taggers(input: &str) -> Option<String> {
    if let Some(result) = custom_rules::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::es::whitelist::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::es::punctuation::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::es::word::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::es::time::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::es::date::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::es::money::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::es::measure::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::es::electronic::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::es::ordinal::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::es::fraction::parse(input) {
        return Some(result);
    }
    if let Some(result) = itn::es::decimal::parse(input) {
        return Some(result);
    }
    if let Some(num) = itn::es::cardinal::parse(input) {
        return Some(num);
    }
    // Telephone last since it can match digit sequences
    if let Some(result) = itn::es::telephone::parse(input) {
        return Some(result);
    }
    None
}

/// Decompose precomposed Devanagari nukta characters to base + nukta.
/// This ensures consistent matching regardless of input encoding.
fn decompose_devanagari_nukta(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 16);
    for c in input.chars() {
        match c {
            '\u{0958}' => {
                out.push('\u{0915}');
                out.push('\u{093C}');
            } // क़
            '\u{0959}' => {
                out.push('\u{0916}');
                out.push('\u{093C}');
            } // ख़
            '\u{095A}' => {
                out.push('\u{0917}');
                out.push('\u{093C}');
            } // ग़
            '\u{095B}' => {
                out.push('\u{091C}');
                out.push('\u{093C}');
            } // ज़
            '\u{095C}' => {
                out.push('\u{0921}');
                out.push('\u{093C}');
            } // ड़
            '\u{095D}' => {
                out.push('\u{0922}');
                out.push('\u{093C}');
            } // ढ़
            '\u{095E}' => {
                out.push('\u{092B}');
                out.push('\u{093C}');
            } // फ़
            '\u{095F}' => {
                out.push('\u{092F}');
                out.push('\u{093C}');
            } // य़
            _ => out.push(c),
        }
    }
    out
}

/// ITN for Hindi.
///
/// Hindi ITN uses a sentence-scanning approach. Each processor scans the
/// full input for its patterns and replaces Hindi number word spans in-place.
/// Order matters — more specific patterns (money, measure, time, date)
/// run before generic cardinal replacement.
fn normalize_lang_hi(input: &str) -> String {
    // Normalize precomposed nukta characters to decomposed form
    let input = decompose_devanagari_nukta(input);
    let mut result = input;

    // 1. Whitelist (abbreviations: डॉक्टर→डॉ., etc.)
    result = itn::hi::whitelist::process(&result);

    // 2. Money (number + currency name → symbol + digits)
    result = itn::hi::money::process(&result);

    // 3. Date (day + month [+ year], ranges, eras)
    result = itn::hi::date::process(&result);

    // 4. Time (X बजे/घंटा + मिनट/सेकंड)
    // Before measure so "X घंटा Y मिनट" isn't caught as measure
    result = itn::hi::time::process(&result);

    // 5. Measure (number + unit → digits + symbol)
    result = itn::hi::measure::process(&result);

    // 6. Fractions (X बटा Y, X सही Y बटा Z)
    result = itn::hi::fraction::process(&result);

    // 7. Ordinal (Xवां, Xवीं, Xवें)
    result = itn::hi::ordinal::process(&result);

    // 8. Decimal (X दशमलव Y)
    result = itn::hi::decimal::process(&result);

    // 9. Cardinal — convert compound number words (with scale words) and
    //    single number words to Devanagari digits. Must run BEFORE
    //    telephone/address so compound numbers like "एक सौ" are grouped.
    result = itn::hi::cardinal::process(&result);

    // 10. Telephone (digit-by-digit sequences ≥ 4 Devanagari digits)
    result = itn::hi::telephone::process(&result);

    // 11. Address (digit-by-digit with हाइफ़न/बटा, comma-separated digits)
    result = itn::hi::address::process(&result);

    result
}

// ── Japanese ITN ────────────────────────────────────────────────────────

/// ITN for Japanese.
///
/// Japanese ITN uses a sentence-scanning approach: each processor scans the
/// full input for its patterns and replaces kanji number spans in-place.
/// Order matters — more specific patterns (fractions, decimals, dates, times)
/// run before generic cardinal replacement.
fn normalize_lang_ja(input: &str) -> String {
    let mut result = input.to_string();

    // 1. Fractions first (X分のY) — before time which also uses 分
    result = itn::ja::fraction::process(&result);

    // 2. Decimals (X点Y) — before cardinal swallows the kanji
    result = itn::ja::decimal::process(&result);

    // 3. Dates (年月日, 世紀, 年代, weekdays, ranges)
    result = itn::ja::date::process(&result);

    // 4. Time (時, 分) — after fractions to avoid 分の collision
    result = itn::ja::time::process(&result);

    // 5. Ordinals (番目, 第)
    result = itn::ja::ordinal::process(&result);

    // 6. Cardinal — catch remaining standalone kanji number spans
    result = itn::ja::cardinal::replace_kanji_numbers(&result);

    result
}

// ── Chinese ITN ─────────────────────────────────────────────────────────

/// ITN for Chinese.
///
/// Chinese ITN uses a sentence-scanning approach similar to Japanese.
/// Each processor scans the full input for its patterns and replaces
/// Chinese number spans in-place.
/// Order matters — whitelist, money, and specific patterns run before cardinal.
fn normalize_lang_zh(input: &str) -> String {
    let mut result = input.to_string();

    // 1. Whitelist (abbreviation mappings)
    result = itn::zh::whitelist::process(&result);

    // 2. Money (before decimal to catch currency-specific decimal patterns like 一点五万美元)
    result = itn::zh::money::process(&result);

    // 3. Fractions (X分之Y) — before time which also uses 分
    result = itn::zh::fraction::process(&result);

    // 4. Time (X点Y分, X分钟, X秒钟) — before decimal so 点 with 分/刻/半 isn't consumed as decimal
    result = itn::zh::time::process(&result);

    // 5. Decimals (X点Y)
    result = itn::zh::decimal::process(&result);

    // 6. Dates (年月日, 公元/纪元)
    result = itn::zh::date::process(&result);

    // 7. Ordinals (第X)
    result = itn::zh::ordinal::process(&result);

    // 8. Cardinal — catch remaining standalone Chinese number spans
    result = itn::zh::cardinal::replace_zh_numbers(&result);

    result
}

// ── Multi-language TN helpers ──────────────────────────────────────────

/// Try TN taggers for a specific language.
///
/// Each language module provides: money, measure, date, time, ordinal, decimal, cardinal.
fn tn_normalize_for_lang(input: &str, lang: &str) -> String {
    let input = input.trim();

    match lang {
        "en" => tn_normalize(input),
        "fr" => tn_normalize_lang_fr(input),
        "es" => tn_normalize_lang_es(input),
        "de" => tn_normalize_lang_de(input),
        "zh" => tn_normalize_lang_zh(input),
        "hi" => tn_normalize_lang_hi(input),
        "ja" => tn_normalize_lang_ja(input),
        _ => tn_normalize(input),
    }
}

fn tn_normalize_lang_fr(input: &str) -> String {
    if let Some(r) = tn::fr::whitelist::parse(input) {
        return r;
    }
    if let Some(r) = tn::fr::money::parse(input) {
        return r;
    }
    if let Some(r) = tn::fr::measure::parse(input) {
        return r;
    }
    if let Some(r) = tn::fr::date::parse(input) {
        return r;
    }
    if let Some(r) = tn::fr::time::parse(input) {
        return r;
    }
    if let Some(r) = tn::fr::electronic::parse(input) {
        return r;
    }
    if let Some(r) = tn::fr::telephone::parse(input) {
        return r;
    }
    if let Some(r) = tn::fr::ordinal::parse(input) {
        return r;
    }
    if let Some(r) = tn::fr::decimal::parse(input) {
        return r;
    }
    if let Some(r) = tn::fr::cardinal::parse(input) {
        return r;
    }
    input.to_string()
}

fn tn_normalize_lang_es(input: &str) -> String {
    if let Some(r) = tn::es::whitelist::parse(input) {
        return r;
    }
    if let Some(r) = tn::es::money::parse(input) {
        return r;
    }
    if let Some(r) = tn::es::measure::parse(input) {
        return r;
    }
    if let Some(r) = tn::es::date::parse(input) {
        return r;
    }
    if let Some(r) = tn::es::time::parse(input) {
        return r;
    }
    if let Some(r) = tn::es::electronic::parse(input) {
        return r;
    }
    if let Some(r) = tn::es::telephone::parse(input) {
        return r;
    }
    if let Some(r) = tn::es::ordinal::parse(input) {
        return r;
    }
    if let Some(r) = tn::es::decimal::parse(input) {
        return r;
    }
    if let Some(r) = tn::es::cardinal::parse(input) {
        return r;
    }
    input.to_string()
}

fn tn_normalize_lang_de(input: &str) -> String {
    if let Some(r) = tn::de::whitelist::parse(input) {
        return r;
    }
    if let Some(r) = tn::de::money::parse(input) {
        return r;
    }
    if let Some(r) = tn::de::measure::parse(input) {
        return r;
    }
    if let Some(r) = tn::de::date::parse(input) {
        return r;
    }
    if let Some(r) = tn::de::time::parse(input) {
        return r;
    }
    if let Some(r) = tn::de::electronic::parse(input) {
        return r;
    }
    if let Some(r) = tn::de::telephone::parse(input) {
        return r;
    }
    if let Some(r) = tn::de::ordinal::parse(input) {
        return r;
    }
    if let Some(r) = tn::de::decimal::parse(input) {
        return r;
    }
    if let Some(r) = tn::de::cardinal::parse(input) {
        return r;
    }
    input.to_string()
}

fn tn_normalize_lang_zh(input: &str) -> String {
    if let Some(r) = tn::zh::whitelist::parse(input) {
        return r;
    }
    if let Some(r) = tn::zh::money::parse(input) {
        return r;
    }
    if let Some(r) = tn::zh::measure::parse(input) {
        return r;
    }
    if let Some(r) = tn::zh::date::parse(input) {
        return r;
    }
    if let Some(r) = tn::zh::time::parse(input) {
        return r;
    }
    if let Some(r) = tn::zh::electronic::parse(input) {
        return r;
    }
    if let Some(r) = tn::zh::telephone::parse(input) {
        return r;
    }
    if let Some(r) = tn::zh::ordinal::parse(input) {
        return r;
    }
    if let Some(r) = tn::zh::decimal::parse(input) {
        return r;
    }
    if let Some(r) = tn::zh::cardinal::parse(input) {
        return r;
    }
    input.to_string()
}

fn tn_normalize_lang_hi(input: &str) -> String {
    if let Some(r) = tn::hi::whitelist::parse(input) {
        return r;
    }
    if let Some(r) = tn::hi::money::parse(input) {
        return r;
    }
    if let Some(r) = tn::hi::measure::parse(input) {
        return r;
    }
    if let Some(r) = tn::hi::date::parse(input) {
        return r;
    }
    if let Some(r) = tn::hi::time::parse(input) {
        return r;
    }
    if let Some(r) = tn::hi::electronic::parse(input) {
        return r;
    }
    if let Some(r) = tn::hi::telephone::parse(input) {
        return r;
    }
    if let Some(r) = tn::hi::ordinal::parse(input) {
        return r;
    }
    if let Some(r) = tn::hi::decimal::parse(input) {
        return r;
    }
    if let Some(r) = tn::hi::cardinal::parse(input) {
        return r;
    }
    input.to_string()
}

fn tn_normalize_lang_ja(input: &str) -> String {
    if let Some(r) = tn::ja::whitelist::parse(input) {
        return r;
    }
    if let Some(r) = tn::ja::money::parse(input) {
        return r;
    }
    if let Some(r) = tn::ja::measure::parse(input) {
        return r;
    }
    if let Some(r) = tn::ja::date::parse(input) {
        return r;
    }
    if let Some(r) = tn::ja::time::parse(input) {
        return r;
    }
    if let Some(r) = tn::ja::electronic::parse(input) {
        return r;
    }
    if let Some(r) = tn::ja::telephone::parse(input) {
        return r;
    }
    if let Some(r) = tn::ja::ordinal::parse(input) {
        return r;
    }
    if let Some(r) = tn::ja::decimal::parse(input) {
        return r;
    }
    if let Some(r) = tn::ja::cardinal::parse(input) {
        return r;
    }
    input.to_string()
}

/// TN parse span for a specific language.
fn tn_parse_span_lang(span: &str, lang: &str) -> Option<(String, u8)> {
    if span.is_empty() {
        return None;
    }

    macro_rules! try_lang_taggers {
        ($mod:path) => {{
            use $mod as lang;
            if let Some(r) = lang::whitelist::parse(span) {
                return Some((r, 100));
            }
            if let Some(r) = lang::money::parse(span) {
                return Some((r, 95));
            }
            if let Some(r) = lang::measure::parse(span) {
                return Some((r, 90));
            }
            if let Some(r) = lang::date::parse(span) {
                return Some((r, 88));
            }
            if let Some(r) = lang::time::parse(span) {
                return Some((r, 85));
            }
            if let Some(r) = lang::electronic::parse(span) {
                return Some((r, 82));
            }
            if let Some(r) = lang::telephone::parse(span) {
                return Some((r, 78));
            }
            if let Some(r) = lang::ordinal::parse(span) {
                return Some((r, 75));
            }
            if let Some(r) = lang::decimal::parse(span) {
                return Some((r, 73));
            }
            if let Some(r) = lang::cardinal::parse(span) {
                return Some((r, 70));
            }
        }};
    }

    match lang {
        "en" => {
            try_lang_taggers!(tn::en);
        }
        "fr" => {
            try_lang_taggers!(tn::fr);
        }
        "es" => {
            try_lang_taggers!(tn::es);
        }
        "de" => {
            try_lang_taggers!(tn::de);
        }
        "zh" => {
            try_lang_taggers!(tn::zh);
        }
        "hi" => {
            try_lang_taggers!(tn::hi);
        }
        "ja" => {
            try_lang_taggers!(tn::ja);
        }
        _ => {
            return tn_parse_span(span);
        }
    }

    None
}

/// Try to parse a span of text using sentence-safe taggers.
///
/// Returns `(replacement, priority_score)` if a tagger matches.
/// Taggers are ordered by precision: high-confidence patterns first,
/// broad patterns (cardinal) last and limited to short spans.
///
/// Excluded in sentence mode: `word` and `telephone` (over-fire on natural language).
///
/// `concat_compound`: when `true`, `cardinal::parse_aviation` is tried at
/// priority 89 (above `date`=88 and `time`=85, below `measure`=90 /
/// `money`=95) and the regular cardinal fallback at 70 is skipped (the
/// concat-compound reader already falls back to grammatical when the
/// concat pattern does not apply).
fn parse_span(
    span: &str,
    concat_compound: bool,
    disable_bare_second: bool,
) -> Option<(String, u8)> {
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

    // Concat-compound cardinal opt-in: priority 89, beats date/time. No
    // short-span gate — this is opt-in, so the caller has accepted
    // aggressive matching across longer spans like "one thousand two
    // hundred thirty four". `parse_aviation` falls back to grammatical when
    // the concat pattern does not apply, so non-concat phrases still resolve.
    if concat_compound {
        if let Some(result) = cardinal::parse_aviation(span) {
            return Some((result, 89));
        }
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
    // Issue #22: when `disable_bare_second` is set, the bare standalone
    // word `"second"` is *not* converted to `"2nd"` so phrases like
    // `"give me a second"` stay literal. Compound ordinals
    // (`"twenty second"`) still flow through this branch because they
    // span 2+ tokens.
    let skip_ordinal =
        disable_bare_second && token_count == 1 && span.trim().eq_ignore_ascii_case("second");
    if !skip_ordinal {
        if let Some(result) = ordinal::parse(span) {
            return Some((result, 75));
        }
    }

    // Default cardinal fallback (priority 70). In concat-compound mode the
    // cardinal path is already covered by the priority-89 branch above.
    if !concat_compound && token_count <= 4 {
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
    normalize_sentence_inner(input, DEFAULT_MAX_SPAN_TOKENS, false, false)
}

/// Unified sentence-mode entry point.
///
/// Combines `concat_compound_numbers` and `max_span_tokens` configuration in
/// a single call. When `max_span_tokens` is `None`, [`DEFAULT_MAX_SPAN_TOKENS`]
/// (16) is used.
///
/// ```
/// use text_processing_rs::{normalize_sentence_with_options, NormalizeOptions};
///
/// // Default behavior, default span
/// assert_eq!(
///     normalize_sentence_with_options("I have twenty one apples", NormalizeOptions::new()),
///     "I have 21 apples"
/// );
///
/// // Concat-compound (aviation-style), custom span
/// let opts = NormalizeOptions::new()
///     .with_concat_compound_numbers(true)
///     .with_max_span_tokens(8);
/// assert_eq!(
///     normalize_sentence_with_options("United seven eighty eight", opts),
///     "United 788"
/// );
/// ```
pub fn normalize_sentence_with_options(input: &str, options: NormalizeOptions) -> String {
    let max_span = options.max_span_tokens.unwrap_or(DEFAULT_MAX_SPAN_TOKENS);
    normalize_sentence_inner(
        input,
        max_span,
        options.concat_compound_numbers,
        options.disable_bare_second,
    )
}

/// Per-pretoken record: the token text plus the original separator that
/// preceded it (`" "` for whitespace, `""` if the token came from a
/// punctuation split or starts the input).
struct Pretoken {
    text: String,
    sep: &'static str,
}

/// ASCII punctuation characters that are split off the leading or trailing
/// edge of a whitespace-separated word so taggers see clean number phrases
/// (issue #21). Apostrophes and hyphens are intentionally excluded so
/// contractions ("don't") and hyphenated words ("twenty-one") stay intact.
fn is_split_punct(c: char) -> bool {
    matches!(
        c,
        ',' | '.' | ';' | ':' | '!' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '`'
    )
}

/// Fold NeMo's double-backtick quotes to a straight double quote so the
/// pretokenizer splits them off and the quoted content still normalizes
/// (`` ``cat`` `` → `"cat"`). TN sentence mode only.
fn unify_tn_quotes(input: &str) -> String {
    input.replace("``", "\"")
}

/// Split each whitespace-separated word into leading punctuation, core, and
/// trailing punctuation pretokens, preserving the original leading separator
/// on the first piece of each word so output reconstruction matches input
/// spacing.
fn pretokenize(input: &str) -> Vec<Pretoken> {
    let mut out: Vec<Pretoken> = Vec::new();
    let mut first_word = true;

    for word in input.split_whitespace() {
        let word_lead: &'static str = if first_word { "" } else { " " };
        first_word = false;

        // char_indices keeps multi-byte chars intact when slicing.
        let chars: Vec<(usize, char)> = word.char_indices().collect();
        if chars.is_empty() {
            continue;
        }

        let mut start = 0usize;
        while start < chars.len() && is_split_punct(chars[start].1) {
            start += 1;
        }
        let mut end = chars.len();
        while end > start && is_split_punct(chars[end - 1].1) {
            end -= 1;
        }

        let mut next_sep = word_lead;

        // Leading punctuation: each character becomes its own pretoken.
        for &(_, ch) in &chars[..start] {
            out.push(Pretoken {
                text: ch.to_string(),
                sep: next_sep,
            });
            next_sep = "";
        }

        // Core text (may be empty if the whole word was punctuation).
        if start < end {
            let core_start = chars[start].0;
            let core_end = if end < chars.len() {
                chars[end].0
            } else {
                word.len()
            };
            out.push(Pretoken {
                text: word[core_start..core_end].to_string(),
                sep: next_sep,
            });
            next_sep = "";
        }

        // Trailing punctuation.
        for &(_, ch) in &chars[end..] {
            out.push(Pretoken {
                text: ch.to_string(),
                sep: next_sep,
            });
            next_sep = "";
        }
    }

    out
}

/// Shared sliding-window match loop used by the three sentence-mode entry
/// points. `parser` returns `Some((replacement, score))` if the joined span
/// can be normalized.
fn sentence_loop<F>(pretokens: &[Pretoken], max_span_tokens: usize, parser: F) -> String
where
    F: Fn(&str) -> Option<(String, u8)>,
{
    let max_span = if max_span_tokens == 0 {
        1
    } else {
        max_span_tokens
    };

    let mut out = String::new();
    let mut i = 0usize;

    while i < pretokens.len() {
        let max_end = usize::min(pretokens.len(), i + max_span);
        let mut best: Option<(usize, String, u8)> = None;

        // Longest-span-first search keeps replacements stable and
        // non-overlapping. Reconstruct each span using the per-pretoken
        // separator so adjacency is preserved: pretokens that came from a
        // single original word (e.g. `"Dr."` -> `["Dr", "."]` with sep=`""`
        // on the period) re-emerge as `"Dr."`, not `"Dr ."`. This is what
        // the TN whitelist needs to match abbreviations like `"e.g."` /
        // `"Prof."` (PR #25 review feedback). Trying smaller `end` values
        // already handles "ignore trailing punctuation" cases like
        // `"twenty one,"` -> match `"twenty one"`.
        for end in (i + 1..=max_end).rev() {
            let mut span = String::new();
            for (idx, p) in pretokens[i..end].iter().enumerate() {
                if idx > 0 {
                    span.push_str(p.sep);
                }
                span.push_str(&p.text);
            }
            let Some((candidate, score)) = parser(&span) else {
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
            out.push_str(pretokens[i].sep);
            out.push_str(&replacement);
            i = end;
        } else {
            out.push_str(pretokens[i].sep);
            out.push_str(&pretokens[i].text);
            i += 1;
        }
    }

    out
}

/// Sentence-mode dispatch loop. The `concat_compound` and
/// `disable_bare_second` flags are forwarded to [`parse_span`] so each span
/// sees the right tagger priorities.
fn normalize_sentence_inner(
    input: &str,
    max_span_tokens: usize,
    concat_compound: bool,
    disable_bare_second: bool,
) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return trimmed.to_string();
    }

    let pretokens = pretokenize(trimmed);
    sentence_loop(&pretokens, max_span_tokens, |span| {
        parse_span(span, concat_compound, disable_bare_second)
    })
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
/// assert_eq!(result, "five dollars fifty cents");
/// ```
pub fn tn_normalize(input: &str) -> String {
    let input = input.trim();

    if let Some(result) = tn::en::whitelist::parse(input) {
        return result;
    }
    if let Some(result) = tn::en::math::parse(input) {
        return result;
    }
    if let Some(result) = tn::en::roman::parse(input) {
        return result;
    }
    if let Some(result) = tn::en::money::parse(input) {
        return result;
    }
    if let Some(result) = tn::en::measure::parse(input) {
        return result;
    }
    if let Some(result) = tn::en::date::parse(input) {
        return result;
    }
    if let Some(result) = tn::en::time::parse(input) {
        return result;
    }
    if let Some(result) = tn::en::fraction::parse(input) {
        return result;
    }
    if let Some(result) = tn::en::electronic::parse(input) {
        return result;
    }
    // Range before telephone: a typed range ("1980-1986") must not be read as
    // a phone-style digit group. Phone numbers fall through (range returns None).
    if let Some(result) = tn::en::range::parse(input) {
        return result;
    }
    if let Some(result) = tn::en::telephone::parse(input) {
        return result;
    }
    if let Some(result) = tn::en::ordinal::parse(input) {
        return result;
    }
    if let Some(result) = tn::en::decimal::parse(input) {
        return result;
    }
    if let Some(result) = tn::en::cardinal::parse(input) {
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

    if let Some(result) = tn::en::whitelist::parse(span) {
        return Some((result, 100));
    }
    if let Some(result) = tn::en::math::parse(span) {
        return Some((result, 96));
    }
    if let Some(result) = tn::en::roman::parse(span) {
        return Some((result, 94));
    }
    if let Some(result) = tn::en::money::parse(span) {
        return Some((result, 95));
    }
    if let Some(result) = tn::en::measure::parse(span) {
        return Some((result, 90));
    }
    if let Some(result) = tn::en::date::parse(span) {
        return Some((result, 88));
    }
    if let Some(result) = tn::en::time::parse(span) {
        return Some((result, 85));
    }
    if let Some(result) = tn::en::fraction::parse(span) {
        return Some((result, 81));
    }
    if let Some(result) = tn::en::electronic::parse(span) {
        return Some((result, 82));
    }
    // Range above telephone: a typed range must win over phone-style grouping.
    if let Some(result) = tn::en::range::parse(span) {
        return Some((result, 79));
    }
    if let Some(result) = tn::en::telephone::parse(span) {
        return Some((result, 78));
    }
    if let Some(result) = tn::en::ordinal::parse(span) {
        return Some((result, 75));
    }
    if let Some(result) = tn::en::decimal::parse(span) {
        return Some((result, 73));
    }
    if let Some(result) = tn::en::cardinal::parse(span) {
        return Some((result, 70));
    }
    // Last-resort spell-out for leftover symbol / alphanumeric tokens.
    if let Some(result) = tn::en::word::parse(span) {
        return Some((result, 60));
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

/// Normalize written-form text to spoken form for a specific language.
///
/// Supported languages: "en", "fr", "es", "de", "zh", "hi", "ja".
/// Falls back to English for unrecognized language codes.
///
/// ```
/// use text_processing_rs::tn_normalize_lang;
///
/// assert_eq!(tn_normalize_lang("123", "fr"), "cent vingt-trois");
/// assert_eq!(tn_normalize_lang("123", "en"), "one hundred and twenty three");
/// ```
pub fn tn_normalize_lang(input: &str, lang: &str) -> String {
    tn_normalize_for_lang(input, lang)
}

/// Normalize a full sentence (TN) for a specific language.
///
/// Supported languages: "en", "fr", "es", "de", "zh", "hi", "ja".
/// Falls back to English for unrecognized language codes.
pub fn tn_normalize_sentence_lang(input: &str, lang: &str) -> String {
    tn_normalize_sentence_with_max_span_lang(input, lang, DEFAULT_MAX_SPAN_TOKENS)
}

/// Normalize a full sentence (TN) for a specific language with configurable max span.
pub fn tn_normalize_sentence_with_max_span_lang(
    input: &str,
    lang: &str,
    max_span_tokens: usize,
) -> String {
    match lang {
        "en" | "" => tn_normalize_sentence_with_max_span(input, max_span_tokens),
        _ => {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                return trimmed.to_string();
            }

            let unified = unify_tn_quotes(trimmed);
            let pretokens = pretokenize(&unified);
            sentence_loop(&pretokens, max_span_tokens, |span| {
                tn_parse_span_lang(span, lang)
            })
        }
    }
}

/// Normalize a full sentence (TN) with a configurable max span size.
pub fn tn_normalize_sentence_with_max_span(input: &str, max_span_tokens: usize) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return trimmed.to_string();
    }

    let unified = unify_tn_quotes(trimmed);
    let pretokens = pretokenize(&unified);
    sentence_loop(&pretokens, max_span_tokens, tn_parse_span)
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
        assert_eq!(tn_normalize("123"), "one hundred and twenty three");
        assert_eq!(tn_normalize("0"), "zero");
        assert_eq!(tn_normalize("1000"), "one thousand");
    }

    #[test]
    fn test_tn_money() {
        assert_eq!(tn_normalize("$5.50"), "five dollars fifty cents");
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
        assert_eq!(tn_normalize("2:00 PM"), "two PM");
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
