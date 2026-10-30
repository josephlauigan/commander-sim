# Phase 3, PR 1: one permanent arena (design)

Brief: `rust/agents/phase3-arena.md`. Handoff: `audit/phase3/arena-handoff.md`.

## Today
- `Game::perms: Vec<Perm>` (nonland permanents and tokens, `PermId` = index) and `Game::lands: Vec<Land>`
  (`LandId` = index; `Land { id, cd: CardId, owner, tapped, data: PermData, on_bf }`). Two id spaces, both never
  reused.
- `Player::lands: Vec<LandId>` and `Player::perms: Vec<PermId>`: the battlefield, in entry order.
- Only `Game::add_land` makes a `Land` (on the battlefield at once, `on_bf = true`); `turn::remove_land` takes it off
  (`on_bf = false`, kept in the arena).
- A manland or Nissa land "animates" by leaving `player.lands` and getting a stand-in token `Perm` (no card, the
  `land` subtype, `DataKey::Land = Val::Land(l)`); `g.animated` reverts it (`lands.rs` `animate`, `partials.rs`).

## Design
**A land is a `Perm`.** `add_land` calls `new_perm(p, Some(cd), ..)` (name, P/T and keyword tags from the card, as
for any permanent) and sets `tapped`, `on_bf = true`, `sick = false`. `Land.owner` is the controller, so it maps to
`Perm.owner`, and `Perm.orig` (the owner) is the same player, as lands never change control today. `tapped`, `data`,
`on_bf` and `id` keep their names. The one field that changes type is `cd`: `CardId` on a `Land`, `Option<CardId>`
on a `Perm` (None for a token). `Perm::card()` returns the card (`cd.expect`) for code that knows it holds a card;
land tokens don't exist yet.

**One id type.** `LandId` becomes `type LandId = PermId` during the move (Stage A), then goes (Stage B). Ids stay
"index in `Game::perms`, never reused". Lands and nonland permanents now share the numbering, so a nonland
permanent's id is no longer its rank among nonland permanents, but ids are still increasing in creation order, so
every comparison or sort between two nonland ids (or between two land ids) gives the same answer as before. No code
compares a land id with a permanent id (they were different types), hashes ids into a `HashMap`, or hard-codes an id
number (checked: grep for `HashMap<PermId`, `PermId(<digit>)`).

**Two lists, unchanged.** `player.lands` holds the land `PermId`s and `player.perms` the nonland ones, pushed and
removed exactly where they are now, so every iteration order is the same. Stage C decides whether to merge them; the
plan is to keep both lists (a land never sits in `player.perms`) and make the ability language walk
`player.lands` after `player.perms`, so the new land sources come last and cannot reorder anything that ran before.

**Arena-wide loops.** Four loops walk all of `Game::perms`: `turn::step_start` (clears `eot_pt` / `eot_kw`),
`t6_scion::copy_revert` and `alela::mirror_revert` (look for a nonland data key), `t45_edgar::groom_as_enters`
(looks for a permanent with the Groom's card). None can match a land, so lands in the arena change none of them.

**Special paths onto it (Stage C).** One path per effect, as in Phase 2:
| Land effect | Today | After |
|---|---|---|
| Activated abilities (Boseiju, Castle Ardenvale, Treasure Vault ...) | never offered | `dsl::ability_options` walks `player.lands` |
| Statics and triggers in the ability language | never run on lands | `dsl::statics` / `dsl::fire` walk `player.lands` |
| Temples scry, gain lands gain | `land_etb_fx` (`zones::lands_entered`) | kept where no compiled ability covers it, else the compiled one; never both |
| Surveil lands (4) | nothing | surveil 1 as they enter |
| Fetches | `f` tag, `turn::crack_fetch` | kept (the compiled ability must not also be offered) |
| Reliquary Tower | `nomax` tag | kept |
| Card-code `land_options`, `t6_lands.rs` | card code | kept, unless the compiled ability now offers the same thing |
| Manlands / Nissa | stand-in token `Perm` | kept for now (one path); the land itself animating is a later PR |

## Risks
- **Iteration order**: guarded by keeping both player lists and every push / retain where it is.
- **`on_bf` semantics**: a land `Perm` is `on_bf` from `add_land` to `remove_land`, as a `Land` was. `new_perm`
  starts nonland permanents off the battlefield; `add_land` sets it at once.
- **Mana code**: `mana::Source::Land(l)` and `Source::Perm(m)` stay separate variants, so mana units keep their kind.
- **Tokens that are lands**: none today (a land needs a card). `Perm::card()` would panic on one; Stage C adds none.
- **Dryad Arbor, animated lands**: Dryad Arbor plays as a land today (a land with the creature type isn't seen
  as a creature); unchanged. Animated lands keep their stand-in token.
- **`enter` paths**: lands don't go through `zones::enter` (no `born`, no ETB hooks for nonland permanents); unchanged.
- **The AI's land valuation**: reads `player.lands` and the card; unchanged in A/B. In C, newly offered land
  abilities are scored like any other ability option.
- **Speed**: a `Perm` is bigger than a `Land` (more fields, two more empty `Vec`s), so copying a game for the
  look-ahead copies more bytes. No sims this PR (JD), so no timing; `examples/clone_cost.rs` gives the copy cost.
- **No byte-identity runs** (JD, 2026-10-29: no sims until Phase 4): "no behaviour change" in A and B is checked by
  the test suite only.
