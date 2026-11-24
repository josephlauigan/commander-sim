# Paired tests of the PR #394 package, 2026-11-24

Raw outputs of three paired runs on main 654db2060917380d2fc068d1a8e5c146ed20568d (the merge of PR #394; `commit.txt`),
loose profile, each deck's own tiers, 1,500 games per list per tier, seed 500000, same opponents and draws in both
lists. `status.txt` has the start and end times (its first line is the disk check). The findings are written up in
`audit/urdragon/FINDINGS.md`, `audit/yoshimaru/FINDINGS.md` and `audit/yshtola/FINDINGS.md`
(entries "2026-11-24: paired test of the PR #394 package").

## What was run

Each run is `commander-sim --deck D --pool all --swap "A=>B" ...` with one `--swap` per pair (the output's `Swaps:` line
prints them with `->`). The outputs do not echo the command line; it is rebuilt here from the `Swaps:` lines and from
the script that ran them:

- `yshtola-paired.txt`: `--deck yshtola --swap "Gleaming Splendor=>Esper Sentinel"` (t5).
- `urdragon-paired.txt`: `--deck urdragon` with six swaps, `Sylvan Library=>Dragonlord Silumgar`,
  `Deflecting Swat=>Teferi's Protection`, `Talisman of Hierarchy=>Steel Hellkite`,
  `Urza's Incubator=>Ancient Copper Dragon`, `Jeska's Will=>Korvold, Fae-Cursed King`,
  `Kyodai, Soul of Kamigawa=>Klauth, Unrivaled Ancient` (t5).
- `yoshimaru-paired.txt`: `--deck yoshimaru --swap "Akroma's Will=>Teferi's Protection"` (t4 and t5).

## How to read the direction

The swaps REVERSE the package. "Current" (the baseline, the left number) is the NEW list, the one main plays now.
"Swapped" (the variant, the right number) is the OLD list from before PR #394. So the printed `delta` is old minus new,
and a printed `BETTER` or `worse` is about the old list. "Swapped IN" rows are the old cards; "Swapped OUT" rows are the
new cards, so read the new cards' cast and stuck numbers from the "Swapped OUT" table. The findings entries restate
every result as new against old.

## Limits

- One seed (500000) and one profile (loose): a single draw of 1,500 games per list per tier.
- The old arm is not exactly the list that was played before PR #394. That PR also removed the plan rows of the cards that left, so
  those old cards play on the generic AI here: Ur-Dragon lost Klauth's wish row and `bigs` entry, the five old Dragons' `plays`
  entries and the Teferi's Protection hold row (it protected Terror of the Peaks and The One Ring); Y'shtola lost the Esper
  Sentinel priority row. For Ur-Dragon this handicaps the old list, and it still scored higher, so the -2.7 is, if anything, conservative.
  Yoshimaru's Teferi's Protection row was recorded as changing no play, so its old arm is fair.
- Library order: the deck lists are sorted and `--swap` puts each card at the other card's position, so a rebuilt old list has a
  different card order from the 11-23 list and the same seeds deal different libraries. Any comparison with the 11-23 baselines
  (Ur-Dragon 27.6%, Y'shtola 35.1%) is therefore an unpaired re-draw and not evidence of an engine change. Only the new-against-old
  comparison inside one run is paired.
- The Arzakon deck (`arzakon`) was NOT tested. The package also changed it; it needs a Cali paired run with
  `--swap-commander`, which has not been run.

The raw outputs have trailing spaces stripped (the repository's whitespace check requires it); no number or word changed.
