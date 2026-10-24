# Taxed costs in the AI's checks (branch `taxed-costs`, 2026-10-24)

## The bug

Normal casts pay `cost_of`, which includes taxes (Thorn of Amethyst, Sphere of Resistance, Defense Grid, Damping
Sphere). Several AI paths checked the printed cost instead:

- combo affordability (`combos::affordable` / `mana_need`);
- the combo runner's casts, which also *paid* the printed cost, so they underpaid under a tax;
- removal (`rem_cost`), kicker, held answers, burn to the face, end-of-turn and flash casts;
- the `pool.rs` plan casts and the Tier 6 protection check.

A line such as Thassa's Oracle + Demonic Consultation could start on mana for the printed {U}{U} + {B} and then stop
at the taxed second piece.

## The fix

- `engine/mana.rs` `seq_cost(g, p, cards)` gives the cost of casting cards one after another. It is each card's
  `cost_of` plus {1} per Damping Sphere for each spell of the line cast before it.
- Combo checks (`combos::mana_need`, the Tier 6 line helpers in `t6_combos_t6.rs`) use `seq_cost`.
- The other paths use `cost_of`.
- Tests:
  - `engine_t6_combo3.rs`: under Damping Sphere or Thorn of Amethyst, the Oracle line waits on three lands and wins
    on four. Both tests failed before the fix.
  - `cards_t6_stax.rs`: a unit test of `seq_cost`.
- Full suite: 3006 passed, 0 failed, 3 ignored.

## Before and after (Tier 6, adaptive AI, loose, same seeds)

Scorecard, 8200 games (`base-8200.txt`, `fix-8200.txt`). Every change is inside the 95% CI:

| statistic | before | after |
| --- | --- | --- |
| median win round | 9.0 | 9.0 |
| combo / alt wins | 69.2% | 69.2% |
| damage wins | 29.9% | 30.0% |
| deck spread (pts) | 19.4 | 18.8 |
| Dack / Kinnan / Kraum-Tymna / Rograkh-Silas / Rograkh-Thrasios / Sisay | 35.6 / 22.4 / 29.7 / 22.7 / 16.3 / 23.5 | 35.3 / 22.1 / 29.4 / 23.0 / 16.6 / 23.7 |

The direct counts moved. From the `--calibrate within` report, 4000 games (`base-cal.txt`, `fix-cal.txt`):

| count | before | after |
| --- | --- | --- |
| Oracle line stopped "by an unnamed answer" (a cost, a ward or a lock), Kraum/Tymna | 60 | 33 |
| the same, Rograkh/Silas | 132 | 104 |
| combos stopped, Kraum/Tymna | 364 of 945 (39%) | 327 of 913 (36%) |
| combos stopped, Rograkh/Silas | 513 of 1057 (49%) | 496 of 1055 (47%) |

Over 150 traced games, refused plays ("couldn't play") went from 2057 to 1778. Half-cast stops ("fizzles" or "stops
at step") went from 63 to 55.

## What it means

The fix is correct and removes a real class of wasted turns, but taxes were too rare in these games to explain Tier 6
being slow.

Still open:

- The remaining refused plays are mostly zero-cost or cheap cards: Lotus Petal 119, Expedition Map 109, Culling the
  Weak 103, Mana Vault 79, Mox Diamond 75, Chrome Mox 44. Their cause isn't cost. The AI lists something that
  `act::perform` then rejects, probably a lock or a castability rule its options don't check.
- The biggest "unnamed answer" stops are Ad Nauseam (Kraum/Tymna 137) and Sisay's Derevi + Emiel line (96). They
  belong to steps 3 and 4.
- Godo's `t6_combos_godo::affordable` sums `cost_of` per part without Damping Sphere's growing tax. That's minor:
  Godo is not in the current Tier 6 pool.

## Later the same day: refused plays, Sisay, Rograkh/Thrasios, Kraum/Tymna

Every run below is Tier 6, adaptive AI, loose profile, on the same seeds. "Scorecard" means 8200 games
(`*-8200.txt`); "report" means the 4000-game `--calibrate within` report (`*-cal.txt`).

### 1. Refused plays (commit 06265a65)

The main options listed casts that `do_cast` then refused:
- a lock, or a free spell that Vexing Bauble or Chalice of the Void would counter (Lotus Petal, the Moxen);
- an additional cost that couldn't be paid (Culling the Weak with no creature).

The options now run the same checks. Over 150 traced games, refused plays went from 1778 to 170. The scorecard
didn't change beyond noise (`opt-8200.txt`).

### 2. Sisay (commits 693bd4b0, 61ab71f8)

- **Fetched lands entered as the wrong thing.** A legendary land fetched by Sisay (Mount Doom, Gaea's Cradle)
  entered as a permanent, not a land. The DDR and Flash Unicorn lines then never found their Mount Doom and stopped
  as "no longer ready". Fetched lands now enter as lands (with enters-tapped, land enter effects and landfall) and
  don't use the land drop.
- **Ioreth was spent as mana.** With Relic of Legends or Enduring Vitality, paying for the next fetch could tap the
  Ioreth just fetched. `Game::keep_untapped` now keeps a combo piece out of the mana payment, and the DDR steps keep
  Ioreth.
- **Blind Obedience.** The DDR line waits while a fetched Ioreth would enter tapped.
- **Counters named in the report.** Azorius Guildmage and Tishana's Tidebinder now name themselves in the combo
  report, as Louisoix's Sacrifice does.

Effect:

| | before | after |
| --- | --- | --- |
| Sisay's lines ready | 10% of games | 41% |
| Sisay's win rate | 24% | 42% |
| Sisay's median win round | 13 | 9 |
| Tier 6 median win round | 9 | 8 (now passes the 6-8 band) |
| deck spread (pts) | 18 | 29.8 |
| combo wins | 69% | 82% |

Sisay is now the strongest deck by far. Opponents interact with it as before, but its lines are abilities that
counterspells can't touch.

I checked the Flash Unicorn line against Oracle, since Gaea's Cradle makes only {G}. Derevi's enter trigger untaps
any permanent, so the infinite blinks can untap a land that makes {B} or {R}. The line is real.

### 3. Rograkh/Thrasios (diagnosis only)

A temporary diagnostic checked Plan A on its own turns from round 5 (2000 games). Plan A is Oboro + Cradle +
Talon Gates + 6 creatures + Thrasios and a finisher.

- All five conditions are met on 2% of turns, and four of five on 16%.
- With one missing, it's Talon Gates in the library (472) or Oboro in the library (381).
- At those times the hand rarely holds a tutor that finds the missing piece: Crop Rotation, Expedition Map and
  Mycospawn almost never show up.
- Finale of Devastation (X=2 finds Oboro) is in hand in 112 of 707 Oboro-missing states. It is held as the
  finisher.

So the slowness is the list's tutor density and that finisher choice, not a rules bug. It belongs to the Tier 6
list work.

### 4. Kraum/Tymna (commit d77f42ba) and Dack

- **Kraum/Tymna.** It held the Oracle line whenever Kraum was out and an opponent had one card and one mana up.
  Kraum draws on an opponent's second spell, so it now needs an opponent who can cast a second spell. Result:
  - ready games it never went in: 336 → 304;
  - win rate 24.6% → 25.4%;
  - median win round 10 → 9.

  The rest of its holding is the spec's waiting for protection: a style choice, not a bug.
- **Dack.** It goes the turn it's ready (0 holds), now wins 31%, and is no longer the outlier.

Latest scorecard (`kraum-8200.txt`):
- median win round 8 (PASS);
- seats 26.8 / 25.6 / 23.6 / 24.0 against targets of 32.5 / 26.9 / 22.7 / 17.9;
- combo wins 82.8% (target 47.8%);
- deck spread 29.4 (target 0-8);
- decks: Dack 31.3, Kinnan 19.3, Kraum/Tymna 25.4, Rograkh/Silas 20.5, Rograkh/Thrasios 12.1, Sisay 41.5.

## Rebalance (JD: "Yes rebalance")

The Tier 6 lists already hold the real winning lists' cores (see `../t6-lists-2026-10-24/REPORT.md`). This round:
- swapped the flex slots the 8 recent EDHTop16 winning lists play least for the cards they play most;
- fixed or modelled what those cards need;
- valued Sisay as an engine.

Target (`decklists/pool/TARGETS-t6.md`): every Tier 6 deck wins 18-32% of the decisive games it plays in.

| Step | Dack | Kinnan | Kraum/Tymna | Rog/Silas | Rog/Thrasios | Sisay | Spread | File |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| after the Sisay and Kraum fixes | 31.3 | 19.3 | 25.4 | 20.5 | 12.1 | 41.5 | 29.4 | `kraum-8200.txt` |
| + list swaps (Sisay 4, Rograkh/Thrasios 3) | 31.9 | 17.1 | 25.0 | 20.0 | 15.2 | 40.9 | 25.8 | `lists1-8200.txt` |
| + Sisay's threat value 8 | 33.4 | 18.5 | 28.1 | 20.1 | 16.1 | 34.0 | 18.0 | `sisay8-8200.txt` |
| + Growing Rites of Itlimoc | 32.8 | 18.9 | 27.8 | 20.8 | 16.3 | 33.6 | 17.3 | `lists2-8200.txt` |

### List swaps

| Deck | Out (EDHTop16 play rate) | In (play rate) |
| --- | --- | --- |
| Sisay | The Queen of Dale (0.02), Elves of Deep Shadow (0.03), Cryptolith Rite (0.04), Avacyn's Pilgrim (0.09) | Delighted Halfling (0.79), Orcish Bowmasters (0.71), Swords to Plowshares (0.54), Into the Flood Maw (0.46) |
| Rograkh/Thrasios | Misdirection (0.04), Dramatic Reversal (0.06), Shang-Chi (0.06), Allosaurus Shepherd (0.10) | Phyrexian Metamorph (0.87), Growing Rites of Itlimoc (0.82), Seedborn Muse (0.77), Sylvan Scrying (0.60) |

### Engine changes

- **Land search.** `dsl::search` sent every "search for a land card, put it into your hand" to `land_to_hand`, which
  takes a basic only. Sylvan Scrying (and any such tutor) could never find Gaea's Cradle. Only `basic = true` searches
  take that path now; the rest go through the normal pick, which follows the deck's wish list.
- **Growing Rites of Itlimoc** (new, Approximate, `impls/t6_green2.rs`):
  - when it enters, it takes the best creature card from the top four;
  - at your end step with four or more creatures, it transforms;
  - Itlimoc taps for {G} per creature, and the Rograkh/Thrasios lines take it as their Cradle;
  - in 60 traced games it was cast 17 times and transformed 11 times.
- **Sisay's threat value.** Opponents' removal valued Sisay as a body (about 5), while Kinnan has 8 (`common.rs`
  PVAL). Real tables kill Sisay first: every Sisay line runs through her fetches, which counterspells can't touch. She
  now has 8, as Kinnan does.

### Where it stands

- Kraum/Tymna, Rograkh/Silas and Kinnan are inside 18-32% (Kinnan only just).
- Dack (32.8) and Sisay (33.6) are just over the top of the band.
- Rograkh/Thrasios (16.3) is just under the bottom. Its lines are ready in 14% of games, up from 8%.
- The spread is 17.3 points against 0-8.
- The table's median win round is 8 (passes).
- Combo wins are 82% against a 48% target. This is Playgroup's casual-log target, flagged earlier as one to revisit.
