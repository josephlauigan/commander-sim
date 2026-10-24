# Task: titania-cards — model the Titania deck's last 8 cards and write its AI plan

Read `rust/agents/OVERNIGHT-RULES.md` first: its hard rules (never run Python, memory cap, edit only your files,
commit often, handoff, 15-line report) apply here. Where it says "Kefka/Zada", read "Titania".

Base branch: `bill-deck` (3c1830e). Deck: `decklists/JD/titania-lands.md` (key `titania`): mono-green, lands going to
the graveyard. Titania, Protector of Argoth: when she enters, return a land card from your graveyard to the
battlefield; whenever a land you control is put into a graveyard from the battlefield, create a 5/3 green Elemental.
Your module: `impls/titania.rs` (stub in place), tests `tests/cards_titania.rs`. Smoke run: `--deck titania`.
The Bill deck (`impls/bill.rs`, `rust/agents/report-bill-cards.md`) shares many cards and has machinery to reuse.

## Cards to model (until `cardgen audit --mine` shows `titania: ... 0 not Full`)
Unmodeled: Follow the Lumarets (infusion). Partial: Annoyed Altisaur (cascade: check for existing cascade machinery
first), Arachnogenesis (X 1/2 reach Spiders for the creatures attacking you, prevent combat damage from non-Spiders:
an instant cast in the declare-attackers/blockers window when you are attacked; find how fogs are modeled), Glorious
Decay (the 4-damage-to-a-flier mode), Infestation Expert (daybound/nightbound: if day/night isn't modeled, say how the
transform matters here and model what can matter), Scout the City, Something Worth Saving (mill then take a permanent),
Malevolent Rumble (Approximate: make it exact).

**Check Titania herself first** (the audit says Full): her graveyard trigger must fire when Evolving Wilds,
Terramorphic Expanse and Fabled Passage are sacrificed, and when a land dies any other way; her enter trigger must
return a land from the graveyard. If either is wrong, fix it and say so.

## AI plan for titania (in `impls/titania.rs`, through `titania_prio` and the deck-options dispatch)
- Cast Titania as early as the mana allows (5 mana); ramp first.
- **Fetch lands:** while Titania is out, crack them at once (a 5/3 each). While she isn't but can be cast within a turn
  or two, hold an uncracked fetch land on the battlefield (play it, don't crack it) so it makes a 5/3 later; with no
  Titania in sight, crack for mana as normal. Gate any change to `engine/turn.rs` land choice or other shared code to
  `key == "titania"`.
- **Her enter trigger:** return a fetch land from the graveyard if there is one (it can be cracked again).
- Graveyard fillers (Satyr Wayfinder, Malevolent Rumble, Something Worth Saving, Scout the City) before Titania, so her
  enter trigger has a fetch land to return.
- Blossoming Defense and Royal Treatment in response to removal on Titania; Arachnogenesis when an attack would deal
  a lot (a third of your life or more) or kill you.
- Garruk's Uprising before the 5/3s when possible; Overwhelming Stampede / End-Raze Forerunners when the attack is
  lethal through the blockers (Bill's `damage_through` in `impls/bill.rs` already counts blockers: reuse it).

## Done when
The OVERNIGHT-RULES checks all pass (full `cargo test --release`, no warnings, `all_decks adaptive 3` and
`lookahead 1`, audit `titania: 0 not Full`, a 48-game smoke run with `--deck titania`). Report as those rules say:
line 1 DONE or HANDOFF, line 2 your model ID.
