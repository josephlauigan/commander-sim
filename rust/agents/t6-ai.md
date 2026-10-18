# Brief: Tier 6 Stage 5 — teach the AI cEDH habits

Read `rust/agents/RULES.md` (memory cap, no Python, commits, short reports; ignore the Python-port parts and the
`git merge rust-port` step). Your task, worktree and branch are in the prompt that started you; the task's section
below says what to build. Work only in your worktree.

Background: Tier 6 is six cEDH pool decks (`decklists/pool/t6-cedh/`) plus JD's Godo deck (`godo`, matched to t6 only).
Every card and combo is modeled (Stage 3). Each deck has a behaviour spec, `audit/t6/<deck>/SPEC.md` (game plan, win
lines, mulligan, turn by turn, interaction, when it goes for the win, what it fears, decision scenarios). The
targets are in `decklists/pool/TARGETS-t6.md` (seat shares 34/27/23/17 of decisive games, each deck 18-32%, best-worst
gap 8 points or less). Stage 5 makes the decks PLAY like their specs; Stage 6 (later, not you) tunes numbers.

A first look (300 games per profile, `--calibrate within --pool t6`): games average 9.4 rounds, 2-3% timeouts;
Kraum 32%, Dack 28-37%, Rograkh&Thrasios or Rograkh&Silas (the report prints both as "Rograkh") 10-12%, Sisay 20-24%,
Kinnan 24-26%. In traced games (`--deck godo --pool t6 --trace N`) Dack's first flip went off on round 3 with no
answer from three opponents holding cards.

## Rules for every Stage 5 task

- **Tiers 0-5 must not change.** Gate every AI change to Tier 6 games or decks (the player's deck key is a t6 pool
  deck or `godo`; ask a helper, don't scatter key lists). Proof: from `rust/`,
  `./target/release/commander-sim --calibrate within --pool t3 --games 300 --seed 500000 --jobs N` must match
  `rust/agents/perf-lock-t3.txt` except the progress and timing lines (`diff` after `grep -vEi
  "progress|elapsed|minutes|seconds|games/h|worker threads|s total"`). Run it before your final report and after any
  change to shared AI code. A real rules fix that changes other tiers is allowed only if you say so in the report.
- **Sims:** only Tier 6 (`--calibrate within --pool t6`, or `--deck godo --pool t6`), at most 600 games per run, plus
  the lock above. `--trace N` (with `--deck godo --pool t6`) for single games.
- Shared machine: `CARGO_BUILD_JOBS=5 nice -n 19 systemd-run --user --scope -q -p MemoryMax=14G -p MemorySwapMax=0
  cargo ...` from `rust/`; sims with `--jobs 5` under the same wrapper. Run only the test files you touch while
  working; the full `cargo test --release` once at the end, and `cargo build --release --all-targets` with no
  warnings.
- Progress: create `rust/agents/handoff-t6-ai-<task>.md` FIRST; update it after each step and BEFORE any long build
  or sim. A lead checks on an agent silent for 12 minutes. Commit after each working piece with `git add -A rust data
  audit` (check `git status` is clean after), standalone commits, no Co-Authored-By.
- Timebox: if one sub-part resists for ~15 minutes, commit what works, write the gap in the report, move on.
- Done: full tests pass, no warnings, lock identical. Report in `rust/agents/report-t6-ai-<task>.md`; final message
  at most 12 lines.

## Task `report` — the Tier 6 measuring tool

Everyone after you measures with this, so make it exact and cheap (no extra games).

1. Deck names: pool decks with partners print both names, short ("Kraum/Tymna", "Rograkh/Silas", "Rograkh/Thrasios")
   everywhere a deck name is printed (calibration, traces, practice lists if they share the helper). Fix tests that
   pinned the old names.
2. `--calibrate within --pool t6` (and any tier, if it costs nothing) prints after the balance table, per deck:
   wins split by how (combo line name, damage, commander damage, poison, other), the average and median round of
   its wins, how often it went for a combo and how often that was stopped and by what card (the combo framework in
   `impls/combos.rs` has per-step answer windows; record the answering card), mulligans taken, and the interaction
   it cast (counters, removal, stax) with whose spell or permanent it hit. Plus, per tier: the win share by seat
   (seat 1-4 of decisive games) and the draw/timeout rate, in the TARGETS-t6 terms. Collect the counts during the game
   (a small stats struct on the game), not by parsing logs.
3. Tests for the counters (a scripted game state, the counter increments) and for the seat shares.

## Task `table` — the shared cEDH table rules, and the plan slots

Read all seven SPEC.md files ("Interaction", "When it goes for the win", "What it fears") and `rust/agents/report-t6-e-answer.md`
(how answer windows work), then audit what the AI already does (`ai/brain.rs` counter/answer choices,
`engine/stack.rs` `counter_window`, `would_be_cast_countered`, `combos.rs` `combo_options` and its `ctr_risk`). Build,
gated to Tier 6 players:

1. **Archenemy:** a score per opponent for "about to win" (combo pieces held or on board, tutors and mana up, a
   known line's `ready`), used when choosing what to counter or remove and whom to attack.
2. **Spend answers on the win:** counters and removal are held for the archenemy's win attempt or key engine, not
   fired at the first spell; each combo step's answer window is used by whoever can stop it (not only the first
   opponent asked). A free counter (Force of Will, Pact, Fierce Guardianship, Deflecting Swat) is used when the step
   would otherwise win.
3. **Hold up vs develop:** when an opponent is close, a player holding instant-speed interaction keeps the mana for
   it instead of tapping out for a value spell.
4. **Go when it can't be stopped:** before going for a combo, count the table's open mana and cards in hand (and known
   free interaction: Force, Fierce Guardianship, Pact, Swat) against the line's steps; go when the risk is low, or
   when waiting is worse (someone else is about to win). Protection first if the deck holds it (spec says which).
5. **Ad Nauseam, Silence, Orim's Chant, Angel's Grace** (carry-overs): Ad Nauseam is a counter target like any win
   spell; Silence and Orim's Chant are cast in the upkeep or in response before going for the win; Angel's Grace
   answers a lethal Ad Nauseam or a damage kill.
6. **Plan slots for the deck agents:** one file per deck, `impls/t6_plan_<deck key>.rs` (the seven keys:
   the six t6 pool deck keys and `godo`), each with stub functions the shared AI calls, all returning "no opinion" for now:
   cast priority (`prio`), tutor wish list (`wish`), mulligan keep (`keep`), whether to hold mana up this turn
   (`hold_up`), and whether to go for a given combo now (`go_now`). Wire the dispatch once (one match on the
   key, in one place), so deck agents only edit their own file. Write the slot contract at the top of
   `impls/t6_plans.rs` (or the dispatch file).
7. **Decision tests**: for the shared rules, set up a position and check the choice (e.g. three opponents, one
   casts Thassa's Oracle with Consultation on the stack: the player holding Force of Will counters it; a player with
   Swan Song up does not tap out for a creature while an opponent has Breach + mana).

Measure Tier 6 before and after (`--calibrate within --pool t6 --games 300`), in a table in the report: per deck
win %, average game rounds, timeouts.

## Task `deck:<key>` — one deck's plan (later wave)

Fill `impls/t6_plan_<key>.rs` from the deck's SPEC.md: mulligan (keep rule), turn-by-turn priorities, tutor
targets by situation, when to hold up which interaction, when to go for which line, protection before going. Add
the spec's "Decision scenarios" as AI tests (`tests/ai_t6_<key>.rs`). Measure with the `report` tool: the deck's win
routes and win rounds should look like its spec (the spec's main line is its most common win, backup lines rarer).
Note in the report what still differs from the spec and why.

## Before the deck plans (2026-10-18, after `report` + `table`, commit 423c135)

`audit/t6/stage5-baseline.txt` is the full Tier 6 report (300 games per profile) every deck agent starts from. What
it shows, deck by deck, is in the `deck:<key>` prompts; across the table:

- **Mulligans are far too many**: 36-48% of hands, with 9-19 triple mulligans per deck in 200 games. The shared rule
  keeps 2-5 lands; cEDH decks keep on fast mana and a plan with 1-2 lands. Each deck's `keep` slot should follow its
  spec's mulligan section.
- **Wins come too late**: median win round 7-8 for the fastest decks, 12-15 for Sisay and Rograkh/Thrasios (they
  win by combat damage, 80-95%, and almost never go for their combos: 0.02-0.07 tries per game). The specs' decks
  aim to win on turns 3-6.
- **Seat edge too flat** (seat 1 wins 25-27% of decisive games; target 30-38%): slow games wash out the first-seat
  edge, so faster, plan-driven decks should fix most of it. Don't tune seat shares directly.
- Deck agents measure their own deck with `--calibrate within --pool t6 --games 300` (the report block for the deck),
  and must not change other decks' plan files.

## Task `windows` — answer windows when a line is assembled by normal play (2026-10-18)

Godo wins 50-55% against Tier 6 (target 15-35%). Traces (`--deck godo --pool t6 --trace N`, N = 0-24) show the
cause: about half its wins are labelled "damage" but are Godo + Helm of the Host played out by normal actions (Godo
cast turn 2, its ETB puts Helm onto the battlefield, turn 3 `equips Helm of the Host to Godo`, Helm's copies give
endless combats). That path never goes through `combo_options`, so no combo step answer window opens, `t6_going` is
never set, and nobody removes Godo or Helm in response to the equip or to Helm's beginning-of-combat copy trigger.
Counters on the Godo spell itself are fine now (b92bd42: a permanent that completes or advances a line is valued as a
piece); on turn 2 the opponents have one land each, so only Force of Will can answer the cast, as in real games.

Build, gated to Tier 6 players: when an action of a Tier 6 player (an equip, an activation, a cast, a trigger) would
complete or execute one of its modeled lines (`combos::piece_gain` >= 1.0, or the line's `ready`), the opponents get
the same answers they get at a combo step: instant-speed removal or bounce on the key permanent (Godo, Helm) in
response to the equip, a Stifle-type or removal answer to Helm's trigger, and the archenemy/`counter_verdict` logic
treating it as a win step. Prefer one general mechanism (e.g. mark the line as going, `t6_going`, when the action
starts it, so the existing windows apply) over Godo-only code. Check Kinnan + Basalt and the Oboro line the same way.
Tests: the Godo + Helm equip with an opponent holding Swords to Plowshares / Snap / Chain of Vapor and mana up gets
answered; with no answer it still wins. Measure Godo (`--deck godo --pool t6 --games 300`) and Tier 6
(`--calibrate within --pool t6 --games 300`) before/after; the t3 lock stays identical.
