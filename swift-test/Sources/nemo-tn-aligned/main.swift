import Foundation
import CNemoTextProcessing

// MARK: - Wrapper

struct AlignedSpan: Encodable {
    let inputStart: Int
    let inputEnd: Int
    let original: String
    let normalized: String
    let kind: String

    enum CodingKeys: String, CodingKey {
        case inputStart = "input_start"
        case inputEnd = "input_end"
        case original
        case normalized
        case kind
    }
}

struct Alignment: Encodable {
    let input: String
    let language: String
    let normalized: String
    let spans: [AlignedSpan]
}

enum Nemo {
    static func tnNormalizeAligned(_ input: String, language: String) -> Alignment? {
        guard let resultPtr = nemo_tn_fst_aligned(input, language) else {
            return nil
        }
        defer { nemo_tn_alignment_free(resultPtr) }

        let result = resultPtr.pointee
        guard let normalized = result.normalized else {
            return nil
        }

        let count = Int(result.span_count)
        if count > 0 && result.spans == nil {
            return nil
        }

        let nativeSpans = UnsafeBufferPointer(start: result.spans, count: count)
        var spans: [AlignedSpan] = []
        spans.reserveCapacity(count)
        for span in nativeSpans {
            guard let original = span.original,
                  let normalized = span.normalized,
                  let kind = span.kind else {
                return nil
            }
            spans.append(AlignedSpan(
                inputStart: Int(span.input_start),
                inputEnd: Int(span.input_end),
                original: String(cString: original),
                normalized: String(cString: normalized),
                kind: String(cString: kind)
            ))
        }

        return Alignment(
            input: input,
            language: language,
            normalized: String(cString: normalized),
            spans: spans
        )
    }

    static var version: String {
        guard let pointer = nemo_version() else { return "unknown" }
        return String(cString: pointer)
    }
}

// MARK: - CLI

let usage = """
    nemo-tn-aligned - FST Text Normalization with source alignment

    USAGE:
      nemo-tn-aligned [-l <lang>] <written text>
      echo "text" | nemo-tn-aligned [-l <lang>]
      nemo-tn-aligned --version
      nemo-tn-aligned --help

    OPTIONS:
      -l, --lang <lang>  en, fr, es, de, zh, hi, or ja (default: en)

    OUTPUT:
      One compact JSON object per input. Stdin mode emits JSON Lines.
      Source offsets are half-open UTF-8 byte offsets.

    EXAMPLE:
      nemo-tn-aligned --lang en 'The price is $1,234.56.'
    """

let args = Array(CommandLine.arguments.dropFirst())

if args.contains("--help") || args.contains("-h") {
    print(usage)
    exit(0)
}

if args.contains("--version") || args.contains("-V") {
    print("nemo-tn-aligned \(Nemo.version)")
    exit(0)
}

var language = "en"
var inputArgs: [String] = []
var index = 0
var optionsEnded = false
while index < args.count {
    let argument = args[index]
    if !optionsEnded && argument == "--" {
        optionsEnded = true
    } else if !optionsEnded && (argument == "-l" || argument == "--lang") {
        index += 1
        guard index < args.count else {
            fputs("nemo-tn-aligned: --lang requires a language code\n", stderr)
            exit(1)
        }
        language = args[index]
    } else if !optionsEnded && argument.hasPrefix("--lang=") {
        language = String(argument.dropFirst("--lang=".count))
    } else {
        inputArgs.append(argument)
    }
    index += 1
}

let supportedLanguages = ["en", "fr", "es", "de", "zh", "hi", "ja"]
guard supportedLanguages.contains(language) else {
    fputs(
        "nemo-tn-aligned: unsupported language '\(language)'; expected one of \(supportedLanguages.joined(separator: ", "))\n",
        stderr
    )
    exit(1)
}

if inputArgs.isEmpty && isatty(fileno(stdin)) != 0 {
    fputs(usage, stderr)
    exit(1)
}

let encoder = JSONEncoder()
encoder.outputFormatting = [.sortedKeys, .withoutEscapingSlashes]

func emit(_ input: String) -> Bool {
    guard let alignment = Nemo.tnNormalizeAligned(input, language: language) else {
        fputs("nemo-tn-aligned: could not classify input as \(language) text\n", stderr)
        return false
    }

    do {
        let data = try encoder.encode(alignment)
        FileHandle.standardOutput.write(data)
        FileHandle.standardOutput.write(Data([0x0A]))
        return true
    } catch {
        fputs("nemo-tn-aligned: failed to encode JSON: \(error)\n", stderr)
        return false
    }
}

if !inputArgs.isEmpty {
    exit(emit(inputArgs.joined(separator: " ")) ? 0 : 1)
}

var failed = false
while let line = readLine() {
    guard !line.trimmingCharacters(in: .whitespaces).isEmpty else { continue }
    if !emit(line) {
        failed = true
    }
}
exit(failed ? 1 : 0)
