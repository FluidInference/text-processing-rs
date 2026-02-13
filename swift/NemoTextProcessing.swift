import Foundation

/// Swift wrapper for NeMo Text Processing (Inverse Text Normalization).
///
/// Converts spoken-form ASR output to written form:
/// - "two hundred thirty two" → "232"
/// - "five dollars and fifty cents" → "$5.50"
/// - "january fifth twenty twenty five" → "January 5, 2025"
public enum NemoTextProcessing {

    /// Normalize spoken-form text to written form.
    ///
    /// - Parameter input: Spoken-form text from ASR
    /// - Returns: Written-form text, or original if no normalization applies
    ///
    /// Example:
    /// ```swift
    /// let result = NemoTextProcessing.normalize("two hundred")
    /// // result is "200"
    /// ```
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

    /// Get the library version.
    public static var version: String {
        guard let versionPtr = nemo_version() else {
            return "unknown"
        }
        return String(cString: versionPtr)
    }
}
