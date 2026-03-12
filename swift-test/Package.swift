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
                    "-lnemo_text_processing"
                ])
            ]
        ),
        .executableTarget(
            name: "nemo-itn",
            dependencies: ["CNemoTextProcessing"],
            linkerSettings: [
                .unsafeFlags([
                    "-L../target/aarch64-apple-darwin/release",
                    "-lnemo_text_processing"
                ])
            ]
        ),
        .executableTarget(
            name: "nemo-tn",
            dependencies: ["CNemoTextProcessing"],
            linkerSettings: [
                .unsafeFlags([
                    "-L../target/aarch64-apple-darwin/release",
                    "-lnemo_text_processing"
                ])
            ]
        ),
    ]
)
