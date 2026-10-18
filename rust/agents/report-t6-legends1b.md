# Report t6-legends1b
- Ioreth of the Healing House: Full (both untap modes; AI taps for a gain of >= 1 mana, max 6 uses a turn per player).
- Marvin, Murderous Mimic: Partial (Ioreth's abilities only; other creatures' abilities not copied).
- Shang-Chi, Master of Kung Fu: Partial (dork {C}; restricted mana and engine-wide ability haste missing). `ability_haste(g, p)` is the helper.
- Tyvar, Jubilant Brawler: Partial (+1 and -2 done via walker; haste static only honored by this batch).
- Loki's Scepter: Full (steal until end of turn via marchesa::steal; mana via rock tag).
Engine needs: rust/agents/engine-needs-legends1b.md. Shared files touched: impls/mod.rs, data/card_overrides.toml (+ regenerated data exports).
