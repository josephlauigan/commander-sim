# Handoff: practice-cards

Branch base: rust-port (practice-core + session merged) + practice-core branch worktree-agent-ac27fe0996d4a60cd (merged 62cb457).

## Done (all committed, `cargo build --release --all-targets` clean)
- ALL `HUMAN(phase 9)` sites in `impls/*.rs` filled; yshtola `log_secret` sites use `Game::log_secret`.
- `human/abilities.rs`: complete port of play/abilities.py. Representation: `Ability<T>{label, act: Box<dyn Fn(&mut Game,PlayerId,T)->Res<Option<String>>>}`,
  `PermAbility`/`LandAbility`; `permanent_abilities`, `land_abilities`. (Changed core's human.rs call sites from `(String, fn)` tuples
  to this struct: `AbilityKind::Extra(abilities::Act<PermId>)`, `a.label` / `a.act`.)
- `human/cards.rs`: complete port of play/cards.py: helpers, `abilities()` registry (walkers, outlets, lands, Galadriel/Zur/Y'shtola
  decks, prepared faces), HAND casts (`hand_override`/`cast_from_hand`, twinflame, ephemerate, concealment...), `needs`, `convoke_pay`,
  `copy_card`/`copy_in_response`, `jeskas_will`, `crackle`, `mastery_target` (-> `Res<Option<CardId>>`, call site in human.rs changed,
  new `Ctx.mastery_pick` used in engine/cast.rs Mastery resolve), `flashback_target`/`granted_flashback` (new `Player.fb_grant`),
  `dredge`, `cast_target` (Take Up the Shield), `gy_override` (always false: Python's GY table is empty).
- Shared additive edits: state.rs (`Ctx.mastery_pick`, `Player.fb_grant`, `Player.necro_wait`), impls/mine.rs (`cast_copy_with`,
  `pw_once`/`pw_use`/`loop_card`/`insight_cost` made pub), impls/galadriel.rs (`rebel_cards`, `put_rebel`, `shapeshifter` pub),
  engine/cast.rs (Mastery pick).

## Next (in order)
1. Tests `sim-core/tests/play_{cards,abilities,marchesa,sauron,seph,veyran}.rs` (79 Python tests in tests/test_play_*.py). Use
   `tests/play_common/mod.rs` (core's) and `ScriptHuman`. Fix bugs found in the ports above.
2. Checks (RULES.md: cargo test --release, all_decks adaptive 3 / lookahead 1, difftest 0 AI/build diffs), grep no HUMAN(phase 9) in impls,
   report-practice-cards.md (≤15-line summary to caller).

## For the report
- Necropotence: the human's ability stores cards in `Player.necro_wait` (ais.necro_exile ported in cards.rs) but delivery
  (`cardcode::necro_deliver`, `PORT(M5)` placeholder, not mine) must move them to hand at the end step.
- Prepared-face copies by a person under Thousand-Year Storm: Storm copies only the effects that equal prepared_effect's numbers
  (Seething Song, tutor, Ancestral on yourself); Bolt / Ancestral on others are not copied (closure can't be passed to the hook).
- Guildmage "counter ability": `ability_window` target is None (Python passes the stack item).

## Gotchas
- Python `human.choose` default cancel='cancel' -> cards.rs `pick()`; `cancel=None` -> `choices::choose_one`.
- Python `hc.search` default allow_none=True; `choices::search(.., true)`.
- `gen` is reserved in Rust 2024: use `gn`. zsh: quote globs in grep.
- The shell tool refuses compound commands with some heredocs: write patch scripts to the scratchpad and run them.
- Auto mode blocks `git stash drop` / overwriting conflicted files with `git show >`; edit the file instead.
