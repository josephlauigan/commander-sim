# Phase 2 doubles: handoff (paused 2026-10-26, evening)

Brief: `rust/agents/phase2-doubles.md`. Branch `phase2-doubles`, not pushed.

## Done (committed)
- 34a92a6a: the doubles. Cost hooks deleted (Ruby/Pearl/Jet Medallion, Foundry Inspector, Etherium Sculptor, Starfield
  Mystic: compiled static:cost alone); toxic combat-damage hooks deleted (Dune Mover, Paladin of Predation, Ribskiff,
  Venomous Brutalizer) and `HOOK_TOXIC` removed (toxic_hit gives compiled toxic for all); Snap's hand resolve deleted
  (compiled spell). `dsl = []`: Laelia (compiled attack draw ran on top: two cards per attack), Liliana Waker of the
  Dead (compiled partial +1 competed with the walker), Oath of Teferi (compiled exile never returned), Heliod's Pilgrim
  (replaces the load-time `data_fixes` in sim-core cards.rs). `paths_ok`: false positives Allosaurus Shepherd,
  Hexing Squelcher (self vs your spells), Contaminant Grafter (combat_damage code is proliferate), Tataru Taru (you
  draw / opponent draws), Twilight Diviner, Daxos, Elemental Mascot, Static Net (different triggers); splits after
  deletion: Starfield Mystic, Ribskiff, Venomous Brutalizer. The 18 shadowed dsl-covers spells keep their compiled
  spell, declared `paths_ok`: it never resolves but is the AI's value (dsl::card_value) -- a diagnostic showed
  Liliana's Triumph, Vona's Hunger, Kaya's Wrath, Calamity's Wake, Erode and seph's Open the Armory are cast only on
  that value, so `dsl = []` would stop the AI casting them. ai/pool.rs: a compiled cost static ranks 68/52 like the
  old cost hook (keeps the Medallions' priority; also lifts Emerald and Sapphire Medallion). Tests:
  `sim-core/tests/paths_doubles.rs` (exactly-once per card) and cardgen `tests/paths.rs` rewritten.
- be11c4f0: `audit/phase2/paths-after-doubles.md`: hard 21 -> 0, dsl-covers 28 -> 20 (8 deletions), split (paths_ok) 29.
- d6e8b034, 2afe37db: memory fix (below). Full `cargo test --release --no-fail-fast`: 3524 passed, 0 failed;
  `cargo build --release --all-targets`: no warnings.

## The seed 500162 investigation (t5 exit 137)
`--realism --pool t5` ran out of memory on the base binary too (seeds 500162 and one in 500300-399). gdb with
`prlimit --as` showed `mana::mana_units` pushing one Unit per Treasure (then per floating mana): Miirym's board had
5 Thrakkus copies (double Dragons' power), 5 Old Gnawbone copies (Treasures = damage) and 3 Roaming Thrones, so
Treasures reached billions. Fix: at most 1000 Treasure units and 1000 floating units per kind in mana_units,
saturating Treasure counts and token multiplication. With it t5 runs. Still open: `--deck yshtola` vs t5 logs 3
"attempt to add/multiply with overflow" panics (seeds 500493, 500788, 501138, before run) -- likely the same boards
(damage / power arithmetic); not fixed.

## Measurements (scratchpad, kept)
`/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase2-doubles/`: `before/` = base binary + base data snapshot (`base/data`, via COMMANDER_SIM_DATA): realism t0-t4, t6
with the plain base binary; t5 and the three deck runs (`--deck K --pool all --games 2000`) with
`base/commander-sim-fix` (base + the two memory commits). `after/`: t0, t1 done (`after-sim` binary =
branch head); t2 was stopped mid-run and its file deleted.

## Next steps
1. `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase2-doubles/run_sims.sh` reruns everything; to finish only what is left, run t2-t6 realism (4000 games, --jobs 20) and
   cloud, seph, yshtola (`--deck K --pool all --games 2000 --jobs 20`) with `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase2-doubles/after-sim` into `after/`
   (no COMMANDER_SIM_DATA: the branch data).
2. `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase2-doubles/score.sh before` / `score.sh after`: compare deck shares and targets per tier.
3. Final report per the brief: card table, false positives, DSL follow-ups (Laelia impulse + counter trigger; Oath
   exile-until-end-step; Static Net exile-until-leaves; Liliana +1 drain clause; the 18 partial spells), the paths
   summary, before/after scorecards, risks (Snap's compiled bounce retargets if its target is gone; the pool-AI
   cost rule also moves Emerald/Sapphire Medallion; the overflow panics above). JD decks touched: cloud and seph
   (Open the Armory, paths_ok only), yshtola (Static Net, paths_ok only) -- no rule change in them.
