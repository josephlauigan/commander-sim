# Marchesa, the Black Rose — findings (living file)

What the simulator has taught us about this deck: strengths, weaknesses, and what was tried and why it did or didn't
work. Updated after every run. Newest evidence first; numbers are look-ahead, loose profile, vs Marchesa's matched
tiers (t3, t4: `data/my_decks.toml`) unless noted. The guide is `decklists/JD/marchesa-grixis-sacrifice-recursion.md`; the
plan file is `data/plans/marchesa.toml`. No game has been simulated with this list yet.

## Design 2026-11-26 (JD's choice, not simulated)
**Goal.** JD split the old Sheoldred list in two. The sacrifice and recursion half (Mikaeus, Triskelion, Yawgmoth, the
outlets and payoffs, the graveyard start) is the new Marchesa deck, a Grixis list with Marchesa, the Black Rose
({1}{U}{B}{R}) as commander; Sheoldred keeps the draw punisher and drain engine (`audit/sheoldred/FINDINGS.md`). The
design review (Opus, 2026-11-26) asked for redundant two-card wins and named the engine risks listed below.

**Packages (99 cards plus the commander: 64 spells and 35 lands).**
| Package | Slots | Cards | Job |
|---|---|---|---|
| Mana | 9 | Sol Ring, Arcane Signet, Dimir, Izzet and Rakdos Signets, Talismans of Creativity, Dominance and Indulgence, Jet Medallion | Eight mana rocks and a cost reducer |
| Sacrifice outlets | 6 (+ Phyrexian Tower, a land) | Viscera Seer, Carrion Feeder, Woe Strider, Ashnod's Altar, Phyrexian Altar, Altar of Dementia | Free, repeatable sacrifice |
| Death payoffs | 7 | Blood Artist, Zulaport Cutthroat, Bastion of Remembrance, Syr Konrad, Mayhem Devil, Sephiroth, Grave Pact | Drain, ping or edict on each death |
| Undying, counters, combo pieces | 8 | Mikaeus, Triskelion, Yawgmoth, Nim Deathmantle, The Ozolith, Walking Ballista, Vampire Socialite, Gravecrawler | The loops and the counter makers (Mikaeus's undying, Triskelion and Ballista entering with counters, The Ozolith, Vampire Socialite) |
| Enters-the-battlefield creatures | 9 | Accursed Marauder, Plaguecrafter, Gonti, Mulldrifter, Grave Titan, Phyrexian Delver, Perigee Beckoner, Priest of Gix, Sauron | Value that returning creatures replay |
| Reanimation | 3 | Entomb, Reanimate, Animate Dead | The graveyard start |
| Tutors | 4 | Demonic Tutor, Diabolic Tutor, Grim Tutor, Vampiric Tutor | Find a missing loop piece |
| Interaction | 12 | Bitter Triumph, Fatal Push, Lightning Bolt, Terminate, Bedevil, Chaos Warp, Blasphemous Act, Counterspell, Arcane Denial, Swan Song, Mindbreak Trap, Propaganda | Removal, one wipe, four counters, an attack tax |
| Draw and finisher | 5 | Fact or Fiction, Deadly Dispute, Skullclamp, Black Market Connections, Bolas's Citadel | Refill; Citadel plays off the top for life and is a ten-permanent finisher |
| Protection | 1 | Lightning Greaves | Shroud and haste |
By type (the guide's list): creatures 23, enchantments 5, artifacts 17, sorceries 5, instants 14, lands 35, commander 1.
Average mana value of the 64 spells and the commander: 2.74 (X spells at X = 0).

**Win lines (all in the engine's combo catalogue unless noted).** Mikaeus + Triskelion (`mikaeus_trisk.rs`); Yawgmoth +
Mikaeus + two nontoken non-Human creatures + a death payoff (`combos.rs`); Nim Deathmantle + Ashnod's Altar + Priest
of Gix or Grave Titan (`mine.rs` LOOPS); Gravecrawler + Phyrexian Altar + a Zombie + a payoff; Sauron's attack
replaying an enters trigger; Entomb into Reanimate; the aristocrats drain. Dethrone combat is a bonus, not a plan.

**Game Changers: 3** (Bolas's Citadel, Demonic Tutor, Vampiric Tutor). The infinite loops make it Bracket 4.

**Mana base.** 35 lands: Badlands, Barad-dûr, Blood Crypt, Bloodstained Mire, Command Tower, Crumbling Necropolis, Dragonskull
Summit, Drowned Catacomb, Haunted Ridge, Luxury Suite, Morphic Pool, Path of Ancestry, Phyrexian Tower, Polluted Delta,
Scalding Tarn, Spirebluff Canal, Steam Vents, Sulfur Falls, Underground Sea, Volcanic Island, Watery Grave, 3 Island,
2 Mountain, 9 Swamp. Land sources: 24 make {B} (25 counting Phyrexian Tower's {B}{B}), 16 make {U}, 16 make {R}; the
rocks add five sources of each colour (Signets, Talismans and Arcane Signet), so about 29 / 21 / 21 with them. Counting rule: Command
Tower and Path of Ancestry make any colour; the fetchlands count for the colours of the land types they fetch (Bloodstained
Mire: Swamp or Mountain; Polluted Delta: Island or Swamp; Scalding Tarn: Island or Mountain). The hard costs are
Mikaeus, Bolas's Citadel and Grave Pact ({B}{B}{B}), Counterspell and Mindbreak Trap ({U}{U}) and Bedevil ({B}{B}{R}).

**Models below Full.** Grave Pact is Full-auto (compiled from Oracle text, not hand-verified); every other card is Full
(`cardgen find --full` for each of the 86 non-basic cards; `cardgen audit --mine` reports 0 not Full for 89 unique
cards, counting Full-auto as Full). Grave Pact goes to JD as a below-Full card with the swap list.

**Ownership** (`data/ownership.toml`, `[deck.marchesa]`, not confirmed by JD): 25 `have` and 60 `proxy` of the 85
non-basic cards; Marchesa herself is a proxy. A deck without `confirmed` is unconfirmed: treat the statuses as unknown
until JD marks the list.

**Simulator limits (known before the first run).**
- **No dethrone targeting.** `dethrone` appears nowhere in `rust/crates/sim-core/src/ai/`. The counters work
  mechanically (`cardcode.rs`), but no attack logic picks the player with the most life, so the engine runs on counters from
  Mikaeus's undying, Carrion Feeder, Triskelion, Walking Ballista and Vampire Socialite, not from combat.
- **No Marchesa hand AI.** The old Marchesa deck AI was removed (c02b10e6, 2026-10-07). The deck plays the generic
  policy plus `data/plans/marchesa.toml` (priorities 90 for the commander, 60 for the outlets and payoffs). Only
  `marchesa_sac_worth` and `card_etb_value` survive for sacrifice choices. Expect plan-file work, and possibly AI work for
  sacrificing a countered creature before the end step, Blasphemous Act timing and Nim Deathmantle payments.
- **Commander return.** Engine PR #408 makes Marchesa's end-step return (and Kaya's Ghostform's) take a commander only
  from the graveyard: a card put in the command zone is a new object and cannot be returned, and the engine now leaves a
  commander in the graveyard while her own return is pending. Known limits of that PR: a graveyard-exile effect (Bojuka
  Bog and the like) before the end step strands the commander in exile with no command-zone choice, and a commander that
  returns and dies again before the same end step can be returned again.
- **Yawgmoth loop readiness.** Engine PR #408 makes the check need two nontoken undying creatures, as the rules do
  (Mikaeus gives himself no undying and Yawgmoth is a Human); the earlier check was looser. Any combo number depends on
  the engine that includes it.
- **Two Spellbook combos have no engine line** (listed in `KNOWN_GAPS` in `rust/crates/sim-core/tests/spellbook.rs`): Walking Ballista + Mikaeus (628-3693) and Priest of Gix + Nim Deathmantle + Phyrexian Altar (586-4050-5003). The AI does not recognise them as wins. Walking Ballista + Mikaeus is a real kill line of this deck (Ballista pings itself, dies with no +1/+1 counter, undying returns it with one; with Blood Artist, Zulaport Cutthroat, Syr Konrad or Bastion of Remembrance out it kills the table), so the simulator undercounts the deck: a limit for JD to accept, or a ticket (one catalogue entry shaped like the Gravecrawler + Phyrexian Altar line, plus a check that the Ballista has a counter or can be cast for X of 1 or more). Priest of Gix + Nim Deathmantle + Phyrexian Altar is mana-neutral (the Altar's 1 and the Priest's BBB pay Nim's {4}) and needs a payoff, so it cannot be folded into the Ashnod's Altar entry; the Nim + Ashnod's Altar + Priest of Gix loop is modelled, the Phyrexian Altar variant is not (the same family as the Nether Traitor + Phyrexian Altar gaps already listed for another deck).
- **Murderous Redcap, Xander's Lounge and Scrawling Crawler have no engine record**, and the catalogue has no Redcap +
  Mikaeus + outlet loop; none is in the list.

**Not measured.** Everything: no win rate, no combo rate, no plan timing, no cast rates. The design is a proposal from card
text and the engine's catalogue.

**To check in the first run (smoke of about 100 games, then the baseline, t3 and t4, loose).**
- `marchesa_returns`, dethrone counters and which combos fire (the catalogue's names); a smoke run reads the logs
  before the 1,500-game baseline.
- Cards never cast or untagged (that is what hid Torment of Hailfire and Call of the Ring in the Sheoldred deck): the
  card report's "never cast by the sim" list.
- The win rate against a stated target band (for example Sheoldred's 30% plus or minus 3 over t3, t4), the share of wins by
  combo, drain and damage, and Marchesa's own cast turn and removal rate.
- Whether the Nim Deathmantle line, the Gravecrawler line and the Sauron attack are played by the AI, and what Entomb finds
  (the generic reanimation AI bins loop pieces with Entomb: Triskelion and Mikaeus).
- How often the colour sources cost a turn: Marchesa needs {U}{B}{R}, Mikaeus {B}{B}{B}, Counterspell {U}{U}.
- The tiers' hard opponents: Tergrid (it takes each nontoken permanent the deck sacrifices), Krenko and Aurelia
  (combat), Chulane and Prosper (combo), and graveyard hate.
