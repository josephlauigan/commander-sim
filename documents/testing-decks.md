# Testing a change to one of JD's decks

How a card swap or package for one of JD's decks is checked, run and recorded. The rules here come from
[`CLAUDE.md`](../CLAUDE.md), which wins if the two ever differ.

## 1. Check the swap before proposing it

Before proposing, testing, modeling or buy-advising a card, check all three and say so:

1. **Commander legality.** `legalities.commander` must be `legal` (in `data/scryfall_cache.json`, or
   `https://api.scryfall.com/cards/named?exact=<name>`). Banned or not legal: don't propose it.
2. **Colour identity.** The card's `color_identity` must fit the deck's `ident` in `data/decks.json` (Sheoldred is mono-B; karn_gilded is WUBRG; karn_reforged is colourless).
3. **The cut is in the current list.** Fetch `origin/main` first and check that the card going out is in the deck's
   `cards` in `data/decks.json`. FINDINGS files can name cards that have since left.

Then check how the card is modeled: `cardgen audit --mine`, or `commander-sim --deck <key> --cards` for every card's
tags and abilities. The swap run prints the status too (below). A card that isn't in `data/cards.json` can't be played:
add it to the list and run `cardgen export` first ([`documents/cards.md`](cards.md)).

## 2. The deck's own tiers

Each deck plays only the opponent tiers listed for it in `data/my_decks.toml` (`tiers = [...]`, also in
`data/decks.json`):

| Decks | Tiers |
| --- | --- |
| urdragon | t5 |
| arzakon | t5-t6 |
| sheoldred, marchesa | t3-t4 |
| karn_gilded, karn_reforged | t4-t5 |

`--pool all`, `--all-decks`, `--analyze`, swap runs and `searchtest` use only these tiers; `--deck X --pool tN` outside
them is refused. Report averages over the deck's own tiers and label them ("over t3-t4"). `--any-tier` lifts the
limit; use it only when JD asks.

## 3. The paired swap run

```
commander-sim --deck seph --pool all --swap "Blood Artist=>Grim Tutor"        # every one of the deck's tiers
commander-sim --deck seph --pool t3 --swap "A=>B" --swap "C=>D"               # a package: several swaps at once
commander-sim --deck seph --pool t3 --swap "A=>B" --ai adaptive               # a quick read (seconds)
commander-sim --deck seph --pool t3 --swap-commander "Old Cmdr=>New Cmdr"     # a commander change, paired
```

The run plays the current list and the swapped list on the same seeds. The seat draw depends only on the seed and the
deck keys, and every seat shuffles its own library from its own stream, so both lists face the same opponents, seats
and opponent draws; only the deck's own draws differ (the card coming in goes to the end of the list,
`apply_swaps` in `sim-cli/src/main.rs`). The report gives the paired
difference (variant minus baseline, from per-seed pairs) and its noise band (two standard errors), and a verdict:
BETTER, WORSE, or NO CLEAR EFFECT. Defaults: 1,500 games per list, the look-ahead AI, the loose profile (the only
profile in use; don't pass `--profiles conservative,loose` unless JD asks).

**The preflight** runs first (`rust/crates/sim-cli/src/preflight.rs`):

1. The AI-rule check (instant): each swapped-in card's cast priority on mid-game boards, printed as
   `AI: <card>: priority N (deck rule|generic)`, or a `WARNING AI:` line when the deck's AI would never rank it and it
   is not an instant or a ctr / rem / wipe / prot card.
2. A dry run (about a minute): 200 adaptive games on the first tier. A swapped-in card drawn in 20 or more games and
   cast in under 5% of them stops the run with exit code 3.

`--no-preflight` skips both; `--preflight-only` stops after them.

**The explain breakdown** is on by default (`--no-explain` turns it off; don't, for a decision). Before the run: the
model status of each swapped-in card (Full ... Unmodeled; a WARNING on Partial and Unmodeled). After it: the swapped
cards' drawn / cast / average round / stuck-in-hand numbers for both lists, and the game shape (win, survival, how it
wins and dies, plan timing, commander removals, mana and hand at turns 4 and 8). It costs no extra games, and the
`delta` line is unchanged (`rust/crates/sim-cli/src/explain.rs`).

**A commander change** is a paired run too: `--swap-commander "Old Commander=>New Commander"` (once; it combines with any
number of `--swap` pairs). The variant list is the current list with the new commander in the old one's slot, then the
`--swap` pairs applied to its 99 (so the old commander may come in as a card: `--swap "Some Card=>Old Commander"`).
Because the commander sits outside the library, the 99 keep their positions and both lists get the same library from the
same seed, as for `--swap`. Before any game it refuses: an OLD that is not the deck's commander, a deck with two
commanders, a NEW with no entry in `data/cards.json`, a NEW outside the deck's colours, and a variant list that is
not legal under the deck's format by cardgen's deck checks (Commander legality or the format's ban list, "can be
your commander" (a legendary creature, or text that says so), every card inside NEW's colour identity, singleton,
100 cards). The explain breakdown, the preflight and `--no-explain`, `--no-preflight` and `--preflight-only` work as
for `--swap`: the new commander has its model-status line, its AI-rule line (the deck's priority for the commander)
and a dry-run line (a WARNING only when it is cast in under 5% of the 200 games: a new commander never stops the run), and the card tables list
the new commander (variant) and the old one (current) marked `(cmdr)`. A commander starts in the command zone, so its row
reads drawn 100%, cast% of all games and stuck% = games it was never cast in. The deck's plan file and AI rules belong
to the deck key, not the commander: a commander with its own engine code needs that code and its model status first.

## 4. Deeper looks

```
commander-sim --deck seph --pool t4 --analyze                     # how the deck wins and loses
commander-sim --deck seph --pool t4 --analyze --swap "A=>B"       # the same for the variant
commander-sim --deck seph --pool t3 --trace 7                     # play-by-play of game 7 (seed 500007)
```

`--analyze` reports how the deck wins, who kills it and with what, game-plan timing, damage by source, and a card
report. `--trace N` prints one game's log (the variant's with `--swap`) and exits.

## 5. Record it

After every run, update `audit/<deck>/FINDINGS.md`: the deck's living list of strengths and weaknesses and what was
tested, with the date, the command (tiers, games, AI), the result against its noise band, where the raw output is, and
why the swap helped or not. Tell JD the why along with the number.

If the change is kept, write the new list with `cardgen deck <key> new-list.txt --log "Why."` (it logs the change in
`decklists/CHANGELOG.md`), then `cardgen export`.

## Reading the numbers

- A win rate is reported against the 25% even share, with a Wilson 95% interval: about +/- 2.5 points at 1,500 games.
- The paired difference is the reliable part. A difference inside its noise band is no clear effect.
- The simulator's games are not yet close to real games (see [`documents/realism.md`](realism.md)): it is slower than
  real tables and its win types differ. Treat a result as a comparison between two lists in the sim, not a prediction
  of a real win rate.
