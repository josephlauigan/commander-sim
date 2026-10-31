# Report: Phase 3, PR 4: a mana pool with restrictions (branch `phase3-mana`)

Brief: `rust/agents/phase3-mana-exile.md`. Design (census, bugs, what doesn't fit, decks touched):
`audit/phase3/mana-design.md`. Handoff: `audit/phase3/mana-handoff.md`. Code: `rust/crates/sim-core/src/engine/pool.rs`,
`tests/mana_pool.rs`, `examples/mana_cost.rs`. Commits: b4de6614 (Stage A), 2d562035 (Stage B), cc71b6cc (Stage C).
No sims (JD's rule).

## Stage A (results identical)
`Pool`: a sorted list of mana entries per player (kind W U B R G C or any colour, a restriction, how long it stays,
an amount). `Player::floating` became `Player::mana`; about 45 places that add mana, and every read and spend, go
through it. The seven `Source::Float*` variants became one `Source::Pool(key)`; the planner sees floating units in the
old order, so its choices don't change.

## Stage B (results identical)
Every source restriction is a `Restr` checked by `mana::restr_ok(g, p, r)`, looked up once at load
(`CardDef::mana_restr`): Mishra's Workshop, Powerstones, Mightstone, Giada, Hydro-Channeler, Rivaz; and the "{C}, or
any colour for X" lands (Plaza, Unclaimed Territory / Cavern, Secluded Courtyard, Haven, Voldaren Estate, Avengers
Tower, Delighted Halfling). Deleted: `art_mana_ok`, `dragon_only_mana`, the card-name checks in `mana_units`.

## Stage C: behaviour fixes (8 tests)
1. **Pools empty as each step ends (rule 500.4)**: after main 1, combat, main 2, the end step, and in the start
   step after the draw. Kept all turn: Karn, Birgi, Klauth, Savage Ventmaw, Photon, Rousing Refrain. Mana Drain's
   mana arrives at the start of main 1 (it came in the upkeep); Carpet of Flowers triggers in main 1; Black Market's
   mana empties after main 1, as printed.
2. **A source's leftover keeps its restriction** (Mishra's Workshop paying a one-mana artifact used to float {C}{C}
   for anything; now only for artifact spells). Same for Mightstone, Powerstones, Giada, Rivaz, Hydro-Channeler.
3. White floating mana is spent (Photon's was never used by the AI).
4. Cavern of Souls and Delighted Halfling's coloured mana makes the spell uncounterable.
5. Maelstrom of the Spirit Dragon gives any colour only for a Dragon spell.

## Checks
- `cargo test --release --no-fail-fast`: 3663 passed, 0 failed, 3 ignored (rerun by the coordinator). Build 0
  warnings. Practice 104. cardgen clean, no data change.
- `clone_cost` 2.91-2.94 µs (base 3.03). `mana_cost` within noise of the base, except `mana_units` on an empty pool,
  about 5% slower (the restriction test on rocks and dorks).
- Practice: a person's payment moves all the engine's floating mana into their pool, white included; black floating
  mana is no longer listed as a source to tap ("FB").

## Decks touched (for the recalibration)
- Step emptying: every deck at every tier, all of JD's decks.
- Kept mana: Birgi (prosper t4, rograkh-silas t6); Klauth, Ventmaw, Maelstrom, Mana Drain (miirym t5; Mana Drain also
  kinnan, urza, zur t5); Photon (captain-america t0); Rousing Refrain (stella t0); Black Market (teysa t1).
- Restricted leftovers: Workshop (**JD's karn**, dack t6); Mightstone (**JD's karn**); Giada (giada t0, aurelia t3).
- Riders: Cavern of Souls (marwyn t3, edgar t4, krenko t4, miirym t5, dack t6, kraum-tymna t6); Delighted Halfling
  (chulane t4, kinnan t5, kinnan-bonder t6, rograkh-thrasios t6, sisay t6).

## Not fitted yet
- Practice mode keeps its own pool, emptied between steps, so "until end of turn" mana empties with the step for a
  person. Merging the two pools is the next step.
- "Any colour" mana is still chosen when spent, not when made.
- Shang-Chi, Castle Garenbrig, Master of Dark Rites, and the "or an ability of a ... source" riders don't fit: a
  payment doesn't record whose ability it pays for. Riders don't float.
