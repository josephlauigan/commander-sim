# Task: model Judgment of Alexander and finish Forge Anew (JD's Cloud deck)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/cloud-gaps`, branch `cloud-gaps`, based on
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

## Cloud
JD's `cloud` deck (mono-W, Cloud as commander, about ten Equipment) plays the generic policy plus
`data/plans/cloud.toml`. `cardgen audit --mine` (run inside the worktree) shows two gaps:
`Unmodeled Judgment of Alexander` and `Partial Forge Anew`.

## Judgment of Alexander ({2}{W} Instant)
Oracle text:
> Prevent all damage that would be dealt to you this turn by sources your opponents control. Whenever damage from a
> creature is prevented this way, each commander creature you control deals damage equal to its power to that
> creature.

- **The shield** covers only its caster (not their permanents), only from sources opponents control, and lasts until
  end of turn. It covers combat and noncombat damage alike.
- **The strike back:** whenever damage from a creature is prevented by it, each commander creature its controller
  controls deals damage equal to its power to that creature. It's a trigger, so it uses the stack. With no commander
  creature on the battlefield it does nothing.
- **Reuse first.** Look for existing damage-prevention machinery (fogs, Teferi's Protection, Stand Up for Yourself,
  Sphere of Safety, anything else that prevents damage to a player) before adding engine state. Shared engine
  changes: smallest additive change, listed in your report.
- **AI:** cast it on an opponent's turn when it matters:
  - after attackers are declared, when the attacks on Cloud's controller would deal lethal or heavy damage, or when
    the strike back kills a worthwhile attacker;
  - in response to a big burn spell or ping aimed at Cloud's controller.
  
  Don't fire it into nothing.

## Forge Anew ({2}{W} Enchantment)
Oracle text:
> When this enchantment enters, return target Equipment card from your graveyard to the battlefield.
> During your turn, you may activate equip abilities any time you could cast an instant.
> You may pay {0} rather than pay the equip cost of the first equip ability you activate during each of your turns.

The enter trigger is already modelled; check it, don't redo it. Add two things:
- **The free first equip:** {0} instead of the equip cost for the first equip ability its controller activates on
  each of their turns. The AI should spend it on the most expensive useful equip that turn.
- **Instant-speed equips during its controller's turn.** A simple, useful AI is enough, e.g. moving Equipment onto
  an unblocked attacker after blockers are declared.

## Tests
Add them to a new `rust/crates/sim-core/tests/cards_cloud_gaps.rs`.

**Judgment of Alexander:**
- prevents combat and noncombat damage from opponents' sources to its caster only;
- doesn't prevent damage from its caster's own sources, and ends at end of turn;
- the strike back deals Cloud's power to each attacking creature whose damage was prevented, and does nothing without
  a commander creature out;
- the AI casts it against lethal attacks and holds it against trivial ones.

**Forge Anew:**
- the first equip each turn costs {0}, and the second pays full;
- equip works in combat on its controller's turn, but not on an opponent's turn;
- its enter trigger returns an Equipment.

Then set both cards' status to Full in `data/card_overrides.toml` with a one-line note each, rebuild cardgen, run
`cardgen export`, and commit the generated JSON.

## Finish
- **Checks:** `cargo test --release` (from `rust/`, under the cap) passes, except the known
  `every_plan_file_fits_its_deck` failure for `urdragon` (fixed on another branch). `cargo build --release
  --all-targets` has no warnings.
- **Commits:** don't edit plan files. Everything committed, with `rust/agents/handoff-cloud-gaps.md` kept current
  while you work.
- **Full report** in `rust/agents/report-cloud-gaps.md`: what you modelled, engine changes and every shared file
  touched, and any simplification and why.
- **Final message:** at most 15 lines (commit hash, done or not done, test result, anything the coordinator must act
  on). Then stop.
