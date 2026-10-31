# Ghave, Guru of Spores (key `ghave`): findings

The deck's living list of strengths, weaknesses and what was tested. Opponents: its matched tiers, t3-t4
(`data/my_decks.toml`; Seph's seat since 2026-10-30). Loose profile, look-ahead AI.

## First numbers (2026-10-31, 2000 games per run, seeds 500000-501999, rc=0 each)

| Run | Ghave (plan file, generic policy) | Seph/Atraxa (its hand AI, origin/main before the rename) |
| --- | --- | --- |
| vs t3 | **12.3%** (CI 11.0-13.9) | 22.1% (CI 20.3-23.9) |
| vs t4 | **11.3%** (CI 10.0-12.8) | 17.6% (CI 16.0-19.3) |
| average over t3-t4 | **11.8%** | 19.9% |
| goldfish kill turn (median / mean) | 10 / 10.5 | 11 / 11.2 |
| goldfish: how it wins | combo 54%, damage 41%, commander damage 5% | combo 58%, damage 41%, commander damage 2% |

The Seph column is the old list with its 1,111-line hand AI (ai/seph.rs, now removed). The Ghave column is the new
list (no blue: no counterspells, no Atraxa, no Rhystic Study) with no deck AI at all. The two changes are mixed
together: part of the drop is the list, part is the AI.

## Strengths
- Fast alone: goldfish median kill own turn 10 (Seph 11). Ghave is cast in 95% of goldfish games, median own turn 5.
- Many lines, and the AI finds them. In goldfish games it went for Mikaeus + Triskelion (most often), Ghave + an altar
  + Crusade / Doubling Season + a payoff, Yawgmoth's undying loop, Mikaeus + Finks, Nim + Ashnod + Grave Titan,
  Ghave + Ashnod + Triskelion and Finks + Crusade. At the table, combo is 53-57% of the wins.

## Weaknesses
- At the table it rarely gets there: the first combo attempt comes in only 7.7% (t3) / 6.3% (t4) of games, while
  goldfish games reach a ready line in about half. Losses come at own turn 8.6 (t3) / 7.4 (t4): it dies before the
  loop.
- Who kills it: at t3 Tergrid 27.8% of games (Seph 26.5%), Aurelia 20.1%, Marwyn 17.1%; at t4 Edgar Markov 28.8%,
  Krenko 21.7%, Yuriko 15.1%. Go-wide combat and Tergrid punish a board of Saprolings and utility creatures. Seph had
  blue interaction and reanimated 6+ power bombs early (its plan was online in 71% of t3 games).
- Reanimation now runs only through the generic policy (it reanimates a target already in the graveyard; Entomb
  fills). Seph's hand AI planned fill -> tutor -> reanimate. In goldfish games Necromancy, Animate Dead, Victimize,
  Tortured Existence and Yawgmoth's Will are often held with no target.

## How it wins and dies
- Wins: a catalogue loop (about 55%) or damage (about 45%), around own turn 10-11.
- Dies: to combat (Aurelia, Edgar, Krenko, Marwyn) and to Tergrid, before turn 9.

## Model status
- cardgen audit: 92 unique cards, 0 not Full.
- Ghave: both abilities come from the ability language (paying "remove a +1/+1 counter" is new; before it, his
  Saproling ability never ran). He enters with five counters on any entry (card code; Doubling Season makes ten). His
  {1} sacrifice answers removal and wipes (plan knob `paid_outlets`). The loops run as their result through the combo
  catalogue (impls/ghave.rs).
- Gaps: outside the catalogue lines the AI uses each Ghave ability at most three times a round, and it does not use
  them to grow Triskelion or a lethal attacker. Sephiroth, Fabled SOLDIER counts as a payoff for the lines, but tutors
  don't fetch him for them.

## Next to try
1. `--analyze` at t3 and t4: when Ghave is cast, how often he is removed, what the deck does on turns 3-6.
2. Plan rules for the reanimation start (Entomb / Stitcher's Supplier, then Reanimate / Animate Dead, with a gy:
   condition), as Seph's AI did; a hold for Grand Abolisher on the combo turn.
3. A mulligan rule with pieces (a line piece, or a reanimation pair).

The old Seph notes (audit/seph/FINDINGS.md) describe the Atraxa list and its swaps (Priest of Gix, the Stones); none
of them applies to this list.
