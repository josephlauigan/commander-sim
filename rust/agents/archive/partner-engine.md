# Task: partner-engine — two commanders per seat (Partner), and register the thrasios and kraum decks

Read `rust/agents/TYMNA-RULES.md` first. Base branch: `tymna-thrasios` (the commit that added this brief). Your worktree's branch:
`partner-engine`.

Today every seat has exactly one commander (`engine/turn.rs` `commander: CardId`, `run.rs` `seat(.., commander)`,
`export.rs`, `testkit.rs`, deck loading in `sim-cli/src/decks.rs` and `cardgen/src/decks.rs` / `export.rs` /
`update_deck.rs`; about 120 references in 40 files: `grep -rnw commander --include='*.rs' rust/crates`). Two new decks
have two commanders with plain **Partner**.

## What to build
1. **Deck data.** The two deck documents list both commanders on the line `**Commanders (2).** A, B`. Make the deck
   parsers (`cardgen` export/validate/update_deck and `sim-cli/src/decks.rs`) accept one or two commanders. Validation
   for two: both have plain Partner (an Oracle line `Partner` or starting `Partner (You can have two commanders`),
   colour identity = union of both, 100 cards including both. Keep single-commander decks' data byte-identical
   (`data/decks.json` diff for existing decks must be empty after `cardgen export`).
2. **Engine.** Both commanders start in the command zone. Each has its **own commander tax** (+2 per earlier cast
   from the command zone). Commander damage is tracked **per commander** (21 from one of them eliminates). Every
   "commander" check counts both: the command-zone replacement when either would die or be exiled, "a commander you
   control", "commander creatures you control" (Bastion Protector is in both decks), `impls/common.rs` helpers,
   anything in `impls/` that looks at "the" commander (grep). Prefer a small API (`commanders(p)`, `is_commander(p,
   card)`) and route the old single-commander uses through it, so single decks behave exactly as before.
3. **AI.** Cast both commanders (generic rule: as soon as each is affordable, cheaper first; recast after removal when
   the tax is affordable, as single commanders do now). Opponents' threat and removal logic that values "the
   commander" must see both. Look-ahead (`ai/`) must handle a second commander in the command zone.
4. **Register the decks** `thrasios` and `kraum`: `sim-cli/src/decks.rs` (table + `MINE`), `data/my_decks.toml`,
   deck-prio dispatch in `ai/decks.rs` pointing to a stub module `impls/tymna_partners.rs` (functions
   `thrasios_prio` / `kraum_prio` returning the default; the deck AI plan is written later). Run `cardgen export`
   (online: it fetches the cards missing from `data/scryfall_cache.json`). Do **not** write card code: the
   `tymna-cards` agent models Thrasios, Kraum and the other new cards in parallel (its module `impls/tymna_cards.rs`).
   If a card is missing or unmodeled, the decks must still load and play.

## Tests (`rust/crates/sim-core/tests/partners.rs`)
Both cast from the command zone; separate tax (cast A twice, B once: costs +2/+4 and +0/+2); 21 commander damage from
one partner eliminates, 20 + 20 split across both doesn't; each goes back to the command zone; Bastion Protector
buffs both; a single-commander seat unchanged.

## Same-seed check (behaviour of existing decks unchanged)
Before your first engine change, build and save the binary (`cp target/release/commander-sim /tmp/.../csim-before`
in your scratch). After, run both binaries with the same seed and compare the result lines:
`--deck sauron --pool t3`, `--deck titania --pool t1`, `--deck cloud --pool t5`, each
`--profile loose --games 48 --ai lookahead --jobs 10 --seed 7 --brief`. They must be identical. Say so in the report.

## Done when
OVERNIGHT-RULES checks 1-3 pass; the same-seed check is identical; `cardgen validate` passes for all decks (both new
ones OK); 48-game smoke runs finish for `--deck thrasios` and `--deck kraum` against `--pool t3` (lookahead).
Full report in `rust/agents/report-partner-engine.md`.
