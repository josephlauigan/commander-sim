# Report: t6 counters

Card code: crates/sim-core/src/impls/t6_counters.rs (registered last in impls/mod.rs); tests: tests/cards_t6_counters.rs (20);
data: data/card_overrides.toml (13 entries) and data/cards.json (via `cardgen export --offline`). Engine needs: engine-needs-counters.md.
Nothing shared touched beyond the two registration lines in impls/mod.rs.

Full: Dizzy Spell (-3/-0 resolve, defender use like Icy Reception, Transmute at sorcery speed), Into the Flood Maw (bounce; gift Fish when the target is a noncreature).
Partial: Abeyance (draw + turn flag; engine must enforce), Daze, Flare of Denial, Snapback (hard-cast only; alt cost written),
Disrupting Shoal (ctr tag removed: engine would counter anything for UU), Mindbreak Trap (hard-cast ctr=any ctrexile), Bilbo's Gambit (ctr=any remand),
Tibalt's Trickery (plain counter; side effect written), Misdirection, Commandeer, Sudden Substitution (stack effects written; AI never casts them).
Note: all 13 went into one commit, not one per card (the cards share one file and one test file).
