//! Cardinal number tagger for Korean.
//!
//! Converts Sino-Korean numerals to Arabic numerals:
//! - "일" → "1"
//! - "이천십일" → "2011"
//! - "오천억" → "500,000,000,000"
//! - "십오만" → "150,000"
//!
//! ## Why Korean is more conservative than Japanese / Chinese
//!
//! The `ja` and `zh` cardinal taggers aggressively convert *every*
//! maximal run of numeral characters, because Han digit glyphs
//! (一二三 / 十百千) rarely appear as standalone words in running
//! prose. Korean Sino-numeral *syllables* are different: 이 ("this"),
//! 일 ("day" / "work"), 사 ("buy"), 구 ("phrase"), 만 (the "only"
//! particle), 천 ("cloth"), 조 ("clause") are all extremely common
//! non-numeric morphemes. A blind run-scan would shred ordinary text.
//!
//! So [`replace_sino_korean_numbers`] only rewrites a run when it
//! contains at least one *unit* character (십/백/천/만/억/조) — a
//! reliable signal that the run is genuinely numeric. Bare digit-only
//! runs (이일, 삼사) are left verbatim; the date / time / ordinal
//! taggers handle digit-only spans because their suffixes
//! (년/월/일/시/분/초/번째) supply the disambiguating context.

/// Map a single Sino-Korean digit syllable to its value.
pub fn sino_korean_digit(c: char) -> Option<i64> {
    match c {
        '영' | '공' => Some(0),
        '일' => Some(1),
        '이' => Some(2),
        '삼' => Some(3),
        '사' => Some(4),
        '오' => Some(5),
        '육' => Some(6),
        '칠' => Some(7),
        '팔' => Some(8),
        '구' => Some(9),
        _ => None,
    }
}

/// Check if a character is a Sino-Korean scale unit.
pub fn is_sino_korean_unit(c: char) -> bool {
    matches!(c, '십' | '백' | '천' | '만' | '억' | '조')
}

/// Check if a character is a Sino-Korean numeral (digit or scale unit).
pub fn is_sino_korean_numeral(c: char) -> bool {
    sino_korean_digit(c).is_some() || is_sino_korean_unit(c)
}

/// Parse a Sino-Korean number string to an integer.
///
/// Handles the full Sino-Korean number system:
/// - Scale: 조(10^12), 억(10^8), 만(10^4)
/// - Within each group: 천(1000), 백(100), 십(10) + digits
/// - Implicit leading 1: bare 천 → 1000, 만 → 10000, 십 → 10
/// - Mixed Arabic + Korean unit: "15만" → 150000
///
/// Returns `None` if the string is empty or contains a non-numeral.
///
/// Examples:
/// - "일" → 1
/// - "이십" → 20
/// - "이천십일" → 2011
/// - "오천억" → 500_000_000_000
/// - "십오만" → 150_000
pub fn sino_korean_to_number(input: &str) -> Option<i64> {
    if input.is_empty() {
        return None;
    }
    if !input
        .chars()
        .all(|c| is_sino_korean_numeral(c) || c.is_ascii_digit())
    {
        return None;
    }

    let mut total: i64 = 0;
    let mut section: i64 = 0;
    let mut current: i64 = 0;
    let mut have_current = false;

    for c in input.chars() {
        if let Some(d) = c.to_digit(10) {
            // Arabic digit — accumulate into the in-progress base value
            // so mixed forms like "15만" combine as 15 × 10000.
            current = current * 10 + d as i64;
            have_current = true;
        } else if let Some(d) = sino_korean_digit(c) {
            current = d;
            have_current = true;
        } else {
            match c {
                '십' => {
                    let v = if have_current { current } else { 1 };
                    section += v * 10;
                    current = 0;
                    have_current = false;
                }
                '백' => {
                    let v = if have_current { current } else { 1 };
                    section += v * 100;
                    current = 0;
                    have_current = false;
                }
                '천' => {
                    let v = if have_current { current } else { 1 };
                    section += v * 1000;
                    current = 0;
                    have_current = false;
                }
                '만' => {
                    let v = section + current;
                    let v = if v == 0 { 1 } else { v };
                    total += v * 10_000;
                    section = 0;
                    current = 0;
                    have_current = false;
                }
                '억' => {
                    let v = section + current;
                    let v = if v == 0 { 1 } else { v };
                    total += v * 100_000_000;
                    section = 0;
                    current = 0;
                    have_current = false;
                }
                '조' => {
                    let v = section + current;
                    let v = if v == 0 { 1 } else { v };
                    total += v * 1_000_000_000_000;
                    section = 0;
                    current = 0;
                    have_current = false;
                }
                _ => return None,
            }
        }
    }

    Some(total + section + current)
}

/// Format a number with comma separators.
pub fn format_with_commas(n: i64) -> String {
    if n == 0 {
        return "0".to_string();
    }

    let negative = n < 0;
    let mut num = (n as i128).unsigned_abs();
    let mut groups: Vec<u128> = Vec::new();

    while num > 0 {
        groups.push(num % 1000);
        num /= 1000;
    }
    groups.reverse();

    let mut result = groups[0].to_string();
    for g in &groups[1..] {
        result.push(',');
        result.push_str(&format!("{:03}", g));
    }

    if negative {
        format!("-{}", result)
    } else {
        result
    }
}

/// Split a numeral run at boundaries where a Korean digit is more
/// likely a homographic morpheme than part of the number.
///
/// Two boundaries are recognised:
///
/// 1. **Digit-after-digit.** Sino-Korean numbers carry at most one
///    digit per slot between units, so a Korean digit immediately
///    following another Korean digit ends the numeric expression —
///    typically the start of a classifier. Without this split,
///    `십오일` ("15" + "day") would parse positionally as 10 + 1 = 11
///    because the trailing 일 (digit 1) overwrites 오 (digit 5).
///
/// 2. **Trailing lone digit after a big scale unit.** A single Korean
///    digit at the very end of a run, immediately after 만/억/조, is
///    far more often a subject/object particle (이/가/은/를 …) than a
///    genuine `+X` term — `오천억이` is "500 billion" + 이, not
///    500,000,000,002. (`십X` / `백X` / `천X` are *not* split: there a
///    trailing digit reliably fills the ones slot, e.g. `이천십일`.)
///
/// Arabic digits never trigger a split since they are naturally
/// multi-digit (`15만`, `2025년`).
fn split_at_korean_digit_boundaries(run: &str) -> Vec<String> {
    let chars: Vec<char> = run.chars().collect();
    let mut out = Vec::new();
    let mut cur = String::new();
    for (idx, &c) in chars.iter().enumerate() {
        if sino_korean_digit(c).is_some() && !cur.is_empty() {
            let prev = chars[idx - 1];
            let after_digit = sino_korean_digit(prev).is_some();
            let trailing_after_big_unit =
                idx == chars.len() - 1 && matches!(prev, '만' | '억' | '조');
            if after_digit || trailing_after_big_unit {
                out.push(std::mem::take(&mut cur));
            }
        }
        cur.push(c);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Find and replace Sino-Korean number spans in a string.
///
/// Conservative by design (see module docs): a maximal numeral run is
/// only rewritten when it contains at least one scale unit
/// (십/백/천/만/억/조). Digit-only runs are left untouched so common
/// homographs (이, 일, 사 …) survive.
pub fn replace_sino_korean_numbers(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < chars.len() {
        if is_sino_korean_numeral(chars[i]) || chars[i].is_ascii_digit() {
            let start = i;
            while i < chars.len() && (is_sino_korean_numeral(chars[i]) || chars[i].is_ascii_digit())
            {
                i += 1;
            }
            let run: String = chars[start..i].iter().collect();

            // Homograph guard: only rewrite runs that carry a scale unit.
            if !run.chars().any(is_sino_korean_unit) {
                result.push_str(&run);
                continue;
            }

            for seg in split_at_korean_digit_boundaries(&run) {
                match sino_korean_to_number(&seg) {
                    // A split-off digit-only segment is a homograph
                    // (the 일 in 십오일, the 이 particle in 오천억이) —
                    // keep it verbatim rather than emitting a stray digit.
                    Some(_)
                        if !seg.chars().any(is_sino_korean_unit)
                            && seg.chars().all(|c| sino_korean_digit(c).is_some()) =>
                    {
                        result.push_str(&seg);
                    }
                    Some(num) => result.push_str(&format_with_commas(num)),
                    None => result.push_str(&seg),
                }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        assert_eq!(sino_korean_to_number("일"), Some(1));
        assert_eq!(sino_korean_to_number("십"), Some(10));
        assert_eq!(sino_korean_to_number("이십"), Some(20));
        assert_eq!(sino_korean_to_number("백"), Some(100));
        assert_eq!(sino_korean_to_number("이십오"), Some(25));
    }

    #[test]
    fn test_parse_large() {
        assert_eq!(sino_korean_to_number("이천십일"), Some(2011));
        assert_eq!(sino_korean_to_number("천구백사십"), Some(1940));
        assert_eq!(sino_korean_to_number("오천억"), Some(500_000_000_000));
        assert_eq!(sino_korean_to_number("일만"), Some(10_000));
        assert_eq!(sino_korean_to_number("삼억오천만"), Some(350_000_000));
    }

    #[test]
    fn test_parse_mixed_arabic() {
        assert_eq!(sino_korean_to_number("15만"), Some(150_000));
        assert_eq!(sino_korean_to_number("12만"), Some(120_000));
    }

    #[test]
    fn test_commas() {
        assert_eq!(format_with_commas(1), "1");
        assert_eq!(format_with_commas(1000), "1,000");
        assert_eq!(format_with_commas(150_000), "150,000");
        assert_eq!(format_with_commas(500_000_000_000), "500,000,000,000");
    }

    #[test]
    fn test_replace_requires_unit() {
        // Run with a unit char → converted.
        assert_eq!(replace_sino_korean_numbers("이천십일"), "2,011");
        assert_eq!(replace_sino_korean_numbers("십오만 원"), "150,000 원");
        // Digit-only run → left verbatim (homograph guard).
        assert_eq!(replace_sino_korean_numbers("이 사람"), "이 사람");
        assert_eq!(replace_sino_korean_numbers("이것은"), "이것은");
    }

    #[test]
    fn test_replace_contextual() {
        assert_eq!(
            replace_sino_korean_numbers("예산은 오천억이 되었다"),
            "예산은 500,000,000,000이 되었다"
        );
    }

    #[test]
    fn test_replace_splits_trailing_digit_homograph() {
        // 십오일 = "15" + 일("day"); the split keeps 일 verbatim
        // instead of folding it into a positional 11.
        assert_eq!(replace_sino_korean_numbers("십오일"), "15일");
        // 오천억이 = "500 billion" + 이(particle), not 500,000,000,002.
        assert_eq!(replace_sino_korean_numbers("오천억이"), "500,000,000,000이");
        // 이천십일 keeps the trailing 일 in the ones slot → 2011.
        assert_eq!(replace_sino_korean_numbers("이천십일"), "2,011");
    }
}
