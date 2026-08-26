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
cargo build --release --features "ffi,fst-engine" --target aarch64-apple-darwin

echo "Building for macOS (x86_64)..."
cargo build --release --features "ffi,fst-engine" --target x86_64-apple-darwin

echo "Building for iOS (arm64)..."
cargo build --release --features "ffi,fst-engine" --target aarch64-apple-ios

echo "Building for iOS Simulator (arm64)..."
cargo build --release --features "ffi,fst-engine" --target aarch64-apple-ios-sim

echo "Creating universal macOS library..."
mkdir -p "$BUILD_DIR/macos"
lipo -create \
    target/aarch64-apple-darwin/release/libtext_processing_rs.a \
    target/x86_64-apple-darwin/release/libtext_processing_rs.a \
    -output "$BUILD_DIR/macos/libtext_processing_rs.a"

echo "Creating XCFramework..."
# NOTE: headers live in swift/include/CNemoTextProcessing/ so each slice gets
# Headers/CNemoTextProcessing/module.modulemap. Xcode's ProcessXCFramework copies
# each xcframework's Headers/ into the SHARED $BUILT_PRODUCTS_DIR/include, so a
# top-level include/module.modulemap collides with any other xcframework that
# ships one ("Multiple commands produce .../include/module.modulemap"). Clang
# still resolves `import CNemoTextProcessing` because it looks for a module map
# in a subdirectory named after the module. Keep the directory name == module name.
xcodebuild -create-xcframework \
    -library "$BUILD_DIR/macos/libtext_processing_rs.a" \
    -headers swift/include \
    -library target/aarch64-apple-ios/release/libtext_processing_rs.a \
    -headers swift/include \
    -library target/aarch64-apple-ios-sim/release/libtext_processing_rs.a \
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
