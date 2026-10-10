# Tests

Run every test from the repository root:

```
python3 -m unittest discover -s tests -t .
```

That's 693 tests in about twenty minutes (the look-ahead tests are the slow ones). To run one file, or one test:

```
python3 -m unittest tests.test_my_cards
python3 -m unittest tests.test_my_cards.Sephiroth.test_massacre_wurm
```

The tests need no network access. The card data they use is in `data/scryfall_cache.json`.

## What each file covers

| File | Tests | What it checks |
|---|---|---|
| `test_rules.py` | 34 | Core rules on hand-built positions: mana colours and payment order, Sol Ring, Talisman pain, commander tax, the state-based losses (life, 21 commander damage, empty library, 10 poison), what each counterspell can hit and what it costs, Swords / Path / Bolt / token removal, Toxic Deluge, tutors, mulligans, commander damage, flying, deathtouch, lifelink, and Teferi's Protection (the life lock and protection, phasing, when it ends, which wins it survives, when the AI casts it). |
| `test_my_cards.py` | 129 | Key cards of your four decks against their Oracle text, and Sephiroth's four loops (Mikaeus + Triskelion, Mikaeus or Melira + Kitchen Finks, Nim Deathmantle + Ashnod's Altar + Grave Titan): when each kills, what stops it, and how the AI finds and assembles the pieces. Also Kefka, Brush Off, Melira, Avacyn's Pilgrim, Unsummon, Twinflame, Champion's Helm, Slaughter Pact, Urabrask, the Underworld Breach line (Brain Freeze and Grapeshot with storm; Lotus Petal, Lion's Eye Diamond, Birgi and Jeska's Will paying for it; Laboratory Maniac milling yourself out), and the Sauron AI's priorities for Sheoldred and Consecrated Sphinx. Summon: Bahamut, tested for Sephiroth: each chapter (I and II on the biggest threat, II after your draw step, III's draw, IV's Mega Flare and the sacrifice), a lethal Mega Flare, reanimated and Entombed as a bomb, a blink starting it over at chapter I (Ephemerate, Displacer Kitten; not before a lethal Mega Flare), and hard-casting only with nine mana. The flicker package tested for Sephiroth: Soulherder, Conjurer's Closet and Teleportation Circle at your end step (Teleportation Circle untapping a tapped mana rock when no creature is worth it) (the commander Atraxa again, Bahamut restarted for another destroy but not before a strong Mega Flare, counters kept over a small enter effect), Restoration Angel (saving a bomb from removal, its best non-Angel target, held for the end of an opponent's turn), and Karn's Bastion proliferating Bahamut's lore and the loops' counters. Marchesa, the Black Rose (now in Jodah's 99): her return trigger (wipes, stolen creatures, exile), dethrone, Accursed Marauder, Nim Deathmantle leaving her returns alone; the other cards first written for her deck that other lists still run (Tezzeret's Gambit, Last Gasp, Notion Thief, Coalition Relic). Marchesa with Sauron's Army: dethrone on the Army (with Mauhúr's extra counter) and a returning creature. |
| `test_game_changers.py` | 24 | The Game Changer audit's fixes (audit/GAME_CHANGERS.md): Gaea's Cradle's mana, Ancient Tomb at low life, Mox Diamond's land, Construct tokens and Urza's Saga, Cyclonic Rift's overload at the end of the turn before yours, wipe losses by each wipe's own rule (Teferi's Protection), Farewell keeping reanimation targets, Thassa's Oracle with Tainted Pact, Breach escapes under Drannith Magistrate, Ad Nauseam at end of turn, Rhystic Study payments, tutor targets and timing (Enlightened, Mystical, Gamble, Natural Order, Crop Rotation, late fallbacks), Fierce Guardianship, and Citadel's life margin. |
| `test_search.py` | 7 | The look-ahead AI: game copies are independent, hidden hands are re-dealt correctly, a win always outscores a board, and a whole decision picks one of the options, leaves the real game untouched, and is reproducible. |
| `test_dsl.py` | 5 | The ability language: Oracle text compiles to the expected abilities, unreadable text is reported, and a compiled card works in a game. |
| `test_cli.py` | 14 | The Wilson interval and the paired difference, and every command run the way you run it (`python3 -m ...`) with a few games: a tier run, `--swap`, `--analyze`, `--trace`, `--calibrate`, `--cards`, the validator, the audits and `tools.swaptest`. Also checks that `--jobs 1` and `--jobs 3` give the same result. |
| `test_update_deck.py` | 10 | The deck updater: list formats from deck sites, name matching, a swap rewriting every section, bad lists refused with nothing written, a pool deck's Game Changer rule and header, and `--dry-run`. |
| `test_pool_games.py` | 2 | Seeded games from every tier, alone and with each of your decks, play to the end; one game with the look-ahead AI. |
| `test_pool_sampling.py` | 8 | Seating: opponents drawn without replacement, seeded and uniform. Pairing: a changed list faces the same opponents, seats and draws; games replay exactly. |
| `test_validator.py` | 15 | The decklist checks: size, singleton, commander, name resolution, colour identity, bans, Game Changers per tier. |
| `test_my_decks.py` | 1 | Your deck files parse to the lists recorded in `fixtures/my_decks_parsed.json`. |
| `test_sim_guard.py` | 1 | 42 seeded games (heuristic AI) play out exactly as recorded in `fixtures/sim_guard.json`, so practice-mode hooks can't change a simulation. Re-record after an intended change: `python3 tests/test_sim_guard.py --record`. |
| `test_land_entry.py` | 11 | Lands that enter tapped unless a condition holds, read from their Oracle text: check lands (dual types count), fast lands, slow lands, snarls (a matching card in hand), battle and bond lands, Mystic Sanctuary, and plain tapped/untapped lands.; and lands' enters triggers: you scry when your Temple enters, a gain land gains its life once.; shock lands (you choose to pay 2 life or enter tapped; the AI pays only when it uses the mana) |
| `test_determinism.py` | 4 | The same seed plays the same game wherever objects land in memory: cards, players, permanents and lands hash without their addresses (copies like their originals), and a dead permanent's address is never reused while its game lasts. |
| `test_play.py` | 12 | Practice mode: a session plays a game on its own thread and streams events; the AI's reasoning lines (which name cards in opponents' hands) are kept out of the stream; seats and opponents; step mode waits for you; closing stops the game; the text client; the table view hides other players' hands; look-ahead copies leave the human seat to the AI; actions carry the table's view for playback (only when asked for). |
| `test_play_mana.py` | 15 | Practice mode's mana pool: paying colours and generic, any-colour mana, tapping lands, rocks and Treasures into the pool, the wrong colour refused, Talismans hurting only for coloured mana, the reason when you're short, ritual mana joining the pool, and the pool emptying between steps. |
| `test_play_turn.py` | 9 | Practice mode's main phase: one land per turn, lands and sorceries only in your main phase, instants at any time, the reason when mana is short, commander tax; tap-then-cast, an illegal move explained while you keep priority, casting your commander; and a bot playing whole games of every deck (Galadriel, Y'shtola, Alela and Jodah included) through the human seat. |
| `test_play_abilities.py` | 12 | Practice mode's targets and abilities: what a removal spell may target (your own permanents too; hexproof respected), burn at players (not one under Teferi's Protection), casting with a target, cancelling costs nothing, Bloodchief's Thirst's kicker, Equip (mana and timing), a planeswalker's loyalty ability, a permanent with nothing to activate.; choices point at where they are on the table (for clicking in the browser). |
| `test_play_combat.py` | 17 | The AI splitting its attackers between players (away from a blocker that would eat one, but only as many as there are such blockers; finishing off a player who can't block; nothing moves without a reason; two players attacked in one combat). Practice mode's combat declarations: who can attack (by the rules, not the AI's habits), attacking the only opponent, choosing the defending player, splitting attackers between players (a split that sends them all to one player is one attack; no split for a single attacker), not attacking, blocking, not blocking, and unblockable attackers not being asked about.; with a person at the browser, attack and block requests say where the attackers and blockers are. |
| `test_play_respond.py` | 9 | Practice mode's priority on other turns: countering an opponent's spell, passing so it resolves, priority on every spell (not just ones the AI would counter), tapping then countering, a counterspell that can't hit that kind of spell, no counterspell without a spell to counter, countering back when the AI counters you, and priority when attacked; the priority order on a spell. |
| `test_play_choices.py` | 10 | Practice mode's smaller choices: discards (effects and hand size), edicts, tutors limited to what they can find, finding nothing, basic-land searches, scry, the London mulligan with a free first mulligan, choosing modes (Kolaghan's Command), and yes/no. |
| `test_play_sauron.py` | 17 | Practice mode, Sauron's cards: Orcish Bowmasters' damage when it enters and on extra draws, Kaervek's damage (a creature survives damage below its toughness), the Ring (Ring-bearer, Call of the Ring's "pay 2 life", Sauron's "discard your hand"), Jace's Archivist (and summoning sickness), Aggravated Assault, Rogue's Passage through real combat, Scavenger Grounds, Cyclonic Rift overloaded, Toxic Deluge's X, Bitter Triumph's discard-instead-of-life, Noxious Gearhulk's optional target, Kefka's discard, and opponents' taxes (Rhystic Study asks whether to pay; you aren't asked when you can't). |
| `test_play_seph.py` | 17 | Practice mode, Sephiroth's graveyard cards: Animate Dead from an opponent's graveyard, Unburial Rites only from yours, Entomb, Grisly Salvage, Deadly Dispute's sacrifice, flashback (Deep Analysis), Dread Return's sacrifice-three flashback, Yawgmoth's Will, no graveyard casting otherwise, starting Mikaeus + Triskelion as a shortcut, Aura Shards' optional target, Archon of Cruelty's target opponent, Consecrated Sphinx's "may draw", Nim Deathmantle's "pay {4}", Tortured Existence, and Gifts Ungiven (you pick, the AI opponent splits). |
| `test_play_cards.py` | 16 | Practice mode, your decks' activated abilities by the rules: Vraska's -2 on your pick, once per turn and sorcery speed, Ral Zarek's -2, Mind Stone at any time (and needing mana), a prepared Lightning Bolt, Aetherflux Reservoir (and its 50 life), Triskelion, Strip Mine on any land, Desolate Lighthouse, and no sorcery-speed play in response. |
| `test_play_hand.py` | 11 | Practice mode, casts from hand the card code keeps for the AI: Twinflame (two targets; the extra target's mana; copying a token), Ephemerate and its rebound (declined: stays in exile), cycling at instant speed, Unearth's pick, Sokenzan's channel, Necromancy at instant speed; Return the Favor (needs a spell; copies your Lightning Bolt while you hold priority; never cast for you by the AI). |
| `test_play_veyran.py` | 15 | Practice mode, Veyran's spells: Crackle with Power's X and targets (and stopping early), Mizzix's Mastery's target, Flashback granting flashback, Jeska's Will (mana from an opponent's hand; exiled lands aren't played for you), Expressive Iteration, Stock Up, Prismari Charm's two pings, and Thousand-Year Storm copies taking new targets; Alania, Divergent Storm: the first instant copied once for an opponent's card, Veyran and Harmonic Prodigy each adding a copy, the first sorcery too but not a counterspell, a spell cast before Alania counting as the first, and your choice of who draws (or no copy). |
| `test_play_marchesa.py` | 6 | Practice mode, cards first written for the Marchesa deck (removed 2026-10-07) that other lists still run, and the shared sacrifice outlets: Accursed Marauder (you choose your sacrifice), Mystic Remora's upkeep declined, dredge instead of drawing, Carrion Feeder, Ashnod's Altar, and Coalition Relic's charge. |
| `test_play_server.py` | 12 | Practice mode's browser server: the page and its files (no path escapes), card images served from the cache, the loading screen before the game, the setup options, bad new-game requests (unknown deck, two opponents, a seat that isn't a number), picked opponents with a chosen seat and the practice switches, and a game started over HTTP: the event stream, the mulligan answered (stale and repeated answers refused), the next decision, and opponents' hands hidden in the view; Undo over HTTP (nothing to undo, the mulligan asked again, switched off); Hint over HTTP (and switched off); the review stays closed until the game ends; a page catching up gets only the latest table; saving, listing and loading a game (no path escapes); the auto-pass setting (at the start, changed during the game, bad values refused). |
| `test_play_lan.py` | 15 | Practice mode with two people and two AI opponents: the session seats both, asks each their own mulligan and decisions (each request's table from its own seat), a view for each seat with every action, Undo by either replays both, saving and loading keeps both, separate reviews (and Try it on one person's decision), Hint only for the seat being asked, the same deck twice refused; a tutored card named only to its owner (an AI's to no one; a simulation keeps the whole line); over HTTP: the lobby and its code (hidden from the friend, wrong code, the host's deck, the host's own browser refused), each browser's seat by cookie, host-only actions refused from the other computer, each seat's stream with only its own requests and hand, Undo the other person must agree to (declined, then accepted), a saved two-player game waiting for the friend, and one player's Undo at once. |
| `test_zur.py` | 19 | Cards first written for the Zur deck (removed 2026-10-07) that other lists still run, seated in your Y'shtola deck: Arrest (no attacks, blocks or abilities, until it leaves), Luminous Bonds, Encrust (no mana, no untapping), locked creatures worth less and not locked twice, a lock Aura with nothing to enchant; Zur's fetch from the 99 (mana value 3 or less, a lock Aura ignoring hexproof); Bastion Protector (commander only), The Eternal Wanderer (+1 returns at its owner's end step, the Samurai's double strike, -4), Prayer of Binding, Disenchant's targets, Rootborn Defenses against a wipe; practice mode: Arrest's target chosen once, Zur's fetch, Clever Concealment with convoke, The Eternal Wanderer's abilities.; Necropotence in practice mode (you choose the life; the cards arrive at your next end step, or the one after if paid in your end step; the AI doesn't pay for you). |
| `test_audit_fixes.py` | 11 | The gaps the October 2026 modeling audit found: Muldrotha's one permanent of each type from the graveyard (the AI and you), Restoration Angel's enters trigger (cast normally, it blinks; you choose or decline), Kefka's draw, and choices practice mode used to make for you (Niv-Mizzet, Emeritus of Ideation, Force of Will's exiled card, X). |
| `test_stack.py` | 10 | The stack: a counter war between three players, a countered counterspell doing nothing, responses resolving first with priority going round, the person seeing the whole stack, the stack copying with the game; activated abilities on the stack (the AI and you countering one with Azorius Guildmage, a countered equip not attaching, an ability resolving after its source dies in response, and no round of priority when nobody could answer). |
| `test_triggers.py` | 12 | Triggered abilities on the stack: an enters trigger waiting for the spell resolving to finish, Tidebinder countering a trigger, your priority on an opponent's trigger, the engine's probe never doubling an effect, APNAP order, ordering your own triggers, converted card code having its window; the look-ahead: a copied game keeps its waiting triggers, and a countered trigger moves no card; "same order as last time"; combat damage triggers waiting until all the damage is dealt; Light-Paws's window. |
| `test_steps.py` | 9 | Priority in every step: full control stops at the upkeep, draw step, beginning of combat and end step; by default only the end of each other turn (not every step); the declare blockers step of your combat when you could do something (and not when you couldn't); a creature whose blocker is removed stays blocked; the AI killing a dangerous attacker in the declare attackers step. The auto-pass setting: a change applies from your next answer, Undo replays the old setting then keeps the new, and a saved game keeps them. |
| `test_alela.py` | 29 | Avery's Alela deck: Faeries from artifact and enchantment spells (2/1 with Alela's anthem); the Jace token (empower creates and grows it, -3 draws, it leaves no card behind, Annex lands untapped with it, Plan for All Outcomes on your first noncreature spell); Multiply by Zero (counters save it), Prophesied End's draw (not on an attacker), Plan putting a permanent on top, Karmic Justice (only an opponent destroying a noncreature permanent), Ray of Command (back tapped), Malice (nonblack only), Muddle the Mixture (instants and sorceries); Opposition (their mana in their upkeep, their attackers at their combat); Static Prison's energy and Aether Hub; Plumecreed Mentor with Faerie tokens, Jackdaw Savior, Malcator's end-step Golem, Airlift Chaplain, Skycoach's crew; Helping Hand, Daydream, Ajani Fells the Godsire, Venser's emblem. |
| `test_jodah.py` | 17 | JD's Jodah deck: Jodah's anthem (counts legends only) and legend cascade (from hand only, finds the first cheaper legend); Kaldra (the Helm assembles it: 9/9, indestructible) and the Sword exiling what it damages; Blackblade Reforged and Korlash counting lands and Swamps; Szadek's counters-and-mill instead of damage; Dragonlord Dromoka stopping spells on your turn; Mirri capping attackers; Carth's extra loyalty; Memory Jar's wheel and end-step return; Profane Tutor's suspend; Dissipate and Desertion; Court of Ardenvale; Fyndhorn Elder's two mana. |
| `test_galadriel.py` | 33 | Your Galadriel deck: Alliance (each mode once a turn, your choice, twice with Panharmonicon), Panharmonicon with Cathars' Crusade; the Rebel searchers (within their cap), Maskwood Nexus making every creature a Rebel, Lin Sivvi's X, Ramosian Revivalist; Kindred Discovery naming a type (Sauron's still Orc), Door of Destinies and Vanquisher's Banner, Secluded Courtyard's restricted mana (AI and your pool); Crackdown, Cho-Manno, Knight of the Holy Nimbus (regenerates unless paid), Defiant Vanguard, Amrou Seekers, Abduction (returns its owner's creature when it dies), Bribery, Eerie Interlude, Voice of Resurgence, Mangara, Springleaf Drum and Elvish Archdruid, Return to Dust, Unbreakable Formation's addendum, Elspeth's emblem; practice mode: a searcher, Whipcorder's morph, Mirror Entity, Austere Command's modes, Mentor of the Meek's payment, naming a type. |
| `test_yshtola.py` | 39 | Your Y'shtola deck: her cast trigger (mana value 3 or more, X included; not creatures, small spells or opponents' spells), her end-step draw (life lost counted once), vigilance; Curiosity (a card per opponent her trigger hits; combat damage); Blight-Priest and Sanguine Bond (Bond takes a kill), Debt to the Deathless, the AI's X drain for a kill (and no small X without one), Ill-Gotten Inheritance, Urborg Syphon-Mage; Gonti (cast with any mana; still its owner's card), a taken spell to its owner's graveyard, taken cards kept through hand size, Hostage Taker (returns when it leaves; cast it and keep it), Thief of Sanity; Jester's Cap, Dark Petition's spell mastery, Plea for Guidance, Take Up the Shield saving Y'shtola, Zur in the 99 fetching Curiosity for her, no chump blocks with her; the AI's priorities and tutor picks (Idyllic Tutor too); practice mode: her trigger on your spell, X, Syphon-Mage, Inheritance and Jester's Cap, Gonti's pick, Hostage Taker's target, Idyllic Tutor's search, Take Up the Shield's target. |
| `test_play_undo.py` | 3 | Practice mode's Undo: after 40 decisions, taking back three asks the 37th again with the identical table; replayed history is marked as such; nothing to undo at the start. |
| `test_play_hint.py` | 2 | Practice mode's Hint: asked at every decision of a look-ahead game, the game that follows is identical to one without hints; main-phase hints come with the look-ahead's scores; no decision waiting. |
| `test_play_shadow.py` | 2 | Practice mode's AI comparison: with it on, a look-ahead game plays out identically; main-phase decisions are scored for your choice and the AI's; Undo forgets the comparisons of undone decisions. |
| `test_play_review.py` | 3 | Practice mode's review: after a game, the summary and the compared decisions; "Try it" restarts at a decision with the AI's choice answered for you; the same seed again with the exact seating; Try it needs a compared decision. |
| `test_play_save.py` | 2 | Practice mode's saved games: a look-ahead game saved after 50 decisions and loaded (through JSON) comes back to the identical decision, with its comparisons; a file from another version is refused. |
| `test_play_images.py` | 5 | Practice mode's card images, with stand-ins for Scryfall: cards, the tokens they make and both faces of a two-faced card; nothing fetched the second time; a failed download left out; offline; the names in a game's decks. |
| `test_play_catalog.py` | 3 | Practice mode's setup screen data: your decks (Sephiroth's display name, bracket, Game Changers), the five tiers with their labels, and win rates read from the results table. |

## When a test fails

- **`test_my_decks` fails after you edit a deck.** That's expected: it guards against accidental edits. Once the
  new list is what you want, rewrite the fixture:

  ```
  python3 tests/test_my_decks.py --record
  ```

  `python3 -m commander_sim.update_deck` records it for you when it updates a list.

- **A card test fails.** A card test failing means the simulator no longer does what the card's Oracle text says.
  Fix the card's implementation, not the test, unless the test itself misread the card.

## Coverage

```
python3 -m commander_sim.tools.linecov                     # per-module table and total
python3 -m commander_sim.tools.linecov --missing engine    # also the lines no test runs, for matching modules
```

The tool runs the whole suite while recording which lines of `commander_sim` execute. That includes the
subprocesses the command-line tests start, and their worker processes. It takes about a minute, uses only the
standard library, and exits with an error if a test fails.

Line coverage was 86% in September 2026. It shows which code the tests reach, not whether they check its result:
the end-to-end games reach most of the game code, but only the rule and card tests check what it does.

## Writing a rule or card test

`tests/table.py` builds a position without playing a game:

```python
from tests.table import table, hand, lands, perm, token
from commander_sim import engine as E

g = table('seph', 'veyran')                 # seats, in turn order; hands empty, life 40, first seat active
s, v = g.players
lands(v, 'Island', 2)                       # untapped lands
hand(v, 'Counterspell')                     # cards into a hand (taken from that deck's library if it runs them)
perm(g, s, 'Sheoldred, the Apocalypse')     # onto the battlefield: its enter effects happen
E.draw(g, v, 1)                             # then drive the engine directly
assert v.life == 38
```

Useful engine entry points:

- `E.draw`, `E.on_cast` (cast triggers), `E.magecraft`;
- `E.apply_removal`, `E.apply_wipe`, `E.die`, `E.check_state`;
- `E.pay` and `E.can_pay`;
- `ais.resolve_combat` (attackers against one defender), `ais.attack_triggers`, `ais.upkeep`.

Take the expected numbers from the card's Oracle text. Prefer situations where the AI has no real choice, so
the result doesn't depend on its randomness. Blocks, for example, are only random for chump blocks.
