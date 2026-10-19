# Report: Sisay SPEC updated with the Tolaria_East primer

File: audit/t6/sisay-legends-toolbox/SPEC.md (only file edited besides this report). Not committed.

## What changed
- Sources: [TE] "Oath of Sisay" (Moxfield, 2026-08-29) added first, marked as outranking the rest; link and extract path.
- Confirmed claims retagged [primer: Tolaria_East]: win lines 1-4, Totem-first scenario, Mount Doom finish, Cursed Totem /
  Opposition Agent as top enemies, "double the mana per turn" opening plan, tutor chains.
- New win lines: 7 Selvala end-step chain, 8 rainbow-dork chain, 9 Mana Vault + Kinnan + Cauldron starter, 10 Flash Unicorn on
  the opponent's turn, 11 Ertai lock (not a win). Lines 12 (Oath of Bolas) and 13 (Saheeli / Orcish Bowmasters finishers) marked
  "not in our list".
- Conflict resolved in the primer's favour: Mental Misstep is reserved for counters / Demonic Consultation / 1-MV tutors, not
  Sol Ring, Vault or dorks (old inference and scenario 1 corrected; Against Godo line adjusted).
- New sections: per-card interaction rules table, "uh oh" priority list with our answers (Blood Moon is worse for us: 28
  nonbasic lands, no basic), tutor timing, card roles, mulligan additions, fears rows.
- Scenarios 12-22 (11 new) and combo tests 6-12 added.

## Oracle checks (one Scryfall POST /cards/collection, 20 names, none missing)
Sisay: "mana value less than Sisay's power" (confirms the old spec; [TE]'s "MV <= number of legends" aside is wrong).
Also checked: Mount Doom, Derevi, Emiel, Ertai, Mental Misstep, Marvin, Ioreth, Bloom Tender, Kinnan, Cauldron, Enduring
Vitality, Tyvar, Cursed Totem, Opposition Agent, Deadpool, Shang-Chi, Mindbreak Trap, Cryptolith Rite, Selvala.
Not re-fetched (from memory, flagged in Open questions): Mana Vault, Boseiju, Legolas's Quick Reflexes.
Note: a first POST of the same call lost its output to a jq quoting error, so Scryfall received two POSTs, not one.

## Tag counts in the spec now
[primer: Tolaria_East] about 35 uses; older [primer] tags with letters [A][H][G][J][E]: unchanged; new [inference] tags are few
(line 7 maths, line 9 count, Ertai-in-response, Blood Moon vs our lands, first-fetch note).

## For the lead
- Line 7 (Selvala chain) net mana (+1 per loop) is my own Oracle maths; give it a scripted test before trusting it.
- Our list lacks the author's Oath and Bowmasters pieces, so the Stage 3 combo set should be lines 1-5, 7-10.
