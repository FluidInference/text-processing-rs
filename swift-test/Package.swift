// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "NemoTest",
    platforms: [.macOS(.v14)],
    targets: [
        .systemLibrary(
            name: "CNemoTextProcessing",
            path: "Sources/CNemoTextProcessing"
        ),
        .executableTarget(
            name: "NemoTest",
            dependencies: ["CNemoTextProcessing"],
            linkerSettings: [
                .unsafeFlags([
                    "-L/Users/kikow/brandon/voicelink/NeMo-text-processing-rs/target/aarch64-apple-darwin/release",
                    "-lnemo_text_processing"
                ])
            ]
        ),
    ]
)
