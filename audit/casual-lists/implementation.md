# Casual pool rebuild implementation, 2026-11-03

The source is saved WIP 44630929, based on the existing EDHREC bracket-3 average deck imports and earlier documented pool tuning in `decklists/pool/CHANGELOG.md`, rather than a newly verified external list. Brago retains Azorius blink/control and adds combat pressure, draw and answers; Korvold retains Jund sacrifice and adds recurring bodies, drain, removal and a Gravecrawler line. Their commanders are unchanged. Tier 2 has two Game Changers; Tier 3 has three. `cardgen validate` passed the 100-card, singleton, identity, legality and tier Game Changer checks for all pool decks.

## Evidence and limits

The original brief reports Korvold at 11.2–12.1% within Tier 3 and Brago at 16.5–17.3% within Tier 2 after earlier tuning, with traced games showing Korvold arriving too late and Brago often inactive until rounds 5–6. The surviving scratch files contain old candidate runs under lookahead, loose, 300 games, seed 500000, jobs 8: `chk/k-v3-t3.txt` reports Korvold 14.5% (10.6–19.4), `chk/b-v1-t2.txt` Brago 20.1% (15.5–25.6). These wide intervals do not establish a clear gain, and the exact final lists changed after those runs. The saved `b-v2-t2.txt` was incomplete. No final-list simulations were run here; the requested 20–30% goal remains unproven, especially for Korvold. Raw files are in `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/casual-lists/chk/`. The final 1500-game queue is `/tmp/resume-casual-queue.sh` and must use a binary built from the reviewed branch.

## Combo support and bracket

The implemented Gravecrawler / Phyrexian Altar line (`impls/combos.rs`) requires another Zombie (Carrion Feeder or Midnight Reaper in this list) and an explicitly supported payoff such as Blood Artist, Zulaport Cutthroat or Mayhem Devil. Dreadhorde Invasion can also create a Zombie Army. Chatterfang / Pitiless Plunderer has a simulator line with additional outlet, payoff and fodder requirements (`impls/t3_lines.rs`). Brago's Ghostly Flicker / Drake / Archaeomancer line (`impls/cc_brago.rs`) requires a castable Flicker, four lands and an available Agent finish. Spellbook also identifies Drake / Elesh Norn / Venser; no explicit simulator line for this was found, so it is not claimed as an implemented combo. Spellbook identified an inherited two-card Drake / Eldrazi Displacer infinite mana line; Displacer was removed to satisfy the Tier 2 pool rule, and the refreshed data has no included two-card line. Spellbook records contextual prerequisites; see the data files for them.

## Swap verification

Cuts below were compared to `origin/main:data/decks.json` after fetching. The one additional Swamp replaces a land in Korvold. All additions' cached Commander legality and identity fit the relevant commander, as listed. The final current `cardgen audit --cards` reports every addition Full; Full is the project's status and does not mean every AI choice is exact. Audit notes follow each card below, including known simplifications.

## brago-azorius-blink-control

Cuts present on origin/main: Basalt Monolith, Cloudshift, Dovin's Veto, Eerie Interlude, Eldrazi Displacer, Lightning Greaves, Momentary Blink, Mystic Remora, Panharmonicon, Port Town, Recruiter of the Guard, Solemn Simulacrum, Spellseeker, Witch Enchanter // Witch-Blessed Meadow

Additions:

- Aven Mindcensor: Commander legal, identity W; Game Changer false.
- Brimaz, King of Oreskos: Commander legal, identity W; Game Changer false.
- Consecrated Sphinx: Commander legal, identity U; Game Changer true.
- Council's Judgment: Commander legal, identity W; Game Changer false.
- Dream Trawler: Commander legal, identity UW; Game Changer false.
- Elspeth, Sun's Champion: Commander legal, identity W; Game Changer false.
- Faerie Mastermind: Commander legal, identity U; Game Changer false.
- Fateful Absence: Commander legal, identity W; Game Changer false.
- Fiend Hunter: Commander legal, identity W; Game Changer false.
- Mirror Entity: Commander legal, identity W; Game Changer false.
- Serra Angel: Commander legal, identity W; Game Changer false.
- Solitude: Commander legal, identity W; Game Changer false.
- Spark Double: Commander legal, identity U; Game Changer false.
- The Wandering Emperor: Commander legal, identity W; Game Changer false.
## korvold-jund-sacrifice

Cuts present on origin/main: Bojuka Bog, Crop Rotation, Dark Ritual, Deflecting Swat, Impulsive Pilferer, Juri, Master of the Revue, Marionette Apprentice, Professional Face-Breaker, Reanimate, Reliquary Tower, Ruthless Technomancer, Springbloom Druid, Victimize, Warren Soultrader

Additions:

- Bastion of Remembrance: Commander legal, identity B; Game Changer false.
- Dreadhorde Invasion: Commander legal, identity B; Game Changer false.
- Go for the Throat: Commander legal, identity B; Game Changer false.
- Goblin Rabblemaster: Commander legal, identity R; Game Changer false.
- Gravecrawler: Commander legal, identity B; Game Changer false.
- Heartless Act: Commander legal, identity B; Game Changer false.
- Impact Tremors: Commander legal, identity R; Game Changer false.
- Lightning Bolt: Commander legal, identity R; Game Changer false.
- Pawn of Ulamog: Commander legal, identity B; Game Changer false.
- Sheoldred, the Apocalypse: Commander legal, identity B; Game Changer false.
- Skullclamp: Commander legal, identity ; Game Changer false.
- Terror of the Peaks: Commander legal, identity R; Game Changer false.
- Worldly Tutor: Commander legal, identity G; Game Changer true.

## Current card audit

```text
Full | Aven Mindcensor | flash, flying; opponents searching their library search only the top four cards
Full | Brimaz, King of Oreskos | vigilance; a 1/1 Cat attacking with it, and one blocking with it
Full | Consecrated Sphinx | flying; may draw two whenever an opponent draws
Full | Council's Judgment | will of the council: each player votes for a nonland permanent they don't control (their biggest threat); every permanent with the most votes is exiled
Full | Dream Trawler | flying, lifelink; draws on attack (+1/+0 per draw); discard for hexproof in response
Full | Elspeth, Sun's Champion | +1 three Soldiers, -3 destroys power 4+, -7 emblem (+2/+2, flying)
Full | Faerie Mastermind | flash, flying; draws when an opponent draws their second card each turn (the symmetric draw ability is never worth it and is not used)
Full | Fateful Absence | destroys the target creature or planeswalker; its controller investigates
Full | Fiend Hunter | exiles the best opposing creature (the AI never takes its own); the card returns under its owner's control when Fiend Hunter leaves
Full | Mirror Entity | changeling; {X}: until end of turn your creatures have base power and toughness X/X and every creature type (the AI pumps with its spare mana before combat)
Full | Serra Angel | vanilla creature (keywords only)
Full | Solitude | flash, lifelink; exiles the best opposing creature on entry; evoke by exiling a white card (free)
Full | Spark Double | enters as a copy of your best creature or planeswalker with a +1 counter
Full | The Wandering Emperor | flash: cast during an opponent's attack to exile a tapped attacker (-2); otherwise +1 counter and first strike, -1 2/2 Samurai
Full | Bastion of Remembrance | Human Soldier on entry; each creature you control dying drains each opponent 1, you gain 1
Full | Dreadhorde Invasion | upkeep: lose 1 life and amass Zombies 1; a Zombie token you control with power 6 or greater gains lifelink as it attacks (the Army counts as a Zombie while Invasion is out)
Full | Go for the Throat | tags: `rem=destroy tgt=cna`
Full | Goblin Rabblemaster | a Goblin with haste each combat that must attack; other Goblins must attack; +1/+0 per other attacking Goblin
Full | Gravecrawler | castable from the graveyard with a Zombie; the Phyrexian Altar loop is a combo (see combos)
Full | Heartless Act | destroys a creature with no counters, or removes up to three counters (and kills it if that is lethal)
Full | Impact Tremors | 
Full | Lightning Bolt | tags: `rem=dmg3 tgt=c face`
Full | Pawn of Ulamog | a 0/1 Spawn when a nontoken creature of yours dies (sacrifice: {C})
Full | Sheoldred, the Apocalypse | tags: `bomb=7 pow=4 tgh=5 dt leg sheoA`
Full | Skullclamp | equip {1} onto 1-toughness creatures to draw two, up to twice a turn
Full | Terror of the Peaks | flying; opponents' spells targeting it cost 3 life; damage equal to power when your creatures enter
Full | Worldly Tutor | a creature on top of the library: the wish list first, cast at the end of the turn before yours

```

## Coordinator validation corrections

The full release suite found a stale Brago Cloudshift priority; it was removed from the plan.
The Spellbook guard then found unsupported Gravecrawler + Pitiless Plunderer + Viscera Seer / Carrion Feeder loops.
Gravecrawler is therefore excluded from the final list and the original Full-modeled Impulsive Pilferer retained.
Earlier swap tables above describe the intermediate list; this correction supersedes its Gravecrawler entry.
