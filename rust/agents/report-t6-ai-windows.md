# Report t6-ai-windows (Stage 5, answer windows for lines played out by normal actions)

## Mechanism
`combos::normal_play_window(g, p, acting)` (end of `impls/combos.rs`). For a Tier 6 player (`is_t6`) not already inside
`attempt` (`t6_going != p`), an action with permanent `acting` that is a piece of one of the player's modeled lines
(`lines_of`, quiet lines included) whose every piece group is already on the battlefield opens the same answer window
as a combo step: `interrupted` (each opponent's instant-speed removal/bounce on the line's pieces, Tidebinder-type
ability counters). If it stops the line the action does not happen; the stop is recorded (`combo_stop`, stat
`combo_stopped`) so the Tier 6 report shows "stopped by <card>". Hooks (two lines of shared code):
- `engine/stack.rs` `ability_window`: any activated ability with a source permanent, so equip (via `equip_to`),
  Oboro's untap, Kinnan/Basalt activations and others go through it (general, not Godo-only).
- `impls/t3.rs` `helm`: Helm of the Host's beginning-of-combat trigger, before its trigger window.
Not done: Godo's own ETB/Hammer's attach trigger have no separate window (Godo's cast is a piece in `counter_verdict`
since b92bd42; the Helm trigger at the next combat is the first window after Hammer attaches). Kinnan + Basalt is
mostly played through `attempt` already (own windows); the generic hook covers its activations only when they go
through `ability_window`. Gate: Tier 6 players only; t3 lock identical (diff empty).

## Before / after (300 games per profile, seed 1, same seeds)
Godo vs Tier 6 (`--deck godo --pool t6`):

| | before cons / loose | after cons / loose |
|---|---|---|
| win % | 51.7 / 59.0 | 49.0 / 54.7 |
| wins by | damage 50 / 46, combo 50 / 54 | damage 49 / 41, combo 51 / 59 |
| went for a combo (stopped) | 108 (29%) / 124 (23%) | 106 (36%) / 128 (31%) |
| stopped by card | FoW 4, Swan Song 4, Mindbreak Trap 4, none-named 4 / FoW 4, Bilbo's Gambit 3, Path 2 | Chain of Vapor 7, Mindbreak Trap 5, FoW 4, Swan Song 3 / Chain of Vapor 4, Snap 3, Pact 3, Swan Song 3 |
| win round avg | 6.8 / 7.1 | see output files |

Tier 6 pool (`--calibrate within --pool t6`), cons / loose win % and average rounds:

| deck | before | after |
|---|---|---|
| Dack | 32.7 / 27.0 | 30.1 / 27.0 |
| Kinnan | 35.1 / 35.6 | 34.6 / 39.3 |
| Kraum/Tymna | 24.5 / 27.9 | 27.0 / 25.5 |
| Rograkh/Silas | 13.0 / 19.5 | 13.5 / 17.0 |
| Rograkh/Thrasios | 18.9 / 20.4 | 18.9 / 18.4 |
| Sisay | 26.6 / 20.2 | 26.6 / 23.6 |
| avg rounds (timeouts) | 10.3 (3.3%) / 10.2 (3.7%) | 10.3 (3.7%) / 10.3 (4.0%) |

Godo moves about -3 to -4 points, which is inside the noise (95% CI +-5.5). The new answers are real (Chain of Vapor
and Snap now appear as the stoppers of the equip and Helm-trigger lines) but the pool decks hold few instants, so
the effect is small; Godo is still ~49-55%, well above the 15-35% target. The remaining cause is not missing windows:
about half the wins are damage from ordinary play and Godo's many fetch/copy lines. Stage 6 numbers (Godo's list or
the opponents' interaction density) are the lever. No "internal simulator error" in any run. Tests:
`tests/ai_t6_windows.rs` (4: Swords stops the Godo + Helm equip, no answer wins through, no mana no answer, non-Tier-6
gets no window). Full `cargo test --release` passes, build with no warnings, t3 lock identical.
Outputs: /tmp/claude-1000/t6ai-windows/{base,new}-{godo,t6}.txt.
