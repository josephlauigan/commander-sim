# Card-pile swap packages (2026-11-06, not yet run)

JD's pile of 35 spare cards (many freed by retiring Kefka and Sauron), plus commander-sim-82's Aragorn sweeper
recommendations, built into one package per deck. JD approved these on 2026-11-06; the runs were stopped before any
finished, so **there are no results yet**. Base: origin/main `d1763b2d` (PRs #290 and #291 merged).

Checks done: every card going in is Commander-legal and fits its deck's colour identity; every card coming out is in
that deck's list in `data/decks.json` at `d1763b2d`. Preflight (`--preflight-only`) passed for all of them: every card
is modelled Full and none was stopped by the dry run.

## The packages

Each deck runs only against its own tiers (`--pool all`), loose profile, explanations on (the defaults). Each command
runs under the memory cap: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <command>`.

| Run | Deck (tiers) | Out => In |
|---|---|---|
| P1 | aragorn (t4-t5) | Big Score => Windborn Muse; Mystic Confluence => Disdainful Stroke; Comet Storm => Spark Double; Island => Mistrise Village; Mountain => Sokenzan, Crucible of Defiance; Blossoming Defense => Nibelheim Aflame |
| P2 | aragorn | as P1, but Blossoming Defense => Austere Command |
| P3 | aragorn | as P1, but Blossoming Defense => Chandra's Ignition |
| P4 | thanos (t5) | Urza's Bauble => Crucible of Worlds; Wastes => Strip Mine; Oblivion Stone => Elesh Norn, Mother of Machines; Wayfarer's Bauble => Spark Double; Expedition Map => Paradise Druid; Ugin, the Spirit Dragon => Summon: Bahamut; Decimate => Meteor Golem; Ratchet Bomb => Hangarback Walker |
| P5 | sheoldred (t3-t4) | Raise Dead => Reanimate; Cast Away Doubt => Entomb; Dark Deal => Yawgmoth's Will; Vanquish the Weak => Massacre Wurm; Diabolic Tutor => Diabolic Intent; Otherworld Atlas => Hangarback Walker; Howling Mine => Palantír of Orthanc |
| P6 | yshtola (t5) | Memory Trap => Counterspell; Ill-Gotten Inheritance => Windborn Muse; Arrest => Meteor Golem; Read the Bones => Fact or Fiction; Dark Petition => Yawgmoth's Will |
| P7 | cloud (t1-t3) | Disenchant => Sting, the Glinting Dagger; Loyal Warhound => Hangarback Walker |
| P8 | thrasios (t6) | Mockingbird => Llanowar Elves; Snapback => Counterspell |

The three Aragorn variants share the Blossoming Defense cut so the sweepers compare with each other (commander-sim-82
found Krenko, Edgar Markov and Yuriko cause about 52% of Aragorn's t4 eliminations). Vandalblast was left out: it only
hits artifacts. Lotus Petal, the rituals and Arcane Signet stay because they feed the Breach line and the mana.

Other choices JD made: Ugin rather than Nicol Bolas, the Ravager is cut for Bahamut. Raise Dead stays as a slot
(JD kept it over Necrosquito on 11/04), so it becomes Reanimate and Yawgmoth's Will takes Dark Deal's slot.

Example: `commander-sim --deck sheoldred --pool all --swap "Raise Dead=>Reanimate" --swap "Cast Away Doubt=>Entomb" ...`
(one `--swap "Out=>In"` per pair). Expected time is about 13 minutes per two-tier package, about 1.5 hours for all eight.

## Watch in the results

- The AI casts some cards rarely (dry run, share of the games they were drawn): Yawgmoth's Will 9-15% (priority 0),
  Entomb 19%, Massacre Wurm 20-26%. Low numbers for these may reflect the AI rather than the card.
- Hangarback Walker in Cloud and Reanimate in Sheoldred also got the "priority 0" AI warning, though the dry runs cast
  them about half the time or more.
- Sting: the model covers the combat untap and the Goblin/Orc first strike. The Scryfall-text check listed those as
  "not modeled", so confirm from Sting's numbers.

## One copy each

JD owns one copy of each, so the packages compare homes: Spark Double (P1-P3 or P4), Windborn Muse (P1-P3 or P6),
Meteor Golem (P4 or P6), Hangarback Walker (P4, P5 or P7), Counterspell (P6 or P8), Yawgmoth's Will (P5 or P6).
