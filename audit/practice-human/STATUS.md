# Practice mode: the person's choices — status and how to continue

**Status: paused on 19 Oct 2026 (JD's call).** The engine-wide fixes are done. The per-card work and two engine items
are pinned until JD picks the work up again. This file is for the session that does it: read it, then
[`REPORT.md`](REPORT.md) (the audit) and the `findings-g*.md` tables it refers to.

## What this is about
JD plays his own decks in practice mode (the browser page and the iPad app, both on `rust/crates/practice`). The
engine was built for AI-vs-AI simulations, so in many places the card code made the person's choices for them, or a
card did nothing when a person controlled it. An audit on 19 Oct (`REPORT.md`, about 165 rows in `findings-g1.md` to
`findings-g5.md`) found four causes. Each fix below gives the **person** a choice and leaves the **AI exactly as it
was**, so simulation results don't move.

## Done (all with tests; each proven not to change the AI)
| PR | Fix | Where |
|---|---|---|
| #102 | Sephiroth, Fabled SOLDIER asks what to sacrifice; transformed face shows; identical triggers aren't ordered | `impls/sheoldred.rs`, `human/human.rs::order_triggers` |
| #104 | Beginning-of-combat triggers fire before the person declares attackers; Ardenn asks what to attach | `engine/combat.rs`, `impls/bulk_white.rs` |
| #107 | The person's creatures can attack planeswalkers | `human/combat.rs`, `Game::human_walker_atk` |
| #108 | The person chooses when an opponent's card affects them (edicts, Fact or Fiction piles both ways, Torment of Hailfire, Esper Sentinel / Norn's Annex payments, Sauron's ward) | `choices::sacrifice_permanent`, the cards' functions |
| #109 | "You may" triggers ask the person, about 28 hand-written cards | `engine::stack::may_window` / `may_window_ask` / `may_window_card` |
| #110 | "You may" effects of DSL cards ask the person, 28 effects | cardgen `dsl_parse.rs` sets `optional` / `if_you_do` / `cont`; `dsl/mod.rs` asks |
| #113 | Alternative costs and graveyard / land abilities, 22 cards (13 with the person's own choices, ~9 with the AI's choices inside) | `human/alt.rs` (reads `hand/gy/land_options`), `forced_*` tables |
| #114 | Targets, modes, X, kicks and additional costs for 21 resolve-coded spells | `human/specs.rs` (a per-card spec the cast flow reads) |
#113 and #114 are stacked: merge #113 first (#114 carries the fix for their overlap: the alt fallback skips cards that
have a spec). If either is still open when you start, check `gh pr list`.

## Pinned (not started)
1. **Per-card fixes, deck by deck** (the biggest part, about 4-6 hours at 6 threads). The "chosen" rows in the
   findings that the work above didn't cover: card code that picks a target, creature, card or player for the person
   with AI logic (no `choices::` call). Examples: Godo, Bandit Warlord's Equipment; Kor Outfitter, Armory Automaton,
   Brass Squire, Cid, Helitrooper, Armored Skyhunter, Gilgamesh (Cloud and Godo Equipment); Genesis Hydra,
   Cartographer's Survey, Court of Ardenvale; Titania's land (now in the Omnath deck); Kura, Boughside Wanderers
   (`t1.rs::look_take`), Follow the Lumarets; Grapeshot and Brain Freeze copies; Triple Triad (auto-plays every
   card); Thousand-Year Storm copies (no new targets); Proliferate (always "everything"); Alpharael, Stadium
   Tidalmage's discard; Moseo's card; Airlift Chaplain; Venser, the Sojourner (-1 missing); The Wasp; Victimize-style
   inner choices; Mana Vault; Springleaf Drum; Izzet Boilerworks; Mystic Sanctuary. Go deck by deck (JD's decks:
   `data/decks.json` `.mine[]`), cross off the findings rows already fixed by the PRs above (each PR's description lists
   its cards), and fix the rest with `choices::` helpers behind `human::is_human`.
2. **Several blockers on one attacker, and damage assignment order / trample** for the person. `engine/combat.rs::
   resolve_combat_inner` handles one blocker per attacker (`blocker_of` takes the first pair; the ~90-line damage
   block assumes one), and the AI never multi-blocks. It needs that block restructured; do it as its own task with
   extra sims (combat touches every game). See `rust/agents/report-practice-walkers.md`.
3. **Smaller leftovers:**
   - Web-slinging, plot, foretell and landcycling (Titania's old cards, now Omnath's) aren't option hooks, so #113
     didn't reach them.
   - Graveyard abilities are sorcery-speed only for the person (Reassembling Skeleton can't return on an opponent's
     turn).
   - ~9 alternative costs run the AI's choices inside (#113's list): give them their own person paths.
   - Saying yes to a DSL "may discard" with an empty hand still runs the dependent draw (#110).
   - Restoration Angel, Sun Titan (hand-written, `dsl: null`), Recruiter of the Guard and Planar Genesis: their "may"
     isn't asked yet.
   - Accursed Marauder-style edicts are done; Annex, Kefka's {8}, Sauron's ward, Obliterator, Despair, End of the
     Hunt, Devourer and Braids III (#108) have no direct test.
   - Hints (`practice/src/advisor/`) don't know about the new choices yet (they give a general tip for them).

## How the work was done (keep doing it this way)
- **Shared rules for each agent:** `rust/agents/practice-fixes-common.md` (on main): the AI must not change; ask through
  `human/choices.rs`; tests with a scripted person (`tests/play_common/mod.rs`); limits. Each task had a brief in
  `rust/agents/practice-<task>.md` and left `report-` / `handoff-` files there.
- **Proving the AI didn't change:** build `commander-sim` on main and on the branch, run the same fixed-seed games
  (`--deck <d> --pool <t> --games 200 --ai adaptive --profile loose --jobs N --no-preflight`, drop the progress and
  timing lines), and diff: they must be identical. Use several decks when a change touches shared code (#114 used
  seven). For a change to `data/cards.json`, compare main's binary with main's data against the branch's binary with
  its data.
- **Test the open PRs together** before calling them ready: merge them in a scratch worktree and run the full suite
  (that is how the #113 / #114 overlap was found).
- **CPU:** the machine is shared. At the pause JD's split was this session 6 threads, the other session 16, 2 free;
  log runs in `/home/joseph/projects/mtg_sims/cpu-ledger.md`. Agents build with `-j 3` each when two run together.
- **Gotchas found on the way:**
  - `find` here is `bfs`: `-newermt "-10 minutes"` errors silently. Use `-mmin -10` in stall watchers.
  - Scripted-person tests that pick "the first way to cast" break when a card gains another way.
  - `ScriptHuman` with no answers left returns `Cancel` (an error): use `from_fn` when a test may be asked things it
    doesn't care about.
  - Regenerate `data/cards.json` only with a cardgen built from the same code (a stale binary drops new flags).
