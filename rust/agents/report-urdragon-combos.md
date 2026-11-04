# Report: Ur-Dragon infinite combos in the catalogue (2026-11-04)

The agent's harness refused report files, so the coordinator wrote this from the agent's final message and its own
review of commit 477f73e4 and the fixes on top.

## The three combos (`impls/t45_urdragon_combos.rs`, chained in `combos::all()`)
These are stepped combos in the pattern of `t6_combos_godo.rs`. Each step pays its own mana, read through `cost_of`,
so eminence and taxes count.

1. **Astral Dragon + Cursed Mirror** (Spellbook 703-5291).
   - **Ready:** p's turn before combat. The Mirror is out as a noncreature artifact (not a creature copy, no
     `revert_to`), and Astral Dragon is castable and affordable.
   - **Steps:** the cast (counter window), Project Image (ability-answer path), then both pieces must stay.
   - **Finish:** `None`, so the player wins by combat with hasty Dragons, like Helm + Combat Celebrant.
2. **Miirym + Astral Dragon + Doubling Season** (Spellbook 380-703-4772).
   - **Ready:** the pieces and the mana are there. There's no timing gate, because the tokens are summoning sick
     anyway.
   - **Steps:** the cast, Miirym's trigger, Project Image, then the pieces must stay.
   - **Finish:** `dragon_flood` fills the board with 3/3 flying Dragon tokens up to `TOKEN_CAP`. The kill is the next
     combat, and a sweeper still answers it.
3. **Garth One-Eye + Twinflame + Spark Double + Doubling Season** (Spellbook 1171-1235-3908-4772).
   - **Ready:** p's turn before combat. Doubling Season, a plain Garth and a Spark Double Garth are out, and Twinflame
     is castable.
   - **Mana:** seven, which is {1}{R} (Twinflame), {1}{G} (Regrowth) and {2}{R} (the second Twinflame's strive).
   - **Steps:** Twinflame, a token Garth's ability, the Regrowth copy, then the pieces must stay.
   - **Finish:** `None`, so the player wins by combat with hasty Garths.

## Coordinator's fixes in review
- **The Garth line's mana check was too low.** The agent had it at four mana: {1}{R} and {1}{G}. But one Twinflame on
  one Garth makes two tokens, Regrowth plus one Black Lotus, which nets −1 a round. The loop gains mana only once
  Twinflame strives onto two or more Garths, netting 3n − 4 for n targets. So the check now also requires the {2}{R}
  strive, seven mana in all, the same as Spellbook's {4}{R}{R}{R}. The tests were updated: seven lands, and "six open
  mana" is not ready.
- **Cursed Mirror stays an artifact when the AI holds Astral Dragon** (`t0_stella::mirror_copy`). Otherwise the AI
  could copy a creature with it and push the loop back a turn. There's a new test for this.

## Plan file (`data/plans/urdragon.toml`)
- **Lines:** three `[[line]]` entries ahead of the value lines, each with `combo` set to the exact catalogue name.
- **Priorities:** Astral Dragon is 92 when a Mirror is out, or when Miirym and Doubling Season are both out. Cursed
  Mirror is 80 when Astral Dragon is held.
- **Header:** it now names `decklists/JD/ur-dragon-dragon-copies.md`.

## Shared files
- `impls/combos.rs`: one `.chain`.
- `impls/mod.rs`: one `pub mod`.
- `impls/t45_miirym.rs`: `astral_etb` returns early while `g.combo_spell` is set, because the combo's Trigger step
  stands in for it.
- `impls/t0_stella.rs`: the Mirror choice above.
- **Global effect:** the six cards are now catalogue piece names for every deck (`is_piece_name`). Opponents' removal
  and counter valuation reads them, so the Miirym pool deck and Sauron may shift slightly.

## Simplifications
- The Lotus and Braingeyser rounds aren't played out; the finish resolves the loop.
- Twinflame targets one Garth in the modelled steps.
- The Regrowth copy fires no cast triggers.
- Lines 1 and 3 need the Mirror or Spark Double already on the battlefield. A piece still in hand doesn't count.
- Line 2 caps the board at the engine's token limit.

## Tests
`tests/combos_urdragon.rs` has 18 tests. Per combo, they cover:
- not ready with a piece missing, too little mana, or the wrong timing;
- ready with everything in place;
- a winning or board-filling run;
- Counterspell and Swords to Plowshares stopping it.

The Mirror test is the 18th.
