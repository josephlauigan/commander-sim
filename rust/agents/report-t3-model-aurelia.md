# Report: Tier 3 cards for aurelia-boros-extra-combats

(Saved by the main session from the agent's final message on 2026-10-24: the harness refused the agent's own write of
this file. Merged into trace-fixes as 3ad617b5.)

Branch `t3-model-aurelia`. Commits: bc8903dc (the modeling) and d5e8d116 (the tests, and the Fighter Class tag fix). The build has no warnings. `cargo test --release`: 184 suites, 3046 passed, 0 failed. The new test file is `rust/crates/sim-core/tests/cards_t3_aurelia.rs` with 23 tests.

### Card status (before -> after)
- **Blade Historian:** Partial -> Full-auto. New pattern "attacking creatures you control have X", using a new `attacking` filter on the creatures in `g.in_combat`.
- **Avacyn, Angel of Hope:** Partial -> Full-auto. New pattern "(other) permanents you control have X"; noncreature permanents and lands get it too. Lands are covered in `destroy_land` through `dsl::lands_have_kw`.
- **Firemane Avenger:** Partial -> Full-auto. New battalion trigger (`min_atk` 3).
- **Aurelia, the Law Above:** Partial -> Full-auto. New trigger "a player attacks with N or more creatures" (any player's attack, `min_atk`). New effect "deals N damage to each of your opponents".
- **Chance for Glory:** Unmodeled -> Full. Your creatures gain indestructible, then you take a losing extra turn (the Final Fortune mechanism). The AI casts it only in its second main phase, and only when a rough count of next turn's damage beats every opponent's life.
- **Trouble in Pairs:** Unmodeled -> Full. Opponents' extra turns are skipped (`skips_extra_turn`). You draw when an opponent attacks you with two or more creatures. You also draw once per opponent and turn when they draw their second card or cast their second spell.
- **Wear // Tear:** Unmodeled -> Approximate. It is tagged `rem=destroy tgt=ae`, so the AI casts it as removal. On resolve it fuses with {W} if the other half has a target. Left out: Tear alone costs {1}{R} instead of {W}.
- **Angelic Skirmisher:** Partial -> Full. At each combat it gives your creatures first strike if a fight is possible, lifelink if you attack into an open board, and vigilance otherwise.
- **Battle Angels of Tyr:** Partial -> Full. Myriad was already handled by the engine. Combat damage now gives a draw, a Treasure and 3 life when that player has the most cards in hand, lands and life.
- **Boros Garrison:** Partial -> Full. It gets the override tag `bounceland`, the engine's existing bounce-land rule.
- **Bruna, the Fading Light:** Partial -> Full. The cast trigger returns the best Angel or Human creature card from your graveyard. Meld can't happen because the deck has no Gisela, the Broken Blade.
- **Fighter Class:** Partial -> Approximate.
  - Its enters trigger searches for an Equipment.
  - Level 2 makes equip abilities cost {2} less (in `cloud::equip_cost`).
  - Level 3 forces an opposing creature to block each unblocked attacker that kills it and survives.
  - Left out: those targets are chosen when blocks are declared, not when attackers are.
  - The automatic `tut` tag was removed.
- **Gisela, Blade of Goldnight:** Partial -> Approximate. `gisela_scale` doubles damage to opponents and their permanents and prevents half of the damage to you and yours (rounded up). It covers combat damage to players and planeswalkers, the kill checks in blocked fights, burn, and `dmgN` removal and its legal targets. Left out: trample's lethal assignment, infect counters on creatures and board-wipe damage keep the printed amount.
- **Iroas, God of Victory:** Partial -> Full. It is overridden to type E and becomes a 7/4 creature at red and white devotion 7 or more. Damage to your attacking creatures is prevented (`values::no_damage`). Menace was already compiled.
- **Odric, Lunarch Marshal:** Partial -> Approximate. At each combat your creatures share their keywords. Deathtouch uses a marker that is cleared at the end step. Left out: skulk.
- **Slayers' Stronghold:** Partial -> Full. In the first main phase the AI gives haste to its biggest new creature of power 3 or more.
- **Sunhome, Fortress of the Legion:** Partial -> Full. In the first main phase the AI gives double strike to its best ready attacker of power 3 or more that no opposing creature can block.
- **Sunforger:** Partial -> Approximate. {R}{W}, unattach: find a red or white instant with mana value 4 or less and cast it free. Left out: the AI only fetches removal (against a target worth 4 or more), never protection.

### Shared files changed (all small and additive)
- `cardgen/src/dsl_parse.rs`: the Tier 3 section. The existing "you attack with two/three or more" trigger now keeps its count; no current card uses it.
- `sim-core/src/dsl/model.rs`: added `Filter.attacking` and `Triggered.min_atk`.
- `sim-core/src/dsl/mod.rs`:
  - `matches` reads `attacking`.
  - `trigger_matches` reads `min_atk` and the new source `any_player`.
  - `has_kw` lets a permanent-wide filter reach noncreature permanents.
  - New function `lands_have_kw`.
- Engine files:
  - `engine/life.rs` (Gisela, non-combat damage).
  - `engine/combat.rs` (Gisela on combat damage and the kill checks).
  - `engine/removal.rs` (Gisela on `dmgN` removal and legal targets).
  - `engine/values.rs` (`no_damage`: Iroas).
  - `engine/turn.rs` (`run_extra_turns`: Trouble in Pairs).
- Card-code files: `impls/cloud.rs` (equip cost), `impls/fixes.rs` and `impls/lands.rs` (`destroy_land`: Avacyn), `impls/mod.rs` (registration).
- Data:
  - `data/card_overrides.toml`: Boros Garrison, Iroas, Wear // Tear, Fighter Class.
  - `data/audit.toml`: new [card_notes] entries.
  - `data/cards.json` re-exported; only these cards changed.

### Patterns added
- Static: "attacking creatures you control have X".
- Static: "(other) permanents you control have X".
- Trigger: battalion, "~ and at least two other creatures attack".
- Trigger: "a player attacks with N or more creatures".
- Effect: "deals N damage to each of your opponents".

### Left Partial, and known limits
- Needleverge Pathway is still Partial (back face mana); it wasn't in the brief's list.
- The AI's attack planning doesn't foresee Blade Historian's double strike.
- Chance for Glory is never cast in response to anything.
- Sunforger never fetches protective instants.

### Tier 3 scorecard (loose, adaptive, 4000 games, this branch alone)
1 of 9 targets pass. Seats 27.2 / 26.6 / 23.9 / 22.3% (target 31.3 / 27.8 / 21.8 / 19.1); median win round 11 (8);
combo or alternate wins 2.7% (20.0%); damage wins 86.8% (62.2%); commander damage 10.4% (17.4%); poison 0.0% (0.4%).
Each deck's share of the decisive games it sat in: Atraxa 35.4%, Aurelia 32.5%, Tergrid 32.5%, Korvold 14.9%, Marwyn
9.7%. No run was made before the change, so there is no delta. (The branch predates the poison-death fix, so its
poison wins read 0.)
