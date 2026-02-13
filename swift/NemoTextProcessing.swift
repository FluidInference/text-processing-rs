import Foundation

/// Swift wrapper for NeMo Text Processing (Inverse Text Normalization).
///
/// Converts spoken-form ASR output to written form:
/// - "two hundred thirty two" → "232"
/// - "five dollars and fifty cents" → "$5.50"
/// - "january fifth twenty twenty five" → "January 5, 2025"
/// - "period" → "."
public enum NemoTextProcessing {

    // MARK: - Normalization

    /// Normalize spoken-form text to written form.
    ///
    /// Tries to match the entire input as a single expression.
    /// Use `normalizeSentence` for inputs containing mixed natural language and spoken forms.
    ///
    /// - Parameter input: Spoken-form text from ASR
    /// - Returns: Written-form text, or original if no normalization applies
    public static func normalize(_ input: String) -> String {
        guard let cString = input.cString(using: .utf8) else {
            return input
        }

        guard let resultPtr = nemo_normalize(cString) else {
            return input
        }

        defer { nemo_free_string(resultPtr) }

        return String(cString: resultPtr)
    }

    /// Normalize a full sentence, replacing spoken-form spans with written form.
    ///
    /// Scans for normalizable spans within a larger sentence using a sliding window.
    /// Uses a default max span of 16 tokens.
    ///
    /// - Parameter input: Sentence containing spoken-form spans
    /// - Returns: Sentence with spoken-form spans replaced
    ///
    /// Example:
    /// ```swift
    /// let result = NemoTextProcessing.normalizeSentence("I have twenty one apples")
    /// // result is "I have 21 apples"
    /// ```
    public static func normalizeSentence(_ input: String) -> String {
        guard let cString = input.cString(using: .utf8) else {
            return input
        }

        guard let resultPtr = nemo_normalize_sentence(cString) else {
            return input
        }

        defer { nemo_free_string(resultPtr) }

        return String(cString: resultPtr)
    }

    /// Normalize a full sentence with a configurable max span size.
    ///
    /// - Parameters:
    ///   - input: Sentence containing spoken-form spans
    ///   - maxSpanTokens: Maximum consecutive tokens per normalizable span (default 16)
    /// - Returns: Sentence with spoken-form spans replaced
    public static func normalizeSentence(_ input: String, maxSpanTokens: UInt32) -> String {
        guard let cString = input.cString(using: .utf8) else {
            return input
        }

        guard let resultPtr = nemo_normalize_sentence_with_max_span(cString, maxSpanTokens) else {
            return input
        }

        defer { nemo_free_string(resultPtr) }

        return String(cString: resultPtr)
    }

    // MARK: - Custom Rules

    /// Add a custom spoken→written normalization rule.
    ///
    /// Custom rules have the highest priority, checked before all built-in taggers.
    /// Matching is case-insensitive on the spoken form.
    /// If a rule with the same spoken form exists, it is replaced.
    ///
    /// - Parameters:
    ///   - spoken: The spoken form to match (e.g., "gee pee tee")
    ///   - written: The written replacement (e.g., "GPT")
    public static func addRule(spoken: String, written: String) {
        spoken.withCString { spokenPtr in
            written.withCString { writtenPtr in
                nemo_add_rule(spokenPtr, writtenPtr)
            }
        }
    }

    /// Remove a custom normalization rule.
    ///
    /// - Parameter spoken: The spoken form to remove
    /// - Returns: True if the rule was found and removed
    @discardableResult
    public static func removeRule(spoken: String) -> Bool {
        return spoken.withCString { spokenPtr in
            nemo_remove_rule(spokenPtr) != 0
        }
    }

    /// Clear all custom normalization rules.
    public static func clearRules() {
        nemo_clear_rules()
    }

    /// The number of custom rules currently registered.
    public static var ruleCount: Int {
        Int(nemo_rule_count())
    }

    // MARK: - Info

    /// Get the library version.
    public static var version: String {
        guard let versionPtr = nemo_version() else {
            return "unknown"
        }
        return String(cString: versionPtr)
    }
}
