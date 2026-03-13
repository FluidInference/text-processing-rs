//! Money tagger for Hindi.
//!
//! Converts Hindi currency expressions to symbolic form:
//! - "बारह हज़ार तेरह डॉलर" → "$१२०१३"
//! - "दो सौ छह रुपये दो सौ छह पैसे" → "₹२०६.२०६"
//! - "साढ़े सात सौ डॉलर" → "$७५०"
//! - "ढाई करोड़ रुपए" → "₹२५००००००"

use super::cardinal;

/// Currency mappings: (Hindi names, symbol)
/// Multiple Hindi names can map to the same symbol.
/// Longer names listed first to avoid partial matches.
const CURRENCIES: &[(&[&str], &str)] = &[
    (&["अल्जीरियाई दिनार"], "دج"),
    (&["बेलारूसी रूबल"], "br"),
    (&["चीनी युआन"], "元"),
    (&["आर्मेनियाई ड्राम"], "֏"),
    (&["अरूबान फ्लोरिन"], "ƒ"),
    (&["त्रिनिदाद और टोबैगो डॉलर"], "tt$"),
    (&["तुर्की लिरा"], "₺"),
    (&["युगांडा शिलिंग"], "ush"),
    (&["यूक्रेनी ग्रिव्ना"], "₴"),
    (&["वेनेजुएलन बोलिवार"], "bs."),
    (&["साइप्रस पाउंड"], "cyp"),
    (&["बहरीन दिरहम"], ".د.ب"),
    (&["अजरबैजानी मनात"], "₼"),
    (&["बुरुंडी फ्रैंक"], "fbu"),
    (&["कैमन आइलैंड्स डॉलर"], "ci$"),
    (&["लिलांगेनी"], "l"),
    (&["बिटकॉइन"], "₿"),
    (&["वॉन"], "₩"),
    (&["लीरा"], "₺"),
    (&["यूरो"], "€"),
    (&["डॉलर"], "$"),
    (&["रुपये", "रुपए", "रुपिया", "रुपेया"], "₹"),
    (&["पैसे", "पैसा"], "p"),
];

/// Process money patterns in a string.
pub fn process(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return input.to_string();
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < words.len() {
        // Try to find a currency name starting at various positions
        if let Some((money_str, consumed)) = try_parse_money(&words, i) {
            // Remove any number words we already added to result
            result.push(money_str);
            i += consumed;
            continue;
        }

        result.push(words[i].to_string());
        i += 1;
    }

    result.join(" ")
}

/// Try to parse a money expression starting at or before position `start`.
fn try_parse_money(words: &[&str], start: usize) -> Option<(String, usize)> {
    // Scan forward from `start` looking for a currency name
    // The pattern is: [number words] [दशमलव digit-words] currency_name
    // or: [number words] currency_name [number words] [पैसे/पैसा unit]

    // First, try to find a currency name within a reasonable range
    let max_look = (start + 20).min(words.len());

    for end in start..max_look {
        // Try matching currency names starting at position `end`
        for &(names, symbol) in CURRENCIES {
            for &name in names {
                let name_words: Vec<&str> = name.split_whitespace().collect();
                let name_len = name_words.len();

                if end + name_len > words.len() {
                    continue;
                }

                // Check if words match the currency name
                let matches = name_words
                    .iter()
                    .enumerate()
                    .all(|(j, &nw)| words[end + j] == nw);
                if !matches {
                    continue;
                }

                // Found a currency at position end..end+name_len
                // Now parse the number before it
                let (num_start, amount, has_decimal) = parse_money_amount(words, start, end);

                if num_start != start {
                    // Not starting at our position
                    continue;
                }

                if let Some(amount_str) = amount {
                    // Special handling for रुपये + पैसे pattern
                    if symbol == "₹" {
                        let after_currency = end + name_len;
                        // Direct: "X रुपये Y पैसे"
                        if let Some((paise_str, paise_consumed)) =
                            try_parse_paise(words, after_currency)
                        {
                            let money = format!("₹{}.{}", amount_str, paise_str);
                            return Some((money, end + name_len + paise_consumed - start));
                        }
                        // With और: "X रुपेया और Y पैसा"
                        if after_currency < words.len() && words[after_currency] == "और" {
                            if let Some((paise_str, paise_consumed)) =
                                try_parse_paise(words, after_currency + 1)
                            {
                                let money = format!("₹{}.{}", amount_str, paise_str);
                                return Some((money, end + name_len + 1 + paise_consumed - start));
                            }
                        }
                    }

                    // Check if this is a पैसे amount (separate from rupees)
                    if symbol == "p" {
                        let money = format!("p{}", amount_str);
                        return Some((money, end + name_len - start));
                    }

                    let money = if has_decimal {
                        format!("{}{}", symbol, amount_str)
                    } else {
                        format!("{}{}", symbol, amount_str)
                    };
                    return Some((money, end + name_len - start));
                }
            }
        }
    }

    None
}

/// Parse the money amount (number + optional दशमलव digits) before a currency name.
/// Returns (actual_start, formatted_amount, has_decimal).
fn parse_money_amount(
    words: &[&str],
    start: usize,
    currency_pos: usize,
) -> (usize, Option<String>, bool) {
    if currency_pos <= start {
        return (start, None, false);
    }

    // Check for "दशमलव" in the span
    let span = &words[start..currency_pos];

    // Find "दशमलव" position
    let dashm_pos = span.iter().position(|&w| w == "दशमलव");

    if let Some(dp) = dashm_pos {
        // Integer part before दशमलव
        let int_words = &span[..dp];
        let frac_words = &span[dp + 1..];

        if int_words.is_empty() {
            return (start, None, false);
        }

        // Check all int_words are number words or modifiers
        if !int_words
            .iter()
            .all(|w| cardinal::is_hi_number_word(w) || cardinal::is_modifier(w))
        {
            return (start, None, false);
        }

        let int_val = match cardinal::words_to_number(&int_words.to_vec()) {
            Some(v) => v,
            None => return (start, None, false),
        };

        // Parse fractional digits individually
        let frac_digits: Vec<i64> = frac_words
            .iter()
            .filter_map(|w| cardinal::word_to_value(w).filter(|&v| v <= 9))
            .collect();

        if frac_digits.len() != frac_words.len() {
            return (start, None, false);
        }

        let int_str = cardinal::to_devanagari(int_val);
        let frac_str: String = frac_digits
            .iter()
            .map(|&d| cardinal::to_devanagari_digit(d as u8))
            .collect();

        return (start, Some(format!("{}.{}", int_str, frac_str)), true);
    }

    // No decimal — just a number
    let num_words: Vec<&str> = span.to_vec();
    if !num_words
        .iter()
        .all(|w| cardinal::is_hi_number_word(w) || cardinal::is_modifier(w))
    {
        return (start, None, false);
    }

    let val = match cardinal::words_to_number(&num_words) {
        Some(v) => v,
        None => return (start, None, false),
    };

    (start, Some(cardinal::to_devanagari(val).to_string()), false)
}

/// Try to parse a पैसे/पैसा amount after the main currency.
/// Pattern: number_words "पैसे"/"पैसा"
fn try_parse_paise(words: &[&str], start: usize) -> Option<(String, usize)> {
    if start >= words.len() {
        return None;
    }

    let mut end = start;
    while end < words.len()
        && (cardinal::is_hi_number_word(words[end])
            || cardinal::is_modifier(words[end])
            || words[end] == "दशमलव")
    {
        end += 1;
    }

    if end == start || end >= words.len() {
        return None;
    }

    // Must be followed by पैसे/पैसा
    if words[end] != "पैसे" && words[end] != "पैसा" {
        return None;
    }

    let num_words: Vec<&str> = words[start..end].to_vec();
    let val = cardinal::words_to_number(&num_words)?;
    let result = cardinal::to_devanagari(val).to_string();

    Some((result, end + 1 - start))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(process("बारह हज़ार तेरह डॉलर"), "$१२०१३");
        assert_eq!(process("छियासठ तुर्की लिरा"), "₺६६");
    }

    #[test]
    fn test_decimal() {
        assert_eq!(process("बाईस दशमलव शून्य पाँच यूक्रेनी ग्रिव्ना"), "₴२२.०५");
    }

    #[test]
    fn test_modifier() {
        assert_eq!(process("डेढ़ सौ यूरो"), "€१५०");
        assert_eq!(process("डेढ़ हजार रुपए"), "₹१५००");
    }
}
