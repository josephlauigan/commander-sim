# Report: the rest of common, partials and fixes (+ four bug fixes)

- Ported (commit 57c168d, previous agent): pillowfort taxes, stax, planeswalker framework, Ballista, graveyard hate, Otawara,
  Reflector Mage, Spark Double, Frost Titan, Dream Trawler, rebound, Narset, Static Net/Powerstone, DYN_MANA, proliferate,
  Sign in Blood, saga_step/CI.SAGA, Coat of Arms/Bloodline Keeper, Golgari Charm, Hunter's Insight, Karn's Bastion.
- Bug fixes in Rust only, each with a test in tests/cards_common2.rs: Druid Class levels 1->3; Mardu Charm discard mode; Angelic
  Destiny returns only when the host died; proliferate acts on the real counters.
- This session: merged rust-port (conflicts in state.rs, hooks.rs, cardcode.rs, common.rs; unions kept, duplicate Unearth variant and
  loyalty_extra field dropped; cardcode::self_cost = Emry's match arm, falling through to the `self_cost` slot).
- t3.rs and t5.rs local proliferate copies removed: they call common::proliferate(g, p, 1) (t2.rs had none).
- common.rs LOCK_AURAS removed: uses zur::lock_kind.
- Difftest (--gen 1000 --seed 7 --decks t2,t3,t4,t5): AI 0, build 1 (random 959: Opposition Agent takes a Solemn Simulacrum basic
  search into hand; Forest vs Swamp, a random basic pick the harness files under build).
