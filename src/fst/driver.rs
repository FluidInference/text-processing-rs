//! Port of NeMo's `normalize.py` driver: classify → parse tags → permute
//! fields → verbalize → join.
//!
//! NeMo's classifier emits a tagged string such as
//! `tokens { cardinal { integer: "123" } }` (nested, unordered fields). The
//! verbalizer only accepts fields in a specific order, so the driver tries
//! every field permutation of each token until one verbalizes.

use super::engine::{apply, apply_with_alignment};
use super::{AlignedNormalization, AlignedSpan, TokenKind};
use rustfst::prelude::*;

/// A parsed tag value: a leaf string, a nested tag, or a bare boolean flag.
#[derive(Clone)]
enum Val {
    Str(String),
    Map(Vec<(String, Val)>),
    Bool,
}

/// Recursive-descent parser over the classifier's tagged output.
struct TagParser {
    chars: Vec<char>,
    byte_offsets: Vec<usize>,
    byte_len: usize,
    pos: usize,
}

impl TagParser {
    fn new(s: &str) -> Self {
        TagParser {
            chars: s.chars().collect(),
            byte_offsets: s.char_indices().map(|(offset, _)| offset).collect(),
            byte_len: s.len(),
            pos: 0,
        }
    }

    fn byte_position(&self) -> usize {
        self.byte_offsets
            .get(self.pos)
            .copied()
            .unwrap_or(self.byte_len)
    }

    fn skip_ws(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn key(&mut self) -> String {
        let mut k = String::new();
        while self.pos < self.chars.len()
            && (self.chars[self.pos].is_ascii_alphanumeric() || self.chars[self.pos] == '_')
        {
            k.push(self.chars[self.pos]);
            self.pos += 1;
        }
        k
    }

    /// Parse a `{ ... }` body into an ordered list of `(key, value)` pairs.
    fn fields(&mut self) -> Vec<(String, Val)> {
        let mut out = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.chars.len() || self.chars[self.pos] == '}' {
                break;
            }
            let k = self.key();
            if k.is_empty() {
                break;
            }
            self.skip_ws();
            if k == "preserve_order" {
                // `preserve_order: true` — a flag, not a nested value. Its
                // presence pins field order (skip permutation).
                if self.pos < self.chars.len() && self.chars[self.pos] == ':' {
                    self.pos += 1;
                }
                self.skip_ws();
                while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_alphabetic() {
                    self.pos += 1;
                }
                out.push((k, Val::Bool));
            } else {
                out.push((k, self.value()));
            }
        }
        out
    }

    /// Parse root fields while retaining the output byte boundary at which
    /// each top-level `tokens { ... }` record ends. The selected FST path maps
    /// that boundary back to an input byte position.
    fn aligned_fields(&mut self) -> Vec<(String, Val, usize)> {
        let mut out = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.chars.len() || self.chars[self.pos] == '}' {
                break;
            }
            let key = self.key();
            if key.is_empty() {
                break;
            }
            let value = self.value();
            out.push((key, value, self.byte_position()));
        }
        out
    }

    /// Parse either a `: "quoted string"` value or a nested `{ ... }` tag.
    ///
    /// A quoted value ends at a `"` *followed by a space* (or end of input), so
    /// internal quotes are preserved — matching NeMo's `parse_string_value`
    /// (e.g. `name: ""He's"` yields the value `"He's`).
    fn value(&mut self) -> Val {
        self.skip_ws();
        if self.pos < self.chars.len() && self.chars[self.pos] == ':' {
            self.pos += 1;
            self.skip_ws();
            if self.pos < self.chars.len() && self.chars[self.pos] == '"' {
                self.pos += 1;
            }
            let mut s = String::new();
            while self.pos < self.chars.len() {
                if self.chars[self.pos] == '"'
                    && (self.pos + 1 >= self.chars.len() || self.chars[self.pos + 1] == ' ')
                {
                    break;
                }
                s.push(self.chars[self.pos]);
                self.pos += 1;
            }
            if self.pos < self.chars.len() {
                self.pos += 1; // closing quote
            }
            Val::Str(s)
        } else {
            if self.pos < self.chars.len() && self.chars[self.pos] == '{' {
                self.pos += 1;
            }
            let inner = self.fields();
            self.skip_ws();
            if self.pos < self.chars.len() && self.chars[self.pos] == '}' {
                self.pos += 1;
            }
            Val::Map(inner)
        }
    }
}

/// All permutations of a slice (Heap-style, recursive).
fn perms<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut out = Vec::new();
    for i in 0..items.len() {
        let mut rest = items.to_vec();
        let head = rest.remove(i);
        for mut p in perms(&rest) {
            p.insert(0, head.clone());
            out.push(p);
        }
    }
    out
}

/// Reconstruct tagged strings for every field ordering of `pairs`, recursing
/// into nested tags. Mirrors NeMo's `_permute`: when a `preserve_order` flag is
/// present the original order is kept; otherwise all orderings are produced.
fn permute(pairs: &[(String, Val)]) -> Vec<String> {
    let pinned = pairs.iter().any(|(k, _)| k == "preserve_order");
    let orderings = if pinned {
        vec![pairs.to_vec()]
    } else {
        perms(pairs)
    };
    let mut out = Vec::new();
    for perm in orderings {
        let mut variants = vec![String::new()];
        for (k, v) in &perm {
            match v {
                Val::Str(s) => {
                    variants = variants
                        .iter()
                        .map(|x| format!("{}{}: \"{}\" ", x, k, s))
                        .collect()
                }
                Val::Bool => {
                    variants = variants
                        .iter()
                        .map(|x| format!("{}{}: true ", x, k))
                        .collect()
                }
                Val::Map(inner) => {
                    let rec = permute(inner);
                    let mut next = Vec::new();
                    for x in &variants {
                        for r in &rec {
                            next.push(format!("{} {} {{ {} }} ", x, k, r));
                        }
                    }
                    variants = next;
                }
            }
        }
        out.extend(variants);
    }
    out
}

/// Run the full classify → verbalize → post-process pipeline, reproducing
/// NeMo's `normalize(..., punct_post_process=True)` end to end.
///
/// `sep` joins verbalized tokens: `" "` for space-delimited languages (English,
/// French, …), `""` for scriptio-continua languages (Chinese, Japanese).
/// `post` is the optional post-processing FST (English/Hindi have one; others
/// pass `None`). Returns the input unchanged if classification fails (matching
/// NeMo's passthrough for out-of-domain text).
pub fn normalize(
    classify: &VectorFst<TropicalWeight>,
    verbalize: &VectorFst<TropicalWeight>,
    post: Option<&VectorFst<TropicalWeight>>,
    input: &str,
    sep: &str,
) -> String {
    let Some(tagged) = apply(classify, input) else {
        return input.to_string();
    };
    let tokens = TagParser::new(&tagged).fields();

    let mut parts = Vec::with_capacity(tokens.len());
    for (k, v) in tokens {
        parts.push(verbalize_token(verbalize, k, v).unwrap_or_default());
    }

    finish_normalization(parts, post, input, sep)
}

/// Run TN while retaining the input span consumed by every top-level NeMo
/// classifier token.
pub(super) fn normalize_aligned(
    classify: &VectorFst<TropicalWeight>,
    verbalize: &VectorFst<TropicalWeight>,
    post: Option<&VectorFst<TropicalWeight>>,
    input: &str,
    sep: &str,
) -> Option<AlignedNormalization> {
    let classified = apply_with_alignment(classify, input)?;
    let tokens = TagParser::new(&classified.output).aligned_fields();
    let mut parts = Vec::with_capacity(tokens.len());
    let mut spans = Vec::with_capacity(tokens.len());
    let mut previous_input_end = 0usize;

    for (key, value, output_end) in tokens {
        let raw_input_end = source_char_boundary(
            input,
            classified
                .input_at_output_boundary
                .get(output_end)
                .copied()
                .unwrap_or(input.len())
                .min(input.len()),
        );
        let (input_start, input_end) =
            trim_source_span(input, previous_input_end.min(raw_input_end), raw_input_end);
        previous_input_end = raw_input_end;

        let original = input[input_start..input_end].to_string();
        let kind = token_kind(&value, &original);
        let normalized = verbalize_token(verbalize, key, value).unwrap_or_default();
        parts.push(normalized.clone());
        spans.push(AlignedSpan {
            input_start,
            input_end,
            original,
            normalized,
            kind,
        });
    }

    Some(AlignedNormalization {
        normalized: finish_normalization(parts, post, input, sep),
        spans,
    })
}

fn source_char_boundary(input: &str, mut position: usize) -> usize {
    while !input.is_char_boundary(position) {
        position -= 1;
    }
    position
}

fn verbalize_token(
    verbalize: &VectorFst<TropicalWeight>,
    key: String,
    value: Val,
) -> Option<String> {
    let single = vec![(key, value)];
    for candidate in permute(&single) {
        if let Some(output) = apply(verbalize, &candidate) {
            return Some(output);
        }
    }
    None
}

fn finish_normalization(
    parts: Vec<String>,
    post: Option<&VectorFst<TropicalWeight>>,
    input: &str,
    sep: &str,
) -> String {
    // NeMo's post-verbalization steps (normalize.py): collapse spaces, apply the
    // post-processing FST, Moses-detokenize, then re-align punctuation spacing to
    // the original input.
    let joined = collapse_spaces(parts.join(sep));
    let processed = match post {
        Some(pp) => apply(pp, &joined).unwrap_or(joined),
        None => joined,
    };
    post_process_punct(input, &moses_despace(&processed))
}

fn trim_source_span(input: &str, start: usize, end: usize) -> (usize, usize) {
    let raw = &input[start..end];
    let trimmed_start = raw.trim_start_matches(char::is_whitespace);
    let leading_bytes = raw.len() - trimmed_start.len();
    let trimmed = trimmed_start.trim_end_matches(char::is_whitespace);
    (start + leading_bytes, start + leading_bytes + trimmed.len())
}

fn token_kind(value: &Val, original: &str) -> TokenKind {
    let class = match value {
        Val::Map(fields) => fields
            .iter()
            .map(|(key, _)| key.as_str())
            .find(|key| *key != "preserve_order"),
        _ => None,
    };

    match class {
        Some("cardinal") => TokenKind::Cardinal,
        Some("ordinal") => TokenKind::Ordinal,
        Some("decimal") => TokenKind::Decimal,
        Some("fraction") => TokenKind::Fraction,
        Some("time") => TokenKind::Time,
        Some("measure") => TokenKind::Measure,
        Some("percent") => TokenKind::Percent,
        Some("date") => TokenKind::Date,
        Some("telephone") => TokenKind::Telephone,
        Some("money") => TokenKind::Money,
        Some("electronic") => TokenKind::Electronic,
        Some("verbatim") => TokenKind::Verbatim,
        Some("letters") => TokenKind::Letters,
        Some("abbreviation") => TokenKind::Abbreviation,
        Some("name") | None
            if !original.is_empty()
                && original
                    .chars()
                    .all(|c| !c.is_alphanumeric() && !c.is_whitespace()) =>
        {
            TokenKind::Punctuation
        }
        Some("name") | None => TokenKind::Word,
        Some(other) => TokenKind::Other(other.to_string()),
    }
}

/// Collapse runs of spaces to one and trim (NeMo's `SPACE_DUP` + strip).
fn collapse_spaces(s: String) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c == ' ' {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

/// The one Moses-detokenizer behavior NeMo's outputs depend on: drop a space
/// before sentence/clause punctuation. `post_process_punct` then restores any
/// spacing the original input actually had.
fn moses_despace(s: &str) -> String {
    let mut out = s.to_string();
    for p in [" .", " ,", " !", " ?", " ;", " :"] {
        out = out.replace(p, &p[1..]);
    }
    out
}

/// Port of NeMo's `data_loader_utils.post_process_punct`: for each punctuation
/// mark present in `input`, align the spacing around that mark in `normalized`
/// to match the input. This is what makes `978-0` read `…eight-zero` (no spaces)
/// while `9 - 0` keeps them.
fn post_process_punct(input: &str, normalized: &str) -> String {
    // NeMo replaces `` with " in the input before aligning (the post-processing
    // WFST emits " for `` quotes).
    let rewritten;
    let input: &str = if input.contains("``") && !normalized.contains("``") {
        rewritten = input.replace("``", "\"");
        &rewritten
    } else {
        input
    };
    let inp: Vec<char> = input.chars().collect();
    // Each cell starts as one char and may grow a trailing space or empty out.
    let mut nt: Vec<String> = normalized.chars().map(|c| c.to_string()).collect();
    let punctuation = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
    let space = " ".to_string();

    for p in punctuation.chars().filter(|c| inp.contains(c)) {
        let ps = p.to_string();
        let equal =
            inp.iter().filter(|&&c| c == p).count() == nt.iter().filter(|s| **s == ps).count();
        let (mut idx_in, mut idx_out) = (0usize, 0usize);
        while inp[idx_in..].contains(&p) {
            let Some(io) = (idx_out..nt.len()).find(|&i| nt[i] == ps) else {
                break;
            };
            let Some(ii) = (idx_in..inp.len()).find(|&i| inp[i] == p) else {
                break;
            };
            idx_out = io;
            idx_in = ii;
            let is_valid =
                (idx_out > 0 && idx_in > 0 && nt[idx_out - 1] == inp[idx_in - 1].to_string())
                    || (idx_out < nt.len() - 1
                        && idx_in < inp.len() - 1
                        && nt[idx_out + 1] == inp[idx_in + 1].to_string());
            if !equal && !is_valid {
                idx_in += 1;
                continue;
            }
            if idx_in > 0 && idx_out > 0 {
                if nt[idx_out - 1] == space && inp[idx_in - 1] != ' ' {
                    nt[idx_out - 1] = String::new();
                } else if nt[idx_out - 1] != space && inp[idx_in - 1] == ' ' {
                    nt[idx_out - 1].push(' ');
                }
            }
            if idx_in < inp.len() - 1 && idx_out < nt.len() - 1 {
                if nt[idx_out + 1] == space && inp[idx_in + 1] != ' ' {
                    nt[idx_out + 1] = String::new();
                } else if nt[idx_out + 1] != space && inp[idx_in + 1] == ' ' {
                    nt[idx_out].push(' ');
                }
            }
            idx_out += 1;
            idx_in += 1;
        }
    }
    // NeMo ends with `re.sub(' +', ' ')` — collapse runs but do NOT trim.
    let joined = nt.concat();
    let mut out = String::with_capacity(joined.len());
    let mut prev_space = false;
    for c in joined.chars() {
        if c == ' ' {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out
}
