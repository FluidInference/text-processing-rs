import Foundation
import CNemoTextProcessing

// MARK: - Wrapper

enum Nemo {
    static func tnNormalize(_ input: String) -> String {
        guard let ptr = nemo_tn_normalize(input) else { return input }
        defer { nemo_free_string(ptr) }
        return String(cString: ptr)
    }

    static func tnNormalizeSentence(_ input: String) -> String {
        guard let ptr = nemo_tn_normalize_sentence(input) else { return input }
        defer { nemo_free_string(ptr) }
        return String(cString: ptr)
    }

    static var version: String {
        guard let p = nemo_version() else { return "unknown" }
        return String(cString: p)
    }
}

// MARK: - CLI

let usage = """
    nemo-tn - Text Normalization (written → spoken)

    USAGE:
      nemo-tn <written text>           Normalize a single expression
      nemo-tn -s <sentence>            Normalize spans within a sentence
      echo "text" | nemo-tn            Read from stdin (one line per output)
      echo "text" | nemo-tn -s         Sentence mode from stdin
      nemo-tn --version                Show version
      nemo-tn --help                   Show this help

    EXAMPLES:
      nemo-tn 123
      nemo-tn '$5.50'
      nemo-tn 'January 5, 2025'
      nemo-tn -s 'I paid $5 for 23 items'
      echo '2:30 PM' | nemo-tn
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
    print("nemo-tn \(Nemo.version)")
    exit(0)
}

let sentenceMode = args.contains("-s") || args.contains("--sentence")
args.removeAll { $0 == "-s" || $0 == "--sentence" }

let transform: (String) -> String = sentenceMode ? Nemo.tnNormalizeSentence : Nemo.tnNormalize

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
