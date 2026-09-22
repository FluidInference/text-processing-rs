// swift-tools-version: 5.9

import PackageDescription

#if arch(arm64)
let rustTarget = "aarch64-apple-darwin"
#elseif arch(x86_64)
let rustTarget = "x86_64-apple-darwin"
#else
#error("Unsupported macOS architecture")
#endif

let rustLinkerSettings: [LinkerSetting] = [
    .unsafeFlags([
        "../target/\(rustTarget)/release/libtext_processing_rs.a"
    ])
]

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
            linkerSettings: rustLinkerSettings
        ),
        .executableTarget(
            name: "nemo-itn",
            dependencies: ["CNemoTextProcessing"],
            linkerSettings: rustLinkerSettings
        ),
        .executableTarget(
            name: "nemo-tn",
            dependencies: ["CNemoTextProcessing"],
            linkerSettings: rustLinkerSettings
        ),
        .executableTarget(
            name: "nemo-tn-aligned",
            dependencies: ["CNemoTextProcessing"],
            linkerSettings: rustLinkerSettings
        ),
    ]
)
