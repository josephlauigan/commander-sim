# Task: the rest of common.py, partials.py and fixes.py (+ four bug fixes)

Worktree: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/agent-ae9db43dd10e4ee36`
(branch `worktree-agent-ae9db43dd10e4ee36`). A previous agent committed 57c168d ("the rest of common, partials and
fixes; four bug fixes") but was stopped while reworking four failing tests; check `cargo test --release` first.

Files: `impls/common.rs`, `impls/partials.rs`, `impls/fixes.rs`, Karn's Bastion's part of `impls/lands.rs`, tests
`tests/cards_common2.rs` (or the existing cards_common.rs / cards_rules.rs).

1. Port everything still unported in common.py, partials.py and fixes.py (each function, `@on` hook and table entry
   vs the Rust): pillowfort taxes, stax pieces, planeswalkers (walker framework; `loyalty_extra` exists now), Walking
   Ballista, graveyard hate, Otawara, Reflector Mage, Spark Double, Frost Titan, Dream Trawler, Ephemerate's rebound,
   Narset, Static Net / Powerstone, DYN_MANA mana cards, proliferate, Sign in Blood, saga_step (Urza's Saga) and
   CI.SAGA, Coat of Arms and Bloodline Keeper (`coat_bonus` / `lineage_bonus`), Golgari Charm (`regen_wipe`),
   Hunter's Insight (`p.insight`), Karn's Bastion.
2. Replace partials.rs's local copies of common's helpers (best_opp_creature, best_opp_nonland, auras_on, own_host)
   with calls into common.rs.
3. Bug fixes (Rust only, a test each): (a) Druid Class never reaches level 2 (offer 1→2 too); (b) Mardu Charm's
   discard mode is never chosen (`ctx['mode']` never set: let the AI choose); (c) Angelic Destiny-style Auras
   (`back: host_dies`) return however the host leaves: only when it died; (d) common.proliferate reads
   `m.data['counters']`, which nothing writes: act on the real counters.
4. After merging rust-port: t2.rs, t3.rs and t5.rs carry local copies of `common.proliferate`; point them at
   common's once it's complete. common.rs's `LOCK_AURAS` duplicates `zur::LOCKS`: use zur's.

Difftest: `python3 rust/tools/difftest.py --gen 1000 --seed 7 --decks t2,t3,t4,t5` (from the repository root).
Commit message: "Rust port, phase 6: the rest of common, partials and fixes; four bug fixes".
