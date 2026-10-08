#!/bin/bash
# Stage the current code, decks and card data, then build and start the app in the iPad simulator (on the Mac):
#
#   ios/run.sh                                   # on the iPad Pro 13-inch (M5)
#   DEVICE="iPad Air 11-inch (M3)" ios/run.sh    # another simulator (xcrun simctl list devices shows their names)
#
# Extra arguments go to briefcase run, e.g. ios/run.sh -r (rebuild the app's Python packages).
set -e
DEVICE="${DEVICE:-iPad Pro 13-inch (M5)}"
cd "$(dirname "$0")"
python3 stage.py
briefcase run iOS -u -d "$DEVICE" "$@"         # -u puts the newly staged code into the app first
