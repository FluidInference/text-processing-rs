//! Time TN tagger.
//!
//! Converts written clock times to spoken form (NeMo conventions):
//! - "01:00" → "one o'clock"      (minute zero, no meridiem)
//! - "23:00" → "twenty three o'clock" (24-hour clock kept as-is)
//! - "01:00 am" → "one AM"        (minute zero + meridiem)
//! - "1:59 p.m. est" → "one fifty nine PM EST"
//! - "1:01:01" → "one hour one minute and one second"
//! - "5pm" → "five PM", "1.59 p.m." → "one fifty nine PM"

use super::number_to_words;

/// Meridiem spellings, longest first so dotted forms win over bare ones.
const MERIDIEMS: &[(&str, &str)] = &[
    ("a.m.", "AM"),
    ("p.m.", "PM"),
    ("a.m", "AM"),
    ("p.m", "PM"),
    ("am", "AM"),
    ("pm", "PM"),
];

/// Time-zone spellings (dotted variants first).
const TIMEZONES: &[(&str, &str)] = &[
    ("e.s.t", "EST"),
    ("est", "EST"),
    ("p.s.t", "PST"),
    ("pst", "PST"),
    ("c.s.t", "CST"),
    ("cst", "CST"),
    ("m.s.t", "MST"),
    ("mst", "MST"),
    ("g.m.t", "GMT"),
    ("gmt", "GMT"),
    ("u.t.c", "UTC"),
    ("utc", "UTC"),
];

/// Parse a written time to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let lower = input.trim().to_lowercase();

    // Peel an optional trailing time zone, then an optional meridiem.
    let (rest, tz) = strip_suffix(&lower, TIMEZONES);
    let (core, meridiem) = strip_suffix(rest.trim_end(), MERIDIEMS);
    let core = core.trim();

    let core_words = if core.contains(':') {
        // "… o'clock" only when nothing else marks the hour (no meridiem).
        spell_colon_time(core, meridiem.is_none())?
    } else if meridiem.is_some() {
        // Bare hour ("5pm") or dotted ("1.59 p.m.") only reads as a time when
        // a meridiem anchors it; otherwise it's a plain number/decimal.
        spell_bare_or_dotted(core)?
    } else {
        return None;
    };

    let mut result = core_words;
    if let Some(m) = meridiem {
        result.push(' ');
        result.push_str(m);
    }
    if let Some(z) = tz {
        result.push(' ');
        result.push_str(z);
    }
    Some(result)
}

/// Strip the longest matching suffix from `input` (case already lowered),
/// returning the remainder and the mapped spelling.
fn strip_suffix<'a>(
    input: &'a str,
    table: &[(&str, &'static str)],
) -> (&'a str, Option<&'static str>) {
    for &(pat, spoken) in table {
        if let Some(rest) = input.strip_suffix(pat) {
            return (rest, Some(spoken));
        }
    }
    (input, None)
}

/// Spell `H:MM` or `H:MM:SS`. `allow_oclock` renders an on-the-hour `H:MM`
/// as "… o'clock" (false when a meridiem already marks the hour).
fn spell_colon_time(core: &str, allow_oclock: bool) -> Option<String> {
    let parts: Vec<&str> = core.split(':').collect();
    let nums: Vec<u32> = parts
        .iter()
        .map(|p| p.trim())
        .map(|p| {
            if p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty() {
                p.parse().ok()
            } else {
                None
            }
        })
        .collect::<Option<Vec<u32>>>()?;

    match nums.as_slice() {
        [hour, minute] => {
            let (hour, minute) = (*hour, *minute);
            if hour > 23 || minute > 59 {
                return None;
            }
            Some(spell_hour_minute(hour, minute, allow_oclock))
        }
        [hour, minute, second] => {
            let (hour, minute, second) = (*hour, *minute, *second);
            if hour > 23 || minute > 59 || second > 59 {
                return None;
            }
            // Verbose reading: "one hour one minute and one second".
            Some(format!(
                "{} {} {} {} and {} {}",
                number_to_words(hour as i64),
                unit(hour, "hour"),
                number_to_words(minute as i64),
                unit(minute, "minute"),
                number_to_words(second as i64),
                unit(second, "second"),
            ))
        }
        _ => None,
    }
}

/// Spell a bare hour ("5") or a dotted `H.MM` ("1.59"); the caller has already
/// confirmed a meridiem is present.
fn spell_bare_or_dotted(core: &str) -> Option<String> {
    if let Some((h, m)) = core.split_once('.') {
        let (h, m) = (parse_u32(h)?, parse_u32(m)?);
        if h > 23 || m > 59 {
            return None;
        }
        return Some(spell_hour_minute(h, m, false));
    }
    let hour = parse_u32(core)?;
    if hour > 23 {
        return None;
    }
    // Bare hour with a meridiem is on the hour.
    Some(number_to_words(hour as i64))
}

/// `H:MM` core words. With a meridiem present the on-the-hour form is just the
/// hour ("one"); otherwise it reads "… o'clock".
fn spell_hour_minute(hour: u32, minute: u32, allow_oclock: bool) -> String {
    let hour_words = number_to_words(hour as i64);
    if minute == 0 {
        if allow_oclock {
            format!("{} o'clock", hour_words)
        } else {
            hour_words
        }
    } else if minute < 10 {
        format!("{} oh {}", hour_words, number_to_words(minute as i64))
    } else {
        format!("{} {}", hour_words, number_to_words(minute as i64))
    }
}

fn parse_u32(s: &str) -> Option<u32> {
    let t = s.trim();
    if t.is_empty() || !t.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    t.parse().ok()
}

fn unit(n: u32, singular: &str) -> String {
    if n == 1 {
        singular.to_string()
    } else {
        format!("{}s", singular)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oclock_and_24h() {
        assert_eq!(parse("01:00"), Some("one o'clock".to_string()));
        assert_eq!(parse("23:00"), Some("twenty three o'clock".to_string()));
        assert_eq!(parse("2:30"), Some("two thirty".to_string()));
        assert_eq!(parse("2:05"), Some("two oh five".to_string()));
    }

    #[test]
    fn test_meridiem() {
        assert_eq!(parse("01:00 am"), Some("one AM".to_string()));
        assert_eq!(parse("01:00 a.m."), Some("one AM".to_string()));
        assert_eq!(parse("1:59 p.m."), Some("one fifty nine PM".to_string()));
        assert_eq!(parse("5pm"), Some("five PM".to_string()));
        assert_eq!(parse("1 a.m."), Some("one AM".to_string()));
        assert_eq!(parse("1:00a.m."), Some("one AM".to_string()));
    }

    #[test]
    fn test_timezone() {
        assert_eq!(
            parse("1:59 p.m. est"),
            Some("one fifty nine PM EST".to_string())
        );
        assert_eq!(
            parse("1:59 p.m.est"),
            Some("one fifty nine PM EST".to_string())
        );
        assert_eq!(
            parse("1:59 p.m. e.s.t"),
            Some("one fifty nine PM EST".to_string())
        );
    }

    #[test]
    fn test_dotted_meridiem() {
        assert_eq!(parse("1.59 p.m."), Some("one fifty nine PM".to_string()));
    }

    #[test]
    fn test_hms_verbose() {
        assert_eq!(
            parse("1:01:01"),
            Some("one hour one minute and one second".to_string())
        );
        assert_eq!(
            parse("14:10:30"),
            Some("fourteen hours ten minutes and thirty seconds".to_string())
        );
        assert_eq!(
            parse("10:00:00 p.m. e.s.t"),
            Some("ten hours zero minutes and zero seconds PM EST".to_string())
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("25:00"), None);
        assert_eq!(parse("12:60"), None);
        assert_eq!(parse("5"), None); // bare number, no meridiem
    }
}
