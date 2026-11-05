# Task: the karn deck's non-Full cards to Full (sim and practice)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/model-gaps-a1` (branch model-gaps-a1, from model-gaps).
Task name: `model-gaps-a1`. Another agent (model-gaps-a2) works on other cards in parallel; the coordinator merges both.

## Goal
`cardgen audit --mine` rates 13 cards of JD's karn deck below Full. Bring each to Full where the Oracle text can be
modelled, and set its `status` / `note` in `data/card_overrides.toml` honestly. Where a part really can't be done in
reasonable scope, leave the status below Full with the note saying exactly what is missing and why. Read each card's
current note in `data/card_overrides.toml` and its code first (`grep -rn "<name>" rust/crates/sim-core/src/impls`;
much of it is in `impls/karn_engines.rs` and `impls/t5.rs`), and check every ability against the Oracle text
(`jq '.["<lowercased name>"].oracle_text' data/scryfall_cache.json`).

Cards and the gap their current note names (verify; the note may be incomplete):
1. Tezzeret, Cruel Captain (Partial): the -7 emblem is missing (see `rust/agents/engine-needs-godo.md`; needs emblem
   support for it if the engine has none: check how other emblems are done, e.g. Venser's `venser_emblem`).
2. Blightsteel Colossus: infect damage to creatures is plain damage (should be -1/-1 counters); "if it would be put into a
   graveyard from anywhere" shuffle is only from the battlefield (mill/discard still goes to the graveyard).
3. Cityscape Leveler: the unearth "exile it instead if it would leave the battlefield" clause.
4. Darksteel Citadel: indestructible is not modelled.
5. Liberator, Urza's Battlethopter: check what keeps it Approximate (flash timing for the AI); make it Full or say why.
6. Liquimetal Torque: the AI never uses the second ability. Give the AI a use where it measurably helps (e.g. making a
   permanent an artifact for Karn / Construct counts or a Tezzeret/Mirrorworks interaction), or justify leaving it.
7. Mirrorworks: artifact lands (Darksteel Citadel) are not copied.
8. Mishra's Factory: `{T}: target Assembly-Worker gets +1/+1` not modelled.
9. Mishra's Foundry: `{1},{T}: target attacking Assembly-Worker gets +2/+2` not modelled.
10. Mystic Forge: removal, wipes, X spells and counterspells on top of the library are never cast from there.
11. Pithing Needle: a person in practice mode can still activate the named card's abilities (human ability menus don't
    check `rules::ability_locked`); also check the mana-ability exception matches the Oracle text.
12. Steel Hellkite: the `{2}: +1/+0` pump is not modelled.
13. Ugin, the Ineffable: find what keeps it Approximate (compare with Oracle) and fix it.

AI changes are expected here (that's the point: the coordinator sims karn before and after). Keep them minimal and
sensible: the AI should use a new ability only when it helps.

## Tests
New file `rust/crates/sim-core/tests/model_gaps_karn.rs`: at least one test per fixed card showing the Oracle behaviour
(e.g. Citadel survives a destroy, Blightsteel milled is shuffled in, Factory pumps a Worker, Forge casts a removal spell
from the top). Run it plus the existing test files covering code you touched (`cards_karn_*`, `cards_t5`, any
`practice_*` you touched, `play_*` that cover them).

After editing `data/card_overrides.toml`, run `./rust/target/release/cardgen export --offline` (build cardgen first) so
`data/cards.json` matches, and `cardgen audit --mine` from the worktree to confirm the new statuses. Commit the
regenerated JSON too (the coordinator re-exports after merging both branches).

## Done when
- Each of the 13 cards is Full, or below Full with an honest note; tests above pass; `cargo build --release --all-targets`
  has no warnings in files you touched.
- `rust/agents/report-model-gaps-a1.md`: per card: before/after status, what changed (rules and AI), files, tests; every
  change to shared files (engine/*, ai/*, state.rs, cardcode.rs, hooks.rs) listed.

## Ground rules (these override the older rust/agents/RULES.md, which is from the Python-port era)
- Work only in your worktree (path above); `cd` into it for every command. Don't merge other branches. Don't push.
  Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`.
- **Never run Python** (a hook refuses it). Use jq, shell tools, or Rust.
- **Memory cap on every cargo command**: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 cargo ...`
  (run from `rust/`). The shell is zsh: write the prefix out in full, don't store it in a variable. Exit 137 = cap hit = a bug.
- **Don't run the full suite** (`cargo test --release` with no filter) and don't run sims: the coordinator does that. Build and
  run only your own test file and the existing test files you touched or that test the code you changed
  (`cargo test --release -q -p sim-core --test <name>`). Run test commands in the foreground with a long timeout; don't start
  background jobs and don't leave anything running when you finish.
- Practice mode: the person's seat is `crate::human::is_human(g, p)`; questions go through `human::choices::{choose,
  choose_one, pick_cards, pick_any, yes_no, search, ...}`. A new ability a person can use needs a person path too (look at
  how `human/` handles similar abilities); AI and person code stay separate (`if is_human`).
- Never let an unsigned counter go below zero (release builds have overflow checks).
- `rustfmt --edition 2024 <your new files>` only; never `cargo fmt` on the workspace (the repo isn't fmt-clean).
- Commit often (`git add -A rust/ data/ && git commit -q -m "..."`); **no Co-Authored-By line**. Keep
  `rust/agents/handoff-model-gaps-a1.md` current (done, next, gotchas). If your context gets long (~60+ tool calls), stop at a
  clean point with everything committed and end your reply with a first line `HANDOFF`.
- Finish with: everything committed; full details in the report file (committed); a reply of at most 15 lines (commit hash,
  done / not done per card, test results, anything the coordinator must act on). Then stop.
