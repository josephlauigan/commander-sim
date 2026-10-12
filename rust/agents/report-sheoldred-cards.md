# Report: sheoldred-cards (agents 1 and 2, 10/11)

## Result
`cardgen audit --mine`: `sheoldred: 67 unique cards, 0 not Full`. cards_sheoldred.rs: 50 tests. Full `cargo test --release` passes,
`cargo build --release --all-targets` has no warnings, `all_decks adaptive 3` and `lookahead 1` finish (no panic), 48-game smoke
`--deck sheoldred --pool t3 --profile loose --ai lookahead --jobs 6`: 16.7% wins (CI 8.7-29.6), no panic.

## Finding for the lead
The audit's Full verdicts were STALE for cards that came from the retired candidate decks (glissa, gix, yawgmoth): the card code was
deleted but card_overrides.toml still said Full. 18 such cards in this deck did nothing in a game; they are restored in
impls/sheoldred.rs (list in handoff). Other decks rebuilt from retired candidates may have the same problem: check with a cast test,
not the audit.

## Cards (impls/sheoldred.rs)
- Locust Spray: -1/-1 (kills via `shrink`); cast at an opposing creature the -1/-1 kills, else cycled for {B} after the turn's plays.
- Ribskiff: draw on enter; crew 3 through the Crew event (idle/sick creatures only, never Sheoldred); toxic 2 on combat damage.
- Tome of Legends: page counters (enters with 1, commander enters/attacks +1), {1},{T}, remove: draw. carddb tag `x` removed.
- Diary of Dreams: page counter per instant/sorcery cast; {5}-pages, {T}: draw. Override `dsl = []` drops the compiled {5} draw.
- Leech Collector // Bloodletting: prepared on the first lifegain each turn; sorcery-speed {B} copy, each opponent loses 2.
- Moseo: end step with lifegain returns the best creature card with mana value <= life gained that turn.
- Lethal Throwdown: own options/plan; pays the sacrifice (a modified creature preferred), destroys, draws if it was modified.
  Override `tags = ""`, `dsl = []` so the generic removal path (which skipped the sacrifice) no longer casts it.
- Restless Bloodseeker: override status key fixed to the full "A // B" name.
Tag-only cards verified by cast tests (all work, none modeled): Hullcarver, Multiply by Zero, Depressurize, Overkill, Lash of the
Balrog, Deadly Dispute (via pool.rs spell_options). Dark Endurance is a protection instant used by the new protect hook.

## Part 2
- Memory Jar (jodah.rs `jar`): with key sheoldred and Sheoldred out it is cracked at once (utility 8); otherwise Jodah's rule.
- Toxic Deluge (engine/removal.rs, ai/decks.rs wipe_eval): `sheoldred::deluge_cap` = her toughness-1 (and life-1); X is capped
  and, for that deck only, `deluge_x` is set so the sweep really is -X/-X (before, the AI wipe destroyed everything whatever X).
  Other decks are unchanged. X life is paid (tested).
- Bitter Triumph, Korlash: tests (3 life paid; P/T = Swamps, regenerates).
## AI plan
Sheoldred prio 96; Arcane Signet/Fellwar Stone 92 while she is not out; Helm/Plate 78 once she is; the three protection instants prio 0
(only cast as responses). `sheoldred_special` offers equip Helm/Plate onto her at utility ~9. `sheoldred_protect` answers
destroy/damage removal at her with the cheapest of Masterful Flourish, Battle-Rage Blessing, Dark Endurance (not exile or -X/-X,
which indestructible does not stop; Helm's hexproof already stops targeted removal). `sheoldred_hold` keeps mana up for one.
Cost of Brilliance / Vault Plunderer aim at the lowest-life opponent while she is out (agent 1).

## Shared-file changes
state.rs (`Player::bloods`), engine/zones.rs die_rest hook (Perigee Beckoner), impls/rules.rs (`replaced_tag_engine` + Seymour Flux),
impls/jodah.rs `jar` (sheoldred branch), engine/removal.rs (Deluge X cap, local `deluge_x`), ai/decks.rs wipe_eval ("minus" arm),
ai/brain.rs hold_value (sheoldred branch, 6 lines), data/carddb.txt (Tome of Legends tag x removed), data/card_overrides.toml
(verdicts), data/cards.json + decks.json (cardgen export).
## Not modeled / simplifications
Bitter Triumph always pays 3 life (never discards); Dark Endurance's {1} discount for a blocking creature ignored; Ribskiff crew
happens once before the first combat.
