# Task: cand-glissa — model the Glissa Sunslayer candidate deck (and the B/G cards the Jenova deck shares)

Base branch: `rework-candidates`. Deck: `decklists/JD/candidates/glissa.md` (key `glissa`): Golgari fight club,
deathtouch creatures plus spells where your creature deals the damage. Glissa Sunslayer is already modeled. The deck
is wired into every deck list; your module is `rust/crates/sim-core/src/impls/glissa.rs` (stub in place), tests in
`rust/crates/sim-core/tests/cards_glissa.rs`. Several of your cards are also in the Jenova candidate deck: you model
them; the cand-jenova agent will not.

## Cards to model (until `cardgen audit --mine` shows each as Full)
Unmodeled: Bite Down, Burrog Barrage, Compel Brutality, Cost of Brilliance, Ferocity of the Hunt, Hard-Hitting
Question. Partial: Armored Scrapgorger, Canyon Crawler, Chelonian Tackle, Culling Ritual, Diplomatic Relations,
Foolish Fate, Forum Necroscribe, Hardbristle Bandit, Hunter's Axe, Knockout Maneuver, Lashwhip Predator, Lecturing
Scornmage, Mabel, Bitter Recluse, Melancholic Poet, Paladin of Predation, Tenured Concocter, Track Down, Vault
Plunderer, Witherbloom Charm, Writhing Necromass. Approximate: Bugenhagen, Wise Elder, Noxious Newt, Seymour Flux.

"Bite" (your creature deals damage equal to its power to their creature; deathtouch makes any bite lethal) and
"repartee" (whenever you cast an instant or sorcery that targets a creature) are mechanics several cards share:
build each once.

## AI plan for glissa
- Bite and fight with the best deathtouch creature when one is out (any bite kills); otherwise the biggest creature.
- Aim removal and bites at the biggest threat; Glissa attacks when her first strike plus deathtouch makes blocking
  bad, and picks "draw a card" from her trigger unless an enchantment or counters matter more.
- Ramp first, then threats; protect Glissa with Champion's Helm or Darksteel Plate.

Smoke run: `--deck glissa`.
