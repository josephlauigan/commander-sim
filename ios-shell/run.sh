#!/bin/bash
# Build the Rust library, generate the Xcode project, then build and start the app in the iPad simulator:
#
#   ios-shell/run.sh                                   # on the iPad Pro 13-inch (M5)
#   DEVICE="iPad Air 11-inch (M3)" ios-shell/run.sh    # another simulator (xcrun simctl list devices shows their names)
set -e
DEVICE="${DEVICE:-iPad Pro 13-inch (M5)}"
cd "$(dirname "$0")"
./build-rust.sh
xcodegen generate
xcodebuild -project CommanderPractice.xcodeproj -scheme CommanderPractice -configuration Debug \
  -destination "platform=iOS Simulator,name=$DEVICE" -derivedDataPath build/derived build
APP=build/derived/Build/Products/Debug-iphonesimulator/CommanderPractice.app
xcrun simctl boot "$DEVICE" 2>/dev/null || true
open -a Simulator
xcrun simctl install "$DEVICE" "$APP"
xcrun simctl launch "$DEVICE" com.josephlauigan.commander-practice
