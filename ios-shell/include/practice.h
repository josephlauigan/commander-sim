// The Rust library's C interface (rust/crates/practice/src/lib.rs).
#include <stdint.h>

// Start practice mode's server on 127.0.0.1:port (0: any free port) in a background thread, with the card data
// (cards.json, decks.json) in data_dir. Returns the port it listens on, or 0 if it couldn't start.
uint16_t practice_start(const char *data_dir, uint16_t port);
