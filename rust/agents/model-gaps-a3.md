# Task: the remaining model gaps (no shortcoming left without JD's approval)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/model-gaps-a3` (branch model-gaps-a3, from model-gaps).
Task name: `model-gaps-a3`. Read `rust/agents/report-model-gaps-a1.md` and `report-model-gaps-a2.md` first: this task
finishes what they left.

**JD's rule for this task: no gap may be left quietly.** Do not mark anything Approximate/Partial, add a "left out"
note, or leave a caveat on your own judgement. If a part truly can't be done, stop work on that part, commit what you
have, and put it at the top of your reply under `NEEDS JD` with: the exact gap, its effect in games, and what doing it
would take. The coordinator takes it to JD.

## Items, in this order
1. **Token lands (a bug the A1 work introduced).** Mirrorworks can now make a token copy of an artifact land
   (Darksteel Citadel, Treasure Vault), kept in `Player::lands`. `lands::sac_land` knows a token land ceases to exist,
   but the other ~38 callers of `engine::turn::remove_land` (bounce, destroy, mass sacrifice in `rules.rs`, Kefka's
   paths, flicker, exile ...) push the land's card to a zone, so a token copy leaves a duplicate card in a graveyard,
   hand, library or exile. Fix it centrally: one token-aware helper (e.g. `remove_land` returns whether the card should
   go anywhere, or a `land_to_zone` helper) used by every caller, so no path can put a token land's card in a zone.
   Grep every `remove_land(` call and every `lands.retain` / `lands.remove` that bypasses it. Test bounce, destroy,
   mass sacrifice and exile of a token Citadel/Vault copy.
2. **Astral Dragon** (`impls/t45_miirym.rs`): Oracle "any noncreature permanent": allow a land (the token copy is a land
   token in `Player::lands`, with item 1's support, that is also a 3/3 flying Dragon creature: it must be able to attack
   and block, tap for mana, and die as a creature) and an Aura (the copy enters attached to something legal, or per
   the rules for an Aura token entering: its controller chooses what it enchants; if nothing legal, it goes to the
   graveyard, i.e. ceases to exist). A person may pick these; the AI may keep preferring nonland targets.
3. **Ward on every targeting** (Hexing Squelcher's "Ward—Pay 2 life", and the engine's mana ward): today the engine
   checks ward only when a removal spell resolves on the permanent (`engine/removal.rs`, `t45_krenko::life_ward`).
   Oracle: whenever it becomes the target of a spell or ability an opponent controls, counter it unless that player
   pays. Move the check to where targets are chosen for spells and abilities controlled by an opponent (removal,
   bounce, steal, tap, -X/-X, targeted abilities of compiled cards and card code), so it applies to every ward card
   (mana, life, Prismari's 5 life ...). The AI decides to pay as today; a person is asked. Find the targeting choke
   points; if some card-code paths choose targets on their own, list them and route them through the check.
4. **Mystic Forge**: a counterspell on top of the library can be cast from there in response (the response code only
   counts counterspells in hand). Make the response/counter windows also see the top card while Mystic Forge (or any
   "cast from the top" permission that covers it) is out, for the AI and the person.
5. **Peter Parker's Camera**: copy hand-written activated abilities. In JD's sauron deck: Expedition Map, Jace's
   Archivist, Mind Stone (their activated abilities are card code, not compiled). The obstacle (A2): those functions
   pay their cost inside. Split cost from effect for hand-written activated abilities (e.g. an effect function the
   copy can call without paying), at least for every hand-written activated ability in JD's decks, and make the
   Camera's option list and the person's "copy which ability?" offer them.
6. **Marvin, Murderous Mimic**: borrow creatures' hand-written activated abilities too (Oracle: all activated
   abilities of creatures you control that don't share its name). Use item 5's cost/effect split where it applies.
   At least every hand-written creature activated ability in JD's decks and the pool decks kefka plays against (t3).

AI changes are expected (the coordinator sims afterwards). Keep them minimal and sensible.

## Tests
New file `rust/crates/sim-core/tests/model_gaps_a3.rs`, at least one test per item showing the Oracle behaviour (each
removal path for a token land; Astral Dragon copying a land that attacks and an Aura that attaches; ward on a bounce
spell and on a targeted ability; Forge countering from the top; Camera copying Mind Stone / Archivist / Map; Marvin
using a hand-written ability). Run it plus `model_gaps_karn`, `model_gaps_others`, `practice_model_gaps`, the three
`practice_*_probe` files and the existing test files covering code you touched.

After editing `data/card_overrides.toml` / `data/audit.toml`, run `./rust/target/release/cardgen export --offline` (build
cardgen first) and `cardgen audit --mine` from the worktree: the target is **every card in JD's decks Full**.

## Done when
- All six items done (or under `NEEDS JD` as above); `cardgen audit --mine` shows 0 not Full; the tests pass;
  `cargo build --release --all-targets` has no warnings.
- `rust/agents/report-model-gaps-a3.md`: per item: what changed (rules and AI), files, tests; every change to shared
  files (engine/*, ai/*, state.rs, cardcode.rs, hooks.rs) listed.

## Ground rules (these override the older rust/agents/RULES.md, which is from the Python-port era)
- Work only in your worktree (path above); `cd` into it for every command. Don't merge other branches. Don't push.
  Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`.
- **Never run Python** (a hook refuses it). Use jq, shell tools, or Rust.
- **Memory cap on every cargo command, no exceptions**: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 cargo ...`
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
  `rust/agents/handoff-model-gaps-a3.md` current (done, next, gotchas). If your context gets long (~60+ tool calls), stop at a
  clean point with everything committed and end your reply with a first line `HANDOFF`.
- Finish with: everything committed; full details in the report file (committed); a reply of at most 15 lines (commit hash,
  done / not done per card, test results, anything the coordinator must act on). Then stop.
