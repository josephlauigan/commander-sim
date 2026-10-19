# Engine needs: batch stax

**t6-e-locks DONE:** the player cast locks (Silence, Chant, Mandate, Abeyance, Calamity, Bilbo) and split second (Angel's Grace, Sudden Substitution, Legolas) are engine features now (`engine/cast.rs`); see report-t6-e-locks.md. Still open here: Chant kicker + no-attack, Mandate "only during combat", Grace "can't lose", the Bilbo gift choice, and the non-counter response / control-change parts.

## Silence, Orim's Chant, Mandate of Peace
Needs: the player-level "can't cast spells this turn" check that `engine-needs-counters.md` (Abeyance, Bilbo's Gambit) and `engine-needs-redirects.md` (Calamity's Wake) already ask for: `castable` refusing every spell (creatures and flash too) from a flagged player, and the AI's option lists reading it.
Where it would hook: `engine/cast.rs::castable` (next to the `hope_lock` / `g.silence` checks), reading `impls::t6_stax::cast_lock(g, p)`.
Card side done: `impls/t6_stax.rs` sets the turn flag `cast_lock` on each affected player (Silence: every opponent; Chant: the target; Mandate: every opponent) and, as a stand-in the engine reads today, `Player::hope_lock` (no noncreature spells this turn). The AI never casts them (priority 0): when they win a turn is the deck plan's call.
Test to add once the engine has it: after Silence an opponent holding a flash creature cannot cast it that turn but can the next turn.

## Orim's Chant
Needs: (1) the optional additional cost "kicker {W}" chosen when casting (a general kicker prompt: `ctx.kicks` exists for Comet Storm's multikicker but `cast_card` never sets it for a plain kicker), and (2) "creatures can't attack this turn" in the attack declaration (`attack_cap`-style, but from a spell, not a permanent); plus the cast lock above.
Where it would hook: `engine/cast.rs::cast_card` (cost payment) and `engine/combat.rs` / `ai/brain.rs` attack declaration, reading `impls::t6_stax::attack_lock(g)`.
Card side done: the target (face or the opponent with most cards) can't cast spells (stand-in above); `ctx.kicks > 0` sets the turn flag `stax_no_attack` (`attack_lock`).
Test to add once the engine has it: a kicked Chant costs {W}{W}, and no creature can be declared as an attacker that turn.

## Mandate of Peace
Needs: "end the combat phase" (remove all attackers and blockers from combat, skip the rest of combat including extra combats' remaining steps, exile the stack), "cast only during combat" for a card in hand, and the cast lock above.
Where it would hook: `engine/combat.rs` (the combat loop, `g.in_combat`) and `engine/cast.rs::castable` (a per-card cast-timing check; `impls::t6_stax::in_combat_now` answers it).
Card side done: opponents locked (stand-in above) and `Game::fog = this turn` (no combat damage) as the combat ending.
Test to add once the engine has it: cast after attackers are declared, the attackers are out of combat, no damage is dealt, and the opponents cannot cast creature spells either.

## Angel's Grace
Needs: (1) split second (no responses, no spells or non-mana abilities while it is on the stack: the same item `engine-needs-counters.md` lists for Sudden Substitution), (2) a "can't lose / opponents can't win this turn" player flag read by `check_state` (the Herald of Eternal Dawn check, `t0_giada::cant_lose`, is battlefield-only), and (3) the damage floor: damage that would put you below 1 leaves you at 1 this turn.
Where it would hook: `engine/stack.rs` `take_priority` / `ai_respond`; `engine/life.rs::check_state` and `lose_life` (damage branch), reading `impls::t6_stax::grace_active` / `grace_floor`.
Card side done: resolving sets the turn flag; `grace_active` and `grace_floor` answer both rules. The AI never casts it (priority 0).
Test to add once the engine has it: at 3 life, Angel's Grace in response to lethal damage leaves the player at 1 and alive; no response can be cast while it is on the stack.

## Blind Obedience
DONE by t6-e-tapped: `enters_tapped` slot (hooks.rs), `engine/hooks.rs::enters_tapped`, applied in `zones::enter` and `make_tokens`; Treasures via `Player.tapped_treasures`. Food/Clue still untapped.
Needs: "enters tapped" as a replacement effect for a class of permanents (an opponent's artifacts and creatures), applied before the permanent is created: the same item `engine-needs-copies.md` / Thalia, Heretic Cathar's approximation share. Reuse that general hook, not a card-specific one.
Where it would hook: `engine/zones.rs::enter` (before `new_perm`) and `make_tokens` (before the tokens' enter triggers), asking a new `enters_tapped(g, src, owner, perm_or_card)` slot on battlefield cards.
Card side done: extort (pay {W}, else {B}, with spare mana as Crypt Ghast's does; each opponent loses 1, the controller gains the total) and the tap right after entering through `etb` (cards and artifacts) and `tokens_enter` (creature tokens), `impls/t6_stax.rs`; the same stand-in as Thalia, Heretic Cathar (`common.rs::thalia_hc`).
Test to add once the engine has it: an opposing creature with "when this enters, untap it" ends untapped never (it enters tapped, then the trigger untaps it); a Soul Warden-style enters trigger sees `tapped` already true.

## Damping Sphere (DONE: t6-e-legend, Event::LandColorless)
Needs: a colour override for a land's mana from card code ("produces {C} instead of any other type"): `land_cols` in `engine/mana.rs` only has the Blood Moon, Lantern and Dryad cases, and `land_mana` slots change only the amount. The same hook would serve Blood Moon's "nonbasic lands are Mountains" (`engine-needs-godo.md`).
Where it would hook: `engine/mana.rs::land_cols` (next to `moon`), a `land_colors` slot on `CardImpl` returning an override for a land.
Card side done: `impls/t6_stax.rs`: the amount of any land that taps for two or more is cut to one (`land_mana` slot; only the earliest Sphere acts), and the spell tax of {1} per other spell the caster cast this turn (`cost` slot, exact).
Test to add once the engine has it: with the Sphere out, Izzet Boilerworks taps for one {C} only, never {U} or {R}.

## Pithing Needle
Needs: a name-based "activated abilities can't be activated" lock that the activation paths ask, separate from Disruptor Flute's: `engine/values.rs::stopped` is the Flute's check and `engine/mana.rs::cost_of` also uses it for the {3} spell tax, so the Needle cannot just push into `g.flutes`. A general `ability_locked_by_name(g, name)` (the Flute's ability half plus Needle) called from `combat::blocked`, `dsl/mod.rs` (the two `stopped` calls), `ai/decks.rs`, `combos.rs`, `common.rs` sac-outlet checks, with mana abilities exempt. `rules.rs::ability_locked` takes a source permanent and hard-codes card names (Collector Ouphe, Cursed Totem ...); it would also be the place.
Where it would hook: the places above, reading `impls::t6_stax::needle_stops(g, name)`.
Card side done: `impls/t6_stax.rs`: as it enters the Needle names the card `ai::decks::flute_pick` would name for the Flute (stored as `DataKey::Imprint` = the named card, read with `needle_name`), and `needle_stops` answers the rule.
Test to add once the engine has it: with the Needle naming an opposing planeswalker, its loyalty abilities are not offered, and its mana abilities (and other cards' abilities) still are.

## Chalice of the Void, Void Mirror (DONE: t6-e-castctr, rust/agents/report-t6-e-castctr.md)
Needs: the counter-on-cast hook already asked for by Vexing Bauble (`engine-needs-godo.md`: `Event::CastCounter` in `engine/cast.rs::cast_card` right after `on_cast`, next to Jin-Gitaxias's counter): a battlefield card counters a spell as it is cast when a condition on the cast holds (Chalice: mana value equals the charge counters; Void Mirror: no coloured mana was spent, read from the mana actually paid, `cur_cast` plus `ctx.paid_otherwise`). Chalice also needs an X choice for a card with no X tag ({X}{X}: the data has no mana cost; `ai/brain.rs` only chooses X for `Tokx` / `Xtutor` cards and spends all spare mana).
Where it would hook: `engine/cast.rs::cast_card` (the `jin_kind` counter), a new `Event::CastCounter` slot on `CardImpl`; X: `engine/mana.rs::cost_of` and `ai/brain.rs` (~line 436).
Card side done: both are `can_cast` locks like Vexing Bauble's (`impls/t6_stax.rs`): nobody casts a nonland spell whose mana value equals Chalice's counters (X fixed at 1, `self_cost` makes it cost {2}, the counters are `DataKey::Charge`); Void Mirror refuses spells with no coloured mana in the cost when the caster has no untapped coloured source (see the card). Differences: the spell is not cast (no cast triggers, the card stays in hand instead of going to the graveyard) and free alternative costs are not caught.
Test to add once the engine has it: with Chalice at 1, Sol Ring is cast and countered (graveyard, a Rhystic Study-style cast trigger happens); X=2 counters mana value 2 but not 1; a Mox Diamond-style free cast with Void Mirror out is countered.
