# Brief: the `scion` deck key, a goldfish mode, and Scion of the Ur-Dragon's dragon line

Work in a git worktree of this repository on a new branch `scion-goldfish` from `scion-goldfish-brief`, the branch
that holds this brief and the v1 list
(`git worktree add .claude/worktrees/scion-goldfish -b scion-goldfish scion-goldfish-brief`). It is your whole world:
run every command from there. Read `CLAUDE.md` first (memory cap, no Python, loose profile, check every card). Do not
use `rust/agents/RULES.md`.

## Why

JD is designing a cEDH turbo deck around Scion of the Ur-Dragon (five colours). Its first design decision (D1) sets a
speed target the simulator must measure:

- a median goldfish kill (no opponents interacting) by the deck's turn 3, about 30% of games by turn 2 and about 70%
  by turn 3;
- at least one protection card held on the kill turn in about 60% or more of attempts.

The simulator can't measure any of this today:

1. There is no goldfish mode: every run seats three pool decks.
2. There is no `scion` deck key.
3. Scion's own ability isn't modeled. `impls/t45_urdragon.rs` models only its Rivaz use (a Dragon into the graveyard;
   "Left out: Scion becoming a copy"). Prismari, the Inspiration is not in `data/cards.json`. Ancient Copper Dragon's
   Treasures are card code (`copper_damage`) that fires only when the attacker is the Copper Dragon permanent itself.

This task adds all three and measures the baseline. It does not design the deck: JD makes those decisions (D2-D12).

## Part 1: goldfish mode (`sim-cli`)

1. **A flag**, `--goldfish` (your call on the name; add it to the `args.rs` help and the README's main commands). The
   deck plays alone against three inert seats: 40 life, a library large enough not to run out in 20 rounds, and they
   never cast, activate, attack, block or respond. Turns still pass in the normal order so round counting is real.
   The deck under test is always seat 1.
2. **Which decks:** JD's deck keys and the six Tier 6 pool keys (for reference numbers). No tier limit applies: there
   are no opponents to draw.
3. **Same deck behaviour:** the deck's own mulligan rule and AI (`--ai lookahead` by default, `adaptive` works), the
   same seed streams as today (`lib:{seed}:{key}`), so the same seed gives the same result.
4. **The report**, per deck:
   - **kill turn:** the deck's own turn number on which a combo resolves or the last opponent loses. Median, mean, the
     cumulative share of games won by turn 1, 2, 3 ... 8, and the share with no kill by the round cap;
   - **how it won:** the share by combo name, and by damage;
   - **mulligans:** the share of games kept at 7, 6, and 5 or fewer cards;
   - **ready turn:** the first turn a combo is ready, as `--analyze` measures it;
   - **protection on the kill turn:** the share of wins where, when the deck went for the win, it held at least one
     protection card in hand or had a cast lock in force (Silence or Orim's Chant cast that turn, Grand Abolisher on
     the battlefield). Define protection from the existing tags (counterspells and protection, `sim-core/src/tag.rs`)
     plus those three cards, and list every card that counts in the report.
5. **Tests:** the inert seats never act (no action by seats 2-4 in a goldfish game's log); the same seed gives the
   same output.

## Part 2: the `scion` deck key

1. **The list:** `rust/agents/scion-v1-list.txt` (lines starting `//` are comments). It is v1, the consensus list
   built from EDHTop16 data, and the baseline every later version is measured against. Write
   `decklists/JD/scion-ur-dragon-turbo.md` in the layout of an existing JD deck file (`## Import list`, the
   `- **Commander:**` line, `**Game Changers (N)**` with the count `cardgen` reports). Keep its strategy text short:
   it is the v1 baseline, and JD's design document holds the plan. Check legality and colour identity with
   `cardgen deck` (`--dry-run` first), then `cardgen export`. Tinder Wall and Prismari, the Inspiration are not in
   `data/cards.json` yet; export fetches them. Check every card's model with `cardgen audit --deck scion` and list the
   ones that are not Full in the report.
2. **`data/my_decks.toml`:** `[scion]` with file `JD/scion-ur-dragon-turbo.md`, name `"Scion"`, commander
   `"Scion of the Ur-Dragon"`, ident `"WUBRG"`, tiers `["t6"]`.
3. **Wire the key the way `godo` was wired before its retirement.** `report-retire-omnath-godo.md` lists every place
   it was removed from, and `git log -S'"godo"' -- rust/crates/sim-cli/src/decks.rs` finds when it was added. That
   includes `sim-cli/src/decks.rs` (FILES, MINE), `args.rs` (DECK_KEYS), practice (`catalog.rs`, `session/setup.rs`),
   `ai/mod.rs` styles (a turbo style; say which values and why), `settings.rs` and `impls/rules.rs` counter thresholds,
   `impls/t6_plans.rs` (T6_KEYS and the dispatch), `t6stats.rs`, and the tests that count JD's decks (12 to 13).
4. **`impls/t6_plan_scion.rs`**, under the slot contract in `t6_plans.rs`:
   - `keep` starts from the Rograkh/Silas turbo keep rule (`t6_plan_rograkh_silas.rs`), with a comment that it is
     provisional until JD decides mulligans (D11);
   - `prio`, `wish`, `hold_up` and `go_now` return `None`, except where Part 3 needs a rule for Scion.
5. **Docs:** the opponent tiers line in `CLAUDE.md`, the tier tables in `README.md` and `documents/testing-decks.md`,
   and the Tier 6 deck list in `documents/how-it-works.md`.

## Part 3: Scion's dragon line

The Oracle text (check each against `data/scryfall_cache.json`):

- **Scion of the Ur-Dragon:** "{2}: Search your library for a Dragon permanent card and put it into your graveyard. If
  you do, Scion of the Ur-Dragon becomes a copy of that card until end of turn. Then shuffle." Rulings: once it is a
  copy, it has lost this ability for the turn (activations already on the stack still resolve, so several can be
  stacked first); removing the card from the graveyard does not end the copy. There is no {T} in the cost, so it works
  the turn Scion enters.
- **Prismari, the Inspiration:** a 7/7 flying Elder Dragon with ward (pay 5 life); "Instant and sorcery spells you
  cast have storm."
- **Ancient Copper Dragon:** whenever it deals combat damage to a player, roll a d20 and create that many Treasures
  (card code exists: `copper_damage` in `t45_urdragon.rs`).
- **Korvold, Fae-Cursed King:** modeled (attack, enters and sacrifice hooks).

Do:

1. **"Becomes a copy until end of turn"** for a permanent already on the battlefield. Reuse the `CopyOf` / `enter_copy`
   machinery in `impls/t6_copies.rs` where you can. The copy ends at cleanup. While Scion is a copy, the copied card's
   hooks and static abilities apply to it (Copper Dragon's Treasures, Korvold's draws, Prismari's storm), and Scion's
   own ability is gone until the copy ends.
2. **Prismari's storm:** while Prismari (or Scion as Prismari) is on the battlefield, your instants and sorceries have
   storm. A card that already has storm (Brain Freeze) gets a second storm trigger.
3. **The Scion AI:** when to activate and which Dragon to fetch (the list holds Ancient Copper Dragon, Korvold and
   Prismari). Keep the rule simple and explain it in the plan file.
4. **The lines:** from the Oracle text, list every win the Scion package enables, with its exact sequence and mana.
   Add each real kill to the combo catalogue in the existing style (`impls/t6_combos_*.rs`), with a test. A line the
   cards don't support stays out, and an activation that only makes mana or cards is an engine, not a combo. The
   existing Breach + Lion's Eye Diamond + Brain Freeze and Thassa's Oracle lines are already catalogued; check that
   the `scion` seat finds them.
5. **Tests**, one per behaviour: the copy and its end at cleanup; the lost ability; stacked activations; Prismari's
   storm on a ritual; Brain Freeze's two storm triggers; Copper Dragon's Treasures when Scion is the copy; Korvold's
   draws when Scion is the copy; the ward on Prismari.

## Measure

Record everything in a new `audit/scion/FINDINGS.md` (the deck's living findings file, like the others) with raw
output in `audit/scion/<date>-baseline/`:

1. Goldfish, 2,000 games, seed 500000, look-ahead AI: `scion` and the six Tier 6 pool decks (their kill turns are
   the reference for the clock to beat).
2. The same for `scion` with `--ai adaptive`, so later quick reads can be compared.
3. `commander-sim --deck scion --pool t6 --games 2000` (loose, look-ahead). Label it comparative only, as
   `audit/godo/FINDINGS.md` does: the Tier 6 table wins at a median round of 9 against about 7 in real games, so it
   flatters fast decks.
4. In the goldfish games: how often each line won, how often Scion was cast, and how often it was activated (into
   which Dragon).

## Out of scope

- No list changes beyond v1, and no tuning of the keep rule, priorities or table AI past the starting points above.
- The two hate scenarios in JD's design document (a counter on the first combo piece; Drannith Magistrate in play) are
  a later task. Say in the report what the goldfish mode would need to support them.

## Rules

- **Commands.** Run every build, test and sim from `rust/`, under the memory cap in `CLAUDE.md`.
- **Thread limits.** Pass `--jobs 16` to every `commander-sim` run and `-j 16` to cargo (lower both on a machine with
  fewer cores). Run one sim or cargo command at a time, and long commands in the background.
- **No Python**, not even inside a longer command.
- **Shared files** (`engine/*`, `ai/*`, `state.rs`, `hooks.rs`, `combos.rs`, `t6_copies.rs`, `t45_urdragon.rs`): make
  the smallest additive change, mark it `// --- scion`, and list each one in your report. Don't change how existing
  cards behave for the decks that already play them (the Tier 4/5 Ur-Dragon pool deck plays Scion and Copper Dragon).
- **Formatting.** Format only the files you write (`rustfmt --edition 2024 <file>`), never `cargo fmt`. Lines go up to
  120 characters.
- **Commits.** Commit after each part, with no Co-Authored-By line, and never push. Keep
  `rust/agents/handoff-scion-goldfish.md` up to date (done, next, gotchas) and commit it with your work.
- **Before you finish:** `cargo test --release --no-fail-fast` passes, `cargo build --release --all-targets` has no
  warnings, and `cardgen export` leaves `data/` unchanged.
- **Report** in your final reply, 15 lines or fewer, with details in `rust/agents/report-scion-goldfish.md`:
  - the goldfish numbers for `scion` and the six Tier 6 decks (median kill turn, % by turns 2 and 3, protection share);
  - each Scion line found, with its sequence and whether it is catalogued;
  - the cards that are not Full, and the shared-file changes;
  - anything JD must decide.
