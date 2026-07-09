//! Port of NeMo's `normalize.py` driver: classify → parse tags → permute
//! fields → verbalize → join.
//!
//! NeMo's classifier emits a tagged string such as
//! `tokens { cardinal { integer: "123" } }` (nested, unordered fields). The
//! verbalizer only accepts fields in a specific order, so the driver tries
//! every field permutation of each token until one verbalizes.

use super::engine::apply;
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
    pos: usize,
}

impl TagParser {
    fn new(s: &str) -> Self {
        TagParser {
            chars: s.chars().collect(),
            pos: 0,
        }
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

/// Run the full classify → verbalize pipeline.
///
/// `sep` joins verbalized tokens: `" "` for space-delimited languages (English),
/// `""` for scriptio-continua languages (Chinese). Returns the input unchanged
/// if classification fails (matching NeMo's passthrough for out-of-domain text).
pub fn normalize(
    classify: &VectorFst<TropicalWeight>,
    verbalize: &VectorFst<TropicalWeight>,
    input: &str,
    sep: &str,
) -> String {
    let Some(tagged) = apply(classify, input) else {
        return input.to_string();
    };
    let tokens = TagParser::new(&tagged).fields();

    let mut parts = Vec::with_capacity(tokens.len());
    for (k, v) in tokens {
        let single = vec![(k, v)];
        let mut verbalized = None;
        for candidate in permute(&single) {
            if let Some(out) = apply(verbalize, &candidate) {
                verbalized = Some(out);
                break;
            }
        }
        parts.push(verbalized.unwrap_or_default());
    }
    parts.join(sep)
}
