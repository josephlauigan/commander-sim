# Tymna/Kraum (kraum) — AI audit, phase 1
Strategy: two Partner commanders. Tymna draws one card per opponent dealt combat damage (pay X life); Kraum draws when an
opponent casts a second spell in a turn. Attack taxes (Ghostly Prison, Propaganda, Crawlspace, Windborn Muse, Archon,
Baird) keep the table off her while evasive creatures (Adeline, Hero, fliers, Dauthi, Lightning) hit every opponent; Elspeth,
the Titans, Angels and dragons finish; Austere Command / Avenge / Sunblast reset a lost board; Underworld Breach recasts
the dead after a wipe. Tier t5 in `data/my_decks.toml` (the deck doc still says T3).
Priority function: `kraum_prio` -> `partner_prio` (`impls/tymna_partners.rs:46`), shared with thrasios; commanders first
(`commander_first`, `:259`); attack spreading `spread` (`:187`); tutors `tymna_cards::tutor_pick` (`:1286`, "any" only);
equipment `kraum_special` (Flail only). Protection and wipe responses are the *pool* AI's (`kraum` is not a "main" deck,
`ai/mod.rs:52`): `ai/pool.rs` `PROTECTORS` (`:886`). Plan config: none (the analysis prints "Plan:" blank and
"Online at all: 0.0%", so plan timing is not measured for this deck).
Evidence base: `--analyze --pool t5 --ai adaptive --games 300` (conservative 30.0%, loose 30.7%); three big
`--swap` runs (21+21+17 cards swapped for unrelated cards, 900 games, loose, `--pool t5`) used only for their
"Swapped OUT" tables, which give each card's drawn / cast / stuck / win-when-cast in the *unchanged* baseline games
(baseline 33.4-33.5% loose); 40 traces (`--trace 1..40`); and the code. Numbers: drawn% = games the card was drawn in;
cast% = of those; stuck% = drawn but not cast.

## Problems (ranked by likely impact on win rate)
1. **Underworld Breach is never cast, and its recast line is blocked even if it were.** Drawn 19.8%, cast 0.0%, stuck
   100% (swap table). `partner_prio` has no Breach rule, so it falls to `generic_prio`, which has no branch for
   `Tag::Breach` for a non-creature, non-combo permanent and ends at `0` (`ai/pool.rs:326`). The escape side exists
   (`breach_gc_options`, `impls/mine.rs:1917`) but only runs once Breach is on the battlefield. Even then
   `breach_candidates` (`ai/decks.rs:335-345`) skips every wipe, counterspell and X card, so the doc's main line,
   "recast Austere Command / Avenge after a wipe", can't happen; only Elspeth, Titans, Angels and the like qualify.
   Breach is also sacrificed at the end step (`engine/turn.rs:686`), so it must be cast and used the same turn. Effect now:
   a Game Changer slot that is a dead card in 1 in 5 games. Fix: in `partner_prio`, give Breach a priority only when
   `breach_candidates(g,p,false)` is non-empty with {R} and 4+ cards in the graveyard; allow the deck's wipes in the
   candidate list (or take the card out: that is a list decision, not mine).
2. **Dark Endurance is never cast.** Drawn 22.8%, cast 0.0%, stuck 100%. The doc protects Tymna with it. `kraum` uses
   the pool protection list (`PROTECTORS`, `ai/pool.rs:886`), which has no Dark Endurance entry (the only code for it is
   Sheoldred's, `impls/sheoldred.rs:1348,1450`), and nothing else casts it. Tymna is the most removed card (0.18-0.19
   per game, vs 0.05-0.08 for Kraum, analysis). Fix: add a `How::Indes`, `Scope::One` entry ("Dark Endurance", {1}{B}) to
   `PROTECTORS`, or route kraum to `sheoldred_protect`'s shield logic.
3. **Tymna hits about one opponent per turn, not three.** "Opponents hit per Tymna turn 1.03 (cons.) / 1.05 (loose)",
   "Tymna cards drawn per game 3.2-3.4 over 3.4-3.5 Tymna turns" (analysis). The doc's whole engine is hitting every
   opponent (Adeline, Hero, evasive bodies). `spread` (`:187`) only reassigns attackers that already exist and
   "connect" (no untapped blocker), and Tymna lands on turn 4.2-4.3 (doc: turn 3). Not a single-card bug; likely a mix of
   too few bodies and a conservative attack. Suggested check: count attackers per turn while Tymna is out, and add
   attackers-per-opponent to the priority of the 2-3 mana evasive creatures once Tymna is in play.
4. **Diabolic Tutor fetches Conqueror's Flail every time it can** (4 of 4 traced casts, turns 9-13). `tutor_pick`
   (`tymna_cards.rs:1286`): after the land and wipe rules, "cres >= 2 and mine+2 >= theirs" returns the Flail before any
   finisher, and the finisher list is only Terror, Lyra, Gilgamesh, Sphinx (never Titans, Baneslayer, Elspeth, dragons).
   The doc says Elspeth when behind, a Titan or Terror to close. The Flail is also in the deck's list as a 29%-drawn card
   cast 86% of the time already, so the tutor adds little. Fix: drop the Flail line unless the table can cast spells on
   your turn that matter (combo); add Grave/Sun Titan and Baneslayer to the list. Diabolic Tutor is cast in 44% of the games
   it is drawn.
5. **Attack-tax creatures are valued as vanilla bodies.** Baird, Steward of Argive: drawn 22.4%, cast 38.1%
   (stuck 62%); Archon of Absolution (flier, gets the evasive 78) 76.4%; Ghostly Prison 67.8%, Propaganda 62.7%, Crawlspace
   72.8%, Windborn Muse 73.2%. `generic_prio` returns the creature value (42 + 2*power, `ai/pool.rs:271`) before it reaches
   the pillowfort branch (`:276-283`, 45-70 by opponents' power), so Baird gets about 46 instead of the pillowfort value.
   The doc says Baird stacks with the others. Medium confidence (mana also limits a 4-drop; Hero of Bladehold, same
   cost and a vanilla body, is cast 40.9%). Fix: in `partner_prio`, give `attack_tax`-hook creatures the pillowfort value.
6. **Model gaps on creatures that are cast often.** From `--cards` and `grep`: Lightning, Security Sergeant has no menace
   (tags `leg human pow=2 tgh=3` only) and no "play the exiled top card" (no code by name); Firebird, Blazing Ranger has no
   attack pump (no code by name); Nightwind Glider's protection from black is missing (tags: `fly`); The Wasp's ETB/attack
   are modeled (`blue_cards.rs:542`); Dauthi Voidwalker is Approximate (`common.rs:2284`). Lightning is cast in 79.6% of the
   games it is drawn and Firebird in 80.2%, so these gaps show in many games: Lightning counts as non-evasive in
   `evasive_card` (no menace keyword) and so isn't picked for Tymna's spread. Win rate when cast is only 55.1%
   (Lightning), 51.3% (Firebird), 46.6% (Nightwind Glider, the lowest). Fix: add the text to `card_overrides.toml` or code.
7. **The 5-6 mana top end is cast rarely** (cast / stuck of the games drawn): Grave Titan 20.7%, Sun Titan 22.7%,
   Elspeth 27.6%, Elenda 27.7%, Sunblast 29.0%, Thundermaw 31.9%, Marchesa 31.7%, Gilgamesh 35.4%, Lyra 37.4%,
   Terror 37.7%, Glorybringer 38.3%, Baneslayer 46.6%, Serra 44.4%. They win 63-84% when cast, so the issue may be mana
   (mana on turn 4 is 4.0 and on turn 6 is 5.7; Kraum himself lands on turn 6.5, in 60% of the games) rather than priority;
   I did not find an AI cause, so this is a list/mana question for phase 2 (unknown).
8. **Tooling/doc mismatches.** `--cards` prints "not modeled" for Kraum's draw, Black Widow, Smaug's attack damage,
   Elemental Mascot's Opus and Gilgamesh, but the code has them (`tymna_cards.rs:135,149,246,943`, `smaug.rs`); the
   deck doc says "matched tier T3" while `my_decks.toml` says t5.

## All cards
Drawn / cast / stuck are the baseline games (loose, t5, 900 games) from the swap tables; "win" is the win rate when cast
(correlation). Model status is from `--cards` and the code.
| Card | Model | Valuation | Evidence / note |
|---|---|---|---|
| Tymna the Weaver | Full (`legends.rs`) | deck (91 - tax) | cast 86.7-88%, win when cast 33.5%; removed 0.18-0.19/game; lands T4.2 (problem 3) |
| Kraum, Ludevic's Opus | Full (`tymna_cards.rs:135`); `--cards` stale | deck (84 - tax) | cast 61-63%, win when cast 45%, lands T6.5 |
| Adeline, Resplendent Cathar | Full (combat.rs tokens) | generic-ok | 23.0 / 61.4 / 38.6, win 63.8 |
| Archon of Absolution | Full (tax hook) | deck (evasive 78) | 22.6 / 76.4 / 23.6 |
| Aven Mindcensor | Full (`partials.rs:1360`) | generic-ok | 18.6 / 63.5 / 36.5 |
| Baird, Steward of Argive | Full (tax hook) | generic-WRONG | 22.4 / 38.1 / 61.9 (problem 5) |
| Baneslayer Angel | Full | generic-ok | 21.4 / 46.6 / 53.4, win 73.3 |
| Bastion Protector | Full | deck (74 with both commanders) | 22.7 / 61.3 / 38.7 |
| Black Widow, Agile Avenger | Full (`tymna_cards.rs:149`) | generic-ok | 20.3 / 83.6 / 16.4 |
| Dauthi Voidwalker | Approximate | generic-ok | 21.8 / 65.3 / 34.7; shadow counted evasive only by keyword |
| Elemental Mascot | Partial (Opus pump; `:246`) | generic-ok | 21.9 / 71.6 / 28.4, win 50.4 |
| Elenda, the Dusk Rose | Full | generic-ok | 20.4 / 27.7 / 72.3 (problem 7) |
| Falcon and Redwing | Full (`:264`) | deck (evasive) | 22.7 / 85.8 / 14.2 |
| Firebird, Blazing Ranger | **Partial: attack pump missing** | generic-ok | 21.9 / 80.2 / 19.8, win 51.3 (problem 6) |
| Gilgamesh, Master-at-Arms | Full (`:943`) | deck (68) | 21.7 / 35.4 / 64.6 |
| Glorybringer | Full (t2.rs) | generic-ok | 22.9 / 38.3 / 61.7 |
| Grave Titan | Full | generic-ok | 20.4 / 20.7 / 79.3, win 84.2 |
| Hero of Bladehold | Full | generic-ok (about 48) | 20.1 / 40.9 / 59.1 |
| Lightning, Security Sergeant | **Unmodeled: no menace, no exile-play** | generic-WRONG | 21.8 / 79.6 / 20.4, win 55.1 (problem 6) |
| Lyra Dawnbringer | Full | generic-ok | 20.8 / 37.4 / 62.6 |
| Marchesa, the Black Rose | Full (marchesa.rs) | generic-ok | 22.4 / 31.7 / 68.3 |
| Nightveil Predator | Full | deck (named evasive) | 20.7 / 47.3 / 52.7 |
| Nightwind Glider | Partial (pro-black missing) | deck (evasive) | 21.4 / 84.5 / 15.5, win 46.6 |
| Resplendent Angel | Full | deck (evasive) | 20.0 / 76.7 / 23.3 |
| Restoration Angel | Full | deck (pool protector) | 21.4 / 69.4 / 30.6, win 50.0 |
| Serra Angel | Full | deck (evasive) | 20.8 / 44.4 / 55.6 |
| Skyclave Apparition | Full | generic-ok | 21.9 / 71.1 / 28.9 |
| Smaug the Magnificent | Full (`smaug.rs`); `--cards` stale | generic-ok | 23.0 / 55.1 / 44.9, win 67.5 |
| Sphinx of New Prahv | Full (ward 2) | deck (evasive) | 20.9 / 48.9 / 51.1, win 75 |
| Sun Titan | Full | generic-ok | 22.6 / 22.7 / 77.3 (problem 7) |
| Sunblast Angel | Full | generic-ok | 21.4 / 29.0 / 71.0 |
| Terror of the Peaks | Full | generic-ok | 23.0 / 37.7 / 62.3 |
| The Wasp, Winsome Avenger | Full (`blue_cards.rs:542`) | deck (evasive) | 22.2 / 82.0 / 18.0 |
| Thundermaw Hellkite | Full | generic-ok | 21.2 / 31.9 / 68.1 |
| Vampire Nighthawk | Full | deck (evasive) | 21.8 / 66.3 / 33.7 |
| Windborn Muse | Full | deck (evasive) | 22.0 / 73.2 / 26.8 |
| Elspeth, Sun's Champion | Full | generic-ok | 21.3 / 27.6 / 72.4 |
| Arcane Signet / Fellwar Stone / Orzhov Signet / Talisman of Conviction | Full | deck (ramp 82-89) | cast 81-85% at round 5.4-5.6; win-when-cast 43-53% (cheap early cards; correlation) |
| Conqueror's Flail | Full | deck (`kraum_special`, tutor target) | 29.4 / 86.4 / 13.6; over-tutored (problem 4) |
| Crawlspace | Full (combat.rs) | generic-ok (pillowfort 45-70) | 22.4 / 72.8 / 27.2 |
| Ghostly Prison | Full | generic-ok | 23.1 / 67.8 / 32.2 |
| Propaganda | Full | generic-ok | 23.6 / 62.7 / 37.3 |
| Helm of Kaldra | Full | deck (equipment 56) | 24.6 / 60.2 / 39.8 |
| Shield of Kaldra | Full | deck (equipment 56) | 23.1 / 45.2 / 54.8 |
| Lightning Greaves | Full | generic-ok | 24.3 / 82.6 / 17.4 |
| Darksteel Mutation | Full | generic-ok | 21.3 / 90.1 / 9.9, win 46.2 (cast on whatever is there: unknown) |
| Underworld Breach | Full as a card; line blocked | generic-WRONG | 19.8 / 0.0 / 100 (problem 1) |
| Memory Trap | Full | generic-ok | 19.9 / 81.6 / 18.4 |
| Arcane Denial | Full (counter) | generic-ok | 19.1 / 57.0 / 43.0 |
| Bedevil | Full (`rem=destroy`) | generic-ok | 21.8 / 57.1 / 42.9 |
| Dark Endurance | Full on paper | generic-WRONG | 22.8 / 0.0 / 100 (problem 2) |
| Disenchant | Full | generic-ok | 24.0 / 67.6 / 32.4 |
| Fact or Fiction | Full | generic-ok | 20.7 / 37.6 / 62.4, win 72.9 |
| Path to Exile / Swords to Plowshares | Full | generic-ok | cast 90-91% at round 5.8-5.9 |
| Austere Command | Full | generic-ok | 21.3 / 43.8 / 56.2 (held for a lost board) |
| Avenge | Full (cost reduction unverified) | generic-ok | 20.0 / 24.4 / 75.6, win 47.7; held |
| Diabolic Tutor | Full | generic-WRONG (target) | 23.8 / 44.4 / 55.6 (problem 4) |
| Expressive Iteration | Full (mine.rs) | generic-ok | 21.7 / 60.5 / 39.5 |
| Lands (checks, pain, filter, Command Tower, Exotic Orchard, Unclaimed Territory, Spectator Seating, Port Town, Furycalm Snarl) | Full | generic-ok | no evidence of a problem; `Missed land drops, turns 1-5` 0.39 |
