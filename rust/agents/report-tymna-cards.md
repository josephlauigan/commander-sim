# Report: tymna-cards

Branch `tymna-cards` (from `tymna-thrasios`). Audit of the 121 cards (`agents/tymna-cards-audit-{before,after}.txt`):
before 93 Full / 10 Full-auto / 3 Approximate / 4 Partial / 11 Unmodeled; after 114 Full / 6 Full-auto / 1 Approximate.

## Tooling
- `cardgen audit --cards FILE [--fetch]`: per-card verdicts; `--fetch` pulls missing cards from Scryfall into the cache.
  Abigale is cached under both names (as the other prepared cards are); the deck list's short name resolves once the
  decks are registered.

## Cards (module `impls/tymna_cards.rs`, tests `tests/cards_tymna.rs`, 27 tests; overrides in data/card_overrides.toml)
- Thrasios: {4} scry, reveal, land tapped / draw. AI: end-of-turn window (the turn before its own) first, as often as
  affordable; second main phase last (utility -2, just above "stop"); never in the first main phase.
- Kraum: opponent's 2nd spell each turn draws. Black Widow: opponent's 2nd draw each turn, counter + draw.
- Abigale (both names): prepared on a creature spell cast; {1}{W/B} copy of Heroic Stanza (counter on best evasive creature).
- Cuboid Colony (increment, mana spent = mana value), Elemental Mascot (compiled +1/+0; added the 5+ exile-and-play),
  Falcon and Redwing (autotag made it enter with two tokens: fixed, now Birds equal to damage + counter), Photon
  (floating mana of the colour the hand wants), Faeburrow Elder (self_pt + one mana unit per colour), Paradox Surveyor,
  Spectral Lynx (regenerate {B}), Twisted Fates (counters always on the caster's creatures), Nightveil Predator (data only),
  Sphinx of New Prahv (modeled as ward {2}: same effect for spells; engine charges it, AI won't aim what it can't pay).
- King Darien: sacrifice ability (tokens hexproof + indestructible) used against removal on a token worth more than the
  King and against destroy sweepers (`protect`, `darien_wipe`). Razia: redirects burn <= 3 aimed at a creature that would die;
  **combat damage is not redirected** (the one card left Approximate: an 8-mana top end that rarely survives to use it).
- Hand-checked Full-auto cards that were WRONG and are fixed: Dakkon (compiled "per X" gave a 1/1: now lands count),
  Elenda (compiled death trigger always made 3 tokens: now X = her power), Embrace the Paradox (land now enters tapped),
  Silverquill Charm (tag path let the exile hit any creature and drain double-gained life: now a three-mode hand play),
  Moss Diamond and Star Compass (engine ignores "enters tapped" on artifacts: card code taps them; Compass makes only the
  colours of our basic lands).
- Tymna (legends.rs) checked, no change: every combat hit by any of your creatures marks that opponent, she offers to pay
  that much life to draw that many, once per postcombat main; tests exist in cards_legends.rs.
- Evasion/trigger spot-checks: Reconnaissance Mission, The Aetherspark, Lightning, Terror of the Peaks, Bastion
  Protector were Full in the before-audit and left as is (not re-derived this run).

## Left as Full-auto with known deviations (cards existing decks also run, so I did not touch them)
- Memory Trap: exile is permanent (Oracle: until it leaves). yshtola runs it.
- Potioner's Trove: the {T} gain-2 ability ignores "only if you cast an instant or sorcery this turn". sheoldred runs it.
- Make Your Move: hits creatures with power 4+ only (Oracle also any artifact or enchantment). captain-america runs it.
- Wander Off, Mana Leak, Lyra (tested): correct.

## Shared-file changes (all small)
- `impls/mod.rs`: module + register line. `ai/decks.rs`: one line each at the top of `protect_response` and
  `wipe_response` (no-ops unless Razia / King Darien are on the battlefield). `engine/mana.rs`: Star Compass colours
  (name-guarded like Fellwar Stone). `cardgen/src/main.rs`: `--cards`, `--fetch`. `tests/cards_legends.rs`: the Sisay
  test now has a land so Dakkon (0/0 without one, per Oracle) survives.
- `data/scryfall_cache.json`, `data/cards.json`: new cards; **these will conflict with partner-engine's branch** (both
  fetch cards): regenerate with `cardgen export` after the merge.

## Effects on existing decks
Shared engine code is a no-op for them. Card fixes change games only for decks that run the card: Falcon and Redwing,
Photon, Black Widow (captain-america-avengers-assemble, t0 pool; Falcon no longer enters with two Birds), Elenda
(teysa-orzhov-aristocrats). No same-seed before/after comparison was run for those.

## Checks
`cargo test --release` all pass; `cargo build --release --all-targets` no warnings; `all_decks adaptive 3` (180 games)
and `all_decks lookahead 1` (60 games) finish with no panic. No smoke run of the decks (not registered yet).
