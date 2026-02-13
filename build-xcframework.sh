#!/bin/bash
set -e

# Build universal static library for Apple platforms
# Outputs: NemoTextProcessing.xcframework

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/build"
OUTPUT_DIR="$SCRIPT_DIR/output"

rm -rf "$BUILD_DIR" "$OUTPUT_DIR"
mkdir -p "$BUILD_DIR" "$OUTPUT_DIR"

echo "Building for macOS (arm64)..."
cargo build --release --features ffi --target aarch64-apple-darwin

echo "Building for macOS (x86_64)..."
cargo build --release --features ffi --target x86_64-apple-darwin

echo "Building for iOS (arm64)..."
cargo build --release --features ffi --target aarch64-apple-ios

echo "Building for iOS Simulator (arm64)..."
cargo build --release --features ffi --target aarch64-apple-ios-sim

echo "Creating universal macOS library..."
mkdir -p "$BUILD_DIR/macos"
lipo -create \
    target/aarch64-apple-darwin/release/libnemo_text_processing.a \
    target/x86_64-apple-darwin/release/libnemo_text_processing.a \
    -output "$BUILD_DIR/macos/libnemo_text_processing.a"

echo "Creating XCFramework..."
xcodebuild -create-xcframework \
    -library "$BUILD_DIR/macos/libnemo_text_processing.a" \
    -headers swift/include \
    -library target/aarch64-apple-ios/release/libnemo_text_processing.a \
    -headers swift/include \
    -library target/aarch64-apple-ios-sim/release/libnemo_text_processing.a \
    -headers swift/include \
    -output "$OUTPUT_DIR/NemoTextProcessing.xcframework"

echo "Copying Swift wrapper..."
cp swift/NemoTextProcessing.swift "$OUTPUT_DIR/"

echo ""
echo "Done! Output:"
echo "  $OUTPUT_DIR/NemoTextProcessing.xcframework"
echo "  $OUTPUT_DIR/NemoTextProcessing.swift"
echo ""
echo "Add both to your Xcode project."
