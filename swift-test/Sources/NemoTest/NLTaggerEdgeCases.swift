/// NLTagger edge case tests for context-aware punctuation spotting.
///
/// Tests that Apple's NLTagger correctly identifies when ambiguous words
/// like "period", "dash", "colon" are used as natural language vs punctuation.
///
/// Run with: swift run NemoTest --nltagger

import Foundation
import NaturalLanguage

/// Words that are both punctuation spoken forms AND common English words.
let ambiguousWords: Set<String> = [
    "period", "dash", "colon", "pipe", "slash", "dot", "plus", "hash", "percent",
]

struct NLTaggerTestCase {
    let sentence: String
    let word: String
    let expectedTag: String  // "noun", "verb", "adjective", "other", etc.
    let shouldNormalize: Bool  // false = NLTagger should protect it
}

func runNLTaggerTests() {
    let tagger = NLTagger(tagSchemes: [.lexicalClass])

    let testCases: [NLTaggerTestCase] = [
        // "period" as a noun (time period) — should NOT be normalized
        NLTaggerTestCase(
            sentence: "that was the best period of my life",
            word: "period",
            expectedTag: "Noun",
            shouldNormalize: false
        ),
        // "period" at end of statement (punctuation intent) — debatable, but NLTagger sees it as noun
        NLTaggerTestCase(
            sentence: "end of the period",
            word: "period",
            expectedTag: "Noun",
            shouldNormalize: false
        ),
        // "dash" as a verb (running) — should NOT be normalized
        NLTaggerTestCase(
            sentence: "I need to dash to the store",
            word: "dash",
            expectedTag: "Verb",
            shouldNormalize: false
        ),
        // "dash" as a noun (a dash of salt) — should NOT be normalized
        NLTaggerTestCase(
            sentence: "add a dash of salt",
            word: "dash",
            expectedTag: "Noun",
            shouldNormalize: false
        ),
        // "colon" as a noun (body part) — should NOT be normalized
        NLTaggerTestCase(
            sentence: "the doctor examined my colon",
            word: "colon",
            expectedTag: "Noun",
            shouldNormalize: false
        ),
        // "period" standalone — could be punctuation command
        NLTaggerTestCase(
            sentence: "period",
            word: "period",
            expectedTag: "Noun",
            shouldNormalize: true  // standalone = punctuation intent
        ),
        // "dot" in tech context
        NLTaggerTestCase(
            sentence: "press the red dot",
            word: "dot",
            expectedTag: "Noun",
            shouldNormalize: false
        ),
        // "plus" as adjective/preposition
        NLTaggerTestCase(
            sentence: "that is a plus for us",
            word: "plus",
            expectedTag: "Noun",
            shouldNormalize: false
        ),
        // "hash" as a noun (hashtag)
        NLTaggerTestCase(
            sentence: "use the hash symbol",
            word: "hash",
            expectedTag: "Noun",
            shouldNormalize: false
        ),
        // "percent" in natural context
        NLTaggerTestCase(
            sentence: "fifty percent of people agree",
            word: "percent",
            expectedTag: "Noun",
            shouldNormalize: false
        ),
    ]

    var passed = 0
    var failed = 0

    print("=== NLTagger Edge Case Tests ===\n")

    for tc in testCases {
        tagger.string = tc.sentence

        // Find the word range
        guard let range = tc.sentence.range(of: tc.word) else {
            print("  SKIP: Could not find '\(tc.word)' in '\(tc.sentence)'")
            continue
        }

        let (tag, _) = tagger.tag(at: range.lowerBound, unit: .word, scheme: .lexicalClass)
        let tagName = tag?.rawValue ?? "nil"

        let isNaturalLanguage = tag == .noun || tag == .verb || tag == .adjective || tag == .adverb
        let isStandalone = tc.sentence.split(separator: " ").count == 1
        let wouldNormalize = !isNaturalLanguage || isStandalone

        let passNormalize = (wouldNormalize == tc.shouldNormalize)

        if passNormalize {
            passed += 1
            print("  PASS: '\(tc.sentence)' — '\(tc.word)' tagged as \(tagName), normalize=\(wouldNormalize)")
        } else {
            failed += 1
            print("  FAIL: '\(tc.sentence)' — '\(tc.word)' tagged as \(tagName), normalize=\(wouldNormalize) (expected \(tc.shouldNormalize))")
        }
    }

    print("\n=== Results: \(passed)/\(passed + failed) passed ===")

    if failed > 0 {
        print("\n⚠ Some NLTagger results may vary by OS version. The tagger is a heuristic.")
    }
}
