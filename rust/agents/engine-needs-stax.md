# Engine needs: batch stax

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
