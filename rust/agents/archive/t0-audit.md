# Task: t0-audit — are the Tier 0 precons too strong because of generous card code?

Read `rust/agents/T0-RULES.md` (Hard rules, How cards are modeled, Checks before you finish items 1-3, Keeping context
small apply; your branch was made from `calib-1014`). Handoff `rust/agents/handoff-t0-audit.md`, full report
`rust/agents/report-t0-audit.md`. Use at most `-j 3` for builds and `--jobs 4` for sims.

## Why
The five Tier 0 precons (`decklists/pool/t0-b2-precons/`, unmodified WotC lists) were just fully modeled by five agents
(code in `impls/t0_*.rs`). Tier 0 is meant to be the weakest tier (Bracket 2), below Tier 1 (High B2 / Low B3). But with
the look-ahead AI, each Tier 1 deck alone against three Tier 0 decks wins only **19.7%** (25% = even; Light-Paws 33.6,
Teysa 20.8, Isshin 17.9, Lathril 15.2, Tatyova 10.8), and within Tier 0 Anikthea (33%) and Giada (32%) run hot
(Zurgo 25, Captain America 19, Stella 15). Results in `audit/pool/2026-10-14-t0/`.

Either the precons really are this strong, or some card code is more generous than the Oracle text. **Find out.**

## How
1. **Hunt for over-generous code**, hottest decks first (Anikthea, Giada, then Zurgo): for each card in `impls/t0_*.rs`
   compare the code with its Oracle text (`jq '.["name lowercased"].oracle_text' data/scryfall_cache.json`). Typical
   faults: triggers that fire too often (once per object instead of once per event, "whenever" vs "the first time each
   turn"), missing costs or conditions ("if you control", "only as a sorcery", mana value limits), effects applied to
   all players' things instead of yours (or vice versa), counters/tokens doubled, stat bonuses that stack wrongly,
   "until end of turn" effects that last, a static the AI gets for free, copies that keep copying. The approximations
   the agents listed in `rust/agents/report-t0-*.md` are a good starting list.
2. **Look at how the precons win**: trace some Tier 1 vs Tier 0 games (`commander-sim --deck <any JD deck> --pool t0
   --any-tier --trace N`, or whatever trace path suits) and check the winning lines are legal and plausible.
3. **Fix real faults** in the card code (never the decklists), each with a test. Don't nerf anything the Oracle text
   allows; if a card is simply strong, leave it.
4. **Measure** after fixes, with the same seeds as the baseline:
   `commander-sim --calibrate ordering --pool t1 --profile conservative --games 1000 --ai lookahead --jobs 4 --seed 1014`
   and `--calibrate within --pool t0 --profile loose --games 2000 ...`. Save outputs to `audit/pool/2026-10-14-t0/`
   (after-*.txt) and commit.

## Deadline
The lead needs your result by **01:30**: commit what you have by then (stop at a clean point), with the report saying
which cards were wrong, what was fixed, and before/after numbers. If you found nothing wrong, say so plainly: that is a
valid result.

## Lanes
Other agents are working on Tier 2 lists/AI and on DSL "X" amounts. Touch only `impls/t0_*.rs`, their tests, Tier 0
entries in `data/card_overrides.toml`, and the smallest shared-file changes (list them).
