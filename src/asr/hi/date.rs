//! Date tagger for Hindi.
//!
//! Converts Hindi date expressions to Devanagari form:
//! - "छः मई" → "६ मई"
//! - "पच्चीस मार्च दो हज़ार दस" → "२५ मार्च, २०१०"
//! - "मार्च तीस उन्नीस सौ नब्बे" → "मार्च ३०, १९९०"
//! - "उन्नीस सौ नब्बे से उन्नीस सौ इक्यानबे" → "१९९०-१९९१"
//! - "चौंतीस सौ ईसा पूर्व" → "३४०० ई.पू."
//! - "दसवें शताब्दी" → "१०वें शताब्दी"

use super::cardinal;

/// Hindi month names.
const MONTHS: &[&str] = &[
    "जनवरी", "फ़रवरी", "फरवरी", "मार्च", "अप्रैल", "मई", "जून",
    "जुलाई", "अगस्त", "सितंबर", "अक्टूबर", "नवंबर", "दिसंबर",
];

fn is_month(word: &str) -> bool {
    MONTHS.contains(&word)
}

/// Process date patterns in a string.
pub fn process(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return input.to_string();
    }

    // First handle special patterns, then fall through to ordinal+cardinal processing
    let mut result = Vec::new();
    let mut i = 0;

    while i < words.len() {
        // Check for "शताब्दी" pattern — this is handled by ordinal processor
        // Check for "ईसा पूर्व" / "ईस्वी" / "ईसवी" suffixes
        // Check for "की" + number pattern (मार्च की दो → मार्च २)
        // Check for "से" range pattern (X से Y → X-Y)
        // Check for "वर्ष" / "सन" prefix

        // "वर्ष" or "सन" followed by number → "वर्ष/सन" + Devanagari
        if (words[i] == "वर्ष" || words[i] == "सन") && i + 1 < words.len() {
            let (year_end, year_val) = find_number_span(&words, i + 1);
            if let Some(yv) = year_val {
                result.push(words[i].to_string());
                result.push(cardinal::to_devanagari(yv));
                i = year_end;
                continue;
            }
        }

        // Month + "की" + number → month + number
        if is_month(words[i]) && i + 2 < words.len() && words[i + 1] == "की" {
            let (num_end, num_val) = find_number_span(&words, i + 2);
            if let Some(nv) = num_val {
                result.push(words[i].to_string());
                result.push(cardinal::to_devanagari(nv));
                i = num_end;
                continue;
            }
        }

        // Check for date range: "X से Y" where both are numbers
        // or "X से Y तक"
        if i > 0 && words[i] == "से" && i + 1 < words.len() {
            // Check if previous words form a number and next words form a number
            // This is complex; handle it after basic patterns
        }

        // Number + Month + Year pattern (with optional ईसवी/ईसा पूर्व)
        // Month + Number + Year pattern
        if is_month(words[i]) {
            // Month-first: "मार्च तीस उन्नीस सौ नब्बे"
            // Try to find day (1-31) then year
            if i + 1 < words.len() {
                // First try: day as a greedy number span, then year
                let (day_end, day_val) = find_number_span(&words, i + 1);
                if let Some(dv) = day_val {
                    // Check for year after day
                    let (year_end, year_val) = find_number_span(&words, day_end);
                    if let Some(yv) = year_val {
                        let (era_end, era_str) = find_era_suffix(&words, year_end);
                        result.push(format!("{} {},", words[i], cardinal::to_devanagari(dv)));
                        if let Some(era) = era_str {
                            result.push(format!("{} {}", cardinal::to_devanagari(yv), era));
                        } else {
                            result.push(cardinal::to_devanagari(yv));
                        }
                        i = era_end;
                        continue;
                    }
                    // Just month + day
                    result.push(format!("{} {}", words[i], cardinal::to_devanagari(dv)));
                    i = day_end;
                    continue;
                }

                // Second try: if greedy failed, try day as single word (1-31), rest as year
                if let Some(dv) = cardinal::word_to_value(words[i + 1]) {
                    if dv >= 1 && dv <= 31 && i + 2 < words.len() {
                        let (year_end, year_val) = find_number_span(&words, i + 2);
                        if let Some(yv) = year_val {
                            let (era_end, era_str) = find_era_suffix(&words, year_end);
                            result.push(format!("{} {},", words[i], cardinal::to_devanagari(dv)));
                            if let Some(era) = era_str {
                                result.push(format!("{} {}", cardinal::to_devanagari(yv), era));
                            } else {
                                result.push(cardinal::to_devanagari(yv));
                            }
                            i = era_end;
                            continue;
                        }
                    }
                }
            }

            result.push(words[i].to_string());
            i += 1;
            continue;
        }

        // Number + Month pattern (day first)
        if cardinal::is_hi_number_word(words[i]) || cardinal::is_modifier(words[i]) {
            let (num_end, num_val) = find_number_span(&words, i);
            if let Some(nv) = num_val {
                // Check if followed by month
                if num_end < words.len() && is_month(words[num_end]) {
                    let month = words[num_end];
                    // Check for year after month
                    let (year_end, year_val) = find_number_span(&words, num_end + 1);
                    if let Some(yv) = year_val {
                        // Check for era suffix
                        let (era_end, era_str) = find_era_suffix(&words, year_end);
                        if let Some(era) = era_str {
                            result.push(format!("{} {},", cardinal::to_devanagari(nv), month));
                            result.push(format!("{} {}", cardinal::to_devanagari(yv), era));
                        } else {
                            result.push(format!("{} {},", cardinal::to_devanagari(nv), month));
                            result.push(cardinal::to_devanagari(yv));
                        }
                        i = era_end;
                        continue;
                    }
                    // Just day + month
                    result.push(format!("{} {}", cardinal::to_devanagari(nv), month));
                    i = num_end + 1;
                    continue;
                }

                // Check for "से" range pattern
                if num_end < words.len() && words[num_end] == "से" {
                    let (end2, val2) = find_number_span(&words, num_end + 1);
                    if let Some(v2) = val2 {
                        // Check for era suffix after range
                        let (era_end, era_str) = find_era_suffix(&words, end2);
                        // Check for "तक" after range
                        let (tack_end, has_tack) = if era_end < words.len() && words[era_end] == "तक" {
                            (era_end + 1, true)
                        } else {
                            (era_end, false)
                        };

                        if let Some(era) = era_str {
                            if has_tack {
                                result.push(format!(
                                    "{}-{} {} तक",
                                    cardinal::to_devanagari(nv),
                                    cardinal::to_devanagari(v2),
                                    era
                                ));
                            } else {
                                result.push(format!(
                                    "{}-{} {}",
                                    cardinal::to_devanagari(nv),
                                    cardinal::to_devanagari(v2),
                                    era
                                ));
                            }
                        } else if has_tack {
                            result.push(format!(
                                "{}-{} तक",
                                cardinal::to_devanagari(nv),
                                cardinal::to_devanagari(v2),
                            ));
                        } else {
                            result.push(format!(
                                "{}-{}",
                                cardinal::to_devanagari(nv),
                                cardinal::to_devanagari(v2),
                            ));
                        }
                        i = tack_end;
                        continue;
                    }
                }

                // Check for era suffix directly after number
                if num_end < words.len() {
                    let (era_end, era_str) = find_era_suffix(&words, num_end);
                    if let Some(era) = era_str {
                        result.push(format!("{} {}", cardinal::to_devanagari(nv), era));
                        i = era_end;
                        continue;
                    }
                }
            }
        }

        // Default: pass through
        result.push(words[i].to_string());
        i += 1;
    }

    result.join(" ")
}

/// Find a number span starting at position `start`.
/// Returns (end_position, value).
fn find_number_span(words: &[&str], start: usize) -> (usize, Option<i64>) {
    if start >= words.len() {
        return (start, None);
    }

    let mut end = start;
    while end < words.len() {
        if cardinal::is_hi_number_word(words[end]) || cardinal::is_modifier(words[end]) {
            end += 1;
        } else {
            break;
        }
    }

    if end == start {
        return (start, None);
    }

    let span: Vec<&str> = words[start..end].to_vec();
    let val = cardinal::words_to_number(&span);
    if val.is_some() {
        (end, val)
    } else {
        (start, None)
    }
}

/// Find an era suffix (ईसा पूर्व, ईस्वी, ईसवी) starting at `start`.
/// Returns (end_position, era_string).
fn find_era_suffix(words: &[&str], start: usize) -> (usize, Option<&'static str>) {
    if start >= words.len() {
        return (start, None);
    }

    // "ईसा पूर्व" → "ई.पू."
    if start + 1 < words.len() && words[start] == "ईसा" && words[start + 1] == "पूर्व" {
        return (start + 2, Some("ई.पू."));
    }

    // "ईस्वी" or "ईसवी" → "ई."
    if words[start] == "ईस्वी" || words[start] == "ईसवी" {
        return (start + 1, Some("ई."));
    }

    (start, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_month() {
        assert_eq!(process("छः मई"), "६ मई");
        assert_eq!(process("तीस जून"), "३० जून");
    }

    #[test]
    fn test_day_month_year() {
        assert_eq!(process("पच्चीस मार्च दो हज़ार दस"), "२५ मार्च, २०१०");
    }
}
