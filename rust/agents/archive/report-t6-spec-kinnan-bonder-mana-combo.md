# Report: Tier 6 spec, Kinnan, Bonder Prodigy (kinnan-bonder-mana-combo)

Spec: audit/t6/kinnan-bonder-mana-combo/SPEC.md (not committed).

## Sources opened
learncedh.com/decklists/kinnan; EDHREC article "Building Kinnan, Bonder Prodigy for cEDH" (2026-05-19, closest to our list);
Commander's Herald intro (2023-12-08); cEDH Decklist Database entry (2023-12-15); Forest Birds Go (2020); Commander Spellbook and
EDHREC combo pages; EDHTop16 commander and decklist pages; EDHREC cEDH page; Moxgate (Bracket 3, low weight); Scryfall oracle for 77 cards.
Blocked: Moxfield, TappedOut, MTGNexus (403), Reddit (redirect), topdeck.gg (empty).

## Claims
40 tags [primer], 25 tags [inference] (counts of tag marks in the file, including table rows).

## Findings the lead must know
1. Our list has no Thassa's Oracle, Isochron Scepter, Walking Ballista, Seedborn Muse, Nezahal, Crop Rotation, Cephalid Coliseum or Mana Drain.
   Primer lines that need them are marked unavailable.
2. Dramatic Reversal is not a stand-alone win here: every primer's Reversal combo uses Isochron Scepter. TARGETS/pool text says
   "Basalt Monolith, Dramatic Reversal". Decide: model Reversal as ramp only (the spec does) or fix the pool text.
3. Win lines in our list: (A) Basalt Monolith + Kinnan with Thrasios or Mirage Mirror as outlet, finishing with Finale of Devastation X of 10+
   and an attack; (B) Hullbreaker Horror + Mox Amber + Lotus Petal loop; (C) Horror lock; (D) finite big turn. Thrasios is the only draw outlet.
4. Emrakul, the Promised End is in the tournament list but in no primer: its role is an open question.
5. Against Godo: Fierce Guardianship, Force of Negation, Offer and Commandeer cannot hit Godo (a creature spell). Real answers are Force of Will,
   Pact, Disrupting Shoal (X=6), and bounce (Snapback, Chain of Vapor, Sink into Stupor) on Godo while Helm's attach or equip is on the stack.
6. Interaction priority, two-threat choice and go/wait conditions are inference (no source covers them).
