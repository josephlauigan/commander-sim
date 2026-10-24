# Report: Rograkh & Thrasios SPEC updated with AnthonyD3288's primer

File: audit/t6/rograkh-thrasios-creature-combo/SPEC.md (rewritten in place; nothing committed). Source: JD's extract
primers/rograkh-thrasios-anthonyd.md (Moxfield, changelog to 2026-10-12), added to Sources as [AD], ranked first.
Scryfall: one POST to /cards/collection (about 40 names). Not found by that name: Invasion of Ikoria // Zilortha.
Not rechecked: Veil of Summer, Archon of Emeria (wording from [AD]).

## What changed
- Sources, Game plan, Win lines (now A-K), tutor table, Turn by turn, Interaction, Go/wait, Fears, Godo, scenarios
  (11 -> 22), combo tests, Open questions all revised. Mulligan unchanged (AD gives none).
- New lines in our list: A Oboro+Cradle+Talon Gates (AD Plan A, with phase-out, instant speed, no spells), C Badgermole
  Cub + clone + Oboro + Cradle, D Cub x2 + two earthbent lands, E Faerie Mastermind + GSZ (+ Veil), F Finale (main phase 1
  only), G Springheart Nantuko + Vibrance loop. Oboro rule: do not cast Oboro on a non-winning turn.
- Thrasios-buries-interaction pattern added (Interaction, Godo, scenario 17). Fears section now follows AD's list
  (Cursed Totem, Cage/Runestone, Opposition Agent, Rule of Law family, wipes, Bowmasters, Silence effects).
- "Secret priority" trick: Open questions only, marked out of scope; no scenario uses it.
- New scenarios 12-22 are tagged [primer: AnthonyD] (inferences marked where mixed). Tag count: about 60 AnthonyD tags.

## Tags moved [inference] -> [primer: AnthonyD]
Line A pieces/steps, Cradle count 6 (5 for line B), Crop Rotation/Chord/Nature's Rhythm/Speaker/Spellseeker/Vibrance
tutor roles, Thrasios as instant-speed outlet, early creature count as priority, Springleaf Drum/early Cradle,
two cheap commanders enabling free spells, Finale from graveyard dodging Opposition Agent, removal and wipes as fears,
Rhystic/spell-limit resilience of line A, Gates ETB as a real benefit (was "optional"), recursion for Oboro.

## Conflicts and corrections
1. Old spec: Finale not findable by Spellseeker. Wrong by Oracle: Finale in library is MV 2 ({X}{G}{G}), so
   Spellseeker can find it [inference, Oracle]. Fixed.
2. Old spec: "every named loop needs Cradle". AD has Cradle-free lines (Cub x2, Breach, Guildmage); Cub x2 is in our list.
3. Old spec treated Gates' phase-out as optional extra; AD treats it as a main benefit (phases out the table, even at 5).
4. EDM listed Brain Freeze and Underworld Breach; our list has neither (re-verified: 0 hits). AD prefers them; open for lead.
5. Old Finale note lacked "main phase 1 before combat" (AD). Added.

## Lines not in our list
Breach + Brain Freeze + LED (Gamble and Shifting Woodland are in); Izzet Guildmage; This Town Ain't Big Enough lines
(Witness + This Town + Candelabra, Vibrance + Witness + This Town, "Exodia"); Displacer Kitten; Marwyn + Six + LED;
Flash Photography. Also absent: Orcish Bowmasters, Kinnan, Imperial Recruiter, Seedborn Muse, Thassa's Oracle.

## For the lead
- Decide if the sim's Rograkh list stays the Pittsburgh list or gains the Breach pieces (AD's "best way").
- Cub lines (C, D) and the GSZ re-draw mechanic in line E are not mana-counted in AD; flagged in Open questions.
- Scenarios 14, 16, 20, 21 mix primer facts with my reading; their tags say so.
