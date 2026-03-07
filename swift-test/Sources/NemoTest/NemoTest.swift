import Foundation
import CNemoTextProcessing

// MARK: - Wrapper

enum NemoTextProcessing {
    static func normalize(_ input: String) -> String {
        guard let resultPtr = nemo_normalize(input) else { return input }
        defer { nemo_free_string(resultPtr) }
        return String(cString: resultPtr)
    }

    static func normalizeSentence(_ input: String) -> String {
        guard let resultPtr = nemo_normalize_sentence(input) else { return input }
        defer { nemo_free_string(resultPtr) }
        return String(cString: resultPtr)
    }

    static func normalizeSentence(_ input: String, maxSpanTokens: UInt32) -> String {
        guard let resultPtr = nemo_normalize_sentence_with_max_span(input, maxSpanTokens) else { return input }
        defer { nemo_free_string(resultPtr) }
        return String(cString: resultPtr)
    }

    // MARK: - Text Normalization (written → spoken)

    static func tnNormalize(_ input: String) -> String {
        guard let resultPtr = nemo_tn_normalize(input) else { return input }
        defer { nemo_free_string(resultPtr) }
        return String(cString: resultPtr)
    }

    static func tnNormalizeSentence(_ input: String) -> String {
        guard let resultPtr = nemo_tn_normalize_sentence(input) else { return input }
        defer { nemo_free_string(resultPtr) }
        return String(cString: resultPtr)
    }

    static func tnNormalizeSentence(_ input: String, maxSpanTokens: UInt32) -> String {
        guard let resultPtr = nemo_tn_normalize_sentence_with_max_span(input, maxSpanTokens) else { return input }
        defer { nemo_free_string(resultPtr) }
        return String(cString: resultPtr)
    }

    static func addRule(spoken: String, written: String) {
        nemo_add_rule(spoken, written)
    }

    @discardableResult
    static func removeRule(spoken: String) -> Bool {
        nemo_remove_rule(spoken) != 0
    }

    static func clearRules() { nemo_clear_rules() }

    static var ruleCount: Int { Int(nemo_rule_count()) }

    static var version: String {
        guard let p = nemo_version() else { return "unknown" }
        return String(cString: p)
    }
}

// MARK: - Test Harness

typealias TC = (input: String, expected: String)

struct TestResults {
    var passed = 0
    var failed = 0
    var failures: [(group: String, input: String, expected: String, got: String)] = []
    var total: Int { passed + failed }
}

func runGroup(_ name: String, cases: [TC], transform: (String) -> String) -> TestResults {
    var r = TestResults()
    print("  \(name)")
    for tc in cases {
        let got = transform(tc.input)
        if got == tc.expected {
            r.passed += 1
        } else {
            r.failed += 1
            r.failures.append((name, tc.input, tc.expected, got))
            print("    FAIL \"\(tc.input)\" -> \"\(got)\" (expected \"\(tc.expected)\")")
        }
    }
    return r
}

// MARK: - Test Data

let cardinalTests: [TC] = [
    ("zero", "zero"),
    ("one", "1"),
    ("twenty one", "21"),
    ("one hundred", "100"),
    ("two hundred and fifty four", "254"),
    ("one thousand thirteen", "1013"),
    ("two thousand and twenty five", "2025"),
    ("one million one hundred fifty six thousand one hundred seventy three", "1156173"),
    ("minus sixty", "-60"),
    ("minus twenty five thousand thirty seven", "-25037"),
]

let ordinalTests: [TC] = [
    ("first", "1st"),
    ("second", "2nd"),
    ("third", "3rd"),
    ("fourth", "4th"),
    ("eleventh", "11th"),
    ("twelfth", "12th"),
    ("twenty first", "21st"),
    ("twenty third", "23rd"),
    ("one hundredth", "100th"),
    ("one thousandth", "1000th"),
]

let moneyTests: [TC] = [
    ("one dollar", "$1"),
    ("two dollars", "$2"),
    ("one cent", "$0.01"),
    ("five dollars and fifty cents", "$5.50"),
    ("nine hundred ninety three dollars and ninety two cents", "$993.92"),
    ("fifteen thousand dollars", "$15000"),
    ("fifty million dollars", "$50 million"),
    ("two point five billion dollars", "$2.5 billion"),
    ("thirty billion yen", "\u{00A5}30 billion"),
]

let timeTests: [TC] = [
    ("two thirty", "02:30"),
    ("three o'clock", "03:00"),
    ("quarter past one", "01:15"),
    ("half past three", "03:30"),
    ("two p m", "02:00 p.m."),
    ("eight fifty one", "08:51"),
    ("eleven fifty five p m", "11:55 p.m."),
    ("seven a m e s t", "07:00 a.m. est"),
]

let dateTests: [TC] = [
    ("january first", "january 1"),
    ("june thirty", "june 30"),
    ("july twenty fifth twenty twelve", "july 25 2012"),
    ("nineteen eighties", "1980s"),
    ("twenty twelve", "2012"),
    ("nineteen seventeen", "1917"),
    ("the fifteenth of january", "15 january"),
    ("the twenty fifth of july twenty twelve", "25 july 2012"),
]

let decimalTests: [TC] = [
    ("five point two million", "5.2 million"),
    ("fifty billion", "50 billion"),
    ("four hundred million", "400 million"),
    ("four point eight five billion", "4.85 billion"),
    ("one hundred thirty two billion", "132 billion"),
]

let measureTests: [TC] = [
    ("two hundred meters", "200 m"),
    ("two hundred kilometers per hour", "200 km/h"),
    ("minus sixty six kilograms", "-66 kg"),
    ("ninety grams", "90 g"),
    ("three hours", "3 h"),
    ("four hundred forty milliliters", "440 ml"),
    ("seventy two degrees fahrenheit", "72 \u{00B0}F"),
]

let electronicTests: [TC] = [
    ("a at gmail dot com", "a@gmail.com"),
    ("a dot b c at nvidia dot com", "a.bc@nvidia.com"),
    ("a b c at g mail dot a b c", "abc@gmail.abc"),
    ("h t t p colon slash slash w w w dot o u r d a i l y n e w s dot com dot s m",
     "http://www.ourdailynews.com.sm"),
    ("w w w dot c o m d a i l y n e w s dot a b slash s m",
     "www.comdailynews.ab/sm"),
]

let telephoneTests: [TC] = [
    ("one two three one two three five six seven eight", "123-123-5678"),
    ("plus nine one one two three one two three five six seven eight", "+91 123-123-5678"),
    ("seven nine nine", "799"),
    ("a b nine", "ab9"),
    ("x eighty six", "x86"),
]

let whitelistTests: [TC] = [
    ("doctor dao", "dr. dao"),
    ("misses smith", "mrs. smith"),
    ("mister dao", "mr. dao"),
    ("saint george", "st. george"),
    ("s and p five hundred", "S&P 500"),
    ("r t x", "RTX"),
]

let sentenceTests: [TC] = [
    ("I have twenty one apples", "I have 21 apples"),
    ("the price is five dollars and fifty cents", "the price is $5.50"),
    ("she arrived at two thirty", "she arrived at 02:30"),
    ("hello world", "hello world"),
    ("call me at two p m on january first", "call me at 02:00 p.m. on january 1"),
]

// MARK: - TN Test Data (written → spoken)

let tnCardinalTests: [TC] = [
    ("0", "zero"),
    ("1", "one"),
    ("21", "twenty one"),
    ("100", "one hundred"),
    ("123", "one hundred twenty three"),
    ("1000", "one thousand"),
    ("1,000,000", "one million"),
    ("-42", "minus forty two"),
]

let tnOrdinalTests: [TC] = [
    ("1st", "first"),
    ("2nd", "second"),
    ("3rd", "third"),
    ("4th", "fourth"),
    ("11th", "eleventh"),
    ("12th", "twelfth"),
    ("21st", "twenty first"),
    ("100th", "one hundredth"),
]

let tnMoneyTests: [TC] = [
    ("$1", "one dollar"),
    ("$5", "five dollars"),
    ("$5.50", "five dollars and fifty cents"),
    ("$0.01", "one cent"),
    ("$0.99", "ninety nine cents"),
    ("\u{00A3}1", "one pound"),
    ("\u{20AC}100", "one hundred euros"),
    ("$2.5 billion", "two point five billion dollars"),
]

let tnTimeTests: [TC] = [
    ("2:30", "two thirty"),
    ("2:05", "two oh five"),
    ("2:00 PM", "two p m"),
    ("2:30 PM", "two thirty p m"),
    ("8:15 AM", "eight fifteen a m"),
    ("14:00", "two p m"),
]

let tnDateTests: [TC] = [
    ("January 5", "january fifth"),
    ("December 25", "december twenty fifth"),
    ("January 5, 2025", "january fifth twenty twenty five"),
    ("1980s", "nineteen eighties"),
    ("1990s", "nineteen nineties"),
]

let tnDecimalTests: [TC] = [
    ("3.14", "three point one four"),
    ("0.5", "zero point five"),
    ("1.5 billion", "one point five billion"),
]

let tnMeasureTests: [TC] = [
    ("200 km/h", "two hundred kilometers per hour"),
    ("1 kg", "one kilogram"),
    ("2 kg", "two kilograms"),
    ("-66 kg", "minus sixty six kilograms"),
    ("50%", "fifty percent"),
]

let tnElectronicTests: [TC] = [
    ("test@gmail.com", "t e s t at g m a i l dot c o m"),
    ("http://www.example.com", "h t t p colon slash slash w w w dot e x a m p l e dot c o m"),
    ("www.example.com", "w w w dot e x a m p l e dot c o m"),
]

let tnTelephoneTests: [TC] = [
    ("123-456-7890", "one two three, four five six, seven eight nine zero"),
    ("(555) 123-4567", "five five five, one two three, four five six seven"),
]

let tnWhitelistTests: [TC] = [
    ("Dr.", "doctor"),
    ("Mrs.", "misses"),
    ("Mr.", "mister"),
    ("e.g.", "for example"),
]

let tnSentenceTests: [TC] = [
    ("I paid $5 for 23 items", "I paid five dollars for twenty three items"),
    ("hello world", "hello world"),
]

// MARK: - Main

@main
struct NemoTest {
    static func main() {
        let args = CommandLine.arguments

        if args.contains("--nltagger") {
            runNLTaggerTests()
            return
        }

        let verbose = args.contains("--verbose") || args.contains("-v")
        let filterCategory = args.dropFirst().first(where: { !$0.hasPrefix("-") })

        print("NeMo Text Processing v\(NemoTextProcessing.version)")
        print(String(repeating: "=", count: 60))
        print()

        var overall = TestResults()

        // ITN: spoken → written
        let itnGroups: [(String, [TC], (String) -> String)] = [
            ("Cardinals", cardinalTests, NemoTextProcessing.normalize),
            ("Ordinals", ordinalTests, NemoTextProcessing.normalize),
            ("Money", moneyTests, NemoTextProcessing.normalize),
            ("Time", timeTests, NemoTextProcessing.normalize),
            ("Dates", dateTests, NemoTextProcessing.normalize),
            ("Decimals", decimalTests, NemoTextProcessing.normalize),
            ("Measurements", measureTests, NemoTextProcessing.normalize),
            ("Electronic", electronicTests, NemoTextProcessing.normalize),
            ("Telephone", telephoneTests, NemoTextProcessing.normalize),
            ("Whitelist", whitelistTests, NemoTextProcessing.normalize),
            ("Sentences", sentenceTests, NemoTextProcessing.normalizeSentence),
        ]

        // TN: written → spoken
        let tnGroups: [(String, [TC], (String) -> String)] = [
            ("TN-Cardinals", tnCardinalTests, NemoTextProcessing.tnNormalize),
            ("TN-Ordinals", tnOrdinalTests, NemoTextProcessing.tnNormalize),
            ("TN-Money", tnMoneyTests, NemoTextProcessing.tnNormalize),
            ("TN-Time", tnTimeTests, NemoTextProcessing.tnNormalize),
            ("TN-Dates", tnDateTests, NemoTextProcessing.tnNormalize),
            ("TN-Decimals", tnDecimalTests, NemoTextProcessing.tnNormalize),
            ("TN-Measurements", tnMeasureTests, NemoTextProcessing.tnNormalize),
            ("TN-Electronic", tnElectronicTests, NemoTextProcessing.tnNormalize),
            ("TN-Telephone", tnTelephoneTests, NemoTextProcessing.tnNormalize),
            ("TN-Whitelist", tnWhitelistTests, NemoTextProcessing.tnNormalize),
            ("TN-Sentences", tnSentenceTests, NemoTextProcessing.tnNormalizeSentence),
        ]

        let allGroups = itnGroups + tnGroups

        for (name, cases, transform) in allGroups {
            if let filter = filterCategory, !name.lowercased().hasPrefix(filter.lowercased()) {
                continue
            }
            let r = runGroup(name, cases: cases, transform: transform)
            overall.passed += r.passed
            overall.failed += r.failed
            overall.failures += r.failures

            let mark = r.failed == 0 ? "PASS" : "FAIL"
            print("    \(mark) \(r.passed)/\(r.total)")
            print()
        }

        // Custom rules test
        if filterCategory == nil || "custom".hasPrefix(filterCategory!.lowercased()) {
            print("  Custom Rules")
            var cr = TestResults()

            NemoTextProcessing.clearRules()
            NemoTextProcessing.addRule(spoken: "gee pee tee", written: "GPT")
            NemoTextProcessing.addRule(spoken: "ell ell em", written: "LLM")

            func check(_ label: String, _ got: String, _ expected: String) {
                if got == expected {
                    cr.passed += 1
                } else {
                    cr.failed += 1
                    cr.failures.append(("Custom Rules", label, expected, got))
                    print("    FAIL \(label): \"\(got)\" (expected \"\(expected)\")")
                }
            }

            check("normalize(gee pee tee)", NemoTextProcessing.normalize("gee pee tee"), "GPT")
            check("normalize(ell ell em)", NemoTextProcessing.normalize("ell ell em"), "LLM")
            check("ruleCount", "\(NemoTextProcessing.ruleCount)", "2")

            NemoTextProcessing.removeRule(spoken: "gee pee tee")
            check("ruleCount after remove", "\(NemoTextProcessing.ruleCount)", "1")

            NemoTextProcessing.clearRules()
            check("ruleCount after clear", "\(NemoTextProcessing.ruleCount)", "0")

            let mark = cr.failed == 0 ? "PASS" : "FAIL"
            print("    \(mark) \(cr.passed)/\(cr.total)")
            print()

            overall.passed += cr.passed
            overall.failed += cr.failed
            overall.failures += cr.failures
        }

        // Summary
        print(String(repeating: "=", count: 60))
        print("Total: \(overall.passed)/\(overall.total) passed", terminator: "")
        if overall.failed > 0 {
            print(" (\(overall.failed) failed)")
            if verbose {
                print("\nFailures:")
                for f in overall.failures {
                    print("  [\(f.group)] \"\(f.input)\" -> \"\(f.got)\" (expected \"\(f.expected)\")")
                }
            }
            Foundation.exit(1)
        } else {
            print()
            Foundation.exit(0)
        }
    }
}
