# Report: blue-spells (all 15 cards Full in `cardgen audit --mine`)

Module `impls/bulk_blue.rs`, tests `tests/cards_bulk_blue.rs` (29 tests), overrides at the end of
`data/card_overrides.toml`.

## How each card is modeled
- **Jace, Reality Sculptor**: walker framework. +1 empower Jace X (X = Islands) via `alela::empower`; -3 flags the Jace
  and its `attack` hook gives attackers of its controller -5/-0 until the controller's next upkeep; 0 exiles all but the
  bottom card of each opponent's library, only at 25+ loyalty among Jaces (checked on offer and on resolution).
- **Relm's Sketching / Echocasting Symposium**: `enter_token_copy`. Target chosen as it resolves: best creature (any
  controller for Relm) or artifact by pval + half the enter-trigger value; legends skipped (legend rule); Relm falls
  back to a token copy of one of our lands under 9. Paradigm: exiled as it resolves, `rebound` list keeps it, the
  engine's turn-start rebound step (just before the first main phase) casts a countering-capable copy
  (`cast_spell_copy`) each turn.
- **Phantom Interference**: `ctr=any soft=2`, counter mode costs {1}{U} (`counter_cost`); Spirit mode ({3}) via a
  hand option at an opponent's end step, and added to a counter cast with 3 spare mana (`spree_extra`).
- **Whoosh!**: resolve bounces ctx.target, kicked (`ctx.mode == "kick"`) draws. AI: own enter-trigger creature at an
  opponent's end step (kicked when affordable) or the best opposing permanent kicked after combat; the engine's
  removal option still covers the unkicked opposing use.
- **Amazing Acrobatics**: counter (`ctr=any`) and its tap mode taps the two best untapped opposing creatures when it
  counters (`counter_side_effects`); as the attacker (`hand_attack`) it taps blockers for a big attack.
- **Icy Reception**: `ctr=crleg soft=3` (new `counter_ok` value: creature or legendary); -5/-0 as the defender
  (`hand_defend`, wired for the elrond seat in `cardcode::defend_hooks`).
- **Sudden Setback / Vanish from Sight**: new removal kind `topbot` (`top_or_bottom`): token gone, commander to the
  command zone, else top if the owner values the card (card_worth >= 45 and cmc <= lands + 2; a land while under 5
  lands) and bottom otherwise. Setback also counters a spell (`ctr lapse`): cast.rs asks `spell_to_library(.., choose)`.
  Vanish adds surveil 1.
- **Kasmina's Transmutation**: a zur.rs lock Aura (`rem=kasmina etb`); `kasmina_attach` sets base 1/1, no abilities
  (neutered, flying/deathtouch/vigilance/lifelink off), `kasmina_detach` restores everything when the Aura leaves.
- **Retrieve the Esper**: 3/3 blue Robot Warrior artifact token; new `Ctx.from_gy` (set as a flashback resolves) adds two
  +1/+1 counters.
- **Rousing Read**: Aura spec +1/+1, flying, enter trigger draw two discard one.
- **Unfathomable Truths**: draw three + Eldrazi Spawn (devoid is irrelevant for an instant).
- **West Coast Expansion**: X = all spare mana in the main phase once X >= 3 (or 2 with a nearly empty hand); X >= 5
  casts the best Hero in hand free.
- **Sleight of Hand**: keeps the better of the top two by card worth, other to the bottom.

## Shared-file changes (all small, additive)
`state.rs` (`Ctx.from_gy`), `engine/cast.rs` (set `from_gy` for gy casts; Lapse branch -> `spell_to_library`),
`engine/stack.rs` (`counter_ok` "crleg", `counter_cost` Phantom, `spree_extra`/`spirit_token` in
`cast_counter_spell`, `acrobatics_tap` in `counter_side_effects`), `engine/removal.rs` (kind `topbot`),
`impls/zur.rs` (Kasmina in LOCKS + `kasmina_attach`), `impls/common.rs` (`aura_fall` -> `kasmina_detach`),
`cardcode.rs` (hand_defend for the elrond seat).

## Simplifications to know
- Nothing is chosen at cast time for target-at-resolution cards (Relm, Echocasting): the AI cheats by picking as it resolves.
- Setback/Acrobatics counter-side plays are chosen by the generic counter logic (ai_respond), not an elrond plan.
- Sudden Setback on the stack uses `Lapse`-style placement; the owner chooses via card_worth, not a human prompt
  (practice mode: HUMAN(phase 9) not done).
