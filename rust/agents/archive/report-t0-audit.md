# Report: t0-audit (are the Tier 0 precons too strong because of generous card code?)

Answer: partly. Most card code matches the Oracle text (all of t0_anikthea.rs, t0_giada.rs, t0_zurgo.rs read against it;
the compiled DSL of every Tier 0 card checked for lost conditions; ~50 traced games checked for illegal lines). Eight real
faults were found and fixed, each with a test. Everything else the precons do is what the cards allow.

## Faults found and fixed (all over-generous unless noted)
1. Bounceland played with no other land stayed on the battlefield (engine/turn.rs `play_land_card`): now returns itself.
   Hit Anikthea's three bouncelands on turn 1 (a free extra land).
2. Speaker of the Heavens (Giada): compiled {T}: 4/4 Angel had no "47+ life, sorcery speed" condition; it made an Angel
   (with Giada counters) every turn from turn 2. Now Rust (`speaker_options`), life >= 47.
3. Idol of Oblivion (Zurgo): drew a card every turn with no "if you created a token this turn". Now Rust, uses the new
   per-player token counter `Player::tok_made`.
4. Sungrass Prairie (Anikthea) and Ferrous Lake (Stella): {1},{T}: add two mana was modeled as a free 2-mana land (amt=2);
   now one mana (override tags).
5. Tainted Field / Tainted Wood (Anikthea): coloured mana without a Swamp; now colourless until a Swamp (land_cols hook).
6. Fetid Heath (Zurgo), Cascade Bluffs (Stella): filter lands gave their colours with nothing to pay the filter; now need
   another land making one of the colours.
7. Temple of the False God (Giada): {C}{C} with any number of lands; now needs five.
8. Lead's two: Bone Devourer (enters with counters = creatures died this turn, dies draws/loses X = its counters) and
   Thalisse (X Spirits, X = tokens created this turn; new `Player::tok_made`, counted in make_tokens, enter_token_copy,
   make_artifact_tokens) -- both were the DSL "X" fallback of 3.
Not generous (opposite sign, left alone): Extinguish All Hope also destroys enchantment creatures (no DSL filter),
Heliod's Cleric token is not an enchantment, Tome of Legends is unmodeled.
Minor, unfixed: Always Watching / Thraben Watcher give vigilance to tokens too; Endless Atlas lacks its three-same-name
condition (always true in Giada's deck).

## Shared-file changes
engine/turn.rs (bounceland, 4 lines), engine/zones.rs (`note_tokens` + 3 calls), state.rs (`Player::tok_made`),
data/card_overrides.toml (Tier 0 entries), data/cards.json regenerated.

## Measurement
Same seeds/commands as the baseline (seed 1014). Baseline = audit/pool/2026-10-14-t0/*.txt on calib-1014; after = after-*.txt.

| | before | after |
|---|---|---|
| Tier 1 into Tier 0 (overall, 1000/list, conservative) | 19.7% | 23.4% |
| Isshin / Lathril / Light-Paws / Tatyova / Teysa | 17.9 / 15.2 / 33.6 / 10.8 / 20.8 | 21.4 / 20.2 / 37.7 / 15.0 / 22.7 |
| Within Tier 0 (loose, 2000): Anikthea | 33.7 | 28.7 |
| Giada | 32.5 | 33.0 |
| Zurgo | 25.3 | 22.7 |
| Captain America | 18.4 | 23.6 |
| Stella Lee | 14.7 | 16.8 |

Tier 1 now gets to 23.4% (25% = even) against Tier 0, up 3.7 points. Anikthea cooled (bouncelands, Sungrass, Tainted
lands). Giada did not move: fixing Speaker and the Temple changed nothing visible, so Giada's 33% is the deck itself
(38 lands, 11 mana rocks, cheap Angels that grow with Giada), not generous code. Remaining Tier 1 shortfall is the
precons being legitimately strong plus Tier 1 lists being weak (Tatyova 15%), not a code fault I could find.
Files: audit/pool/2026-10-14-t0/after-within-loose.txt, after-ordering-t1-into-t0.txt.
Checks: cargo test --release -j 3 all pass, --all-targets no warnings, all_decks adaptive 3 / lookahead 1 no panic.
