# Brief: Tier 6 Stage 3a — wire t6 and Godo into the simulator

Read `rust/agents/RULES.md` (memory cap, no Python, commit rules, short reports; ignore the Python-port parts and
the `git merge rust-port` step). Work in `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/t6-cedh`
(branch `t6-cedh`). Commit after each piece (standalone commits, no Co-Authored-By). Don't push.

**Shared machine:** build and test with `nice -n 19`, `CARGO_BUILD_JOBS=6`, and
`systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0`. **Never run a sim** (no `commander-sim`
runs except `--help`); tests only. Every network call gets `curl -m 30` (cardgen's own fetches are fine).

**Progress file:** create `rust/agents/handoff-t6-wiring.md` FIRST, and update it after each step (what's done,
what's next). A lead watches it and stops an agent that writes nothing for 10 minutes.

## Steps

1. **Tier 6 everywhere tiers are listed.** Add `"t6"` to every hard-coded tier list: `sim-cli/src/decks.rs`
   (`TIERS`, `tier_label` → `"Tier 6 (cEDH, B5)"`), `sim-cli/src/args.rs` (the `--pool` choices),
   `practice/src/session/setup.rs`, `practice/src/catalog.rs` (check the `TIERS[1..]` win-rate zip still lines up),
   `cardgen/src/data.rs` (`TIERS`), and the tests that loop over tiers (`sim-core/tests/game.rs`; check
   `cards_yshtola.rs:653` and `cards_t3.rs:445` and decide whether t6 belongs there). `grep -rn '"t5"'` over
   `rust/crates` to find any others. `cardgen/src/decks.rs::gc_rules`: t6 is unrestricted (falls through to None).
2. **Partner commanders for pool decks.** A pool deck's `- **Commander:** A / B` line means two commanders.
   `cardgen/src/decks.rs::load_pool_deck` sets `commander2` from it; validation checks both (it already handles two
   for JD's decks); `export.rs` writes `commander2` for pool decks in `data/decks.json`; check `sim-core/src/run.rs`
   seats a pool deck's `commander2` (it does for JD's decks) and `practice` shows both. Add a test.
3. **Move the decks in.** `git mv decklists/pool/staging-t6-cedh decklists/pool/t6-cedh`. Add a Tier 6 row to the
   tiers table and the six decks to the decks table in `decklists/pool/README.md` (same format as the other rows).
4. **Godo as JD's deck.** Add `[godo]` to `data/my_decks.toml` (file `JD/godo-cedh-equipment.md`, name `Godo`,
   commander `Godo, Bandit Warlord`, ident `R`, tiers `["t6"]`), and to `MINE` in `sim-cli/src/decks.rs` and
   practice's `MY_DECKS` if those lists are hand-maintained. Check `docs` or tests that count JD's decks.
5. **Export and validate.** `cardgen export` (fetches new cards into the Scryfall cache), then `cardgen validate`
   (all decks pass) and `cardgen audit --deck <key>` for each t6 deck and godo. Don't use `--md` (it writes a file).
6. **Tests.** `cargo test --release` passes. `cargo build --release --all-targets` with no warnings. If a test plays
   whole games for every tier and t6 games panic or loop on unmodeled cards, report which card and stop there (don't
   model cards in this task).

## Report

`rust/agents/report-t6-wiring.md`: files changed, the audit table (Full / Full-auto / Approximate / Partial /
Unmodeled per t6 deck and Godo) and the list of Unmodeled and Partial cards across all seven decks. Final message at
most 15 lines.
