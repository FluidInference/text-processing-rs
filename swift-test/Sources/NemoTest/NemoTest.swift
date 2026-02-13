import Foundation
import CNemoTextProcessing

/// Swift wrapper for NeMo Text Processing
enum NemoTextProcessing {
    static func normalize(_ input: String) -> String {
        guard let resultPtr = nemo_normalize(input) else {
            return input
        }
        defer { nemo_free_string(resultPtr) }
        return String(cString: resultPtr)
    }

    static var version: String {
        guard let versionPtr = nemo_version() else {
            return "unknown"
        }
        return String(cString: versionPtr)
    }
}

@main
struct NemoTest {
    static func main() {
        let args = CommandLine.arguments
        if args.contains("--nltagger") {
            runNLTaggerTests()
            return
        }

        print("NeMo Text Processing v\(NemoTextProcessing.version)")
        print(String(repeating: "=", count: 50))
        print()

        let testCases = [
            // Cardinals
            ("one", "1"),
            ("twenty one", "21"),
            ("one hundred", "100"),
            ("two thousand and twenty five", "2025"),

            // Money
            ("five dollars", "$5"),
            ("five dollars and fifty cents", "$5.50"),
            ("one point five billion dollars", "$1.5 billion"),

            // Dates
            ("january first", "january 1"),
            ("july fourth twenty twenty", "july 4 2020"),

            // Time
            ("two thirty", "02:30"),
            ("quarter past one", "01:15"),
            ("two pm", "02:00 p.m."),

            // Measurements
            ("seventy two degrees fahrenheit", "72 °F"),
            ("one hundred meters", "100 m"),

            // Electronic
            ("test at gmail dot com", "test@gmail.com"),

            // Passthrough
            ("hello world", "hello world"),
        ]

        var passed = 0
        var failed = 0

        for (input, expected) in testCases {
            let result = NemoTextProcessing.normalize(input)
            let status = result == expected ? "✓" : "✗"

            if result == expected {
                passed += 1
                print("\(status) \"\(input)\" → \"\(result)\"")
            } else {
                failed += 1
                print("\(status) \"\(input)\" → \"\(result)\" (expected \"\(expected)\")")
            }
        }

        print()
        print(String(repeating: "=", count: 50))
        print("Results: \(passed)/\(passed + failed) passed")
    }
}
