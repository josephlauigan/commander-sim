# Brief: AI foundations step 3, Tier 6 card and mana flow, then the Tier 6 fit

Work in the worktree `.claude/worktrees/ai-foundations` (branch `ai-foundations`, after step 2). It is your whole
world. Read `CLAUDE.md` first. Don't use `rust/agents/RULES.md`.

## Why

JD approved three steps: bug fixes (done, `step1-fixes.md`), a line planner (done, `step2-planner.md`), and a Tier 6
recalibration (this brief).

**Where things stand:**
- The planner converts what the draws give it. With the legal-tutor best case, the kill lands on the best-case turn in
  66% of the games where one exists.
- That best case is late: scion's is turn 8, and it exists in only 45% of games.
- At the Tier 6 table, the scored misses (`data/realism/t6.toml`) are:
  - the seat edge: the sim gives 28.6 / 25.7 / 23.7 / 22.1% against a real 31.8 / 27.3 / 22.1 / 18.8%;
  - the deck spread: 10.7 points against 8 or less.
- The median win round is 8 against a real 7, inside its band.

**Not a target:** the casual win-type mix ("combo 47.8%"). JD (2026-10-24): cEDH keeps its own metrics, and that mix
is reference only.

`audit/realism/t6-seat-edge-2026-10-24.md` found the mechanism. The first player to be ready wins 76% of the time from
any seat, but 72% of player-games never have a line ready. Games aren't races, so going first decides little. Real
cEDH games are races. The question for this step is therefore **card and mana flow**: are Tier 6 decks seeing and
using as many cards and as much mana per turn as real cEDH decks do?

Don't fit the seat numbers with seat-specific knobs. Fix flow at its source, then measure the fit.

## Part 1: measure flow (stats only)

At the Tier 6 table (`--pool t6`, the six pool decks, loose) and in the goldfish (`--goldfish t6,scion`), record per
deck and per own turn 1-8:
- cards seen: drawn, plus looked at by cantrips and tutors;
- cards cast;
- mana available and mana spent;
- life paid for cards (Necropotence, Ad Nauseam);
- land drops made and missed.

Split cards drawn by source: draw step, cantrip, engine (Rhystic Study, Mystic Remora, Esper Sentinel, Sylvan
Library, Necropotence, The One Ring, Ad Nauseam), wheel, and commander (Kraum, Tymna, Thrasios, Dack's loots).

Then build a **reference by hand** for what a competent pilot of each list would see by turns 2, 3 and 4. Use the
list's own cantrips, engines and fast mana. Show the arithmetic and the assumptions: what real opponents do that feeds
Rhystic Study and Remora, and which engines a pilot casts on turns 1-2. The arithmetic is the reference; there is no
public dataset.

Compare the two, and list each engine or card class that under-delivers, with the size of the gap.

## Part 2: find out why, per under-delivering engine

For each large gap, read the card's model (`cardgen audit` status, its DSL or hand code) and the AI's use of it. Then
trace a few games and classify the cause:
- **(a) the card's rules model** is wrong or missing: Rhystic Study's tax decision, Mystic Remora's upkeep and its
  noncreature trigger, Necropotence's life-for-cards, Sylvan Library, wheels;
- **(b) the AI** doesn't cast it early or doesn't use it: it pays too little life to Necropotence, or never keeps
  Remora;
- **(c) the opponents' AI** doesn't feed it as real players do: whether to pay the Rhystic tax, and spell counts per
  turn;
- **(d) the list** has it, but the model is Partial or Unmodeled.

Check every card's Oracle text against `data/scryfall_cache.json` before you change its model.

## Part 3: fix and measure

Fix the causes in order of size, one commit each. Use real rules modelling or the AI's real decision, not a bonus
constant. Tier 6 only when the change is AI behaviour; for every deck when it's a card's rules model (rules apply to
everyone). After each commit:
- run `--realism --pool t6 --games 4000`;
- run the goldfish;
- record seat shares, deck spread, median win round, and the Part 1 flow numbers.

At the end run the full set: `--realism` t0-t6 and `--all-decks --pool all --games 1000`, against the step 2 end
state. Explain any lower-tier or JD-deck move beyond noise; a rules-model fix touches every deck that plays the card.

Stop when the seat edge and the spread are in band, or when what's left isn't flow. In the second case, write down
what it is: for example the tournament clock or the AI's threat assessment. That's an acceptable outcome.

Write it all up in `audit/ai-foundations/step3-flow.md`, with the raw output in `audit/ai-foundations/raw/step3/`.

## Rules

- **Commands.** Run every build, test and sim from `rust/` with `run_in_background`, under
  `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 ...`. Use a detached
  `systemd-run --user --unit=step3-<name> ...` for anything over about 10 minutes, and poll its output files. Never use
  one blocking command longer than 10 minutes.
- **Threads.** Use `--jobs 16` and `-j 16`, and run one sim or cargo command at a time.
- **No Python, ever**, in any position in a command. Not even `python3 --version`, and not after `cd` or `&&`.
- **Shared files.** Mark each change `// --- ai-flow`. Comments explain why, briefly, in the surrounding style.
- **Formatting.** Format only the files you write (`rustfmt --edition 2024 <file>`), never `cargo fmt`. Lines go up
  to 120 characters.
- **Commits.** One commit per fix. No Co-Authored-By line. Never push. Keep `rust/agents/handoff-ai-t6-flow.md` up to
  date and commit it with your work.
- **Before you finish:** `cargo test --release --no-fail-fast` passes, `cargo build --release --all-targets` has no
  warnings, and nothing of yours is still running (`pgrep -af 'commander-sim|cargo'`).
- **Decisions.** JD may be away. Make the conservative call, record it with the alternative in a "Decisions" list, and
  keep going.
- **Report** in your final reply, in 15 lines or fewer: the flow gaps found, the fixes, the Tier 6 scorecard before
  and after, the lower-tier and JD-deck moves, and anything JD must decide.
