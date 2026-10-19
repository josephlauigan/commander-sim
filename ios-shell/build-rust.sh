#!/bin/bash
# Build the Rust library for iOS and package it for Xcode: build/PracticeCore.xcframework (a device slice, and a
# simulator slice for Apple silicon and Intel Macs). Then fill Resources/ with what the app carries:
#   Resources/data/       cards.json decks.json (the engine), my_decks.toml scryfall_cache.json printings/ (the setup
#                         screen and the card images' printings), bundle.json (this build's stamp: commit + time), and
#                         images/ if data/images exists here (it is not in git; without it the app downloads the
#                         images when it is online and draws cards from their names when it is not)
#   Resources/decklists/  the deck lists and tier folders (the setup screen's strengths, summaries and win rates)
#   Resources/static/     the practice page (rust/crates/practice/static)
set -e
cd "$(dirname "$0")"
ROOT="$(cd .. && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
cd "$ROOT/rust"
# from rust/, so the targets go to the toolchain rust/rust-toolchain.toml pins (not the default one)
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
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

rm -rf Resources
mkdir -p Resources/data Resources/decklists Resources/static
D="$ROOT/data"
cp "$D/cards.json" "$D/decks.json" "$D/my_decks.toml" "$D/scryfall_cache.json" Resources/data/
cp -R "$D/printings" Resources/data/printings
if [ -d "$D/images" ]; then cp -R "$D/images" Resources/data/images; fi
# decklists without the retired decks (the Briefcase app left them out too)
rsync -a --exclude retired --exclude '*.tmp' "$ROOT/decklists/" Resources/decklists/
cp -R "$ROOT/rust/crates/practice/static/." Resources/static/
COMMIT="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"
NOW="$(date +%Y-%m-%dT%H:%M:%S)"
NIMG="$(ls Resources/data/images 2>/dev/null | wc -l | tr -d ' ')"
printf '{"commit":"%s","built":"%s","images":%s}\n' "$COMMIT" "$NOW" "$NIMG" > Resources/data/bundle.json
echo "built build/PracticeCore.xcframework; Resources/ filled (commit $COMMIT, $NIMG image files)"
