# Report: doomsday-b (follow-up to PR 2's open items, 2026-11-06)

Branch doomsday-b (from doomsday 41dacca6). Brief: `rust/agents/doomsday-b.md`. The agent returned this report as text;
the coordinator saved it (from a fresh worktree: see "Process" at the end). No sims.

## Item 1: pile extras (`impls/doomsday.rs`; tests in `tests/doomsday.rs`)
- **Thrasios {4} dig**: a search move (scry 1, then reveal: a land enters, anything else is drawn; the library is one
  shorter either way; {4} cumulative). Played by `tymna_cards::pile_dig` (the normal activation stays refused mid-pile);
  the scry by `pile_scry` with n = 1. The {4} is paid from lands the rest of the line doesn't need (the engine's generic
  payment tapped the blue lands Silence and the Oracle needed: found by a failing test).
- **Devotion permanents in hand**: Mockingbird, Mystic Remora, Rhystic Study, Valley Floodcaller each add 1 devotion for
  their blue cost (in hand only; no pile slot).
- **Veil of Summer**: counts as the lock when held with {G} left after the Oracle's cost (the engine casts it in response to
  a blue/black counter). Used only when no opponent's deck can hold Mindbreak Trap, Red Elemental Blast or Pyroblast
  (`veil_answers`); takes a pile slot only when Silence can't be the lock. The cache key includes `veil_answers`,
  `forced_draws` and the draw-step extras.
- **Forced draws**: with our own Esper Sentinel and an opponent holding a card, a pile must still win after one forced
  draw; Howling Mine adds to the draw step in the end-step case.
- **Borne Upon a Wind as enabler**: its draw happens before Doomsday resolves and the pile is built after it, so it was
  already counted; a test added, no code.

## Item 2: Noxious Revival
A Revival in hand or drawn by the line makes the pile "backed" (`Built::backup`, `Want::backup`); a Revival in the library
or graveyard goes into a pile slot when the line draws it in time; piles score (lock, backup, fewer spells); `plan_cast`
no longer declines an unlocked pile into two stoppers when it is backed; Revival targets the countered Oracle during a
pile (`t6_green2::revival_pick`; before it took the best-valued card in the graveyard); its generic cast priority is 0
mid-pile.

## Item 3: colour-source keep term
New quantities `hand-gaps[:set]` and `hand-castable[:set]` (`ai/plan_data.rs`, from `impls/t6_keep.rs::hand_sources`):
the hand's lands, fetches (as the colours of the library's lands), and the rocks and dorks the hand can cast, to a
fixpoint; Chrome Mox counts as nothing. `thrasios.toml`: `hand:action>=1 & hand-castable:action=0` goes back (replaces the
Doomsday-only rule; `hand-b-lands` still parses). No other plan file uses it.

## Item 4: table risk check
`t6_table::go_risk_covered(g, p, covered)` and `go_threshold` (`go_ok` unchanged); `plan_cast` calls `risk_ok` after the
existing declines and the plan gate. Cover: Silence or Orim's Chant in hand, a free counter, the pile's lock, or Veil with {G}.

## Tests
22 new (40 in `tests/doomsday.rs`): Thrasios dig wins a pile; a revealed land still shortens the library; the AI doesn't
dig by itself; Mockingbird shortens a pile; Veil as the lock; Veil answers a Counterspell on the Oracle; a forced draw from
an empty library stops the end-step cast; Howling Mine; Borne as enabler; Revival backs a line, takes a slot, targets the
Oracle; the cast waits under three tables, against Trap decks, and for Veil as cover; three keep-rule tests. The agent ran
1128 tests across 150 test files and 108 in `-p practice`: 0 failed; build without warnings.

## Still open
- Recovery after a countered Oracle trigger (Stifle): Mockingbird X = 1 (correct per Oracle: it copies a creature with
  mana value up to the mana spent), Snapback, Chain of Vapor.
- Thrasios cast from the command zone mid-pile (devotion plus dig).
- An opponent's Esper Sentinel taxing our first noncreature spell ({1}, or they draw); in the main phase the Doomsday cast
  itself is that spell.
- **Engine Veil of Summer (shared rules gap)**: `rules::veil_response` answers only blue/black counters, once per Veil.
  Oracle: "Spells you control can't be countered this turn", so it also stops Red Elemental Blast's counter mode and every
  counter that turn (not Mindbreak Trap, which exiles). Not changed: shared with other decks.
- **Pilot guide text** (`decklists/JD/tymna-thrasios-doomsday.md`): "Esper Sentinel's draw is forced" is not accurate for an
  opponent's Sentinel (its controller draws); "swap Silence for Veil against Mindbreak Trap" contradicts the guide's own
  later line that Veil doesn't stop Trap.

## Shared-file changes
`ai/t6_table.rs` (`go_risk_covered`, `go_threshold`), `ai/plan_data.rs` (`hand-gaps`, `hand-castable`), `impls/t6_keep.rs`
(colour-source helpers), `impls/tymna_cards.rs` (`pile_dig`), `impls/t6_green2.rs` (Revival target mid-pile),
`data/plans/thrasios.toml` (keep rule), `audit/thrasios/FINDINGS.md` (Doomsday section). The builder grows when Silence
is unusable (Veil joins the candidates): check pile-build time in the first sim.

## Process (rule breaks, for JD)
- The agent ran a bare `git stash -q` (nothing to stash) and `git stash pop`, which applied another session's stash@{0}
  ("On log-spirit-guide: stale local CLAUDE.md/AGENTS.md (superseded by origin/main)") to its worktree
  `.claude/worktrees/doomsday-b`. The pop conflicted, so `CLAUDE.md` is unmerged there and the stash entry is still in the
  list (nothing lost). Its cleanup (`git checkout HEAD -- CLAUDE.md`) was denied by auto mode; the coordinator did not run
  it either and left it for JD. This report and the checks were done from a fresh detached worktree,
  `.claude/worktrees/dd-b-check`, at the agent's last commit 9ae18909.
- The agent typed `python3 --version` once; the no-Python hook blocked it.
