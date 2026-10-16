# Task: cloud-subs — put Crawlspace and Archon of Absolution into Cloud's list

Read `rust/agents/T0-RULES.md` (Hard rules, Checks before you finish items 1-2). Branch cloud-subs (PR #71's branch +
Archon of Absolution's card model). `-j 16` builds. No sims.

JD couldn't find Ghostly Prison or Archangel of Tithes. Tested substitutes (audit/cloud/2026-10-15/subs/log.txt): vs the
Package D list, T1 -1.5 (noise ±1.9), T2 -0.9 (±1.8) — within noise, so JD adopts them.

1. In `decklists/JD/cloud-midgar-equipment.md`: Ghostly Prison -> **Crawlspace**, Archangel of Tithes -> **Archon of
   Absolution**. Use `cardgen deck cloud <new import list> --log "<text>"` (see `cardgen deck --help`; it rewrites
   the Import list, records tests/fixtures/my_decks_parsed.json and writes the history to decklists/CHANGELOG.md under
   Cloud's section). Log text: the two swaps, why (not findable), the test numbers above, and buys: the six ordered
   Package D cards plus Archon of Absolution ($0.20); Crawlspace comes from the binder.
2. Fix every other place in the doc that names Ghostly Prison or Archangel of Tithes (Decklist by type, Strategy text,
   counts): describe Crawlspace (no more than two creatures can attack you each combat) and Archon of Absolution
   (3/2 flier, protection from white, {1} attack tax per creature) accurately. Also correct the Package D buy list in
   decklists/CHANGELOG.md's 10/15 entry if it lists the two cards as buys (note they were replaced).
3. `binder-cards.txt`: remove the `1x Crawlspace` line (it now lives in Cloud). Keep the file's format/sorting.
4. `cardgen export`, `cardgen validate`, full `cargo test --release -j 16 --no-fail-fast`; build with no warnings.
   Commit (no Co-Authored-By), never push. Report ≤15 lines, line 1 DONE or HANDOFF, line 2 model ID, files changed.
