import Foundation
import CNemoTextProcessing

// MARK: - Wrapper

enum Nemo {
    static func normalize(_ input: String) -> String {
        guard let ptr = nemo_normalize(input) else { return input }
        defer { nemo_free_string(ptr) }
        return String(cString: ptr)
    }

    static func normalizeSentence(_ input: String) -> String {
        guard let ptr = nemo_normalize_sentence(input) else { return input }
        defer { nemo_free_string(ptr) }
        return String(cString: ptr)
    }

    static func addRule(spoken: String, written: String) {
        nemo_add_rule(spoken, written)
    }

    static func clearRules() { nemo_clear_rules() }

    static var version: String {
        guard let p = nemo_version() else { return "unknown" }
        return String(cString: p)
    }
}

// MARK: - CLI

let usage = """
    nemo-itn - Inverse Text Normalization

    USAGE:
      nemo-itn <spoken text>          Normalize a single expression
      nemo-itn -s <sentence>          Normalize spans within a sentence
      echo "text" | nemo-itn          Read from stdin (one line per output)
      echo "text" | nemo-itn -s       Sentence mode from stdin
      nemo-itn --version              Show version
      nemo-itn --help                 Show this help

    EXAMPLES:
      nemo-itn two hundred thirty two
      nemo-itn five dollars and fifty cents
      nemo-itn -s "I have twenty one apples"
      echo "quarter past one" | nemo-itn
    """

var args = Array(CommandLine.arguments.dropFirst())

if args.isEmpty && isatty(fileno(stdin)) != 0 {
    fputs(usage, stderr)
    exit(1)
}

if args.contains("--help") || args.contains("-h") {
    print(usage)
    exit(0)
}

if args.contains("--version") {
    print("nemo-itn \(Nemo.version)")
    exit(0)
}

let sentenceMode = args.contains("-s") || args.contains("--sentence")
args.removeAll { $0 == "-s" || $0 == "--sentence" }

let transform: (String) -> String = sentenceMode ? Nemo.normalizeSentence : Nemo.normalize

if !args.isEmpty {
    // Arguments mode: join all remaining args as the input
    let input = args.joined(separator: " ")
    print(transform(input))
} else {
    // Stdin mode: process each line
    while let line = readLine() {
        let trimmed = line.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { continue }
        print(transform(trimmed))
    }
}
