# Phase 3, PR 5: identity for exiled cards (design)

Branch `phase3-exile`, stacked on `phase3-mana` (PR 4). Brief: `rust/agents/phase3-mana-exile.md`. Handoff:
`audit/phase3/exile-handoff.md`.

## The zone

`engine/exile.rs`. `Player::exile: ExileZone` replaces `Vec<CardId>`. The zone holds the cards (oldest first) and,
in step with them, one object per card:

| field of `ExileObj` | meaning | rule |
|---------------------|---------|------|
| `id: ExileId` | a fresh number on every card put into exile, never reused in that owner's zone; (owner, id) names the object for the whole game | 400.7 |
| `until: Option<PermId>` | exiled "until ~ leaves the battlefield" by this permanent (the O-Ring family) | 610.3 |
| `silver: bool` | the card has a silver counter (Karn, Scion of Urza; Karn, the Great Creator's wish pile is not this) | 122.1 |

A card that leaves exile and is exiled again is a new object (`push` always makes a new id): an effect that remembers
"the card exiled with ~" remembers the object, so a card with the same `CardId` exiled later by something else is not
it, and a counter on the old object is gone (400.7: "it has no memory of and no relation to its previous existence").

Read API: `Deref<Target = [CardId]>` (`contains`, `iter`, `len`, indexing, `last`), `&zone` iterates, and comparisons
with `Vec<CardId>` / arrays work, so the ~200 sites that only look at which cards are in exile compile unchanged.
Write API: `push(c)`, `add(c, until) -> ExileId`, `extend`, `remove(i)`, `pop`, `take(c) -> bool` (the oldest object
of `c` leaves; replaces the ~20 local `remove_card` / `remove_first` copies at exile sites), `retain`, `clear`.
Objects: `objs`, `entries`, `obj(i)` / `obj_mut(i)`, `find(id)`, `newest(c)`.

Cost: an empty zone is two empty `Vec`s and a counter, so copying a game with empty exiles allocates nothing more
than before; a non-empty one copies one extra 12-byte object per card. `examples/clone_cost` before and after.

## Links and how each current tracking maps onto them

| tracking today | where | maps to | stage |
|----------------|-------|---------|-------|
| `DataKey::Oring` on the exiling permanent: a list of (card, owner) | `common::oring_exile` / `oring_return` (O-Ring, Banishing Light, Journey to Nowhere, Grasp of Fate, Static Net, Anikthea's enchantments, Calix's `CalixLink`, Fiend Hunter, All-Fates Stalker ...) | the object's `until = Some(src)`; the list keeps (card, owner, `ExileId`) (`common::link_exiled`; `common::linked` finds it again) for the AI's readers (`alela.rs` "holds something", `t2.rs` reset) | B writes, C reads |
| `DataKey::Serenity` (Angel of Serenity: back to the hand when it leaves), `DataKey::Held` (Detention Sphere: every card of the name) | `t2.rs`, `rules2.rs` | the same `link_exiled` entries | B writes, C reads |
| "returns if any card with that `CardId` is in exile" | `oring_return` | returns the object with that id, only if it is still in exile (`find(id)`) | C (bug fix) |
| `Player::silver: Vec<CardId>` | `impls/karn_engines.rs` | the object's `silver` | B writes, C reads; the `Vec` is deleted in C |
| `Player::impulse: Vec<CardId>` ("held in hand"; the cards are in the hand, not in exile, and the owner's own end step puts back the unplayed ones) | `engine/turn.rs`, `zones.rs` (Urabrask), `tutors.rs` (Jeska's Will), `dsl` "impulse", ~15 card sites | stays "held in hand" (moving the cards into exile would mean a cast-from-exile path for every caster); the permission now ends at the end of the turn it was given in, whoever's turn that is (514: the cleanup of each turn) | C |
| `impulse_long: Vec<(CardId, u32)>` (Prosper, Reckless Impulse: until the end of your next turn) | `t45_prosper.rs`, `turn.rs` | unchanged (already turn-counted) | - |
| `oath_return`, `valakut_cards`, `neriv`, `necro_wait`, `suspended`, `rebound`, `adv_done`, Game's `zur_due` / `eot_returns` / `t12_due` / `t12lp` / `beheld` / `t12_valg` ... | per card | each holds a `CardId` for a card it moves itself, back within the same turn or at a fixed time; none was found returning the wrong card in the paths the decks use. They can move to `(owner, ExileId)` the same way as the O-Ring list when a card needs it | follow-up |
| foretell, plot, warp, cascade-to-exile | `bulk_white.rs`, `t45_prosper.rs` (`mark_exile_cast`), `legends::cascade` | foretold cards wait in the hand (`t45_prosper`), warp is a turn-stamped flag; they don't hold exile identity | follow-up |
| face down (Necropotence in practice mode, foretell) | `necro_wait`, practice | no face-down object yet: no AI decision reads it | follow-up |

## Stages

- **A, results identical:** the zone type with ids; every site compiles through the read API; the copy removal in
  `cast.rs`, the reanimation destination in `human/human.rs` and the `remove_card(&mut ...exile, c)` sites use the
  zone's own methods.
- **B, call sites, results identical:** `oring_exile` records `until` and the `ExileId` in the `Oring` list; Karn
  records `silver` on the object. Reads unchanged.
- **C, behaviour (tests):** `oring_return` returns the object by id; silver counters read from the objects (a card
  that left exile and came back has none; the -1 takes the silver object, not the oldest copy); impulse draw's
  "this turn" ends at the end of that turn.

## Speed

`examples/clone_cost` (2.91-2.94 µs at the base) and `examples/repl_cost`, `mana_cost` before and after each stage.
