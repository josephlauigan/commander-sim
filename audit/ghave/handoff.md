# Handoff: Ghave, Guru of Spores takes Seph's t3-t4 seat (brief: rust/agents/ghave-wire.md)

## Survey: where `seph` was wired, and what happened to each site

| Site | Decision |
| --- | --- |
| `data/my_decks.toml` `[seph]` | renamed `[ghave]` (JD/ghave-abzan-aristocrats.md, Ghave, WBG, t3-t4) |
| `decklists/JD/sephiroth-phyrexian-reanimator.md` | moved to `decklists/pool/retired/` |
| `sim-core/src/testkit.rs` | `seph` kept as a retired test seat (its real list, read from retired/) for the card tests |
| `sim-core/src/ai/seph.rs` (Seph's hand AI, 1111 lines) | deleted; the generic parts (bomb rating, reanimation target value, the Entomb/Buried Alive bin choice) moved to `ai/reanimate.rs` |
| `ai/mod.rs` | `pub mod seph` -> `pub mod reanimate`; `seph` out of MAIN_STYLES and jd_deck; `seph_dredge` / `seph_end_milestones` wrappers deleted; `seph_bval`, `seph_fill_resolve`, `seph_rean_resolve` keep their names (generic, used by every deck's reanimation; renaming would touch brain.rs) |
| `ai/decks.rs` | deleted: seph cast priority, tutor target, protect / protect_silenced, wipe_response, the seph-only wipe-loss discount, the Sac/Desert valuations, the `key == "seph"` exclusion from sac_in_response and the wipe sacrifice |
| `ai/brain.rs` (PR 5b edits it too) | deletions only: the seph hold-up bonus (3 lines), seph's note_bomb on a graveyard cast (3 lines + the now-unused `let ok`), `"seph" => seph::options` (1 line), `key != "seph" &&` in the aristocrat-options condition |
| `ai/veyran.rs` | `seph::boots_cost` inlined (one line) |
| `engine/cast.rs`, `removal.rs`, `stack.rs`, `turn.rs`, `zones.rs`, `impls/common.rs`, `settings.rs` | deleted the seph-only branches: commander/bomb importance, bomb_removed and seph_spell_countered stats, Tireless Tracker first, bomb discard at cleanup, end milestones, dredge, token colour, opponent threat +3, cthresh |
| `impls/mine.rs` (Seph's card code: loops, Necromancy flash, etb values) | kept (card code); its `key == "seph"` gates stay for the retired test seat; the loops are exposed to the combo catalogue for every deck (see below) |
| `impls/sephfs*.rs`, `impls/sheoldred.rs` (Sephiroth, Fabled SOLDIER) | kept (card code; Sephiroth is in Ghave's list) |
| sim-cli `decks.rs`, `args.rs`, `goldfish.rs`, `knobs.rs`, `main.rs`, `poolmode.rs`, README | renamed seph -> ghave |
| sim-cli `report.rs` | ghave's plan turn = first combo attempt; the bomb metrics are labelled for ghave (the reanimation package stays) |
| practice `catalog.rs`, `session/setup.rs` | renamed |
| cardgen `data.rs` | doc comment only |
| `data/spellbook/seph.json` | -> see below |
| `data/spellbook/seph.json` | deleted; `data/spellbook/ghave.json` fetched (scripts/spellbook-fetch.sh ghave; the script now follows the API's `next` pages: Ghave's almost-included list passes 1000). Combo data by Commander Spellbook, https://commanderspellbook.com |
| tests seating `seph` | card tests keep seating the retired `seph` list (testkit); tests of Seph's hand AI deleted (cards_seph.rs: 10, cards_audit_seph.rs: 5); helper tests retargeted to `ai::reanimate` |

## Ghave modelling (done)
- Card check: all 92 unique names legal in Commander, colour identity within W/B/G (scryfall_cache.json); `cardgen audit --mine`: ghave 0 not Full (but Ghave's abilities were not usable before this branch: see below).
- Ghave's abilities: compiled (cardgen DSL). New in the ability language: the cost `other = "remove a +1/+1 counter from a creature you control"` (dsl::pay_ability_cost: the creature with the most counters that survives losing one); before, any `other` cost refused to pay, so the Saproling ability never ran.
- "Enters with five +1/+1 counters": impls/ghave.rs `enters_with`, added in zones::enter (any entry; Doubling Season 10).
- Paid sacrifice outlet in responses: `dsl::paid_outlet` / `activate_sacrificing`; common::sac_in_response and sac_before_wipe use it for a deck with plan knob `knobs.paid_outlets = true` (only ghave), so pool decks are unchanged. decks.rs `sac_doomed` (the main decks' wipe sacrifice, factored out) runs for such a deck after pool::wipe_response finds no protection.
- Combo catalogue (impls/ghave.rs COMBOS, chained last in combos::all): Ghave + altar + Crusade/DS + payoff; Ghave + Ashnod + Crusade/DS + Triskelion; Mikaeus + Triskelion; Mikaeus + Finks + free outlet + payoff; Nim + Ashnod + Grave Titan + payoff; Finks + Crusade + free outlet + payoff (mine.rs `loops_now`: seph_loops without the key gate).
- Plan file data/plans/ghave.toml (style = Seph's, commander prio/turn, the six lines, prio rules, Heroic Intervention hold, mulligan 2-5 lands).

## Status (2026-10-31)
- `cargo build --release --all-targets -j 8`: 0 warnings. `cargo test --release --no-fail-fast -j 8`: 3699 passed, 0
  failed (practice included). `cardgen export`: no diff; `cardgen validate`: all decks passed.
- Sims (loose, 2000 games, seeds 500000-501999, rc=0): ghave t3 12.3% (CI 11.0-13.9), t4 11.3% (CI 10.0-12.8),
  goldfish median kill 10. Seph on origin/main, same seeds: t3 22.1%, t4 17.6%, goldfish median 11.
- FINDINGS.md: the harness refuses report files from subagents; its text is in the final report for the caller to
  commit as audit/ghave/FINDINGS.md.

## Next (not done)
1. `--analyze` at t3/t4; plan rules for the reanimation start (fill then reanimate), a Grand Abolisher hold.
2. Mulligan rule with pieces.

## Gotchas
- Two other agents share the machine: --jobs 8 / -j 8, one sim at a time.
- Baseline Seph sims ran from a data snapshot (COMMANDER_SIM_ROOT) with the binary built before the rename.
