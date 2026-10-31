# Brief: Ghave, Guru of Spores replaces Seph at t3-t4

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/ghave-wire` on branch
`ghave-wire`, from origin/main (with #181 `ghave-deck` merged). Run every command from there. Read `CLAUDE.md` first:
memory cap, no Python, loose profile, opponent tiers, "Check every swap". **Two other agents run sims in parallel**
(Phase 4 PR 5b in `phase4-search-b`; card models in `card-models-1`): use `--jobs 8` for sims and `-j 8` for cargo, one
sim of yours at a time, in the background; check rc=0 on every run you cite.

## JD's decision (2026-10-30)

- **Retire the `seph` seat; Ghave takes it.** Ghave, Guru of Spores becomes JD's deck at **t3-t4** (Seph's tiers).
  The list is `decklists/JD/ghave-abzan-aristocrats.md` (written by the ghave-deck PR, #181; read it all, including its
  Key lines). Move the Seph doc (`decklists/JD/sephiroth-phyrexian-reanimator.md`) to `decklists/pool/retired/`.
- **The precedent** is the Scion -> Thanos change (PR #177, merged; look at its commits with `git log --grep Thanos`
  and `git show` them): rename the deck key, keep the deck's seat, plan file, tests and wiring working, and keep card
  code where it models a card (Sephiroth, Fabled SOLDIER is in Ghave's list too).
- **No new per-deck Rust AI.** Ghave plays from a plan file (`data/plans/ghave.toml`: the combo lines, key cards,
  knobs, `[style]`) through the generic policy. Phase 4's last step removes per-deck AI code, so Seph's hand AI
  (`ai/seph.rs` and its hooks) should go now, if nothing else uses it once the seat is Ghave's; if removing a hook
  needs an edit in `ai/brain.rs`, keep it to the deletion and list it (PR 5b is editing that file too).

## Work

1. **Survey:** every place `seph` is wired: sim-core (`ai/seph.rs`, `ai/decks.rs`, `ai/mod.rs`, `ai/pool.rs`,
   `ai/brain.rs`, `cardcode.rs`, engine call sites), sim-cli (args.rs, decks.rs, goldfish.rs, knobs.rs, main.rs,
   poolmode.rs, report.rs, README), the practice crate (catalog.rs, session/setup.rs, view.rs and its tests), cardgen
   (data.rs), `data/my_decks.toml`, `data/plans/`, `data/spellbook/`, and the tests (`git grep -il seph`). Write the
   list in `audit/ghave/handoff.md` and decide, per site, rename / delete / keep (card code).
2. **Check the cards** (CLAUDE.md "Check every swap"): commander legality and colour identity (W/B/G) of all of
   Ghave's list (`data/scryfall_cache.json`), and each card's model status (`cardgen audit`). Model what is missing or
   Partial for the deck's lines, through the ability language first; Rust card code only for real one-offs.
   - **Ghave:** enters with five +1/+1 counters; {1}, remove a +1/+1 counter from a creature you control: a 1/1 green
     Saproling; {1}, sacrifice a creature: a +1/+1 counter on target creature. Read the Oracle text from the cache.
   - **The lines** (from the doc): Ghave + Ashnod's Altar or Phyrexian Altar + Cathars' Crusade or Doubling Season
     (infinite Saprolings / death triggers / mana); check each piece, and the payoffs (Blood Artist, Zulaport
     Cutthroat, Cruel Celebrant, Sephiroth, Grave Pact, Pitiless Plunderer, Yawgmoth, Skullclamp, Aura Shards, Karn's
     Bastion, Mikaeus, Triskelion). Ghave's activations are AI decisions: when to make Saprolings, what to sacrifice
     and when (in response to removal, for Skullclamp, for Grave Pact): make them generic (ability options with
     utilities), not deck-specific code.
   - A combo line must be found by the plan file and the existing combo machinery; add it to
     `data/spellbook/ghave.json` if the repo's spellbook tool does that (credit Commander Spellbook).
3. **Rename** seph -> ghave (key, deck file entry, tests, CLI help, report labels, the practice catalog). Tests that
   test Seph's hand AI go with it; tests of card behaviour stay (seat them in a ghave or neutral setup).
4. **Docs:** CLAUDE.md "Opponent tiers" ("sheoldred, sauron and ghave t3-t4 (band 2; ghave 10/30: Seph's seat)"),
   README, decklists/CHANGELOG.md, and anything else listing JD's decks. Run `cardgen export` and `cardgen validate`;
   regenerate generated files with the tools, never by hand.
5. **Sims (loose, 2000 games, seeds 500000-501999, rc=0):** `--deck ghave --pool t3` and `--pool t4`, plus a goldfish
   run (kill turn, how it wins). For information, put Seph's last numbers beside them (run `--deck seph` on t3 and t4
   on origin/main before the rename, same seeds). Write `audit/ghave/FINDINGS.md` (strengths, weaknesses, how it wins
   and dies, model status, next to try) and move `audit/seph/` notes' still-true parts there only if they apply.

## Checks

- `cargo test --release --no-fail-fast -j 8` passes (all crates, practice included); `cargo build --release
  --all-targets -j 8` 0 warnings; `cardgen validate` passes; the plan-file test passes.
- Unit tests for Ghave's two abilities and one assembled line (Ghave + altar + Cathars' Crusade loops and drains with
  Blood Artist; the loop stops, no hang).
- JD's decks seated only in their own tiers; don't seat `omnath` or `godo` in new tests.

## Rules

- Stay out of Phase 4's files other than the deletion above: `ai/search.rs`, `ai/ismcts.rs`, `ai/learned.rs`,
  `ai/values.rs`, `data/eval/`, `audit/phase4/`, `data/values_overrides.toml`. If a rename forces a one-line change in
  one of them, make it and list it.
- The card-models agent (`card-models-1`) is modelling Partial cards in JD's decks (Glimpse the Unthinkable for
  kefka, Treasure Vault, Venat, Buster Sword and Thanos/Ur-Dragon cards): don't model those.
- No recalibration. No changes to other decks' lists.
- Format only new files (`rustfmt --edition 2024 <file>`), lines up to 120. Commit often, no Co-Authored-By line,
  never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/ghave-wire/`.
- Keep `audit/ghave/handoff.md` current.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report and FINDINGS
  files from subagents: give the FINDINGS text in the reply if a write is refused): what was renamed, deleted or moved,
  the card checks and models, the sim numbers beside Seph's, and what is still Partial or Unmodeled.
