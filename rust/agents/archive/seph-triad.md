# Task: seph-triad — model Sephiroth, Fabled SOLDIER and Triple Triad; test Sephiroth in Sheoldred

Read `rust/agents/LASTDECK-RULES.md` first (two agents share the machine: `-j 8`, `--jobs 10`; the branch is
`tier0-precons`, not `elrond-cloud-decks`). Neither card is in `data/cards.json` yet: add them the way earlier tests
did (`cardgen deck sheoldred <list with the card swapped in> --dry-run` caches the Scryfall data; then `cardgen
export`; restore nothing else).

## Cards to model (Full in `cardgen audit`; check with a dry-run listing)
- **Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel** ({2}{B}, 3/3; back 5/5 flying), in `impls/sheoldred.rs`:
  whenever it enters or attacks, you may sacrifice another creature; if you do, draw a card. Whenever another creature
  dies, target opponent loses 1 life and you gain 1 life; if this is the fourth time this ability has resolved this
  turn, transform it. Back: as it transforms, you get an emblem "Whenever a creature dies, target opponent loses 1
  life and you gain 1 life"; whenever it attacks, you may sacrifice any number of other creatures, then draw that many.
  Find existing transform and emblem machinery first (grep `transform`, `emblems` in `state.rs`, `impls/rules*.rs`).
  AI: sacrifice only fodder (tokens, low-value creatures; never Sheoldred), target the opponent with the lowest life.
- **Triple Triad** ({3}{R}{R}{R} enchantment), in `impls/mine.rs` (the veyran deck's module): at the beginning of your
  upkeep, each player exiles the top card of their library; until end of turn you may play the card you own exiled
  this way and each other card exiled this way with lesser mana value than it, without paying their mana costs. Cards
  taken from opponents stay theirs (ownership: `tests/ownership.rs`). AI: play every free card worth playing.
One test per card at least (`tests/cards_sheoldred.rs`, `tests/cards_mine.rs` or a new `tests/cards_seph_triad.rs`).

## Measurement (after both are Full and all checks pass)
Sheoldred, paired: `--deck sheoldred --swap "Dark Deal=>Sephiroth, Fabled SOLDIER"` (use the exact card name the
data has), Tiers 1-5, 1,000 games per tier, look-ahead, loose, `--jobs 10`, run in the foreground with a long timeout.
Save raw output to `audit/sheoldred/sephiroth/<tier>.txt` and a README table like `audit/sheoldred/one-ring/README.md`.
Do not test Triple Triad; the lead picks its deck and cut.

## Done when
The LASTDECK-RULES checks pass (no Python; memory cap; commit often; handoff; 15-line report, line 1 DONE or HANDOFF,
line 2 model ID). Full report in `rust/agents/report-seph-triad.md`.
