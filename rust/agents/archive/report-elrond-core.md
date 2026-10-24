# Report: elrond-core

All 22 cards of the brief are Full in `cardgen audit --mine` (verdicts in data/card_overrides.toml, one test or more each in
tests/cards_elrond.rs, 34 tests). Card code and the AI plan are in `impls/elrond.rs`.

## How each card is modeled
- **Elrond**: Etb hook (cards) + TokensEnter hook (tokens), one stack trigger per creature; the resolution count is
  `DataKey::Resolved` / `ResolvedAt` on his permanent; scry 1 each time, `ring_tempt` on the second. Toughness fixed to 2
  in data/carddb.txt (Oracle 3/2; it was 3/3).
- **The Ring / Gandalf**: `mine::ring_tempt` asks `elrond::ring_pick` for the elrond deck (evasive attacker; Gandalf only
  when alone) and calls `elrond::gandalf_draw` (the draw the audit note said "never happens"). Gandalf's flash and
  sorceries-at-instant-speed were already right (checked by a test through `end_of_turn_window`).
- **Stun counters**: `DataKey::Stun`; `turn::step_start` removes one instead of untapping a tapped permanent.
- **Cloudfin Raptor** (evolve, checked on entering and on resolution, tokens too), **Defiler of Dreams** (draw trigger;
  `defiler_discount` takes a {U} off in `mana::cost_of` with 15+ life, the 2 life is paid as the draw trigger resolves),
  **Deluge Virtuoso** (tap + stun; opus +1/+1 or +2/+2 at 5+ mana; compiled opus removed with `dsl = []`), **Rapier Wit**,
  **Matterbending Mage** (own best ETB creature vs. an opposing threat; X spell sets the unblockable stamp),
  **Mindseeker Oculus** (`alela::empower`), **Mouth of the Storm** (`DataKey::Weaken`, read in `values::epow`),
  **Orysa** (`self_cost`), **Professor Hulk**, **Rescue** (+1/+1 counter only for an artifact), **Silver Deputy**,
  **Dune Mover** (compiled search + toxic hook), **Soul of the Rapids** (hexproof keyword, tested), **Starbreach
  Whale / Starwinder** (warp: hand option, end-step exile, cast from exile on a later turn), **Textbook Tabulator**
  (increment), **The Wasp**, **Hope-Ender Coatl** (`soft=1` tag: cast as a counter, then enters), **Fear of Isolation**
  (`self_cast` returns a creature worth re-entering, else a tapped land; prio 0 with nothing to return).
- Silver Deputy and Dune Mover search for "a basic land or a Desert"; the deck has no Desert, so basic only.

## AI plan
Elrond at priority 97 (turn 3); with him out, creatures +10 until two entries are made; flash creatures (the registry
`prio` slot `flash_hold`): held on my turn until the second entry, cast at the end of another player's turn (80 with an
entry already, 60 with a second flash creature to follow); warp offered when the full cost is out of reach before combat or
as the second entry; `elrond_protect` counters the removal spell, else Unsummon / Whoosh! on Elrond or the Ring-bearer.
`elrond::best_own_etb` is the shared helper for "return my best enter-the-battlefield creature" (used by Mage, Rescue,
Fear; the blue-spells agent can use it for Unsummon, Whoosh!, Peerless Ropemaster, Exit Specialist, which are not mine).
Counterspell holding is the generic `brain::hold_value` logic (not changed).

## Shared-file changes (all small)
state.rs (4 DataKey variants), engine/turn.rs (stun), engine/values.rs (one line in epow), engine/mana.rs (3 lines in
cost_of), impls/mine.rs (ring_tempt: 2 calls), dsl/mod.rs (search: a land going to the top or graveyard no longer goes
to the hand; this fixes every compiled "search for a basic land, put it on top" card, not only mine), data/carddb.txt
(Elrond tgh), data/card_overrides.toml (22 verdicts), data/cards.json (exported).

## Notes
- `end_of_turn_window` runs only before the elrond seat's own turn, so "an opponent's end step" means the previous
  seat's. For seat 0 the round counter has already moved on, so the second-entry count reads 0 there.
- Win rate with the other agents' cards still partial: 2-6% against tier 3 (48 games), no panic.
