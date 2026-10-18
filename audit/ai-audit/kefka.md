# Kefka (kefka) — AI audit, phase 1
Strategy: Grixis discard-draw. Kefka (enter/attack) makes everyone discard and draws per card type; {8} flips him to Ruler of
Ruin (draw per life an opponent loses on your turn); Psychosis Crawler or Niv-Mizzet then make a two-card win. Second plan:
Underworld Breach + storm (Brain Freeze / Grapeshot, rituals, Yawgmoth's Will). Tutors: Diabolic Tutor, Profane Tutor (suspend 2),
Grim Servant (MV <= devotion to black); the doc says to use them for Crawler / Niv / Breach / Will "whichever the board needs".
Priority function: `kefka_prio` (rust/crates/sim-core/src/impls/kefka.rs:1390) falling back to `pool::generic_prio`; discards by
`kefka::discard_choice`; flip by `flip_utility`; options `kefka_options` (Breach line, Will, Voidborn). Tutors: `tutor_pick_named`
"kefka" arms (ai/decks.rs:743-760). Protection: `veyran_protect` (decks.rs:1347). Plan config: "Kefka flipped into Ruler of Ruin".

Evidence runs (adaptive AI, `--jobs 2`, nice 19): `--analyze` vs t3 (300/profile: win 21.3% cons / 23.0% loose); paired `--swap` of
24 key cards for Island vs t3, 600 games (used only for drawn / cast / stuck; the -9.7 win delta is the cost of removing 24
cards, not a card verdict); 40 traced games (`--trace 1..40`, seeds 500001+) grepped for tutor results and discards.
Flip online at all 47.7%, by T8 19%, by T12 44%. Kefka cast 92% (median T5), removed 0.25/game. Breach line: pieces held on a turn in
26.3% of games, dry run passed and executed in 9.3%, and every execution won (9.3%); failures mostly "storm count / other" (14.3%).
`cardgen audit --deck kefka` could not run (another deck doc mid-edit in the shared worktree); model grades from `--cards` only.

## Problems (ranked by likely impact on win rate)
1. **The tutors only look for the Breach line, never Crawler / Niv-Mizzet / the board's need.** `tutor_pick_named` "kefka"/"any"
   (decks.rs:743) is unconditional for this deck (`deck_has_storm` is always true): Breach if not held -> Brain Freeze/Grapeshot ->
   Dark Ritual/Lotus Petal/Cabal Ritual/Yawgmoth's Will; only after all of those are gone does the generic pick run. Evidence: in
   40 traced games the Kefka tutors found Underworld Breach in 20 of 29 events (13 "tutors", 7 "searches", incl. Grim Servant), Crawler once, Niv once, Dark
   Ritual once, removal/other 3; swap table: Breach drawn 66.3% vs 43-45% for every other single (Crawler 44.3, Niv 43.8, Will 44.3).
   The doc's own order (flipped Kefka and no Crawler/Niv -> fetch it; Kefka dead or short on mana -> Breach line) is not read.
   And Breach is a win only with a storm card AND the graveyard AND mana: cast 23.6% of draws, 76% stuck. Fix: a board-state arm:
   Kefka flipped (or {8} reachable) and no Crawler/Niv in hand -> Crawler (if library safe) else Niv; else Breach line as now.
   Grim Servant (devotion limit: Crawler MV5 / Niv MV6 are out of reach, Breach MV2 reachable) takes the same generic "any" pick
   and falls back to the best legal card_value; see its row.
2. **Breach and Will are thrown away to Kefka's own discard trigger.** `discard_choice` (kefka.rs:157) scores cards by
   `card_worth`; Breach has cast priority 0 unless 2+ escape targets and 5 mana (kefka_prio:1412), so card_worth = 0 and the
   "new type" bonus (-30, enchantment) makes it a preferred discard; the keep bonus (+80) applies only if a storm card is already
   held. Yawgmoth's Will is generic-held at 0 (cast via `will_options`), so likewise 0 worth. Evidence: of 113 discards in 40
   traced games, Underworld Breach 6, Yawgmoth's Will 6 (neither can be cast from the graveyard; Breach is the deck's best win
   rate card, 51.1% when cast, and was cast 5 times in the same games). Fix: give Breach and Will a keep value (>=60) while
   the storm half is in the library/graveyard, or when a tutor is in hand.
3. **Protection is binned instead of held.** Deflecting Swat is discarded 10 times and Slip Out the Back 3 times in 113 traced
   discards (about half the Swats drawn), because untagged Swat is patched to 40 (kefka.rs:164) and then loses 30 for the
   instant type = 10, below a land (25). Swat cast 3-4% of draws, Slip Out 1.9%, and Kefka is removed 0.25/game. Fix: score
   protection above lands while Kefka is in the command zone or on the battlefield and unflipped.
4. **Breach line fails on "storm count / other" in 14% of games** (dry run tried 25.3%, passed 9.3%). Every execution wins (Brain
   Freeze 73.1% / Grapeshot 82.6% win when cast), but only ~1 of 3 attempts passes. Unknown what the dry run is missing (mana
   4.0%, blue 1.0%, graveyard 1.7% account for the rest). Suggested: have a phase-2 agent print the dry-run shortfall and see
   whether rituals/Petal escapes are counted.
5. **Mana Leak is a soft counter the AI uses like a hard one.** `pick_counter`/`ai_respond` (engine/stack.rs) ignore the soft=3
   tag: the only reference is in `resolve_counter` (stack.rs:752), where the opponent pays if it can. Mana Leak cast 19-22% of
   draws, avg round 4.8 (the earliest cast of any key card), win when cast 25.5%. It is fine early (opponents tapped out),
   wasted into untapped mana. Fix: treat soft counters as value*(chance opponent cannot pay 3) in `ai_respond`; Arcane Denial
   (cast 21%) is a hard counter, fine.
6. **Profane Tutor is suspended only when nothing else is better.** Hand option utility 2.2 (legends.rs:1614), cast 25.6% of
   draws (6.9 avg round; 74% stuck) although {1}{B} on turn 2 is its best use. Fix: raise the suspend utility to ~4 when
   the hand has no 1-2 mana play.
7. **Rhystic Study wins 19.0% when cast (lowest of the key cards; deck 22%).** Cast 61% of draws, round 7.0, `rhystic_prio` is
   generic (30+6/opp+early bonus). Not enough evidence of misplay (a card that gets cast late), just noting; the generic
   prio treats it as a tax engine though opponents here are not forced to pay under adaptive; unknown.
8. **Crawler / Niv timing**: kefka_prio gives 84/70 before the flip and 80 after. The doc says "hold the Crawler until after
   the flip unless the library is short" but also "before the flip it is still worth casting early". Crawler cast 61.7% of
   draws, win 41.5%; Niv 42.2% cast, win 33.3%. The code follows the second doc sentence; the first one is stale. No bug,
   fix the doc.

## Leads asked about
- **Mana Leak** (new): modeled as counter_spell with `soft=3` (Full). Valuation generic (`ctr` tag, threshold 7 minus 0.3 per
  counter in the deck, max 2.5). Does not fit the discard plan specially; problem 5.
- **Expressive Iteration** (new): `mine.rs:3019` Full by the text (hand / exiled-playable / bottom ordered by `topdeck::desire`,
  which values land need, wish list and card_worth). Cast 60.6% of draws, avg round 8.0, win 30.7% (above deck mean 22%).
  Not a problem; one caveat: card_worth(Breach/Will) = 0, so it can bottom them (same root as problem 2).
- **Sleight of Hand** (new): `bulk_blue.rs:726`, picks the higher `card_worth` of two, ignoring land need (a land is 25, any
  castable spell more), so it digs for spells even when short on mana; cast 72% of draws, round 6.8, win 25.9%. Fits the plan
  only as a cheap filter; it does not choose by Kefka's discard needs. Again Breach/Will count 0.
- **Tutors**: problem 1 (Diabolic Tutor, Profane Tutor); Grim Servant: loses 3 life, search "any" with MV <= devotion to black;
  devotion in this deck is small (few BB cards), so Crawler (5) / Niv (6) are rarely reachable: it finds Breach / Brain Freeze /
  rituals or the best legal card. Cast 46% of draws.

## All cards
Model grades are from `--cards` tags and "not modeled" lines (audit tool unavailable). "?" = unverified.
| Card | Model | Valuation | Evidence / note |
|---|---|---|---|
| Kefka, Court Mage // Ruler of Ruin | tag `kefka`, pool | deck (commander prio via generic) | cast 92%, median T5, removed 0.25/game; flip online 47.7% |
| Psychosis Crawler | dsl static scaling + draw->lose_life | deck (84/80, held when loop unsafe) | cast 61.7%, win 41.5 (best key card); never tutored (problem 1) |
| Niv-Mizzet, the Firemind | dsl draw->damage, tap draw | deck (84/80) | cast 42.2% (6 mana), win 33.3; never tutored |
| Underworld Breach | `breach` | deck (hold, 56 w/ 2+ targets, else 0) | win 51.1% cast, cast 23.6% of draws; discarded 6x in 40 games (problem 2) |
| Brain Freeze | `storm stormmill` | deck (held for Breach) | cast 19.5%, win 73.1 |
| Grapeshot | `storm stormburn` | deck (held) | cast 8.9-11%, win 82.6 |
| Dark Ritual / Lotus Petal | dsl / rock | deck (held for the line) | Petal cast 81%, win 32.4; Ritual 57.6% |
| Yawgmoth's Will | `yawg` | `will_options` | cast 16% of draws; discarded 6x in 40 games (problem 2) |
| Diabolic Tutor | `tut=any` | generic-WRONG target | problem 1; cast 53% |
| Profane Tutor | `tut=any`, suspend hand-coded | generic-WRONG target + low suspend utility | problems 1, 6 |
| Grim Servant | dsl etb search + lose 3 | generic-ok | devotion-limited |
| Mana Leak | `ctr=any soft=3`, dsl | generic-ok (see problem 5) | |
| Arcane Denial | `ctr denial` | generic-ok | cast 21%, win 24.6 |
| Expressive Iteration | `mine.rs` iteration | generic-ok | above |
| Sleight of Hand | `bulk_blue.rs` | generic-ok | above |
| Night's Whisper / Twinned Vision / Thrill of Possibility / Big Score / Seize the Spoils / Rapturous Moment | draw tags / dsl | deck (Rapturous 58 w/ 2 others; Twinned Vision via gy option) | Whisper cast 65.7%; Rapturous cast 28% |
| Skullclamp | `clamp` | generic (55) | cast 82%, win 26.4 |
| Rhystic Study | `rhystic` | generic (rhystic_prio) | win cast 19.0%, removed 0.06 |
| Jeska's Will | `jeska` | generic (jeska_prio 25-80) | cast 35% |
| Jin-Gitaxias, Progress Tyrant | `jin` | generic-ok | cast 16% (7 mana); win 35.7 |
| Massacre Wurm | bomb 6 | generic-ok | cast 27%; removed 0.04 |
| Lightning Greaves | `prot=boots` | generic (45 with creatures) | removed 0.04/game |
| Deflecting Swat | tags none; "not modeled" lines are stale: hand-coded in `veyran_protect` (mine.rs:3290), free with commander | patched 40 in discard only | binned 10x in 40 games (problem 3); cast 3-4% |
| Slip Out the Back | `prot=phase` | veyran_protect | binned 3x; cast 1.9% |
| Bitter Triumph / Pongify / Reality Shift / Feed the Swarm / Extinguish the Light / Deadly Embrace / Contract Killing / Chaos Warp / Lightning Bolt | rem tags / dsl | generic-ok | Pongify cast 35%, win 22.6 |
| Crux of Fate | wipe | generic-ok | cast 35.5%; kills Kefka (wipe choice vs own commander unchecked) |
| Scarlet Spider, Kaine | mayhem table (kefka.rs) | deck (held in hand while Kefka out) | discard-GY value; not in sample |
| Demonic Embrace | aura hand-coded gy cast | deck | held, discarded for GY cast |
| Eternal Student, Garrison Excavator | dsl / pool | generic-ok (Excavator gy_left token hook) | |
| Rummaging Goblin, Hallway Heckler, Stadium Tidalmage, Alpharael, Blightspeaker | dsl / hand-coded loot | generic-ok | |
| Rank Rat, Cackling Fiend, Intellect Devourer | dsl discard / devourer hook | generic-ok | |
| Sawblade Scamp, Viashino Fangtail, Screeching Soulbreaker, Susurian Voidborn, Shimmercreep, Sneering Shadewriter, Corroding Dragonstorm, Grab the Prize | hand-coded pings / drain | generic-ok (pings valued after flip via board tags) | Scamp/Firesinger removed 0.04 |
| Blazing Firesinger | `prepare` | generic-ok | |
| Skullport Merchant | dsl treasure + draw | generic-ok | |
| Arcane Signet, Sisay's Ring | rock | generic (85 until T5) | |
| Lands (Blood Crypt, Command Tower, Exotic Orchard, Gemstone Mine, Haunted Ridge, Shivan Reef, Spirebluff Canal, Steam Vents, Stormcarved Coast, Sulfur Falls, Sulfurous Springs, Underground River, basics) | tags | generic-ok | discarded to Kefka as the land type card; fine |
