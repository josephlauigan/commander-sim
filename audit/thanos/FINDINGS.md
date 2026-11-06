# Thanos, the Mad Titan (key `thanos`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/thanos-artifacts-and-dragons.md`, built 2026-11-05 from the Thanos, Karn and Ur-Dragon decks (The
Ur-Dragon in the 99; Basalt Monolith + Rings of Brighthearth and Astral Dragon + Cursed Mirror as the two infinite
combos). **Tiers:** t5 (`data/my_decks.toml`; the old cEDH list played t6). **Plan:** `data/plans/thanos.toml`
(PROVISIONAL: written from the deck document's pilot guide, not from a paired test). **No sims yet.** The findings of the
previous list (the cEDH turbo list, t6) are in `FINDINGS-old-list.md`; the retired decks' findings are in
`audit/retired/karn/` and `audit/retired/urdragon/`.

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
