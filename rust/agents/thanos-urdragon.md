# Brief: Thanos replaces Scion; Ur-Dragon joins as a sim deck

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/thanos-urdragon` on branch
`thanos-urdragon`, stacked on `thanos-cedh` (PR #175: the Thanos deck doc, not merged yet). Run every command from
there. Read `CLAUDE.md` first: memory cap, no Python, loose profile, opponent tiers, "Check every swap". **Another
agent is running sims in parallel** (Phase 4 PR 5, worktree `phase4-search`): use `--jobs 12` for sims and `-j 12` for
cargo, one sim of yours at a time, in the background; check rc=0 on every run you cite.

## JD's decisions (2026-10-30)

- **Rename the `scion` deck key to `thanos`.** Thanos, the Mad Titan becomes JD's cEDH deck at **t6**, taking over
  Scion's seat, its plan file, its tests and its wiring. The list is `decklists/JD/thanos-five-colour-turbo.md`
  (Scion v1 minus Scion, Ancient Copper Dragon, Korvold and Prismari; plus Fierce Guardianship, Deflecting Swat and
  Deadly Rollick). Move the Scion list doc (`decklists/JD/scion-ur-dragon-turbo.md`) to `decklists/pool/retired/`. Keep
  Scion's own card code (its ability, `cards_scion.rs` and the like) as card code where it models a card.
- **Add The Ur-Dragon** as a JD sim deck at **t5**. The list is `decklists/JD/ur-dragon-equipment.md` (Bracket 4,
  17 Game Changers). Its main line is Godo, Bandit Warlord + Helm of the Host (infinite combats). The backups are
  Combat Celebrant + Helm, and Dualcaster Mage + Twinflame. Eminence and the attack trigger are already in
  `impls/t45_urdragon.rs` (PR #139).

## Work

1. **Survey:** every place `scion` is wired, including catalog.rs, setup.rs, sim-cli (decks.rs, args.rs,
   goldfish.rs, poolmode.rs, report.rs), ai/mod.rs, t6stats.rs, impls/t6_scion.rs, impls/t6_combos_scion.rs,
   impls/t6_plans.rs, data/plans/scion.toml, data/spellbook/scion.json, data/my_decks.toml and the tests. The
   precedent for retiring a deck is commits 79d92e6f, 74fbc0f5 and 2c2a08e9 (Sephiroth FS).
2. **Check the cards** (CLAUDE.md "Check every swap"): commander legality and colour identity for the three cards in
   and for all of Ur-Dragon's list (`data/scryfall_cache.json`). Report each card's model status (Full ...
   Unmodeled). Model what is missing or Partial for these lists, through the ability language first; Rust card code
   only for real one-offs.
   - **Thanos's power-up:** {C}{W}{U}{B}{R}{G}, once per game, costing {C}{U}{G} on the turn he entered. It puts two
     +1/+1 counters on him; you choose odd or even, then destroy each other creature with mana value of that parity
     (0 is even). It is an activated ability with no sorcery restriction. Read the Oracle text from the cache. The
     choice of parity and the timing are AI decisions: compare what each parity destroys (opponents' value and
     threats against your own creatures) generically, not with a hard-coded "even".
   - **Fierce Guardianship, Deflecting Swat and Deadly Rollick:** "free if you control a commander" must check a
     commander on the battlefield.
   - **Ur-Dragon:** check Godo, Helm of the Host, Combat Celebrant (exert), Gilgamesh, Master-at-Arms, Maskwood Nexus,
     Hammer of Nazahn, Kaldra Compleat, Gaea's Cradle, Kindred Dominance and the Equipment, plus anything else
     `cardgen validate` or the model status flags.
3. **Rename** scion → thanos (key, deck file entry, tests, CLI help, report labels). Rename the plan file to the
   Thanos deck key and drop the Prismari, Copper and Korvold lines. Rename `data/spellbook/scion.json`, refreshing it
   from Commander Spellbook if the repo's tool does that. Keep t6_plans' slots working (the plan dispatch).
4. **Ur-Dragon:** add a deck key in `data/my_decks.toml` (`tiers = ["t5"]`) and a plan file (`data/plans/<key>.toml`:
   the combo lines, key cards, knobs). **Don't write new per-deck Rust AI.** Phase 4's last step removes that code, so
   new decks start on plan files. Rust code is only for card models.
5. **Docs:** update the deck lists in CLAUDE.md "Opponent tiers" ("thanos t6 (cEDH)", "ur-dragon t5 (band 1)"),
   README, decklists/CHANGELOG.md, and anything else listing JD's decks. Run `cardgen export` and `cardgen validate`.
6. **Sims (loose, 2000 games each, rc=0):** `--deck thanos --pool t6`, and Ur-Dragon on t5. Also goldfish runs
   for both: kill turn and how the deck wins. For information only, put Thanos's t6 share beside Scion's last number
   (30.1% on `--deck scion --pool t6`, seeds 500000-501999). Write `audit/thanos/FINDINGS.md` and
   `audit/<ur-dragon key>/FINDINGS.md` (strengths, weaknesses, how it wins and dies, unmodelled cards).

## Checks

- `cargo test --release --no-fail-fast -j 12` passes; `cargo build --release --all-targets -j 12` 0 warnings;
  `cardgen validate` passes; the plan-file test passes.
- Unit tests for the Thanos power-up (odd and even, 0 counts as even, once per game, the reduced cost on the turn he
  entered) and for the "free with a commander" cards.
- Don't seat `omnath` or `godo` (the pool deck) in new tests.

## Rules

- Stay out of Phase 4's files: `ai/search.rs`, `ai/learned.rs`, `ai/values.rs`, `data/eval/`, `audit/phase4/`,
  `data/values_overrides.toml`. If a rename forces a one-line change in one of them, make it and list it.
- No recalibration. No changes to other decks' lists.
- Format only new files (`rustfmt --edition 2024 <file>`), lines up to 120. Commit often, no Co-Authored-By line,
  never push. Scratch files go in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/thanos-urdragon/`.
- Keep a handoff (`audit/thanos/handoff.md`) current.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report files from
  subagents): what was renamed or moved, the card checks and models, the sim numbers, and what is still Partial or
  Unmodeled.
