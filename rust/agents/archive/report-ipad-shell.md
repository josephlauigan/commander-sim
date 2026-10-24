# Report: ipad-shell
- lib.rs: C ABI `practice_start(bundle_dir, app_dir, port)`, `practice_stop()`, `practice_set_lan(bool)`; Rust
  `start_app/stop_app/set_lan_app`. Old M6 proof page and `start()` removed (main.rs never used `start`).
- `bundle.rs`: port of `sync_data` (stamp compare; plain files replaced; `images/` missing-only + merged index; saves
  untouched). Deviation: `printings/` and `decklists/` are replaced file by file (Python's missing-only rule would keep
  stale printings after a new build). Also copies `decklists/` to `app_dir/decklists` (the catalog reads `data/../decklists`).
- Bundle layout: Resources/{data,decklists,static}; build-rust.sh fills it (data/images only if present locally: not in git).
- Two-player: Briefcase had no gesture; `catalog.app` hides the players pick in the page. set_lan restarts on a new
  port (old listener may hold the port) and returns it; nothing in the app calls it.
- Bundle id changed to com.josephlauigan.commander-ipad (believed to be Briefcase's: bundle + app_name, underscore to
  hyphen; unverified). Saves from the old app don't carry over (different format). Icon: ios/icons 1024 PNG in Assets.xcassets.
- iOS check: `cargo check -p practice --lib --target aarch64-apple-ios` passes here.
- Tests: tests/app_entry.rs (5). Swift/Xcode part unbuilt.
- Not touched: ios/, tools/app_icon.py. Shared files: none.
