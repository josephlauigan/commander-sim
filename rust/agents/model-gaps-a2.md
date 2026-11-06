# Task: the non-Full cards of thanos, kefka, urdragon and sauron to Full (sim and practice)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/model-gaps-a2` (branch model-gaps-a2, from model-gaps).
Task name: `model-gaps-a2`. Another agent (model-gaps-a1) works on the karn deck's cards in parallel; the coordinator
merges both. Don't touch the karn cards (Tezzeret, Blightsteel, Mishra's lands, Mystic Forge, Pithing Needle, Ugin ...).

## Goal
`cardgen audit --mine` rates these 8 cards of JD's decks below Full. Bring each to Full where the Oracle text can be
modelled, and set its `status` / `note` in `data/card_overrides.toml` honestly. Where a part really can't be done in
reasonable scope, leave the status below Full with the note saying exactly what is missing and why. Read each card's
current note in `data/card_overrides.toml` (and `data/carddb.txt`) and its code first
(`grep -rn "<name>" rust/crates/sim-core/src/impls`), and check every ability against the Oracle text
(`jq '.["<lowercased name>"].oracle_text' data/scryfall_cache.json`).

Cards and the gap their current note names (verify; the note may be incomplete):
1. Hexing Squelcher (thanos; audit says Approximate, override says Partial): ward--pay 2 life for itself and your other
   creatures is missing; the engine's ward is mana only (`rust/agents/engine-needs-godo.md`). Add life-payment ward to the
   engine (smallest additive change; AI opponents decide whether to pay like they do for mana ward).
2. Thanos, the Mad Titan (thanos, the commander; `impls/t6_thanos.rs`): "left out: once per game per player rather than
   per Thanos object, and a token's mana value is 0 (a token copy's is the copied card's)". Check the Oracle text and fix.
3. Lotho, Corrupt Shirriff (thanos): only regex tags (`treas`); model it from the Oracle text.
4. Marvin, Murderous Mimic (kefka): copies only the {T} abilities of a fixed list of creatures; Oracle: it has all
   activated abilities of creatures you control that don't have the same name. Generalise as far as the engine allows
   (compiled activated abilities at least) and say in the note what remains.
5. Psychosis Crawler (kefka): only regex tags; model it from the Oracle text (power/toughness = cards in hand; whenever
   you draw a card, each opponent loses 1 life). Check Kefka's draw/discard engines see it.
6. Astral Dragon (urdragon; `impls/t45_miirym.rs`): find what keeps it Approximate vs Oracle and fix (token copies are
   3/3 Dragon creatures with flying in addition to their other types).
7. Scion of the Ur-Dragon (urdragon, the commander): find what keeps it Approximate vs Oracle (copy until end of turn,
   any Dragon permanent card into the graveyard) and fix.
8. Peter Parker's Camera (sauron): hand-written activated abilities (card code calling `stack::ability_window`) can't be
   copied. Make them copyable if a clean way exists; otherwise keep Approximate with the exact note.

AI changes are expected (the coordinator sims these decks before and after). Keep them minimal and sensible.

## Tests
New file `rust/crates/sim-core/tests/model_gaps_others.rs`: at least one test per fixed card showing the Oracle behaviour
(e.g. targeting Squelcher's creature costs 2 life or is countered, Crawler drains on each draw, Marvin copies a
compiled activated ability). Run it plus the existing test files covering code you touched (`cards_kefka`, `cards_t6*`,
`cards_sauron`, urdragon/miirym tests, any `practice_*` you touched).

After editing `data/card_overrides.toml` (or `carddb.txt`), run `./rust/target/release/cardgen export --offline` (build
cardgen first) so `data/cards.json` matches, and `cardgen audit --mine` from the worktree to confirm the new statuses.
Commit the regenerated JSON too (the coordinator re-exports after merging both branches).

## Done when
- Each of the 8 cards is Full, or below Full with an honest note; tests above pass; `cargo build --release --all-targets`
  has no warnings in files you touched.
- `rust/agents/report-model-gaps-a2.md`: per card: before/after status, what changed (rules and AI), files, tests; every
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
  `rust/agents/handoff-model-gaps-a2.md` current (done, next, gotchas). If your context gets long (~60+ tool calls), stop at a
  clean point with everything committed and end your reply with a first line `HANDOFF`.
- Finish with: everything committed; full details in the report file (committed); a reply of at most 15 lines (commit hash,
  done / not done per card, test results, anything the coordinator must act on). Then stop.
