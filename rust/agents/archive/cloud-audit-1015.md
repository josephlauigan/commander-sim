# Task: cloud-audit-1015 — full audit of Cloud's weaknesses (diagnosis only)

Read `rust/agents/T0-RULES.md` (Hard rules, Keeping context small apply; your branch was made from origin/main).
Handoff `rust/agents/handoff-cloud-audit-1015.md`, full report `rust/agents/report-cloud-audit-1015.md`. Use `-j 4`
builds and at most `--jobs 6` sims; another agent (land-audit) is running.

## The deck
JD's Cloud, Midgar Mercenary (key `cloud`, `decklists/JD/cloud-midgar-equipment.md`): mono-white Equipment, Bracket 3.
Matched tiers (data/my_decks.toml): **T1, T2**. Its win rate is far below an even 25% against every tier — the full
run (4,000 games per cell, look-ahead, loose; per-cell `--analyze` output in `audit/pool/2026-10-14-full/cloud-t*.txt`):
T0 7.3, T1 13.5, T2 12.3, T3 9.4, T4 13.4, T5 13.1.

## Earlier work — build on it, don't repeat it
Read first: `rust/agents/report-cloud-audit-c4.md`, `report-cloud-ai-fix2.md`, `report-cloud-ai-equip.md`,
`report-cloud-ai-fix.md`, `audit/cloud/packages/README.md` (two untested packages: D defense, R removal, with
`.swaps` files), and the Cloud history in `decklists/CHANGELOG.md`. Known from 10/13: slow (first 4+ damage hit
median round 6), Cloud removed ~1.3x a game at T2, T1-T3 losses are combat races, AI fixes gave under a point,
equipment stuck in hand.

## What to do (no batch sims beyond what's listed; JD approves bigger runs)
1. **Where the games are lost**, per matched tier (T1, T2) and against T0 (precons, now fully modeled): from the
   4,000-game analyze files, plus ~20 traced games per tier (`commander-sim --deck cloud --pool t1 --trace N`;
   `--any-tier` for t0). Who kills Cloud's controller and how (combat, drain, combo), on what round; Cloud's
   removals per game and by what; how often the plan comes online and when; mana (lands/rocks by round, colour
   screw); hand clog (cards never cast).
2. **Rules/AI faults still left**: check that the equipment, tutor and protection lines are played right in the
   traces (equip targets, carrier choice, holding up protection, attacking into open mana, blocking decisions —
   Cloud dies to combat races, so check its defence: does it block, chump, keep blockers back?). Any wrong rule or
   card model counts as a finding; fix nothing yet, but say exactly where the code is.
3. **List weaknesses**: what the 99 lacks versus what beats it (interaction count by type, wipes, blockers/defence,
   card draw, ramp, curve). Compare with the pinned D and R packages: which weakness does each address.
4. **Ranked recommendations**: AI/model fixes first (with expected impact), then list changes (packages or single
   cuts/adds; cards must be fully modeled or say what's missing), each with the evidence. Propose a test plan
   (paired swaps, look-ahead, 1,000+ games per tier at T1 and T2) with time estimates — the lead asks JD before running.
5. Small diagnostic sims are fine (up to ~200 games per run, any count of runs) when a trace can't answer a question.

## Done when
`rust/agents/report-cloud-audit-1015.md` holds the findings with numbers and file:line references, committed.
Final report ≤15 lines, line 1 DONE or HANDOFF, line 2 your model ID: the top weaknesses, the top recommendations.
