//! Source-alignment coverage for the compiled-FST text normalizer.
//!
//! Requires the `fst-engine` feature:
//! `cargo test --features ffi,fst-engine --test fst_alignment`.
#![cfg(feature = "fst-engine")]

use text_processing_rs::fst::{self, AlignedNormalization, AlignedSpan, TokenKind};

fn assert_source_ranges(input: &str, alignment: &AlignedNormalization) {
    let mut previous_end = 0;

    for span in &alignment.spans {
        assert!(span.input_start >= previous_end, "source spans overlap");
        assert!(
            span.input_start <= span.input_end,
            "source span is reversed"
        );
        assert!(input.is_char_boundary(span.input_start));
        assert!(input.is_char_boundary(span.input_end));
        assert_eq!(&input[span.input_start..span.input_end], span.original);
        previous_end = span.input_end;
    }
}

#[test]
fn aligns_a_money_span_within_a_complete_sentence() {
    let input = "The price is $1,234.56.";
    let alignment = fst::normalize_aligned(input, "en").expect("English input should normalize");

    assert_eq!(
        alignment.normalized,
        "The price is one thousand two hundred and thirty four dollars fifty six cents."
    );
    assert_eq!(
        alignment.spans,
        vec![
            AlignedSpan {
                input_start: 0,
                input_end: 3,
                original: "The".into(),
                normalized: "The".into(),
                kind: TokenKind::Word,
            },
            AlignedSpan {
                input_start: 4,
                input_end: 9,
                original: "price".into(),
                normalized: "price".into(),
                kind: TokenKind::Word,
            },
            AlignedSpan {
                input_start: 10,
                input_end: 12,
                original: "is".into(),
                normalized: "is".into(),
                kind: TokenKind::Word,
            },
            AlignedSpan {
                input_start: 13,
                input_end: 22,
                original: "$1,234.56".into(),
                normalized: "one thousand two hundred and thirty four dollars fifty six cents"
                    .into(),
                kind: TokenKind::Money,
            },
            AlignedSpan {
                input_start: 22,
                input_end: 23,
                original: ".".into(),
                normalized: ".".into(),
                kind: TokenKind::Punctuation,
            },
        ]
    );
    assert_source_ranges(input, &alignment);
    assert_eq!(alignment.normalized, fst::en::normalize(input));
}

#[test]
fn reports_utf8_byte_offsets() {
    let input = "Café costs €12.50.";
    let alignment = fst::normalize_aligned(input, "en").expect("English input should normalize");

    assert_eq!(alignment.spans[0].original, "Café");
    assert_eq!(
        (alignment.spans[0].input_start, alignment.spans[0].input_end),
        (0, 5)
    );
    assert_eq!(alignment.spans[2].original, "€12.50");
    assert_eq!(
        (alignment.spans[2].input_start, alignment.spans[2].input_end),
        (12, 20)
    );
    assert_eq!(alignment.spans[2].kind, TokenKind::Money);
    assert_source_ranges(input, &alignment);
}

#[test]
fn distinguishes_electronic_date_and_punctuation_spans() {
    let input = "Email jane.doe@example.com on 12/31/2025.";
    let alignment = fst::normalize_aligned(input, "en").expect("English input should normalize");

    let mapped = alignment
        .spans
        .iter()
        .map(|span| {
            (
                span.original.as_str(),
                span.normalized.as_str(),
                span.kind.as_str(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        mapped,
        vec![
            ("Email", "Email", "word"),
            (
                "jane.doe@example.com",
                "jane dot doe at example dot com",
                "electronic",
            ),
            ("on", "on", "word"),
            (
                "12/31/2025",
                "december thirty first twenty twenty five",
                "date",
            ),
            (".", ".", "punctuation"),
        ]
    );
    assert_source_ranges(input, &alignment);
}

#[test]
fn classifies_a_phone_number_inside_a_sentence() {
    let input = "Call 415-555-0123.";
    let alignment = fst::normalize_aligned(input, "en").expect("English input should normalize");

    assert_eq!(
        alignment.normalized,
        "Call four one five, five five five, zero one two three."
    );
    assert_eq!(alignment.spans[1].original, "415-555-0123");
    assert_eq!(
        alignment.spans[1].normalized,
        "four one five, five five five, zero one two three"
    );
    assert_eq!(alignment.spans[1].kind, TokenKind::Telephone);
    assert_eq!(
        (alignment.spans[1].input_start, alignment.spans[1].input_end),
        (5, 17)
    );
    assert_source_ranges(input, &alignment);
}

#[test]
fn aligns_spanish_currency_with_non_ascii_offsets() {
    let input = "El precio es 1.234,56 €.";
    let alignment = fst::normalize_aligned(input, "es").expect("Spanish input should normalize");

    assert_eq!(
        alignment.normalized,
        "El precio es mil doscientos treinta y cuatro coma cincuenta y seis euros."
    );
    assert_eq!(alignment.spans[3].original, "1.234,56 €");
    assert_eq!(
        alignment.spans[3].normalized,
        "mil doscientos treinta y cuatro coma cincuenta y seis euros"
    );
    assert_eq!(alignment.spans[3].kind, TokenKind::Money);
    assert_eq!(
        (alignment.spans[3].input_start, alignment.spans[3].input_end),
        (13, 25)
    );
    assert_source_ranges(input, &alignment);
    assert_eq!(alignment.normalized, fst::es::normalize(input));
}

#[cfg(feature = "ffi")]
#[test]
fn exposes_alignment_through_the_c_ffi() {
    use std::ffi::{CStr, CString};

    use text_processing_rs::ffi::{nemo_tn_alignment_free, nemo_tn_fst_aligned};

    unsafe {
        let input = CString::new("The price is $1,234.56.").unwrap();
        let en = CString::new("en").unwrap();
        let result = nemo_tn_fst_aligned(input.as_ptr(), en.as_ptr());

        assert!(!result.is_null());
        let alignment = &*result;
        assert_eq!(alignment.span_count, 5);
        assert_eq!(
            CStr::from_ptr(alignment.normalized).to_str().unwrap(),
            "The price is one thousand two hundred and thirty four dollars fifty six cents."
        );

        let spans = std::slice::from_raw_parts(alignment.spans, alignment.span_count);
        let money = &spans[3];
        assert_eq!((money.input_start, money.input_end), (13, 22));
        assert_eq!(
            CStr::from_ptr(money.original).to_str().unwrap(),
            "$1,234.56"
        );
        assert_eq!(CStr::from_ptr(money.kind).to_str().unwrap(), "money");

        nemo_tn_alignment_free(result);
        nemo_tn_alignment_free(std::ptr::null_mut());

        let unsupported = CString::new("xx").unwrap();
        assert!(nemo_tn_fst_aligned(input.as_ptr(), unsupported.as_ptr()).is_null());
    }
}

#[test]
fn rejects_an_unsupported_language() {
    assert!(fst::normalize_aligned("$12.50", "xx").is_none());
}
