# Tests

Run every test from the repository root:

```
python3 -m unittest discover -s tests -t .
```

That's 398 tests in about a minute. To run one file, or one test:

```
python3 -m unittest tests.test_my_cards
python3 -m unittest tests.test_my_cards.Sephiroth.test_massacre_wurm
```

The tests need no network access. The card data they use is in `data/scryfall_cache.json`.

## What each file covers

| File | Tests | What it checks |
|---|---|---|
| `test_rules.py` | 34 | Core rules on hand-built positions: mana colours and payment order, Sol Ring, Talisman pain, commander tax, the state-based losses (life, 21 commander damage, empty library, 10 poison), what each counterspell can hit and what it costs, Swords / Path / Bolt / token removal, Toxic Deluge, tutors, mulligans, commander damage, flying, deathtouch, lifelink, and Teferi's Protection (the life lock and protection, phasing, when it ends, which wins it survives, when the AI casts it). |
| `test_my_cards.py` | 102 | Key cards of your four decks against their Oracle text, and Sephiroth's four loops (Mikaeus + Triskelion, Mikaeus or Melira + Kitchen Finks, Nim Deathmantle + Ashnod's Altar + Grave Titan): when each kills, what stops it, and how the AI finds and assembles the pieces. Also Kefka, Brush Off, Melira, Avacyn's Pilgrim, Unsummon, Twinflame, Champion's Helm, Slaughter Pact, Urabrask, the Underworld Breach line (Brain Freeze and Grapeshot with storm), and the Sauron AI's priorities for Sheoldred and Consecrated Sphinx. Marchesa: her return trigger (wipes, stolen creatures, exile), dethrone, the Triskelion and Act of Treason lines, and each card the deck needed code for. |
| `test_search.py` | 7 | The look-ahead AI: game copies are independent, hidden hands are re-dealt correctly, a win always outscores a board, and a whole decision picks one of the options, leaves the real game untouched, and is reproducible. |
| `test_dsl.py` | 5 | The ability language: Oracle text compiles to the expected abilities, unreadable text is reported, and a compiled card works in a game. |
| `test_cli.py` | 14 | The Wilson interval and the paired difference, and every command run the way you run it (`python3 -m ...`) with a few games: a tier run, `--swap`, `--analyze`, `--trace`, `--calibrate`, `--cards`, the validator, the audits and `tools.swaptest`. Also checks that `--jobs 1` and `--jobs 3` give the same result. |
| `test_update_deck.py` | 10 | The deck updater: list formats from deck sites, name matching, a swap rewriting every section, bad lists refused with nothing written, a pool deck's Game Changer rule and header, and `--dry-run`. |
| `test_pool_games.py` | 2 | Seeded games from every tier, alone and with each of your decks, play to the end; one game with the look-ahead AI. |
| `test_pool_sampling.py` | 8 | Seating: opponents drawn without replacement, seeded and uniform. Pairing: a changed list faces the same opponents, seats and draws; games replay exactly. |
| `test_validator.py` | 15 | The decklist checks: size, singleton, commander, name resolution, colour identity, bans, Game Changers per tier. |
| `test_my_decks.py` | 1 | Your deck files parse to the lists recorded in `fixtures/my_decks_parsed.json`. |
| `test_sim_guard.py` | 1 | 24 seeded games (heuristic AI) play out exactly as recorded in `fixtures/sim_guard.json`, so practice-mode hooks can't change a simulation. Re-record after an intended change: `python3 tests/test_sim_guard.py --record`. |
| `test_land_entry.py` | 7 | Lands that enter tapped unless a condition holds, read from their Oracle text: check lands (dual types count), fast lands, slow lands, snarls (a matching card in hand), battle and bond lands, Mystic Sanctuary, and plain tapped/untapped lands. |
| `test_determinism.py` | 3 | The same seed plays the same game wherever objects land in memory: cards, players, permanents and lands hash without their addresses (copies like their originals), and a dead permanent's address is never reused while its game lasts. |
| `test_play.py` | 11 | Practice mode: a session plays a game on its own thread and streams events; the AI's reasoning lines (which name cards in opponents' hands) are kept out of the stream; seats and opponents; step mode waits for you; closing stops the game; the text client; the table view hides other players' hands; look-ahead copies leave the human seat to the AI; actions carry the table's view for playback (only when asked for). |
| `test_play_mana.py` | 10 | Practice mode's mana pool: paying colours and generic, any-colour mana, tapping lands, rocks and Treasures into the pool, the wrong colour refused, Talismans hurting only for coloured mana, the reason when you're short, ritual mana joining the pool, and the pool emptying between steps. |
| `test_play_turn.py` | 9 | Practice mode's main phase: one land per turn, lands and sorceries only in your main phase, instants at any time, the reason when mana is short, commander tax; tap-then-cast, an illegal move explained while you keep priority, casting your commander; and a bot playing whole games of every deck through the human seat. |
| `test_play_abilities.py` | 11 | Practice mode's targets and abilities: what a removal spell may target (your own permanents too; hexproof respected), burn at players (not one under Teferi's Protection), casting with a target, cancelling costs nothing, Bloodchief's Thirst's kicker, Equip (mana and timing), a planeswalker's loyalty ability, a permanent with nothing to activate.; choices point at where they are on the table (for clicking in the browser). |
| `test_play_combat.py` | 9 | Practice mode's combat declarations: who can attack (by the rules, not the AI's habits), attacking the only opponent, choosing the defending player, not attacking, blocking, not blocking, and unblockable attackers not being asked about.; with a person at the browser, attack and block requests say where the attackers and blockers are. |
| `test_play_respond.py` | 9 | Practice mode's priority on other turns: countering an opponent's spell, passing so it resolves, priority on every spell (not just ones the AI would counter), tapping then countering, a counterspell that can't hit that kind of spell, no counterspell without a spell to counter, countering back when the AI counters you, and priority when attacked; the priority order on a spell. |
| `test_play_choices.py` | 10 | Practice mode's smaller choices: discards (effects and hand size), edicts, tutors limited to what they can find, finding nothing, basic-land searches, scry, the London mulligan with a free first mulligan, choosing modes (Kolaghan's Command), and yes/no. |
| `test_play_sauron.py` | 17 | Practice mode, Sauron's cards: Orcish Bowmasters' damage when it enters and on extra draws, Kaervek's damage (a creature survives damage below its toughness), the Ring (Ring-bearer, Call of the Ring's "pay 2 life", Sauron's "discard your hand"), Jace's Archivist (and summoning sickness), Aggravated Assault, Rogue's Passage through real combat, Scavenger Grounds, Cyclonic Rift overloaded, Toxic Deluge's X, Bitter Triumph's discard-instead-of-life, Noxious Gearhulk's optional target, Kefka's discard, and opponents' taxes (Rhystic Study asks whether to pay; you aren't asked when you can't). |
| `test_play_seph.py` | 16 | Practice mode, Sephiroth's and Marchesa's graveyard cards: Animate Dead from an opponent's graveyard, Unburial Rites only from yours, Entomb, Grisly Salvage, Deadly Dispute's sacrifice, flashback (Deep Analysis), Dread Return's sacrifice-three flashback, Yawgmoth's Will, no graveyard casting otherwise, starting Mikaeus + Triskelion as a shortcut, Aura Shards' optional target, Archon of Cruelty's target opponent, Consecrated Sphinx's "may draw", Nim Deathmantle's "pay {4}", Tortured Existence, and Gifts Ungiven (you pick, the AI opponent splits). |
| `test_play_cards.py` | 12 | Practice mode, your decks' activated abilities by the rules: Vraska's -2 on your pick, once per turn and sorcery speed, Ral Zarek's -2, Mind Stone at any time (and needing mana), a prepared Lightning Bolt, Aetherflux Reservoir (and its 50 life), Triskelion, Strip Mine on any land, Desolate Lighthouse, and no sorcery-speed play in response. |
| `test_play_hand.py` | 15 | Practice mode, casts from hand the card code keeps for the AI: Twinflame (two targets; the extra target's mana; copying a token), Ephemerate and its rebound (declined: stays in exile), cycling at instant speed, Unearth's pick, Disintegrate at a player, Disembowel's X, Lethal Throwdown, Sokenzan's channel, Necromancy at instant speed; Return the Favor (needs a spell; copies your Lightning Bolt while you hold priority; never cast for you by the AI) and Dualcaster Mage copying an opponent's spell. |
| `test_play_veyran.py` | 10 | Practice mode, Veyran's spells: Crackle with Power's X and targets (and stopping early), Mizzix's Mastery's target, Flashback granting flashback, Jeska's Will (mana from an opponent's hand; exiled lands aren't played for you), Expressive Iteration, Stock Up, Prismari Charm's two pings, and Thousand-Year Storm copies taking new targets. |
| `test_play_marchesa.py` | 12 | Practice mode, Marchesa's cards and the shared sacrifice outlets: Act of Treason's and Enslave's targets, Crux of Fate's mode, Forge Devil's target, Accursed Marauder (you choose your sacrifice), Phyrexian Delver's pick, Al Bhed Salvagers' target opponent, Mystic Remora's upkeep declined, dredge instead of drawing, Carrion Feeder, Ashnod's Altar, and Coalition Relic's charge. |
| `test_play_server.py` | 12 | Practice mode's browser server: the page and its files (no path escapes), card images served from the cache, the loading screen before the game, the setup options, bad new-game requests (unknown deck, two opponents, a seat that isn't a number), picked opponents with a chosen seat and the practice switches, and a game started over HTTP: the event stream, the mulligan answered (stale and repeated answers refused), the next decision, and opponents' hands hidden in the view; Undo over HTTP (nothing to undo, the mulligan asked again, switched off); Hint over HTTP (and switched off); the review stays closed until the game ends; a page catching up gets only the latest table; saving, listing and loading a game (no path escapes). |
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
