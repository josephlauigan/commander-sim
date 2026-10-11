#!/bin/bash
# Build the Rust library for iOS and package it for Xcode: build/PracticeCore.xcframework (a device slice, and a
# simulator slice for Apple silicon and Intel Macs). Copies the card data into Resources/data.
set -e
cd "$(dirname "$0")"
ROOT="$(cd .. && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
cd "$ROOT/rust"
for t in aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios; do
  cargo build --release --target "$t" -p practice --lib
done
cd "$ROOT/ios-shell"
mkdir -p build/sim
lipo -create "$ROOT/rust/target/aarch64-apple-ios-sim/release/libpractice.a" \
             "$ROOT/rust/target/x86_64-apple-ios/release/libpractice.a" -output build/sim/libpractice.a
rm -rf build/PracticeCore.xcframework
xcodebuild -create-xcframework \
  -library "$ROOT/rust/target/aarch64-apple-ios/release/libpractice.a" -headers include \
  -library build/sim/libpractice.a -headers include \
  -output build/PracticeCore.xcframework
mkdir -p Resources/data
cp "$ROOT/data/cards.json" "$ROOT/data/decks.json" Resources/data/
echo "built build/PracticeCore.xcframework; card data in Resources/data"
