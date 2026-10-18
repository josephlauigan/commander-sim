# Handoff t6-dack
Batch: Dack Fayden Helping Hand; Preston the Vanisher; Fiend Hunter; All-Fates Stalker; Radiant Solar; Touch the Spirit Realm; Gideon of the Trials; Paladin Class
Deck key: dack-fayden-white-stax. Code: impls/t6_dack.rs; tests: tests/cards_t6_dack.rs (seat 0 dack, seat 1 seph).
## Done
- Fiend Hunter (Full)
- All-Fates Stalker (Partial: warp recast from exile = engine need)
- Touch the Spirit Realm (Full; channel via g.eot_returns)
## Next (in order)
Radiant Solar (venture on grudge map), Preston, Dack, Gideon, Paladin Class
## Gotchas
- Banisher etb hooks use the enter call's `p` as "you", not perm owner (Dack donates before queued triggers run).
- Use common::oring_exile / oring_return; override dsl = [] in data/card_overrides.toml so the compiled permanent exile does not double.
- Workflow: edit overrides, `./target/release/cardgen export --offline`, `cardgen audit --deck dack-fayden-white-stax`.
- marchesa::steal(g, p, m, false) changes control (Perm.owner = controller, Perm.orig = real owner). Goad: t0_zurgo private fn, DataKey::Goaded, lasts until goader's next turn only.
