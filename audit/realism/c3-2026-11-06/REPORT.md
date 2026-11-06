# C3: why casual games end two rounds late (2026-11-06)

Branch c3-casual-speed (from origin/main 336b44f5). Brief: `rust/agents/c3-casual-speed.md`. The agent returned this
report as text (its write of this file was refused); the coordinator saved it. No sims by the agent: the evidence is 100
traced games per tier (`--calibrate within --pool t3|t5 --ai adaptive --seed 900000 --trace 1..100`, loose) read with
temporary trace lines kept out of the tree. The traced games reproduce the scorecard shape: median win round 10 in both
tiers; first elimination round 8.3 (Tier 3) and 7.5 (Tier 5), third 10.2 and 9.4. The before-files are in this folder;
the coordinator's scorecards are at the end.

## What the traces show (before the changes)

| round | turns with a creature able to attack (T3 / T5) | power able / sent (T3) | mean life at turn start (T3) |
| --- | --- | --- | --- |
| 4 | 44% / 40% | 2.7 / 2.6 | 38.4 |
| 5 | 57% / 53% | 4.4 / 4.2 | 36.6 |
| 6 | 66% / 59% | 10.0 / 9.8 | 33.6 |
| 8 | 74% / 65% | 10.5 / 9.9 | 28.0 |
| 10 | 86% / 82% | 10.5 / 10.0 | 25.1 |

1. **The attack policy is not what holds Tier 3 back.** Rounds 5-10: 96.6% of the power able to attack is sent; 4% of
   turns with a creature send nothing; a table-wide no-attack round is 2.6% of rounds. The hold-back rule
   (`pool::attack_filter`) fires about 5 times a game but holds little power. The most-hit player takes 50% (T3) and 60%
   (T5) of the damage in rounds 1-7. The slowness is the board: by round 5 a player has cast ~2.7 creatures and 4.9 spells;
   removal kills some (3.8 permanents a game); 43% of round-5 player-turns have no creature able to attack and mean life is
   36.6. Mana is spent in Tier 3 (10% of turns end with 2+ unused and a castable card).
2. **A tax on one player idled a whole army (Tier 5, general rule).** The AI picks one defender; with Ghostly Prison or
   Propaganda (Zur's list) `attack_limits` keeps only the attackers it can pay for, and the rest stayed home. Trace T5 game
   49, rounds 8-9: Miirym had 35 creatures (198 power) against opponents at 33 and 41; it paid 8 to attack Zur with 2 and
   left 33 home; the other three seats, afraid of the 198 power, kept creatures home: a table-wide stall made by one rule.
   22 attacks paid a tax in 100 Tier 5 traces; after the fix, unpaid creatures go elsewhere 26 times.
3. **Opponents did not treat infect as a threat.** Infect and toxic power was read as life damage in `chump_prob` (blocks),
   `attack_response` (instant removal on an attack), `Situation::danger` and `pval`, all against a 40-life total, while ten
   counters kill. Atraxa's board drew 3.9 attacks a game against 5.2 for the others; it won 38 of 100 traced games at 11.0
   rounds on average. Rules check against Oracle (all matching): infect damage to a player is poison
   (`karn_cards::infect_hit`); toxic adds N on top of the damage (`tymna_cards::toxic_hit`); proliferate adds poison to each
   opponent who has one and -1/-1 to opposing creatures with one, never its own player (`common::proliferate`); Vraska's -9;
   Ajani's emblem; Ichor Rats; the ten-counter loss in `life::check_state`. **One rules bug found and fixed:** infect damage
   was not "combat damage dealt to a player" (no lifelink, no "deals combat damage to a player" trigger, no commander damage).
4. **Commanders come down about a turn late.** First cast (own turn count): Atraxa 5.0 (cost 4, never cast 8%), Korvold
   5.8 (5, 16%), Tergrid 6.0 (5, 16%), Aurelia 6.7 (6, 20%), Marwyn 3.3 (3, 0%). When offered, the commander was chosen in
   only 54-61% of decisions (Tier 5: 52-73%): sampled from the utilities, a commander worth about two other spells waits a
   turn two times in five. 42% of player-turns at round 10 find the commander in the command zone; removed about once a game
   (Marwyn most).

## Fixes (each tested in `rust/crates/sim-core/tests/c3_casual_speed.rs`)

| # | Cause | Kind | Fix |
| --- | --- | --- | --- |
| F1 | A tax or cap on the chosen defender leaves the unpaid attackers home | AI, general attack policy (`engine/combat.rs`) | `redirect_dropped`: those attackers attack the opponent with the lowest tax, then lowest life, unless a good blocker there would eat them; taxes paid by `attack_limits`; `choose_defender` counts only the power the tax lets it send (`sendable_power`); a Master of Cruelties attack stays alone |
| F2a | Blocks read infect power as life | AI | `ai_blocks`: infect power leaves life alone; `chump_prob_poison` = max(life share, share of counters left); a 1-power infect/toxic attacker can be chumped |
| F2b | Instant removal on an attack ignored poison | AI | `brain::attack_response`: danger includes poison share >= 0.5 |
| F2c | `Situation::danger` ignored poison | AI | each infect/toxic point counts as the share of life ten counters are worth |
| F2d | An infect/toxic creature valued as a plain creature | AI | `values::pval`: +0.6 per poison point a hit gives (max 6) x (1 + 0.25 x the highest poison among its owner's opponents) |
| F2e | The attacker's kill check ignored poison | AI | `choose_defender`: lethal poison counts as a kill; poison share adds a little |
| F2f | Infect damage was not combat damage dealt | Rules | infect hit moved into the common damage path (lifelink, triggers, commander damage follow); `toxic_of` split out of `toxic_hit` |
| F3 | Commander cast a turn late | AI | `brain::COMMANDER_FIRST` = +1.5 utility on a first cast (no tax) for an outside deck; Tier 6 and JD's decks unaffected. The least certain fix: 0.0 removes it |

Tests: 10 new, all pass; with F1, the poison chump block and F3 switched off, the tax, chump and commander tests fail. 83
named test binaries (1678 tests) pass, including JD's deck card tests.

**JD's decks the general changes can touch:** F1 any of them facing an attack tax (Sauron, Cloud, Sheoldred, Kefka,
Y'shtola, Alela, Galadriel, Thanos), as attacker or defender; F2 any facing infect or toxic (`choose_defender`, `ai_blocks`,
`pval`, `Situation::danger`). F3 skips JD's decks and Tier 6.

**Expected (from the 100 traces, ~+/-5 points):** Tier 5 median 10 -> 9, third elimination 9.4 -> 9.0; Tier 3 median stays
10 (first elimination 8.3 -> 8.2); Atraxa wins 38 -> 30; commander first casts 0.1-0.4 turns earlier in Tier 3. Don't
expect the Tier 3 median to reach 8 from these changes.

## Korvold and Miirym (rules and AI only)
- **Miirym (36.5%): no rules or AI error.** Oracle of Miirym, Roaming Throne, Old Gnawbone, Astral Dragon, Ancient Silver
  Dragon match the engine; the 23-35 creature boards are that legal engine. F1 removes the stall that exaggerated it.
- **Korvold (10.6%): no rules or AI error.** Its sacrifice on enter and attack fires, each adds the counter and the draw;
  sacrifice order tokens, then a land with 5+ lands, then the cheapest card; not over-attacked (4.9 attacks a game against
  4.8 average). Slowed by mana and a late commander (first cast 5.8, 16% never cast, 7-8 cards in hand on two lands in
  several traces); Duress, edicts and Braids from Tergrid take the most from it.

## List-level findings (JD's call, nothing changed)
- **Atraxa (Tier 3)** is a pure infect/toxic list (Blighted Agent, Plague Stinger, Skithiryx, Ichor Rats, Ixhel, Phyresis
  Outbreak, Ajani's emblem, Vraska -9, proliferate): its 23.8% poison share against Playgroup's 9.3% (all builds) is the
  list. Its four-colour mana leaves a full hand and nothing cast in rounds 2-4 in many traces.
- **Tier 3 speed:** few early creatures with power; Aurelia's, Atraxa's and Tergrid's first attacks in rounds 5-6 with 3-4
  power.
- **Tier 5 mana:** 35-47% of Tier 5 turns in rounds 5-10 end with 2+ mana unused and a castable card (Tier 3: 10%), mostly
  Zur and the blue decks keeping counter mana up (`brain::hold_of`). A play-style choice for outside decks, left alone:
  JD to decide whether pool counter-holders should hold less.
- **Zur's tax enchantments** slow the whole Tier 5 table once Zur is out.

## Not done or open
- Commander damage (9.5% against 14.4%): F3 and F2f are the only changes aimed at it.
- Not checked: infect with trample over a blocker.

Files changed: `engine/combat.rs`, `ai/brain.rs`, `ai/mod.rs`, `engine/values.rs`, `impls/tymna_cards.rs`,
`tests/c3_casual_speed.rs`, `rust/agents/handoff-c3-casual-speed.md`.
