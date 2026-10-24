# kefka-breach report
Commit: see git log (branch kefka-breach). DONE.
- mine.rs `breach_line`: the `key == "sauron"` gate is gone; non-Sauron decks need Breach out + a storm card in hand/gy (cheap pre-check `decks::breach_pieces`). Sauron path unchanged: `--deck sauron --pool t3 --games 96 --seed 500000 --profile loose --ai adaptive --brief` output identical before/after.
- ai/decks.rs (additive): helpers `breach_out`, `storm_held`, `breach_pieces`, `deck_has_storm`, `breach_hold_prio`, `breach_keep_bonus`; kefka branch in `tutor_pick_named` (Diabolic Tutor: Breach, then Brain Freeze/Grapeshot, then ritual; only when the deck carries a storm card).
- impls/kefka.rs: `kefka_options` adds the line option; `kefka_prio` holds Breach (storm card held), storm cards (Breach out), rituals/Petal (both) at 0; `discard_choice` +80 keep bonus for pieces the other half waits for. Without a storm card Breach prio is the old rule.
- Shared files touched: none beyond mine.rs (smallest change) and ai/decks.rs (appended).
- Tests: tests/cards_kefka_breach.rs (6): line mills table out, not offered w/o mana, holds, old Breach use, discard keep, tutor.
- cargo test --release --no-fail-fast all pass (practice http_lan failed once under load, passed on rerun); build --all-targets 0 warnings; all_decks adaptive 3 and lookahead 1 no panic; audit --mine kefka 0 not Full.
- Smoke (96 games lookahead, swaps Duress->Brain Freeze, Dark Inquiry->Dark Ritual, Feed the Swarm->Lotus Petal; cuts arbitrary, lead picks real): Kefka 24.0% win rate. The analysis output has no per-game line attempt counter (stat `breach_line` exists on players but is not printed), so attempts/wins of the line were not measured.
- Caveat: Breach is held whenever a storm card is in hand/gy, even if the line never becomes live (same as Sauron).
