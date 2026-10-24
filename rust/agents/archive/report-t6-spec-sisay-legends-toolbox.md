# Report: T6 spec, Sisay (legends toolbox)

Spec: audit/t6/sisay-legends-toolbox/SPEC.md (not committed).

Sources opened: Archidekt primer "Sisay It Ain't So" (closest to our list, undated), note.com Sisay write-up 2026-08-19,
Commander's Herald 2024-04-22, TheGamer 2025-03-12, Draftsim 2026-08-20, docs.mdbook.me 2026-10-16, EDHREC combo and cEDH
pages, EDHTop16 list page (no write-up), cEDH Decklist Database (blurbs only). Moxfield: HTTP 403. Oracle text for all 100
cards from Scryfall via /cards/collection (single-card calls were rate-limited by other agents sharing the IP).

Findings:
- No source gives a single full primer for this exact list. The Archidekt primer is the best match (Deadpool + Forbidden Orchard,
  Mount Doom, Cauldron, Marvin + Ioreth are all in our list).
- Win lines in our list: (1) Bloom Tender + Derevi + Emiel mana loop, (2) Derevi + Emiel + Cradle + Cauldron, (3) Marvin +
  Ioreth + legendary mana source, finishers Mount Doom (fast) or Deadpool + Orchard (waits for upkeep), Sisay beats as last resort.
- Lines in older primers that our list CANNOT do: Aminatou + Bolas + Oath of Teferi, Finale of Devastation + Kinnan,
  Thassa's Oracle + Consultation/Pact, Dockside + Emiel (Dockside banned per note.com).
- Fierce Guardianship and Deflecting Swat are only free with Sisay on the battlefield.
- Free counters that can hit Godo's cast: FoW, Pact. Ertai counters Godo's enter trigger. Spider-Woman makes Godo tokens enter tapped.

Claim tags: about 20 [primer] and 16 [inference] tag uses (tags counted per use, not per claim).

Lead decisions:
- Mulligan, interaction spending and who-to-stop have no primer; they are marked [inference].
- Whether the AI should model Marvin + Ioreth with Shang-Chi's restricted mana (works only for creature abilities).
- Mount Doom as the main finisher is the primer's choice but needs {1}{B}{R} each ping and an untap per ping.
