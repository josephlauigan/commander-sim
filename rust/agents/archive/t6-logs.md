# Brief: Tier 6 Stage 5 — annotated game logs (why is Tier 6 slow?)

Tier 6 decks win at median rounds 7-12; real cEDH decks win around turns 3-5 (their specs say when). Godo, which kills on
turn 3 with Helm of the Host, wins ~50% because the table is slow. Your job: read real games and find out, deck by
deck, WHY each deck didn't win sooner. You change no code. Your annotations decide what gets fixed next and are the
Stage 5 log review JD reads, so write them for a reader who knows Magic but not this code.

Read `rust/agents/RULES.md` (no Python, memory cap), each deck's spec `audit/t6/<deck>/SPEC.md` ("Game plan", "Win
lines", "Turn by turn", "When it goes for the win") and its plan file `rust/crates/sim-core/src/impls/t6_plan_<key>.rs`.

## How

From `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/t6-cedh/rust` (the binary is built; do not build,
do not edit any file outside your own output files, do not commit):

    nice -n 19 ./target/release/commander-sim --calibrate within --pool t6 --trace N > /tmp/claude-1000/t6logs/gN.txt

N is the game number from your prompt. The log shows each turn's plays, the AI's choices with their scores
(`[X decides: A 80%, B 20% -> A]`), stack fights and eliminations. For card details use
`./target/release/commander-sim --cards` (needs --deck) or `data/cards.json` with `jq`.

## What to write, per game: `audit/t6/stage5-logs/game-NN.md` (NN two digits)

1. Header: seats in order, winner, how, round.
2. Per deck, a short paragraph: what it did on its key turns; the EARLIEST turn it could have won or gone for its line
   following its spec, given its hand and mana as the log shows them (be concrete: "turn 3: had Thassa's Oracle,
   Demonic Consultation and 4 mana; cast Rhystic Study instead"); and why it didn't, with ONE OR MORE cause tags:
   - `MULL` keep or mulligan choice wrong for the spec
   - `MANA` mana unused, wrong sequencing, fast mana held, land drop missed
   - `TUTOR` tutor fetched the wrong card, or a tutor not cast
   - `HOLD` the line was available and the AI chose to wait (`go_ok`/`go_now` refused, held for protection)
   - `NOLINE` the line was assembled per the spec but the AI never offered it (combo not recognised, `ready` false)
   - `CHOICE` the look-ahead or priorities picked a worse play (quote the decide line)
   - `STOPPED` a fair answer from an opponent stopped it (say what)
   - `RULES` the engine did something wrong by the rules (say what, with the card's Oracle text)
   - `SPEC` the spec's line needs something the deck didn't have this game (fair variance)
3. Anything that looked wrong even if it didn't matter (tag `ODD`).

Keep each game file to about a page. Quote log lines exactly when you cite them.

## Your summary: `audit/t6/stage5-logs/summary-<slice>.md`

A table: deck x cause tag with counts over your games, and the 3-5 most important concrete problems you found, each
with the game and log line that shows it and what fixing it would look like (which slot, rule or card code).

## Progress

Write `/tmp/claude-1000/t6logs/progress-<slice>.txt` after each game (one line: game done, main finding). A lead watches
it and checks on you after 12 silent minutes. Timebox: about 10 minutes per game; final message 10 lines or fewer
(your top problems).
