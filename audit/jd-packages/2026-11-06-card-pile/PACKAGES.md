# Card-pile swap packages (2026-11-06)

JD's pile of spare cards (bulk, a Dragon package and a Ring package, many freed by retiring Kefka and Sauron), plus
commander-sim-82's Aragorn sweeper recommendations, built into packages for five decks (Tymna/Thrasios not looked at).
JD's rule for cuts: take from proxy or Don't Have cards; take a Have card only for a true upgrade. Approved by JD on
2026-11-06 and run on main `b84ab9e6` (PR #292's counterspell AI included). Results go to each deck's
`audit/<deck>/FINDINGS.md`. This replaces the earlier P1-P8 draft, which never ran.

Checks: every card going in is Commander-legal and fits its deck's colour identity; every card coming out is in that
deck's list in `data/decks.json` at `b84ab9e6`. Each card going in is a separate physical copy from the pile.

The machine-readable list is `packages.tsv` (id, deck, `Out=>In` pairs split by `|`). Each package runs as
`commander-sim --deck <deck> --pool all --swap "Out=>In" ...` under the memory cap (own tiers, loose, explain on).

| Run | Deck (tiers) | Out => In (status of the card out) |
|---|---|---|
| TD | thanos (t5) | Wayfarer's Bauble => Scion of Draco (proxy); Urza's Bauble => Crucible of Worlds (proxy); Wastes => Strip Mine (proxy); Red Elemental Blast => Disdainful Stroke (proxy); Oblivion Stone => Elesh Norn, Mother of Machines (proxy); Ugin, the Spirit Dragon => Summon: Bahamut (proxy); Ratchet Bomb => Hangarback Walker (proxy); Decimate => Meteor Golem (Have); Duplicant => Glorybringer (proxy); Gamble => Thundermaw Hellkite (proxy); Lotus Petal => Mox Jasper (proxy); Karn, the Great Creator => Utvara Hellkite (proxy) |
| A1 | aragorn (t4-t5) | Big Score => Austere Command (Have); Blossoming Defense => Nibelheim Aflame (Have); Grand Abolisher => Windborn Muse (proxy); Mountain => Sokenzan, Crucible of Defiance (basic) |
| A2 | aragorn (t4-t5) | Noble Hierarch => Paradise Druid (Don't Have); Three Visits => Spark Double (Don't Have); Growth Spiral => Llanowar Elves (Don't Have) |
| S | sheoldred (t3-t4) | Raise Dead => Reanimate; Cast Away Doubt => Entomb; Diabolic Tutor => Diabolic Intent; Vanquish the Weak => Massacre Wurm; Dark Deal => Yawgmoth's Will; Otherworld Atlas => Dreadhorde Invasion (all Have); Swamp => Barad-dûr (basic) |
| Y | yshtola (t5) | Memory Trap => Counterspell; Dark Petition => Ringsight; Ill-Gotten Inheritance => Palantír of Orthanc (all Have); Island => Mistrise Village (basic) |
| C | cloud (t1-t3) | Plains => Rogue's Passage (basic); Disenchant => Sting, the Glinting Dagger (Have) |

Outside the tests: the bulk Blood Crypt replaces Thanos's proxy Blood Crypt (same card).

Why these packages:
- **Thanos** (20.0% at t5; wins 50% by damage, 46% by combo; dies around turn 8 to Kinnan, Miirym, Zur, Urza and
  Yawgmoth): cheap speed and consistency (Scion of Draco costs about {2} with the duals' land types, Crucible + Strip
  Mine with Urza's Saga, the fetches and Wrenn and Six), interaction, and hasty Dragons for the Terror of the Peaks /
  The Ur-Dragon fair win.
- **Aragorn** (9.9% over t4-t5): A1 answers t4's go-wide decks (Krenko, Edgar Markov, Yuriko, about 52% of its t4
  losses; commander-sim-82's analysis); A2 swaps Don't Have ramp for cards JD owns, and adds Spark Double (copies
  Veyran) in place of Three Visits, so A2 has one ramp card fewer.
- **Sheoldred**: only its combo core, finishers and tutors are proxies, so every cut is a Have card replaced by a true
  upgrade (Reanimate for Raise Dead in the same slot; JD kept Raise Dead over Necrosquito on 11/04).
- **Y'shtola**: noncreature spells of mana value 3+ trigger her drain (Ringsight, Palantír); Counterspell answers the
  Kinnan and Urza combos she loses to. Exquisite Blood is a proxy in her list (it isn't in JD's paste of the deck).
- **Cloud**: Rogue's Passage makes an equipped Cloud unblockable; Sting at JD's request.

Left out (legal, but no slot or off-plan): Fact or Fiction, Glimpse the Unthinkable, Godo, Grave Titan, Inferno Titan,
Kefka, Kuja, Marvin, Chandra's Ignition, Sakura-Tribe Elder, Haunted Ridge, Sulfurous Springs, Underground River,
Corroding Dragonstorm, Dragonlord Ojutai, Kindred Dominance, Kura, Niv-Mizzet the Firemind, Call of the Ring, Mauhúr,
Sauron.

## Outcome (2026-11-07)

| Run | Result | Adopted |
|---|---|---|
| TD | WORSE: -2.8 at t5 (noise ±2.7); raised the curve 3.10 -> 3.49 | No |
| TD2 | Not run | Yes (JD), untested |
| A1 | BETTER: t4 +4.3, t5 +2.4 (outside noise) | Yes (PR #293) |
| A2 | No clear effect | Yes (PR #293) |
| S | No clear effect: t3 +1.9, t4 -0.3 (noise ±2.7); Entomb and Yawgmoth's Will rarely cast | Yes, with Diabolic Tutor kept and Darksteel Plate cut for Diabolic Intent (untested change) |
| Y | Not run | Yes (JD), untested; Memory Trap turned out to be removal, not a counter (see audit/yshtola/FINDINGS.md) |
| C | Stopped part-way, no result | Yes (JD), untested |

Raw outputs of the runs that finished: `runs/`. Each deck's FINDINGS has the reasons.
