# Report: Ur-Dragon's five approximate cards (2026-11-04)

The agent's harness refused report files, so the coordinator wrote this from the agent's final message, after
reviewing the diff of commit 56fdaa0e.

## Per card
- **Cursed Mirror: Full.**
  - Before: it copied only the stats and keywords, with no abilities and no enters trigger.
  - Now it's an `enter_copy` replacement (`t0_stella::mirror_copy`): the permanent is the copied creature's card with
    haste, so that creature's abilities and enters trigger work, and other creatures' enters triggers (Terror of the
    Peaks) see a creature.
  - The Mirror card stays in `phys`, so it dies as the Mirror. `CopyExc::revert_to` and `t0_stella::mirror_revert`
    make it the artifact again at the start of the next turn.
  - Sculpting Steel copying the Mirror, or a token copy of it, copies a creature too (a chained `enter_copy` in
    `zones::enter`).
  - The AI copies when the creature has 3 power or a copy value of 5 or more. That threshold was raised to 5 so a
    Goblin Engineer Mirror stays a mana rock, which `cards_t6_godo` expects. A person picks or declines.
  - The choice comes from `t6_copies::creature_copies`, which excludes tokens, commanders and your own legendaries.
    That avoids the legend rule; the real card could also copy tokens and commanders.
- **Astral Dragon: stays Approximate.**
  - Before: its tokens became creatures only after entering, so Dragon Tempest missed them.
  - `chars::is_creature` now honours `CopyExc::add_types` CREATURE, so the tokens enter as 3/3 flying Dragon
    creatures. Only Astral Dragon sets that type through `add_types`.
  - The AI's target is now `t6_copies::copy_value`, and an opponent's legendary permanent is allowed. A person picks
    the target.
  - Still missing: copying a land, because a token land would need to go in the player's land list. In this deck a
    nonland permanent is always the better target.
- **Nicol Bolas, the Ravager: Full.**
  - The flip and all four Arisen abilities already worked; −12 was tested at 14 loyalty.
  - Fixed: when the Arisen leaves, the card goes to the graveyard as the Ravager (its front face), via `phys`. That
    includes the 0-loyalty path in `dsl::use_loyalty`.
  - A token or copy of Bolas can no longer transform.
- **The Ur-Dragon: Full.**
  - The draw per attacking Dragon and the eminence discount were already right.
  - Land cards can now be put onto the battlefield too, with their enters effects, landfall and fetch cracks
    (`ur_pick`).
  - The AI takes the nonland permanent that saves the most mana; if that saves under 3, it puts in a land instead.
- **Wrenn and Six: Full.**
  - The −7 now gives a `wrenn6` emblem. `t12_windgrace::emblem_retrace_options` lets the AI recast its best instant or
    sorcery from the graveyard by discarding a land. A person gets a `GyKind::Retrace` graveyard cast.
  - The +1 returns the best land, not the first one.
  - The −1 never aims at a planeswalker. That's AI policy.

## Tests
- 20 new tests in `tests/cards_urdragon_approx.rs`.
- Three older tests were updated for the new Mirror behaviour, in `cards_t0_stella` and the Sculpting Steel test in
  `cards_t6_copies`. The permanent now carries the copied creature's name, so they find it by `phys`, and they call
  `mirror_revert`.

## Shared files touched (additive)
- `state.rs` (`CopyExc::revert_to`)
- `engine/zones.rs`, `engine/chars.rs`, `engine/turn.rs`
- `dsl/mod.rs`, `ai/brain.rs`
- `human/legal.rs`, `human/human.rs`
- `crates/practice/src/view.rs` (a `GyKind::Retrace` arm)
- `impls/t6_copies.rs`, `impls/t12_windgrace.rs`, `impls/legends.rs`, `impls/card_models.rs`

Scion, Garth and `data/plans/*` weren't touched.

## Note
The Mirror change also affects the Godo seat's Cursed Mirror combos. Their tests still pass.
