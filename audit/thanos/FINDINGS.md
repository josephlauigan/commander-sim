# Thanos, the Mad Titan (key `thanos`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/thanos-artifacts-and-dragons.md`, built 2026-11-05 from the Thanos, Karn and Ur-Dragon decks (The
Ur-Dragon in the 99; Basalt Monolith + Rings of Brighthearth and Astral Dragon + Cursed Mirror as the two infinite
combos). **Tiers:** t5 (`data/my_decks.toml`; the old cEDH list played t6). **Plan:** `data/plans/thanos.toml`
(PROVISIONAL: written from the deck document's pilot guide, not from a paired test). **No sims yet.** The findings of the
previous list (the cEDH turbo list, t6) are in `FINDINGS-old-list.md`; the retired decks' findings are in
`audit/retired/karn/` and `audit/retired/urdragon/`.

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
**Check:** the WARNING lists 22 skipped games (1.1%) on `Option::unwrap()` on None at `crates/sim-core/src/engine/stack.rs:1668` (seeds
500111, 500163, 500222, 500317, 500422 and 17 more). Possibly tied to the C2 t5 cards above (a guess, not investigated).
Raw: `audit/baseline-2026-11-06/thanos.txt`.
