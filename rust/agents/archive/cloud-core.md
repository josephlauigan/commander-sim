# Task: cloud-core — Cloud, the Equipment, and the Cloud deck's AI plan

Read `rust/agents/LASTDECK-RULES.md` first. Deck: `decklists/JD/cloud-midgar-equipment.md` (key `cloud`), mono-white
Equipment. Your module: `impls/cloud.rs` (stub in place: `cloud_prio`, `cloud_special`, `cloud_protect` are already
wired into the dispatch), tests `tests/cards_cloud.rs`. You own: Equipment (attaching for free, equip cost changes),
the trigger doubling, landwalk, job select.

**Cloud, Midgar Mercenary** ({W}{W}, 2/1): *When Cloud enters, search your library for an Equipment card, reveal it,
put it into your hand, then shuffle. As long as Cloud is equipped, if a triggered ability of Cloud or an Equipment
attached to it triggers, that ability triggers an additional time.* The doubling covers Cloud's own triggers
(including abilities an Equipment grants him: Medic's Kitesail, White Mage's Staff, Conformer Shuriken), and triggers
of the Equipment attached to him (Mask of Memory, Sword of Kaldra, Explorer's Scope, Hero's Blade). Find where
triggers are dispatched and add the doubling there with the least change; test it with Mask of Memory (draw four,
discard two) and Conformer Shuriken (two taps). The enter trigger isn't doubled unless he is already equipped as he
enters. The Kaldra pieces are already Full (from Jodah): check they work on Cloud and that Helm of Kaldra's Kaldra
token still works.

## Cards to model (until `cardgen audit --mine` shows each Full)
Cloud, Midgar Mercenary; Hero's Blade (attach itself when a legendary creature you control enters); Armory Automaton
(attach any number of Equipment on enter and attack); Beatrix, Loyal General (at the start of your combat, attach any
number of Equipment to target creature you control); Puresteel Paladin (draw on Equipment entering; metalcraft equip
{0}); Cid, Freeflier Pilot (Equipment and Vehicle spells cost {1} less; flying on your turn; return Equipment or
Vehicle from graveyard); Helitrooper (flying to another attacker; equip costs targeting it {2} less); Colossus Hammer
(+10/+10, loses flying, equip {8}); Hulkbuster Armor (base 9/9 and flying; equip Hero {3}); Wrecking Ball Arm (base
7/7, can't be blocked by power 2 or less; equip legendary {3}); Conformer Shuriken; Explorer's Scope; Magitek Scythe
(attach on enter, first strike and must be blocked); Squire's Lightblade (flash, attach on enter, first strike);
White Mage's Staff (job select: a 1/1 Hero token wearing it); Trailblazer's Boots (nonbasic landwalk; both decks);
Magnetic Snuffler (return an Equipment from the graveyard attached; counters on artifact sacrifice); Jocasta,
Automaton Avenger (counter on commander combat damage; returns attacking when your commander attacks); Zack Fair (a
+1/+1 counter; sacrifice: indestructible, move his counters and Equipment); Unfinished Business (a creature back,
then up to two Auras and/or Equipment attached to it).

## AI plan for cloud (in `impls/cloud.rs`, through `cloud_prio`, `cloud_special`, `cloud_protect`)
- Cloud on turn 2; his search picks by need: no Equipment out → Hero's Blade or Wrecking Ball Arm; one out → Sword
  of Kaldra or Mask of Memory; Cloud threatened → Shield of Kaldra; the two other Kaldra pieces out → the third.
- Equip Cloud every turn with the best affordable piece (Puresteel's {0}, Wrecking Ball Arm's {3}, Veteran's
  Powerblade's {W} for a Soldier); attack with an equipped Cloud when he survives or trades up; commander damage is
  the main plan.
- Re-buy his search: Ennis, Forum Familiar, Unyielding Gatekeeper, Stormfront Riders and Essence Reliquary target
  Cloud when the Equipment found is worth more than what falls off (the white-rest agent models those cards; you
  choose the targets through the deck plan).
- Protect Cloud: Bastion Protector, Shield of Kaldra, Zack Fair, Patriot, and the protection instants in response to
  removal (`cloud_protect`).

## Done when
The LASTDECK-RULES checks all pass, and every card above is Full. Report as those rules say; full report in
`rust/agents/report-cloud-core.md`.
