# Task: Marvin to Full, the Godo plan's equip on the stack (JD's decisions, 2026-11-05)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/model-gaps-a4` (branch model-gaps-a4, from model-gaps-a3).
Task name: `model-gaps-a4`. Read `rust/agents/report-model-gaps-a3.md` and `rust/agents/handoff-model-gaps-a3.md` first
(the cost/effect split `stack::ability_run_w`, Marvin's borrowing through `options` hooks).

**JD's rule: no gap may be left quietly.** Do not mark anything below Full, add a "left out" note, or leave a caveat on
your own judgement. Anything you truly can't do: stop that part, commit, and put it at the top of your reply under
`NEEDS JD` (exact gap, effect in games, what it would take).

## Items (JD chose each one)
1. **Model five pool creatures' activated abilities** (they have no ability code today, even for their own controller),
   for the AI and for a person, checking each against the Oracle text (`jq '.["<lowercased name>"].oracle_text'
   data/scryfall_cache.json`):
   - Siege-Gang Commander (zurgo t0, korvold t3, krenko t4): {1}{R}, sacrifice a Goblin: 2 damage to any target.
   - Syr Konrad, the Grim (teysa t1, meren t2, korvold t3, tergrid t3): {1}{B}: each player mills a card.
   - Professional Face-Breaker (aurelia t3, prosper t4): sacrifice a Treasure: exile the top card, you may play it this turn.
   - Tinybones, Trinket Thief (tergrid t3): {4}{B}{B}: each opponent with no cards in hand loses 10 life.
   - Ezuri, Renegade Leader (lathril t1, marwyn t3): {2}{G}{G}{G}: Elves you control get +3/+3 and trample until end of
     turn. The previous agent says the AI's overrun is deliberately unregistered: find out why (git log -S / blame), and
     make it right without breaking whatever that avoided.
   Use the existing machinery (compiled abilities in `data/card_overrides.toml` / `carddb.txt` where the ability language
   can express it, else card code with `ability_run_w`), so Marvin borrows each one. The AI should use each when it helps
   (Siege-Gang to finish a player or a walker / kill a key creature, Konrad with its death triggers, Face-Breaker when it
   can play the card, Tinybones when it kills, Ezuri for a lethal or near-lethal attack). These pool decks' play changes.
2. **Marvin: a person chooses** the targets and choices of every ability borrowed through Marvin (today the AI function
   picks them, as with the older Krenko and Kiki-Jiki borrows: fix those too). Each borrowed ability uses the person path
   of the creature's own ability where one exists (same questions), with Marvin as the source.
3. **The Godo plan's Helm of the Host equip** (`impls/t6_godo.rs` / Sauron's Godo line): the {5} equip is one plan step
   with no stack item, so it can't be copied (Peter Parker's Camera, Rings) or responded to. Put it on the stack like
   every other equip (`ability_run_w`), for the AI and the person. Keep the line's sequencing otherwise identical.
4. Set Marvin's status to Full in the overrides once 1 and 2 are done (and keep Camera's note exact).

## Tests
New file `rust/crates/sim-core/tests/model_gaps_a4.rs`: each of the five abilities for its own controller (AI and person)
and borrowed by Marvin; a person asked for a Marvin-borrowed Krenko / Kiki-Jiki / Chatterfang choice; the Godo equip on the
stack and copied by a Camera. Run it plus `model_gaps_a3`, `model_gaps_karn`, `model_gaps_others`, the `practice_*_probe`
files, the test files for the five pool decks' commanders/cards, and Godo/Sauron tests.
Re-export cards (`./rust/target/release/cardgen export --offline`) and run `cardgen audit --mine`: the target is only
Hexing Squelcher below Full (JD accepted its gap).

## Done when
- Items 1-4 done (or under `NEEDS JD`); tests pass; `cargo build --release --all-targets` has no warnings.
- `rust/agents/report-model-gaps-a4.md`: per item: what changed (rules and AI), files, tests; every shared-file change.

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
  `rust/agents/handoff-model-gaps-a4.md` current (done, next, gotchas). If your context gets long (~60+ tool calls), stop at a
  clean point with everything committed and end your reply with a first line `HANDOFF`.
- Finish with: everything committed; full details in the report file (committed); a reply of at most 15 lines (commit hash,
  done / not done per card, test results, anything the coordinator must act on). Then stop.

## Follow-up (JD's decisions, 2026-11-05, after items 1-4)
5. **Marvin: the rules reading for self-named abilities** (rule 201.5: an ability that names its object means the object
   that has it, i.e. Marvin; 701.28c: a permanent that isn't a double-faced card can't transform). Switch Kefka and
   Tekuthal to match the other nine (Hope of Ghirapur, Hawkeye, Balan, Drivnod, Sisay, Scion, Thanos, Nicol Bolas, Venat):
   Tekuthal's indestructible counter goes on Marvin; a Marvin-borrowed Kefka {8} does the rest of its text (check Kefka's
   Oracle via the card_faces of `kefka, court mage // kefka, ruler of ruin` in data/scryfall_cache.json) but transforms
   nothing (Kefka is not transformed). AI and person. Make sure the kefka deck's AI no longer counts Marvin as a way to
   transform Kefka (grep the kefka plan / combo code for Marvin). Tests for both.
6. **Proliferate: a person chooses** (rule 701.27: any number of permanents and/or players that have a counter; each
   gets one more of each kind it already has). One shared person path (e.g. `human::choices::pick_any` over the
   permanents and players with counters, with the AI's "useful" picks preselected or listed first) used by every
   proliferate source a person controls: `cardcode::proliferate_all` callers, Karn's Bastion, Vraska's 0, Cankerbloom,
   High Perfect Morcant, Marvin's borrowed ones, and any compiled proliferate effect (grep "proliferate"). AI unchanged.
   Tests: a person proliferates a chosen subset (including choosing an opponent's poison or not), per source kind.
Rules for these: as above (NEEDS JD for anything you can't do). Update the report and handoff.
