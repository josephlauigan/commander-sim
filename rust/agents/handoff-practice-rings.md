# Handoff: practice-rings

Done (see report-practice-rings.md): engine Rings offer (`engine/stack.rs`: `rings_offer`, `ability_window_rc`,
`ability_run`, `with_rings_auto`), hooked in `dsl::activate` / `use_loyalty` (via `ability_window_ex`), `common::walker_go`,
`human/karn.rs` (Grim Monolith, Hellkite pump, Ballista grow/ping, Thrasios), the loop shortcut and Staff "draw N" in
`human/abilities.rs`, `choices::amount_choices`, tests in `crates/sim-core/tests/play_rings.rs`.

Next if anything is left: run the wider checks (cargo build --release --all-targets warnings, the human/practice tests that
touch `human.rs` X menus), hook the remaining karn.rs windows (see the report's "not covered" list).
