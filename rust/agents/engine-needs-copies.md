# Engine needs: batch copies

## Clever Impersonator (and the other clones: Phantasmal Image, Flesh Duplicate, Mockingbird, Copy Artifact, Mirrormade, Sculpting Steel)
Needs: an "enters as a copy of X" replacement effect with a per-card filter and exceptions (extra types, extra keywords/abilities on the copy). Today only `Tag::Clone` (Phyrexian Metamorph: creature or artifact) is handled in `enter`, and a 0/0 creature dies in `creature_entered` before its enters hook can swap it.
Where it would hook: `engine/zones.rs::enter` (the `Tag::Clone` block before `new_perm`), `creature_entered` (the `etgh <= 0` check).
Card side done: `impls/t6_copies.rs`: the enters hook leaves and re-enters as the best legal target (clone card in `phys`); the three printed-0/0 cards carry 1/1 in `data/card_overrides.toml` so they are still there when the hook runs, and die if nothing can be copied. Side effect: the stand-in's own "creature entered" triggers fire once.
Test to add once the engine has it: a clone enters once (no stand-in enter trigger, e.g. Soul Warden gains 1 life, not 2).

## Mockingbird, Copy Artifact, Molten Duplication, Phantasmal Image
Needs: types added to a permanent or token copy (Bird, enchantment, artifact, Illusion).
Where it would hook: `state.rs::Perm` (a list of added types next to `ttypes`) and `engine/values.rs::subtypes`/`has_type`, `card_is`.
Card side done: Mockingbird's copy has flying (`Perm::fly`); the copies are otherwise exact.
Test to add once the engine has it: Mockingbird copying a Wizard is a Bird Wizard; Copy Artifact's copy is an enchantment (Disenchant-style or enchantment counts see it).

## Phantasmal Image
Needs: a "becomes the target of a spell or ability" event (a trigger on the target, for any source).
Where it would hook: wherever targets are chosen (`engine/removal.rs::legal_targets` callers, `cast.rs::spell_targets`, ability targeting).
Card side done: the copy of the best creature (the exception is missing).
Test to add once the engine has it: targeting the Image with a removal spell or an ability sacrifices it.

## Flesh Duplicate
Needs: abilities granted to a copy (vanishing 3: three time counters, remove one each upkeep, sacrifice when the last goes), unless the copied creature already has vanishing.
Where it would hook: the same copy-exceptions mechanism as above plus an upkeep hook that is not tied to the card (`engine/turn.rs` upkeep, `Perm::data` counters).
Card side done: the copy of the best creature (without the time limit).
Test to add once the engine has it: the copy gets 3 time counters and is sacrificed at the third upkeep.

## Imposter Mech
Needs: vehicles (crew N: tap creatures with total power N to make it a creature until end of turn) and a copy that becomes a Vehicle artifact with crew 3 and loses its other card types.
Where it would hook: `Event::Crew` / `DataKey::Crewed` exist but "crew" is in `data/audit.toml` unsupported_keywords; copy exceptions as above.
Card side done: none (prio 0: the AI does not cast it).
Test to add once the engine has it: copy an opposing creature; it is a non-creature artifact until crewed with power 3, then it attacks as the copied creature's size.

## Peter Parker's Camera
Needs: copying a triggered or activated ability on the stack (and choosing new targets), paid for with {2}, {T} and a film counter.
Where it would hook: `engine/stack.rs` (`queue_triggers` / `flush_triggers`, `ability_window`): a window where a permanent may copy a `Trigger` or ability item. `Event::TriggerCopies` (Panharmonicon) takes `&Game` so it cannot pay or tap; reusing it would need a mutable variant.
Card side done: enters with three film counters (`DataKey::Counters`).
Test to add once the engine has it: Godo's enters trigger with the Camera out fetches twice (the copy resolves first), the Camera has two counters left and is tapped.

## Volatile Stormdrake
Needs: "hexproof from activated and triggered abilities" (hexproof from a source kind; spells can still target it).
Where it would hook: `engine/values.rs::untargetable` (reads the `hexproof` keyword as full hexproof) and its callers that know whether the source is a spell or an ability.
Card side done: the exchange of control (reusing `marchesa::steal` both ways, so the opponent really gets the Drake), the four energy and paying energy equal to the mana value or sacrificing the creature.
Test to add once the engine has it: a removal spell can target the Drake, an activated ability cannot.

## Lead note (merge, 2026-10-17): undo the 1/1 stand-in stats
Clever Impersonator, Phantasmal Image and Flesh Duplicate carry 1/1 in data/card_overrides.toml so they survive
long enough for their enter hook to copy. Once the engine has an "as this enters, it becomes a copy" replacement,
restore their printed 0/0 and drop the stand-in (and Clever Impersonator's stand-in firing its own enter triggers).

## Engine pass t6-e-copies (done)
The `enter_copy` slot + `CopyExc` mechanism (see report-t6-e-copies.md) covers the clones' replacement effect, the 0/0 restore, Mockingbird (Bird, flying), Copy Artifact (enchantment), Molten Duplication (artifact). Still open: Phantasmal Image's becomes-target sacrifice, Flesh Duplicate's vanishing 3, Imposter Mech, Peter Parker's Camera, Volatile Stormdrake.
