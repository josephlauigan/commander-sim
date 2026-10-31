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
