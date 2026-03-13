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

use asr::en::{
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
    if let Some(result) = asr::fr::whitelist::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::fr::punctuation::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::fr::word::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::fr::time::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::fr::date::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::fr::money::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::fr::measure::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::fr::electronic::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::fr::ordinal::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::fr::decimal::parse(input) {
        return Some(result);
    }
    if let Some(num) = asr::fr::cardinal::parse(input) {
        return Some(num);
    }
    // Telephone last since it can match numbers
    if let Some(result) = asr::fr::telephone::parse(input) {
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
    if let Some(num) = asr::fr::cardinal::words_to_number(&first_lower) {
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
    if let Some(result) = asr::de::whitelist::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::de::punctuation::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::de::time::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::de::date::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::de::money::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::de::measure::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::de::electronic::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::de::ordinal::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::de::fraction::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::de::decimal::parse(input) {
        return Some(result);
    }
    if let Some(num) = asr::de::cardinal::parse(input) {
        return Some(num);
    }
    // Telephone last since it can match digit sequences
    if let Some(result) = asr::de::telephone::parse(input) {
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
    if let Some(result) = asr::es::whitelist::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::es::punctuation::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::es::word::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::es::time::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::es::date::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::es::money::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::es::measure::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::es::electronic::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::es::ordinal::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::es::fraction::parse(input) {
        return Some(result);
    }
    if let Some(result) = asr::es::decimal::parse(input) {
        return Some(result);
    }
    if let Some(num) = asr::es::cardinal::parse(input) {
        return Some(num);
    }
    // Telephone last since it can match digit sequences
    if let Some(result) = asr::es::telephone::parse(input) {
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
    result = asr::hi::whitelist::process(&result);

    // 2. Money (number + currency name → symbol + digits)
    result = asr::hi::money::process(&result);

    // 3. Date (day + month [+ year], ranges, eras)
    result = asr::hi::date::process(&result);

    // 4. Time (X बजे/घंटा + मिनट/सेकंड)
    // Before measure so "X घंटा Y मिनट" isn't caught as measure
    result = asr::hi::time::process(&result);

    // 5. Measure (number + unit → digits + symbol)
    result = asr::hi::measure::process(&result);

    // 6. Fractions (X बटा Y, X सही Y बटा Z)
    result = asr::hi::fraction::process(&result);

    // 7. Ordinal (Xवां, Xवीं, Xवें)
    result = asr::hi::ordinal::process(&result);

    // 8. Decimal (X दशमलव Y)
    result = asr::hi::decimal::process(&result);

    // 9. Cardinal — convert compound number words (with scale words) and
    //    single number words to Devanagari digits. Must run BEFORE
    //    telephone/address so compound numbers like "एक सौ" are grouped.
    result = asr::hi::cardinal::process(&result);

    // 10. Telephone (digit-by-digit sequences ≥ 4 Devanagari digits)
    result = asr::hi::telephone::process(&result);

    // 11. Address (digit-by-digit with हाइफ़न/बटा, comma-separated digits)
    result = asr::hi::address::process(&result);

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
    result = asr::ja::fraction::process(&result);

    // 2. Decimals (X点Y) — before cardinal swallows the kanji
    result = asr::ja::decimal::process(&result);

    // 3. Dates (年月日, 世紀, 年代, weekdays, ranges)
    result = asr::ja::date::process(&result);

    // 4. Time (時, 分) — after fractions to avoid 分の collision
    result = asr::ja::time::process(&result);

    // 5. Ordinals (番目, 第)
    result = asr::ja::ordinal::process(&result);

    // 6. Cardinal — catch remaining standalone kanji number spans
    result = asr::ja::cardinal::replace_kanji_numbers(&result);

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
    result = asr::zh::whitelist::process(&result);

    // 2. Money (before decimal to catch currency-specific decimal patterns like 一点五万美元)
    result = asr::zh::money::process(&result);

    // 3. Fractions (X分之Y) — before time which also uses 分
    result = asr::zh::fraction::process(&result);

    // 4. Time (X点Y分, X分钟, X秒钟) — before decimal so 点 with 分/刻/半 isn't consumed as decimal
    result = asr::zh::time::process(&result);

    // 5. Decimals (X点Y)
    result = asr::zh::decimal::process(&result);

    // 6. Dates (年月日, 公元/纪元)
    result = asr::zh::date::process(&result);

    // 7. Ordinals (第X)
    result = asr::zh::ordinal::process(&result);

    // 8. Cardinal — catch remaining standalone Chinese number spans
    result = asr::zh::cardinal::replace_zh_numbers(&result);

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
    if let Some(r) = tts::fr::whitelist::parse(input) {
        return r;
    }
    if let Some(r) = tts::fr::money::parse(input) {
        return r;
    }
    if let Some(r) = tts::fr::measure::parse(input) {
        return r;
    }
    if let Some(r) = tts::fr::date::parse(input) {
        return r;
    }
    if let Some(r) = tts::fr::time::parse(input) {
        return r;
    }
    if let Some(r) = tts::fr::electronic::parse(input) {
        return r;
    }
    if let Some(r) = tts::fr::telephone::parse(input) {
        return r;
    }
    if let Some(r) = tts::fr::ordinal::parse(input) {
        return r;
    }
    if let Some(r) = tts::fr::decimal::parse(input) {
        return r;
    }
    if let Some(r) = tts::fr::cardinal::parse(input) {
        return r;
    }
    input.to_string()
}

fn tn_normalize_lang_es(input: &str) -> String {
    if let Some(r) = tts::es::whitelist::parse(input) {
        return r;
    }
    if let Some(r) = tts::es::money::parse(input) {
        return r;
    }
    if let Some(r) = tts::es::measure::parse(input) {
        return r;
    }
    if let Some(r) = tts::es::date::parse(input) {
        return r;
    }
    if let Some(r) = tts::es::time::parse(input) {
        return r;
    }
    if let Some(r) = tts::es::electronic::parse(input) {
        return r;
    }
    if let Some(r) = tts::es::telephone::parse(input) {
        return r;
    }
    if let Some(r) = tts::es::ordinal::parse(input) {
        return r;
    }
    if let Some(r) = tts::es::decimal::parse(input) {
        return r;
    }
    if let Some(r) = tts::es::cardinal::parse(input) {
        return r;
    }
    input.to_string()
}

fn tn_normalize_lang_de(input: &str) -> String {
    if let Some(r) = tts::de::whitelist::parse(input) {
        return r;
    }
    if let Some(r) = tts::de::money::parse(input) {
        return r;
    }
    if let Some(r) = tts::de::measure::parse(input) {
        return r;
    }
    if let Some(r) = tts::de::date::parse(input) {
        return r;
    }
    if let Some(r) = tts::de::time::parse(input) {
        return r;
    }
    if let Some(r) = tts::de::electronic::parse(input) {
        return r;
    }
    if let Some(r) = tts::de::telephone::parse(input) {
        return r;
    }
    if let Some(r) = tts::de::ordinal::parse(input) {
        return r;
    }
    if let Some(r) = tts::de::decimal::parse(input) {
        return r;
    }
    if let Some(r) = tts::de::cardinal::parse(input) {
        return r;
    }
    input.to_string()
}

fn tn_normalize_lang_zh(input: &str) -> String {
    if let Some(r) = tts::zh::whitelist::parse(input) {
        return r;
    }
    if let Some(r) = tts::zh::money::parse(input) {
        return r;
    }
    if let Some(r) = tts::zh::measure::parse(input) {
        return r;
    }
    if let Some(r) = tts::zh::date::parse(input) {
        return r;
    }
    if let Some(r) = tts::zh::time::parse(input) {
        return r;
    }
    if let Some(r) = tts::zh::electronic::parse(input) {
        return r;
    }
    if let Some(r) = tts::zh::telephone::parse(input) {
        return r;
    }
    if let Some(r) = tts::zh::ordinal::parse(input) {
        return r;
    }
    if let Some(r) = tts::zh::decimal::parse(input) {
        return r;
    }
    if let Some(r) = tts::zh::cardinal::parse(input) {
        return r;
    }
    input.to_string()
}

fn tn_normalize_lang_hi(input: &str) -> String {
    if let Some(r) = tts::hi::whitelist::parse(input) {
        return r;
    }
    if let Some(r) = tts::hi::money::parse(input) {
        return r;
    }
    if let Some(r) = tts::hi::measure::parse(input) {
        return r;
    }
    if let Some(r) = tts::hi::date::parse(input) {
        return r;
    }
    if let Some(r) = tts::hi::time::parse(input) {
        return r;
    }
    if let Some(r) = tts::hi::electronic::parse(input) {
        return r;
    }
    if let Some(r) = tts::hi::telephone::parse(input) {
        return r;
    }
    if let Some(r) = tts::hi::ordinal::parse(input) {
        return r;
    }
    if let Some(r) = tts::hi::decimal::parse(input) {
        return r;
    }
    if let Some(r) = tts::hi::cardinal::parse(input) {
        return r;
    }
    input.to_string()
}

fn tn_normalize_lang_ja(input: &str) -> String {
    if let Some(r) = tts::ja::whitelist::parse(input) {
        return r;
    }
    if let Some(r) = tts::ja::money::parse(input) {
        return r;
    }
    if let Some(r) = tts::ja::measure::parse(input) {
        return r;
    }
    if let Some(r) = tts::ja::date::parse(input) {
        return r;
    }
    if let Some(r) = tts::ja::time::parse(input) {
        return r;
    }
    if let Some(r) = tts::ja::electronic::parse(input) {
        return r;
    }
    if let Some(r) = tts::ja::telephone::parse(input) {
        return r;
    }
    if let Some(r) = tts::ja::ordinal::parse(input) {
        return r;
    }
    if let Some(r) = tts::ja::decimal::parse(input) {
        return r;
    }
    if let Some(r) = tts::ja::cardinal::parse(input) {
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
            try_lang_taggers!(tts::en);
        }
        "fr" => {
            try_lang_taggers!(tts::fr);
        }
        "es" => {
            try_lang_taggers!(tts::es);
        }
        "de" => {
            try_lang_taggers!(tts::de);
        }
        "zh" => {
            try_lang_taggers!(tts::zh);
        }
        "hi" => {
            try_lang_taggers!(tts::hi);
        }
        "ja" => {
            try_lang_taggers!(tts::ja);
        }
        _ => {
            return tn_parse_span(span);
        }
    }

    None
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

    if let Some(result) = tts::en::whitelist::parse(input) {
        return result;
    }
    if let Some(result) = tts::en::money::parse(input) {
        return result;
    }
    if let Some(result) = tts::en::measure::parse(input) {
        return result;
    }
    if let Some(result) = tts::en::date::parse(input) {
        return result;
    }
    if let Some(result) = tts::en::time::parse(input) {
        return result;
    }
    if let Some(result) = tts::en::electronic::parse(input) {
        return result;
    }
    if let Some(result) = tts::en::telephone::parse(input) {
        return result;
    }
    if let Some(result) = tts::en::ordinal::parse(input) {
        return result;
    }
    if let Some(result) = tts::en::decimal::parse(input) {
        return result;
    }
    if let Some(result) = tts::en::cardinal::parse(input) {
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

    if let Some(result) = tts::en::whitelist::parse(span) {
        return Some((result, 100));
    }
    if let Some(result) = tts::en::money::parse(span) {
        return Some((result, 95));
    }
    if let Some(result) = tts::en::measure::parse(span) {
        return Some((result, 90));
    }
    if let Some(result) = tts::en::date::parse(span) {
        return Some((result, 88));
    }
    if let Some(result) = tts::en::time::parse(span) {
        return Some((result, 85));
    }
    if let Some(result) = tts::en::electronic::parse(span) {
        return Some((result, 82));
    }
    if let Some(result) = tts::en::telephone::parse(span) {
        return Some((result, 78));
    }
    if let Some(result) = tts::en::ordinal::parse(span) {
        return Some((result, 75));
    }
    if let Some(result) = tts::en::decimal::parse(span) {
        return Some((result, 73));
    }
    if let Some(result) = tts::en::cardinal::parse(span) {
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

/// Normalize written-form text to spoken form for a specific language.
///
/// Supported languages: "en", "fr", "es", "de", "zh", "hi", "ja".
/// Falls back to English for unrecognized language codes.
///
/// ```
/// use text_processing_rs::tn_normalize_lang;
///
/// assert_eq!(tn_normalize_lang("123", "fr"), "cent vingt-trois");
/// assert_eq!(tn_normalize_lang("123", "en"), "one hundred twenty three");
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
                    let Some((candidate, score)) = tn_parse_span_lang(&span, lang) else {
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
    }
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
