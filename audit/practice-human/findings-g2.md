# Findings: g2 (partials.rs, sheoldred.rs, alela.rs, blue_cards.rs, fixes.rs, sephfs_cards.rs, bulk_blue.rs, titania.rs, zur.rs)

59 rows: 14 high, 33 med, 7 low, 5 check. Unreachable 11, skipped 6, chosen 38, opponent-choice 2, check 5.
Paths under rust/crates/sim-core/src/impls/.

## Systemic causes (agent's analysis)
1. `trigger_window` / `ability_window` never ask: every "you may" trigger always happens for a person (~20 cards:
   Seymour Flux, Karmic Justice, Plan for All Outcomes, Rooftop Assassin, Soul Enervation, Delver, Moseo, Airlift
   Chaplain, Jackdaw Savior, The Wasp, Mana Vault ...).
2. `hand_options`, `land_options`, `gy_options`, `extra_mana` are never offered to a person (only `cards::hand_ways`,
   `cards::land_abilities`): Muddle the Mixture transmute, Reconstructed Thopter unearth, Jaxis blitz, Perigee
   Beckoner warp, Lethal Throwdown's sacrifice cost.
3. A spell with a `resolve` hook and no Rem tag gets an empty Ctx from the person's cast: it does nothing (Lethal
   Throwdown, Presumed Dead, Dark Endurance) or applies an AI-chosen target (Cost of Brilliance, Painful Lesson, Peer
   into the Abyss always target the caster; Daydream, Helping Hand, Rewrite Regrets pick the card/creature).
4. An `options` hook not gated on the person still shows only the AI's single best option, and its `*_go` picks
   targets itself (Gix, Mirage Mirror, Credit Voucher, Grave Researcher, Venser, Serra, Emry).
5. `discard_worst` and `zones::edict` are person-aware: "each opponent discards / sacrifices" is fine when the person
   is the affected player.

## High
| Card | Decks | File:line | Kind | What happens for the person |
|---|---|---|---|---|
| Lethal Throwdown | sheoldred | sheoldred.rs:1404/1372 | skipped | No target, no sacrifice cost: destroys and draws nothing (sac + target only in the AI plan) |
| Presumed Dead | sephfs | sephfs_cards.rs:585/596-657 | skipped | No target: nothing happens (real use AI-only, presumed_protect/option) |
| Dark Endurance | kraum | sheoldred.rs:1425/1530 | skipped (verify) | Only tag prot:ind, no DSL/resolve; effect only in sheoldred_protect (skipped for a person) |
| Pestilence | sephfs | sephfs_cards.rs:394 | unreachable | "{B}: 1 damage to each creature and player" not available |
| Blood on the Snow | sephfs | sephfs_cards.rs:189 | skipped / chosen | Forced creature mode; X = 0 for the return half |
| Extinction Event | sephfs | sephfs_cards.rs:143/117 | chosen | Odd/even picked by AI |
| Braids's Frightful Return | sephfs | sephfs_cards.rs:670/681/745 | chosen / opponent-choice | Read-ahead chapter, sac fodder, returned card AI-picked; ch. III target opponent (a person) never chooses |
| Otherworld Atlas | sheoldred | sheoldred.rs:1993 | unreachable | |
| Geier Reach Sanitarium, Mikokoro | sheoldred | sheoldred.rs:2028/2034 | unreachable | land_options |
| Phyrexian Tower | sephfs | sheoldred.rs:2445 | unreachable | No sacrifice for {B}{B} |
| Leech Collector // Bloodletting | sheoldred | sheoldred.rs:1300 | unreachable | Not among the four PREPARED cards in the person's flow |
| Venser, the Sojourner | alela | alela.rs:1252-1345 | chosen / unreachable | -1 unblockable not a walker ability (venser_minus skips a person); +2 hidden when AI prefers -1; blink target AI-picked |
| Scout the City, Glorious Decay | titania | titania.rs:286/309 | chosen | Mode (and Decay's target) AI-picked |

## Med (chosen unless noted)
Necromantic Selection (sephfs :175, best creature returned); Rewrite Regrets (sephfs :568, no target prompt); Grave
Researcher // Reanimate (sephfs :472, offered only with life-cost >= 8, AI target); Moseo (sheoldred.rs:1341, AI
return, always); Phyrexian Delver (:972, AI card, forced life loss); Soul Enervation (:906); Rooftop Assassin (:519);
Perigee Beckoner (:991, :1603, ETB target AI; warp / cast from exile unreachable); Vault Plunderer (:363, target player
AI: can't target an opponent); Al Bhed Salvagers (:425, weakest opponent); Seph FS drain (:2325/:2334, weakest
opponent); End of the Hunt (sephfs, :779/:804, opponent-choice); Cost of Brilliance (sheoldred, sauron, :136, target
always the caster, counter on best_creature); Painful Lesson (:1765, caster); Peer into the Abyss (:2093, caster); Plan
for All Outcomes (alela :291); Muddle the Mixture (alela :398, transmute unreachable); Karmic Justice (alela :479);
Airlift Chaplain (:734); Jackdaw Savior (:775); Helping Hand (:1135); Daydream (:1155); Reconstructed Thopter (:1023,
unearth unreachable); Credit Voucher (:1366); Mirage Mirror (:1551, best target power 3+ only); Something Worth Saving,
Malevolent Rumble (titania :215/:245/:253); Follow the Lumarets (titania :300, t1.rs:736 look_take); Titania (titania.rs:166,
land AI-picked, fetches auto-cracked); Gix (sheoldred, partials.rs:1801, discards all AI junk, AI target); Jaxis (godo,
partials.rs:571/:652, blitz unreachable); Mana Vault (godo, fixes.rs:706, pays/untaps by AI judgment); Sleight of Hand
(kefka, bulk_blue.rs:727); The Wasp (kraum, blue_cards.rs:542/:559); Voltaic Key (godo, partials.rs:3268, extra_mana only,
verify); Simian Spirit Guide (godo, partials.rs:428, verify).

## Low
Seymour Flux (:389, optional draw always); Pileated Provisioner, Plumecreed Mentor (alela :212); Desperate Futurescribe
(:861); Emry (:966); Serra the Benevolent (:1860); Cabal Patriarch (:1771); Crystal Chimes (:1925); The One Ring (godo,
sephfs, partials.rs:2891, tap hidden at low life with 2+ burden).

## Check
Angelic Shield (alela :1957, DSL sacrifice ability?); Arachnogenesis (titania :377, X computed if unset); Basalt / Grim
Monolith untap (godo, fixes.rs:470); Skyclave Apparition ETB target (outside these files); Necromantic Selection / Blood
on the Snow (Tag::Wipe + resolve: engine/cast.rs:1103 may apply the wipe twice for a person).

**Fine:** Melancholic Poet, Foolish Fate, Vona's Hunger, Dark Deal, Ambition's Cost, Syphon Mind, Callous Inspector,
Necrosquito, Ulna Alley Shopkeep, Howling Mine, Seizan, Underworld Dreams, Fate Unraveler, Swiftfoot Boots, Champion's
Helm, Darksteel Plate, Masterful Flourish, Battle-Rage Blessing, Living Death, Grave Betrayal, Dread Slaver, Kaya's
Ghostform, Feather of Flight, Reanimate, Massacre Girl, Wheel of Fortune, Chaos Warp, Paradise Druid, Aven Mindcensor,
Stirring Hopesinger, Malcator, Angelic Page, Wall of Junk, Lightning Greaves, Scrapheap, Prayer of Binding, The Eternal
Wanderer, Gamble, Annoyed Altisaur.

**Not checked:** non-tsv cards; partials.rs beyond 12 tsv cards (Urza not opened); zur.rs lock cast paths; blue_cards.rs
beyond The Wasp; fixes.rs Ancient Tomb, Charcoal Diamond; DSL-implemented cards.
