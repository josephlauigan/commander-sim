# Engine needs: batch `godo`

## Godo, Bandit Warlord
Needs: no cap on additional combat phases (the combat loop stops at four combats a turn); Godo + Helm of the Host makes one more combat per hasty token, so the loop needs to run until the table is dead.
Where it would hook: `engine/combat.rs` `combat` (`g.player(p).extra_combats > 0 && ncomb < 4`, line ~1127); a bound that stops a true infinite loop (all opponents dead, or a few dozen combats) should replace the 4.
Card side done: ETB fetches an Equipment onto the battlefield (Helm, or Hammer first with a copier), first attack each turn untaps Godo and Samurai and adds one combat (`extra_combats += 1`), `impls/t6_godo.rs`. Helm's token copy (t3::helm) triggers Godo's ETB and attack as the card does.
Test to add once the engine has it: Godo + Helm with six opposing creatures-free players at 40 life: all opponents die in the one turn.

## Sting, the Glinting Dagger
Needs: (1) a beginning-of-combat event for every player's combat, fired before attackers are declared and also when nobody attacks (`cardcode::combat_start` runs only for the attacking player, after attackers are chosen, and only if something attacks); (2) a conditional keyword depending on the blocker/attacker pairing ("first strike while blocking or blocked by a Goblin or Orc"): `GrantKwFn` has no combat context.
Where it would hook: `engine/combat.rs` ~line 1004 (the `combat_start` call) and the damage step near line 628 (`first strike` checks, `kw(g, m, "first strike")`).
Card side done: +1/+1, haste, equip {2} (compiled); untap of the equipped creature through the existing `combat_start` slot for its controller's combat (`impls/t6_godo.rs`).
Test to add once the engine has it: an opponent's combat untaps the equipped tapped creature (it can block); a Sting-equipped creature blocked by a Goblin token deals first-strike damage.

## Tezzeret, Cruel Captain
Needs: a new emblem kind for the walker framework: "at the beginning of combat on your turn, put three +1/+1 counters on target artifact you control; if it is not a creature it becomes a 0/0 Robot artifact creature" (the -7). `rules::give_emblem(g, p, key)` only knows its fixed keys, and the beginning-of-combat event (`combat_start`) is a permanent-slot hook, with no player-level one.
Where it would hook: `impls/rules.rs` `give_emblem` and the code that reads the emblem keys (grep `give_emblem`), plus a player-level `CombatStart` run in `engine/combat.rs` ~line 1004.
Card side done: starts at 4 loyalty; +1 loyalty for each artifact entering under its controller; 0 (untap, +1/+1 counter on an artifact creature) and -3 (artifact with mana value 1 or less to hand) as `common::walker` abilities (`impls/t6_godo.rs`); the -7 is left out.
Test to add once the engine has it: at 7+ loyalty -7 gives the emblem, and at the next beginning of combat an artifact gets three +1/+1 counters (a non-creature becomes a 0/0 Robot creature with three counters).

## Party Thrasher
Needs: "noncreature spells you cast from exile have convoke": tapping creatures to pay {1} or a colour of the creature for a spell cast from exile (the impulse list is `Player::impulse`; there is no convoke at cast time for cards in it).
Where it would hook: `engine/cast.rs` cost payment (convoke for spells in `Player::impulse`, with the creature's colours) and `engine/mana.rs` `can_pay`/`total_mana` so the AI counts untapped creatures when it checks a spell in `impulse`.
Card side done: the trigger (discard a spare card, exile the top two, keep the better in the hand/impulse list for this turn, the other stays exiled) in `impls/t6_godo.rs`.
Test to add once the engine has it: a 3-mana noncreature spell in `impulse` with two untapped creatures and one land is castable, taps both creatures; a creature spell from exile gets no convoke.

## Vexing Bauble (DONE: t6-e-castctr, rust/agents/report-t6-e-castctr.md)
Needs: a counter-on-cast hook: a permanent that counters a spell as it is cast when a condition on the cast holds ("if no mana was spent to cast it"), with the spell going to its owner's graveyard, as Jin-Gitaxias's counter does in `engine/cast.rs` (`jin_kind`, before the stack item is pushed). The mana actually spent is `cur_cast` plus `ctx.paid_otherwise`.
Where it would hook: `engine/cast.rs` `cast_card` right after `on_cast` (the Jin-Gitaxias counter), reading a new `Event::CastCounter` slot.
Card side done: the draw ability (compiled) and a stand-in lock in `impls/t6_godo.rs` using the existing `can_cast` slot (`engine/hooks.rs::allowed`): nobody casts a spell whose cost is {0}. Differences: the countered card stays in hand instead of the graveyard, and free alternative costs (Daze-style, "without paying") are not caught.
Test to add once the engine has it: casting Mox Amber with the Bauble out puts it in the graveyard; a spell cast for {0} by an alternative cost is countered; Sol Ring resolves.

## Hexing Squelcher
Needs: ward with a non-mana cost (pay life, here 2; Sauron's sacrifice-a-legend ward is a special case), and ward granted by a static ability ("other creatures you control have ward"): `CardData::ward` is a mana number, and `cardcode::aura_ward` only reads Auras.
Where it would hook: `engine/removal.rs` (ward check ~line 179, and the targeting filter ~line 100, which both read `d.ward` and `aura_ward`): a ward cost enum (mana n / life n / sacrifice) and a static-ward slot on `CardImpl`.
Card side done: can't be countered (its `unc` tag), and spells you control can't be countered (the `uncounterable` slot) in `impls/t6_godo.rs`.
Test to add once the engine has it: a removal spell targeting a creature you control with the Squelcher out costs its caster 2 life or is countered; the ward does not apply to your own spells.

## Blood Moon (DONE: t6-e-legend, engine::mana::moon_land)
Needs: "nonbasic lands are Mountains" as a layer effect: such lands lose all abilities and other land types (no Ancient Tomb damage, Urza's Saga without chapters and its tutor ability, no manland or other activated abilities, no "enters tapped" read later) and gain the Mountain type. The engine's `blood_moon` slot only changes the colour a nonbasic land taps for (`engine/mana.rs` `land_cols`).
Where it would hook: `engine/mana.rs` (`Tag::Tomb`/`Tag::Pain` damage, lines ~117, 264, 382), `impls/lands.rs` land options and land abilities (`land_options`, `land_etb`), reading `cardcode::blood_moon(g)`.
Card side done: the existing `blood_moon` slot (colour R), `land_mana` slot cutting two-mana lands to one, and an AI cast priority (`impls/t6_godo.rs`).
Test to add once the engine has it: with Blood Moon out, tapping Ancient Tomb deals no damage; Urza's Saga's tutor ability is gone; a manland cannot animate.

## Final Fortune, Last Chance (DONE: t6-e-turns, turn::add_extra_turn / run_extra_turns)
Needs: extra turns (a queue of extra turns on the game that `turn.rs` plays right after the current one, with the same player active), and a delayed "at the beginning of that turn's end step, you lose the game" effect tied to that extra turn. The deck uses them for a second combat phase / full untap on the combo turn (spec line 7), so the AI also needs a rule to cast one only when the extra turn wins.
Where it would hook: `engine/turn.rs` (the seat loop around `step_start`, ~line 1140 where `skip_turns` is read) and the end-step code next to `Event::EndStep`; a `Player::extra_turns` counter plus a per-turn "loses at end" flag.
Card side done: both cards' `resolve` slot goes to the graveyard (priority 0, so the AI never casts either: it would only lose the card) in `impls/t6_godo.rs`; to finish, the resolve should add one extra turn and mark it as the losing turn, and the priority should say when the extra turn wins.
Test to add once the engine has it: resolving Final Fortune gives the caster a second consecutive turn, and the caster loses at that turn's end step (not before).
