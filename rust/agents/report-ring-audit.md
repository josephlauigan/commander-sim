# ring-audit report (2026-10-14)

Method: sephfs with `--swap "Caged Zombie=>The One Ring"`, seed 1014, look-ahead, loose. 200 traced games each on t1, t3,
t5 (games 0-199), parsed with grep/perl. Trace lines added (partials.rs): cast step, each tap (burden, life, step, whose turn),
each burden loss. Seph's ring games: t1 53, t3 53, t5 ~24 (t5 traces also contain opponents' own Rings, filtered by owner).

## Casting
- Cast whenever it is in hand and affordable: seen as an option in 54 (t1) / 56 (t3) games, cast in 53 / 53. Median cast
  turn 6, mode turn 4-5. 92-96% in main phase 1 (t1 49 of 53, t3 52 of 58 incl. countered), the rest main 2.
- It competes with Sephiroth, wipes and creatures; the search often picks it over the commander (e.g. Sephiroth 62-92%
  prior, Ring chosen). Priority 55 (+15 if life margin < 6, -20 if life <= 10). Nothing wrong found; casting is "mana is free
  and nothing better", which is right for a card-advantage engine. Protection is not a reason it is cast (it is an artifact:
  protection only covers one opposing round after our own main phase).
- About 5-10% of casts are countered/removed (19 removals by opponents in 133 ring games; Ring was never sacrificed by us).

## Protection
- Prevented damage to Sephiroth FS: t1 73 in 21 events, t3 41 in 16, t5 8 in 7 across 200 games each, in 11/11/4 games.
  Mean 1.4 / 0.8 / 0.3 life per game. It saved a game rarely (t3/121 only lost anyway). Protection is worth little here; the
  value is the draw.

## Activation
- Tap point (t1/t3/t5, Seph's Rings): own main 1 (115/115/50), own main 2 (56/77/21), end of an opponent's turn (41/66/14).
  So it mostly taps in its own turn, main 1, which is at least as good as the opponent's end step (cards usable the
  same turn, burden loss lands at the next upkeep either way). The search sometimes declines the tap in main 1 and taps
  in the opponent's end step instead.
- Roughly 1 tap per turn cycle with the Ring out (avg 3.9 / 4.9 / 3.3 taps per ring game). Skipped cycles: about half are
  explained by the safety rule in `ring_draw` (b >= 2 and life - 2(b+1) < 12), the others are end-step taps, counters,
  removal and game end. No clear mistake.
- Taps usually reach burden 5-7 in long games.

## Burden life loss
- 12.7 (t1) / 17.0 (t3) life per ring game; ~3 per upkeep. Draws are 1+2+3+...: about 10 cards per 6 life.
- Games lost with burden >= 5: t1 1, t3 4, t5 3 of ~130 ring games; burden loss clearly mattered in about 3 (t3/86, t3/120,
  t3/138: 5-6 life per upkeep for 3+ turns at 20-30 life while opponents attacked). The AI stops tapping at that point but
  the drain continues. **Possible improvement (not done): sacrifice the Ring (Carrion Feeder, Altar, Seer ...) when burden >= 5
  and life is low.** It never sacrifices it. Affects ~2% of ring games.

## Rules check
- Protection from everything stops damage from our own sources too. The model only stopped opponents' damage. **Fixed**
  (engine/life.rs): Pestilence on ourselves and Ancient Tomb now prevented; paying life is still not damage. Impact: 6
  Pestilence activations in 54 t1 ring games happened under protection (6 life), 0 in t3/t5: tiny. Opponent pool decks
  that run the Ring (kinnan, yawgmoth, urza, heliod) get the same rule: their games can change only when they damage
  themselves under the Ring.
- Everything else checked matches the Oracle text: protection ends at our next turn start, burden loss at upkeep, draw
  equals burden after the counter.

## Tier 5
- -1.2 +-2.7 (noise); fixed binary -0.7. Eliminated-by: Kinnan 24.8%, Urza 18.7%, Winota 15.8%: fast combos, where a
  4-mana card-advantage engine on turn 5-6 does not help (win rate in games where it was cast 42% of 26, sample too small).
  Nothing indicates a play error; it is a deck/matchup effect.

## Paired test, before/after the rules fix (1,000 games, seed 1014, loose; audit/sephfs/2026-10-14-ring-fix/)
| tier | before | after |
| --- | --- | --- |
| t1 | +1.9 | +2.0 |
| t2 | +3.1 | +3.2 |
| t3 | +2.4 | +2.4 |
| t4 | +0.5 | +0.6 |
| t5 | -1.2 | -0.7 |
Mean +1.3 -> +1.5. Noise per tier +-2.7 to 3.8.

## Verdict
AI plays the Ring reasonably; no big errors. Remaining small loss: no voluntary sacrifice at high burden. Result stays
"small positive, within noise".
