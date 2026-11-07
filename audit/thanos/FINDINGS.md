# Thanos, the Mad Titan (key `thanos`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/thanos-artifacts-and-dragons.md`, built 2026-11-05 from the Thanos, Karn and Ur-Dragon decks (The
Ur-Dragon in the 99; Basalt Monolith + Rings of Brighthearth and Astral Dragon + Cursed Mirror as the two infinite
combos). **Tiers:** t5 (`data/my_decks.toml`; the old cEDH list played t6). **Plan:** `data/plans/thanos.toml`
(PROVISIONAL: written from the deck document's pilot guide, not from a paired test). **No sims yet.** The findings of the
previous list (the cEDH turbo list, t6) are in `FINDINGS-old-list.md`; the retired decks' findings are in
`audit/retired/karn/` and `audit/retired/urdragon/`.

## Baseline 2026-11-07 (after TD2; main 7c622f5f; look-ahead, loose, 10,000 games, seed 500000)
**18.9%** (18.2-19.7) at t5, against 18.6% (17.0-20.4) in Baseline B (2,000 games): no change in the number. The way it
wins changed: damage 65% of wins (was 50%), combo 34% (was 46%); plan online (first combo attempt) 6.9% of games (was
8.8%). That is TD2's trade: hasty Dragons and Utvara for the damage win, and the combo a little less often (Lotus Petal,
Gamble, Karn the Great Creator, Ugin and Portal gone). Who eliminates it: Miirym 19.5%, Kinnan 18.8%, Urza 16.1%, Zur 14.6%,
Yawgmoth 12.0%. The analyze run (2,000 games) is in `audit/baseline-2026-11-07/thanos-analyze.txt`. Raw:
`audit/baseline-2026-11-07/thanos.txt`.

**Analyze (2,000 games, t5).** Losses (eliminated in 80.8% of games, round 8.5): the combo decks, Kinnan 13.6%, Urza 13.6%,
Zur 11.2% and Yawgmoth 8.0% (about 46% together), then Miirym by combat 11.0% and triggers 7.0%. Wins: 94% of its damage is
combat; 0.56 opponents eliminated per game. Plan online 6.7% of games; Thanos resolves by turn 4 in 57% (median 4.0) and is
the card opponents remove most (0.42 per game). The new top end wins when it lands: Utvara Hellkite wins 53.4% of the
games it is cast in (+34.3 over the deck), Summon: Bahamut 43.5% (+24.3), Scion of Draco 36.6% (+17.4), but Utvara (43.5%
cast of the games drawn) and Bahamut (43.8%) sit in hand more than half the time, and Disdainful Stroke is cast in 47.4%.
Low win rates when cast (cheap cards, the telling ones): Mox Jasper +5.3, Crucible of Worlds +6.3, Walking Ballista +3.6,
The One Ring +4.5. Open: the combos rarely go (6.7%); more interaction for the combo decks is where the losses are.

## 2026-11-07: card-pile packages TD (tested, worse) and TD2 (adopted untested) (TD: paired swap run, t5, look-ahead, loose, 1,500 games per list, main b84ab9e6 with #292)
Twelve cards from JD's spare-card pile (`audit/jd-packages/2026-11-06-card-pile/`): Scion of Draco, Crucible of Worlds,
Strip Mine, Disdainful Stroke, Elesh Norn, Summon: Bahamut, Hangarback Walker, Meteor Golem, Glorybringer, Thundermaw
Hellkite, Mox Jasper, Utvara Hellkite (cheap speed and interaction, plus hasty Dragons for the Terror / Ur-Dragon damage
win, which is half the deck's wins).

**TD, with cheap cuts: WORSE (borderline).** Out: Wayfarer's Bauble, Urza's Bauble, Wastes, Red Elemental Blast,
Oblivion Stone, Ugin, Ratchet Bomb, Decimate, Duplicant, Gamble, Lotus Petal, Karn the Great Creator. 23.0% -> 20.2%
(-2.8, noise ±2.7; the game-shape table's band is ±3.0, so it is at the edge). Outside noise: plan online 11.3% -> 8.0%,
cards drawn 13.9 -> 12.8, spells cast 12.0 -> 10.6, tutors resolved 0.92 -> 0.75, rounds survived 9.39 -> 8.94, and
damage to opponents 19.9 -> 24.0 (almost all combat). Why: the cuts averaged mana value 2.73 and the cards in 5.09, so
the deck's average rose 3.10 -> 3.49. The cards out were cast in about 90% of their games (the Baubles, Lotus Petal,
Red Elemental Blast) and fed Urza, Mystic Forge and the combos; half the cards in sat in hand (stuck: Elesh Norn 58%,
Glorybringer 53%, Disdainful Stroke 53%, Utvara 52%, Bahamut 51%, Crucible 44%). The Dragons hit harder but the combo
was slower. Played well: Mox Jasper (cast 92%, round 4.3), Hangarback Walker (81%), Scion of Draco (71%). Raw:
`audit/jd-packages/2026-11-06-card-pile/runs/TD-thanos.txt`.

**TD2, the same cards with cuts matched on mana value: adopted by JD 2026-11-07, not measured** (the run was stopped
before it started). Out, all proxies: Blightsteel Colossus, Oblivion Stone, Wastes, Gamble, Wurmcoil Engine, Portal to
Phyrexia, Ratchet Bomb, Karn Liberated, Duplicant, Karn the Great Creator, Lotus Petal, Ugin. Cuts average 5.27 against
5.09 in; the deck's average stays 3.10 -> 3.07, and the cheap cards TD cut stay. It swaps the old fair-win top end
(Wurmcoil, Portal, Karn Liberated, Blightsteel, Ugin) for the new one. 16 Game Changers (Gamble left). To measure in the
next baseline: the stuck-in-hand numbers of the 5-9 drops, and plan timing against the 11/06 baseline.

## Baseline B 2026-11-06 (after #278-#284; main 63d783b5; look-ahead, loose, 2,000 games per tier, seed 500000)
**18.6%** (17.0-20.4) at t5, against 20.0% in the morning: -1.4, inside the noise (intervals overlap). Plan online 8.8% of games.
What changed at its table since the morning: Tier 5 pool decks no longer keep mana up for protection they rarely use (624 -> 71
held turns), and they now cast Garruk's Uprising and Welding Jar (#284); pool Urza's Whir of Invention now fetches with real X
(#281). A table that taps out a little more and an Urza that finds its pieces could both cost a point; the run can't separate
them from noise. Raw: `audit/baseline-2026-11-06-b/thanos.txt`.

## Baseline 2026-11-06 (after the rewire, Doomsday and Phase C engine work)
Thanos, **first number for the new artifacts-and-Dragons list**, t5 (High B4), 2,000 games requested, seed 500000, loose, look-ahead,
build ef343cc1: **19.7%** (95% CI 18.0-21.5) over t5, 1,978 games counted (22 hit an internal simulator error, see below). Even share is
25%: -5.3 points. Wins by damage 49%, combo 47%, timeout 3%; average turn of wins 11.6, of losses 8.0. The plan (first combo attempt) is
online in only 9.9% of games, yet combo is 47% of the wins. Deaths by opponent: Miirym 18.4%, Kinnan 19.0%, Zur 15.0%, Urza 14.9%,
Yawgmoth 12.5%; won when seated: Kinnan 32.0%, Miirym 29.4%, Urza 26.4%, Zur 25.5%, Yawgmoth 20.2%.
No old-build comparison: the old list played t6 and has no 11/05 run at t5 (its numbers are in `FINDINGS-old-list.md`).
**The plan file `data/plans/thanos.toml` is PROVISIONAL**: written from the deck document's pilot guide, not from a paired test, so this
number measures list plus provisional plan together. Engine changes that can touch the deck: C2 Maelstrom (t5 Miirym) and Forensic
Gadgeteer (t5 Urza) change two opponents; C3 opponents cast their commander on time (+1.5). The rewire (#270) is the list itself.
**Re-run after the crash fix:** the 22 skipped games panicked in the look-ahead's `cast_counter` (a colourless counterspell cast
from the top of the library under Mystic Forge, latent on main; fixed on #274). Re-run on the fixed integration build (b889e9c6,
same command): **20.0%** (95% CI 18.3-21.8) over t5, all 2,000 games, no errors; damage 50%, combo 46%, timeout 4%; plan online
9.8%. This is the number of record. Raw: `audit/baseline-2026-11-06/thanos-rerun.txt`.
First run (22 games skipped): `audit/baseline-2026-11-06/thanos.txt`.
