# Findings: g4 (t1.rs, t3.rs, kefka.rs, t2.rs, t0_stella.rs, bulk_white.rs, t4.rs, t5.rs)

32 findings: chosen 18, unreachable 10, skipped / forced "may" 10, opponent-choice 2, check 2 (some rows two kinds);
high 8, med 17, low 7. Paths under rust/crates/sim-core/src/impls/.

| Card | Decks | File:line | Kind | What happens for the person | What the card says | Sev |
|---|---|---|---|---|---|---|
| Stargaze | sephfs | kefka.rs:646, :671 | unreachable + chosen | x=0, takes 0 cards: does nothing; X set only by the AI plan `stargaze_go` | Look at 2X, take X, lose X life | high |
| Splatter Technique | veyran | kefka.rs:791, :819, :861 | unreachable | ctx.mode set only by the AI: a person always gets the 4-damage wipe (own board too), never draw four | Choose one: draw four, or 4 damage to each creature and planeswalker | high |
| Grab the Prize | kefka | kefka.rs:563, :573 | skipped + chosen | Discard not asked (or engine-picked); x stays 0 so the 2 damage to each opponent never happens | Additional cost: discard; draw two; if nonland discarded, 2 damage to each opponent | high |
| Accursed Marauder | sephfs, sheoldred | t3.rs:1561, :1576 | chosen + opponent-choice | AI sacrifices the lowest-pval nontoken creature for every player, the person included | Each player sacrifices a nontoken creature (their choice) | high |
| Reassembling Skeleton | sephfs | t3.rs:388 (`gy_options`) | unreachable | {1}{B} return from the graveyard AI-only | {1}{B}: return from graveyard tapped | high |
| Victimize | seph | t2.rs:1236 | chosen | AI picks the sacrifice and the two returned creatures | Choose two target creature cards; sacrifice a creature | high |
| Glorybringer | kraum | t2.rs:644 | chosen | AI decides exert and target (pval >= 2) | Exert: 4 damage to target non-Dragon | high |
| Exalted Angel | cloud | bulk_white.rs:385, :359 | unreachable | Face-down cast and turning up AI-only (hard-cast works) | Morph {2}{W}{W} | high |
| Twinned Vision | kefka | kefka.rs:610, :615 | unreachable | Flashback (with discard) AI-only; the draw-two half unreachable | Flashback; draw two when not cast from hand | med |
| Demonic Embrace | kefka | kefka.rs:878 | unreachable | Cast from graveyard (3 life + discard) AI-only | May cast from graveyard | med |
| Scarlet Spider, Kaine | kefka | kefka.rs:940, :917 | unreachable + chosen | Mayhem AI-only; ETB discard AI-picked, skipped unless "spare" | Mayhem; ETB may discard for a +1/+1 counter | med |
| Susurian Voidborn | kefka | kefka.rs:1215, :1232, :1268, :1162 | unreachable + chosen | Warp (hand and exile) AI-only; drain target = lowest-life opponent | Warp {B}; target opponent loses 1 | med |
| Decorum Dissertation | kefka | kefka.rs:363, :371 | chosen | AI picks the target player; Paradigm recast not verified | Target player draws two, loses 2 | med |
| Alpharael, Dreaming Acolyte | kefka | kefka.rs:1040 | chosen | AI picks artifact vs two cards, and which | Draw two, discard two unless you discard an artifact | med |
| Stadium Tidalmage | kefka | kefka.rs:1076, :1087, :1095 | skipped (may forced) + chosen | Loot forced; discard AI's pick | May draw, then discard | med |
| Intellect Devourer | kefka | kefka.rs:1330 | opponent-choice | Each opponent's exiled card is min_by worth | Each opponent exiles a card from hand | med |
| Corroding Dragonstorm | kefka | kefka.rs:1284, :1313 | chosen | kefka::surveil is AI logic | Surveil 2 | med |
| Titania, Protector of Argoth | titania | t1.rs:1616 | chosen | AI picks the land (first_max by colour count) | Return target land card from graveyard | med |
| Flickering Hound | seph | t2.rs:309 | chosen + skipped | Blinks the best creature, or none if not "worth" it | May blink another creature | med |
| Conjurer's Closet / Teleportation Circle | seph | t2.rs:297, :302 (`end_step_flicker`) | chosen + skipped | Target from the flicker_worth list | May blink at end step | med |
| Ragavan, Nimble Pilferer | godo | t4.rs:1516 | chosen + skipped | Permanents auto-cast; instants, sorceries, lands dropped; no decline | May cast the exiled card this turn | med |
| Cursed Mirror | godo | t0_stella.rs:952 | chosen + skipped | AI copies the best creature, only if power >= 3 | May become a copy of any creature | med |
| Armored Skyhunter | cloud | bulk_white.rs:1532 | chosen | AI picks card and holder; bottom order random | Look at six; may put an Aura/Equipment onto the battlefield | med |
| Silverblade Paladin | cloud | bulk_white.rs:1442 | chosen + skipped | Pairs only with a partner scoring >= 3, AI-chosen | May pair with a creature | med |
| Forum Familiar | cloud | bulk_white.rs:385 | unreachable | Disguise AI-only | Disguise | med |
| Gix, Yawgmoth Praetor | sheoldred | t3.rs:1710 | skipped + forced | Fires only at life > 10 and library > 10; life payment forced | May pay 1 life to exile and play the top card | low |
| Helm of the Host | godo | t3.rs:919 | forced may | Token copy forced every combat | May create a token copy | low |
| Flux Channeler / Inexorable Tide | veyran / other | t3.rs:981, :992 (common.rs:2802 proliferate) | chosen | Proliferate hits all own permanents | Proliferate: choose which | low |
| Ardenn, Intrepid Archaeologist | cloud | bulk_white.rs:1356 | chosen (Auras) | The fixed path never offers Auras | Any number of Auras and Equipment | low |
| Gemstone Mine | kefka | kefka.rs:1375 | check | Colour choice left to the generic mana UI | Any colour | low |
| Sawblade Scamp | kefka | kefka.rs:1127 | check | options hook reachable: probably fine | Remove oil counter: 1 damage to each opponent | low |

**Cross-cutting (agent's claim, to verify):** `hand_options` and `gy_options` are never offered to a person (the
person's UI has only `human/cards.rs::hand_ways` and `legal::gy_mode`), so every alternative cost there is AI-only.
`engine/cast.rs:62 additional_cost` picks the sacrificed land/creature or the discard for a person on DSL cards.

**Fine:** Mentor of the Meek, Welcoming Vampire, Sram, Tireless Tracker, Recruiter of the Guard (tutor_named),
Consecrated Sphinx, Sanguine Bond, Ruby Medallion, Worldly Tutor (tutor_to_top), Zur on the Y'shtola path, Urza, Notion
Thief, Restoration Angel, Thundermaw Hellkite, Resplendent Angel's end step, Screeching Soulbreaker, Extinguish the
Light, Lay Down Arms, Fate of the Sun-Cryst, Loyal Warhound, Balan, Necropotence, Triskelion, Niv-Mizzet.

**Not checked:** Kefka himself and his flip (mine.rs); kefka.rs Bribery, Brain Freeze, Grapeshot, Underworld Breach,
Fiery Emancipation, Jeska's Will, Rapturous Moment, Deflecting Swat, Psychosis Crawler, Garrison Excavator; Phyrexian
Altar's ability path for a person; whether Grab the Prize has a parsed additional cost.
