//! Time tagger for German.
//!
//! Converts spoken German time expressions to written form:
//! - "acht uhr" → "8 Uhr"
//! - "acht uhr sieben" → "08:07 Uhr"
//! - "halb zwölf" → "11:30 Uhr"
//! - "viertel vor zwölf" → "11:45 Uhr"
//! - "null uhr null minuten null sekunden" → "00:00:00 Uhr"

use super::cardinal;

/// Parse spoken German time expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Try HH:MM:SS pattern: "X uhr Y minuten Z sekunden"
    if let Some(result) = parse_hms(input_trim) {
        return Some(result);
    }

    // Try "halb X" pattern (half past = X-1:30)
    if let Some(result) = parse_halb(input_trim) {
        return Some(result);
    }

    // Try "viertel vor/nach X" patterns
    if let Some(result) = parse_viertel(input_trim) {
        return Some(result);
    }

    // Try "N vor X" / "N nach X" patterns
    if let Some(result) = parse_vor_nach(input_trim) {
        return Some(result);
    }

    // Try compound "Xuhr" patterns (no space before uhr)
    if let Some(result) = parse_compound_uhr(input_trim) {
        return Some(result);
    }

    // Try standard "X uhr [Y]" pattern
    if let Some(result) = parse_standard_uhr(input_trim) {
        return Some(result);
    }

    None
}

/// Parse HH:MM:SS: "null uhr null minuten null sekunden" → "00:00:00 Uhr"
fn parse_hms(input: &str) -> Option<String> {
    if !input.contains("minuten") && !input.contains("minute") {
        return None;
    }
    if !input.contains("sekunden") && !input.contains("sekunde") {
        return None;
    }

    // Extract timezone at end
    let (time_part, tz) = extract_timezone(input);

    // Split by "uhr", "minuten/minute", "sekunden/sekunde"
    let uhr_pos = time_part.find(" uhr ")?;
    let hour_str = &time_part[..uhr_pos];

    let after_uhr = &time_part[uhr_pos + 5..];

    // Find minuten/minute
    let min_end = if let Some(p) = after_uhr.find(" minuten") {
        p
    } else if let Some(p) = after_uhr.find(" minute") {
        p
    } else {
        return None;
    };

    let min_str = after_uhr[..min_end].trim();
    let after_min_keyword = if after_uhr[min_end..].starts_with(" minuten ") {
        &after_uhr[min_end + 9..]
    } else if after_uhr[min_end..].starts_with(" minute ") {
        &after_uhr[min_end + 8..]
    } else {
        return None;
    };

    // Find sekunden/sekunde
    let sec_end = if let Some(p) = after_min_keyword.find(" sekunden") {
        p
    } else if let Some(p) = after_min_keyword.find(" sekunde") {
        p
    } else {
        after_min_keyword.len()
    };

    let sec_str = after_min_keyword[..sec_end].trim();

    let hour = parse_time_number(hour_str)?;
    let min = parse_time_number(min_str)?;
    let sec = parse_time_number(sec_str)?;

    let result = format!("{:02}:{:02}:{:02} Uhr", hour, min, sec);
    if let Some(tz_str) = tz {
        Some(format!("{} {}", result, tz_str))
    } else {
        Some(result)
    }
}

/// Parse "halb X" → "(X-1):30 Uhr"
fn parse_halb(input: &str) -> Option<String> {
    if !input.starts_with("halb ") {
        return None;
    }
    let rest = input.strip_prefix("halb ")?;
    let hour = cardinal::words_to_number(rest)? as i64;
    let actual_hour = (hour - 1 + 24) % 24;
    Some(format!("{:02}:{:02} Uhr", actual_hour, 30))
}

/// Parse "viertel vor/nach X"
fn parse_viertel(input: &str) -> Option<String> {
    if input.starts_with("viertel vor ") {
        let rest = input.strip_prefix("viertel vor ")?;
        let (hour_part, modifier) = extract_time_modifier(rest);
        let hour = cardinal::words_to_number(hour_part.trim())? as i64;
        let actual_hour = (hour - 1 + 24) % 24;
        let result = format!("{:02}:{:02} Uhr", actual_hour, 45);
        return Some(append_modifier(&result, modifier));
    }
    if input.starts_with("viertel nach ") {
        let rest = input.strip_prefix("viertel nach ")?;
        let (hour_part, modifier) = extract_time_modifier(rest);
        let (time_part, tz) = extract_timezone(hour_part.trim());
        let hour = cardinal::words_to_number(time_part.trim())? as i64;
        let result = format!("{:02}:{:02} Uhr", hour, 15);
        let result = append_modifier(&result, modifier);
        if let Some(tz_str) = tz {
            return Some(format!("{} {}", result, tz_str));
        }
        return Some(result);
    }
    None
}

/// Extract time modifier (nachts, mittags, morgens, abends) from end
fn extract_time_modifier(input: &str) -> (&str, Option<&str>) {
    let modifiers = ["nachts", "mittags", "morgens", "abends"];
    for &m in &modifiers {
        if input.ends_with(m) {
            let before = input[..input.len() - m.len()].trim();
            return (before, Some(m));
        }
    }
    (input, None)
}

fn append_modifier(base: &str, modifier: Option<&str>) -> String {
    if let Some(m) = modifier {
        format!("{} {}", base, m)
    } else {
        base.to_string()
    }
}

/// Parse "N vor X" / "N nach X"
fn parse_vor_nach(input: &str) -> Option<String> {
    // "drei vor zwölf" → "11:57 Uhr"
    if let Some(pos) = input.find(" vor ") {
        let min_str = &input[..pos];
        let hour_str = &input[pos + 5..];
        let minutes = cardinal::words_to_number(min_str)? as i64;
        let hour = cardinal::words_to_number(hour_str)? as i64;
        let actual_hour = (hour - 1 + 24) % 24;
        let actual_min = 60 - minutes;
        return Some(format!("{:02}:{:02} Uhr", actual_hour, actual_min));
    }

    // "drei nach zwölf" → "12:03 Uhr"
    if let Some(pos) = input.find(" nach ") {
        let min_str = &input[..pos];
        let hour_str = &input[pos + 6..];
        let (time_part, tz) = extract_timezone(hour_str);
        let minutes = cardinal::words_to_number(min_str)? as i64;
        let hour = cardinal::words_to_number(time_part.trim())? as i64;
        let result = format!("{:02}:{:02} Uhr", hour, minutes);
        if let Some(tz_str) = tz {
            return Some(format!("{} {}", result, tz_str));
        }
        return Some(result);
    }

    None
}

/// Parse compound "Xuhr" pattern (no space before uhr)
/// "vierundzwanziguhr" → "24 Uhr"
/// "vierundzwanziguhrzweiundzwanzig" → "24:22 Uhr"
/// "vierundzwanziguhrzweiundzwanzigest" → "24:22 Uhr est"
/// "vierundzwanziguhrzweiundzwanzig e s t" → "24:22 Uhr est"
fn parse_compound_uhr(input: &str) -> Option<String> {
    // Extract timezone first (space-separated letters at end)
    let (main_part, tz) = extract_timezone(input);

    // Look for "uhr" embedded in the string (not as a separate word)
    if main_part.contains(" uhr") || !main_part.contains("uhr") {
        return None;
    }

    let uhr_pos = main_part.find("uhr")?;
    let hour_str = &main_part[..uhr_pos];
    let after_uhr = &main_part[uhr_pos + 3..];

    let hour = cardinal::words_to_number(hour_str)? as i64;

    if after_uhr.is_empty() {
        let result = format!("{} Uhr", hour);
        if let Some(tz_str) = tz {
            return Some(format!("{} {}", result, tz_str));
        }
        return Some(result);
    }

    // Try to parse minutes, potentially with appended timezone
    // "zweiundzwanzigest" → try "zweiundzwanzig" + "est"
    if let Some(minutes) = cardinal::words_to_number(after_uhr) {
        let minutes = minutes as i64;
        if minutes <= 59 {
            let result = format!("{:02}:{:02} Uhr", hour, minutes);
            if let Some(tz_str) = tz {
                return Some(format!("{} {}", result, tz_str));
            }
            return Some(result);
        }
    }

    // Try stripping common timezone suffixes from the end of after_uhr
    let tz_suffixes = ["est", "pst", "cst", "mst", "cet", "gmt", "utc"];
    for &tz_suffix in &tz_suffixes {
        if after_uhr.ends_with(tz_suffix) {
            let min_str = &after_uhr[..after_uhr.len() - tz_suffix.len()];
            if let Some(minutes) = cardinal::words_to_number(min_str) {
                let minutes = minutes as i64;
                if minutes <= 59 {
                    return Some(format!("{:02}:{:02} Uhr {}", hour, minutes, tz_suffix));
                }
            }
        }
    }

    None
}

/// Parse standard "X uhr [Y]" pattern
fn parse_standard_uhr(input: &str) -> Option<String> {
    let (time_part, tz) = extract_timezone(input);

    // Check for modifier: "mittags", "nachts"
    let modifiers = ["mittags", "nachts", "morgens", "abends"];
    let mut modifier = None;
    let mut cleaned = time_part.to_string();
    for &m in &modifiers {
        if cleaned.ends_with(m) {
            modifier = Some(m);
            cleaned = cleaned[..cleaned.len() - m.len()].trim().to_string();
            break;
        }
    }

    if !cleaned.contains(" uhr") {
        return None;
    }

    // Split on " uhr"
    let parts: Vec<&str> = cleaned.splitn(2, " uhr").collect();
    if parts.len() != 2 {
        return None;
    }

    let hour_str = parts[0].trim();
    let min_str = parts[1].trim();

    let hour = parse_time_number_or_zero(hour_str)?;

    let result = if min_str.is_empty() {
        format!("{} Uhr", hour)
    } else {
        let minutes = parse_time_number_or_zero(min_str)?;
        if minutes > 59 {
            return None;
        }
        format!("{:02}:{:02} Uhr", hour, minutes)
    };

    let result = if let Some(m) = modifier {
        format!("{} {}", result, m)
    } else {
        result
    };

    if let Some(tz_str) = tz {
        Some(format!("{} {}", result, tz_str))
    } else {
        Some(result)
    }
}

/// Parse a time number (handles "null" → 0, "ein/eine" → 1, etc.)
fn parse_time_number(input: &str) -> Option<i64> {
    let trimmed = input.trim();
    if trimmed == "null" {
        return Some(0);
    }
    if trimmed == "eine" || trimmed == "ein" || trimmed == "einer" || trimmed == "eins" {
        return Some(1);
    }
    cardinal::words_to_number(trimmed).map(|n| n as i64)
}

/// Parse a time number that may be zero or a single-digit word
fn parse_time_number_or_zero(input: &str) -> Option<i64> {
    let trimmed = input.trim();
    if trimmed == "null" {
        return Some(0);
    }
    parse_time_number(trimmed)
}

/// Extract timezone suffix (space-separated single letters at end)
/// "viertel nach zwölf nachts" → ("viertel nach zwölf", Some("nachts"))
/// "vierundzwanziguhrzweiundzwanzigest" → ("vierundzwanziguhrzweiundzwanzig", Some("est"))
/// "vierundzwanziguhrzweiundzwanzig e s t" → ("vierundzwanziguhrzweiundzwanzig", Some("est"))
fn extract_timezone(input: &str) -> (&str, Option<String>) {
    // Check for single-letter timezone: "e s t", "p s t", etc.
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.len() >= 3 {
        // Check if last N tokens are all single letters
        let mut tz_start = tokens.len();
        for i in (0..tokens.len()).rev() {
            if tokens[i].len() == 1 && tokens[i].chars().all(|c| c.is_ascii_alphabetic()) {
                tz_start = i;
            } else {
                break;
            }
        }
        if tz_start < tokens.len() && tokens.len() - tz_start >= 2 {
            let tz: String = tokens[tz_start..].join("");
            let time_part = tokens[..tz_start].join(" ");
            // Return references won't work since we're creating new strings
            // We need to handle this differently
            let time_end = input.len() - tokens[tz_start..].iter().map(|t| t.len()).sum::<usize>()
                - (tokens.len() - tz_start); // spaces
            let time_part_ref = input[..time_end].trim();
            return (time_part_ref, Some(tz));
        }
    }

    // Check for "est", "pst" etc. appended directly (compound form)
    // e.g., "vierundzwanziguhrzweiundzwanzigest" - the "est" at the end
    // This is tricky - we'd need to know it's a tz. Skip for now.

    (input, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard() {
        assert_eq!(parse("acht uhr"), Some("8 Uhr".to_string()));
        assert_eq!(parse("achtzehn uhr"), Some("18 Uhr".to_string()));
        assert_eq!(parse("acht uhr sieben"), Some("08:07 Uhr".to_string()));
    }

    #[test]
    fn test_halb() {
        assert_eq!(parse("halb zwölf"), Some("11:30 Uhr".to_string()));
    }

    #[test]
    fn test_viertel() {
        assert_eq!(parse("viertel vor zwölf"), Some("11:45 Uhr".to_string()));
        assert_eq!(parse("viertel nach zwölf"), Some("12:15 Uhr".to_string()));
    }

    #[test]
    fn test_vor_nach() {
        assert_eq!(parse("drei vor zwölf"), Some("11:57 Uhr".to_string()));
        assert_eq!(parse("drei nach zwölf"), Some("12:03 Uhr".to_string()));
    }

    #[test]
    fn test_mittags() {
        assert_eq!(parse("zwölf uhr mittags"), Some("12 Uhr mittags".to_string()));
    }

    #[test]
    fn test_hms() {
        assert_eq!(
            parse("null uhr null minuten null sekunden"),
            Some("00:00:00 Uhr".to_string())
        );
    }
}
