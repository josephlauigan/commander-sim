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
Triple Triad. The plan file now has a priority row for Triple Triad (62 from six lands and mana permanents, no else), because
Sauron's 11/01 findings record that the generic AI once never cast it. Still to check in the first run: Triple Triad's cast and
stuck-in-hand numbers, and what it flips.

## What the simulator cannot do yet

- **Plan file:** `data/plans/urdragon.toml` (written 2026-11-14, provisional: from the pilot guide, not a paired test). Its header lists what a plan file cannot say.
- **Card-code fixes (tests in `tests/ai_urdragon_garth_line.rs`):** The Ur-Dragon's free drop no longer puts Tiamat onto the battlefield (her search is "if you cast it"; she stays in hand to be cast). Twinflame is held while the Garth line is alive (Garth, Spark Double and Doubling Season each in hand, library or on the battlefield), so the combo is its only play once the line is ready. The hold is released when a piece is dead, when the table's visible attack can kill the seat, or when Twinflame's copies and the ready fliers are lethal on one opponent in the first main phase. That lethal estimate counts what the resolution really makes (Doubling Season's extra copies, bounded by the token cap; no copy of a creature with protection from red), the damage that can reach the opponent (combat damage only if the opponent has no untapped flier or reach blocker, no attack cap is out anywhere and the opponent's attacks are not taxed; creatures locked by Arrest-style Auras, creatures whose damage is prevented and, in a fog turn, all combat damage are left out), and Terror of the Peaks' triggers, replayed one at a time the way the engine aims them (an opponent the trigger can kill takes it first, else a creature the engine would remove, else the most threatening opponent, with each opponent's life updated after every trigger). An opponent is counted only when nothing reduces or stops the damage: The One Ring protection, Glacial Chasm, a locked life total, Gisela's halving, Angel's Grace, Phyrexian Unlife, Herald of Eternal Dawn, Serra's emblem and Pariah all make it uncountable; a hexproof opponent takes no Terror damage in the count. Spark Double copies Garth when Doubling Season (in hand or on the battlefield) and Twinflame (in hand only: a Twinflame on the battlefield is spent) are there (or a Demonic or Vampiric Tutor can fetch the missing one), else the best creature as before; only the seat with the key `urdragon` takes the Garth branches. Two corrections to the fair Twinflame itself apply to every seat (it is in no other registered deck): its copies are doubled by Doubling Season (one `repl::tokens` count per target, up to the token cap), and a creature with protection from red is skipped as a target. Not simulated yet: the effect on the kill turn and the share of wins by the Garth route is for the first goldfish and tier-5 runs.
- **Still open:** a Spark Double cast, or free-dropped by The Ur-Dragon, before Garth is on the battlefield copies something else and kills the line; holding Spark Double until Garth lands is a separate change to the cast logic. Twinflame also stays held for as long as the pieces are in the library, which is most of the game: whether that hold should last that long is an open policy question, not a measured result. A permanent that locks the line (an opponent's Rule of Law) does not release the hold either: it is zone-only.
- **Limits of the lethal release (recorded behaviour, not simulated):**
  - The estimate counts each Terror of the Peaks trigger from the Terrors already on the battlefield. When Twinflame copies a Terror and Doubling Season makes two tokens, the second token's entry also triggers the first token, which the estimate leaves out. The error only delays the release (Twinflame held when the copies would already be lethal), never spends it early.
  - The estimate reads only what is visible when Twinflame is cast. A fog, Teferi's Protection, Angel's Grace or instant-speed removal an opponent casts after it, during combat, can still stop the kill; that is true of any attack the AI makes and is not modelled for this release. The same goes for the order Terror's triggers resolve in when an opponent can answer them (the replay assumes the order they were put on).
  - The release does not count combat damage to an opponent whose attacks are taxed (Propaganda, Ghostly Prison), or while any attack cap is out (Angelic Arbiter, Sandwurm Convergence, Crawlspace, Silent Arbiter). It also does not treat as killable an opponent whose damage is halved, floored or not lethal (Gisela, Angel's Grace, Phyrexian Unlife, the Serra emblem) or whose life can't change. In those states, or while an opponent's Blind Obedience-style effect makes the copies enter tapped (Blind Obedience, Authority of the Consuls: a tapped copy can't attack, but its entry still triggers Terror), Twinflame stays held unless Terror's triggers alone are lethal. The AI does not keep mana back for an attack tax, so the mana left after Twinflame is not counted on.
  - The release assumes the AI then attacks the opponent the estimate found lethal with its fliers and the copies. The AI's defender choice does not count the copies (hasty but summoning sick) toward its kill bonus, so at a four-player table it can attack someone else. This is an open question, not a measured defect.
  - The copies' power is the printed power. An opponent's static power reduction would make the estimate too high; no card in the tier lists does that.
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
