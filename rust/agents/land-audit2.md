# Task: land-audit2 — nonbasic lands that make more mana than their Oracle text allows

Read `rust/agents/T0-RULES.md` (Hard rules, How cards are modeled, Checks before you finish items 1-3, Keeping context
small apply). Your worktree is `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/land-audit2`, branch
`land-audit2` from origin/main (2026-10-17). Handoff `rust/agents/handoff-land-audit2.md`, full report
`rust/agents/report-land-audit2.md`. Builds and tests: `nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G
-p MemorySwapMax=0 cargo ... -j 6`; sims `--jobs 6` (another sim may be running; check `uptime` and use `--jobs 16`
when the 1-minute load is under 4). Don't push. This restarts the 2026-10-15 land-audit, which was stopped before
any work was saved.

## Why
On 10/14 the Tier 0 audit found seven precon lands making free mana (commit `5fedc2c`, read it first:
`git show 5fedc2c`): Sungrass Prairie and Ferrous Lake made 2 mana, Tainted Field/Wood gave coloured mana without a
Swamp, Fetid Heath and Cascade Bluffs gave filter colours with nothing to pay, Temple of the False God made CC with
fewer than 5 lands. They were fixed for the precons only. The same kind of fault may sit in **every** deck: JD's
(`decklists/JD/`), Avery's (`decklists/Avery/`) and the pool (`decklists/pool/t0..t5`). Known suspects: Twilight Mire
(Meren), Rugged Prairie (Aurelia, Winota), Mystic Gate (Brago).

## What to do
1. List every nonbasic land in every deck (`data/decks.json`), with how the sim models its mana (tags in
   `data/carddb.txt` / `data/card_overrides.toml`, `land_cols`/`land_mana` hooks, engine `mana.rs`).
2. Compare each with its Oracle text (`jq '.["name lowercased"].oracle_text' data/scryfall_cache.json`). Look for:
   filter lands (Shadowmoor/Eventide: need a mana in to filter), horizon-style or "two mana" lands, conditional colours
   (Tainted lands, Temple of the False God, Ancient Tomb's damage, City of Brass / painland damage, Command Tower /
   Exotic Orchard / Reflecting Pool colour rules, Cavern/Plaza restrictions), lands that should enter tapped, and
   anything that taps for more than one mana. Also too-strict models (a land giving less than it should) — report them.
3. Fix every real fault with a test, the way 5fedc2c did. Keep each fix to the land's own data or code.
4. **Same-seed check:** build main's binary (or use `git stash`-free approach: a detached worktree at origin/main
   under .claude/worktrees/, removed when done) and compare `commander-sim --deck <d> --pool all --profile loose
   --games 300 --ai adaptive --jobs 8 --seed 7 --brief` for all of JD's and Avery's decks in `data/my_decks.toml` (12 now, including `kraum`; use `--pool
   all`, which runs each deck on its matched tiers only), before vs after. Report which decks moved
   and by how much (decks whose lands didn't change should be identical, except via opponents' lands).
5. Commit; report as the rules say (lands fixed per deck, before/after numbers, anything too strict).

## Lanes
Touch only land card data/code and their tests (plus the smallest engine change if a land needs one; list it).
