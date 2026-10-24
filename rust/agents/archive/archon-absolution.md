# Task: archon-absolution — model Archon of Absolution (one card)

Read `rust/agents/T0-RULES.md` (Hard rules, How cards are modeled, Checks before you finish items 1-3). Branch
cloud-subs (= PR #71's branch). Use `-j 6` builds (sims are running on the rest of the machine).

Archon of Absolution ({3}{W}, Creature — Archon 3/2): Flying; protection from white; creatures can't attack you or
planeswalkers you control unless their controller pays {1} for each of those creatures. It is not in the card database
yet ("not in the card database" in `cardgen audit --cards`): add it the way other cards were added (Scryfall data via
`cardgen`'s fetch path — batch collection POST, never one-by-one; then carddb/overrides), reuse the existing attack-tax
machinery (Ghostly Prison / Windborn Muse / Norn's Annex already work: grep them) and the protection-from-colour
machinery. Mark it Full only when every clause works, with a test (tax charged per attacker; pro-white: can't be
blocked by/targeted by/damaged by white). `cardgen export`, `cardgen validate`, full `cargo test --release -j 6
--no-fail-fast`, `cardgen audit --cards` shows Full. Commit (no Co-Authored-By), never push. Report ≤15 lines,
line 1 DONE or HANDOFF, line 2 model ID.
