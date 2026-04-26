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
                    "-L../target/aarch64-apple-darwin/release",
                    "-ltext_processing_rs"
                ])
            ]
        ),
        .executableTarget(
            name: "nemo-itn",
            dependencies: ["CNemoTextProcessing"],
            linkerSettings: [
                .unsafeFlags([
                    "-L../target/aarch64-apple-darwin/release",
                    "-ltext_processing_rs"
                ])
            ]
        ),
        .executableTarget(
            name: "nemo-tn",
            dependencies: ["CNemoTextProcessing"],
            linkerSettings: [
                .unsafeFlags([
                    "-L../target/aarch64-apple-darwin/release",
                    "-ltext_processing_rs"
                ])
            ]
        ),
    ]
)
