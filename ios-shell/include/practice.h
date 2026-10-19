// The Rust library's C interface (rust/crates/practice/src/lib.rs).
#include <stdbool.h>
#include <stdint.h>

// Start practice mode's server inside the app, on 127.0.0.1:port (0: any free port), in background threads.
//   bundle_dir: the app bundle's read-only Resources folder (data/, decklists/, static/).
//   app_dir:    a writable folder (Application Support): the bundled data is copied to app_dir/data when this
//               build's data/bundle.json differs from the last copy; saved games and the autosave go to app_dir/saves.
// Returns the port it listens on, or 0 if it couldn't start (the reason is written to stderr).
uint16_t practice_start(const char *bundle_dir, const char *app_dir, uint16_t port);

// Stop the server (a game in progress ends; its autosave stays, for Continue). Safe to call when none runs.
void practice_stop(void);

// The hidden two-player option: listen on the local network too (true) or on this device only (false).
// Restarts the server on a NEW port and returns it (load the page from there again), or 0 on failure.
uint16_t practice_set_lan(bool lan);
