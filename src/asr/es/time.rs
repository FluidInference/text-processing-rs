//! Time tagger for Spanish.
//!
//! Converts spoken Spanish time expressions to written form:
//! - "las dieciséis cincuenta" → "las 16:50"
//! - "la una y cuarto" → "la 1:15"
//! - "las dos menos cuarto" → "la 1:45"
//! - "cuarto para las dos" → "la 1:45"

use super::cardinal;

/// Parse spoken Spanish time expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Try "X para las Y" pattern (including "cuarto para las X", "un cuarto para las X")
    if let Some(result) = parse_para(input_trim) {
        return Some(result);
    }

    // Try "X y media de la tarde" (no article output)
    if let Some(result) = parse_media_de_la_tarde(input_trim) {
        return Some(result);
    }

    // Try "la/las X" patterns
    if input_trim.starts_with("la ") || input_trim.starts_with("las ") {
        return parse_article_time(input_trim);
    }

    None
}

/// Parse "X para las Y" → "las (Y-1):(60-X) Uhr"
fn parse_para(input: &str) -> Option<String> {
    // "cuarto para las dos" → "la 1:45"
    // "un cuarto para las dos" → "la 1:45"
    // "diez para las doce" → "las 11:50"

    let para_pos = input.find(" para las ")?;
    let before = &input[..para_pos];
    let after = &input[para_pos + 10..]; // " para las " is 10 chars

    let hour = parse_hour_word(after)?;
    let minutes = parse_minutes_before(before)?;

    let (actual_hour, actual_min) = subtract_time(hour, minutes);

    let article = if actual_hour == 1 { "la" } else { "las" };
    Some(format!("{} {}:{:02}", article, actual_hour, actual_min))
}

/// Parse "X y media de la tarde" → "X:30 p.m."
fn parse_media_de_la_tarde(input: &str) -> Option<String> {
    if !input.ends_with(" de la tarde") {
        return None;
    }
    let before = input[..input.len() - 12].trim();

    // "dos y media" → hour=2, min=30
    if before.ends_with(" y media") {
        let hour_part = before[..before.len() - 8].trim();
        let hour = parse_hour_word(hour_part)?;
        return Some(format!("{}:{:02} p.m.", hour, 30));
    }

    None
}

/// Parse "la/las X ..." time patterns
fn parse_article_time(input: &str) -> Option<String> {
    let (article, rest) = if input.starts_with("la ") {
        ("la", &input[3..])
    } else if input.starts_with("las ") {
        ("las", &input[4..])
    } else {
        return None;
    };

    // Extract timezone suffix "u t c más X"
    let (time_part, tz) = extract_timezone(rest);

    // Extract AM/PM modifier
    let (time_part, ampm) = extract_ampm(time_part);
    let time_part = time_part.trim();

    // Extract "de la tarde" → p.m.
    let (time_part, de_la) = extract_de_la(time_part);
    let time_part = time_part.trim();
    let ampm = ampm.or(de_la);

    // Try "X menos Y" pattern
    if let Some(result) = parse_menos(time_part, ampm.as_deref(), tz.as_deref()) {
        return Some(result);
    }

    // Try "X y cuarto" → X:15
    if time_part.ends_with(" y cuarto") {
        let hour_part = &time_part[..time_part.len() - 9];
        let hour = parse_hour_word(hour_part)?;
        let out_article = if hour == 1 { "la" } else { article };
        return Some(format_time(out_article, hour, 15, ampm.as_deref(), tz.as_deref()));
    }

    // Try "X y media" → X:30
    if time_part.ends_with(" y media") {
        let hour_part = &time_part[..time_part.len() - 8];
        let hour = parse_hour_word(hour_part)?;
        let out_article = if hour == 1 { "la" } else { article };
        return Some(format_time(out_article, hour, 30, ampm.as_deref(), tz.as_deref()));
    }

    // Try "X y MINUTES" → X:MM
    if let Some(y_pos) = time_part.find(" y ") {
        let hour_part = &time_part[..y_pos];
        let min_part = &time_part[y_pos + 3..];

        let hour = parse_hour_word(hour_part)?;
        let minutes = cardinal::words_to_number(min_part)? as i64;
        if minutes > 59 { return None; }

        let out_article = if hour == 1 { "la" } else { article };
        return Some(format_time(out_article, hour, minutes, ampm.as_deref(), tz.as_deref()));
    }

    // Try "X MINUTES" (no connector) → X:MM
    let tokens: Vec<&str> = time_part.split_whitespace().collect();
    if tokens.len() >= 2 {
        // Try to find where hour ends and minutes begin
        // First token(s) = hour, remaining = minutes
        let hour = parse_hour_word(tokens[0])?;
        let min_str = tokens[1..].join(" ");
        if let Some(minutes) = cardinal::words_to_number(&min_str) {
            let minutes = minutes as i64;
            if minutes <= 59 && minutes > 0 {
                let out_article = if hour == 1 { "la" } else { article };
                return Some(format_time(out_article, hour, minutes, ampm.as_deref(), tz.as_deref()));
            }
        }
    }

    // Try bare hour: "la una" / "las dos"
    if tokens.len() == 1 {
        // Check if it's actually a time (not "las tres personas")
        if parse_hour_word(tokens[0]).is_some() {
            // Bare hours with AM/PM should be formatted
            if ampm.is_some() {
                let hour = parse_hour_word(tokens[0])?;
                let out_article = if hour == 1 { "la" } else { article };
                return Some(format_time(out_article, hour, 0, ampm.as_deref(), tz.as_deref()));
            }
            // Bare hours without AM/PM pass through
            return None;
        }
        return None;
    }

    None
}

/// Parse "X menos Y" → subtract Y from X
fn parse_menos(input: &str, ampm: Option<&str>, tz: Option<&str>) -> Option<String> {
    let menos_pos = input.find(" menos ")?;
    let hour_part = &input[..menos_pos];
    let min_part = &input[menos_pos + 7..];

    let hour = parse_hour_word(hour_part)?;
    let minutes = parse_minutes_before(min_part)?;

    let (actual_hour, actual_min) = subtract_time(hour, minutes);

    let article = if actual_hour == 1 { "la" } else { "la" };
    Some(format_time(article, actual_hour, actual_min, ampm, tz))
}

/// Parse minutes for "before" patterns
fn parse_minutes_before(input: &str) -> Option<i64> {
    let trimmed = input.trim();
    match trimmed {
        "cuarto" | "un cuarto" => Some(15),
        "media" => Some(30),
        _ => cardinal::words_to_number(trimmed).map(|n| n as i64),
    }
}

/// Subtract minutes from hour
fn subtract_time(hour: i64, minutes: i64) -> (i64, i64) {
    let total_minutes = hour * 60 - minutes;
    let actual_hour = total_minutes.div_euclid(60).rem_euclid(24);
    let actual_min = total_minutes.rem_euclid(60);
    (actual_hour, actual_min)
}

/// Parse hour word to number
fn parse_hour_word(input: &str) -> Option<i64> {
    let trimmed = input.trim();
    match trimmed {
        "cero" => Some(0),
        "una" | "uno" | "un" => Some(1),
        _ => cardinal::words_to_number(trimmed).map(|n| n as i64),
    }
}

/// Extract AM/PM: "a eme" → "a.m.", "pe eme" → "p.m."
fn extract_ampm(input: &str) -> (&str, Option<String>) {
    let trimmed = input.trim();
    if trimmed.ends_with(" a eme") {
        return (&trimmed[..trimmed.len() - 6], Some("a.m.".to_string()));
    }
    if trimmed.ends_with(" pe eme") {
        return (&trimmed[..trimmed.len() - 7], Some("p.m.".to_string()));
    }
    (trimmed, None)
}

/// Extract "de la tarde" → "p.m.", "de la mañana" → "a.m."
fn extract_de_la(input: &str) -> (&str, Option<String>) {
    let trimmed = input.trim();
    if trimmed.ends_with(" de la tarde") {
        return (&trimmed[..trimmed.len() - 12], Some("p.m.".to_string()));
    }
    if trimmed.ends_with(" de la mañana") {
        return (&trimmed[..trimmed.len() - 13], Some("a.m.".to_string()));
    }
    (trimmed, None)
}

/// Extract timezone: "u t c más cuatro" → "UTC+4"
fn extract_timezone(input: &str) -> (&str, Option<String>) {
    let trimmed = input.trim();
    // "u t c más X"
    if let Some(pos) = trimmed.find(" u t c más ") {
        let before = &trimmed[..pos];
        let tz_num = &trimmed[pos + 11..];
        if let Some(num) = cardinal::words_to_number(tz_num) {
            return (before, Some(format!("UTC+{}", num)));
        }
    }
    if let Some(pos) = trimmed.find(" u t c menos ") {
        let before = &trimmed[..pos];
        let tz_num = &trimmed[pos + 13..];
        if let Some(num) = cardinal::words_to_number(tz_num) {
            return (before, Some(format!("UTC-{}", num)));
        }
    }
    (trimmed, None)
}

/// Format time output
fn format_time(article: &str, hour: i64, minutes: i64, ampm: Option<&str>, tz: Option<&str>) -> String {
    let time = if minutes == 0 && ampm.is_some() {
        format!("{} {}:{:02}", article, hour, minutes)
    } else if minutes > 0 {
        format!("{} {}:{:02}", article, hour, minutes)
    } else {
        format!("{} {}", article, hour)
    };

    let time = if let Some(ap) = ampm {
        format!("{} {}", time, ap)
    } else {
        time
    };

    if let Some(tz_str) = tz {
        format!("{} {}", time, tz_str)
    } else {
        time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digital() {
        assert_eq!(parse("las dieciséis cincuenta"), Some("las 16:50".to_string()));
    }

    #[test]
    fn test_y_cuarto() {
        assert_eq!(parse("la una y cuarto"), Some("la 1:15".to_string()));
    }

    #[test]
    fn test_menos() {
        assert_eq!(parse("las dos menos veinte"), Some("la 1:40".to_string()));
    }

    #[test]
    fn test_para() {
        assert_eq!(parse("cuarto para las dos"), Some("la 1:45".to_string()));
    }
}
