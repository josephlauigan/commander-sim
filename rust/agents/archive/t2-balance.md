# Task: t2-balance — make Tier 2 (Mid Bracket 3) even within itself

Read `rust/agents/T0-RULES.md` first: its **Hard rules** and **Keeping context small** sections apply to you as
written (memory cap, `-j 3` builds, no wait loops, commit often, handoff file, 15-line report), and so does
**Checks before you finish** items 1-3. Skip its card-modeling sections unless you change card code. Your handoff file
is `rust/agents/handoff-t2-balance.md`, full report `rust/agents/report-t2-balance.md`.

## Why
JD is about to run every JD deck against every tier at 4,000 games per cell and set each deck's bracket from it. Tier 2
(`decklists/pool/t2-mid-b3/`: Brago, Kaalia, Lord Windgrace, Meren, Sythis) is uneven, so a random Tier 2 pod is often
softer or harder than the tier really is. Last measured (September, 240 games per tier, look-ahead AI, after one round
of list tuning; `decklists/pool/pool-results.md` section 0b):

| profile | within Tier 2 |
|---|---|
| loose | Sythis 37, Meren 31, Kaalia 22, Brago 19, Lord Windgrace 17 |
| conservative | Sythis 32, Kaalia 30, Meren 26, Brago 23, Lord Windgrace 16 |

The engine has changed a lot since, so **measure first**.

## Targets (all with the look-ahead AI)
1. **Within Tier 2, both profiles: every deck between 20% and 30%** (25% is an even share; four of the five decks sit
   at each table). Confirm the final state at 2,000 games per profile.
2. **Tier order stays intact:** Tier 2 into Tier 1 (each Tier 2 deck alone against three Tier 1 decks, conservative)
   averages **at least 25%**, and Tier 3 into Tier 2 still averages at least 25%. Prefer raising the cold decks over
   cutting the hot ones, so the tier doesn't get weaker overall.
3. Each deck keeps its axis (blink control, creature cheat, lands, recursion, enchantress) and stays **Mid Bracket 3**:
   at most 3 Game Changers (official list), no two-card infinite combos, no mass land denial, no chained extra turns.

## How
1. **Tooling first:** `commander-sim --calibrate within|ordering` runs every tier. Add an optional tier filter
   (`--calibrate within --pool t2` = only Tier 2's within-tier table; `--calibrate ordering --pool t2` = only "t2 into
   t1"; `--pool t3` gives "t3 into t2"), in `rust/crates/sim-cli/src/{args,main,poolmode}.rs`, with a parse test.
   Keep it small; JD will reuse it for Tier 0.
2. **Baseline:** within Tier 2 at 2,000 games per profile (`--profile loose` and `conservative`), plus both orderings
   (t2 into t1, t3 into t2) at 1,000 games per deck, `--ai lookahead --jobs 7 --seed 1014`. Save outputs under
   `audit/pool/2026-10-14-t2/` (baseline-*.txt) and commit them.
3. **Fidelity before lists:** for each deck outside 20-30%, trace a few of its games
   (`--calibrate` has no trace; use `commander-sim --deck <JD deck> --pool t2 --any-tier --trace N` and follow that
   deck, or whatever trace path exists) and look for AI or rules faults: Brago's blink targets and attack decisions,
   Kaalia getting in and cheating the right creature, Lord Windgrace's land loop and +2/−3 use, Sythis/Meren engines.
   Fix real faults in the deck's plan (`ai/decks.rs`, `ai/pool.rs`) or card code. Re-measure.
4. **Then lists, if still outside the band:** small swaps (about 1-6 cards per deck) are approved by JD (10/14) for
   Tier 2 only. Keep each swap in the deck's theme and the bracket rules above. Record them in the deck file the way
   the September tuning was recorded ("Tuned in October 2026 (with JD's approval) ...: X, Y made way for A, B ...
   Within Tier 2: before -> after"), and update the deck's Game Changers line if it changes. Any new card must be
   fully modeled (`cardgen audit --deck <key>` all Full, with a test); prefer cards that already are.
   After list edits run `cardgen export` and `cardgen validate`.
5. For quick iterations use 600 games per profile with the same seed (paired), then confirm at 2,000.
6. **Write it up:** add the new measurements to `decklists/pool/pool-results.md` section 0a/0b (a dated "October
   2026" row; leave the September rows), with what changed and why.

## Lanes
Five Tier 0 agents are modeling precon cards in parallel and also touch `ai/*` and engine files. Keep your shared-file
changes small and additive; don't touch `impls/t0_*.rs`, Tier 0 decklists or `data/card_overrides.toml` entries
of Tier 0 cards. Never edit JD's decks (`decklists/JD/`) or Avery's, or any pool tier other than Tier 2.

## Machine
You may use up to **7 sim threads** (`--jobs 7`) plus `-j 3` builds. Run sims under
`nice -n 19 systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0 ...`, in the foreground with a long
timeout (a 2,000-game Tier 2 run takes several minutes). No background runs left behind.

## Done when
Targets 1-3 hold at 2,000 games per profile (or you report HANDOFF/DONE with exactly which deck is still outside and
what you tried), the checks pass, results are committed, and the report says per deck: baseline, final, and every
change (AI fix or list swap) with its measured effect.
