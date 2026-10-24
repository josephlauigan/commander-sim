# Task: x-fallback — compiled "X" that silently becomes 3

Read `rust/agents/T0-RULES.md` (Hard rules, How cards are modeled, Checks before you finish items 1-3, Keeping context
small apply; your worktree branch was made from `calib-1014`). Handoff `rust/agents/handoff-x-fallback.md`, full report
`rust/agents/report-x-fallback.md`.

## The bug
`cardgen` compiles "that many", "for each ...", "equal to the number of ..." and similar into the DSL amount `"X"`.
At run time `"X"` reads `ctx.x`, and when no X was chosen it falls back to 3
(`rust/crates/sim-core/src/dsl/mod.rs` lines ~53 and ~1017: "Python's ctx.get('x', 3)"). So a card whose text counts
something (creatures that died, Equipment you control, opponents, damage dealt) instead always uses 3. The t0-captain
agent found it on Ant-Man (always 3 Treasures) and Avenge (always gain 3).

`rust/agents/x-fallback-cards.tsv` lists the 16 cards whose compiled DSL uses `"X"` without an {X} in their cost
(name, source, start of the DSL). Decks: JD's **kefka** (Deadly Embrace, Shimmercreep) and **cloud** (Adelbert Steiner);
pool **light-paws-aura-voltron** and **sythis-selesnya-enchantress** (Kor Spiritdancer); the rest are Tier 0 precon
cards (some were just hand-coded by the Tier 0 agents and may already bypass the DSL; Ant-Man and Avenge did).

## What to do
1. For each of the 16: find out what actually runs (hand-coded hook replacing the DSL? `dsl = []` override? or the
   compiled DSL with X = 3) — a unit test that checks the real number is the proof. Compare with the Oracle text
   (`jq '.["name lowercased"].oracle_text' data/scryfall_cache.json`). Fix each one that is wrong: either give the DSL
   a real amount (if the DSL already has a named amount for it, e.g. `creatures_you_control`, `opponents`, see
   `dsl/mod.rs` around the `"X" =>` match) or hand-code it in the module that already holds that deck's cards.
   Some may be legitimately X (an X spell whose X lives elsewhere, e.g. Finale of Revelation): confirm and say so.
2. **Close the hole:** look for cards beyond these 16 where the same fallback can fire (e.g. a DSL "X" reached from a
   trigger or static rather than a spell with {X}). Best fix: make cardgen stop emitting a bare `"X"` for "that
   many"/counting phrases it can't name (leave the card Partial/Unmodeled in the audit instead, so it shows up), and/or
   make the run-time fallback impossible to hit silently (a debug assertion or audit flag). Choose the smallest
   change that makes it visible; don't change any card's behaviour beyond the fixes in step 1. After cardgen changes run
   `cardgen export` and `cardgen audit --md` and report any card whose verdict changed.
3. Same-seed check that nothing else moved: run `commander-sim --deck sephfs --pool t1 --games 200 --ai adaptive
   --jobs 3 --seed 7 --brief` before and after; a JD deck that holds none of the 16 must give identical numbers.
   For kefka and cloud, report the before/after on their own tiers (`--deck kefka --pool all --games 400 --ai lookahead
   --jobs 3 --seed 7 --brief`, same for cloud): a small change is expected.

## Lanes
Other agents are editing `impls/t0_zurgo.rs` (Zurgo precon: Bone Devourer, Grand Crescendo, Thalisse are in its deck)
and Tier 2 AI files. For those three Zurgo cards, only report what is wrong; don't edit `impls/t0_zurgo.rs`. Keep shared
files to small additive changes.

## Done when
Every one of the 16 is right or reported, the hole is visible from now on, checks pass, report written.
