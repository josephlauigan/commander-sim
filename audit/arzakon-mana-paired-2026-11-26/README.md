# Paired tests of the Arzakon mana package, 2026-11-26

Raw outputs of the paired runs on the branch `arzakon-mana-30` (the commit is in `commit.txt`; `status.txt` has the disk check and the
times), loose profile, the deck's own tiers (t5 and t6), 3,000 games per list per tier, seed 500000, look-ahead AI, explain on, same
opponents and draws in both lists. `run.sh` is the script that ran them. First Cali (1v1, 30 life) measurement of this list.

## The final list and what was run

The list is out Llanowar Elves, Avacyn's Pilgrim, Exsanguinate, Coalition Relic; in Mana Confluence, City of Brass, Misty Rainforest,
Chromatic Lantern (Meteor Golem stays, JD's choice). This is the BASELINE ("current") of both runs, with the engine fixes of the last
commit (Mana Confluence's life payment and guards, Chromatic Lantern with multi-mana lands, the AI tapping Confluence and City late).
The printed `delta` is variant minus baseline; "swapped IN" rows are the variant's cards.

- `old-vs-new.txt`: the OLD list (27 lands) as the variant: `Mana Confluence=>Llanowar Elves`, `City of Brass=>Avacyn's Pilgrim`,
  `Misty Rainforest=>Exsanguinate`, `Chromatic Lantern=>Coalition Relic`. Read a printed `-1.9` as the new list being 1.9 points better,
  and a printed `WORSE` as being about the old list. Run with `--no-preflight`: the preflight stopped the first attempt because the
  AI casts Exsanguinate in almost no game (0 of 20 draws in its dry run; the main run had it cast in 1.1% and 2.8% of the games it was drawn in).
- `a-vs-new.txt`: arm A, Meteor Golem out and Avacyn's Pilgrim kept, as the variant (`Meteor Golem=>Avacyn's Pilgrim`).

## `round1-before-engine-fixes/`

The first round, on the earlier engine and the other arm (Meteor Golem cut, Avacyn's Pilgrim kept, as its baseline): kept as the record
only. Its `a2-vs-new.txt` is the Pilgrim-cut arm; its numbers are not comparable with round 2 (different list and engine).

## Limits

- One seed and one profile (loose): a single draw of 3,000 games per list per tier. The noise beside each delta is the paired band; the
  by-axis table prints a wider band (two standard errors of the difference).
- The explain output has no colour-source counts and no life lost to lands, so those are not measured.
- Engine drift: both lists and the opponents' decks that play Mana Confluence or City of Brass use the changed engine (see
  `audit/arzakon/FINDINGS.md`, 2026-11-26), so no number here is comparable with an older run.
- The plan has no lines for the Thassa's Oracle and Tinker routes, where most of the blue and black gain lands.
