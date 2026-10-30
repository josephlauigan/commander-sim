# Report: Phase 2 PR 4, combos from Commander Spellbook

Branch `phase2-spellbook`, commits d0465c75 (data, script, guard) and 1bd0fd36 (audit, handoff), plus this report.
Not pushed. Combo data by Commander Spellbook (https://commanderspellbook.com).

## What was done

- **Fetch**: `scripts/spellbook-fetch.sh` (curl + jq, no Python) POSTs each deck of `data/decks.json` (48: 12 of JD's,
  36 pool) to `find-my-combos` once, 2 s apart, named User-Agent, 429 handled, and skips a deck whose request hash is
  unchanged. Output: `data/spellbook/<deck>.json` (3.3 MB), with source, credit, date, request hash, and per combo:
  id, cards, results, templates, prerequisites, mana, legality; `included` and `almost_included` apart.
  Spellbook's API is MIT code and asks for credit and a link; there is no separate data licence; credited in every
  file and the audit.
- **Compare**: `rust/crates/sim-core/tests/spellbook.rs` matches each combo's cards against `combos::all()` (72 lines,
  quiet lines by their real groups) and Sephiroth's `mine::LOOPS`, one piece per group (Exact / Smaller / Template /
  Bigger / None), plus 8 hand-checked card-code cases. The ignored `spellbook_report` writes the per-deck tables in
  `audit/phase2/spellbook-2026-10-29.md`.
- **Guard**: `included_combos_of_catalogue_pieces_are_modelled` fails on any new uncovered, legal, included combo
  whose cards are all line pieces; the 15 current ones are in `KNOWN_GAPS`. `spellbook_files_match_the_decks` checks
  that Spellbook's names match the lists.
- No combo modelled, no deck or engine change.

## Findings

- 253 legal included combos; the sim covers 120. 32 uncovered 2-3 card lines win; 20 are Yawgmoth's (t5):
  Soultrader + Gravecrawler / Forsaken Miner loops, and drains the sim doesn't count (Vengeful Bloodwitch,
  Sephiroth, Ayara, Nest of Scarabs, Gray Merchant).
- JD's decks: 18 included, 16 covered. Gaps: Kefka and Scion hold Underworld Breach + Lotus Petal + Brain Freeze
  (also kraum-tymna and rograkh-silas at t6). Sheoldred + Peer and the Kefka loops are played by card code.
- Other wins with tutors: Krenko's Kiki-Jiki + Thornbite Staff + Bombardment / Sharpshooter, Skirk + Pashalik Mons +
  Mana Echoes; Aurelia + Helm / Sword of Hearth and Home, Sword of F&F + Aggravated Assault; Meren's Yawgmoth +
  Mikaeus + Gray Merchant; Korvold's Bootleggers' Stash + Revel.
- Sim lines Spellbook doesn't list: Scion-as-Prismari Breach (a commander copy), Dark Depths + Stage (a 20/20, not
  an infinite), Aluren + Chulane + Drake (right by Oracle text), Dack's piles, Ad Nauseam, Badgermole Cub lines.
  Meren's Yawgmoth + Mikaeus + Blood Artist is worth a look: Spellbook's versions need a second non-Human body.
- Spot checks: 5 matches and 5 gaps checked against the lists and Oracle text (in the audit).

## Checks (each rc=0)

- `cargo test --release -j 12` (memory cap): 217 binaries, 3,587 passed, 0 failed.
- `cargo build --release -j 12 --all-targets`: 0 warnings. `cardgen validate`: all decks passed.
- `git diff origin/main -- rust/crates/sim-core/src` is empty, so sim results cannot change; no byte-identity run
  was needed.
