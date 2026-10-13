# Report: comet-storm

Comet Storm modeled Full: {X}{R}{R} instant, multikicker {1}; X damage to each of 1 + kicks distinct targets.

- Code: `impls/threats.rs` (section "Comet Storm": `comet_resolve`, `comet_plan`, `comet_options`, `comet_go`).
  Card data: `data/card_overrides.toml` entry, `data/cards.json` re-exported (`cardgen export`).
- Shared-file change: `state.rs` `Ctx` gains `pub kicks: u32` (additive).
- AI (any deck holding the card, not gated on Sauron): splits all its mana between X and kicks for the best target
  score (lethal faces 100, killable creatures/planeswalkers worth pval >= 1.5 by value, other faces by chip damage).
  Casts when it kills an opponent (own main phase, or the end-of-turn window of an opponent's turn), or late
  (turn >= 8 or 9+ mana) at X >= 4 when every opponent is hit. Targets are picked at resolution (a simplification:
  no responses change them between cast and resolve in this engine).
- Tests: `tests/cards_comet_storm.rs` (8): one target / kicks add targets / creatures and planeswalkers / survivors not
  targeted / AI split lethal on three opponents / AI pays and resolves / no early cast without a kill / end-of-turn needs a kill.
- Checks: `cargo test --release -j 8` all pass; `--all-targets` build 0 warnings; `all_decks adaptive 3` and
  `lookahead 1` finish; `cardgen audit --mine` clean (Comet Storm is in no list, status Full).
- Smoke: `--deck sauron --pool t3 --games 96 --ai lookahead --jobs 8 --analyze --swap "Brain Freeze=>Comet Storm"`
  finished (20.8% win, 96 games; the card report is empty at that size). Per-card cast counts came from 60 traced
  adaptive games (seeds 0-59): cast in 4 games (about 7% of games; roughly a third of the games where it is drawn by
  turn 12, estimated, the trace does not log draws). Casts: 2 killed an opponent (R9, R10), 2 were late wide hits
  (X=6 with 4 kicks on three opponents; X=5 with 1 kick).
