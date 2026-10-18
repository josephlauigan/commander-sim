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
