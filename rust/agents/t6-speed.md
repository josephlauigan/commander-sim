# Brief: Tier 6 speed-up (2026-10-19)

Read `rust/agents/RULES.md`, `rust/agents/t6-ai.md` (Tier 6 AI: plan slots, table rules, report tool),
`rust/agents/t6-calib.md` (Stage 6) and `audit/t6/calibration-2026-10-19/REPORT.md` (where Tier 6 stands).

**Goal (JD):** Tier 6 decks win at a **median round of 5-6** (real cEDH: turns 4-6). Now: median round 7-9 for
Dack, Kinnan, Kraum/Tymna, Rograkh/Silas; 12-13 for Sisay and Rograkh/Thrasios; only ~21% of wins by round 6. The
Stage 6 targets still apply (each deck 18-32%, gap 8 points or less, seats 30-38/24-31/19-26/14-21); a faster table
should also fix the flat seat edge.

## Rules
- Tiers 0-5 stay identical: the t3 lock `rust/agents/perf-lock-t3.txt` (diff after
  `grep -vEi "progress|elapsed|minutes|seconds|games/h|worker threads|s total"`). Gate every change to Tier 6.
- Every build, test and sim runs in the background (run_in_background) under `nice -n 19 systemd-run --user --scope -q
  -p MemoryMax=16G -p MemorySwapMax=0`; thread counts as your prompt says. Never kill -STOP, never pgrep -f.
- Handoff `rust/agents/handoff-t6-speed-<task>.md` FIRST, committed, updated after each step. Commits with
  `git add -A rust data audit`, no Co-Authored-By. Report `rust/agents/report-t6-speed-<task>.md`, final message 12
  lines or fewer.

## Task `measure` — where does the time go?

For each Tier 6 deck, per game, add to the Tier 6 stats (sim-core/src/t6stats.rs, sim-cli/src/t6report.rs; stats
only, no decisions, no RNG):
1. **ready round**: the first round the deck had one of its lines `ready` (all pieces in hand/battlefield, the mana)
   at a point it could act (its own main phase, or an instant window for instant-speed lines). Hidden information is
   fine here: this is a measurement.
2. **reachable round**: the first round a line was one tutor away (the missing piece findable by a tutor in hand
   with the mana for tutor + line), per the spec's tutor tables.
3. **go round**: the first round it actually went for a line; and the win round.
4. Per deck: median ready / reachable / go / win rounds, the share of games where ready came before go (and by how
   many rounds), and the most common thing it did instead on its ready-but-didn't-go turns (the chosen option label).
Then run `--calibrate within --pool t6 --games 1000 --seed 500000` and write the table in the report, deck by deck:
is the delay AI-limited (ready/reachable early, go late) or draw-limited (ready late)?

## Measure result (2026-10-19, report-t6-speed-measure.md, audit/t6/speed-2026-10-19/cal1000.txt)

Lines become ready late (median round 6-7 even for the fast decks; Dack 20%, Sisay 14%, Rograkh/Thrasios 16% of
games ever ready), and the decks go at once when ready, except Kraum/Tymna (ready 51%, go 30%: "stop (hold mana)"
656 times). Mana is the bottleneck: ~4.4 mana on turn 4; Dack's line is Dack alone ({4}{W}{W}) and is ready at round 7.

## Task `mana` — fast mana and early development, all six decks

1. Measure mana available at the start of each turn 1-5 per deck (stats only) and the number of mana permanents /
   fast-mana spells cast on turns 1-3 vs held in hand. Compare with what each list should do (its spec's "Turn by
   turn" and mulligan sections: cEDH decks aim for 4-6 mana on turn 2-3).
2. Find why fast mana is held or played late: plan priorities, the look-ahead's "stop", Chrome Mox / Mox Diamond
   gates, rituals held, Lotus Petal never sacrificed, dorks cast after other spells, land drops, mulligans keeping
   mana-light hands.
3. Check the mana sources' models: Ancient Tomb, City of Traitors, Gemstone Caverns, Mana Vault, Grim/Basalt
   Monolith, Lotus Petal, Elvish Spirit Guide / Simian Spirit Guide, Dark Ritual and other rituals, Gaea's Cradle,
   Mox Amber, Mox Opal, Chrome Mox, Mox Diamond, Jeweled Lotus, the dorks. Fix rules errors.
4. A shared Tier 6 rule is allowed (e.g. "on turns 1-3, mana first": cast every mana source that pays for itself
   before other spells, gated to Tier 6), plus per-deck plan priorities.
Measure: mana by turn, ready/go/win medians, win % (600 games while working, 2000 to confirm).

## Task `kraum-go` — Kraum/Tymna waits with a ready line

Ready in 51% of games, goes in 30%; on ready-but-didn't-go turns it chose "stop (hold mana)" 656 times or cast a
commander. Its go_now (impls/t6_plan_kraum.rs) demands creature cover (FoW with 2 blue, Pact, Silence, Chant) while
any opponent has cards plus mana: that is nearly always. Make it go like the spec says (when the table is tapped
low, or with one piece of protection, or before an opponent's win), and measure win round and win %; Kraum/Tymna
must stay inside 18-32%, so trade some of the extra speed for fewer, better-timed tries if needed.
