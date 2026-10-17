# Godo, Bandit Warlord — Behaviour Spec (Tier 6)

Sources (all opened 2026-10-17):

- **[primer: RL]** gtoast99, Insertcleverphrasehere and pwpryan (Red Love community), "Waiting for Godo - cEDH Primer", Moxfield, printed by JD 2026-10-17 (20 pages). https://moxfield.com/decks/URpEcoe6e0GBp3GzsstyBg/primer. Extract: `primers/godo-waiting-for-godo.md`. **The first real Godo primer. It outranks every source below, including [LC] and [notes]; where they differ, this spec follows [primer: RL].** Their list differs from JD's: they run Loki's Scepter, Panharmonicon, Diviner's Wand, Meteor Sword, Strength-Testing Hammer, Rings of Brighthearth, Claim the Firstborn, Mishra's Workshop, Prized Statue, Anje's Ravager, Moonsilver Key, Void Mirror, Devastating Onslaught and more, none of which is in our list (checked against `decklists/JD/godo-cedh-equipment.md` on `godo-cedh`). They do not run Pyroblast or Red Elemental Blast; ours does.
- **[LC]** Learn cEDH, "Godo" decklist page, Dec 11, by Evan Pierce. https://learncedh.com/decklists/godo
- **[DDB]** cEDH Decklist Database, entry "Godo Helm" (2022-06-29), described as a mono-red ramp list with an infinite combo using only its commander. It links a Moxfield list ("Waiting for Godo") and a Discord, no primer. https://cedh-decklist-database.com
- **[3F1]** Three for One Trading, "A Casual Guide to CEDH", Ben Guilfoyle, Oct 16, 2025. https://www.threeforonetrading.com/en/a-casual-guide-to-cedh
- **[CH]** Commander's Herald, "Five Budget Commanders and Their Two-Card Combos", Benjamin Levin, May 9, 2024. https://commandersherald.com/?p=32239
- **[ER-combo]** EDHREC combo page, Godo + Helm of the Host. https://edhrec.com/combos/r/1692-4251
- **[ER-avg]** EDHREC average cEDH Godo list (242 decks with user-set brackets): a list only, no text. https://edhrec.com/average-decks/godo-bandit-warlord/cedh/equipment
- **[notes]** Our own notes, `decklists/JD/godo-cedh-equipment.md` on branch `godo-cedh`. This is JD's deck write-up, not an outside source. A claim tagged [notes] is in that file and no web page confirms it.
- Oracle text from Scryfall API (2026-10-17) for every card named in a win line or decision.

Earlier drafts of this spec had no primer (the database links none; Moxfield returns 403 to fetches). JD supplied [primer: RL] later the same day; claims it confirms are retagged [primer: RL], conflicts follow [primer: RL], and gaps (mulligan, removal order, sequencing) are filled from it. Where only [notes] or [LC] says something, it is tagged so. Spec written 2026-10-17.

Tags: **[primer: RL]** the Red Love primer says it (page numbers in `primers/godo-waiting-for-godo.md`). **[primer]** another outside source says it. **[notes]** our deck notes say it. **[inference]** my reasoning from the Oracle text.

## Game plan

Godo is a one-card combo from the command zone: cast him, his enter trigger puts Helm of the Host onto the battlefield, Helm makes a hasty token Godo each combat, and each token's first attack adds a combat, so the table dies in one turn [primer: LC, 3F1, ER-combo]. The deck exists to reach the six mana (plus attach) as early as possible with 18 rocks, rituals, mana creatures and fast lands [notes]. Turn 1 or 2 wins are "realistic" with fast mana [primer: LC]; a turn-2 or turn-3 cast is the design target [notes]. Stance: race. It holds up cheap protection (Deflecting Swat, Pyroblast, Red Elemental Blast) for the cast turn only, plays little else on the board, and waits ("be patient, let opponents exhaust resources") only when the table shows interaction it cannot beat [primer: LC]. The plan B is Dualcaster Mage with a copy spell, which does not need Godo [primer: LC]. The plan C is a tempo/stax game with Blood Moon, Pyrokinesis and Blasphemous Act [primer: LC]. It has little card draw, which makes mulligans critical [primer: LC, RL]. [primer: RL] The pilot "can equally push for an early win, or show restraint and look for an opportunity to exploit": because Godo needs no combo pieces in hand, it can sit with a hair trigger all game and go when a window opens. Fast hands win by turn 2, or turn 3 from an early seat; slower hands first set up permanent mana and a protection piece. In a midrange, card-draw meta it lets opponents spend resources on each other rather than acting as table police [primer: RL]. [inference] The sim should therefore prefer one decisive attempt with a backup over a probing early cast.

## Win lines

Costs checked against Scryfall: Godo {5}{R}; Helm {4}, equip {5}; Hammer of Nazahn {4}; Magnetic Theft {R}; Brass Squire {3}; Peter Parker's Camera {1} (ability {2},{T}); Dualcaster Mage {1}{R}{R} flash; Twinflame {1}{R}; Molten Duplication {1}{R}; Heat Shimmer {2}{R}; Electroduplicate {2}{R}; Cursed Mirror {2}{R}; Jaxis {3}{R} (blitz {1}{R}); Ruby Medallion {2}.

| Line | Pieces (in our list) | Mana (total, colours) | Steps | Where it can be answered |
|---|---|---|---|---|
| **1. Godo + Helm, equip paid** [primer: LC, 3F1, ER-combo] | Godo | 11 (6 + 5), one R; Ruby Medallion makes Godo 5 | Cast Godo; ETB fetches Helm; pay equip {5}; beginning of combat Helm makes a hasty token Godo; token's ETB fetches another Equipment; token attacks, extra combat, repeat | (a) Godo spell: counterspell, Mental Misstep does not hit (MV 6). (b) ETB trigger on stack: Stifle effects, removal on Godo (Helm then has nothing to copy). (c) Equip ability: Pithing Needle naming Helm. (d) Helm: bounce or artifact removal (Cyclonic Rift, Kozilek's Command, Touch the Spirit Realm). (e) Combat: Blind Obedience (tokens enter tapped), Fog, Orim's Chant kicked, Static Prison. [inference] for all of (a)–(e) unless stated |
| **2. Godo + Helm, equip skipped** [primer: RL, CH, LC] | Godo + one of: Hammer of Nazahn (out first, attaches Helm as it enters), Brass Squire (untapped, not summoning-sick), Magnetic Theft, Cursed Mirror/Jaxis route | Hammer: 4 + 6 = 10, R. [primer: RL] Hammer first is the main mana-reducer; Hammer also gets round Null Rod, since nothing is equipped by activation. Squire: 3 + 6 = 9. Theft: 6 + 1 = 7, RR. All less with Medallion | As line 1, but the attach is free or cheap. Hammer's trigger attaches each Equipment that enters, so Godo fetching Helm attaches it at once [primer: LC says Hammer or Theft bypass the equip cost; the exact trigger is Oracle] | Same as line 1, plus the attach piece: Theft is MV 1 (Mental Misstep, Chalice X=1), Squire needs a turn to lose sickness (an inference: Squire has {T}), Hammer is a 4-mana artifact on the stack |
| **3. Godo + Camera, or Hammer first then copy Godo** [primer: RL; Camera by [notes]] | Godo, Hammer in library, and one copier: Peter Parker's Camera (copies the ETB), or Twinflame, Molten Duplication, Heat Shimmer, Electroduplicate, Cursed Mirror (copy Godo himself) | Camera: 8 with Camera out (9 if cast that turn). Copy spell: Godo 6 + Twinflame or Molten Duplication 2 = 8, Heat Shimmer or Electroduplicate 3 = 9 (RR in all cases) [inference from costs] | [primer: RL] Version A: cast Godo; his ETB fetches Hammer first; cast the copier on Godo; the copy's ETB fetches Helm; Hammer attaches it free (the copy dies to the legend rule, Godo stays). Version B ([notes], RL names the Camera): cast Godo; ETB on stack; {2},{T}, remove counter: copy the trigger. Copy resolves first and fetches Hammer; original fetches Helm; Hammer attaches Helm | Same windows as line 1, plus the copier (a second trigger means a single Stifle fails). Version A also needs Godo to survive until the copy resolves |
| **4. Welder + Engineer, then Godo** [notes] | Goblin Welder (out since last turn), Goblin Engineer, Godo | Engineer 2, Welder ability 0 (it taps), then Godo 6: 8, RR+ | Engineer ETB puts Hammer (or Helm) in graveyard; Welder swaps a used rock for it; then Godo fetches the other | Kill Welder/Engineer in response; artifact graveyard hate; Engineer's own search can be stopped by Wan Shi Tong style punishers (inference, minor) |
| **5. Rebuild after a stopped trigger** [primer: RL names Cursed Mirror and Jaxis as doublers; Sculpting Steel [notes]] | Cursed Mirror (Goblin Engineer's usual target [primer: RL]), Sculpting Steel (copying Cursed Mirror then Godo), Jaxis | Mirror 3, or Jaxis blitz 2 + {R} ability | A copy of Godo gets its own enter trigger: fetch Helm, attach, attack | Needs Godo on the battlefield; removal on the original Godo ends it |
| **6. Dualcaster Mage + copy spell** [primer: LC] | Dualcaster Mage plus Twinflame, Heat Shimmer, Molten Duplication or Electroduplicate | 5 (Twinflame or Molten Duplication + Dualcaster: RRR) or 6 (Heat Shimmer or Electroduplicate + Dualcaster); needs a creature on the battlefield to target | Cast the copy spell on a creature you control, hold priority; flash Dualcaster; its ETB copies the spell, retargeted at Dualcaster; the token Dualcaster ETB copies the spell again; stop at enough hasty bodies, attack | The original spell (counter it and the loop never starts); the copy spell is a sorcery or instant spell, so Flusterstorm and Dispel-type cards can counter copies; Teferi, Time Raveler removes the flash; Mental Misstep hits nothing here (MV 2–3). [primer: LC names Twinflame, Heat Shimmer, "Molten Echoes"; Molten Echoes is not in our list, Molten Duplication is] |
| **7. Extra-turn finish** [primer: RL; notes] | Final Fortune, Last Chance | RR | Take an extra turn so summoning-sick or tapped pieces can act; you lose at that extra turn's end step | Only a support line: cast it only when the extra turn wins (Oracle confirmed). [primer: RL] Time Walks are also rituals: develop, untap with full mana for the combo turn; with 5RRR resolve Godo, then the Time Walk, then untap and pay the equip. In main phase 2 after an infinite-mana win, Final Fortune gives another combat |
| **8. Basalt Monolith + a copied Kinnan** [primer: RL, "a surprisingly real wincon"] | Basalt Monolith, an opponent's Kinnan, Bonder Prodigy (Kinnan is a Tier 6 deck), and a copier: Heat Shimmer or Cursed Mirror | Heat Shimmer 3 (or Mirror 3) plus Basalt 3 already out; then colourless is unlimited. Godo still needs {R}: Ogre, a Mountain or a Mox | Copy Kinnan; tap Basalt for {C}{C}{C}, Kinnan adds {C}{C}{C} more; pay {3} to untap; repeat. Then cast Godo with equip paid, or draw with The One Ring [inference: Wand is not in our list] | Kill the Kinnan copy in response (it is a hasty token, exiled at end step); counter Heat Shimmer. Oracle: Kinnan adds one mana of the type a tapped nonland permanent produced [Scryfall]. |

Lines in [primer: RL] that need cards **not in our list** (do not build AI plans on them): Loki's Scepter, Goblin Welder + Scepter + Sculpting Steel/Panharmonicon, Engineer + Panharmonicon + Scepter + Prized Statue, Rings of Brighthearth (Basalt + Rings; Metalworker + Voltaic Key + Rings; Engineer + Scepter + Rings + LED/KCI), Diviner's Wand, Mishra's Workshop, Strength-Testing Hammer and Sting loop for The One Ring protection (Sting is in our list, Strength-Testing Hammer is not), Devastating Onslaught, Eldrazi Confluence, Flare of Duplication, Return the Favor (copying Godo's ETB), Warrior's Oath, Meteor Sword, Wild Slash, Blind Obedience. Our list does have Basalt Monolith, Metalworker, Voltaic Key, Manifold Key, Krark-Clan Ironworks, Lion's Eye Diamond and Goblin Welder + Engineer, but without Rings or Scepter those do not form [primer: RL]'s infinite-mana lines. [inference] Do not assume they do.

[primer: RL] Dualcaster Mage line: main phase 1, cast the doubler, **hold priority**, flash Dualcaster in; its copy targets Dualcaster; repeat. Floor of the card: a Fork. Doublers also reduce Godo's cost with Hammer. Twinflame can copy any instant with Dualcaster (Seething Song for infinite mana, Lightning Bolt for infinite damage). Note [inference] Twinflame, Electroduplicate and Molten Duplication require a creature you control, so the Dualcaster line needs one on the battlefield.

Tutors that find each piece [notes unless marked]:

- **Godo**: always in the command zone; Command Beacon puts him back in hand without extra tax. Casting from the command zone costs {2} more per previous cast.
- **Helm**: Godo's ETB, Gamble, Goblin Engineer + Welder, Inventors' Fair (needs three artifacts).
- **Hammer**: Godo's ETB (when Helm is already out), Gamble, Engineer + Welder. (Urza's Saga and Tezzeret find only MV 0–1 artifacts, so not Hammer.)
- **Dualcaster Mage**: Imperial Recruiter (power 2), Gamble.
- **Treasonous Ogre**: Imperial Recruiter's usual target unless life is low [primer: RL]; Gamble. Goblin Engineer usually finds Cursed Mirror; Urza's Saga and Tezzeret find Vexing Bauble, a mana source or an untapper (Voltaic Key, Manifold Key); Urza's Cave finds Ancient Tomb, City of Traitors, Cavern of Souls or Talon Gates (Mishra's Workshop is not in our list) [primer: RL].
- **Copy spells**: Gamble, or draws (four of them).
- **Mana pieces**: Tezzeret −3 and Urza's Saga find MV 0–1 artifacts.

## Mulligan

[primer: RL] "A good 4 is better than a bad 7." Mono-red cannot dig out of a mediocre start. A strong hand has (a) a path to victory (a cost-reducer line, an alternate win or a value engine) and (b) enough fast mana. "If you can't see how you plan to win from the opening hand in the first 3 turns, and you're relying on your topdecks, you should probably send it back."

- **Keep, turbo**: a hand that **wins by turn 2, or turn 3 from an early seat** [primer: RL]. Most such hands go through Treasonous Ogre, directly or via Gamble or Imperial Recruiter; Magnetic Theft, Final Fortune or other cost reducers with fast mana and rituals also qualify. Better with a protection piece (Defense Grid, Vexing Bauble) [primer: RL]. Also [primer: LC, notes]: two or more mana sources with fast mana, plus Helm, Hammer, a tutor, or Dualcaster with a copy spell.
- **Keep, engine**: an early value engine with enough **permanent** mana for a long game: Party Thrasher or Staff of Compleation on turn 1, The One Ring on turn 1-2 ("one of the best starts available") [primer: RL; Anje's Ravager is not in our list]; permanent mana means Basalt/Grim Monolith with a Key, big rocks, Thran Dynamo. Burn early rituals into long-term mana if expecting a long game [primer: RL]. Basalt Monolith with Voltaic Key or Manifold Key is an explosive, consistent start [primer: RL].
- **At five or fewer cards**: keep a draw effect or a truly explosive enabler; keep a 3-5 card hand with The One Ring or Treasonous Ogre even without fast mana [primer: RL].
- **Ship**: "a bit of ramp without a clear path to victory is not worth keeping" (ramp-only hands) [primer: RL]; all lands, one land with no rocks, no acceleration and no tutor [notes]. Do not keep an early Trinisphere or other stax piece without a payoff soon: "don't stax for the sake of stax" [primer: RL].
- **_____ Goblin**: reveal three sticker sheets before the mulligan; only those names are available, so they affect keeps [primer: RL]. [inference] The sim treats it as a fixed mana amount; the primer's best nine sheets give at least {R}{R}{R}{R}.
- Mulligans are "critical" because mono-red has no draw engine [primer: LC]. Speed matters more than a perfect hand: A seven with one land, Sol Ring, Lotus Petal, a ritual and Gamble is a keep [inference: Gamble finds the missing piece, and the zero-drops make the turn-2 cast possible without a second land]. [primer: RL] is stricter than [LC]: a hand of rocks and rituals with no named win path is a ship.
- The first mulligan is free in multiplayer [notes].
- A typical keep (turbo): Sol Ring, a Mox or Lotus Petal, a ritual, Imperial Recruiter or Gamble with Treasonous Ogre as the target, and one protection piece.
- A typical keep (older, [notes]): one land, Sol Ring, a Mox or Lotus Petal, a ritual, Gamble or Imperial Recruiter, a protection spell (Deflecting Swat counts only after Godo is out).

## Turn by turn

Turn 1–2 [notes; LC for the general idea]:

1. Zero- and one-cost mana first: Sol Ring, Mana Vault, Grim Monolith, the Moxes, Lotus Petal, Ancient Tomb, City of Traitors.
2. Then support that makes the finish cheaper or safer: Hammer of Nazahn (any Equipment then attaches itself), Brass Squire, Goblin Welder.
3. Blood Moon or Trinisphere early against a greedy table. Blood Moon also hits the deck's own Ancient Tomb, City of Traitors and Urza's Saga, so play those first [notes]. Vexing Bauble counters your own free spells (Moxes, Petal, a free Swat), so play zero-drops before it [notes].

[primer: RL] Rituals need not be saved for the combo turn: it can be worth burning a ritual to get a permanent mana source or a draw engine into play early. Sequence for the key plays: resolve the tutor and Treasonous Ogre **the same turn** (Recruiter or Gamble, then Ogre) rather than tutoring and passing: it is less efficient and opens you to Mindbreak Trap, but tutoring over two turns is telegraphed. Treasonous Ogre is "the most important single mana source"; most of the fastest lines go through it. [inference] Ogre pays 3 life per {R}, so six of Godo's mana from Ogre alone costs 18 life: check life first.

For the Godo cast: fetch Hammer first when a copier is in hand (line 3). A protective permanent followed by an extra-turn spell is "particularly potent" [primer: RL]: for example Defense Grid or Vexing Bauble, then Final Fortune/Last Chance to untap with all mana. Trinisphere only if a payoff follows soon [primer: RL].

Turn 3–4: if the cast has not happened, hold protection and mana, play the card-flow pieces (The One Ring for protection and cards, Wheel of Fortune with Lion's Eye Diamond in response) and cast Godo the first turn the table shows few open answers [notes]. Use Gamble or Imperial Recruiter to find the missing piece.

Later: rebuild with Cursed Mirror, Jaxis or Dualcaster; sweep with Blasphemous Act or Pyrokinesis if the table's creatures block the way; Final Fortune or Last Chance only when it wins [notes; LC plan C].

The win turn: six mana for Godo plus a way to attach, with Deflecting Swat, Pyroblast or Red Elemental Blast ready for the response [notes].

## Interaction

What it holds and what each is for [notes unless marked]:

| Card | Used on |
|---|---|
| Pyroblast, Red Elemental Blast | A blue counterspell aimed at Godo (counters it) or a blue permanent such as Teferi, Time Raveler (destroys it). Oracle confirmed. |
| Deflecting Swat (free with a commander out) | Removal aimed at Godo: choose a new target for the spell or ability. It does nothing against a counterspell on Godo himself (Godo is not out yet) and cannot be cast under Silence. Oracle confirmed. |
| Redirect Lightning ({R} + 5 life, or {2} extra), Untimely Malfunction (modes: destroy artifact, or redirect a spell or ability with one target), Ricochet Trap (free-ish after a blue spell) | Single-target removal or counters. Ricochet Trap changes the target of a spell with a single target, so it can aim a counter at an opposing spell. Oracle confirmed. |
| Tibalt's Trickery | Counter target spell, the caster gets a random free spell instead. Last-resort counter. |
| Chaos Warp, Untimely Malfunction | Remove a stax or hate piece (Blind Obedience, Chalice, Cursed Totem). |
| Hexing Squelcher | Spells you control can't be countered, and creatures have ward—pay 2 life. Cast it before Godo. |
| Cavern of Souls (Human) | Makes Godo, Dualcaster, Recruiter, Jaxis and Magus uncounterable. |
| Defense Grid, Conqueror's Flail, Vexing Bauble | Taxing and locking the table on your turn. Conqueror's Flail attached stops opponents casting spells on your turn (Oracle). |
| Blood Moon, Magus of the Moon, Trinisphere | Stax against multicolour mana bases and cheap free spells. |

**Removal priority** [primer: RL]: Drannith Magistrate and Opposition Agent (stax creatures that stop the combo or the tutor). Our removal for them: Chaos Warp, Pyrokinesis, Blasphemous Act (best wipe [primer: RL]), Goblin Welder as pseudo-removal [primer: RL]. Command Beacon returns Godo to hand and Command Beacon wins through Drannith Magistrate [primer: RL]. Talon Gates of Madara protects the cast; Urza's Cave finds it [primer: RL]. Blind Obedience (opponent's artifacts and creatures enter tapped) is a **silver bullet** against Godo, because every token enters tapped [primer: RL]; our list has no Meteor Sword, so remove it with Chaos Warp before starting the loop [inference].

**Pyroblast and Red Elemental Blast**: the primer's list does not run them; it prefers to "allow our opponents to spend resources stopping each other, rather than trying to play as table police" [primer: RL]. Ours runs both, so [inference]: keep them for the turn Godo is cast or the Dualcaster line starts (to stop a counter or a blue removal), or to kill Teferi, Time Raveler. Do not spend them on blue spells that do not threaten the win.

**Protection kinds** [primer: RL]: redirects (Redirect Lightning, Deflecting Swat) against counterspells or targeted removal; forks (Dualcaster) win counter wars; Cavern of Souls resolves Godo; Defense Grid, Vexing Bauble, Conqueror's Flail and Hexing Squelcher protect the combo turn.

It lets something resolve when the thing is not the win piece and the protection would cost the cast. With two threats on the table it stops the faster, or the one that can win through its own protection first [inference]; the deck has no way to know a hidden Oracle line, so it counts opponents' untapped mana and the cards in their hands.

## When it goes for the win

It tries a win line when [notes + inference]:

| Condition | Go | Wait |
|---|---|---|
| Open mana (blue) | Opponents have 0–1 blue mana, or it holds Pyroblast/REB plus the mana | Several opponents with blue open and no protection |
| Own mana | The line total is available and one spare mana beyond it for protection | Exactly the line total, no protection, and a table showing a counter |
| Hand | The win piece plus at least one protection spell, or Hexing Squelcher already resolved | Only the win piece against open Force of Will mana |
| Board hate | No Blind Obedience, Pithing Needle, Cursed Totem, Teferi, Chalice X=1 | Any such piece is out: remove it first (Chaos Warp, Pyroblast on Teferi) |
| Race | An opponent is about to win | Go anyway, it is the best chance |
| Window [primer: RL] | A hair-trigger window opens (counters tapped, stax creature removed) | Hold the line until it opens, but a fast turn-2/3 win beats waiting |
| Defender with indestructible or large first-strike blocker [primer: RL] | Attack the other two players first; damage marked on a surviving blocker stays until end of turn; then everything at the last player | None |
| The One Ring in play [primer: RL] | Token ETBs fetch Hammer then Sting (untaps this Godo each combat), then Strength-Testing Hammer; the primer's loop needs Strength-Testing Hammer and Wild Slash, **not in our list** | In our list the Ring-protection workaround is unavailable; do not run into a Ring |

## What it fears

[LC: stax and no card advantage, Drannith Magistrate and Null Rod; telegraphed wins; opponents hold interaction for the right moment. CH: Fog effects, first strike or indestructible blockers.] [notes] add:

- **Creature removal in response to Godo's trigger**: Helm has nothing to copy. Play around with Deflecting Swat, Hammer (indestructible), or the Cursed Mirror/Jaxis rebuild.
- **Trigger counters** (Stifle, Tishana's Tidebinder): Camera, Mirror and Jaxis give a second trigger.
- **Artifact hate** (Null Rod, Collector Ouphe, wipes): keep creatures, rituals and lands; kill the hate piece with Chaos Warp or Untimely Malfunction.
- **Stax creatures** [primer: RL]: Drannith Magistrate (stops casting Godo from the command zone) and Opposition Agent (steals tutors). Kill them first with Chaos Warp, Pyrokinesis or Blasphemous Act; Command Beacon wins through Magistrate.
- **Blind Obedience** [primer: RL]: tokens enter tapped; the primer's silver bullet. See Interaction.
- **Turbo Necro / Naus** [primer: RL]: Godo is outpaced by them; all-in stax is no longer viable. Mitigate with speed and Defense Grid-style broad protection.
- **Graveyard hate** matters only for the Welder and Engineer lines.
- **"Can't cast spells" effects** [inference, Oracle confirmed]: Silence ("your opponents can't cast spells this turn") and Orim's Chant (target player can't cast spells this turn) are in Kraum, Sisay and Dack. Cast in response to Godo, they shut off every protection spell for the rest of the turn, including Deflecting Swat, Pyroblast and Magnetic Theft. Godo's trigger and Hammer/Squire attaches still work. Prefer an ability-based attach (Hammer, Squire) over Theft when Silence is possible.
- **Tax and lock pieces in this tier** [inference]: Dack Fayden's list has Chalice of the Void, Trinisphere, Damping Sphere, Blind Obedience, Cursed Totem, Pithing Needle, Static Prison, Deafening Silence, Void Mirror. Their effects on Godo are in "Against Godo".

## Against Godo

How Godo beats each Tier 6 deck and what it must play around. All [inference] from the decklists and Oracle text (checked on Scryfall) unless marked.

**Which counters can hit Godo's cast.** Godo is a creature spell. Fierce Guardianship and Force of Negation counter only noncreature spells, Swan Song only enchantments, instants and sorceries, Dispel only instants, Flusterstorm only instants and sorceries. None of those stops the Godo cast. What does: Force of Will, Pact of Negation, Flare of Denial, Daze, Mindbreak Trap. Mental Misstep hits only MV 1, so Godo is safe, but Magnetic Theft, Gamble, Sol Ring, Mana Vault, Lotus Petal, Camera and Goblin Welder are not. The noncreature counters do hit Helm only if it is cast, and Godo fetches it without casting, so they never touch the main line. They do hit Gamble, rituals, Theft and the copy spells.

| Opponent | How Godo beats it | What Godo must play around |
|---|---|---|
| **Kraum & Tymna** (blue midrange) | A turn-2 or turn-3 Godo meets only Force of Will, Pact of Negation and Mindbreak Trap as counters. Pyroblast or REB beats Force of Will; Hexing Squelcher beats all three. | Silence and Orim's Chant in response to the cast (no protection spells afterwards). Snapback and Into the Flood Maw on Godo with the trigger on the stack. Mental Misstep on Theft or Gamble. Commandeer takes a copy spell or Gamble. Bowmasters punishes extra draws. Deflecting Swat (their own copy) can redirect your single-target removal. Fierce Guardianship and Force of Negation matter only for Gamble, rituals and the Dualcaster line. |
| **Kinnan** (mana creatures) | It plays no removal that stops Godo except Chain of Vapor and Snapback; Godo wins the race if it casts first. | Force of Will, Pact, Flare-type counters on Godo; Chain of Vapor and Snapback on Godo or Helm in response; Wan Shi Tong, Librarian draws them a card per search (cost of casting Godo, not a stop); Misdirection redirects a Swat or Malfunction. Pyroblast and REB are live against most of its hand. |
| **Rograkh & Silas** (turbo Oracle/Breach) | The fastest deck in the field: Godo wins only by casting on turn 2–3. Its interaction is Force of Will, Pact, Daze, Flusterstorm, Mental Misstep, Pyroblast, REB. | Daze on the cast (keep {1} spare). Mental Misstep on the attach piece. Its Pyroblast and REB cannot hit Godo or Helm (neither is blue), so they are dead cards against us. The real danger is the race: if it threatens Oracle first, Godo should go with less protection. |
| **Rograkh & Thrasios** (green creature combo) | Slower to a win than Godo; it has Force of Will, Pact, Flare of Denial, Snap, Chain of Vapor, Swan Song, Veil of Summer, Mindbreak Trap. | Snap and Chain of Vapor bounce Godo with the trigger on the stack (Helm then has nothing to copy: line 5 rebuild); Wan Shi Tong, Librarian draws them cards; Flare of Denial and Force of Will on the cast. Veil of Summer makes its spells uncounterable and stops Pyroblast/REB from touching its blue/green cards. |
| **Sisay** (legends toolbox) | Slower than Godo. Its best answers are Dismember (−5/−5 kills Godo even with Hammer), Abrupt Decay (MV 3 or less: hits Camera, Squire, Sculpting Steel and most rocks, not Hammer or Helm), Cyclonic Rift, Silence, Deafening Silence, Teferi, Time Raveler. It also has red cards (Deflecting Swat, Redirect Lightning) that redirect Godo's answers. | Teferi, Time Raveler (no flash, so no Dualcaster line; bounce Helm or Godo; destroy it with Pyroblast/REB, it is blue). Deafening Silence (one noncreature spell per turn: kills ritual chains and the Dualcaster line). Silence in response. Cyclonic Rift on Godo or Helm at end of turn. Esper Sentinel taxes the first noncreature spell by {1}. Dismember on Godo in response to the ETB: hold Swat to redirect it. |
| **Dack Fayden** (white stax) | Beats it before the stax is out: Dack Fayden, Helping Hand costs six and its own deck is slow. If a lock piece is out, remove it (Chaos Warp, Untimely Malfunction on artifacts) and go. | Blind Obedience (tokens enter tapped: no Helm loop, no Dualcaster loop), Chalice of the Void (X=1 counters Sol Ring, Vault, Camera, Theft), Trinisphere, Damping Sphere (Ancient Tomb gives {C}), Deafening Silence, Cursed Totem (stops Squire, Welder, Jaxis, Engineer, Metalworker, creature mana), Pithing Needle on Helm, Void Mirror (spells paid with no coloured mana), Static Prison / Touch the Spirit Realm / Portable Hole (MV 2 or less) / Fiend Hunter / Swords / Path / Get Lost on Godo, Silence, Orim's Chant. |

Dack Fayden, Helping Hand (Oracle): when it enters, reveal creature cards until X creatures (X = opponents), put them onto the battlefield goaded, and each opponent gains control of one. It is a slow six-drop that gives the table creatures; it does not stop Godo directly.

## Decision scenarios (become AI tests)

1. **Turn 2, 6 mana, Hammer out, one opponent has 2 open blue mana and 1 card.** Hexing Squelcher in hand. Right: cast Squelcher first, then Godo. Reason: Squelcher makes spells uncounterable and also gives Godo ward. Tag [notes]. [inference] for the Force of Will angle.
2. **Turn 2, 7 mana, Godo castable, opponents show Force of Will mana, Pyroblast in hand.** Right: cast Godo holding {R}; if Force of Will, respond with Pyroblast targeting it. Reason: Pyroblast counters a blue spell. Tag [notes; Oracle].
3. **Godo's trigger on the stack, opponent casts Swords to Plowshares on Godo, Deflecting Swat in hand.** Right: cast Swat free and redirect Swords to another creature (or the opponent's own). Reason: Godo must stay for Helm. Tag [notes].
4. **Godo's trigger on the stack and Stifle on the trigger, Camera out with {2} spare.** Right: activate Camera in response to the Stifle only if the Stifle targets the original trigger; copy the trigger. Reason: a second trigger. Tag [notes].
5. **Teferi, Time Raveler is out, Dualcaster line available.** Right: do not go for line 6; remove Teferi with Pyroblast/REB (it is blue) or Chaos Warp, or run line 1 at sorcery speed. Reason: Teferi removes flash. Tag [inference].
6. **Blind Obedience on the table, Godo and Helm in hand or fetchable.** Right: remove it first with Chaos Warp (Untimely Malfunction hits artifacts only); do not start the loop into tokens that enter tapped. Reason: tapped tokens cannot attack, so no extra combat. Tag [inference].
7. **Chalice of the Void at 1, hand: Sol Ring, Mox, Gamble, Godo in the command zone.** Right: Mox (MV 0) first, do not cast MV1 spells; cast Godo if 6 mana without Sol Ring. Reason: Chalice counters MV 1. Tag [inference].
8. **Mental Misstep deck (Kraum) with open blue, hand: Magnetic Theft, Brass Squire.** Right: use Squire (MV 3) as the attach, keep Theft; or cast Theft only after the Misstep is spent. Reason: Misstep hits MV 1 only. Tag [inference].
9. **Hand: Godo cast ready but opponents each hold 3+ untapped mana and the table has shown Force of Will.** Right: wait one turn, cast Squelcher, play mana; do not run Godo into a counter without protection. Reason: Godo is the only reliable piece and the cast risks the game. Tag [primer: LC "be patient"].
10. **Turn 3 against Rograkh & Silas, which has shown Breach-and-Oracle mana; Godo can win this turn but holds no protection.** Right: go. Reason: waiting loses the race. Tag [inference].
11. **Nonbasic-heavy table (Kraum, 25 nonbasic lands), turn 2, Blood Moon and Godo both castable, 6 mana.** Right: Blood Moon first only if it does not cost the Godo cast this turn; cast Godo if the line wins now. Reason: Blood Moon is for a slow game. Tag [notes].
12. **Wan Shi Tong, Librarian on the board; cast Godo.** Right: still cast Godo; the extra card for them is small against a win. Reason: the ETB search triggers Wan Shi Tong once. Tag [inference].
13. **Kraum & Tymna player has W open and Silence in hand (Silence/Orim's Chant are in its list); hand: Godo, Magnetic Theft, Hammer in play.** Right: expect Silence in response to Godo; have the attach done by Hammer, not Theft, and do not rely on Swat/Pyroblast after it. Reason: Silence stops all later spells this turn. Tag [inference].
14. **Hand: Treasonous Ogre, Imperial Recruiter, 7 mana, life 40, turn 3 early seat.** Right: Recruiter for Ogre and cast Ogre the same turn; not tutor-and-pass. Reason: the Ogre line wins next turn; passing is telegraphed and gives a turn. Risk: Mindbreak Trap. Tag [primer: RL]. [inference] if life is under about 20, take a different target.
15. **Hand: Godo (cmd), Twinflame, 8 mana, no Hammer out, Hammer in library.** Right: cast Godo, fetch Hammer, cast Twinflame on Godo; the copy's ETB fetches Helm, Hammer attaches it; go to combat. Reason: 8 mana instead of 11 for the equip. Tag [primer: RL].
16. **Opening seven: Sol Ring, Mox, Lotus Petal, Rite of Flame, Mana Vault, Lion's Eye Diamond, one land; no tutor, no win piece.** Right: ship. Reason: ramp without a path to victory. Tag [primer: RL].
17. **Opening seven: The One Ring, Basalt Monolith, Voltaic Key, one land.** Right: keep. Reason: early engine plus permanent mana. Tag [primer: RL].
18. **Opening six after a mulligan: Treasonous Ogre, Sol Ring, Mox, Mountain, Gamble, Defense Grid.** Right: keep. Reason: a path through Ogre; protection piece. Tag [primer: RL].
19. **Opponents: Drannith Magistrate and Opposition Agent on the table; hand: Chaos Warp, Pyrokinesis, Godo (cmd).** Right: remove Magistrate first (it blocks casting from the command zone), then Agent; Command Beacon if neither can be removed. Tag [primer: RL] for the priority; [inference] for the card choice.
20. **Defender has a Darksteel Colossus-style indestructible blocker; three opponents.** Right: attack the other two players first, then everything at the last. Reason: damage marked on the blocker stays; each token can still go wide. Tag [primer: RL].
21. **Hand: Defense Grid, Final Fortune, 10 mana, Godo castable.** Right: cast Defense Grid first, then Final Fortune, untap, then Godo with equip paid; do not cast Fortune with no win ready. Tag [primer: RL] pattern; [inference] on mana counts.
22. **Pyroblast in hand; an opponent casts a blue draw spell; Godo is not castable this turn.** Right: do not Pyroblast it. Reason: the list runs Pyroblast, but [primer: RL] says not to police the table; save it for the cast. Tag [primer: RL] restraint; [inference] for applying it to our list.
23. **Blind Obedience is out and Godo is in the command zone.** Right: Chaos Warp it before starting the loop, or delay. Tag [primer: RL] that it is the silver bullet.

## Combo tests (become scripted tests)

1. **Godo + Helm, equip paid.** Board: 11 mana available (five Moxes/rocks etc.), Godo in the command zone, empty opposing board. Expect: Godo enters, Helm is fetched and attached by paying {5}, combat creates a hasty token Godo, extra combat, a second token, and so on; opponents die in that turn.
2. **Godo with Hammer out.** Board: Hammer of Nazahn on the battlefield, 6 mana. Expect: Godo's ETB fetches Helm, Hammer attaches it as it enters, the loop runs with no equip payment.
3. **Godo + Camera.** Board: Camera out, 8 mana. Expect: trigger copied; copy fetches Hammer, original fetches Helm; Hammer attaches Helm; combat starts the loop.
4. **Dualcaster loop with Twinflame.** Board: any creature you control (for example Brass Squire), 5 mana (RRR), Twinflame and Dualcaster in hand. Expect: Twinflame targets the creature; Dualcaster flashes in; copies of Twinflame target Dualcaster; the tokens each copy it again; N hasty Dualcasters attack; opponents with no blockers die.
5. **Welder + Engineer.** Board: Welder (untapped, not sick), a used Sol Ring, 8 mana. Expect: Engineer puts Hammer in the graveyard; Welder swaps Sol Ring for Hammer; Godo fetches Helm; Hammer attaches it.
6. **Rebuild after a Stifle.** Board: Godo on the battlefield (trigger stopped), Cursed Mirror in hand, 3 mana. Expect: Mirror enters as Godo, its ETB fetches Helm, attach, attack.
7. **Hammer first, then copy.** Board: 8 mana, Hammer in library, Twinflame in hand. Expect: Godo fetches Hammer; Twinflame on Godo; the copy's ETB fetches Helm; Hammer attaches it with no equip payment; combat makes a token. [primer: RL]
8. **Basalt + Kinnan copy.** Board: Basalt Monolith out, an opponent's Kinnan, 3 spare mana, Heat Shimmer in hand. Expect: the copy of Kinnan; Basalt taps for six, untap for three; a net gain each loop. [primer: RL; inference]
9. **Counter test.** Godo cast into Force of Will with Pyroblast in hand: expect Force of Will countered, Godo resolves.

## Open questions

- [primer: RL] is the first real primer and fills mulligan, sequencing and removal priority. It is for a different list. Lines that need Loki's Scepter, Rings, Panharmonicon, Diviner's Wand, Workshop and Strength-Testing Hammer are marked "not in our list". The lead should decide whether the Basalt + Kinnan copy (opponent's Kinnan only appears in one Tier 6 deck) is worth an AI plan.
- [primer: RL] does not run Pyroblast or Red Elemental Blast and argues against policing; JD's list runs them. The "keep their use narrow" rule here is an inference, not a primer claim.
- [primer: RL] does not give win turns or percentages. It says only that Godo is outpaced by turbo Necro/Naus.
- [LC] earlier items still open: [LC] lists Molten Echoes; our list has Molten Duplication and Electroduplicate.
- [LC] lists Molten Echoes; our list has Molten Duplication and Electroduplicate instead. Both are token-copy sorceries; confirm Molten Duplication works with Dualcaster (it targets "artifact or creature you control" and the token is sacrificed at the end step).
- [3F1] and [CH] say eleven mana; our deck can do it in 7 to 10 with Hammer, Theft or Squire. How often the AI reaches each number is for the Stage 6 calibration, not this spec.
- Real tournament data: 78 entries, 21.4% game win rate in the last six months (TARGETS.md). No source gives Godo's usual win turn.
- Whether the AI should wait for protection or go on the first castable turn is not settled by any source; scenario 9 and 10 pick opposite answers depending on the race.
- Silence and Orim's Chant timing: the AI test (scenario 13) assumes opponents cast them in response to the first spell. No source says how often real pilots do.
