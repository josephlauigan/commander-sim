# Task: cloud-sequencing — does the AI cast ramp and tutors too late? (diagnosis only)

Read `rust/agents/T0-RULES.md` (Hard rules, Keeping context small). Branch cloud-mana. `-j 16` builds, `--jobs 16`
for ≤500-game diagnostic runs. Report `rust/agents/report-cloud-sequencing.md`. **Change no code or lists.**

## Question
In Cloud's mana package tests (audit/cloud/2026-10-15/mana/, `audit/cloud/FINDINGS.md`), cheap ramp and tutors were
first cast around round 7: Worn Powerstone avg round 6.8 (stuck 28%), Burnished Hart 8.8, Stoneforge Mystic 7.1, and
mana on turn 4 didn't move. Is that because the cards are drawn late (fine), or because the AI holds them while
casting other things (a fault)?

## How
1. Measure **draw-to-cast delay**: for each mana card (Sol Ring, Arcane Signet, Mind Stone, Fellwar Stone, Thought
   Vessel, Worn Powerstone, the land-ramp creatures/spells in the list, Stoneforge Mystic, Smothering Tithe) the turn
   drawn (or in the opening hand) vs the turn cast, and what was cast instead on the turns it waited, with the mana
   available. Use traces (`commander-sim --deck cloud --pool t1 --trace N --swap ...` with the M package swaps from
   `audit/cloud/packages/m-mana.swaps` so the new cards are in the deck; ~30 games), and a small instrumented count if
   a trace can't give it (a temporary local change you don't commit is fine; say so).
2. Find **where the priority comes from**: cloud's priorities (`impls/cloud.rs` `cloud_prio` and friends), the generic
   cast ordering (`ai/`), and the look-ahead's choice. Does ramp score below Equipment/creatures on turns 2-4? Is
   there a "ramp early" rule other decks have (grep: other JD decks' prio for rocks) that Cloud lacks?
3. Report: the numbers (median delay per card, % of games a rock waited ≥2 turns with mana for it), the cause with
   file:line, and the smallest fix (gated to cloud) with the expected effect. Also note any other sequencing fault you
   see (e.g. equipping vs casting, holding mana).

Final report ≤15 lines: line 1 DONE or HANDOFF, line 2 model ID, findings and proposed fix.
