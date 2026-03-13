//! Fraction tagger for Hindi.
//!
//! Converts Hindi fraction expressions to numeric form:
//! - "एक सौ नौ बटा एक सौ चौबीस" → "१०९/१२४"
//! - "एक सौ तैंतीस सही एक बटा दो" → "१३३ १/२"
//! - "डेढ़" → "१ १/२"
//! - "ढाई" → "२ १/२"
//! - "आधा" → "१/२"
//! - "सवा पैंतीस" → "३५ १/४"
//! - "तीन चौथाई" → "३/४"
//! - "साढ़े चार सौ बटा दस" → "४५०/१०"

use super::cardinal;

/// Check if the words starting at `start` contain a scale word.
fn has_scale_word(words: &[&str], start: usize) -> bool {
    for j in start..words.len() {
        if cardinal::scale_value(words[j]).is_some() {
            return true;
        }
        if !cardinal::is_hi_number_word(words[j]) && !cardinal::is_modifier(words[j]) {
            break;
        }
    }
    false
}

/// Check if word is a unit/currency/time marker that means this modifier is NOT a fraction context.
fn is_non_fraction_context(word: &str) -> bool {
    // Time markers
    if matches!(word, "बजे" | "बजकर" | "बजके" | "घंटा" | "घंटे")
    {
        return true;
    }
    // Measure/money context will be handled by those modules
    false
}

/// Process fraction patterns in a string.
pub fn process(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return input.to_string();
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < words.len() {
        // Check for standalone special fractions
        match words[i] {
            "आधा" => {
                result.push("१/२".to_string());
                i += 1;
                continue;
            }
            "पाव" => {
                result.push("१/४".to_string());
                i += 1;
                continue;
            }
            _ => {}
        }

        // Check for "X चौथाई" or "X तिहाई" patterns
        if i + 1 < words.len() {
            if let Some(n) = cardinal::word_to_value(words[i]) {
                if words[i + 1] == "चौथाई" {
                    result.push(format!("{}/४", cardinal::to_devanagari(n)));
                    i += 2;
                    continue;
                }
                if words[i + 1] == "तिहाई" {
                    result.push(format!("{}/३", cardinal::to_devanagari(n)));
                    i += 2;
                    continue;
                }
            }
        }

        // Check for "X सही Y बटा Z" pattern (mixed fraction) — BEFORE बटा
        if let Some((frac_str, consumed)) = try_parse_sahi_fraction(&words, i) {
            result.push(frac_str);
            i += consumed;
            continue;
        }

        // Check for "X बटा Y" pattern (simple fraction)
        // This handles modifier-led numerators too: "साढ़े चार सौ बटा दस" → "४५०/१०"
        if let Some((frac_str, consumed)) = try_parse_bata_fraction(&words, i) {
            result.push(frac_str);
            i += consumed;
            continue;
        }

        // Check for standalone modifier-based fractions
        // ONLY when the modifier is truly standalone (not followed by scale words or non-fraction context)
        if cardinal::is_modifier(words[i]) {
            if let Some((frac_str, consumed)) = try_parse_modifier_fraction(&words, i) {
                result.push(frac_str);
                i += consumed;
                continue;
            }
        }

        result.push(words[i].to_string());
        i += 1;
    }

    result.join(" ")
}

/// Try to parse a "X बटा Y" fraction.
fn try_parse_bata_fraction(words: &[&str], start: usize) -> Option<(String, usize)> {
    // Find "बटा" in the upcoming words
    let mut bata_pos = None;
    let max_look = (start + 12).min(words.len());

    for j in start..max_look {
        if words[j] == "बटा" {
            bata_pos = Some(j);
            break;
        }
        // Stop looking if we hit a non-number, non-modifier word
        if !cardinal::is_hi_number_word(words[j]) && !cardinal::is_modifier(words[j]) {
            break;
        }
    }

    let bata_pos = bata_pos?;

    // Parse numerator (before बटा)
    if bata_pos == start {
        return None; // No numerator
    }

    let num_words: Vec<&str> = words[start..bata_pos].to_vec();

    // Check if numerator words are valid (number words or modifiers)
    if !num_words
        .iter()
        .all(|w| cardinal::is_hi_number_word(w) || cardinal::is_modifier(w))
    {
        return None;
    }

    let numerator = cardinal::words_to_number(&num_words)?;

    // Parse denominator (after बटा)
    let denom_start = bata_pos + 1;
    let mut denom_end = denom_start;
    while denom_end < words.len()
        && (cardinal::is_hi_number_word(words[denom_end])
            || cardinal::is_modifier(words[denom_end]))
    {
        denom_end += 1;
    }

    if denom_end == denom_start {
        return None;
    }

    let denom_words: Vec<&str> = words[denom_start..denom_end].to_vec();
    let denominator = cardinal::words_to_number(&denom_words)?;

    let frac_str = format!(
        "{}/{}",
        cardinal::to_devanagari(numerator),
        cardinal::to_devanagari(denominator)
    );
    Some((frac_str, denom_end - start))
}

/// Try to parse a "X सही Y बटा Z" mixed fraction.
fn try_parse_sahi_fraction(words: &[&str], start: usize) -> Option<(String, usize)> {
    // Find "सही" in the upcoming words
    let mut sahi_pos = None;
    let max_look = (start + 12).min(words.len());

    for j in start..max_look {
        if words[j] == "सही" {
            sahi_pos = Some(j);
            break;
        }
        if !cardinal::is_hi_number_word(words[j]) && !cardinal::is_modifier(words[j]) {
            break;
        }
    }

    let sahi_pos = sahi_pos?;

    if sahi_pos == start {
        return None;
    }

    // Parse whole number (before सही)
    let whole_words: Vec<&str> = words[start..sahi_pos].to_vec();
    if !whole_words
        .iter()
        .all(|w| cardinal::is_hi_number_word(w) || cardinal::is_modifier(w))
    {
        return None;
    }
    let whole = cardinal::words_to_number(&whole_words)?;

    // After सही, expect "Y बटा Z"
    let frac_start = sahi_pos + 1;
    if let Some((frac_str, consumed)) = try_parse_bata_fraction(words, frac_start) {
        let result = format!("{} {}", cardinal::to_devanagari(whole), frac_str);
        return Some((result, sahi_pos - start + 1 + consumed));
    }

    None
}

/// Try to parse modifier-based fractions.
/// Only handles truly standalone modifiers (not followed by scale words or non-fraction context).
/// - "डेढ़" (alone or followed by non-number) → "१ १/२"
/// - "ढाई" (alone or followed by non-number) → "२ १/२"
/// - "सवा X" (X has no scale word) → "X १/४"
/// - "साढ़े X" (X has no scale word) → "X १/२"
/// - "पौने X" (X has no scale word) → "(X-1) ३/४"
fn try_parse_modifier_fraction(words: &[&str], start: usize) -> Option<(String, usize)> {
    let modifier = words[start];

    match modifier {
        "डेढ़" => {
            // Only standalone — NOT followed by scale word or number+scale
            if start + 1 < words.len() {
                let next = words[start + 1];
                // If followed by a number word or scale word, let cardinal/money/measure handle it
                if cardinal::is_hi_number_word(next)
                    || cardinal::is_modifier(next)
                    || is_non_fraction_context(next)
                {
                    return None;
                }
            }
            Some(("१ १/२".to_string(), 1))
        }
        "ढाई" => {
            if start + 1 < words.len() {
                let next = words[start + 1];
                if cardinal::is_hi_number_word(next)
                    || cardinal::is_modifier(next)
                    || is_non_fraction_context(next)
                {
                    return None;
                }
            }
            Some(("२ १/२".to_string(), 1))
        }
        "सवा" => {
            // सवा + number (no scale) → "N 1/4"
            if start + 1 < words.len() {
                // If the following number words contain a scale word, let cardinal handle it
                if has_scale_word(words, start + 1) {
                    return None;
                }
                // If followed by time/money context, skip
                if is_non_fraction_context(words[start + 1]) {
                    return None;
                }
                // Collect number words
                let mut end = start + 1;
                while end < words.len() && cardinal::is_hi_number_word(words[end]) {
                    end += 1;
                }
                if end > start + 1 {
                    let num_words: Vec<&str> = words[start + 1..end].to_vec();
                    if let Some(val) = cardinal::words_to_number(&num_words) {
                        return Some((
                            format!("{} १/४", cardinal::to_devanagari(val)),
                            end - start,
                        ));
                    }
                }
            }
            // सवा alone at end of input
            Some(("१/४".to_string(), 1))
        }
        "साढ़े" => {
            if start + 1 < words.len() {
                // If the following number words contain a scale word, let cardinal handle it
                if has_scale_word(words, start + 1) {
                    return None;
                }
                if is_non_fraction_context(words[start + 1]) {
                    return None;
                }
                // Collect number words
                let mut end = start + 1;
                while end < words.len() && cardinal::is_hi_number_word(words[end]) {
                    end += 1;
                }
                if end > start + 1 {
                    let num_words: Vec<&str> = words[start + 1..end].to_vec();
                    if let Some(val) = cardinal::words_to_number(&num_words) {
                        return Some((
                            format!("{} १/२", cardinal::to_devanagari(val)),
                            end - start,
                        ));
                    }
                }
            }
            // साढ़े alone
            Some(("१/२".to_string(), 1))
        }
        "पौन" | "पौना" | "पौने" => {
            if start + 1 < words.len() {
                // If the following number words contain a scale word, let cardinal handle it
                if has_scale_word(words, start + 1) {
                    return None;
                }
                if is_non_fraction_context(words[start + 1]) {
                    return None;
                }
                // Collect number words
                let mut end = start + 1;
                while end < words.len() && cardinal::is_hi_number_word(words[end]) {
                    end += 1;
                }
                if end > start + 1 {
                    let num_words: Vec<&str> = words[start + 1..end].to_vec();
                    if let Some(val) = cardinal::words_to_number(&num_words) {
                        let whole = val - 1;
                        return Some((
                            format!("{} ३/४", cardinal::to_devanagari(whole)),
                            end - start,
                        ));
                    }
                }
            }
            // पौन/पौना alone
            Some(("३/४".to_string(), 1))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bata() {
        assert_eq!(process("एक सौ नौ बटा एक सौ चौबीस"), "१०९/१२४");
        assert_eq!(process("एक सौ एक बटा दो"), "१०१/२");
    }

    #[test]
    fn test_sahi() {
        assert_eq!(process("एक सौ तैंतीस सही एक बटा दो"), "१३३ १/२");
    }

    #[test]
    fn test_standalone() {
        assert_eq!(process("डेढ़"), "१ १/२");
        assert_eq!(process("ढाई"), "२ १/२");
        assert_eq!(process("आधा"), "१/२");
    }
}
