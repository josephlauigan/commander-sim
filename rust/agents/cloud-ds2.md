# Task: cloud-ds2 — land the double-strike package on the real Cloud list

Read `rust/agents/T0-RULES.md` (Hard rules, Checks before you finish items 1-3). Branch cloud-ds2 = cloud-d2b (Cloud list
with Crawlspace, Archon of Absolution and Package D2) + DS records. `-j 16`. Be quick; JD is waiting.

1. Bring in the engine fix and card models from commit **b1b9fb4** (branch cloud-ds): double strike as two damage steps
   in engine/combat.rs, pro_colors in engine/values.rs, Duelist's Heritage and Akroma's Will models, their overrides and
   tests (tests/cards_cloud_ds.rs). Cherry-pick it; resolve conflicts (card_overrides.toml: keep both sides; regenerate
   data/cards.json and data/decks.json with `cardgen export` rather than merging them). Do NOT take its list or
   decklist changes if any — the list is applied in step 2.
2. Apply the package to Cloud's list with `cardgen deck cloud <new import list> --log "<text>"` (write the log entry so
   it names the three Out and three In cards — check the CHANGELOG line afterwards):
   Secure the Scene -> Fireshrieker ($0.23), Veteran's Powerblade -> Duelist's Heritage ($2.46),
   Nim Deathmantle -> Akroma's Will ($16.67). Log: why (double strike on an equipped Cloud = 4 Sword triggers per attack
   + double commander damage; engine fix made double strike two damage steps), numbers (quick T1 +6.1 ±3.6; 2,000 games
   T1 25.9 -> 30.3, +4.5 ±2.6; T2 20.4 -> 24.8, +4.3 ±2.5; measured on the list before the Crawlspace/Archon substitutes,
   which tested within noise of it), buys $19.36.
3. Fix the deck doc's other mentions of the three cut cards and describe the three new cards from Oracle text. Add the
   three cut cards to bulk-cards.txt.
4. Append a row to audit/cloud/FINDINGS.md's "What was tried" table for double strike (+4.5 / +4.3, confirmed) and a line
   under "Current state" with the new numbers.
5. `cardgen export`, `cardgen validate`, full `cargo test --release -j 16 --no-fail-fast`, `--all-targets` no warnings.
   Commit (no Co-Authored-By), never push. Report ≤10 lines: line 1 DONE/HANDOFF, line 2 model ID, files, test results.
