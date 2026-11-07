# Report: pilot guide for decklists/JD/yshtola-esper-drain.md

Lines 275 -> 465. Import list, Decklist by type and Bracket and Rule 0 blocks are byte-identical to before; header table unchanged.

## Added or rewritten (under ## Strategy, in the brief's order)
Commander (Oracle quoted, trigger table, when to cast), How the deck wins (drain-pieces table, Lines 1-5: Blood + Bond, X drain with Bond,
Coffers + Urborg, Torment of Hailfire, the grind), Game plan (+ go/hold, pillowfort), Engine cards (kept from the old doc), Mulligans (keep table + 8 hands),
What to tutor for (+ Ringsight and Zur), Protecting the win turn / Y'shtola, Interaction table, Matchups (5 rows), After a stopped attempt,
Life management (two tables), Common mistakes (10). The old "Key lines", "Life gain is the engine", "Weak spots" sections were folded into these.

## Corrections to the old text (found by reading Oracle text and counting the list)
- "Thirty-four spells trigger her" -> 33 (31 noncreature MV 3+ cards plus Exsanguinate and Torment with X >= 1).
- Sphere of Safety "the deck runs 15" enchantments -> 13. Card-type line (15 ench, 8 art, 11 instants) -> 13 / 9 / 12.
- Curve line -> 1 / 8 / 10 / 22 / 13 / 6 / 3 (counted from scryfall_cache cmc; average 3.08).
- Mistrise Village enters tapped (no Mountain or Forest in the list); the table said 0 lands always enter tapped -> 1.
- Skyclave Apparition: the old text said the exiled card returns when it leaves; Oracle says the owner gets an X/X Illusion token and the card stays exiled.
- Removed dated notes ("JD, 2026-11-04", "JD 11/04 package") and Winota (no longer in t5).

## Rules points I worked out (please spot-check)
- Line 1: Y'shtola's cast trigger of the second enchantment does NOT start the Blood + Bond loop (it resolves with the piece still on the stack); a later life change does.
- Ringsight: colourless cards and lands can't be found; with no Y'shtola the Ring-bearer (made legendary) supplies the colour. Ring text itself is not in data/scryfall_cache.json; I wrote it from memory.
- Coffers without Urborg counts 12 Swamp-type lands (8 Swamp + Underground Sea, Watery Grave, Scrubland, Godless Shrine).
- Padeem makes Urza's artifacts hexproof (my targeted answers fail); Phyrexian Unlife in Zur's list keeps an opponent alive at 0 or less under the Blood + Bond loop; Drannith Magistrate stops casting her from the command zone and suspended Profane Tutor.

## Sim claims (only from audit/yshtola/FINDINGS.md)
~35% of t5 games lost to Kinnan/Urza combos (10/18, 11/07 entries); ~49% to Kinnan, Urza, Zur combos plus Miirym combat (11/04); 26.5% baseline B (before package Y); plan online ~65%;
~a third of wins by combo (34% baseline, 27-29% Blood + Bond in 11/04 package runs); Torment cast in 39% of draws at round 9.2. FINDINGS has nothing on Yawgmoth, so the Yawgmoth row is from the pool list only and says so.

## Could not verify
- P/T for most creatures is missing from the cache (I avoided stating P/T except where the cache has it); Esper Sentinel's power is not stated.
- Fierce Guardianship, Force of Negation text were not read from the cache (opponent cards, named only).
- The hand-by-hand turn sequencing in the 8 sample hands is my own reading of mana/colours; no sim backs the calls.
