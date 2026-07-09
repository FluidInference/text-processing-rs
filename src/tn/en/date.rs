//! Date TN tagger.
//!
//! Converts written date expressions to spoken form:
//! - "January 5, 2025" → "january fifth twenty twenty five"
//! - "January 5" → "january fifth"
//! - "1/5/2025" → "january fifth twenty twenty five"
//! - "1980s" → "nineteen eighties"

use super::number_to_words;

const MONTH_NUMBERS: &[(&str, u32)] = &[
    ("january", 1),
    ("february", 2),
    ("march", 3),
    ("april", 4),
    ("may", 5),
    ("june", 6),
    ("july", 7),
    ("august", 8),
    ("september", 9),
    ("october", 10),
    ("november", 11),
    ("december", 12),
];

/// Parse a written date to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Try decade: "1980s" → "nineteen eighties"
    if let Some(result) = parse_decade(trimmed) {
        return Some(result);
    }

    // Try word dates: "January 5, 2025", "jul 25 2012" (US Month-Day) and
    // "25 july 2012" (British Day-Month), with abbreviated/cased months.
    if let Some(result) = parse_word_date(trimmed) {
        return Some(result);
    }

    // Try numeric: "1/5/2025" or "01/05/2025"
    if let Some(result) = parse_numeric_date(trimmed) {
        return Some(result);
    }

    // Fiscal quarter: "2Q22" → "the second quarter of twenty two".
    if let Some(result) = parse_quarter(trimmed) {
        return Some(result);
    }

    // Era: "340 A.D" → "three forty AD".
    if let Some(result) = parse_era(trimmed) {
        return Some(result);
    }

    // Bare 4-digit year → year-style reading ("1994" → "nineteen ninety
    // four"). Out-of-range values fall through to the cardinal tagger.
    if let Some(result) = parse_bare_year(trimmed) {
        return Some(result);
    }

    None
}

/// A standalone 4-digit number in a plausible year range (1000–2099) reads
/// year-style. Other 4-digit numbers are left for the cardinal tagger.
fn parse_bare_year(input: &str) -> Option<String> {
    if input.len() != 4 || !input.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let year: u32 = input.parse().ok()?;
    // Round centuries (1000, 1900, 2000, …) are ambiguous and read better as
    // cardinals ("one thousand", "two thousand"), so leave them.
    if !(1000..=2099).contains(&year) || year.is_multiple_of(100) {
        return None;
    }
    verbalize_year(year)
}

/// Pluralize the final word of a spelled decade into its `-s`/`-ies` form,
/// e.g. `"eighty"` → `"eighties"`, `"hundred"` → `"hundreds"`. Returns `None`
/// for any word that is not a round decade term (so `"zero"` from `"00s"` is
/// rejected rather than becoming `"zeros"`).
fn pluralize_decade_word(word: &str) -> Option<&'static str> {
    let plural = match word {
        "ten" => "tens",
        "twenty" => "twenties",
        "thirty" => "thirties",
        "forty" => "forties",
        "fifty" => "fifties",
        "sixty" => "sixties",
        "seventy" => "seventies",
        "eighty" => "eighties",
        "ninety" => "nineties",
        "hundred" => "hundreds",
        "thousand" => "thousands",
        _ => return None,
    };
    Some(plural)
}

/// Parse decade: "1980s" → "nineteen eighties", "2000s" → "two thousands",
/// "90s"/"'90s" → "nineties". A leading apostrophe (elided century) is
/// accepted; two-digit forms read as the century-less tens word, which the
/// spoken form drops anyway ("'90s" and "1990s" are both "nineties").
fn parse_decade(input: &str) -> Option<String> {
    // NeMo's tokenizer emits spaced decades ("1980 s"); fold the space so the
    // rest of the logic sees the compact "1980s" form.
    let owned;
    let input = if let Some(pre) = input.strip_suffix(" s") {
        owned = format!("{}s", pre);
        owned.as_str()
    } else {
        input
    };

    // Strip the trailing plural `s`, then an optional leading apostrophe.
    // `is_split_punct` keeps `'` attached to the token, so we handle it here.
    let s = input.strip_suffix('s')?;
    let s = s.strip_prefix('\'').unwrap_or(s);

    if s.is_empty() || !s.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    // Two-digit decade ("90s", "'90s") → tens word pluralized. "00s" maps to
    // "zero", which `pluralize_decade_word` rejects, so it is left untouched.
    if s.len() == 2 {
        let tens: u32 = s.parse().ok()?;
        if tens % 10 != 0 {
            return None; // "95s" is not a valid decade
        }
        let word = number_to_words(tens as i64);
        return pluralize_decade_word(&word).map(str::to_string);
    }

    // Four-digit decade ("1980s") → year-style reading, last word pluralized.
    if s.len() != 4 {
        return None;
    }

    let year: u32 = s.parse().ok()?;
    if year < 1000 {
        return None;
    }

    if year % 10 != 0 {
        return None; // "1985s" is not a valid decade
    }

    let year_words = verbalize_year(year)?;
    let words: Vec<&str> = year_words.split_whitespace().collect();
    let last = *words.last()?;
    let plural = pluralize_decade_word(last)?;

    if words.len() == 1 {
        Some(plural.to_string())
    } else {
        let prefix = words[..words.len() - 1].join(" ");
        Some(format!("{} {}", prefix, plural))
    }
}

/// Parse a word date in either order — US `Month Day[, Year]` (`jul 25 2012`
/// → "july twenty fifth …") or British `Day Month [Year]` (`25 july 2012` →
/// "the twenty fifth of july …"). Months may be full or 3-letter
/// abbreviations, any case, with an optional trailing dot.
fn parse_word_date(input: &str) -> Option<String> {
    // Commas separate here, and a hyphen joins month-name forms ("Jan-15",
    // "Jan-15-2020"); sentence mode re-attaches any real commas.
    let cleaned = input.replace([',', '-'], " ");
    let tokens: Vec<&str> = cleaned.split_whitespace().collect();
    if tokens.len() < 2 || tokens.len() > 3 {
        return None;
    }
    let year = if tokens.len() == 3 {
        Some(parse_year_word(tokens[2])?)
    } else {
        None
    };
    // NeMo keeps a comma written before the year ("august 23, 2002" → "august
    // twenty third, two thousand two").
    let comma = year.is_some() && input.contains(',');

    // US order: Month Day [Year].
    if let Some(month) = parse_month(tokens[0]) {
        let day = parse_day(tokens[1])?;
        return Some(with_year(
            format!("{} {}", month, ordinal_word(day)),
            year,
            comma,
        ));
    }

    // British order: Day Month [Year].
    if let Some(day) = parse_day(tokens[0]) {
        let month = parse_month(tokens[1])?;
        return Some(with_year(
            format!("the {} of {}", ordinal_word(day), month),
            year,
            comma,
        ));
    }

    None
}

fn with_year(base: String, year: Option<String>, comma: bool) -> String {
    match year {
        Some(y) if comma => format!("{}, {}", base, y),
        Some(y) => format!("{} {}", base, y),
        None => base,
    }
}

/// Recognize a month name (full or 3-letter abbreviation, any case, optional
/// trailing dot) and return its spoken (lower-case) form.
fn parse_month(token: &str) -> Option<&'static str> {
    let t = token.trim().trim_end_matches('.').to_lowercase();
    Some(match t.as_str() {
        "january" | "jan" => "january",
        "february" | "feb" => "february",
        "march" | "mar" => "march",
        "april" | "apr" => "april",
        "may" => "may",
        "june" | "jun" => "june",
        "july" | "jul" => "july",
        "august" | "aug" => "august",
        "september" | "sep" | "sept" => "september",
        "october" | "oct" => "october",
        "november" | "nov" => "november",
        "december" | "dec" => "december",
        _ => return None,
    })
}

/// Parse a day-of-month token, plain or with an ordinal suffix (`25`, `25th`).
fn parse_day(token: &str) -> Option<u32> {
    let digits = token
        .trim()
        .trim_end_matches("st")
        .trim_end_matches("nd")
        .trim_end_matches("rd")
        .trim_end_matches("th");
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let day: u32 = digits.parse().ok()?;
    (1..=31).contains(&day).then_some(day)
}

/// Parse a bare 4-digit year token year-style. Only a trailing period is
/// tolerated (single-expression input like `"March 8, 2026."` has no
/// pretokenizer to strip it); other trailing punctuation is rejected so a
/// sentence-mode date span cannot silently swallow a following ")" or "]"
/// (the shorter, punctuation-free span wins instead).
fn parse_year_word(token: &str) -> Option<String> {
    let t = token.trim().trim_end_matches('.');
    if t.len() == 4 && t.chars().all(|c| c.is_ascii_digit()) {
        return verbalize_year(t.parse().ok()?);
    }
    None
}

/// Parse a numeric (or month-name) three-field date with `/`, `-`, or `.`
/// separators. Recognizes ISO `YYYY-MM-DD`, US `MM/DD/YYYY` and `MM/DD/YY`,
/// British `DD.MM.YYYY`, and `Mon-DD-YYYY`.
fn parse_numeric_date(input: &str) -> Option<String> {
    let sep = ['/', '-', '.']
        .into_iter()
        .find(|&s| input.matches(s).count() == 2)?;
    let parts: Vec<&str> = input.split(sep).map(|p| p.trim()).collect();
    if parts.len() != 3 {
        return None;
    }
    let (p0, p1, p2) = (parts[0], parts[1], parts[2]);

    // Month name first: "Jan-15-2020" → "january fifteenth twenty twenty".
    if let Some(month) = parse_month(p0) {
        let day = parse_day(p1)?;
        let year = parse_year_field(p2)?;
        return Some(format!("{} {} {}", month, ordinal_word(day), year));
    }

    // ISO year-first: "2006-08-05" → "august fifth two thousand six".
    if p0.len() == 4 {
        let year = p0.parse::<u32>().ok()?;
        let (month, day) = (parse_month_num(p1)?, parse_day(p2)?);
        return Some(format!(
            "{} {} {}",
            month,
            ordinal_word(day),
            verbalize_year(year)?
        ));
    }

    // Otherwise the last field is the year; disambiguate day-first vs
    // month-first by whether the first field can be a month.
    let year = parse_year_field(p2)?;
    let n0: u32 = p0.parse().ok()?;
    let n1: u32 = p1.parse().ok()?;
    if n0 > 12 && (1..=12).contains(&n1) && n0 <= 31 {
        // British "DD-MM-YYYY".
        return Some(format!(
            "the {} of {} {}",
            ordinal_word(n0),
            month_num_name(n1)?,
            year
        ));
    }
    if (1..=12).contains(&n0) && (1..=31).contains(&n1) {
        // US "MM-DD-YYYY".
        return Some(format!(
            "{} {} {}",
            month_num_name(n0)?,
            ordinal_word(n1),
            year
        ));
    }
    None
}

/// Fiscal quarter: "2Q22" / "2q2022" → "the second quarter of <year>".
fn parse_quarter(input: &str) -> Option<String> {
    let (q, rest) = input.split_once(['Q', 'q'])?;
    let quarter: u32 = q.trim().parse().ok()?;
    if !(1..=4).contains(&quarter) {
        return None;
    }
    let rest = rest.trim();
    if !rest.chars().all(|c| c.is_ascii_digit()) || !matches!(rest.len(), 2 | 4) {
        return None;
    }
    let year = verbalize_year(rest.parse().ok()?)?;
    Some(format!("the {} quarter of {}", ordinal_word(quarter), year))
}

/// Era: "340 A.D" / "1200 BC" → "<year-style> AD"/"BC".
fn parse_era(input: &str) -> Option<String> {
    let (num, era) = input.rsplit_once(' ')?;
    let spoken = match era.trim().to_uppercase().replace('.', "").as_str() {
        "AD" => "AD",
        "BC" => "BC",
        _ => return None,
    };
    let num = num.trim();
    if num.is_empty() || !num.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("{} {}", verbalize_year(num.parse().ok()?)?, spoken))
}

/// Recognize a numeric month field (1–12) and return its spoken name.
fn parse_month_num(field: &str) -> Option<&'static str> {
    month_num_name(field.parse().ok()?)
}

fn month_num_name(n: u32) -> Option<&'static str> {
    MONTH_NUMBERS
        .iter()
        .find(|(_, num)| *num == n)
        .map(|(name, _)| *name)
}

/// Year field: 4 digits read year-style; a 2-digit field reads digit-by-digit
/// when it has a leading zero ("05" → "zero five") and cardinal-style otherwise
/// ("98" → "ninety eight").
fn parse_year_field(field: &str) -> Option<String> {
    if !field.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    match field.len() {
        4 => verbalize_year(field.parse().ok()?),
        2 if field.starts_with('0') => Some(super::spell_digits(field)),
        2 => Some(number_to_words(field.parse::<i64>().ok()?)),
        _ => None,
    }
}

/// Verbalize a year.
///
/// - 2025 → "twenty twenty five"
/// - 2000 → "two thousand"
/// - 2001 → "two thousand one"
/// - 1990 → "nineteen ninety"
/// - 1900 → "nineteen hundred"
/// - 800 → "eight hundred"
pub fn verbalize_year(year: u32) -> Option<String> {
    if year == 0 {
        return Some("zero".to_string());
    }

    if year < 100 {
        return Some(number_to_words(year as i64));
    }

    // Years 100-999 read year-style: round hundreds as "N hundred", otherwise
    // split into hundreds + remainder ("340" → "three forty", "105" → "one oh
    // five").
    if year < 1000 {
        let hundreds = year / 100;
        let remainder = year % 100;
        if remainder == 0 {
            return Some(format!("{} hundred", number_to_words(hundreds as i64)));
        }
        let second = if remainder < 10 {
            format!("oh {}", number_to_words(remainder as i64))
        } else {
            number_to_words(remainder as i64)
        };
        return Some(format!("{} {}", number_to_words(hundreds as i64), second));
    }

    let century = year / 100;
    let remainder = year % 100;

    if remainder == 0 {
        // 2000 → "two thousand", 1900 → "nineteen hundred"
        if year % 1000 == 0 {
            let thousands = year / 1000;
            return Some(format!("{} thousand", number_to_words(thousands as i64)));
        }
        return Some(format!("{} hundred", number_to_words(century as i64)));
    }

    // 2001-2009: "two thousand one" through "two thousand nine"
    if century == 20 && remainder < 10 {
        return Some(format!(
            "two thousand {}",
            number_to_words(remainder as i64)
        ));
    }

    // Standard: split into two halves
    // 2025 → "twenty" + "twenty five"
    // 1990 → "nineteen" + "ninety"
    // 1901 → "nineteen" + "oh one"
    let first_half = number_to_words(century as i64);
    let second_half = if remainder < 10 {
        format!("oh {}", number_to_words(remainder as i64))
    } else {
        number_to_words(remainder as i64)
    };

    Some(format!("{} {}", first_half, second_half))
}

/// Convert a day number to its ordinal word form.
fn ordinal_word(n: u32) -> String {
    match n {
        1 => "first".to_string(),
        2 => "second".to_string(),
        3 => "third".to_string(),
        4 => "fourth".to_string(),
        5 => "fifth".to_string(),
        6 => "sixth".to_string(),
        7 => "seventh".to_string(),
        8 => "eighth".to_string(),
        9 => "ninth".to_string(),
        10 => "tenth".to_string(),
        11 => "eleventh".to_string(),
        12 => "twelfth".to_string(),
        13 => "thirteenth".to_string(),
        14 => "fourteenth".to_string(),
        15 => "fifteenth".to_string(),
        16 => "sixteenth".to_string(),
        17 => "seventeenth".to_string(),
        18 => "eighteenth".to_string(),
        19 => "nineteenth".to_string(),
        20 => "twentieth".to_string(),
        21 => "twenty first".to_string(),
        22 => "twenty second".to_string(),
        23 => "twenty third".to_string(),
        24 => "twenty fourth".to_string(),
        25 => "twenty fifth".to_string(),
        26 => "twenty sixth".to_string(),
        27 => "twenty seventh".to_string(),
        28 => "twenty eighth".to_string(),
        29 => "twenty ninth".to_string(),
        30 => "thirtieth".to_string(),
        31 => "thirty first".to_string(),
        _ => {
            // Fallback for larger numbers (shouldn't happen for days)
            let cardinal = number_to_words(n as i64);
            format!("{}th", cardinal)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_month_day() {
        assert_eq!(parse("January 5"), Some("january fifth".to_string()));
        assert_eq!(
            parse("December 25"),
            Some("december twenty fifth".to_string())
        );
    }

    #[test]
    fn test_abbreviated_and_cased_months() {
        assert_eq!(
            parse("jul 25 2012"),
            Some("july twenty fifth twenty twelve".to_string())
        );
        assert_eq!(parse("SEPT. 15"), Some("september fifteenth".to_string()));
        assert_eq!(
            parse("Jan. 15 2020"),
            Some("january fifteenth twenty twenty".to_string())
        );
    }

    #[test]
    fn test_british_day_month() {
        assert_eq!(
            parse("25 july 2012"),
            Some("the twenty fifth of july twenty twelve".to_string())
        );
        assert_eq!(
            parse("15 january"),
            Some("the fifteenth of january".to_string())
        );
        assert_eq!(
            parse("16 July 1943"),
            Some("the sixteenth of july nineteen forty three".to_string())
        );
        assert_eq!(
            parse("25th july 2012"),
            Some("the twenty fifth of july twenty twelve".to_string())
        );
    }

    #[test]
    fn test_month_day_year() {
        // A written comma before the year is kept (NeMo).
        assert_eq!(
            parse("January 5, 2025"),
            Some("january fifth, twenty twenty five".to_string())
        );
        assert_eq!(
            parse("July 4, 1776"),
            Some("july fourth, seventeen seventy six".to_string())
        );
        assert_eq!(
            parse("January 5 2025"),
            Some("january fifth twenty twenty five".to_string())
        );
    }

    #[test]
    fn test_bare_year() {
        assert_eq!(parse("1994"), Some("nineteen ninety four".to_string()));
        assert_eq!(parse("2012"), Some("twenty twelve".to_string()));
        assert_eq!(parse("1155"), Some("eleven fifty five".to_string()));
        // Round centuries and out-of-range values fall through to cardinal.
        assert_eq!(parse("1000"), None);
        assert_eq!(parse("2000"), None);
        assert_eq!(parse("9000"), None);
        assert_eq!(parse("123"), None);
    }

    #[test]
    fn test_decade() {
        assert_eq!(parse("1980s"), Some("nineteen eighties".to_string()));
        assert_eq!(parse("2000s"), Some("two thousands".to_string()));
        assert_eq!(parse("1990s"), Some("nineteen nineties".to_string()));
        assert_eq!(parse("1900s"), Some("nineteen hundreds".to_string()));
        assert_eq!(parse("2010s"), Some("twenty tens".to_string()));
    }

    #[test]
    fn test_two_digit_and_apostrophe_decade() {
        // Two-digit and apostrophe-elided decades read as the tens word.
        assert_eq!(parse("90s"), Some("nineties".to_string()));
        assert_eq!(parse("'90s"), Some("nineties".to_string()));
        assert_eq!(parse("80s"), Some("eighties".to_string()));
        assert_eq!(parse("'20s"), Some("twenties".to_string()));
        assert_eq!(parse("10s"), Some("tens".to_string()));
        assert_eq!(parse("'1980s"), Some("nineteen eighties".to_string()));
    }

    #[test]
    fn test_invalid_decade() {
        assert_eq!(parse("95s"), None); // not a round decade
        assert_eq!(parse("00s"), None); // "zero" is not a decade word
        assert_eq!(parse("'00s"), None);
        assert_eq!(parse("5s"), None); // single digit
        assert_eq!(parse("'s"), None); // no digits
    }

    #[test]
    fn test_numeric_date() {
        assert_eq!(
            parse("1/5/2025"),
            Some("january fifth twenty twenty five".to_string())
        );
        assert_eq!(
            parse("12/25/2000"),
            Some("december twenty fifth two thousand".to_string())
        );
    }

    #[test]
    fn test_year_verbalization() {
        assert_eq!(verbalize_year(2025), Some("twenty twenty five".to_string()));
        assert_eq!(verbalize_year(2000), Some("two thousand".to_string()));
        assert_eq!(verbalize_year(2001), Some("two thousand one".to_string()));
        assert_eq!(verbalize_year(1990), Some("nineteen ninety".to_string()));
        assert_eq!(verbalize_year(1900), Some("nineteen hundred".to_string()));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
    }
}
