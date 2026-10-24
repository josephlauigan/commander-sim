# Handoff: realism scorecard (Phase 0 of the improvement plan, in progress)

Read this file, then `CLAUDE.md`. Everything needed to continue is here; the chat that produced it is not.

## Where things are

- **Code:** all of it is on `main` (merged in PR #131, 2026-10-23). Work on a new branch from `main`; JD asks for PRs
  explicitly.
- **Plan document** (claude.ai Doc, JD's): https://claude.ai/artifact/Fa1usjF6hHZiSUBXwLzGNJ, "Commander Sim:
  Improvement and Refactoring Plan". A session with the Claude Docs connector can read and edit it; without it, this
  file summarises what matters (below).
- **Commits (PR #131):** `8685877` (targets format + `--realism`), `74a2a48` (README), `f4c1822` (t6 baseline),
  `46fb1bc` (+/- 1 bands), `7fb4cd3` (CI), `9567a2f` (flaky practice image test fixed), and this handoff.

## The plan in brief (from the document)

Goal: keep the Rust engine, but fit the sim to real games instead of tuning it against itself. Phases, each with a
gate that must pass before the next starts:

| Phase | Work | Gate |
| --- | --- | --- |
| 0 Ground truth | Realism scorecard and `--realism`, run in CI; pod game log; EDHTop16 / TopDeck.gg import; Playgroup data; targets for Tier 6 first, then one casual tier | Scorecard runs nightly with targets for two tiers |
| 1 Fit and guard | Fit AI style knobs against the scorecard; generic rule checks (Pact upkeep, tutor then discard); sequential stopping and an interval on every delta | Tier 6 seat edge inside its target band |
| 2 One card model | Export refuses cards modeled by two paths; combos from Commander Spellbook; coverage per deck on every run | No double-modeled cards; coverage printed per deck |
| 3 Engine structures | Characteristics pass, one replacement step, mana pool with restrictions, identity for exiled cards, one permanent arena | Paired tests level, scorecard holds, under 10% slower |
| 4 Generic AI | Plan files as data, card values from abilities, learned evaluation, tree search on high-stakes choices; delete per-deck hand code | Each deck level with its hand code in a paired test |

Scorecard statistics (per tier): seat win rates, median and spread of the winning round, win-type mix, win-rate
spread across decks, games over 15 rounds; later commander casts and removals, wipes and counters per game, and
per-commander win rates.

## Done so far (Phase 0)

1. **Targets format**, `data/realism/<tier>.toml`: `tier = "tN"` and one table per statistic with `value`, `lo`, `hi`
   (the band the sim must land in), `n` (real sample size), `source`, `date`, optional `note`. Statistic keys:
   `seat1_win`..`seat4_win`, `win_round_median`, `win_round_spread`, `win_combo_alt`, `win_damage`,
   `win_commander_damage`, `win_poison`, `deck_win_spread`, `games_over_15`. A missing or unknown field, an unknown
   statistic, a wrong `tier`, or a value outside its band is refused before any game is played.
2. **t6 filled** (sources below); **t3 filled** (2026-10-24, Playgroup: seats, median win round, win types; Atraxa
   left out, see its file); **t0-t2, t4, t5** hold only `tier` and comments naming their future sources.
3. **`commander-sim --realism --pool tN`** (`rust/crates/sim-cli/src/realism.rs`): plays the tier's own decks, 4 a
   pod, using the `--calibrate within` games (`Ctx::tier_games` in `poolmode.rs`, shared with `calibrate`). Prints
   sim value, 95% CI, target, band, PASS/FAIL, then games, decisive share, timeouts, each deck's decisive share and
   the target sources. `--pool all`, `--deck`, `--swap`, `--calibrate` with it are refused.
   - Shares are % of *decisive* games (a winner that is not the round-cap tiebreak). Win types come from
     `t6report::win_route`: `damage` -> win_damage, `commander damage`, `poison`; everything else (a named combo line,
     "other" = alternate win or mill) -> win_combo_alt. The median CI uses order statistics; spread and
     deck_win_spread have no CI. `games_over_15` counts round > 15 (the round cap is 20, `play.rs`).
   - Tests: unit tests in `realism.rs` (parser refusals, every repository targets file valid, quantiles) and
     `rust/crates/sim-cli/tests/realism.rs` (end-to-end on 4-8 adaptive games; refusals).
4. **Pod log template**, `data/realism/pod_games.toml`: one `[[game]]` per game with `date`, `seats` (four decks in
   turn order: JD's deck key or the commander's name), `winner` (seat 1-4, 0 = draw), `turn` (round it ended), `how`
   (combat | commander damage | other damage | combo | alt win | mill | poison | concession | draw), optional `notes`.
   Nothing reads it yet.
5. **README** documents `--realism`.
6. **t6 baseline**, `audit/realism/t6-baseline.md` (+ raw `t6-baseline-2200.txt`): 2,200 look-ahead games, loose,
   seeds 500000-502199. **3 of 10 pass.**

| Statistic | Sim | Target (band) |
| --- | --- | --- |
| Seat 1 / 2 / 3 / 4 | 27.5 / 27.5 / 22.4 / 22.7% | 32.5 / 26.9 / 22.7 / 17.9% (each +/- 1) |
| Median win round | 9 | 7 (6-8) |
| Combo or alt / damage / commander damage / poison | 76.0 / 23.5 / 0.6 / 0.0% | 47.8 / 39.9 / 11.6 / 0.7% (each +/- 1) |
| Deck spread | 16.3 pts (Kraum/Tymna 32.8%, Rograkh/Thrasios 16.5%) | 4 (0-8) |
| Games over 15 rounds | 11.2% | no target |

The slow table and flat seat edge match `audit/t6/speed-2026-10-19/REPORT.md`, which found the decks draw- and
mana-limited rather than waiting.

7. **t3 baseline**, `audit/realism/t3-baseline.md` (+ raw `t3-adaptive-8200.txt`): 8,200 adaptive games. **1 of 9
   pass** (poison). Median win round 13 against 8, 33% of games past round 15, combo 1.9% against 20%, seat 1 26.5%
   against 31.3%. Korvold (13.9%) and Marwyn (11.7%) are the weakest decks, though on Playgroup they win most.

## JD's decisions

- 2026-10-23: seat targets are the EDHTop16 monthly-report shares. Every share target is held to +/- 1 point. Win
  types stay scored (although their source is casual logs). Fitting may overrule any hand-set value or threshold,
  the loose profile's included. Each deck may get its own fitted settings, to close the deck spread.
- 2026-10-24: pool decks may be updated to fit or reflect the real data (for example, the Tier 3 Atraxa made the
  deck players actually log, so it can rejoin the targets).
- Standing rules in CLAUDE.md still apply: loose profile only, each of JD's decks only against its own tiers, no
  Python, the memory cap, subagents on the default model, check subagents' work.
- Still open in the plan document: start logging pod games now (by hand or from practice mode)? Order of the engine
  work (permanent arena or characteristics pass first)? Pause new card work during the card-model move?

## Next, in order

**Finish Phase 0** (gate: nightly scorecard with targets for two tiers):

1. Targets for one casual tier: **done**, Tier 3 (`data/realism/t3.toml`, branch `realism-casual-targets`). The gate
   passes once that branch is on `main` and the nightly run scores it. The same method fits the other casual tiers:
   commanders.json for win types, each commander page's "Does seating order matter?" block for seats,
   `/api/public/v1/commanders/<id>.json` `stats.average_wins_by_turn` (one number, not a distribution) for speed.
   Check each commander's win types against the pool deck's archetype first (Atraxa's were 45% poison).
2. Nightly run: done (JD approved CI, 2026-10-23). `.github/workflows/ci.yml` builds and tests on
   every push and PR; `.github/workflows/realism.yml` runs the scorecard nightly (07:17 UTC) and on demand: adaptive
   AI on every tier at 8,200 games, look-ahead on Tier 6 at 2,200 (input `lookahead_games`). Tables go to the run's
   summary and an artifact; a failing target does not fail the job. Both are live on `main` (the scorecard runs
   nightly; "Run workflow" on the Actions tab runs it on demand). Once Phase 1 passes, make a target miss fail it.
3. Optionally a reader for `pod_games.toml` (validate records; summarise seat, round and win-type stats).

**Phase 1** (gate: Tier 6 seat edge inside its band):

1. Make the knobs data, with no behaviour change at current values (a paired run must show 0 difference):
   `ai/mod.rs` `Style { temp, aggression, caution }` per deck (`ai/plans.rs`, `ai/pool.rs` `DEFAULT_STYLE` 1.0 /
   0.6 / 0.5); `settings.rs` `Settings::new` counter thresholds (`cthresh`, loose 6) and `instant_extra` (loose 0);
   `Search` weights `combo_w`, `ramp_w`, `hold_w` (env `SEARCH_RAMP_W` / `SEARCH_HOLD_W`) and `top_k`, `rollouts`,
   `horizon`.
2. A fit command (e.g. `commander-sim --fit --pool t6`): derivative-free search (CMA-ES or similar, in Rust), on the
   adaptive AI (about 1,000 games a minute), same seeds for every candidate, loss = distance to each target. Per-deck
   parameters are allowed.
3. Confirm with the look-ahead AI at about 8,200 games (needed to resolve +/- 1; about 5 hours on 4 cloud cores, faster
   on JD's machine). Write `audit/realism/t6-fit-<date>.md`; update the plan document's Phase status.

## Data sources (verified 2026-10-22, with what each offers)

- **Playgroup.gg**: `https://playgroup.gg/api/public/v1/commanders.json` lists 5,142 commanders in one request
  (games, wins, `average_round_amount`, win counts by combat, combo, commander damage, non-combat damage,
  alternative, mill, poison; mulligans, damage). `/api/public/v1/commanders/<id>.json` adds `average_wins_by_turn`.
  Commander HTML pages (`/commanders/<id>-<slug>`) show seat win rates. Stats are CC BY 4.0 (`/legal`): credit them.
  Terms (`/tos`) have no scraping or API clause; robots.txt allows crawling. No bracket field, no per-game export.
  Turn-order page (`/commander/turn-order`, computed 2026-10-19): 577,284 finished 3+ player games; 4-player
  single-winner (284,950): seats 29.2 / 25.7 / 23.6 / 21.5%; median game 9 rounds.
- **EDHTop16**: `https://edhtop16.com/reports/monthly_seat_winrates` is a public page with cEDH seat 1-4 win rates and
  draw rate by month since March 2024. t6 seat targets = April-September 2026, sample-weighted, each seat's rate
  divided by the four seats' total (they cover 91.7% of entries; 20.2% draws). Commander pages show entries, meta
  share and conversion only. The GraphQL API accepts persisted queries only.
- **TopDeck.gg**: REST API v2 (`/docs/tournaments-v2`), free key, 100 requests/minute, "must include a visible credit
  and link back to TopDeck.gg". Standings, decklists, round tables (players, winner); no seat field. Bracket events
  exist but are few and small.
- **EDHREC** "Does cEDH have a seat order problem?" (24 Mar 2026): ranges only, no source: not usable.
- The other t6 targets: median 7 = JD's speed target (`decklists/pool/TARGETS-t6.md`), consistent with Playgroup's
  average win turn of the t6 commanders (6-7). Deck spread 4 (0-8) = EDHTop16 six-month decisive shares of the five
  established t6 decks, 23.9-27.9% (TARGETS-t6.md, pulled 2026-10-17; not re-readable now).

## Gotchas

- **Cloud sessions have no systemd**, so the CLAUDE.md `systemd-run` cap fails. This branch's runs used a cgroup v1
  memory group instead (12 GB, swap 0; exit 137 on a breach): create `/sys/fs/cgroup/memory/<own group>/simcap-$$`,
  write the limit to `memory.limit_in_bytes` and `memory.memsw.limit_in_bytes`, put the shell's pid in `cgroup.procs`,
  then exec the command. On JD's machine use the CLAUDE.md command as written.
- Timing on 4 cores: look-ahead about 2.3 s a game (wall), adaptive about 0.007 s. `--realism` with `--quiet` prints
  only at the end; redirect to a file. A full `cargo test --release` passes (2,986 tests) and takes about 10 minutes.
- Pre-existing rustfmt differences in `poolmode.rs` and `decks.rs`: format only the files you write
  (`rustfmt --edition 2024 <file>`).
