# Thanos, the Mad Titan (key `thanos`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/cali_commander/thanos-artifact-combo.md`, built 2026-11-13 for Cali Commander (1v1, 30 life) from
the previous Thanos list, the Aragorn deck and cards taken from the other decks. Routes: Basalt Monolith + Rings of
Brighthearth, Staff of Domination + Faeburrow Elder, Tinker into Blightsteel Colossus (Sneak Attack as a second way to
put it in), and Thassa's Oracle with Demonic Consultation or Tainted Pact (Underworld Breach with Brain Freeze as a storm
line). **No sims yet.** The findings of the artifacts-and-Dragons list are in `FINDINGS-artifacts-and-dragons-list.md`,
those of the cEDH list in `FINDINGS-old-list.md`.

## 2026-11-15: Triple Triad => Gifts Ungiven (JD's choice, not simulated)
Gifts Ungiven ({3}{U} instant, a Game Changer; the list now has 17 by the cache's flags) takes Triple Triad's slot. Why:
an instant tutor at the opponent's end step with the combo pieces already in the deck. Its piles that hold for every split
(the opponent chooses two of the four for the graveyard) each need one piece already in hand: Thassa's Oracle in hand with
Demonic Consultation, Tainted Pact, Underworld Breach and Demonic Tutor (five of the six splits leave an exile half; if
the opponent bins Consultation and Pact you hold Breach and Demonic Tutor, and Breach has to escape an exile half from the
graveyard, a much bigger turn); Underworld Breach in hand with Oracle and Consultation (both go to the graveyard, then
Breach escapes them). Tinker + Blightsteel + Oracle + Consultation does not hold, and a pile of the Basalt-route pieces
does not either (the opponent bins the Monolith and no tutor finds it in the graveyard). So it adds redundancy and the
Breach pair trick, not a new route, on top of eight 1-3 mana tutors (Demonic, Vampiric, Mystical, Enlightened, Worldly,
Grim, Gamble, Diabolic Intent) and Tinker. What it costs: Triple Triad ({R}{R}{R}) flips few free spells at an average
nonland mana value of 2.53 before the swap, 2.50 after (it went to the Ur-Dragon, where the Dragons cost 5 to 7). Not measured. An Opus consult rated Gifts the weaker of its two
adds for this deck (Mindbreak Trap went to Y'shtola); Fiery Emancipation, the other {R}{R}{R} six-drop, is its next cut. In
the simulator the AI builds Gifts piles from tutor value and graveyard worth (`engine/tutors.rs` `pile_tutor`), not from
the combo, and the Oracle line, Tinker and Sneak Attack have no card code or plan lines for `thanos`, so a sim number
would understate it. Check the blue count when a run exists: 12 land sources make {U}, 14 counting Exotic Orchard (blue
only if an opponent's land makes it) and Plaza of Heroes (which cannot pay for Gifts).

## What the simulator cannot do yet

- **No Cali mode:** the engine plays 4-player pods at 40 life. Cali is 1v1 at 30 life, and the player on the play skips
  their first draw.
- **No Cali ban list:** `cardgen deck` rejects Mana Crypt, Tinker and Primeval Titan as banned in Commander; Cali allows
  them. This list was written by hand and does not pass `cardgen deck` until that is changed.
- **Cards not modelled:** Tinker, Sneak Attack, Mana Crypt, Staff of Domination and the Oracle line (Thassa's Oracle
  with Demonic Consultation or Tainted Pact) need card code or catalogue combos; Haunted Ridge and An Offer You Can't
  Refuse are new.
- **Plan file:** `data/plans/thanos.toml` still describes the previous list and is rewritten with the engine work.

## Open questions

- Staff of Domination, Watery Grave and Woodland Cemetery are binder-only copies; if any is not in the binder it is a
  proxy.
- Whether proxies are allowed at the Cali events JD plays.
