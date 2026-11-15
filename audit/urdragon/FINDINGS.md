# The Ur-Dragon (key `urdragon`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/urdragon-five-colour-dragons.md`, built 2026-11-13: an optimised high-bracket-4 Dragons list with
proxies allowed, reworked 2026-11-14 (28 Dragons besides the commander, 29 with Morophon, ten original duals, seven fetches, five shocks; the Garth One-Eye + Spark Double + Twinflame +
Doubling Season combo). **Tiers:** t5 (`data/my_decks.toml`). **No plan file and no sims yet.** The findings of the
retired Ur-Dragon decks are in `audit/retired/urdragon/`.

## What the simulator cannot do yet

- **Plan file:** there is none; the generic pool policy plays the deck until `data/plans/urdragon.toml` is written.
- **Cards modelled before:** most Dragons were modelled for the retired pool deck (`impls/t45_urdragon.rs`) and the Garth combo for the retired JD deck. Not re-checked for this list. Klauth and Rivaz
  are Full for the AI and a person (spell-only turn-long mana; a graveyard cast once per turn). Relic of Sauron (two mana
  in blue, black or red, and a {3} loot) and Smaug the Impenetrable are Full; a person picks each colour Relic makes.
  Niv-Mizzet and Utvara Hellkite are compiled from Oracle text and covered by `tests/cards_urdragon_swaps.rs` (Utvara
  triggers only for Dragons that attack). The other new cards are Full. Nature's Lore and Three Visits fetch any Forest
  card (a person picks; the AI takes the best dual) and follow the land's own enters rule.

## Open questions

- Ten original duals are the expensive part of the mana base; the alternative is the other five shocks and triomes.
- White and blue are the thinnest colours (15 land sources each) for Niv-Mizzet, Talrand, Fact or Fiction and Austere Command, and Exotic Orchard depends on what the table's lands make.
- Whether the top end (Utvara Hellkite at eight, Summon: Bahamut at nine, Steel Hellkite, Niv-Mizzet) is too heavy now that Chrome Mox and Farseek are gone, and whether Young Pyromancer and Talrand earn their slots with seventeen instants and sorceries.
- Protection is thin after the swap (Fierce Guardianship, Teferi's Protection, Dragonlord Ojutai): the deck's known weakness is a board wipe after it has committed its Dragons.
- Whether the three tutors (Demonic, Vampiric, Worldly) make the Garth combo too consistent for the table.
