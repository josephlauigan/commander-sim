# Handoff t6 legends1
Status: on card 1 (Derevi). Code in impls/t6_legends1.rs (etb + combat_damage tap/untap), tests in tests/cards_t6_legends1.rs; registered in impls/mod.rs.
A cargo test build is running (first build of the test binary, slow). Not yet committed.
Next: when Derevi passes, add overrides in data/card_overrides.toml (Partial: command-zone ability needs an engine hook), run `cardgen export --offline` from repo root if tags change, commit.
Then in order: Emiel, Selvala, Agatha's Soul Cauldron, Deadpool, Ioreth, Marvin, Shang-Chi, Tyvar, Loki's Scepter.
Plan: Selvala/Shang-Chi mana via extra_mana slot (see t6_mana Jeweled Amulet); haste-for-abilities helper (Shang-Chi/Tyvar) in my file.
