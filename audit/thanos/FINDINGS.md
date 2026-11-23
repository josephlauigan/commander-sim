# Thanos, the Mad Titan (key `thanos`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/cali_commander/thanos-artifact-combo.md`, built 2026-11-13 for Cali Commander (1v1, 30 life) from
the previous Thanos list, the Aragorn deck and cards taken from the other decks. Routes: Basalt Monolith + Rings of
Brighthearth, Staff of Domination + Faeburrow Elder, Tinker into Blightsteel Colossus (Sneak Attack as a second way to
put it in), and Thassa's Oracle with Demonic Consultation or Tainted Pact (Underworld Breach with Brain Freeze as a storm
line). **No sims yet.** The findings of the artifacts-and-Dragons list are in `FINDINGS-artifacts-and-dragons-list.md`,
those of the cEDH list in `FINDINGS-old-list.md`.

## 2026-11-22: Thought Vessel, Fiery Emancipation, Nibelheim Aflame => Mox Opal, Tezzeret, Cruel Captain, Chrome Mox (JD's choice, not simulated)
An artifact package of three pairs, none of them measured (the engine has no Cali mode, so no paired test could judge it).
**Mox Opal** ({0}, legendary, metalcraft: any colour with three or more artifacts) takes Thought Vessel's slot, rock for rock: Thought
Vessel made only {C}, which the deck has many other sources of (Sol Ring, Mana Crypt, Mana Vault, Basalt Monolith, Mind Stone, the Mightstone
and Weakstone, Ancient Tomb, Strip Mine, Urza's Saga, the painlands). An Opus consult picked Thought Vessel because JD did not want Fellwar
Stone cut; the runner-up was Mind Stone, which can cash in for a card. The artifact count is unchanged by this pair. **Tezzeret, Cruel
Captain** ({3}) takes Fiery Emancipation's slot: his 0 untaps the Basalt Monolith (twice with Rings of Brighthearth) and his -3 tutors an
artifact of mana value 1 or less (Sol Ring, Mana Crypt, Mana Vault, Lotus Petal, the Moxen, Esper Sentinel, and Walking Ballista or Hangarback Walker at
mana value 0 in the library). **Chrome Mox** ({0}, a Game Changer) takes Nibelheim Aflame's slot (JD preferred it to Grim Monolith): it
imprints a nonartifact, nonland card, so it costs a card. Fiery Emancipation was the card the earlier notes named as the weakest; Nibelheim Aflame is the other {R}{R} card that left. What it costs: two red cards leave the deck (Thanos stays the red commander, with Chaos Warp, Deflecting Swat, Jeska's
Will, Gamble, Underworld Breach, Sneak Attack and Ragavan still red); the Moxen's colours need a card to imprint, and Mox Opal is dead until
three artifacts are out (the deck has 25 with the five artifact creatures, 24 before). Counts after: 15 rocks and rituals (14), enchantments
4 (5), sorceries 7 (8), one planeswalker, Game Changers +1; lands and land colour sources unchanged. A fourth pair (Windborn Muse => Mishra's
Bauble) was left out (JD took three). Engine status: Mox Opal, Tezzeret and Chrome Mox are Full (`cardgen find --full`). The plan file's
turn<=4 rock priority now lists Mox Opal and Chrome Mox in place of Thought Vessel; Tezzeret has no plan line yet. Sculpting Steel must not copy
Mox Opal (legendary) or Chrome Mox (the copy has nothing imprinted). Staff of Domination stays (a proxy, printing cmr 343). Route A2 is unchanged.
Nothing in `data/spellbook/thanos.json` was re-fetched: the two entries that used Fiery Emancipation were removed, and combos for the new
cards are not in the file.

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
adds for this deck (Mindbreak Trap went to Y'shtola); Fiery Emancipation, the other {R}{R}{R} six-drop, was its next cut (it went out on 2026-11-22, above). In
the simulator the AI builds Gifts piles from tutor value and graveyard worth (`engine/tutors.rs` `pile_tutor`), not from
the combo, and the Oracle line, Tinker and Sneak Attack have no card code or plan lines for `thanos`, so a sim number
would understate it. Check the blue count when a run exists: 12 land sources make {U}, 14 counting Exotic Orchard (blue
only if an opponent's land makes it) and Plaza of Heroes (which cannot pay for Gifts).

## 2026-11-15: Elesh Norn => Sculpting Steel (JD's choice, not simulated)
Sculpting Steel ({3} artifact, colourless, Commander-legal) takes Elesh Norn, Mother of Machines' slot. Why JD chose it:
a flexible artifact that copies the best artifact already on the battlefield. In this list: a second Rings of Brighthearth
(two Rings let one Basalt Monolith untap be copied twice: {7} spent for three taps of {C}{C}{C}, two extra mana per loop
instead of one), a second Blightsteel Colossus (an artifact creature) after Tinker or Sneak Attack as insurance against a
blocker or removal, or a second colourless rock for Thanos's power-up. What it costs: it needs an artifact already out, so
it finds no combo piece, and it is a plain artifact on an empty board. Elesh Norn doubled the enter triggers of Urza,
Meteor Golem, Primeval Titan and The Mightstone and Weakstone, and stopped the opponent's enter triggers (a defence
against an opposing Thassa's Oracle), so those go too. An Opus consult (Dockside Extortionist, not Steel, was the question
then) ranked Mana Drain, Chrome Mox and Imperial Seal first and advised cutting Fiery Emancipation, the weakest card,
rather than Elesh Norn; Fiery Emancipation went out on 2026-11-22 (above). The engine models Sculpting Steel (Full: it enters as a copy of the best artifact on the
battlefield, `card_overrides.toml`), but `thanos` has no plan line for it and Tinker, Sneak Attack and the Rings loop are
not modelled, so a sim number would not reflect the Rings and Blightsteel lines. Not measured. Commander
Spellbook lists no combo with Sculpting Steel for this list.

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
