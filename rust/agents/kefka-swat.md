# Task: kefka-swat — find why Kefka almost never casts Deflecting Swat, and fix it

Read `rust/agents/LASTDECK-RULES.md` for the hard rules and checks, with these changes: base branch `leftovers-apply`
(f90767c); your worktree's branch `kefka-swat`; at most 8 threads (`cargo -j 8`, `--jobs 8`), everything under
`nice -n 19 systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0`; commit without any Co-Authored-By
line; never push; no batch sims over 48 games except the measurement in step 4.

## The symptom
`audit/leftovers/2026-10-14/analyze-kefka-t1.txt` and `-t3.txt` (look-ahead, loose, 320 games) list Deflecting Swat as
"cast 5.0% / 0.0% of the games it was drawn [never cast by the sim: untagged]", while Kefka is the most-removed card
(about 0.33 removals per game). Deflecting Swat ({2}{R} instant, free if you control your commander: "You may choose
new targets for target spell or ability.") is modeled in `impls/mine.rs` (`veyran_protect`, around line 3270), listed in
`ai/pool.rs` (line ~876), and wired for Kefka in `ai/decks.rs` (~line 1316, Kefka uses Veyran's responses).

## What to find out (traces, unit tests)
1. Why the analysis calls it "untagged" (grep the analysis code for that label): is a tag missing in
   `data/carddb.txt` / `data/card_overrides.toml` so the card report and any tag-gated AI path skip it? Slip Out the
   Back shows "[reactive: only cast in response]" — compare.
2. When removal targets Kefka (single-target destroy/exile/bounce, and -X/-X), does the protect path reach the Swat
   code for the kefka seat with Swat in hand? Check: the free-cost condition (commander on the battlefield), the
   removal kinds it accepts, whether a better response (Slip Out the Back, Lightning Greaves) pre-empts it, and
   whether the new target is a legal permanent of the caster's. Write a testkit test: Kefka on the battlefield, Swat in
   hand, an opponent's Swords to Plowshares / Murder / Chaos Warp at Kefka: Swat is cast and the spell hits the
   caster's creature (or fizzles when it has none). Same for removal at another key Kefka permanent if the code
   protects those.
3. Fix what is wrong (one commit per fix, with a test). Keep Veyran's behaviour unless it has the same bug (say so).
4. Measure: a same-seed before/after of `--deck kefka --pool t3 --profile loose --games 320 --ai lookahead --jobs 8
   --analyze` (Swat cast rate; Kefka removals answered), and report the numbers.

## Done when
LASTDECK-RULES checks 1, 2 (full `cargo test --release -j 8`, no warnings) and the step-4 numbers. Report at most 15
lines (line 1 DONE or HANDOFF, line 2 your model ID); full report `rust/agents/report-kefka-swat.md`, committed;
handoff `rust/agents/handoff-kefka-swat.md`.
