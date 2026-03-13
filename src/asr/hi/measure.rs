//! Measure tagger for Hindi.
//!
//! Converts Hindi measurement expressions to numeric form:
//! - "दो सौ छह ग्राम" → "२०६ g"
//! - "दो सौ छह दशमलव दो नौ ग्राम" → "२०६.२९ g"
//! - "दो बाई दो" → "२x२"
//! - "साढ़े सात वर्ष" → "७.५ yr"
//! - "पौने ग्यारह घंटे" → "१०.७५ h"
//! - "डेढ़ दर्जन" → "१.५ doz"

use super::cardinal;

/// Unit mappings: (Hindi name variants, symbol)
const UNITS: &[(&[&str], &str)] = &[
    (&["वर्गसेंटीमीटर", "वर्ग सेंटीमीटर"], "cm²"),
    (&["क्यूबिकमिलीमीटर", "क्यूबिक मिलीमीटर", "घन मिलीमीटर"], "mm³"),
    (&["वर्ग माइक्रोमीटर"], "µm²"),
    (&["घन फीट", "घनफीट"], "ft³"),
    (&["किलोमीटर प्रति घंटा"], "km/h"),
    (&["मील प्रति घंटा"], "mi/h"),
    (&["मीट्रिक टन"], "t"),
    (&["मिलीमीटर"], "mm"),
    (&["मिलिग्राम"], "mg"),
    (&["माइक्रॉन"], "µm"),
    (&["सेल्सियस"], "°C"),
    (&["डेसिग्राम"], "dg"),
    (&["कैल्विन"], "K"),
    (&["किलोमीटर"], "km"),
    (&["हेक्टेयर"], "ha"),
    (&["ऐंपीयर"], "A"),
    (&["गैलन"], "gal"),
    (&["महीने", "महीना"], "mo"),
    (&["दर्जन"], "doz"),
    (&["लीटर"], "L"),
    (&["पिंट"], "pt"),
    (&["ग्राम"], "g"),
    (&["इंच"], "in"),
    (&["फुट"], "ft"),
    (&["एकड़"], "ac"),
    (&["किग्रा"], "kg"),
    (&["मीटर"], "m"),
    (&["वर्ष"], "yr"),
    (&["घंटे", "घंटा"], "h"),
];

/// Process measure patterns in a string.
pub fn process(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return input.to_string();
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < words.len() {
        // Check for "X बाई Y" (dimension) pattern
        if let Some((dim_str, consumed)) = try_parse_dimension(&words, i) {
            result.push(dim_str);
            i += consumed;
            continue;
        }

        // Check for number + unit pattern
        if let Some((measure_str, consumed)) = try_parse_measure(&words, i) {
            result.push(measure_str);
            i += consumed;
            continue;
        }

        result.push(words[i].to_string());
        i += 1;
    }

    result.join(" ")
}

/// Try to parse a measurement expression.
fn try_parse_measure(words: &[&str], start: usize) -> Option<(String, usize)> {
    // Find a unit within reasonable range after number words
    let max_look = (start + 15).min(words.len());

    for end in start..max_look {
        // Try matching unit names starting at position `end`
        for &(names, symbol) in UNITS {
            for &name in names {
                let name_words: Vec<&str> = name.split_whitespace().collect();
                let name_len = name_words.len();

                if end + name_len > words.len() {
                    continue;
                }

                let matches = name_words.iter().enumerate().all(|(j, &nw)| words[end + j] == nw);
                if !matches {
                    continue;
                }

                // Found unit at end..end+name_len
                // Parse number before it
                let span = &words[start..end];
                if span.is_empty() {
                    continue;
                }

                // Check for दशमलव (decimal)
                let dashm_pos = span.iter().position(|&w| w == "दशमलव");

                if let Some(dp) = dashm_pos {
                    let int_words = &span[..dp];
                    let frac_words = &span[dp + 1..];

                    if int_words.is_empty() || !int_words.iter().all(|w| cardinal::is_hi_number_word(w) || cardinal::is_modifier(w)) {
                        continue;
                    }

                    let int_val = cardinal::words_to_number(&int_words.to_vec())?;

                    let frac_digits: Vec<i64> = frac_words
                        .iter()
                        .filter_map(|w| cardinal::word_to_value(w).filter(|&v| v <= 9))
                        .collect();

                    if frac_digits.len() != frac_words.len() {
                        continue;
                    }

                    let int_str = cardinal::to_devanagari(int_val);
                    let frac_str: String = frac_digits
                        .iter()
                        .map(|&d| cardinal::to_devanagari_digit(d as u8))
                        .collect();

                    let result = format!("{}.{} {}", int_str, frac_str, symbol);
                    return Some((result, end + name_len - start));
                }

                // No decimal — check for modifiers that produce decimals
                if !span.iter().all(|w| cardinal::is_hi_number_word(w) || cardinal::is_modifier(w)) {
                    continue;
                }

                // Check if modifier produces a decimal result
                if let Some(measure_str) = try_modifier_measure(span, symbol) {
                    return Some((measure_str, end + name_len - start));
                }

                // Plain number
                let num_words: Vec<&str> = span.to_vec();
                let val = cardinal::words_to_number(&num_words)?;
                let result = format!("{} {}", cardinal::to_devanagari(val), symbol);
                return Some((result, end + name_len - start));
            }
        }
    }

    None
}

/// Handle modifier-based measures that produce decimal output.
/// Uses find_lowest_scale to correctly apply modifiers to the scale, not the total.
/// e.g., "साढ़े सात" + yr → "७.५ yr"
/// "पौने ग्यारह" + h → "१०.७५ h"
/// "डेढ़" + doz → "१.५ doz"
/// "ढाई" + mo → "२.५ mo"
fn try_modifier_measure(span: &[&str], symbol: &str) -> Option<String> {
    if span.is_empty() {
        return None;
    }

    let modifier = span[0];
    if !cardinal::is_modifier(modifier) {
        return None;
    }

    let rest = &span[1..];

    match modifier {
        "डेढ़" => {
            if rest.is_empty() {
                return Some(format!("१.५ {}", symbol));
            }
            let base = cardinal::words_to_number(&rest.to_vec())?;
            let lowest = cardinal::find_lowest_scale(rest);
            let result = base + lowest / 2;
            return format_measure_result(result as f64, lowest as f64 / 2.0, symbol);
        }
        "ढाई" => {
            if rest.is_empty() {
                return Some(format!("२.५ {}", symbol));
            }
            let base = cardinal::words_to_number(&rest.to_vec())?;
            let lowest = cardinal::find_lowest_scale(rest);
            let result = base + lowest + lowest / 2;
            return format_measure_result(result as f64, (lowest + lowest / 2) as f64, symbol);
        }
        "साढ़े" => {
            if rest.is_empty() {
                return None;
            }
            let base = cardinal::words_to_number(&rest.to_vec())?;
            let lowest = cardinal::find_lowest_scale(rest);
            let half = lowest as f64 / 2.0;
            let result = base as f64 + half;
            return format_measure_decimal(result, symbol);
        }
        "सवा" => {
            if rest.is_empty() {
                return None;
            }
            let base = cardinal::words_to_number(&rest.to_vec())?;
            let lowest = cardinal::find_lowest_scale(rest);
            let quarter = lowest as f64 / 4.0;
            let result = base as f64 + quarter;
            return format_measure_decimal(result, symbol);
        }
        "पौने" | "पौन" | "पौना" => {
            if rest.is_empty() {
                return None;
            }
            let base = cardinal::words_to_number(&rest.to_vec())?;
            let lowest = cardinal::find_lowest_scale(rest);
            let quarter = lowest as f64 / 4.0;
            let result = base as f64 - quarter;
            return format_measure_decimal(result, symbol);
        }
        _ => None,
    }
}

/// Format a measure result as decimal or integer.
fn format_measure_decimal(result: f64, symbol: &str) -> Option<String> {
    if result == result.floor() {
        Some(format!("{} {}", cardinal::to_devanagari(result as i64), symbol))
    } else {
        let formatted = format!("{:.2}", result);
        let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
        Some(format!("{} {}", cardinal::to_devanagari_str(trimmed), symbol))
    }
}

fn format_measure_result(result: f64, _fraction: f64, symbol: &str) -> Option<String> {
    format_measure_decimal(result, symbol)
}

/// Try to parse a "X बाई Y" dimension pattern.
fn try_parse_dimension(words: &[&str], start: usize) -> Option<(String, usize)> {
    // Find "बाई" in upcoming words
    let max_look = (start + 8).min(words.len());

    for j in start..max_look {
        if words[j] == "बाई" {
            // Parse X before बाई
            let x_words: Vec<&str> = words[start..j].to_vec();
            if x_words.is_empty() || !x_words.iter().all(|w| cardinal::is_hi_number_word(w)) {
                continue;
            }
            let x = cardinal::words_to_number(&x_words)?;

            // Parse Y after बाई
            let mut y_end = j + 1;
            while y_end < words.len() && cardinal::is_hi_number_word(words[y_end]) {
                y_end += 1;
            }
            if y_end == j + 1 {
                continue;
            }
            let y_words: Vec<&str> = words[j + 1..y_end].to_vec();
            let y = cardinal::words_to_number(&y_words)?;

            // Check for trailing unit
            let mut unit_str = String::new();
            let mut final_end = y_end;
            if y_end < words.len() {
                for &(names, symbol) in UNITS {
                    for &name in names {
                        let name_words: Vec<&str> = name.split_whitespace().collect();
                        let name_len = name_words.len();
                        if y_end + name_len <= words.len() {
                            let matches = name_words.iter().enumerate().all(|(k, &nw)| words[y_end + k] == nw);
                            if matches {
                                unit_str = format!(" {}", symbol);
                                final_end = y_end + name_len;
                                break;
                            }
                        }
                    }
                    if !unit_str.is_empty() {
                        break;
                    }
                }
            }

            let dim = format!("{}x{}{}", cardinal::to_devanagari(x), cardinal::to_devanagari(y), unit_str);
            return Some((dim, final_end - start));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(process("दो सौ छह ग्राम"), "२०६ g");
    }

    #[test]
    fn test_decimal_measure() {
        assert_eq!(process("दो सौ छह दशमलव दो नौ ग्राम"), "२०६.२९ g");
    }

    #[test]
    fn test_dimension() {
        assert_eq!(process("दो बाई दो"), "२x२");
    }
}
