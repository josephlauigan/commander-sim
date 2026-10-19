# Godo, Bandit Warlord (key godo): findings

The deck's living list of strengths, weaknesses and what was tested. Godo plays Tier 6 (cEDH) only.

## 2026-10-19: baseline vs Tier 6 (Stage 7; 2,000 games per profile, look-ahead, two seeds)

**49.5 / 50.9% (seed 500000, conservative / loose); 49.8 / 49.3% (seed 900000).** The result repeats across seeds.
Raw: `2026-10-19-baseline/godo-vs-t6-s500000.txt`, `godo-vs-t6-s900000.txt`.

**Read this number with care.** It is far above Godo's real result (21.4% game wins, about 28% of decisive games, 78
entries on EDHTop16 over six months) and outside the 15-35% range that TARGETS-t6.md treats as plausible. The reason
is the table, not Godo: the Tier 6 decks still win at median round 7-13, while real cEDH games end around turns 4-6
(audit/t6/calibration-2026-10-19/REPORT.md). Godo kills at median round 6, so it out-races slow opponents. Use Godo
results against Tier 6 comparatively (swap A/B tests, both lists against the same table) until the table is faster.

How it wins (seed 500000, conservative): combo 56%, damage 44%. The "damage" wins are mostly the Godo + Helm of the
Host extra-combat loop played out by normal actions, not separate beatdowns. Lines: Hammer of Nazahn out + a token
copy spell on Godo 13%, Godo + Helm (Hammer out) 12%, Godo + Helm (equip {5}) 8%, Dualcaster and others the rest.
Median win round 6; it goes for a combo 0.39 times per game.

How it loses: Dack eliminates it in about 12% of games, Kinnan 10%, Kraum/Tymna and Rograkh/Silas about 8%, Sisay 7%,
Rograkh/Thrasios 3-5%. Its combo tries are stopped by Force of Will, Swan Song, Snap, Chain of Vapor, Mindbreak Trap,
Fierce Guardianship and Pact of Negation; about 40 stops per 2,000 games name no card (a cost or lock).

Mulligans: 39.5-41.5% of opening hands (spec keep: a hand needs a named way to win; ramp-only hands are shipped).

Known modeling notes: Staff of Compleation fixed 2026-10-19 (it now destroys only your permanents an opponent controls).
Not yet modeled per the spec: Squelcher before Godo, Pyroblast held for the cast, Magistrate/Agent and Blind Obedience
removal order, Chalice at 1, same-turn Ogre, Basalt + Kinnan copy.
