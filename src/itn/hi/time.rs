//! Time tagger for Hindi.
//!
//! Converts Hindi time expressions to formatted form:
//! - "एक बजे सात मिनट" → "१:०७"
//! - "ग्यारह बजे" → "११:००"
//! - "बारह पन्द्रह" → "१२:१५"
//! - "चार बजे पाँच सेकंड" → "४:००:०५"
//! - "सोलह घंटा एक मिनट सत्ताईस सेकंड" → "१६:०१:२७"
//! - "ढाई बजे" → "२:३०"
//! - "सवा चार बजे" → "४:१५"
//! - "साढ़े ग्यारह" → "११:३०"
//! - "पौने पाँच" → "४:४५"
//! - "तीन मिनट उन्नीस सेकंड" → "००:०३:१९"

use super::cardinal;

fn is_baje(w: &str) -> bool {
    matches!(w, "बजे" | "बजकर" | "बजके")
}

fn is_minute_word(w: &str) -> bool {
    w == "मिनट"
}

fn is_second_word(w: &str) -> bool {
    matches!(w, "सेकंड" | "सेकण्ड")
}

fn is_hour_word(w: &str) -> bool {
    // Only match singular "घंटा" for time; "घंटे" (plural/oblique) is for measure/duration
    w == "घंटा"
}

/// Check if a word is a measurement unit that means this is NOT a time context.
fn is_measure_unit(w: &str) -> bool {
    matches!(
        w,
        "ग्राम"
            | "किग्रा"
            | "मीटर"
            | "किलोमीटर"
            | "मिलीमीटर"
            | "लीटर"
            | "पिंट"
            | "गैलन"
            | "इंच"
            | "फुट"
            | "एकड़"
            | "हेक्टेयर"
            | "वर्ष"
            | "महीने"
            | "महीना"
            | "दर्जन"
            | "सेल्सियस"
            | "कैल्विन"
            | "ऐंपीयर"
            | "माइक्रॉन"
            | "मिलिग्राम"
            | "डेसिग्राम"
            | "मीट्रिक"
            | "वर्ग"
            | "वर्गसेंटीमीटर"
            | "क्यूबिकमिलीमीटर"
            | "घन"
            | "दशमलव"
            | "घंटे"
    )
}

/// Process time patterns in a string.
pub fn process(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return input.to_string();
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < words.len() {
        // 1. Modifier-led time: डेढ़/ढाई बजे/घंटा, सवा/साढ़े/पौने + number + बजे/घंटा
        //    Also: साढ़े X (standalone time) and पौने X (standalone time)
        //    But NOT when followed by a unit word (measure context)
        if cardinal::is_modifier(words[i]) {
            if let Some((time_str, consumed)) = try_parse_modifier_time(&words, i) {
                result.push(time_str);
                i += consumed;
                continue;
            }
        }

        // 2. Duration: X मिनट Y सेकंड (no hour)
        if cardinal::is_hi_number_word(words[i]) || cardinal::is_modifier(words[i]) {
            if let Some((time_str, consumed)) = try_parse_duration(&words, i) {
                result.push(time_str);
                i += consumed;
                continue;
            }
        }

        // 3. Standard time: X बजे/बजकर/बजके [Y मिनट] [Z सेकंड]
        if cardinal::is_hi_number_word(words[i]) {
            if let Some((time_str, consumed)) = try_parse_standard_time(&words, i) {
                result.push(time_str);
                i += consumed;
                continue;
            }
        }

        // 4. X घंटा Y मिनट/सेकंड (only with following मिनट/सेकंड)
        if cardinal::is_hi_number_word(words[i]) {
            if let Some((time_str, consumed)) = try_parse_ghanta_time(&words, i) {
                result.push(time_str);
                i += consumed;
                continue;
            }
        }

        // 5. Two-number time: "बारह पन्द्रह" → "१२:१५"
        //    Only at END of input or followed by non-number, non-time-marker word
        //    and NOT preceded by another digit word
        if cardinal::is_hi_number_word(words[i]) {
            if let Some((time_str, consumed)) = try_parse_two_number_time(&words, i, &result) {
                result.push(time_str);
                i += consumed;
                continue;
            }
        }

        result.push(words[i].to_string());
        i += 1;
    }

    result.join(" ")
}

/// Try to parse modifier-led time.
fn try_parse_modifier_time(words: &[&str], start: usize) -> Option<(String, usize)> {
    let modifier = words[start];

    match modifier {
        "डेढ़" => {
            // डेढ़ बजे → 1:30, डेढ़ घंटा → 1:30
            if start + 1 < words.len()
                && (is_baje(words[start + 1]) || is_hour_word(words[start + 1]))
            {
                return Some(("१:३०".to_string(), 2));
            }
        }
        "ढाई" => {
            if start + 1 < words.len()
                && (is_baje(words[start + 1]) || is_hour_word(words[start + 1]))
            {
                return Some(("२:३०".to_string(), 2));
            }
        }
        "सवा" => {
            // सवा X बजे → X:15
            if start + 2 < words.len() {
                if let Some(hour) = cardinal::word_to_value(words[start + 1]) {
                    if hour >= 1 && hour <= 24 && is_baje(words[start + 2]) {
                        return Some((format!("{}:{}", cardinal::to_devanagari(hour), "१५"), 3));
                    }
                }
            }
        }
        "साढ़े" => {
            if start + 1 < words.len() {
                if let Some(hour) = cardinal::word_to_value(words[start + 1]) {
                    if hour >= 1 && hour <= 24 {
                        // साढ़े X बजे → X:30
                        if start + 2 < words.len() && is_baje(words[start + 2]) {
                            return Some((
                                format!("{}:{}", cardinal::to_devanagari(hour), "३०"),
                                3,
                            ));
                        }
                        // साढ़े X alone — ONLY if NOT followed by unit word or number
                        if start + 2 < words.len() {
                            let next = words[start + 2];
                            if cardinal::is_hi_number_word(next)
                                || cardinal::is_modifier(next)
                                || is_measure_unit(next)
                            {
                                return None;
                            }
                        }
                        return Some((format!("{}:{}", cardinal::to_devanagari(hour), "३०"), 2));
                    }
                }
            }
        }
        "पौने" | "पौन" | "पौना" => {
            if start + 1 < words.len() {
                if let Some(hour) = cardinal::word_to_value(words[start + 1]) {
                    if hour >= 2 && hour <= 24 {
                        let actual_hour = hour - 1;
                        // पौने X बजे → (X-1):45
                        if start + 2 < words.len() && is_baje(words[start + 2]) {
                            return Some((
                                format!("{}:{}", cardinal::to_devanagari(actual_hour), "४५"),
                                3,
                            ));
                        }
                        // पौने X घंटा → (X-1):45
                        if start + 2 < words.len() && is_hour_word(words[start + 2]) {
                            return Some((
                                format!("{}:{}", cardinal::to_devanagari(actual_hour), "४५"),
                                3,
                            ));
                        }
                        // पौने X alone — ONLY if NOT followed by unit word or number
                        if start + 2 < words.len() {
                            let next = words[start + 2];
                            if cardinal::is_hi_number_word(next)
                                || cardinal::is_modifier(next)
                                || is_measure_unit(next)
                            {
                                return None;
                            }
                        }
                        return Some((
                            format!("{}:{}", cardinal::to_devanagari(actual_hour), "४५"),
                            2,
                        ));
                    }
                }
            }
        }
        _ => {}
    }

    None
}

/// Try to parse standard time: X बजे/बजकर/बजके [Y मिनट] [Z सेकंड]
fn try_parse_standard_time(words: &[&str], start: usize) -> Option<(String, usize)> {
    let mut hour_end = start;
    while hour_end < words.len() && cardinal::is_hi_number_word(words[hour_end]) {
        hour_end += 1;
    }

    if hour_end == start || hour_end >= words.len() {
        return None;
    }

    let time_marker = words[hour_end];
    if !is_baje(time_marker) {
        return None;
    }

    let hour_words: Vec<&str> = words[start..hour_end].to_vec();
    let hour = cardinal::words_to_number(&hour_words)?;

    let mut pos = hour_end + 1;
    let mut minute: Option<i64> = None;
    let mut second: Option<i64> = None;

    // Look for minutes
    let (min_end, min_val) = find_number_then_keyword(words, pos, is_minute_word);
    if let Some(mv) = min_val {
        minute = Some(mv);
        pos = min_end;
    }

    // Look for seconds
    let (sec_end, sec_val) = find_number_then_keyword(words, pos, is_second_word);
    if let Some(sv) = sec_val {
        second = Some(sv);
        pos = sec_end;
    }

    // If no minutes found but seconds directly follow
    if minute.is_none() && second.is_none() {
        let (sec_end2, sec_val2) = find_number_then_keyword(words, pos, is_second_word);
        if let Some(sv) = sec_val2 {
            second = Some(sv);
            pos = sec_end2;
        }
    }

    let time_str = format_time(hour, minute.unwrap_or(0), second);
    Some((time_str, pos - start))
}

/// Try to parse "X घंटा Y मिनट/सेकंड" (requires at least मिनट or सेकंड following).
fn try_parse_ghanta_time(words: &[&str], start: usize) -> Option<(String, usize)> {
    let mut hour_end = start;
    while hour_end < words.len() && cardinal::is_hi_number_word(words[hour_end]) {
        hour_end += 1;
    }

    if hour_end == start || hour_end >= words.len() {
        return None;
    }

    if !is_hour_word(words[hour_end]) {
        return None;
    }

    let hour_words: Vec<&str> = words[start..hour_end].to_vec();
    let hour = cardinal::words_to_number(&hour_words)?;

    let mut pos = hour_end + 1;
    let mut minute: Option<i64> = None;
    let mut second: Option<i64> = None;

    // Look for minutes
    let (min_end, min_val) = find_number_then_keyword(words, pos, is_minute_word);
    if let Some(mv) = min_val {
        minute = Some(mv);
        pos = min_end;
    }

    // Look for seconds
    let (sec_end, sec_val) = find_number_then_keyword(words, pos, is_second_word);
    if let Some(sv) = sec_val {
        second = Some(sv);
        pos = sec_end;
    }

    // If no minutes found but seconds directly follow
    if minute.is_none() && second.is_none() {
        let (sec_end2, sec_val2) = find_number_then_keyword(words, pos, is_second_word);
        if let Some(sv) = sec_val2 {
            second = Some(sv);
            pos = sec_end2;
        }
    }

    // Must have found at least one of मिनट or सेकंड to be a time expression
    if minute.is_none() && second.is_none() {
        return None;
    }

    let time_str = format_time(hour, minute.unwrap_or(0), second);
    Some((time_str, pos - start))
}

/// Try to parse two consecutive number words as hour:minute.
/// Very restrictive: only matches when it's clearly a standalone time expression.
/// Must not be part of a longer digit word sequence (address/telephone).
fn try_parse_two_number_time(
    words: &[&str],
    start: usize,
    result: &[String],
) -> Option<(String, usize)> {
    if start + 1 >= words.len() {
        return None;
    }

    // Both must be single-word values
    let hour = cardinal::word_to_value(words[start])?;
    let minute = cardinal::word_to_value(words[start + 1])?;

    // Valid ranges — hour must be reasonable for time
    if hour < 1 || hour > 24 || minute < 0 || minute > 59 {
        return None;
    }

    // Minute word must represent a value >= 10 (like पन्द्रह=15, अठारह=18)
    // Single digits 0-9 are too ambiguous (could be address digits)
    if minute < 10 {
        return None;
    }

    // Must NOT be followed by another digit/number word (would be address/telephone)
    if start + 2 < words.len() {
        let next = words[start + 2];
        if cardinal::is_hi_number_word(next) || cardinal::is_modifier(next) {
            return None;
        }
        if next == "दशमलव" || is_measure_unit(next) {
            return None;
        }
    }

    // Must NOT be preceded by a digit result or number word
    if let Some(last) = result.last() {
        if last.chars().all(|c| "०१२३४५६७८९".contains(c)) {
            return None;
        }
    }
    // Also check if the word before start is a digit word (not yet processed into result)
    if start > 0 && cardinal::is_hi_number_word(words[start - 1]) {
        return None;
    }

    let time_str = format!(
        "{}:{}",
        cardinal::to_devanagari(hour),
        format_two_digit_devanagari(minute)
    );
    Some((time_str, 2))
}

/// Try to parse a duration: X मिनट Y सेकंड (no hour)
fn try_parse_duration(words: &[&str], start: usize) -> Option<(String, usize)> {
    let (min_end, min_val) = find_number_then_keyword(words, start, is_minute_word);
    if let Some(mv) = min_val {
        let (sec_end, sec_val) = find_number_then_keyword(words, min_end, is_second_word);
        if let Some(sv) = sec_val {
            let time_str = format!(
                "{}:{}:{}",
                "००",
                format_two_digit_devanagari(mv),
                format_two_digit_devanagari(sv)
            );
            return Some((time_str, sec_end - start));
        }
    }
    None
}

/// Find a number span followed by a keyword.
fn find_number_then_keyword(
    words: &[&str],
    start: usize,
    is_keyword: fn(&str) -> bool,
) -> (usize, Option<i64>) {
    if start >= words.len() {
        return (start, None);
    }

    let mut end = start;
    while end < words.len()
        && (cardinal::is_hi_number_word(words[end]) || cardinal::is_modifier(words[end]))
    {
        end += 1;
    }

    if end == start || end >= words.len() || !is_keyword(words[end]) {
        return (start, None);
    }

    let num_words: Vec<&str> = words[start..end].to_vec();
    let val = cardinal::words_to_number(&num_words);
    if val.is_some() {
        (end + 1, val)
    } else {
        (start, None)
    }
}

fn format_two_digit_devanagari(n: i64) -> String {
    let s = format!("{:02}", n);
    cardinal::to_devanagari_str(&s)
}

fn format_time(hour: i64, minute: i64, second: Option<i64>) -> String {
    let h = cardinal::to_devanagari(hour);
    let m = format_two_digit_devanagari(minute);

    if let Some(s) = second {
        let sec = format_two_digit_devanagari(s);
        format!("{}:{}:{}", h, m, sec)
    } else {
        format!("{}:{}", h, m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(process("एक बजे सात मिनट"), "१:०७");
        assert_eq!(process("ग्यारह बजे"), "११:००");
    }

    #[test]
    fn test_modifier() {
        assert_eq!(process("ढाई बजे"), "२:३०");
        assert_eq!(process("सवा चार बजे"), "४:१५");
        assert_eq!(process("साढ़े ग्यारह"), "११:३०");
        assert_eq!(process("पौने पाँच"), "४:४५");
    }
}
