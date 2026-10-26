# Handoff: realism scorecard (Phase 1 of the improvement plan, in progress)

Read this file, then `CLAUDE.md`. Everything needed to continue is here; the chat that produced it is not.

## Where things are

- **Code:** all of it is on `main`. The latest merges, 2026-10-24:
  - PR #133: docs refresh and the Phase 1 tooling;
  - PR #134: taxed costs, Sisay fixes, the Tier 6 rebalance, the re-fit and reports;
  - PR #135: Tier 6 scored on cEDH metrics only;
  - PR #136: the Tier 6 seat target from TopDeck.gg Swiss pods.

  - PR #138: Tier 6 and Tier 3 traced for bugs, every Tier 3 card modelled, the Tier 3 lists from EDHREC, the Tier 3
    targets rebuilt, Tergrid;
  - PR #139: Tiers 4 and 5 (targets, EDHREC lists, every card modelled; Edgar Markov and Miirym replaced Heliod and
    Winota, The Ur-Dragon being JD's own commander now).

  Open: PR #142 (Tiers 0-2). Work on a new branch from `main`; JD asks for PRs explicitly.
- **Tier 3 report:** `audit/realism/t3-trace-2026-10-24.md` (the trace, the knob fit, the modelling, the new
  targets, Tergrid).
- **Tiers 4 and 5 report:** `audit/realism/t45-targets-2026-10-24.md` (targets, the EDHREC lists, the modelling, the
  combo-commander rule, the Heliod and Winota traces and swaps, and the leader-policing and Edgar-swap experiments
  that changed nothing).
- **Plan document** (claude.ai Doc, JD's): https://claude.ai/artifact/Fa1usjF6hHZiSUBXwLzGNJ, "Commander Sim:
  Improvement and Refactoring Plan". A session with the Claude Docs connector can read and edit it; without it, this
  file summarises what matters (below).
- **Reports**, in `audit/realism/`, read in this order:
  1. `taxed-costs-2026-10-24/REPORT.md`: the fixes and the rebalance;
  2. `t6-fit-2026-10-24.md`: the re-fit;
  3. `t6-seat-edge-2026-10-24.md`: why seat 1 is weak, and the three judgement calls.

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

## Done so far

**Phase 0 is done.** Its gate was a nightly scorecard with targets for two tiers:
- Tier 6 and Tier 3 have targets (`data/realism/t6.toml`, `t3.toml`).
- `.github/workflows/realism.yml` runs every tier nightly (07:17 UTC) and on demand. A target miss does not fail the
  job yet.
- Tiers 0-2, 4 and 5 still have no targets.

**The scorecard** is `commander-sim --realism --pool tN` (`sim-cli/src/realism.rs`):
- It plays the tier's own decks, 4 a pod.
- It prints the sim value, 95% CI, target, band and PASS/FAIL, then each deck's decisive share and the sources.
- A targets table with `scored = false` is a *reference*: printed with its band, result `reference`, left out of the
  pass count and of `--fit`.
- The pod log template is `data/realism/pod_games.toml`; nothing reads it yet.

**Phase 1 tooling:**
- `--knobs FILE` (`sim-cli/src/knobs.rs`).
- `--fit --pool tN [--fit-scope shared|deck] [--fit-gens N] [--fit-stats LIST] [--fit-out F]` (`sim-cli/src/fit.rs`,
  sep-CMA-ES). `--fit-out` is relative to the repo root.
- The Tier 6 calibrate report (`--calibrate within --pool t6`) has a per-deck block:
  - ready, one-tutor-away and go rounds;
  - what each deck chose while ready or one tutor away;
  - combo stops by card;
  - fast mana held;
  - mulligans.

**Phase 1 fixes on 2026-10-24** (all on `main`; details in `taxed-costs-2026-10-24/REPORT.md`):
- **Taxed costs.** The AI checks combos, removal and plan casts against `cost_of`. `mana::seq_cost` adds Damping
  Sphere's tax across a line.
- **Refused plays.** The main options skip casts that `do_cast` would refuse (locks, Bauble or Chalice on free spells,
  unpayable additional costs). Refused plays fell from 1778 to 170 in 150 games.
- **Sisay:**
  - a fetched legendary land enters as a land;
  - `Game::keep_untapped` keeps Ioreth out of the mana payment;
  - the DDR line waits under Blind Obedience.
- **Land search.** "Search for a land card" now finds any land, not only basics.
- **Growing Rites of Itlimoc** is modelled (Approximate).
- **Combo report.** Azorius Guildmage and Tidebinder name themselves in it.
- **Fast-mana report.** Mox Diamond counts as held only with a land to discard.
- **Rebalance:**
  - flex swaps for Sisay and Rograkh/Thrasios: the cards the 8 recent EDHTop16 winning lists play least, for the
    missing cards they play most (`decklists/pool/CHANGELOG.md`);
  - threat values (`common.rs` PVAL): Sisay 8, Kinnan 8 → 5.
- **Kraum/Tymna:**
  - it waits for protection only on a likely stop (stop chance 0.5+, or an opponent with cards and two blue up);
  - Kraum's empty-library check needs an opponent able to cast a second spell.

**Tier 6 now** (8200 adaptive games, loose, seeds 500000+):

| Statistic | Sim | Target (band) |
| --- | --- | --- |
| Seat 1 / 2 / 3 / 4 | 27.6 / 25.8 / 23.8 / 22.8 | 31.8 / 27.3 / 22.1 / 18.8 (each +/- 1; TopDeck.gg Swiss pods) |
| Median win round | 8 (PASS) | 7 (6-8) |
| Deck spread | 15.5 | 4 (0-8) |
| Decks | Dack 31.6, Kinnan 20.2, Kraum/Tymna 28.8, Rograkh/Silas 21.1, Rograkh/Thrasios 16.4, Sisay 32.0 | each 18-32 (TARGETS-t6.md) |
| Win types (reference, not scored) | combo 82.8, damage 16.2, commander damage 0.6 | 47.8 / 39.9 / 11.6 (casual logs) |

**What doesn't move the Tier 6 seat edge:**
- the style knobs (fits don't hold on fresh seeds);
- the lists;
- removing Dack;
- the Mox Diamond rule;
- the tournament-clock draw bias (worth about 1 point);
- one deck going for lines more often (Kraum/Tymna);
- a "do you have it?" counter deferral.

**The seat-edge mechanism** (`t6-seat-edge-2026-10-24.md`):
- The first player to be ready wins 76% in every seat, so seats are treated alike.
- But only about 1.2 players a game ever reach a ready line, and they're ready at a mean round of 7.8, with a
  standard deviation of 2.8 rounds.
- So games are rarely races, and turn order rarely decides them.

## JD's decisions

- **2026-10-23.**
  - Seat targets are EDHTop16's monthly-report shares, each +/- 1.
  - Fitting may overrule any hand-set value.
  - Each deck may get its own fitted settings.
- **2026-10-24.**
  - Pool decks may be updated, or replaced, to fit the real data.
  - EDHREC may be used for the casual decks.
- **2026-10-24 (later):**
  - **cEDH keeps its own metrics.** Tier 6 is scored only on tournament-sourced targets and the speed target. Its
    win types are references until a cEDH source exists. Other tiers keep scoring their win types.
  - **Rebalance Tier 6 with real lists** (done).
  - **Built-in knobs stay**; the fits are not adopted.
  - **The three judgement calls:** Kraum/Tymna waits less (done); Kinnan's threat value 5 (done); free counters
    unchanged (the tried rule had no effect).
  - **Tier 3 (option 2):** model every Partial and Unmodeled card first, then play EDHREC's bracket-3 average lists
    (done; three Game Changers each). Atraxa's list is infect (`atraxa-infect`), so Atraxa rejoins the targets (done;
    the five commanders are weighted equally, the main session's choice: pooled, Atraxa would be half of every
    target).
  - **Tergrid:** opponents race it and feed it only tokens; its threat value is 10 (JD kept it).
- **2026-10-25:**
  - **Tiers 4 and 5:** EDHREC's bracket-4 lists; Tier 4 keeps its no-Lotus-Petal rule. Heliod and Winota retired
    (`decklists/pool/retired/`) for Edgar Markov and The Ur-Dragon. Opponents keep an answer up against a combo
    commander. Win types are references in `t4.toml` and `t5.toml` (as in Tier 6): the lists are tuned combo and
    engine decks while the commanders' Playgroup wins mix every build.
  - **Deck balance left as it is:** Edgar (about 50% in Tier 4) and Ur-Dragon (37%) stay. Leader policing (four
    levers) and up to eight real swaps in Edgar's list moved nothing; its edge is eminence itself.
- **Standing rules in CLAUDE.md still apply:** loose profile only, each of JD's decks only against its own tiers, no
  Python, the memory cap, check subagents' work. Run every sim, test and build in the background. Push-notify JD
  whenever work stops for their direction.

## Next, in order

1. **The Tier 6 seat target is settled: it is real** (`audit/realism/t6-seat-topdeck-2026-10-24.md`).
   - Rebuilt from 29864 TopDeck.gg Swiss pods: 31.8 / 27.3 / 22.1 / 18.8.
   - Round 1, seated at random, shows the same edge, so it is not stronger players seated first.
   - The sim's seat 1 (27.6%) is a real gap of about 4 points.
   - The lever is the AI: more decks reaching ready lines earlier, through wider line catalogues and, longer term,
     Phase 4's planning AI.
   - Real data on assembly speed (win turns from tournament replays, JD's pod log) would help set it.
   - TopDeck.gg key: `~/.config/topdeck/key` (JD's; never commit it). Credit TopDeck.gg wherever its data is used.
2. **Rograkh/Thrasios** is done on branch `t6-rograkh-thrasios` (17.9% to 21.3%, inside the band;
   `audit/realism/t6-rograkh-thrasios-2026-10-26.md`). Its leads: `deck_combos` drops quiet lines from `combo_progress`;
   untagged tutors; Moxen held early.
3. **Tier 3's remaining misses** (combo lines done on branch `t3-misses`: combo/alt 11.5% to 16.5%, on target;
   `audit/realism/t3-lines-2026-10-26.md`) (lists, cards and targets are done; `audit/realism/t3-trace-2026-10-24.md`):
   - median win round 10 against 8;
   - combo wins 8.9% against 16.5%;
   - commander damage 10.5% against 14.4%;
   - poison 19.2% against 9.3% (Atraxa, and Tergrid winning with stolen infect creatures);
   - Tergrid still 36.9% of its decisive games, the others 18-26%.

   Atraxa's last Partial cards are listed in its pool file. Speed and combo, as in Tier 6, are AI quality more than
   card rules.
4. **Every tier now has targets** (Tiers 0-2: PR #142, `audit/realism/t012-targets-2026-10-25.md`; Tier 0 keeps its
   precons, Tiers 1-2 play EDHREC's bracket 2-3 lists, every card modelled; win types scored there). Deck balance in
   Tiers 1-5 (one list wins 37-50% in each: Light-Paws, Sythis, Tergrid, Edgar, Miirym) is AI work for later: the
   opponents seldom hold an answer to it. Casual tiers win by combo far less than real games: the lines the lists
   really hold were added on `casual-combos` (`audit/realism/casual-combos-2026-10-26.md`); the gap left is the lists
   (Playgroup's commander pages pool upgraded builds), not missing lines.
5. **Make a target miss fail the nightly job**, once the Tier 6 gate passes.
6. **Older items:**
   - the Ertai/Tataru bug (`#[ignore]`d test `tataru_with_ertai_in_hand`);
   - the stale tier summary in `CLAUDE.md`, left for JD;
   - the `yshtola-cedh` branch, kept separate.

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
  only at the end; redirect to a file. A full `cargo test --release` passes (3,012 tests) and takes about 10 minutes.
- Pre-existing rustfmt differences in `poolmode.rs` and `decks.rs`: format only the files you write
  (`rustfmt --edition 2024 <file>`).
