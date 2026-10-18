# Sheoldred (sheoldred) — AI audit, phase 1

Strategy (decklists/JD/sheoldred-mono-black.md): mono-black drain. Sheoldred, the Apocalypse (4/5 deathtouch) drains each
opponent 2 per draw and gains 2 per own draw; Underworld Dreams / Fate Unraveler add 1 per opposing draw. The deck makes
everyone draw (Howling Mine, Seizan, Memory Jar, Otherworld Atlas, Mikokoro, Sanitarium, Painful Lesson, Peer into the
Abyss) and finishes with Exsanguinate / Peer / drain creatures. Weak when she is exiled or removed repeatedly, and when
the symmetric draw cards come down with no punisher out. Necrosquito (new) is a flying body that grows from sacrifices.

Priority function: `sheoldred_prio` (rust/crates/sim-core/src/impls/sheoldred.rs:1340), which falls to `pkg2_prio` (Mine,
Fate Unraveler, Ambition's Cost) and then `pool::generic_prio`. Plan config: none (no `plans::config` entry; style
`st(1.0, 0.60, 0.60)` in ai/mod.rs:62). Special plays: `sheoldred_special` (equip, Beckoner warp). Protection:
`sheoldred_protect` / `sheoldred_hold`. Per-card options hooks for the draw engines (Atlas, Bell, Geier, Mikokoro,
Dark Deal, Painful Lesson, Peer, Memory Jar in legends.rs).

Data: `commander-sim --deck sheoldred --pool t3 --ai adaptive --games 300 --jobs 2` (win 33.7% conservative / 33.3% loose
vs t3: Aurelia, Tergrid, Atraxa, Marwyn, Korvold). The `--analyze` card report only lists cards cast in 100+ of 300
games, so it shows only Sheoldred. The per-card drawn / cast / stuck numbers below come from a paired `--swap "X=>Swamp"`
breakdown (the "Swapped OUT" table, which describes the current list), conservative profile, 300 games, drawn% = games in
which the card was drawn, cast% = share of those games it was cast, stuck% = 100 - cast%. Play-by-play checks come from
80 traced games (`--trace 0..79`). Note: the swap-to-Swamp runs show 7 spells -> 7 Swamps gives +2 to +4 points (noise
+-6) and fewer missed land drops (0.58 -> 0.29 over turns 1-5), a hint that the land count is tight, not a card problem.

## Problems (ranked by likely impact on win rate)

1. **Masterful Flourish and Battle-Rage Blessing are cast proactively, wasting the Sheoldred protection.** `sheoldred_prio`
   returns 0 for them ("held for the protect hook"), but `brain::card_utility` (ai/brain.rs ~280) turns a base of 0 into
   `dsl::card_value * 10` for any card that has a DSL and no hand-written `prio` hook, and neither card has one. So
   they are cast like a pump spell. Evidence: Masterful Flourish cast in 85.7% of the games it was drawn, average first
   cast on own turn 4.9 (Battle-Rage Blessing 68%, round 7.2). Traces: "Sheoldred casts Masterful Flourish" on turn 1
   with no creature on the table, and mid-game on a creature before attacking. Sheoldred is removed
   1.26 times per game, so the protection is exactly what the deck needs and it is spent before she lands. Fix: give
   both a `prio = Some(zero_prio)` registration (as Lethal Throwdown, Cost of Brilliance already do) so the DSL fallback
   is skipped; `sheoldred_protect` keeps casting them.
2. **Ad Nauseam is held 76% of the time** (drawn 24.7%, cast 24.3%, avg round 7.9; wins 55% when cast). `adnaus_cast_now`
   needs room for 4 cards under cap `7 + (lands+1)/3` and the opponents' turn. The deck keeps about 5 cards in hand
   (4.9 at turn 8 after the land drop), so room is about 3 and the gate rarely opens; with Sheoldred out the cards
   are not drawn, so the life cost bites. Fix: unknown whether to loosen; check `adnaus_hand_cap` against the real hand
   average, or accept it as a late-game card.
3. **Dark Deal is near-dead: 89% stuck** (drawn 27.7%, cast 10.8%). `deal_options` needs Sheoldred out, opponents holding
   4+ cards, and own hand <= 2 (sheoldred.rs:1753). Wins 67% when cast, so it works when it fires but fires 1 game in 9
   it is drawn. Fix: loosen the own-hand limit to 3 or swap the card out (phase 2 decision).
4. **Seizan, Perverter of Truth: 59% stuck** (cast 41%, but 76% win when cast). `seizan_prio` (sheoldred.rs:1776)
   only sees Sheoldred (80) or a high-life/behind-on-cards case (50); it ignores Underworld Dreams / Fate Unraveler, which
   alone make Seizan a drain (`draw_toll` already counts them). Fix: use `draw_toll(g, p) > 0` instead of
   `sheoldred_out`.
5. **Necrosquito (new) is cast late and often skipped**: drawn 24.3%, cast 57.5% of those, avg first cast round 9.3,
   42.5% stuck, win 35.7% when cast (about the deck average). It has no entry in `sheoldred_prio`; generic creature
   rule gives 42 + 2*power + 4 flying = 46 (power tag is 0), the lowest creature value in the deck, so it loses to
   Al Bhed Salvagers, Vault Plunderer, Sneering Shadewriter, Perigee Beckoner (trace lines show it at 5-30% of the
   choice with the same-priority creatures). Model is fine (see table), but the deck does not feed it: Carnage Altar
   and Deadly Dispute are plain DSL/tag cards with no knowledge of Necrosquito. Fix: raise its priority when the
   board has sacrifice engines or fodder (Altar, Dispute, Treasures, tokens), and prefer it as an Altar sink; at least 55
   so it is not last among the 4-drops.
6. **Rooftop Assassin is over-modeled and late**: its DSL is "destroy target creature an opponent controls" with no
   "that was dealt damage this turn" condition (data/cards.json, unparsed list empty, so the condition was dropped).
   So every Assassin is unconditional removal on a flash lifelinker. It is also 51% stuck (avg round 9.3), so it is not
   being used as flash removal on time. Fix: add the damage condition (or an override note) and decide its use.
7. **Memory Jar: 66% stuck** (drawn 20.7%, cast 33.9%, avg round 9.8; 52% win when cast). The cracking option is fine
   (8.0 utility with Sheoldred out, legends.rs:1705), but the cast itself goes through `generic_prio` as a hooked
   permanent (55 / 68 stax rule?) and costs 5, so it comes late. Unknown whether a Jar cast before Sheoldred is
   wasted; no evidence either way. Low priority.
8. **Phyrexian Delver / Rooftop Assassin / Painful Lesson / Cost of Brilliance / Invoke Despair / Lethal Throwdown /
   Last Gasp are 42-51% stuck.** These are mostly reactive or gated by design (Delver waits for a good target; the
   draw spells for a punisher; Invoke Despair needs BBBB). Not flagged as bugs; Last Gasp 51% stuck at 27.5% win when
   cast is the weakest of the removal. Unknown.
9. **Land count / mana**: 7 spells swapped for 7 Swamps raised mana on turn 6 by +0.37 and removed 0.29 missed land drops
   (see header). Not an AI problem; mentioned for the mana-base review.

## All cards

drawn / cast / stuck are from the paired swap breakdown (conservative). "generic-ok" = fine under generic logic.

| Card | Model | Valuation | Evidence / note |
|---|---|---|---|
| Sheoldred, the Apocalypse | Full (manual, sheoA) | deck (96) | cast in 99.3% of games, median turn 4.0; removed 1.26/game; 169 casts in 80 traced games |
| Accursed Marauder | Full | generic-ok | 21.0 / 81.0 / 19.0; round 7.5 |
| Ad Nauseam | Full (adnaus tag) | deck-ish (plans::adnaus_prio) | 24.7 / 24.3 / 75.7 stuck — see problem 2 |
| Al Bhed Salvagers | Full (hooks) | generic-ok | 25.3 / 82.9 / 17.1 |
| Ambition's Cost | Full | deck (prio 0 at life <= 6) | 27.0 / 64.2 / 35.8; round 8.5 |
| Arcane Signet | Full | deck (92 before Sheoldred) | 20.0 / 90.0 / 10.0; round 5.2 |
| Battle-Rage Blessing | Approximate (DSL grants on two separate "target creature" slots; not Full per Oracle) | generic-WRONG | prio 0 overridden by DSL fallback — problem 1; 25.3 / 68.4 |
| Blood Artist | Full (bartist) | generic-ok (BOARD 62) | 26.3 / 84.8; round 5.8 |
| Burrog Banemaker | Full | generic-ok | 26.7 / 92.5 |
| Carnage Altar | Full (DSL, sacrifices tokens/cheap creatures) | generic-ok, but Necrosquito not considered | 28.0 / 67.9; round 7.5; win 29.8% when cast |
| Cast Away Doubt | Approximate (DSL draw 2 + 2 damage each player; no override note) | generic-ok | 23.7 / 73.2 |
| Charcoal Diamond | Full | generic-ok | 25.3 / 85.5; round 5.8 |
| Commander's Sphere | Full | generic-ok | 22.0 / 74.2 |
| Cost of Brilliance | Full (hook) | deck (zero_prio + options) | 25.3 / 55.3 / 44.7 stuck (held for Sheoldred) |
| Crawlspace | Full (attack cap) | generic-ok (pillowfort rule) | 23.3 / 74.3; removed 0.04-0.05/game (top non-commander) |
| Dark Deal | Full (hook) | deck, too strict | 27.7 / 10.8 / 89.2 stuck — problem 3 |
| Darksteel Plate | Full | deck (78 with Sheoldred out + equip option) | 24.3 / 74.0; 6 equips onto Sheoldred in 80 games |
| Deadly Dispute | Full | generic-ok | 22.0 / 60.6; no Necrosquito link |
| Disembowel | Full | generic-ok | 27.7 / 100.0 |
| Exsanguinate | Full | deck (X-drain option) | 22.7 / 55.9 / 44.1; held for a kill or big X (by design) |
| Fate Unraveler | Full (hook; the `--cards` "not modeled" note is stale) | deck (88) | 22.7 / 83.8; round 7.8 |
| Fellwar Stone | Full | deck (92 before Sheoldred) | 23.7 / 94.4 |
| Fetid Imp | Full | generic-ok | 22.7 / 88.2 |
| Foolish Fate | Full (hook) | generic-ok | 21.7 / 64.6 |
| Gix, Yawgmoth Praetor | Approximate (draw option on combat damage; partials.rs/t3.rs) | generic-ok | 22.7 / 80.9; 14 casts in 80 traces |
| Howling Mine | Full (engine/turn.rs hook) | deck (70 only when a punisher is out) | 24.7 / 66.2; round 9.2 |
| Invoke Despair | Full | generic-ok | 22.7 / 57.4; needs BBBB |
| Last Gasp | Full | generic-ok (held removal) | 26.3 / 57.0 / 43.0 stuck; win 33.3% when cast |
| Leech Collector // Bloodletting | Full (hook) | generic-ok | 22.3 / 88.1 |
| Lethal Throwdown | Full (hook) | deck (zero_prio + options) | 27.7 / 57.8 |
| Masterful Flourish | Full per override but DSL-cast | generic-WRONG | cast 85.7%, round 4.9, turn-1 casts with no creature — problem 1 |
| Melancholic Poet | Full (hook) | generic-ok | 25.0 / 85.3 |
| Memory Jar | Full | generic-ok cast, deck option to crack | 20.7 / 33.9 / 66.1 stuck — problem 7 |
| Mind Stone | Full | generic-ok | 23.3 / 82.9 |
| Necrosquito | Full (oil counters, creature/artifact/token triggers) | generic-WRONG (46, lowest) | 24.3 / 57.5 / 42.5; round 9.3 — problem 5 |
| Otherworld Atlas | Full (options hook) | deck | 24.0 / 70.8; Atlas used in 6 of 80 traces |
| Overkill | Full | generic-ok (held removal) | 26.3 / 74.7 |
| Painful Lesson | Full (hook) | deck (zero_prio + options) | 22.3 / 55.2 |
| Peer into the Abyss | Full (hook) | deck (option 12 on kill, 3.5 else) | 21.3 / 50.0 / 50.0 stuck; win 46.9% when cast |
| Perigee Beckoner | Full | deck (52 fixed + warp options) | 24.0 / 79.2; round 8.4 |
| Phyrexian Arena | Full (eng tag) | generic-ok (64) | 21.7 / 80.0 |
| Phyrexian Delver | Full | deck (74 / 36) | 24.3 / 57.5; win 26.2% when cast (lowest) |
| Phyrexian Rager | Full | generic-ok | 25.3 / 76.3 |
| Potioner's Trove | Full | generic-ok | 26.0 / 76.9 |
| Read the Bones | Full | generic-ok | 24.0 / 84.7 |
| Rooftop Assassin | Partial (damage condition dropped; kills unconditionally) | generic-ok, late | 24.3 / 49.3 / 50.7 stuck — problem 6 |
| Screeching Soulbreaker | Full | generic-ok | 22.7 / 69.1 |
| Seizan, Perverter of Truth | Full (hook) | deck, ignores Dreams/Unraveler | 23.3 / 41.4 / 58.6 stuck; win 75.9% when cast — problem 4 |
| Sephiroth's Intervention | Full | generic-ok | 19.7 / 76.3 |
| Seymour Flux | Full | generic-ok | 25.3 / 67.1 |
| Silence the Echo | Full | generic-ok | 24.3 / 89.0 |
| Skyscanner | Full | generic-ok | 24.7 / 77.0 |
| Sneering Shadewriter | Full | generic-ok | 24.3 / 61.6; round 9.0 (5 mana) |
| Soul Enervation | Full | deck (66 / 34) | 25.3 / 77.6 |
| Stir Up Trouble | Full | generic-ok | 24.7 / 93.2 |
| Swiftfoot Boots | Full | deck (equip option) | 22.3 / 82.1; equipped to other creatures 5 of 10 times (before Sheoldred lands); moved to her later by `equip_options` |
| Syphon Mind | Full | generic-ok | 31.7 / 63.2 |
| Terror | Full | generic-ok | 25.7 / 87.0 |
| Ulna Alley Shopkeep | Full | generic-ok | 24.0 / 69.4 |
| Undercity Dire Rat | Full | generic-ok | 21.7 / 87.7 |
| Underworld Dreams | Full | deck (90) | 27.0 / 88.9 |
| Vanquish the Weak | Full | generic-ok | 24.7 / 70.3 |
| Vault Plunderer | Full (hook) | generic-ok | 19.7 / 78.0 |
| Wander Off | Full | generic-ok | 24.0 / 72.2; win 26.9% when cast |
| Bonders' Enclave | Full | land option (end of turn) | played 14 times in 80 traces; use unknown |
| Geier Reach Sanitarium | Full | deck (land option) | played 12 times in 80 traces |
| Mikokoro, Center of the Sea | Full | deck (land option) | played 20 times in 80 traces; use unknown |
| Command Tower / Swamp | Full | n/a | |
