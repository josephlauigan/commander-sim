# Report: practice-mode simplifications removed (part 2)

Branch practice-simplifications. Tests: `rust/crates/sim-core/tests/practice_simplifications.rs` (20 tests, all pass).
No sims, no full suite run; the existing test files covering the touched code were run and pass (list at the end).

## 1. Urza's {5} and Triple Triad: free until end of turn (person only) - DONE
- Design: a new per-player list `Player::free_play` (state.rs) holds exiled cards the person may play free this turn.
  `human::triggers_t1::allow_free_play` (replaces `play_free_now`) adds the card and tells the person; `urza_top` and
  `mine::triad_person` call it (Triple Triad grants every playable card: own card + lesser mana value, as before).
  The cards stay in exile; `engine/turn.rs` end_step clears `free_play`.
- `human::gaps_c::exile_permits / exile_playable` merge it with Dauthi Voidwalker's permission; `legal::check_cast_exile`
  and new `legal::check_land_exile` use it. New action `{"do":"land","zone":"ex","card":id}` (a land drop, main phase).
  Spells cast through the existing `{"do":"cast","zone":"ex"}` path (target, X = 0, no mana cost). A counterspell is cast
  from exile at a spell via new `human::human::cast_counterspell_free` (respond loop); a card from an opponent's exile
  goes back to that player's graveyard (`cast_rean` now sends such an Aura/reanimation card to its owner's graveyard).
- Also fixed: the respond loop read a `zone: ex/gy` action's `card` as a hand index; now only hand casts do.
- UI: practice `view.rs` exile_playable adds `land`; `app.js` and `text.rs` (`land eN`, help text) updated.
- Tests changed: practice_binder.rs Triple Triad/Urza tests now check the permission then cast in the main phase.
- Files: state.rs, engine/turn.rs, human/{gaps_c,legal,human,triggers_t1}.rs, impls/{t5,mine}.rs, practice/{view,text,app.js}.
- AI paths unchanged.

## 2. Mirage Mirror copies any land (person only) - DONE
- `gaps_c::mirage_mirror` no longer filters lands (fetch, bounce, Chasm, ability lands, own legendary lands).
- The copy's abilities work (they read the copied card). The problem was zone changes reading `Perm::card()` (the copied
  card), so a cracked fetch land would put the fetch land's card in the graveyard. New `Perm::real_card()` (phys or cd)
  is used in: `lands::sac_land`, `lands::destroy_land`, `fixes.rs` (2 sites), `t6_lands` (Abstergo), `human/cloud.rs`
  (Plaza), `human/abilities.rs` (Scavenger Grounds), `human/cards.rs` (Strip Mine), `mine.rs` (2 sites). Same result
  as before unless a permanent has `phys` set (only a Mirror copying a land, so no AI change).
  Not changed: ~25 other land-removal sites (AI sacrifice/bounce effects, Tragic Lesson etc.): a Mirror-as-land hit by
  those would still put the copied card in the zone for the rest of that turn.
- Legend rule (`life::sba_legend`) now includes the land list when some land is a copy (cheap check; no effect
  otherwise); a land loser leaves via `remove_land` + `real_card`.
- Test practice_limits.rs "does not offer a land with abilities" flipped to "offers" (Flooded Strand listed).

## 3. Sylvan Library (person) - DONE
- `Player::drawn_cards` filled by `zones::draw` (cleared when `draw_st` changes); `triggers_t1::drawn_in_hand` matches it
  against the hand one for one (card ids are shared per name, so a played drawn Island with an older Island in hand
  is a known edge). `sylvan_library` offers exactly those cards.
- AI Sylvan (`impls/topdeck.rs::library`) untouched: it looks only at its two extra cards (not a one-line switch).

## 4. Ur-Dragon loop shortcuts - DONE (a) and (b)
- (a) `human::choices::DamageBatch` + `deal_damage_batched`; `Game::damage_batch`. `best_target_any` (t2.rs and t3.rs
  copies; used by Terror of the Peaks, Scourge of Valkas, Dragon Tempest) calls it for a person. Scopes (`batch_begin`
  / `batch_end`) are opened in `zones::make_tokens` around the TokensEnter hook (>1 token), `urdragon::go_loop`
  (whole loop), `twinflame_tokens` (>1 target) and `dualcaster_loop`. First trigger asks "aim all at the same target?"
  then the target once; later triggers hit it while legal (else asked again); "no" keeps per-trigger questions.
  Opened only for a person; no AI change. t45_miirym.rs has no per-token person prompt (its prompt is once per Astral Dragon).
- (b) `urdragon::garth_spare`: after the rounds, the person chooses how many of the newest (untapped) token Garths tap
  instead of attacking; each makes a Black Lotus (3 mana), optionally one casts Braingeyser ({X}{U}{U} from that mana,
  chosen player draws X), the rest of the mana goes to the pool in a chosen colour (or spread). Earlier rounds' Garths
  still count as untapped attackers (as before); a fuller design would tap them and pay each Twinflame's strive from the
  Lotuses round by round.

## 5. Ragavan's exiled card is not a card in hand (AI-AFFECTING, approved) - DONE
- Shared `engine::values::hand_proper / hand_count` (excludes impulse, impulse_long, stolen, agent_ids; same as
  Prosper's old `hand_proper`, which now calls it; identical results when none of those lists is in use).
- Sites changed (counting only): `engine/turn.rs` cleanup `held` (AI max hand size; now also ignores impulse and
  Prosper impulse_long cards), `human/choices.rs::discard_to_hand_size`, `dsl/mod.rs` `cards_in_hand` and
  `event_player_hand`, `impls/t5.rs` Windfall, `impls/generic_paths.rs` + `human/abilities.rs` Jace's Archivist,
  `impls/legends.rs` Mordenkainen's Dog, `impls/common.rs` Empyrial (power/toughness), `impls/kefka.rs` Inner Fire,
  `impls/t45_yuriko.rs` Sea Gate Restoration, `impls/t0_giada.rs` Wojek Investigator, `impls/t3_aurelia.rs` "most cards
  in hand", `impls/t3_korvold.rs` Pox counts, `engine/tutors.rs` AI Jeska's Will, `human/cards.rs` Jeska's Will (2).
  Not changed: discards/moves of cards, "has a card to discard" checks, AI heuristics (hand-size weights).
- Tests: Ragavan hit leaves hand 3 / count 2; Sea Gate draws 3 not 4; no end-of-turn discard with 7 real + Ragavan's.

## 6. Goldspan Dragon (AI-AFFECTING, approved) - DONE (narrow)
- Oracle: "attacks or becomes the target of a spell": abilities do not trigger it (no ability trigger added).
- The real gap: a copy of a spell is a spell. `cast::copy_spell` and `cast::cast_spell_copy` now call
  `card_models3::spell_targeted` (Goldspan treasure; also Rotpriest's poison trigger, which has the same wording).
  AI-affecting: copies by Jin-Gitaxias, Thousand-Year Storm, Dualcaster, Mizzix, Panoptic Mirror, Twinflame-type copy
  effects aimed at a Goldspan Dragon / Glissa's Rotpriest creature now trigger it.
- Not done: spells whose target is picked inside their own resolve code rather than at cast (the engine's targets for
  most removal are in `Ctx` at cast, so this is a small set; not enumerated).

## Existing tests run (all pass)
practice_{simplifications,binder,limits,gaps_c,followups,counter_probe,triggers_t1,ragavan,urdragon_loops},
play_{urdragon_human,thanos_human,fetch_lands,cards,choices,veyran}, cards_{lands,land_audit2,cloud,jd_models,rules,
models3,t3,t4,t5,t0_giada,t3_korvold,t3_aurelia,t45_prosper,t45_yuriko,t45_miirym,t45_krenko,t45_urdragon,kefka,kefka_ai,
t6_legends1,t0_stella,veyran}, engine_{batch_e,t6_combo1,t6_combo3}, generic_paths, rules, ownership; `-p practice`;
whole workspace `cargo build --release --all-targets` compiles.
