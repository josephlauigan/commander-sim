# Paired tests of the Arzakon mana package, 2026-11-26

Raw outputs of two paired runs on the branch `arzakon-mana-30` (commit in `commit.txt`; the start of `status.txt` has the disk check
and the times), loose profile, the deck's own tiers (t5 and t6), 3,000 games per list per tier, seed 500000, look-ahead AI, explain on,
same opponents and draws in both lists. `run.sh` is the script that ran them. First Cali (1v1, 30 life) measurement of this list.

## What was run

Each run is `commander-sim --deck arzakon --pool all --games 3000 --swap ...`. The baseline ("current") is the NEW list on the branch;
the variant ("swapped") is built by swapping cards back:

- `old-vs-new.txt`: `Mana Confluence=>Llanowar Elves`, `City of Brass=>Meteor Golem`, `Misty Rainforest=>Exsanguinate`,
  `Chromatic Lantern=>Coalition Relic`. The "swapped" list is the OLD list (27 lands). The printed `delta` is old minus new:
  read `-1.9` as the new list being 1.9 points better, and a printed `WORSE` as being about the old list.
- `a2-vs-new.txt`: `Avacyn's Pilgrim=>Meteor Golem`. The "swapped" list is arm A2 (Pilgrim out, Meteor Golem kept; the new list cuts Golem).
  The printed `delta` is A2 minus new.

## Limits

- One seed and one profile (loose): a single draw of 3,000 games per list per tier. The noise printed beside each delta is the paired
  band; the by-axis table prints a wider band (two standard errors of the difference).
- The explain output has no colour-source counts and no life lost to lands, so those are not measured.
- Engine drift: Mana Confluence now pays 1 life per tap in both lists and in the opponents' decks that play it (all t6:
  kinnan-bonder-mana-combo, kraum-tymna-blue-breach-midrange, rograkh-silas-turbo-breach, rograkh-thrasios-creature-combo, sisay-legends-toolbox), so no number here is comparable with an older run.
- The plan has no lines for the Thassa's Oracle and Tinker routes, where most of the blue and black gain lands.
