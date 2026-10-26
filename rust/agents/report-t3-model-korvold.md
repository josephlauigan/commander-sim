# Report: Tier 3 cards for korvold-jund-sacrifice and tergrid-mono-black-disruption

(Saved by the main session from the agent's final message on 2026-10-24: the harness refused the agent's own write of
this file. Merged into trace-fixes as 7006eb5c.)

Branch `t3-model-korvold`, commit 27b23f9b. `cargo test --release`: 3044 passed, 0 failed; the new file
`tests/cards_t3_korvold.rs` has 21 tests. `cargo build --release --all-targets` has no warnings.

### Card status
All 21 cards in the brief: 17 Full, 2 Full-auto (Smallpox, Vicious Rumors, now compiled), 2 Approximate.

- Hand-written in `impls/t3_korvold.rs`, all Full with an `audit.toml` note unless marked:
  - Korvold deck: Rhythm of the Wild (riot is haste before combat, else a counter), Impulsive Pilferer encore
    (Approximate: each encore copy doesn't attack its own opponent; the AI picks), Juri, Marionette Apprentice,
    Mazirek, Ruthless Technomancer, Smaug, Wicked Worm.
  - Tergrid deck: Blasphemous Edict (opponents sacrifice first, so Tergrid takes theirs; {B} alternative cost),
    Bottomless Pit, Imp's Mischief (Approximate: only redirects a spell aimed at your key permanent, not counters or
    pumps), Pox, Flare of Malice (with its sacrifice cost), Liliana, Waker of the Dead (all three abilities and the
    emblem), Malakir Rebirth (removal response, and the land face that enters tapped), Mindslicer, Rankle,
    Sangromancer's discard half, The Raven Man's Bird.
- The Reaver Cleaver is compiled; it has a Full note because the audit can't name its X.
- Smallpox and Pox play only when a symmetric-loss estimate favours you; with Tergrid out, opponents' losses count
  1.6x. Smallpox and Bottomless Pit have their own play priorities. Imp's Mischief and Malakir Rebirth are only ever
  cast as responses.
- Aclazotz (Tergrid deck) is still Partial; it wasn't in the brief.

### Shared files changed (small, additive)
- `engine/combat.rs`: passes the combat damage to the ability language, and calls Liliana's emblem at the start of
  combat.
- `ai/decks.rs`: `protect_main` now tries Malakir Rebirth for the pool decks.
- `impls/t6_counters.rs`: Imp's Mischief is a stack response; `pay_response` and `would_redirect` are now
  `pub(crate)`.
- `impls/mod.rs`: registers the new file.
- `dsl/mod.rs`: `Fired.dmg` ("that many" in a combat-damage trigger), and the land branch of "sacrifice".

### Patterns added (under `// --- Tier 3 (korvold)`)
- "..., then discards/mills/sacrifices/loses ..." keeps the sentence's subject. A wider version of the "then" rule
  changed Cut a Deal and Timetwister, so it was narrowed to those four verbs.
- An equipped creature's quoted combat-damage trigger compiles, with "that many" read as the damage dealt.
- "Each player sacrifices a land" now runs.
- Only Pox, Smallpox, Vicious Rumors and The Reaver Cleaver changed in `cards.json`.

### Tier 3 scorecard (t3, loose, adaptive, 4000 games)
The baseline is the parent commit bbfc537b, built from a `git archive` copy.

| | before | after |
|---|---|---|
| Atraxa | 37.3% | 33.2% |
| Aurelia | 31.9% | 28.3% |
| Korvold | 15.5% | 16.7% |
| Marwyn | 9.2% | 8.9% |
| Tergrid | 31.0% | 37.9% |
| median win round | 11 | 11 |
| combo/alt wins | 2.6% | 3.0% |
| targets passing | 1 of 9 | 1 of 9 |
