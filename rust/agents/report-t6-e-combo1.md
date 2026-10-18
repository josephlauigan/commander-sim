# Report t6-e-combo1 (combo framework + Godo combos)
Framework (impls/combos.rs): `combos::all()` = BASE (the 14, unchanged) + `t6_combos_*::COMBOS` slices (add one line in
`all()` per new module). `Combo` fields are pub, with `steps: &[Step]`. Step{label, kind, src, pay, run, keys}; kinds:
Spell (run casts: normal counter window), Trigger (pay, trigger_window with src: ability-answer path), Ability
(ability_window), Permanent (run, then `interrupted` on keys), Engine (the real engine ran it with its own windows; run
checks the outcome). `run_steps`: an answered step stops the combo (log + combo_stopped); all passed: finish/win.
Combos without steps keep the single window. combo_options lets a step combo through when its `ready` is true.
Godo (impls/t6_combos_godo.rs, 9 combos): Hammer out; Brass Squire; Magnetic Theft; {5} equip (each priority-chained so
the cheaper route is the only one ready); Camera (copy fetches Hammer, original Helm); Hammer first + copy spell
(Twinflame/Molten Duplication/Heat Shimmer/Electroduplicate); + Cursed Mirror / Sculpting Steel; + Jaxis; Dualcaster Mage +
copy spell. Steps: cast Godo (Spell) -> enter trigger (Engine) -> attach (Engine/Ability/Spell) -> Helm combat trigger
(Trigger) -> Godo+Helm survive (Permanent) -> win. Real loop not played (shortcut).
Other changes: t6_godo: `godo_fetch` split out, `godo_wants`/`COPIERS` pub(crate), COPIERS += Jaxis, Sculpting Steel (Godo
deck only: Hammer first with those in hand); partials::jaxis_blitz_go pub. No other deck changes.
Tests: tests/engine_t6_combo1.rs (22), full suite 2425 pass.
Gaps: Helm's combat trigger window is only tested synthetically (an earlier Ertai/Tidebinder would stop at Godo's enter trigger
first); Welder+Engineer, Godo-already-out rebuild, Basalt+Kinnan not built.
