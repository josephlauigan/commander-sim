# Isshin T1 regression: INCOMPLETE (stopped by lead, CPU needed overnight)

## Status
No cause found. No game was traced. Both binaries are ready (OLD t6-baseline, NEW t6-cedh).

## What was run
- OLD: `--calibrate within --pool t1 --games 200 --seed 500000 --jobs 4` finished the conservative block:
  Isshin 26.4% (20.2-33.8). Its loose block was cut off.
- NEW: the same command was started, then killed when the lead stopped the work. No result.
- The 200-game sample is too small to find the shift by itself (CI about +-7 pts). The full-size gap was 24.5 -> 30.8.
- No per-game seed list was produced: the calibrate output only gives per-deck rates.

## Hypotheses (from rust/agents/report-t6-e-combat.md, not verified)
1. The `ncomb < 4` cap became MAX_COMBATS=200. Karlach (extra combat) with Isshin's attack-trigger doubling
   (two extra combats) and Aurelia / Scourge of the Throne could now give more than 4 combats a turn.
   This is probably correct rules: the old silent truncation is gone, and the report says so.
   Expected sign: Isshin wins more.
2. Legion Warboss makes a token at the beginning of combat, a different trigger from attack triggers.
   Check that it does not fire once per combat in a way that Isshin doubles. Isshin doubles only attack triggers.
   A bug would show up as 2 Goblin tokens per Warboss combat, or tokens created by the new `each_combat` event.
   Only Sting is supposed to be on `each_combat`.
3. Other changes (legend rule, toughness-0 deaths, enters-tapped) are less likely to favor Isshin specifically.

## Next steps
- Run Isshin only (`--deck isshin --pool t1 --games 1500 --seed 500000`) on both binaries, or a calibrate with more games,
  to confirm the gap is stable.
- Use `--trace N` (the seed offset) with `--deck isshin --pool t1 --seed 500000` on both binaries.
  Try N = 0..40 and pick games whose winner differs.
  Compare "combat" lines per turn, Karlach/Isshin trigger lines, and Warboss token lines.
- Count the combats in one turn: more than 4 with Karlach + Isshin supports hypothesis 1.
- Check Oracle text with a single Scryfall `POST /cards/collection` (curl -m 30) for Karlach, Legion Warboss, Isshin.
