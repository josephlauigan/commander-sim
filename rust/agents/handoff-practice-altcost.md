# Handoff: practice-altcost

## Baseline
Sims (sheoldred t3, cloud t1, kefka t3, 200 games, adaptive loose, jobs 3) saved by scratchpad/sims.sh as
`before-<deck>.txt`; binary copy cs-before. Rerun `sims.sh <bin> after` at the end and diff (ignore progress/elapsed lines).

## Plan
1. New file human/alt.rs: generic fallback. Reads hand_options / gy_options / land_options hooks for a person's card,
   filters `Opt`s with an act (not a plain Cast of the same card), applies the sorcery-timing check, asks the person
   (choices::choose) and runs `ai::act::perform`. `engine::mana::can_pay/pay` already use the person's pool, so the
   hooks' affordability checks work with the mana the person tapped first.
2. Wiring: `cards::hand_override` true for cards with a hand_options hook; `cast_from_hand` adds the hook options
   to the ways (normal cast first). `gy_override`/`apply` zone gy: cards with gy_options; view.rs seam
   graveyard_playable lists them ("alt"). `use_land` adds land hook options; seam::land_has_ability (was a stub: false).
3. Hooks that bail for a person (`human(g,p)` guard, e.g. Phyrexian Tower) and Chalice (kicks) get real person paths.
4. Tests tests/play_altcost.rs.

## Done
(nothing yet)
