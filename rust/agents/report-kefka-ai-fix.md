# Report: kefka-ai-fix (Sonnet 5.5, claude-sonnet-5-5)

Branch `kefka-ai-fix`, from `kefka-veyran-trade`. All measurements: `--deck kefka --pool tN --profile loose --games 600
--seed 500000 --ai adaptive --jobs 8 --analyze`, same seeds before and after.

## Win rates (per tier, before -> after)

| Tier | before | after |
|---|---|---|
| T1 | 20.5% | 28.0% |
| T2 | 23.2% | 30.0% |
| T3 | 14.3% | 20.5% |
| T4 | 26.3% | 31.0% |
| T5 | 26.3% | 29.5% |
| mean | 22.1% | 27.8% |

(600 games per tier: about +-3 points each, +-1.4 on the mean. The Crawler priority alone moved every tier up in
three steps, 62/55 -> 75/70 -> 84/70, so the direction is solid.)

## Cast rates (share of the games where the card was drawn that it was cast), per tier T1..T5

| Card | before | after |
|---|---|---|
| Kaervek's Purge | 0.0 all | 14.1, 10.6, 7.6, 8.3, 12.6 |
| Mystic Confluence | 0.3, 1.7, 1.1, 1.5, 0.5 | 1.0, 3.7, 0.7, 1.6, 2.0 |
| Arcane Omens | 11.2, 14.0, 9.3, 12.7, 13.2 | 10.8, 12.6, 10.3, 12.5, 12.1 |
| Splatter Technique | 17.3, 15.3, 13.4, 14.0, 14.4 | 15.3, 12.3, 12.5, 11.2, 13.0 |
| Triskelion | 14.4, 18.8, 21.5, 14.6, 11.1 | 21.8, 27.5, 23.0, 23.3, 20.8 |
| Rapturous Moment | 17.2, 21.4, 15.4, 12.1, 15.0 | 26.8, 30.1, 25.6, 23.1, 18.3 |
| Inner Fire | 24.5, 18.2, 16.4, 18.7, 18.9 | 22.9, 14.9, 13.1, 16.2, 17.6 |
| Psychosis Crawler | 20.5, 27.2, 22.4, 22.3, 17.8 | 62.7, 69.2, 70.6, 64.5, 61.3 |

## Each card

- **Kaervek's Purge: measurement bug plus a small gate.** `purge_go` (impls/jodah.rs) never put the card in
  `cast_names` (the set the card report reads), so it showed 0% although it was cast; it now records the cast (and
  `spells_cast`, and a log line). Real rate once counted: 3-10%. The gate `pval >= 3` and utility `pval*0.5-0.8` made
  it a weak play; with the flipped Kefka the power it deals to the creature's controller is life lost on our turn =
  cards, so it now also fires on small creatures (power >= 2) and gets `+0.7` per card (cap 6). Genuinely rare
  otherwise: it needs a creature whose mana value the table can pay on top of {B}{R}.
- **Psychosis Crawler: AI gap.** Prio was 30 while Kefka was out and unflipped, 44 otherwise, below the usual 46-54
  band of the deck's filler, and it costs 5 like Kefka. Every card drawn pings every opponent, so Kefka's own discard
  and loot draws are a clock even before the flip. Now 70 (Kefka not out), 84 (Kefka out, unflipped), 80 flipped and
  `loop_safe`, 0 flipped and unsafe (the guard is kept; `flip_utility` still refuses the flip with an unsafe Crawler
  out). Win rate +5.7 points on the mean. In a 300-game trace of stops the Crawler was almost never stuck for lack of
  an option; it just lost the mana to better-valued plays.
- **Triskelion: AI gap (small).** Prio 74 flipped (each of three pings is a card), 56 with Kefka out, generic before.
- **Rapturous Moment: AI gap (small).** Prio 58 with two or more other nonland cards in hand (draw three, discard two,
  and five red back is a free filter whenever the mana can be spent); generic otherwise.
- **Mystic Confluence: genuine, plus a measurement bug.** {3}{U}{U}{U} with 6 Islands and about 12 blue sources, so
  UUU is rare. The draw-three mode is offered by `end_of_turn_window` (utility 2.5 against pass -1.5) and does get
  cast, but `confluence_draw` did not record `cast_names`; it does now (ai/brain.rs, two lines).
- **Arcane Omens: genuine.** {4}{B} that only strips opposing hands; Kefka gets nothing from it. Utility
  `1.5 + 1.2 * min(colours, biggest opposing hand)` already puts it in the middle of the pack. Left as is.
- **Splatter Technique: genuine.** {1}{U}{U}{R}{R} with 6 Islands; both modes are offered whenever UURR is payable.
- **Inner Fire: genuine.** Pays off only with a hand of 6 or more (prio 38) or a flip within reach (72); both already
  there. Left as is.

## Veyran's incoming cards (checked with `--swap`, and a test each)

- **Deflecting Swat, Slip Out the Back:** work for Kefka through `protect_response "kefka" -> veyran_protect`
  (Kefka is a commander permanent so the Swat is free). Tests added; no code change.
- **Underworld Breach:** `kefka_prio` had no Breach case (the generic path gave it 0), so it was never cast. Now 56 with
  at least two escape candidates in the graveyard and 5 mana (the same rule as Sauron's). `breach_escape` (impls/mine.rs)
  exiled the three escape cards and moved the escaped card without telling Garrison Excavator; for the kefka deck it
  now calls `kefka::gy_left` for the three exiled cards and for the escaped card (two Spirits, as Oracle's two
  separate "leave the graveyard" events).
- **Niv-Mizzet, the Firemind:** the DSL models it ("draw -> 1 damage to any target"), aimed at an opponent. Both
  triggers are mandatory, but the target is ours. Added `niv_safe` (library > the opponents' life added up, since one
  ping takes 1 life from one opponent) and `loop_guard` (Kefka flipped and the Crawler or Niv out and the matching
  loop unsafe). While the guard holds, the DSL's "any target" damage for the kefka deck only finishes a player or hits
  a creature, never a face (dsl/mod.rs, 4 lines, gated on `loop_guard`, which is false for every other deck). Because
  of that guard the flip is not blocked by Niv (it is for the Crawler). Cast prio is the Crawler's ladder
  (70 / 84 / 80); `discard_choice` keeps Niv like the Crawler. Note: the DSL stops chains at `MAX_DEPTH` 6, so a
  Niv + flipped Kefka chain ends after about 8 draws in the sim instead of running to the end.
- **Jin-Gitaxias, Progress Tyrant:** the engine already copies the first artifact/instant/sorcery each turn and
  counters the first one an opponent casts (cast.rs); a test shows a Duress copy. His generic cast priority is 40 or
  more with the mana for him; no change.
- Swap run (`Hallway Heckler=>Niv`, `Rummaging Goblin=>Jin`, `Arcane Omens=>Breach`, `Cackling Fiend=>Deflecting Swat`,
  `Rank Rat=>Slip Out the Back`, T3, 200 games adaptive): 24.0 -> 26.0, within noise (+-7.4), no panic.

## Shared-file changes (all small, none changes another deck)

- `impls/jodah.rs`: `purge` gate and utility for the kefka deck only; `purge_go` records the cast (stats only).
- `ai/brain.rs`: `confluence_draw` records `cast_names` and `spells_cast` (stats only).
- `dsl/mod.rs`: "any target" damage honours `kefka::loop_guard` (false unless key is kefka).
- `impls/mine.rs`: `breach_escape` calls `kefka::gy_left` when the key is kefka.

## Invariance

Veyran, Sauron and Jodah, `--seed 502000 --games 200 --ai adaptive --profile loose`, T2 and T4, binary before the
change against binary after: output byte for byte identical in all six runs.

## Checks

- `cargo test --release`: all pass (new: `tests/cards_kefka_ai.rs`, 18 tests).
- `cargo build --release --all-targets`: no warnings.
- `all_decks adaptive 3` and `all_decks lookahead 1`: finished, no panic.
- `cardgen audit --mine`: kefka 73 unique cards, 0 not Full.
- `--deck kefka --pool t3 --profile loose --games 48 --ai lookahead --jobs 8 --brief`: 27.1%, finished.
- Not run: the look-ahead AI win-rate table (the tuning was measured with adaptive only, as the brief asks).

## Lead's review (Opus)

- With no creature to hit, the guarded ping now goes at yourself, not nowhere: the trigger is mandatory, and
  the Kefka trigger counts only opponents (dsl/mod.rs).
- Checked: 1,152 tests, 0 failed; no warnings. Fixed-seed invariance holds for Veyran, Sauron, Jodah, Sephiroth and
  Y'shtola at t2 and t4.
- The gain holds up on a fresh seed (510000, adaptive, 600 games per tier):

  | Tier | Before | After |
  |---|---|---|
  | T1 | 17.0% | 25.2% |
  | T2 | 22.2% | 31.3% |
  | T3 | 15.0% | 18.5% |
  | T4 | 22.5% | 27.8% |
  | T5 | 27.7% | 31.2% |
  | Mean | 20.9% | 26.8% |
