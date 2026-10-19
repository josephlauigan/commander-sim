# Practice mode: where the AI decides for you (audit, 19 Oct 2026)

Read-only audit of the Rust card code and engine for JD's 13 decks (alela, cloud, galadriel, godo, kefka, kraum,
sauron, seph, sephfs, sheoldred, titania, veyran, yshtola). Five agents read the 45 card files and the engine; their
tables are in `findings-g1.md` ... `findings-g5.md` (about 165 rows). The four engine-wide causes below were checked in
the code by hand; the per-card rows are the agents' readings and each needs a look when it is fixed.

## Four causes explain most of it

1. **"You may" triggers never ask.** `engine::stack::trigger_window` only gives everyone priority; it has no yes/no for
   the controller. Whatever the card code does next happens, so every optional trigger is always used (or skipped)
   by the AI's judgment. About 30 cards: Seymour Flux, Karmic Justice, Plan for All Outcomes, Rooftop Assassin, Soul
   Enervation, Phyrexian Delver, Moseo, Airlift Chaplain, Jackdaw Savior, The Wasp, Flickering Hound, Conjurer's Closet,
   Teleportation Circle, Stadium Tidalmage, Jocasta, Helm of the Host, Gix, Silverblade Paladin, Cursed Mirror, Ragavan,
   Boughside Wanderers, Mana Vault ...
2. **Alternative costs and graveyard/land abilities are AI-only.** `hand_options`, `gy_options` and `land_options`
   are read only by `ai/brain.rs` (checked: nothing in `human/` or `practice/` calls them). About 115 hooks in 37
   files; the ones that matter are the alternative ways to cast or use a card: dash (Ragavan), escape (Woe Strider),
   suspend (Profane Tutor), flashback (Twinned Vision), unearth (Reconstructed Thopter), transmute (Muddle the
   Mixture), blitz (Jaxis), warp (Susurian Voidborn, Perigee Beckoner), mayhem (Scarlet Spider), morph / disguise
   (Exalted Angel, Forum Familiar, Greenbelt Radical), web-slinging, plot, foretell, landcycling (Titania's deck),
   multikicker (Everflowing Chalice taps for 0), Reassembling Skeleton's return, Demonic Embrace from the graveyard,
   and land abilities (Command Beacon, Karn's Bastion, Ghost Quarter, Geier Reach Sanitarium, Mikokoro, Phyrexian
   Tower, Otherworld Atlas).
3. **Spells with their own resolve code but no "removal" tag get no target from you.** `human::legal::spell_targets`
   returns nothing unless the card has `Tag::Rem` (checked). Such a spell resolves with an empty target: it does
   nothing (**Lethal Throwdown**, **Presumed Dead**, **Stargaze** with X = 0, **Grab the Prize** without its damage,
   Dark Endurance) or its code picks for you (**Splatter Technique** always the wipe, **Blood on the Snow** always the
   creature mode, **Extinction Event** odd/even, Comet Storm, Cost of Brilliance / Painful Lesson / Peer into the Abyss
   always target you, Rewrite Regrets, Daydream, Helping Hand, Necromantic Selection, Victimize, Scout the City,
   Glorious Decay ...).
4. **Card code with no "is it a person?" branch** makes the choice itself: which target, mode, creature, card or
   player (Inferno Titan's 3 damage, Smaug, Godo's Equipment, Kor Outfitter, Armory Automaton, Gilgamesh, Titania's
   land, Genesis Hydra, Cartographer's Survey, Court of Ardenvale, Triple Triad auto-playing everything, Grapeshot and
   Brain Freeze copies, Bojuka Bog, Farseek, Mystic Sanctuary, Kura, proliferate always "everything" ...).

## Engine-level gaps (all decks)
- **Your creatures can't attack planeswalkers** (`engine/combat.rs:559`, `to_walker` is empty for a person). High.
- Blocks: one blocker per attacker; combat damage and trample assignment are fixed for you. Med.
- Additional costs (sacrifice or discard as a cost: Redirect Lightning, Seize the Spoils, Thrill of Possibility,
  Silence the Echo, Stir Up Trouble): the AI picks what you pay (`engine/cast.rs:62`). Med.
- Legend rule (`life.rs:219`), commander always to the command zone (`zones.rs:559`), Mox Diamond and Chrome Mox
  (forced, AI picks the card): med.

## When you are the opponent (the AI chooses your side)
Fact or Fiction (no piles: the AI splits and picks), Accursed Marauder and Plaguecrafter (your sacrifice), Kefka,
Ruler of Ruin's {8} (your sacrifice), Invoke Despair, Torment of Hailfire, Intellect Devourer, End of the Hunt,
Braids's Frightful Return ch. III, Sauron's ward, Phyrexian Obliterator, Esper Sentinel (pays your {X} for you),
Norn's Annex (auto-pays or drops your attackers). The generic edict and discard (`zones::edict`, `discard_worst`)
already ask you; these cards bypass them.

## Highest-impact cards, by deck
- **sephfs:** Presumed Dead and Stargaze do nothing; Pestilence and Phyrexian Tower can't be activated; Reassembling
  Skeleton can't come back; Blood on the Snow, Extinction Event and Braids's Frightful Return choose for you;
  Accursed Marauder, Plaguecrafter pick your sacrifice.
- **sheoldred:** Lethal Throwdown does nothing; Otherworld Atlas, Geier Reach Sanitarium, Mikokoro, Leech Collector's
  Bloodletting unreachable; Cost of Brilliance, Painful Lesson, Peer into the Abyss always target you.
- **kefka:** Grab the Prize loses its damage; Twinned Vision flashback, Demonic Embrace, Scarlet Spider mayhem,
  Susurian Voidborn warp unreachable; Grapeshot / Brain Freeze copies, Alpharael, Stadium Tidalmage, Decorum
  Dissertation choose for you.
- **veyran:** Splatter Technique is always the wipe; Triple Triad plays everything for you; Mystic Sanctuary.
- **sauron:** Comet Storm, Inferno Titan, Fact or Fiction, Ral ch. II (no decline), Izzet Boilerworks.
- **kraum:** Smaug, Glorybringer, Fact or Fiction, Dauthi Voidwalker (one card only), The Wasp, Dark Endurance.
- **seph:** Victimize, Flickering Hound / Conjurer's Closet / Teleportation Circle blinks, Bojuka Bog, Karn's Bastion.
- **cloud:** Exalted Angel / Forum Familiar morph-disguise unreachable; Kor Outfitter, Armory Automaton, Brass Squire,
  Cid, Helitrooper, Armored Skyhunter, Silverblade Paladin, Akroma's Will choose for you; Zack Fair / Plaza of
  Heroes abilities only used by the AI's protection code.
- **godo:** Godo's Equipment pick, Everflowing Chalice (taps for 0), Ragavan dash and its exiled card, Jaxis blitz,
  Gilgamesh, Cursed Mirror, Mana Vault, Voltaic Key / Monolith untaps (verify).
- **titania:** Titania's land, Kura, Boughside Wanderers, Scout the City, Glorious Decay, Something Worth Saving,
  Malevolent Rumble, Follow the Lumarets; web-slinging / plot / foretell / disguise / landcycling cards unreachable.
- **alela:** Venser (-1 missing), Muddle the Mixture transmute, Reconstructed Thopter unearth, Ghost Quarter,
  Plan for All Outcomes, Karmic Justice, Daydream, Helping Hand, Mirage Mirror, Credit Voucher.
- **galadriel:** Kindred Discovery's creature type (check), Springleaf Drum.
- **yshtola:** Esper Sentinel paying for you; little else found.

## Suggested fix order
1. **Engine:** a yes/no for a person's "may" triggers (an optional flag on `trigger_window`, set per card);
   planeswalker attacks; the opponent-side cards routed through the existing `choices::sacrifice_creature` /
   discard prompts.
2. **A "play it another way" menu:** offer a person the alternative costs and graveyard / land abilities from the
   `*_options` hooks (as the AI's labels at first, then with their own choices).
3. **Targets for resolve-coded spells:** give each such card a target spec (tags or a `human_target` hook) so the
   person picks; fix the "does nothing" five first (Lethal Throwdown, Presumed Dead, Stargaze, Grab the Prize, Dark
   Endurance) and the forced modes (Splatter Technique, Blood on the Snow, Extinction Event).
4. **Per-card person branches** for the "chosen for you" rows, deck by deck, starting with the decks JD plays most.

None of this changes how the AI plays, so sim results stay put.
