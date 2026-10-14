# Task: tymna-cards — model every card of the thrasios and kraum decks

Read `rust/agents/TYMNA-RULES.md` first. Base branch: `tymna-thrasios` (the commit that added this brief). Your worktree's branch:
`tymna-cards`. Your module: `rust/crates/sim-core/src/impls/tymna_cards.rs` (new; register it in `impls/mod.rs`),
tests `rust/crates/sim-core/tests/cards_tymna.rs`, card data in `data/carddb.txt` / `data/card_overrides.toml`.

The decks (`decklists/JD/tymna-thrasios-partners.md`, `decklists/JD/tymna-kraum-partners.md`) are registered by the
`partner-engine` agent in parallel, so `cardgen audit --mine` won't show them yet. **Your card list is
`rust/agents/tymna-cards.list`** (121 unique non-basic cards across both decks).

## Step 1: a per-card audit
Add a small option to cardgen: `cardgen audit --cards FILE` prints `status | name | note` for each name in FILE,
using `Audit::card_status(name, true)` (`cardgen/src/audit.rs`). Additive change in `cardgen/src/main.rs` only.
Run it on the list and keep the output in `rust/agents/tymna-cards-audit-before.txt`.

## Step 2: model until every card is Full (or Full-auto, hand-checked)
- **Not in the card database at all** (11): Thrasios, Triton Hero; Kraum, Ludevic's Opus; Abigale, Poet Laureate
  (prepare: look for existing prepared-card machinery first, other bulk cards use it); Cuboid Colony and Elemental
  Mascot (increment / opus: look for existing machinery); Embrace the Paradox; Faeburrow Elder; Nightveil Predator;
  Paradox Surveyor; Silverquill Charm; Sphinx of New Prahv. Run `cardgen export` (online) to fetch them from Scryfall.
- **Commanders first:** Thrasios ({4}: scry 1, reveal the top card; a land goes onto the battlefield tapped, otherwise
  draw it); Kraum (flying, haste; whenever an opponent casts their second spell each turn, draw a card). **Check
  Tymna** (`impls/legends.rs`, modeled 10-09): she must count each opponent dealt combat damage this turn (by any of
  your creatures, Tymna or not) and offer to pay that much life to draw that many at the start of each postcombat
  main phase. Fix and say so if wrong.
- Then every card your audit shows below Full-auto, and spot-check the Full-auto ones that matter to the plan
  (evasion: can't-be-blocked-by clauses, protection, menace, flying granted conditionally; combat-damage-to-a-player
  triggers: Reconnaissance Mission, The Aetherspark, Falcon and Redwing, Lightning, Photon; Terror of the Peaks;
  Bastion Protector).
- Thrasios's activation needs an AI rule: activate at the end of an opponent's turn (before your untap) with any
  mana left, as many times as affordable; on your own turn only after your main-phase plays. Put it in
  `impls/tymna_cards.rs` behind the card, not the deck key.

## Done when
OVERNIGHT-RULES checks 1-3 pass; `cardgen audit --cards rust/agents/tymna-cards.list` shows every card Full or
Full-auto (anything left below: one line each in the report, saying why it can't matter); each new card has a test.
A smoke run isn't possible before the decks are registered; the lead merges both branches and runs it. Save the
after-audit as `rust/agents/tymna-cards-audit-after.txt`. Full report in `rust/agents/report-tymna-cards.md`.
