# Galadriel (key `galadriel`, tiers t1 and t2): findings

**No longer tracked 2026-11-06 (JD).** The `galadriel` key left the simulator; the list is in `decklists/pool/retired/` and this file is history only.

Avery's deck, in `data/decks.json` `mine` with tiers t1, t2. Plan file through the generic policy (no hand AI). Loose
profile, look-ahead AI. This file starts with the recalibration remeasure; earlier history is in the commit log.

## Baseline 2026-11-06 (after the rewire, Doomsday and Phase C engine work)
Galadriel, t1 / t2, 2,000 games per tier, seed 500000, loose, look-ahead, build ef343cc1: **6.0%** (5.0-7.1) / **7.3%** (6.2-8.5),
**6.6% over t1, t2**. Old build (main 1e027077, 2026-11-05): 5.5% (4.5-6.5) / 6.3% (5.3-7.5), 5.9% over t1, t2. Change +0.5 / +1.0,
average +0.7: noise (intervals overlap, under 2 points), though both tiers move up. The 11/01 remeasure (4.8% average) is on an older
build and list.
Engine changes that can touch this deck: C2 Ramosian Revivalist (its target is now fixed on activation; the deck runs it) and C3
(attack taxes no longer idle attackers; opponents cast their commander on time, +1.5). No effect is separable from noise here.
C1/C1b and the rewire do not touch it.
Raw: `audit/baseline-2026-11-06/galadriel.txt` and `galadriel-2026-11-05-oldbuild.txt`.

## 2026-11-01: recalibration stage 2 remeasure

Galadriel, t1 / t2: **5.3% / 4.3%** (average 4.8%, CI 4.1-5.5) (2000 games, seeds 500000-501999, loose, look-ahead, defaults, rc=0; `audit/recal/stage2-2026-11-01.md` section 4, build efe45425). Plan online 49.5% / 53.4% of games. Stage 1, same
seeds: 5.3 / 3.8 (hand AI off), 5.0 / 5.0 (hand AI on); level. No list change since 10/31.
