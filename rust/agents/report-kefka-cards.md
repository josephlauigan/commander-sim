# Report: kefka-cards

Branch models-kefka-cards. All 24 cards of the brief are Full in `cardgen audit --mine` (the verdicts and notes are in
`data/card_overrides.toml`, written after the code and its tests). Code: `rust/crates/sim-core/src/impls/kefka.rs`;
tests: `rust/crates/sim-core/tests/cards_kefka.rs` (54 tests: one or more per card, the discard policy, the flip, the
Crawler loop in 2-, 3- and 4-player games and with a short library).

## Mechanics (any card can use them)
- **Madness / mayhem**: tables `MADNESS` / `MAYHEM` (card name, cost) in kefka.rs. `zones::discard_cards` /
  `discard_index` call `kefka::discarded` once the cards are in the graveyard: mayhem remembers them for the turn
  (`Player.discard_log`), madness casts at once for the madness cost when the AI has a use (Ichor Slick: only with a
  creature it kills; the card is cast from the graveyard with zone `escape`, a modeling shortcut for "from exile").
- **Casting from the graveyard**: gy_options slots (mayhem, Demonic Embrace for 3 life and a discard, Twinned Vision
  flashback with a discard via zone `gy`, Postmortem Professor's return). Zone `escape` removes the card from the
  graveyard without exiling it.
- Garrison Excavator: `kefka::gy_left` is called where this deck's cards leave the graveyard (the casts above and
  Professor). **Not seen**: an opponent exiling my graveyard, and cards that leave it through code outside this deck's
  (the engine has no single point). Said in the card's note.
- Arcane Omens: own payment planner (`converge_plan`) choosing sources of distinct colours; the colour count rides in
  `ctx.x`. Humans (practice) don't get the option.

## Cards, in short
Duress / Dark Inquiry / Omens (take the best card by its worth to the opponent), Inner Fire (floating red), Decorum
Dissertation (Paradigm: exiled, free copy at each first main phase via `kefka::paradigm`), Ichor Slick (-3/-3 even
when not lethal, cycling, madness), Extinguish the Light (+3 life if MV <= 3), Grab the Prize (discard as a cost,
`ctx.x` = nonland), Twinned Vision, Stargaze (X from spare mana, life and library), Disintegrate (X chosen by the AI:
lethal for a creature worth killing or an opponent; exile instead of dying, no regeneration), Splatter Technique
(both modes, planeswalkers lose 4 loyalty), Alpharael, Scarlet Spider, Tidalmage, Soulbreaker, Sawblade Scamp (oil
counters in `DataKey::Counters`), Susurian Voidborn (artifact tokens sacrificed count; Warp {B}), Corroding
Dragonstorm (also returns to hand when a Dragon enters), Intellect Devourer (held cards as Opposition Agent's/Gonti's,
via `yshtola::take_card`), Gemstone Mine (taps counted in `DataKey::Used`), Postmortem Professor, Garrison Excavator.
Fixed on the way: Professor/Dragonstorm gained life once per opponent in the compiled abilities; now once (Oracle).

## AI plan (kefka.rs)
- `discard_choice`: graveyard-wanting cards first (madness, mayhem, flashback, Embrace, Professor), then the least
  useful card with 30 points off per new card type in the pile. Kefka's wheel (`mine::kefka_wheel`) makes the Kefka
  deck pick last, knowing the pile; `discard_worst` (looters) uses it too.
- `flip_utility`: {8} flip ranks 5.8 + 0.4 per opponent (after the plays worth 6+), less when holding up interaction,
  more with the Crawler out, and no option while the Crawler is out and the library can't finish the table.
- Crawler loop: `loop_safe` = library > the highest opponent life + 1. **Deviation from the brief** ("more than the lowest
  opponent's life"): one draw takes 1 life from every opponent, so the last opponent dies after as many draws as their
  life; starting with less than that kills some opponents and decks me. The Crawler's cast priority is 0 after the
  flip while unsafe; a held Crawler is never the discard.
- Decorum: opponent (lowest life) once Kefka is flipped, else yourself. Scarlet Spider and Demonic Embrace are held
  while an unflipped Kefka is out (discard fodder). Milestone "kefka" (the flip turn) is the plan-online metric.

## Shared-file changes (all small)
- state.rs: `Player.discard_log`, `DataKey::Warp`. zones.rs: two `kefka::discarded` calls in the discard paths.
- engine/cast.rs: `discard_worst` calls `kefka::discard_n` for the kefka key. engine/turn.rs: `kefka::paradigm` after
  the Main1 event. ai/brain.rs: `special_options` arm, `GENERIC_PLAYS`. ai/decks.rs: `deck_prio` arm. ai/mod.rs:
  `MAIN_STYLES` (kefka, 0.65 aggression, 0.55 caution). settings.rs / rules.rs / cast.rs (command-zone importance):
  "kefka" added to my-deck lists. impls/mod.rs: module registered.
- impls/mine.rs: `kefka_wheel` (order, choice), flip utility, milestone. sim-cli: decks.rs `FILES`/`MINE` (8 decks) and
  its test, args.rs `DECK_KEYS`, report.rs plan text. These lists and counts will conflict with the zada branch
  (same lines): merge by hand, both keys in each list.

## Checks run
cargo test --release (all pass), build --all-targets (no warnings), all_decks adaptive 3 and lookahead 1 (no panic),
smoke `--deck kefka --pool t3 --profile loose --games 96 --ai lookahead` (21.9%, no panic; 48 adaptive games: flips in
56% of games). A first smoke run panicked in 2 games (`discard_choice` returning None when the hand held only cards
taken by Intellect Devourer); fixed.

## Open / caveats
- Mistake to report: I ran `python3 --version` once (no script) before noticing the no-Python rule.
- The Demon type of Demonic Embrace is not tracked (nothing reads it). Blazing Firesinger is zada-cards's.
- The Voidborn warp exile is detected by the card sitting in my exile pile (a flag turn stamp keeps it for a later turn).
