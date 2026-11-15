# The Ur-Dragon (key `urdragon`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/urdragon-five-colour-dragons.md`, built 2026-11-13: an optimised high-bracket-4 Dragons list with
proxies allowed, reworked 2026-11-14 (28 Dragons besides the commander, 29 with Morophon, ten original duals, seven fetches, five shocks; the Garth One-Eye + Spark Double + Twinflame +
Doubling Season combo). **Tiers:** t5 (`data/my_decks.toml`). **No sims yet.** The findings of the
retired Ur-Dragon decks are in `audit/retired/urdragon/`.

## 2026-11-15: Talrand, Sky Summoner => Triple Triad (JD's choice, not simulated)
Triple Triad ({3}{R}{R}{R}, model status Full: `data/card_overrides.toml`, tests in
`rust/crates/sim-core/tests/cards_seph_triad.rs`) takes Talrand's slot. Why: at your upkeep each player exiles their top
card and you play your own card and every other exiled card of lower mana value free. About a third of the 99 are lands (no
free spell, but you may play one as your land drop) and 29 are Dragons, mostly costing 5 to 7, so a flip is a free Dragon
about three times in ten, and the higher your flip, the more of the opponents' exiled cards qualify. Talrand triggers on only seventeen instants and sorceries, five of them early tutors and
ramp, his Drakes are not Dragons (no Scourge of Valkas, Dragon Tempest, Ganax, Utvara or Ur-Dragon draw), and he costs
{U}{U} on the thinnest colour. What it costs: a six-mana enchantment that needs {R}{R}{R} (20 red sources) and does nothing
the turn it lands, and it does not help against a wipe. Not measured. An Opus consult ranked Heroic Intervention first for
this slot (it answers the documented weakness, a wipe after the Dragons are committed; JD had cut it on 11/14); JD chose
Triple Triad. The plan file has no priority row for Triple Triad, so how the generic AI casts it here is untested (Sauron's
11/01 findings record that the generic AI once never cast it). To check in the first run: Triple Triad's cast and
stuck-in-hand numbers, and what it flips.

## What the simulator cannot do yet

- **Plan file:** `data/plans/urdragon.toml` (written 2026-11-14, provisional: from the pilot guide, not a paired test). Its header lists what a plan file cannot say and three card-code issues pending a decision (Tiamat's free drop, Twinflame, Spark Double).
- **Cards modelled before:** most Dragons were modelled for the retired pool deck (`impls/t45_urdragon.rs`) and the Garth combo for the retired JD deck. Not re-checked for this list. Klauth and Rivaz
  are Full for the AI and a person (spell-only turn-long mana; a graveyard cast once per turn). Relic of Sauron (two mana
  in blue, black or red, and a {3} loot) and Smaug the Impenetrable are Full; a person picks each colour Relic makes.
  Niv-Mizzet and Utvara Hellkite are compiled from Oracle text and covered by `tests/cards_urdragon_swaps.rs` (Utvara
  triggers only for Dragons that attack). The other new cards are Full. Nature's Lore and Three Visits fetch any Forest
  card (a person picks; the AI takes the best dual) and follow the land's own enters rule.

## Open questions

- Ten original duals are the expensive part of the mana base; the alternative is the other five shocks and triomes.
- White and blue are the thinnest colours (15 land sources each) for Niv-Mizzet, Fact or Fiction and Austere Command, and Exotic Orchard depends on what the table's lands make.
- Whether the top end (Utvara Hellkite at eight, Summon: Bahamut at nine, Steel Hellkite, Niv-Mizzet) is too heavy now that Chrome Mox and Farseek are gone, and whether Young Pyromancer earns his slot with seventeen instants and sorceries.
- Protection is thin after the swap (Fierce Guardianship, Teferi's Protection, Dragonlord Ojutai): the deck's known weakness is a board wipe after it has committed its Dragons.
- Whether the three tutors (Demonic, Vampiric, Worldly) make the Garth combo too consistent for the table.
