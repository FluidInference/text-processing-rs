//! Cardinal number tagger for Hindi.
//!
//! Converts Hindi number words to Devanagari numeral form:
//! - "एक" → "१"
//! - "दो हज़ार दो सौ बाईस" → "२२२२"
//! - "एक लाख एक" → "१००००१"
//! - "सवा सात सौ" → "७२५"
//! - "डेढ़ सौ" → "१५०"

/// Convert an Arabic digit to Devanagari.
pub fn to_devanagari_digit(d: u8) -> char {
    match d {
        0 => '०',
        1 => '१',
        2 => '२',
        3 => '३',
        4 => '४',
        5 => '५',
        6 => '६',
        7 => '७',
        8 => '८',
        9 => '९',
        _ => unreachable!(),
    }
}

/// Convert a number to Devanagari digit string.
pub fn to_devanagari(n: i64) -> String {
    if n < 0 {
        let s = to_devanagari(-n);
        return format!("-{}", s);
    }
    if n == 0 {
        return "०".to_string();
    }
    let s = n.to_string();
    s.chars()
        .map(|c| to_devanagari_digit(c as u8 - b'0'))
        .collect()
}

/// Convert a decimal string like "206.29" to Devanagari "२०६.२९".
pub fn to_devanagari_str(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_digit() {
                to_devanagari_digit(c as u8 - b'0')
            } else {
                c
            }
        })
        .collect()
}

/// Map a single Hindi word to its numeric value.
/// Returns None if the word is not a Hindi number word.
pub fn word_to_value(word: &str) -> Option<i64> {
    match word {
        "शून्य" | "शुन्य" => Some(0),
        "एक" => Some(1),
        "दो" => Some(2),
        "तीन" => Some(3),
        "चार" => Some(4),
        "पाँच" | "पांच" | "पांचो" => Some(5),
        "छह" | "छः" | "छे" => Some(6),
        "सात" => Some(7),
        "आठ" => Some(8),
        "नौ" => Some(9),
        "दस" => Some(10),
        "ग्यारह" => Some(11),
        "बारह" => Some(12),
        "तेरह" => Some(13),
        "चौदह" => Some(14),
        "पन्द्रह" | "पंद्रह" | "पंदरह" | "पंडरह" => Some(15),
        "सोलह" => Some(16),
        "सत्रह" => Some(17),
        "अठारह" | "अठाहर" | "अठाहरवीं" => Some(18),
        "उन्नीस" => Some(19),
        "बीस" => Some(20),
        "इक्कीस" => Some(21),
        "बाईस" => Some(22),
        "तेईस" => Some(23),
        "चौबीस" => Some(24),
        "पच्चीस" => Some(25),
        "छब्बीस" => Some(26),
        "सत्ताईस" => Some(27),
        "अट्ठाईस" => Some(28),
        "उनतीस" => Some(29),
        "तीस" => Some(30),
        "इकतीस" | "इकत्तीस" => Some(31),
        "बत्तीस" => Some(32),
        "तैंतीस" => Some(33),
        "चौंतीस" => Some(34),
        "पैंतीस" | "पैंतिस" => Some(35),
        "छत्तीस" | "छतीस" => Some(36),
        "सैंतीस" => Some(37),
        "अड़तीस" => Some(38),
        "उनतालीस" => Some(39),
        "चालीस" => Some(40),
        "इकतालीस" => Some(41),
        "बयालीस" => Some(42),
        "तैंतालीस" => Some(43),
        "चौवालीस" => Some(44),
        "पैंतालीस" | "पैंतालिस" => Some(45),
        "छियालीस" => Some(46),
        "सैंतालीस" => Some(47),
        "अड़तालीस" => Some(48),
        "उनचास" => Some(49),
        "पचास" => Some(50),
        "इक्यावन" => Some(51),
        "बावन" => Some(52),
        "तिरपन" | "तिरेपन" => Some(53),
        "चौवन" | "चौंवन" => Some(54),
        "पचपन" => Some(55),
        "छप्पन" => Some(56),
        "सत्तावन" => Some(57),
        "अट्ठावन" => Some(58),
        "उनसठ" => Some(59),
        "साठ" => Some(60),
        "इकसठ" => Some(61),
        "बासठ" => Some(62),
        "तिरसठ" => Some(63),
        "चौंसठ" => Some(64),
        "पैंसठ" => Some(65),
        "छियासठ" => Some(66),
        "सड़सठ" | "सरसठ" => Some(67),
        "अड़सठ" => Some(68),
        "उनहत्तर" => Some(69),
        "सत्तर" => Some(70),
        "इकहत्तर" => Some(71),
        "बहत्तर" => Some(72),
        "तिहत्तर" => Some(73),
        "चौहत्तर" => Some(74),
        "पिछत्तर" | "पचहत्तर" => Some(75),
        "छिहत्तर" => Some(76),
        "सतत्तर" => Some(77),
        "अठत्तर" | "अठहत्तर" => Some(78),
        "उनासी" | "उन्नासी" => Some(79),
        "अस्सी" => Some(80),
        "इक्यासी" => Some(81),
        "बयासी" => Some(82),
        "तिरासी" => Some(83),
        "चौरासी" => Some(84),
        "पचासी" | "पच्चासी" => Some(85),
        "छियासी" => Some(86),
        "सत्तासी" => Some(87),
        "अठासी" => Some(88),
        "नवासी" => Some(89),
        "नब्बे" => Some(90),
        "इक्यानबे" | "इक्यानवे" => Some(91),
        "बानवे" => Some(92),
        "तिरानवे" => Some(93),
        "चौरानवे" => Some(94),
        "पिचानवे" | "पंचानवे" => Some(95),
        "छियानवे" => Some(96),
        "सत्तानवे" => Some(97),
        "अट्ठानवे" => Some(98),
        "निन्यानवे" | "निन्यानवें" => Some(99),
        _ => None,
    }
}

/// Check if a word is a scale word (सौ, हज़ार, लाख, करोड़, अरब).
pub fn scale_value(word: &str) -> Option<i64> {
    match word {
        "सौ" => Some(100),
        "हज़ार" | "हजार" => Some(1_000),
        "लाख" => Some(1_00_000),
        "करोड़" => Some(1_00_00_000),
        "अरब" => Some(1_00_00_00_000),
        _ => None,
    }
}

/// Check if a word is a Hindi number word (value or scale).
pub fn is_hi_number_word(word: &str) -> bool {
    word_to_value(word).is_some() || scale_value(word).is_some()
}

/// Check if a word is a special modifier.
pub fn is_modifier(word: &str) -> bool {
    matches!(word, "सवा" | "साढ़े" | "डेढ़" | "ढाई" | "पौने" | "पौन" | "पौना")
}

/// Parse a sequence of Hindi number words into a number.
/// Uses Indian numbering: अरब > करोड़ > लाख > हज़ार > सौ
///
/// Modifier semantics:
/// - सवा N*scale → N*scale + scale/4 (add quarter of the lowest scale)
/// - साढ़े N*scale → N*scale + scale/2 (add half of the lowest scale)
/// - डेढ़ scale → 1.5 * scale
/// - ढाई scale → 2.5 * scale
/// - पौने N*scale → N*scale - scale/4 (subtract quarter of the lowest scale)
pub fn words_to_number(words: &[&str]) -> Option<i64> {
    if words.is_empty() {
        return None;
    }

    // Handle special modifiers at the start
    match words[0] {
        "डेढ़" => {
            if words.len() == 1 {
                return None;
            }
            let rest = &words[1..];
            let base = parse_compound_number(rest)?;
            let lowest = find_lowest_scale(rest);
            return Some(base + lowest / 2);
        }
        "ढाई" => {
            if words.len() == 1 {
                return None;
            }
            let rest = &words[1..];
            let base = parse_compound_number(rest)?;
            let lowest = find_lowest_scale(rest);
            return Some(base + lowest + lowest / 2);
        }
        "सवा" => {
            if words.len() == 1 {
                return None;
            }
            let rest = &words[1..];
            let base = parse_compound_number(rest)?;
            let lowest = find_lowest_scale(rest);
            return Some(base + lowest / 4);
        }
        "साढ़े" => {
            if words.len() == 1 {
                return None;
            }
            let rest = &words[1..];
            let base = parse_compound_number(rest)?;
            let lowest = find_lowest_scale(rest);
            return Some(base + lowest / 2);
        }
        "पौने" | "पौन" | "पौना" => {
            if words.len() == 1 {
                return None;
            }
            let rest = &words[1..];
            let base = parse_compound_number(rest)?;
            let lowest = find_lowest_scale(rest);
            return Some(base - lowest / 4);
        }
        _ => {}
    }

    parse_compound_number(words)
}

/// Find the lowest scale value used in a word sequence.
pub fn find_lowest_scale(words: &[&str]) -> i64 {
    let mut lowest: Option<i64> = None;
    for &w in words {
        if let Some(sv) = scale_value(w) {
            match lowest {
                None => lowest = Some(sv),
                Some(current) => {
                    if sv < current {
                        lowest = Some(sv);
                    }
                }
            }
        }
    }
    lowest.unwrap_or(1)
}

/// Parse a compound Hindi number from words.
/// Handles the Indian number scale: अरब > करोड़ > लाख > हज़ार > सौ
fn parse_compound_number(words: &[&str]) -> Option<i64> {
    if words.is_empty() {
        return None;
    }

    // Single word
    if words.len() == 1 {
        if let Some(v) = word_to_value(words[0]) {
            return Some(v);
        }
        if let Some(s) = scale_value(words[0]) {
            return Some(s);
        }
        return None;
    }

    // Multi-word: accumulate using Indian number system
    let scales: &[(&[&str], i64)] = &[
        (&["अरब"], 1_00_00_00_000),
        (&["करोड़"], 1_00_00_000),
        (&["लाख"], 1_00_000),
        (&["हज़ार", "हजार"], 1_000),
        (&["सौ"], 100),
    ];

    for &(scale_words, scale_val) in scales {
        for (i, &w) in words.iter().enumerate() {
            if scale_words.contains(&w) {
                let before = &words[..i];
                let after = &words[i + 1..];

                let multiplier = if before.is_empty() {
                    1
                } else {
                    parse_compound_number(before)?
                };

                let remainder = if after.is_empty() {
                    0
                } else {
                    parse_compound_number(after)?
                };

                return Some(multiplier * scale_val + remainder);
            }
        }
    }

    // No scale found — try as a single value word
    if words.len() == 1 {
        return word_to_value(words[0]);
    }

    None
}

/// Strip trailing punctuation from a word, returning (core_word, suffix).
fn strip_trailing_punct(word: &str) -> (&str, &str) {
    for punct in &[",", ".", ";", ":", "!", "?"] {
        if word.ends_with(punct) {
            let core = &word[..word.len() - punct.len()];
            return (core, punct);
        }
    }
    (word, "")
}

/// Process Hindi text, replacing Hindi number word sequences with Devanagari numerals.
/// This is a sentence-scanning approach: it finds number word spans within the input
/// and replaces them with their numeric equivalents.
pub fn process(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return input.to_string();
    }

    // Pre-process: strip trailing punctuation for matching purposes
    let stripped: Vec<(&str, &str)> = words.iter().map(|w| strip_trailing_punct(w)).collect();

    let mut result = Vec::new();
    let mut i = 0;

    while i < words.len() {
        // Try to find the longest number word span starting at i
        let mut best_end = i;
        let mut best_val: Option<i64> = None;
        let mut best_suffix = "";

        // Check for modifier-led sequences first
        let has_modifier = is_modifier(stripped[i].0);

        let max_end = words.len().min(i + 15); // reasonable limit
        for end in (i + 1..=max_end).rev() {
            // Build span from stripped words (no trailing punct)
            let span: Vec<&str> = stripped[i..end].iter().map(|(core, _)| *core).collect();

            // At least one word must be a number word or modifier
            let has_number = span.iter().any(|w| is_hi_number_word(w) || is_modifier(w));
            if !has_number {
                continue;
            }

            if let Some(val) = words_to_number(&span) {
                if has_modifier && end > i + 1 {
                    best_end = end;
                    best_val = Some(val);
                    best_suffix = stripped[end - 1].1;
                    break;
                }
                if !has_modifier {
                    best_end = end;
                    best_val = Some(val);
                    best_suffix = stripped[end - 1].1;
                    break;
                }
            }
        }

        if let Some(val) = best_val {
            let num_str = to_devanagari(val);
            if best_suffix.is_empty() {
                result.push(num_str);
            } else {
                result.push(format!("{}{}", num_str, best_suffix));
            }
            i = best_end;
        } else {
            // Try single word (with stripped punctuation)
            let (core, suffix) = stripped[i];
            if let Some(val) = word_to_value(core) {
                let num_str = to_devanagari(val);
                if suffix.is_empty() {
                    result.push(num_str);
                } else {
                    result.push(format!("{}{}", num_str, suffix));
                }
                i += 1;
            } else if let Some(val) = scale_value(core) {
                let num_str = to_devanagari(val);
                if suffix.is_empty() {
                    result.push(num_str);
                } else {
                    result.push(format!("{}{}", num_str, suffix));
                }
                i += 1;
            } else {
                result.push(words[i].to_string());
                i += 1;
            }
        }
    }

    result.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(to_devanagari(1), "१");
        assert_eq!(to_devanagari(100), "१००");
        assert_eq!(to_devanagari(12345), "१२३४५");
    }

    #[test]
    fn test_words_to_number() {
        assert_eq!(words_to_number(&["एक"]), Some(1));
        assert_eq!(words_to_number(&["एक", "सौ"]), Some(100));
        assert_eq!(words_to_number(&["दो", "हज़ार", "दो", "सौ", "बाईस"]), Some(2222));
        assert_eq!(words_to_number(&["एक", "लाख", "एक"]), Some(100001));
    }

    #[test]
    fn test_modifiers() {
        assert_eq!(words_to_number(&["सवा", "सात", "सौ"]), Some(725));
        assert_eq!(words_to_number(&["साढ़े", "सात", "सौ"]), Some(750));
        assert_eq!(words_to_number(&["डेढ़", "सौ"]), Some(150));
        assert_eq!(words_to_number(&["ढाई", "सौ"]), Some(250));
        assert_eq!(words_to_number(&["पौने", "तीन", "सौ"]), Some(275));
        assert_eq!(words_to_number(&["सवा", "सोलह", "सौ"]), Some(1625));
        assert_eq!(words_to_number(&["साढ़े", "सोलह", "सौ"]), Some(1650));
    }
}
