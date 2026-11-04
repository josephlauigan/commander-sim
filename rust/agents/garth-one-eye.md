# Task: model Garth One-Eye (JD's Ur-Dragon deck)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/garth-one-eye`, branch `garth-one-eye`, based on
origin/main (e6243581). `cd` into it for every command. Commit there; do not push, do not open a PR.

## Which rules apply
Read the repo's `CLAUDE.md` and `AGENTS.md`, then `rust/agents/RULES.md` only for its conventions (impls layout,
hooks, registration, `rustfmt` on your files only, commit often, handoff file, short report). **Ignore the parts of
RULES.md that are out of date:** there is no Python reference and no `rust-port` branch, so don't merge `rust-port`
and don't run `difftest`. Never run Python, not even a version check: a hook blocks it. This task is already approved
by JD; work through it without asking.
- Memory cap on every cargo/binary command: `systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0 <cmd>`.
  Exit 137 means the cap was hit: treat it as a memory bug. The shell is zsh: write the cap out in full, don't store
  it in a variable.
- **Run no simulations** (no `commander-sim` runs, no `all_decks`). The coordinator runs the sims afterwards.
- No `Co-Authored-By:` line in commits. Other agents work in sibling worktrees on other cards; stay in yours.

## Oracle text (Garth One-Eye, {W}{U}{B}{R}{G}, Legendary Creature — Human Wizard, 3/3)
> {T}: Choose a card name that hasn't been chosen from among Disenchant, Braingeyser, Terror, Shivan Dragon,
> Regrowth, and Black Lotus. Create a copy of the card with the chosen name. You may cast the copy. (You still pay
> its costs.)

Rulings that matter:
- The copy is created outside the game's normal zones (treat it as in exile). If it isn't cast as the ability
  resolves, it ceases to exist.
- **The copy is not free:** its mana cost is paid normally, and Ur-Dragon's eminence discounts a Shivan Dragon copy
  by {1}.
- Timing permissions still apply: Disenchant and Terror are instants; Braingeyser, Regrowth, Shivan Dragon and Black
  Lotus need sorcery timing.
- The chosen names belong to that Garth object, so a new Garth (after it changes zones) can choose all six again.

The six cards:
- **Black Lotus:** {T}, sacrifice: add three mana of any one colour.
- **Braingeyser:** {X}{U}{U} sorcery, target player draws X.
- **Terror:** destroy target nonartifact, nonblack creature; it can't be regenerated.
- **Disenchant:** destroy target artifact or enchantment.
- **Regrowth:** return target card from your graveyard to your hand.
- **Shivan Dragon:** 5/5 flying, {R}: +1/+0 until end of turn.

## Current state
`cardgen audit --mine` (run inside the worktree) lists Garth as `Partial — body only, abilities not modeled`. Four of
the six names (Black Lotus, Braingeyser, Shivan Dragon, Regrowth) aren't in `data/cards.json`; Disenchant and Terror
are. Find how cards that aren't in any deck get into the card data. See `rust/crates/cardgen/README.md`,
`data/carddb.txt` and `data/card_overrides.toml`, and look for existing "create a copy of a named card" or similar
effects to reuse. Any change to card data must be followed by `cardgen export` (rebuild cardgen first), with the
generated JSON committed.

## Work
1. Get the four missing cards into the card data, each modelled (status Full) for what a copy needs.
2. Implement Garth's ability in a **new file** `rust/crates/sim-core/src/impls/garth.rs`, registered in
   `impls/mod.rs`. Track chosen names per Garth permanent. Reuse existing machinery (`impls/common.rs`, copy and
   cast paths) rather than copying it.
3. **AI.** JD's `urdragon` seat plays the generic policy plus `data/plans/urdragon.toml`, which another branch is
   rewriting; don't edit plan files. Give Garth an options hook that picks sensibly:
   - Black Lotus when the extra three mana enables a better play this turn, or there's nothing better.
   - Braingeyser when X is at least 3 with spare mana.
   - Terror or Disenchant when an opponent has a worthwhile legal target, including at instant speed.
   - Regrowth when the graveyard holds a card worth getting back.
   - Shivan Dragon when the mana is there and there's no better choice.
   - Don't tap Garth when no choice is castable now: the copy would be wasted.
4. Set Garth's status to Full in `data/card_overrides.toml` with a one-line note, then export.
5. **Tests**, in a new `rust/crates/sim-core/tests/cards_garth.rs`:
   - each of the six copies does what its Oracle text says;
   - a name can't be chosen twice by the same Garth, but a new Garth object can choose it again;
   - the copy isn't free: costs are paid, and the eminence discount applies to Shivan;
   - an uncast copy disappears;
   - the AI taps Garth for Lotus or Terror in a fitting board, and doesn't tap it when nothing is castable.

## Finish
- **Checks:** `cargo test --release` (from `rust/`, under the cap) passes, except the known
  `every_plan_file_fits_its_deck` failure for `urdragon` (fixed on another branch). `cargo build --release
  --all-targets` has no warnings.
- **Commits:** everything committed, with `rust/agents/handoff-garth-one-eye.md` kept current while you work.
- **Full report** in `rust/agents/report-garth-one-eye.md`: cards, engine changes and every shared file touched, and
  any simplification you made and why.
- **Final message:** at most 15 lines (commit hash, done or not done, test result, anything the coordinator must act
  on). Then stop.
