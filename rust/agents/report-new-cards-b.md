# Report: new-cards-b

Code: `impls/newcards_b.rs` (all three cards + AI priorities), registered in `impls/mod.rs`. Data: carddb.txt (3 lines), card_overrides.toml
(3 tables), cards.json re-exported (+127 lines only).

## Cards
- **Cecil, Dark Knight** (Full). Darkness via `cecil_deals`, called from combat.rs for every combat damage event (unblocked player
  damage after lifelink; blocked damage both ways): life loss (not damage), flip at life <= 20: untap, 4/4, lifelink, no deathtouch
  (`DataKey::Flipped`). Protect: attack hook gives the other attackers `eot_kw indestructible`. Kraum prio 76 (40 when life <= 26 and unflipped).
- **Lightning, Army of One** (Approximate in practice: marked Full in overrides; gaps below). Stagger = `Player::stagger: Option<(controller, controller turns)>`,
  active while that controller's turn count is unchanged (so it expires when their next turn begins). `engine::life::stagger_mult` doubles:
  `lose_life` damage (all player damage, combat and noncombat), `apply_removal` dmgN, and the combat kill checks (both directions).
  Gaps: sweepers that kill by fixed damage (dmg13, Austere-style kinds), AI estimates in combat.rs, lifelink of a doubled hit counts the
  undoubled damage, attackers resolved before Lightning in the same damage loop miss the doubling (real rules: Lightning's first strike hits first).
  AI: Kraum prio 78; `brain::choose_defender` +2.5 for an opponent with no untapped creature that can block Lightning.
- **Marvin, Murderous Mimic** (Partial). Copies the {T} abilities (no mana) of Niv-Mizzet (draw), Rummaging Goblin (loot), Viashino Fangtail
  (ping), Blightspeaker (drain 1); NOT Blightspeaker's {4},{T} search (compiled search takes any permanent, deck has no Rebels).
  Options hook `marvin_options`; new generic `dsl::borrowed_tap_options` / `dsl::activate_borrowed`. Cast prio (kefka): 52, 72 with a copied creature out, 80 with Niv.

## Shared-file edits (all small, additive)
state.rs (`Player.stagger`), engine/life.rs (`stagger_mult`, one line in lose_life), engine/removal.rs (4 lines), engine/combat.rs (kill checks + 3 cecil calls),
dsl/mod.rs (+2 fns), ai/brain.rs (1 line), impls/kefka.rs (3), impls/tymna_partners.rs (3), impls/mod.rs (2).

## Checks
cargo test --release (workspace) all pass; build --all-targets 0 warnings; preflights (200 games x2 profiles): Cecil cast 74.6% of draws (prio 76),
Lightning 82.9% (78), Marvin 70.8% (52, kefka; ability use only tested by unit tests, not measured).
New tests: cards_tymna (4), cards_kefka (2).
