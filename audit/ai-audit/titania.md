# Titania (titania) — AI audit, phase 1

Strategy (decklists/JD/titania-lands.md): mono-green ramp. Titania, Protector of Argoth makes a 5/3 Elemental whenever
a land goes from the battlefield to the graveyard, and her enter trigger returns a land (a fetch land) from the graveyard
so it can go again. Only three lands do it themselves (Evolving Wilds, Terramorphic Expanse, Fabled Passage); the deck
fills the graveyard (Satyr Wayfinder, Malevolent Rumble, Something Worth Saving, Scout the City, Pothole Mole), ramps,
draws, and wins late with big green creatures and a finisher (Overwhelming Stampede, End-Raze Forerunners). Weak to
graveyard hate and fast decks.

Priority function: `titania_prio` (rust/crates/sim-core/src/impls/titania.rs:647), which wraps `pool::generic_prio`
(Titania 97, fillers 86 / 44, Cartographer's Survey 74, Vorinclex 62, Garruk's Uprising 70, finishers only when lethal,
Royal Treatment / Blossoming Defense / Arachnogenesis 0). Plan config: none (no `plans::config` entry; style
`st(1.0, 0.70, 0.50)`). Special plays: `titania_special` (crack held fetch lands, Greenbelt Radical). Fetch-hold logic:
`hold_fetch`, `land_adjust`. Protection: `titania_protect`. Worldly Tutor: card code in impls/t5.rs:406 (tutor "cre",
top of library), `decks::tutor_pick`.

Data: `commander-sim --deck titania --pool t2 --analyze --ai adaptive --games 300 --jobs 2`: 27.0% win vs t2 (Lord
Windgrace, Brago, Kaalia, Sythis, Meren), eliminated in 70% of games (round 12.9), game plan "first 5/3 from Titania"
online in only 61% of games (T6 34%, T10 52%); Titania is removed 1.99 times per game (189 Titania casts in 80
traced games). The `--analyze` card report lists only cards cast in 100+ games (Satyr Wayfinder, Titania), so the
per-card numbers below come from the paired `--swap "X=>Forest"` breakdown ("Swapped OUT" table = the current list),
conservative profile, 300 games: drawn% / cast% (share of drawn games) / stuck% = 100 - cast%. Play-by-play from 80
traced games (`--trace 0..79`).

## Problems (ranked by likely impact on win rate)

1. **Blossoming Defense is cast on turn 1 with no creature, wasting the protection for Titania.** `titania_prio` returns
   0 for it ("only in response"), but `brain::card_utility` (ai/brain.rs ~280) replaces a base of 0 by
   `dsl::card_value * 10` for a card with a DSL and no hand-written `prio` hook, and Blossoming Defense has the DSL
   (pump + grant) and no hook. Evidence: cast in 97.6% of the games it was drawn, avg first cast own turn 5.9 (a
   response-only card would be cast late and rarely); traces: "Titania casts Blossoming Defense" on R1 with only a Forest
   out, 18 casts in 80 games, most at no removal. Titania is removed 1.99 times a game and her
   recast is the engine, so this is the biggest hole. Royal Treatment is not affected (58% cast, round 11.3, no DSL;
   a real response-only pattern). Fix: register `prio = Some(zero_prio)` for Blossoming Defense (what the sheoldred
   deck does for its held cards) so `card_utility` skips the DSL fallback; `titania_protect` keeps casting it.
2. **Fblthp, Knows the Way: cost and fetch are wrong.** The carddb has cost {G}{G} (generic 0); the real card is
   {X}{G}{G}: "when it enters, search for up to X basic land cards with different names" (data/scryfall_cache.json).
   Here the DSL always fetches 1 basic to hand for GG. In this deck every basic is a Forest, so a real Fblthp fetches at
   most 1 Forest and only for X >= 1 ({1}{G}{G}); at X = 0 it is a 1/2 for GG that finds nothing. The sim gives a land
   for 1 less mana than the real card. Evidence: `--cards` ("search basic, n=1"), cards.json `generic 0, pips GG`.
   Cast 98.9% when drawn, drawn 31.0% (above the 24-28% of most singletons). Fix: model X (cast with X = 1 when 3 mana is
   free, 0 otherwise) and count Domain power (Forest only = 1).
3. **Silhana Starfletcher makes colourless mana, not green.** Tag `dork=C`, no override; the Oracle is "as it enters,
   choose a color; {T}: add one mana of the chosen color" (scryfall_cache). In a mono-green deck full of {G}{G} costs
   (Titania {3}{G}{G}, Fblthp, Cactarantula, Vorinclex) a colourless dork is worse than the real card. Cast 89.2% when
   drawn. Fix: tag `dork=G`.
4. **Worldly Tutor early fetches the 4-mana Satyr Wayfinder (or Silhana) on turn 2** and later Ghalta / End-Raze
   Forerunners. 11 casts in 80 games: Satyr Wayfinder 5 (R2 x4, R10), Silhana Starfletcher 1 (R2), Ghalta 3,
   End-Raze Forerunners 3. The early pick comes from `decks::tutor_pick` ranking by `titania_prio` before turn 5
   (`filler` rule gives Satyr Wayfinder 86, above the dorks at 85), then by `tutor_value` (bomb x 10, so Ghalta 80) after
   turn 4. A 4-drop 1/1 put on top at turn 2 (1 land, 2 next turn) takes the draw that would have been a land or
   ramp piece. It does help the plan (fills the graveyard so Titania's trigger has a target), and the win rate when
   Worldly is cast is 37.5% (deck 27%), so the effect is unknown. What it should fetch for her plan: a ramp creature
   early, a land-finder (Fblthp, Spider-Man, Studious First-Year) or a filler at turn 4-5 when Titania is a turn away,
   and a Titania-independent finisher late. Fix: add a Worldly-specific wish list in `titania_prio`/`tutor_pick`
   (needs a decision from JD on the intended targets).
5. **Arachnogenesis is almost always stuck while the deck dies in combat**: 61.1% stuck, cast 38.9% of draws, first cast
   avg round 13.6 against an average elimination round of 12.9; the deck is eliminated in 70% of games, 97% of those by
   combat (Windgrace 20%, Brago 15%, Kaalia 14%). The gate ("a third of the life or more") fires too late. Fix:
   unknown threshold; check the attack-size test against the real killing attacks in `arachno_resolve` / the response
   code.
6. **Plan online in only 61% of games** (first 5/3 from Titania; T10 52%): only the 3 fetch lands (and opponents'
   land destruction) make 5/3s, so games without one drawn never start. Not an AI bug, but the hold/crack logic is
   all there is to optimise: one trace (game with R6 "Titania plays Evolving Wilds ... cracks for Forest") cracked a
   fetch land with Titania not on the battlefield and 6 lands. Unknown whether she was in reach that turn.
7. **Vinelasher Adept 43% stuck** (cast 56.7%, avg round 11.3; its landcycling counts as no cast) and **Tumbleweed Rising
   48% stuck** (cast 51.7%, round 11.6; plot / X by greatest power). Both are late by nature. Not flagged as bugs.
8. **End-Raze Forerunners as a creature is capped at priority 30** unless the attack is lethal (`finisher_prio`);
   Worldly Tutor still fetches it (3 of 11 picks) because `tutor_value` ignores that cap. Cast 75.3%, round 10.8,
   win 41.0% when cast. Unknown effect.

## All cards

drawn / cast / stuck from the paired swap breakdown (conservative). "generic-ok" = generic logic is fine here.

| Card | Model | Valuation | Evidence / note |
|---|---|---|---|
| Titania, Protector of Argoth | Full (t1.rs shared ETB + `land_gy`) | deck (97 - 3 x tax) | cast 99.7%, median turn 5.0; removed 1.99/game; plan online 61% |
| Annoyed Altisaur | Full (cascade) | generic-ok | 35.0 / 82.9 / 17.1; round 10.1; removed 0.10 |
| Arachnogenesis | Full | deck (0, response only) | 24.0 / 38.9 / 61.1 stuck, round 13.6 — problem 5 |
| Bestial Incursion | Full (DSL token) | generic-ok | 30.3 / 87.9 |
| Bite Down | Full (bite) | generic-ok (held removal) | 23.7 / 85.9 |
| Blossoming Defense | Full (DSL) but DSL-cast | generic-WRONG | 27.3 / 97.6 cast, round 5.9, turn-1 casts — problem 1 |
| Boughside Wanderers | Full | generic-ok | 27.3 / 79.3 |
| Bugenhagen, Wise Elder | Full | generic-ok (dork 85 early) | 25.3 / 97.4; removed 0.07 |
| Cactarantula | Full (draw on target; no Desert in the deck) | generic-ok (bomb 6) | 30.3 / 93.4, round 10.2; removed 0.08; 22 casts in 80 traces; win 31.8% when cast |
| Carnivorous Canopy | Full | generic-ok | 24.3 / 91.8 |
| Cartographer's Survey | Full | deck (74 at <= 7 lands) | 28.0 / 91.7 |
| Chelonian Tackle | Full | generic-ok | 27.3 / 79.3 |
| Compel Brutality | Full | generic-ok | 28.3 / 87.1; win 27.0% |
| Crushing Canopy | Full | generic-ok | 30.0 / 78.9 |
| Cultivate | Full | generic-ok | 20.7 / 91.9 |
| Diplomatic Relations | Full | generic-ok | 27.7 / 79.5 |
| End-Raze Forerunners | Approximate (manual endraze tag) | deck (92 only when lethal; else 30) | 27.0 / 75.3, round 10.8 — problem 8 |
| Fblthp, Knows the Way | Partial (X dropped, fetches 1 land for GG) | generic-ok | 31.0 / 98.9 — problem 2 |
| Follow the Lumarets | Full | deck (48) | 25.0 / 96.0 |
| Garruk's Uprising | Approximate (manual uprising tag, no override note) | deck (70 / 40) | 20.3 / 98.4 |
| Genesis Hydra | Full | generic-ok (xtutor, own prio) | 25.3 / 92.1; removed 0.10 |
| Ghalta the Unstoppable | Full | generic-ok | 34.3 / 85.4, round 10.0; removed 0.22 (highest non-commander) |
| Glorious Decay | Full | generic-ok (held removal) | 29.7 / 91.0 |
| Greenbelt Radical | Full | deck (`radical_options`) | 26.7 / 97.5 |
| Greenhouse Propagator | Full | generic-ok | 24.0 / 91.7 |
| Hard-Hitting Question | Full | generic-ok | 26.0 / 94.9 |
| Hardbristle Bandit | Full (crime untap) | generic-ok | 26.3 / 96.2; win 23.7% when cast (lowest) |
| Infestation Expert | Full | generic-ok | 32.7 / 88.8 |
| Knockout Maneuver | Full | generic-ok | 27.0 / 81.5 |
| Kura, the Boundless Sky | Full | generic-ok | 27.3 / 91.5; win 44.0% when cast |
| Lashwhip Predator | Full | generic-ok | 34.0 / 89.2; removed 0.12 |
| Lifestream's Blessing | Full | generic-ok | 24.0 / 81.9 |
| Llanowar Elves | Full | generic-ok (85 early) | 33.3 / 94.0 |
| Malevolent Rumble | Full | deck (filler 86 / 44) | 28.0 / 97.6 |
| Nature's Lore | Full | generic-ok | 26.3 / 94.9 |
| Noxious Newt | Full | generic-ok | 23.0 / 95.7 |
| Overwhelming Stampede | Approximate (manual tag) | deck (92 only when lethal) | 29.0 / 77.0 / 23.0 stuck; win 52.2% when cast (finisher) |
| Paladin of Predation | Full (toxic 6, evasion) | generic-ok | 27.7 / 81.9; removed 0.11 |
| Paradise Druid | Full (hexproof while untapped via `grant_kw`, any colour) | generic-ok (dork 85 early) | 30.3 / 93.4, round 7.2; win 27.1%; 12 casts in 80 traces |
| Pick Your Poison | Approximate (DSL modal; no override) | generic-ok | 24.7 / 95.9 |
| Pothole Mole | Full | generic-ok | 28.3 / 95.3; removed 0.06 |
| Rampant Growth | Full | generic-ok | 25.0 / 97.3 |
| Rending Vines | Full | generic-ok | 22.7 / 97.1 |
| Royal Treatment | Full | deck (0, response) | 28.7 / 58.1, round 11.3 (reactive, fine) |
| Satyr Wayfinder | Full (manual fill) | deck (86 when no fetch in graveyard) | 37.3 / 97.3; win 31.2% when cast (+4.2) |
| Scout the City | Full | deck (filler) | 21.7 / 95.4 |
| Shamanic Revelation | Approximate (manual drawcre tag; no override) | generic-ok (55 at >= 4 creatures) | 29.0 / 79.3; win 50.7% when cast |
| Silhana Starfletcher | Partial (colourless instead of chosen colour) | generic-ok | 31.0 / 89.2 — problem 3 |
| Skyshroud Claim | Full | generic-ok | 25.0 / 84.0 |
| Something Worth Saving | Full | deck (filler) | 23.3 / 97.1 |
| Spider-Man, Brooklyn Visionary | Full | generic-ok | 27.3 / 91.5; removed 0.07 |
| Studious First-Year | Full (prepared Rampant Growth) | generic-ok | 28.3 / 96.5 |
| Symbiotic Beast | Full (DSL dies token) | generic-ok | 29.0 / 83.9 |
| Tenured Concocter | Full | generic-ok | 30.7 / 90.2 |
| Thirsting Roots | Full | generic-ok | 24.3 / 98.6 |
| Thrashing Brontodon | Full | generic-ok | 26.7 / 95.0 |
| Topiary Lecturer | Full | generic-ok | 26.0 / 85.9 |
| Track Down | Full | generic-ok | 30.0 / 95.6 |
| Tumbleweed Rising | Full | generic-ok | 29.0 / 51.7 / 48.3 stuck, round 11.6 |
| Vinelasher Adept | Full | generic-ok | 30.0 / 56.7 / 43.3 stuck, round 11.3 |
| Vorinclex, Monstrous Raider | Full | deck (>= 62) | 29.0 / 90.8; removed 0.09 |
| Worldly Tutor | Full (tutor_to_top "cre") | generic-WRONG-ish (see problem 4) | 25.7 / 83.1; picks in 80 traces: Satyr Wayfinder 5, Ghalta 3, End-Raze 3, Silhana 1 |
| Evolving Wilds / Terramorphic Expanse / Fabled Passage | Full (fire `land_gy`) | deck (`hold_fetch`, `titania_special` crack at 25) | drawn 24-28% each; engine of the plan |
| Forest | Full | n/a | |
