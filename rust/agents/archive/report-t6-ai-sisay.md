# Report: t6-ai-sisay (Sisay legends toolbox plan)

Files: `impls/t6_plan_sisay.rs` (keep, prio, wish, hold_up, `fetch`), `tests/ai_t6_sisay.rs` (6 tests).
Hook outside my file (Tier-6-gated): `impls/legends.rs` `sisay_pick` asks `t6_plan_sisay::fetch` first, and `sisay()` gives
the activation utility 11 for Tier 6 (was 1.5 + 0.6 x MV, max 7).

## Why it did not assemble (traces)
- Sisay's activation was used, but late (turn 7+ in traced games) and with a value pick: `sisay_pick` took the dearest
  legendary CREATURE and excluded lands, so Mount Doom (the finisher in both lines) and Gaea's Cradle could never be
  fetched; the combos need Doom on the battlefield or in hand, so the lines were rarely `ready`.
- Tutors (Demonic, Vampiric, Worldly...) went for "closest combo's missing pieces" with no notion of the deck's lines.
- The shared mulligan rule (2-5 lands) ignores fast mana.
- The activation (5 mana) lost to cheaper casts; it is not offered at instant speed (opponent's end step) at all.

## What the plan does now
- fetch: any legendary permanent below her power, lands included; ranks line pieces (Derevi, Emiel, Doom, Cradle once 3+
  creatures, Marvin/Ioreth/Selvala/Shang-Chi, Kinnan), a bonus for the card that leaves its line one piece short, a small
  bonus for a new colour (it is +1 power). wish: the same ranking from the library, Bloom Tender included.
- keep: lands + mana makers (Moxen, Petal, dorks, Sol Ring, Vault) count; keep 3+ sources with 2+ spells, or 2 lands + 2
  sources with a maker/tutor; ship no-land hands, lands-only hands, 5+ lands without 3 spells.
- prio: Sisay at 92 from turn 2, tutors 76, Rhystic/Tithe/Remora early; Silence kept in hand; hold_up -2 (develop) until turn 3.
- go_now: no opinion (the shared check already lets it go; when ready it went every time in traces).

## Before / after (Sisay block, `--calibrate within --pool t6 --games 300 --seed 500000`, conservative profile)
| | baseline (stage5-baseline) | after |
| --- | --- | --- |
| win % (both profiles) | 29.8 / 29.3 | 25.8 / 30.3 (noise +-6) |
| wins by | damage 81%, Unicorn 16%, DDR 3% | damage 63%, Unicorn 24%, DDR 10%, other 2% |
| combo wins | 11 of 59 | 17 of 51 |
| median win round | 12 (avg 12.4) | 12 (avg 12.3) |
| combo tries (per game) | 11 (0.06) | 19 (0.10) |
| stopped | 0 | 2 (11%) |
| mulligans | 36.4% (50/14/8) | 30.8% (41/14/6) |

## Still differs from the spec
- Win round is still 12 (spec: turns 3-6, "double the mana each turn"). Sisay comes down turn 4-5 in most traced games and
  the fetch costs a whole turn's WUBRG (5 mana, all five colours) per piece; the five coloured sources are the limit.
- No instant-speed / end-step activation (spec: tutor at the opponents' end step, in response to a win, Ertai in response):
  `sisay()` only offers the ability on her controller's own turn. Needs engine option support at other windows.
- Pieces die: Sisay's deck eats the most removal in the table (200-300 per 300 games), so Derevi/Emiel/Marvin/Ioreth are
  often gone before the line is complete.
- Protection-first (Silence/Teferi before going) and the Ertai lock are not planned; the mulligan is still 31%.
- Conservative win rate fell (29.8 to 25.8; loose 29.3 to 30.3, within noise) although combo wins rose: it now spends mana on fetches instead of attacking; Stage 6 tuning.

Checks: cargo test --release all pass; build --all-targets no warnings; t3 lock identical.
