# Handoff: ipad-shell
Limits: cargo -j 2, --test-threads=2 (coordinator change), nice 19, MemoryMax=8G.
Done: src/bundle.rs (sync_data), src/lib.rs (practice_start/stop/set_lan + start_app/stop_app/set_lan_app),
tests/app_entry.rs (written, first run pending).
Next: ios-shell (practice.h, App swift, build-rust.sh, project.yml + icons, README), iOS target check, report, final commit.
Facts: Briefcase had NO two-player gesture; page hides players pick when catalog.app. Catalog reads app_dir/decklists
(data/..), so bundle needs Resources/{data,decklists,static}. Briefcase bundle id com.josephlauigan.commander-practice? (check ios/pyproject: bundle com.josephlauigan + app commander_ipad => com.josephlauigan.commander-ipad).
main.rs did not use the old start(); removed.
