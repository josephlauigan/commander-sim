# Task: swap-explain — every swap test also explains why (paired breakdown by default)

Read `rust/agents/T0-RULES.md` (Hard rules, Checks before you finish items 1-3, Keeping context small). Branch
swap-explain (from PR #71's branch). `-j 16` builds, `--jobs 16` sims. Handoff `rust/agents/handoff-swap-explain.md`,
report `rust/agents/report-swap-explain.md`.

## Why
JD (10/15): "each new simulation run should update our understanding of the specific deck's strengths and weaknesses."
Today Cloud's packages S, C, B, R, Sun Titan and Kor Haven were tested with `commander-sim --swap "OUT=>IN" ...`, which
prints only `win X% -> Y% (delta, noise, paired)`. When a package failed we couldn't say why, and Danitha Capashen's
unmodeled cost reduction was found only by accident. `--analyze` (sim-cli `poolmode.rs` `analyze`, `report.rs`
`print_analysis`) already explains a single list; the paired swap run already plays both lists on the same seeds.

## What to build (sim-cli; keep sim-core untouched unless a stat truly needs a hook — list any change)
1. **Pre-flight:** before a `--swap` run, print each swapped-in card's model status (the same verdict `cardgen audit`
   gives: Full / Full-auto / Approximate / Partial / Unmodeled, with the reason). Partial/Unmodeled get a loud
   WARNING line. (The "not modeled: ..." lines the sim already prints for deck cards are a starting point.)
2. **Paired breakdown, on by default with `--swap`** (a `--no-explain` flag turns it off): collect the analysis for
   BOTH versions during the paired run (no extra games) and print, side by side:
   - **Swapped-in cards:** drawn %, cast/played % when drawn, average round cast, ability uses / triggers per game where
     the analyzer tracks them, "stuck in hand" %.
   - **Swapped-out cards** (from the base side): the same numbers, so we see what they were doing.
   - **Game shape:** win rate; how it wins and what eliminates it (the analyze breakdowns), average rounds survived,
     the deck's plan-online timing and commander removals per game where the deck has them (Cloud does; reuse the
     deck-specific sections), mana (lands/rocks by round 4 and 8), hand size at rounds 4 and 8.
   - Keep the existing one-line `delta` summary line exactly as it is (scripts grep it).
3. **No change to results:** the same seed must give the identical `delta` line as before your change (check with
   `--deck cloud --pool t1 --games 200 --ai adaptive --seed 7 --swap "Mind Stone=>Welcoming Vampire"` old vs new, and
   one non-swap run of another deck). Stats collection must not touch the RNG.
4. Tests for the new parsing/flag and for the breakdown containing the swapped cards.
5. Docs: a short section in `commander-sim --help` and in `CLAUDE.md` + `rust/agents/OVERNIGHT-RULES.md`: "swap and
   package tests run with the paired breakdown (default); after every run, update `audit/<deck>/FINDINGS.md`, the
   deck's living list of strengths and weaknesses and what was tested (with why it helped or not)". Create no
   FINDINGS.md content yourself (the lead does Cloud's next).

Report ≤15 lines: line 1 DONE or HANDOFF, line 2 model ID, commits, a sample of the new output (5-10 lines),
same-seed check result, test results.
