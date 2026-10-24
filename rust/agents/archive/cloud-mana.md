# Task: cloud-mana — design a mana-base upgrade package for Cloud (design only, no batch sims)

Read `rust/agents/T0-RULES.md` (Hard rules, How cards are modeled, Keeping context small). Branch cloud-mana (= PR #71:
Cloud with AI fixes + Package D/substitutes, and the new `--swap` breakdown). `-j 16` builds, `--jobs 16` smoke runs.
Report `rust/agents/report-cloud-mana.md`.

## Why (read `audit/cloud/FINDINGS.md` first)
Cloud's bottleneck is **mana and deployment**: hand ~6.2 cards after the turn-4 land drop, ~5.1 at turn 8; mana ~4.0 on
turn 4 and ~7.1 on turn 8; cheap cards stuck in hand 20-30%; every Equipment costs mana twice (cast + equip); the
plan (equipped Cloud attacking) is online by turn 4 in ~26% of games. Cutting mana rocks or bodies always cost more
than the new cards added (packages S, B, R). Two slots are dead: Unbreakable Formation (cast in 1% of the games it's
drawn) and Ultimate Magic: Holy (11%).

## What to design
A **mana package M** (JD: "it can be more than 2 cards"): rocks, land ramp, cost reducers for colourless/Equipment
spells, mana-efficient lands, and/or +1 land — whatever the data says will get Equipment cast and equipped sooner.
- Look at the current mana base first: lands (count, which), rocks, ramp creatures/spells, and their numbers in a
  breakdown (run a no-op-ish baseline `--analyze` at T1, ≤500 games, to read cast%/stuck%/round for every mana card).
- **Cuts:** the two dead slots first, then the weakest remaining non-creature cards by the data (low cast%, high
  stuck%, little impact). **Don't cut creatures, Package D cards, Cloud's core Swords/Kaldra, or existing rocks.**
- Respect Bracket 3: the deck has 2 Game Changers (Smothering Tithe, Enlightened Tutor); a third is allowed only if it
  clearly earns it — flag any GC (e.g. Ancient Tomb, Mana Vault, Gaea's Cradle-style) explicitly.
- Prefer cards JD owns (`bulk-cards.txt`, `binder-cards.txt`, and not in another deck: grep decklists/JD and
  decklists/Avery), then cheap buys (Scryfall batch collection POST for prices, never one-by-one).
- Every card must be **Full** in `cardgen audit --cards`, and the `--swap` pre-flight must show no WARNING; model any
  strong candidate that isn't (from Oracle text, with a test) or drop it and say so.
- Produce **M** (the main package) and, if the data supports it, **M-lite** (the cheapest/owned subset), as
  `audit/cloud/packages/m-mana.swaps` (and `m-lite.swaps`), plus a README table (card, owned or price, why, what it
  replaces).
- Run each package once as a **smoke test** (`--swap ... --games 200 --pool t1`) only to confirm it loads and the
  pre-flight is clean. No result claims from it.

Commit (no Co-Authored-By), never push. Report ≤15 lines: line 1 DONE or HANDOFF, line 2 model ID, the package(s) with
swaps and cost, and a proposed test (paired, look-ahead, 2,000 games per tier, T1 then T2 only if T1 gains +3).
