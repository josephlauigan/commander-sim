# Task: cloud-d2 — put Package D2 into Cloud's list

Read `rust/agents/T0-RULES.md` (Hard rules, Checks before you finish items 1-2). Branch cloud-d2 (origin/main + the 10-15
records). `-j 16` builds. No sims.

JD adopts D2 (audit/cloud/2026-10-15/d2/: T1 +3.6 ±2.5 at 2,000 games, +3.4 in a quick check; T2 +1.5 ±2.4):
- Unbreakable Formation -> **Winds of Abandon** ($4.79)
- Ultimate Magic: Holy -> **Elspeth, Sun's Champion** ($0.38; JD's copy lives in Galadriel, so this is a buy)
- Recommission -> **Wall of Omens** ($0.29)

1. `cardgen deck cloud <new import list> --log "<text>"` (see `cardgen deck --help`): rewrites the Import list, records
   tests/fixtures/my_decks_parsed.json, writes the history to decklists/CHANGELOG.md under Cloud's section. Log text:
   the three swaps, why (defense is Cloud's one working lever; the cut cards were dead: Formation cast 1%, Holy 11%,
   Recommission stuck 33-39%), the numbers above, and the buys (~$5.46).
2. Fix every other place in `decklists/JD/cloud-midgar-equipment.md` that names the three cut cards (Decklist by type,
   Strategy text, counts) and describe the three new cards accurately from their Oracle text.
3. `bulk-cards.txt`: add the three cut cards (follow its format; increase counts if already present).
4. `cardgen export`, `cardgen validate`, full `cargo test --release -j 16 --no-fail-fast`, `--all-targets` build with no
   warnings. Commit (no Co-Authored-By), never push. Report ≤15 lines: line 1 DONE/HANDOFF, line 2 model ID, files changed.
