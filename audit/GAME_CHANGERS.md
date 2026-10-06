# Game Changer audit (2026-10-05)

Every Game Changer in a deck the simulator plays (42 cards across your 7 decks and the 25 pool decks) was checked
for two things: is it modeled correctly against its Oracle text in ways that matter in games, and does every AI that
holds it decide to cast it from the game state rather than a flat number. Method: a cast-rate sweep
(`audit/gc_sweep.py`: each of your decks against each tier, every deck at the table recorded) plus a code review of
each card and each AI path. The sweep found no Game Changer stuck at "never cast"; the review found the issues below.

## Bugs fixed

| Card(s) | Problem | Fix |
| --- | --- | --- |
| Gaea's Cradle, Mox Opal, Mox Amber, Cabal Coffers, Nykthos, Priest of Titania, Heritage Druid and 7 more | Variable mana always made 1 (only cards with an event hook got their mana function) | `cardimpl.dyn_mana`: being in `DYN_MANA` is enough |
| Cyclonic Rift | Never overloaded at instant speed; spent on one target at end of turn | Instant wipes offered at the end of the turn before yours; overload preferred when it bounces 1.5x the best single target; bounced nontokens valued at half |
| Teferi's Protection | Fired against wipes that missed (Vandalblast vs creatures); pool decks used it on single removal | `ais.wipe_hit` / `wipe_loss`: loss by the wipe's own rule; on single removal only for the commander or 40%+ of the board |
| Farewell | Graveyard mode exiled your own reanimation targets | `ais.gy_worth`: creatures in a graveyard count for players who can bring them back |
| Underworld Breach, Bolas's Citadel | Escapes and casts off the top ignored Drannith Magistrate and other locks | `castable(..., 'gy' / 'lib')` checks (the Breach line too) |
| Necropotence | Cast below its own payment floor, so it only skipped draws | `ais.necro_floor` / `necro_prio`: one threat-based floor for casting and paying |
| Thassa's Oracle, Tainted Pact | Combo fizzled on payment order; Pact cast as a cantrip; Oracle cast as a vanilla 1/3; Pact assumed to work with duplicate basics | Oracle first; Pact only with a duplicate-free library; Oracle's real ETB; cast alone only when it wins |
| Ad Nauseam (and others) | End-of-turn window turned an AI "no" into a cast (0 cards drawn) | The window respects `None`; Sephiroth's Gifts Ungiven got a real priority |
| Mox Diamond | Cast with no land (the rock rule shadowed its check); entered free via Urza's Saga | Its check comes first; the discard replacement also applies in `enter()` |
| Ancient Tomb | Could kill its controller; land choice counted colourless as a new colour | Not tapped at 2 life; land scores by mana made, Workshop by artifacts in hand, Cradle by creatures |
| Natural Order | Counted creatures before the sacrifice; a deliberate 0 was revived by the DSL fallback | `natural_order_prio`; hand-written priorities of 0 stay 0 |
| Gamble | Cast with an empty hand, discarding the tutored card | `gamble_prio`: by the chance of keeping the card |
| Enlightened Tutor (and every DSL search for "artifact or enchantment", "creature or planeswalker" ...) | Found any card | `dsl.CARD_TYPE_OK` |
| Mystical Tutor | Put the card in hand, cast on your own turn | On top of the library (`tutor_to_top`), cast at the end of an opponent's turn |
| Fierce Guardianship | Treated as costing mana with the commander out | `engine.free_counter` in the combo check, held mana and counter risk |

Also found along the way (not Game Changers): **every 0/0 Construct token died on creation** (Urza, Lord High
Artificer's and Urza's Saga's): `make_tokens` checked toughness before the caller attached the Construct's data, so
the +1/+1-per-artifact bonus never applied. It now takes `data=`. **Urza's Saga** makes a Construct on chapter II
and again in response to chapter III (the {2} from other sources, the Saga taps), when the Construct would be 3/3+ or
the mana is spare, and chapter III takes the deck's wish list first.

## Modeling gaps fixed

Rhystic Study / Smothering Tithe payment (an uncastable card in hand no longer stops the payment), Crop Rotation
(fetches Gaea's Cradle or a fetch land, not a random basic), Seedborn Muse (untaps Grim Monolith / Mana Vault), Mana
Vault ({4} upkeep untap), tutor targets (Worldly Tutor, Survival of the Fittest and Natural Order use the wish list; the
fallback ranks by impact after turn 4, not cast priority: no Sol Ring on turn 10), Gifts Ungiven / Intuition piles
(seeded with the deck's tutor target, reanimation targets valued), Aura Shards (artifact and enchantment creatures),
Opposition Agent (taken cards don't count toward hand size), Tergrid (menace), Orcish Bowmasters (each one triggers).

## Contextual priorities (no more flat numbers)

`commander_sim/ai/gc_prio.py` holds the shared functions; every deck's AI uses them: Bolas's Citadel and its
casting floor, Ad Nauseam and its stopping point, The One Ring, Rhystic Study, Jeska's Will, fast mana (Chrome Mox,
Mox Diamond, Grim Monolith; combo pieces first), Seedborn Muse, Braids, Aura Shards, Drannith Magistrate, Opposition
Agent. Also `t4.sphinx_prio`, `rules.tithe_prio`, `ais.necro_prio`, `pool_ai.tutor_prio` (wish-list hits first, by
impact otherwise, life cost for Vampiric Tutor / Imperial Seal). Flash creatures (Notion Thief, Bowmasters,
Opposition Agent) are held for the end of the turn before yours; Force of Will needs a bigger threat when it pitches.

## Known approximations left

Jeska's Will's red mana floats to the end of the turn rather than the step; Teferi's Protection taps lands instead of
phasing them; The One Ring's tap decision considers only your life; Tergrid's deck doesn't hold edicts for Tergrid.

## Pool balance after the fixes (heuristic AI, conservative, 1,000 games)

Broadly unchanged. Marwyn rose from 6.9% to 11.6% within Tier 3 (the Cradle fix); Brago is no longer low; Atraxa,
Aurelia and Winota are still above 35%; Tier 4 is still below Tier 3 with this AI, as before. The tiers are calibrated
with the look-ahead AI (decklists/pool/pool-results.md, section 0), which wasn't re-run.
