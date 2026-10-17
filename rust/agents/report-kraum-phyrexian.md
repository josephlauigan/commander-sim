# Report: kraum-phyrexian (agent model claude-sonnet-5-5)

## Status
DONE. All four cards are Full in `cardgen audit --cards` and each has tests in `tests/cards_tymna.rs` (12 new tests).
`cargo test --release -j 4` passes (all of it), `cargo build --release -j 4 --all-targets` has no warnings. Smoke run
(`--deck kraum --pool t3 --profile loose --games 40 --ai lookahead --jobs 4` with the four swaps) finishes: 20.0% current vs
12.5% swapped over 40 games (noise; a smoke test, not a measurement). Apostle's Blessing was drawn in 10% of games and never
cast (no removal at a worthy target while it was in hand).

## Data
`cardgen audit --cards <file> --fetch` pulled the four cards into `data/scryfall_cache.json`; `cardgen export` then put them in
`data/cards.json` (every cached card is exported). Verdicts in `data/card_overrides.toml` (end of file). The engine already
reads {X/P} (`phyrexian` field; `mana::cost_of` drops a pip the mana can't pay when life > 10, the cast takes 2 life per pip).

## How each card is modeled
- Dismember: tags `rem=shrink5 tgt=c`. Needed one engine fix: the AI's removal path (`brain::removal_options` / `cast_removal`)
  used the printed cost, so Phyrexian removal was never offered without BB. New `brain::rem_cost` uses `cost_of` and
  `phyrexian_life` for cards with Phyrexian symbols (other cards unchanged) and `cast_removal` pays the life.
- Porcelain Legionnaire: `kws = ["first strike"]` in the override; 3/1, artifact creature from Scryfall data. Cast path (creature)
  already paid Phyrexian life.
- Vault Skirge: flying, lifelink from the data (override only adds the note/Full verdict).
- Apostle's Blessing: `tymna_cards::blessing_protect`, called from `tymna_cards::protect` (first thing in
  `ai::decks::protect_response`, so every seat that holds the card uses it). Fires when targeted removal from a spell (not your
  own) hits a commander or a creature/artifact worth >= 4; chooses the spell's first colour, or artifacts for a colourless
  artifact spell; pays {1}{W} or {1} + 2 life (life > 6); goes through `counter_window`; on resolution the permanent gets
  `eot_kw` `pro_<colour>` / `pro_artifacts` and the removal is stopped. `prio` 0 so it is never cast in the main phase.
  Not modeled: protection against abilities (no source colour; the AI doesn't cast it), and against board wipes.

## Shared-file changes (small, additive)
- `ai/brain.rs`: `rem_cost` + use in `removal_options` and `cast_removal` (Phyrexian removal cost and life).
- `ai/decks.rs`: `tymna_cards::protect` now gets `actor` and `spell`.
- `engine/values.rs` `prot_colors`: `eot_kw` `pro_W..pro_G` give protection from that colour.
- `impls/rules2.rs` `prot_vs` and `prot_unblockable`: one-line calls to `tymna_cards::pro_artifacts_vs` (protection from artifacts:
  no damage from, and no blocking by, artifact creatures).
- `data/card_overrides.toml`, `data/cards.json`, `data/scryfall_cache.json` (regenerated/fetched).

## Notes for the lead
- The data export adds every cached card to cards.json, so the diff of `data/cards.json` is the four new cards only (161 lines).
- No deck list changed.
