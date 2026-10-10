#!/bin/sh
# Build the Python module rust_sim (crates/sim-py) and put it where Python finds it: commander_sim/rust_sim.so.
#   rust/build-py.sh            release build (for simulations)
#   rust/build-py.sh --debug    debug build (faster to compile)
set -e
cd "$(dirname "$0")"
if [ "$1" = "--debug" ]; then cargo build -q -p sim-py; dir=debug; else cargo build -q --release -p sim-py; dir=release; fi
cp "target/$dir/librust_sim.so" ../commander_sim/rust_sim.so
echo "built commander_sim/rust_sim.so ($dir)"
