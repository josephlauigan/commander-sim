# Report: rules.rs, rules2.rs, t3.rs (Tier 3)

- WIP commit 7cfbf73 already held the port of the rest of rules, rules2 and t3 plus both bug fixes (tests in cards_t3.rs:
  Bloodchief's Thirst kicker, Legion's Landing -> Adanto; fix comments in rules.rs / rules2.rs).
- Merged rust-port (conflicts in state.rs, hooks.rs, engine/hooks.rs, cardcode.rs: both sides kept; Embercleave's
  self_cost arm added next to Emry's; duplicate Crew field/arm removed).
- t4.rs now calls t3::sac_worst_permanent and rules2::devotion (private copies removed).
- t3.rs keeps its own `proliferate` (common.rs has none; t5.rs has a third copy: unify when common-rest lands).
- t2.rs's dead registrations (Charming Prince, Displacer, Whir, Beseech, Wishclaw) are replaced by rules2's registrations.
- difftest (t3,t2, 1000, seed 7): 0 AI, 0 build; card-code differences all belong to common.py cards (Teferi HoD,
  Elspeth, Liliana DM, Ghostly Prison, Cabal Coffers, Elephant Grass ...), i.e. common-rest.
