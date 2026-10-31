# Card models, part 1: handoff

Brief: `rust/agents/card-models-1.md`. Branch `card-models-1`. Tests for JD's cards: `rust/crates/sim-core/tests/cards_jd_models.rs`.

## JD's Partial and Unmodeled cards (cardgen audit --mine, 10-30, before this PR)

| Deck | Card | Status before | What is missing | Oracle text |
| --- | --- | --- | --- | --- |
| yshtola | Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal | Partial | **all of its card code was dead**: legends.rs registered it as "Venat, Heart of Hydaelyn", a name not in cards.json (the list spells the full DFC name), and the override was keyed the same way. Y'shtola ran it as a 3/3 with the compiled draw trigger (no once-a-turn limit), no Sundering, no Hydaelyn | Whenever you cast a legendary spell, draw a card. This ability triggers only once each turn. Hero's Sundering — {7}, {T}: Exile target nonland permanent. Transform Venat. Activate only as a sorcery. // Indestructible. Blessing of Light — At the beginning of combat on your turn, put a +1/+1 counter on another target creature you control. Until your next turn, it gains indestructible. If that creature is legendary, draw a card. |
| seph | Glimpse the Unthinkable | Unmodeled | nothing compiled ("ten" is not a number word the parser knows); the AI never casts it | Target player mills ten cards. |
| sauron | Baleful Mastery | Partial | the alternative cost {1}{B} (an opponent draws a card) | You may pay {1}{B} rather than pay this spell's mana cost. If the {1}{B} cost was paid, an opponent draws a card. Exile target creature or planeswalker. |
| kefka | Price of Progress | Unmodeled | nothing compiled | Price of Progress deals damage to each player equal to twice the number of nonbasic lands that player controls. |
| cloud | Buster Sword | Partial | the free cast of a spell with MV <= the damage | Equipped creature gets +3/+2. Whenever equipped creature deals combat damage to a player, draw a card, then you may cast a spell from your hand with mana value less than or equal to that damage without paying its mana cost. Equip {2} |
| karn | Treasure Vault | Partial | the {X}{X}, {T}, sacrifice: X Treasures ability (no {X} activation cost in the ability language) | {T}: Add {C}. {X}{X}, {T}, Sacrifice this land: Create X Treasure tokens. |
| thanos | Gemstone Caverns | Partial | the opening-hand clause | If this card is in your opening hand and you're not the starting player, you may begin the game with Gemstone Caverns on the battlefield with a luck counter on it. If you do, exile a card from your hand. {T}: Add {C}. If Gemstone Caverns has a luck counter on it, instead add one mana of any color. |
| thanos | Orim's Chant | Partial | the kicker and "creatures can't attack"; the AI never casts it | Kicker {W}. Target player can't cast spells this turn. If this spell was kicked, creatures can't attack this turn. |
| thanos | Redirect Lightning | Partial | changing the target of an ability | As an additional cost to cast this spell, pay 5 life or pay {2}. Change the target of target spell or ability with a single target. |
| thanos | Shifting Woodland | Partial | modelled as a token copy (enter triggers fire wrongly), creature cards only | This land enters tapped unless you control a Forest. {T}: Add {G}. Delirium — {2}{G}{G}: This land becomes a copy of target permanent card in your graveyard until end of turn. Activate only if there are four or more card types among cards in your graveyard. |
| thanos | Tinder Wall | Partial | {R}, sacrifice: 2 damage to the creature it blocks | Defender. Sacrifice this creature: Add {R}{R}. {R}, Sacrifice this creature: It deals 2 damage to target creature it's blocking. |
| urdragon | Dragonlord Ojutai | Partial | body only: hexproof while untapped, the combat-damage dig | Flying. Dragonlord Ojutai has hexproof as long as it's untapped. Whenever Dragonlord Ojutai deals combat damage to a player, look at the top three cards of your library. Put one of them into your hand and the rest on the bottom of your library in any order. |
| urdragon | Nicol Bolas, the Ravager | Partial | the flip into Nicol Bolas, the Arisen | Flying. When Nicol Bolas enters, each opponent discards a card. {4}{U}{B}{R}: Exile Nicol Bolas, then return him to the battlefield transformed under his owner's control. Activate only as a sorcery. // +2: Draw two cards. −3: 10 damage to target creature or planeswalker. −4: Put target creature or planeswalker card from a graveyard onto the battlefield under your control. −12: Exile all but the bottom card of target player's library. |
| urdragon | Peter Parker's Camera | Partial | copying activated abilities; new targets | This artifact enters with three film counters on it. {2}, {T}, Remove a film counter from this artifact: Copy target activated or triggered ability you control. You may choose new targets for the copy. |

## The pool (cardgen audit, 10-30)

Partial 63 slots, Unmodeled 2, mostly Tier 6 (dack 13, rograkh-thrasios 12, sisay 9, kinnan-bonder 7, atraxa-infect 7+2
Unmodeled, kraum-tymna 6, rograkh-silas 4, prosper 2, kinnan, miirym, tergrid 1 each). Not ranked yet.

## Done

- Venat (yshtola): card code registered under the full name (`impls/legends.rs` VENAT), override re-keyed. Full.
- Glimpse the Unthinkable (now kefka, see below; the first seph-only model was reverted). Was: `fill=glimpse`; the fill plan (FILLERS, after Entomb,
  Buried Alive, Unmarked Grave) casts it, `ai::seph_fill_resolve` mills the caster ten. Full (card-audit.md row).
- Price of Progress (kefka): parser rule + `that_player_nonbasic_lands` per-player count (dsl/mod.rs
  `per_player_n`), valued in `value_of`. Full.
- Baleful Mastery (sauron): `impls/card_models.rs` altcost slot + resolve. Full.
- Orim's Chant (thanos): `combat::combat` reads `t6_stax::attack_lock`; human kicker (`human/specs.rs` Ask::Kick).
  Full (the AI never kicks: no defensive use yet).
- Treasure Vault (karn): the language's `{X}` activation cost (`Cost.x`, parse_cost; `pay_ability_cost` picks the
  largest X; `ability_options` values X and counts no mana cost at the end-of-turn window). Full. Also gives Crypt
  Rats and Fiend Artisan a real X cost (they ran X as a free 3 before).
- Dragonlord Ojutai (urdragon): card_models.rs grant_kw + combat_damage dig. Full.
- Gemstone Caverns (thanos): card_models.rs `caverns_opening`, called by `turn::setup_game` after mulligans. Full
  (a person in practice mode is not asked).
- Buster Sword (cloud): card_models.rs combat_damage (draw + free spell; `t0_stella::free_worth`/`free_cast` made
  pub(crate)); the compiled draw trigger dropped by a dsl override. Full.
- Nicol Bolas, the Ravager (urdragon): the flip (card_models.rs options hook) enters the DFC entry
  "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen", made a 7-loyalty walker by its override (loyalty dsl; new
  effect `exile_library` for the -12; cardgen now applies an override's `loyalty` to any card). **Approximate**: an
  Arisen that leaves is a planeswalker card in the graveyard, not the creature front.
- Tinder Wall (thanos): blocks slot, {R} + sacrifice for 2 damage to the attacker it blocks (AI: toughness <= 2 and
  pval >= 2, so not for a vanilla token). Full.

## Left (JD's decks), in order

1. Peter Parker's Camera (urdragon): copying activated abilities (stack::copy_trigger copies triggers only); new
   targets. Needs an "ability on the stack" object for activated abilities (engine/stack.rs).
2. Shifting Woodland (thanos): a real "becomes a copy of a permanent card until end of turn" on the land itself (now
   a token copy whose enter triggers fire wrongly; creature cards only). Needs a copy layer for a permanent.
3. Redirect Lightning (thanos): changing the target of an ability (now spells only, in the removal-response hook).
   Needs abilities with targets on the stack.

## The pool's Partial and Unmodeled cards, ranked

No report prints per-card drawn counts for outside decks (only `--swap` does, for the swapped cards), so the rank is
the number of pool decks that play the card (every card of a 99 is about equally likely to be drawn; tutor targets
more). All but a few are in Tier 6 (Thanos's tier); the rest are in Tier 3 (Atraxa, Tergrid) and Tiers 4/5
(Prosper, Kinnan, Miirym). The full notes: `cardgen audit --deck <key>`.

| Rank | Card | Decks |
| --- | --- | --- |
| 1 | City of Traitors: **done** (turn::play_land_card runs its check for every played land) | 3 (t6) |
| 2 | Wan Shi Tong, Librarian; Rain of Filth; Mnemonic Betrayal; Invasion of Ikoria // Zilortha; Hidden Strings; Flesh Duplicate | 2 (t6) |
| 2 | Magda, Brazen Outlaw | 2 (t4 prosper, t6) |
| 9 | the other 40 (one deck each) | 1 |

Only City of Traitors was modelled in this PR (JD's cards took the time); the rest is open, in this order.


## Before / after runs (void: see the coordinator section)

Baseline: origin/main's binary (copied to the scratch dir) with origin/main's cards.json / decks.json via
`COMMANDER_SIM_DATA`, `--deck X --pool all --games 2000 --seed 500000 --jobs 12`. Outputs in the scratch dir
`runs/base-<deck>.txt`.

## Gotchas

- `r.card(db, NAME)` under `if db.id(NAME).is_some()` silently skips a card whose name is spelled differently in the
  data (Venat). A double-faced card in a deck list is in cards.json under the list's spelling.
- testkit `hand`/`perm` take the card out of the library: measure library sizes after taking.

## Coordinator changes (10-30) and the state of step 5

- The ghave-deck PR (branch `ghave-deck`, worktree `/home/joseph/projects/mtg_sims/wt-ghave`) retires `seph` and
  moves cards between JD's decks. **Deck sims are on hold until the coordinator says it is in main.** The baseline
  runs started earlier (yshtola, seph, sauron, kefka on the old lists) were stopped and are void.
- Glimpse the Unthinkable moves to Kefka: the seph-only `fill=glimpse` model is reverted; the card compiles and the
  interpreter's `mill_target` / `mill_value` (dsl/mod.rs) choose and price the target. Grinding Station shares it.
- The cards coming into JD's decks: on the ghave branch's data, `cardgen audit --mine` (this branch's binary) shows
  none of Atraxa, Gifts Ungiven, Swan Song, Rhystic Study, Teleportation Circle, Displacer Kitten, Arcane Denial,
  Conjurer's Closet as not Full; their new decks' priorities are above 0 (Gifts 72, Atraxa 66 in thrasios; Kitten 40,
  Rhystic 60, Teleportation Circle in yshtola; Conjurer's Closet in karn: test
  `the_moved_flicker_engines_get_cast_by_their_new_decks`).

### To do once ghave-deck is in main

1. `git merge origin/main`; on conflicts in data/cards.json, data/decks.json or tests/fixtures/my_decks_parsed.json
   take either side and regenerate (`cardgen export`; `cardgen deck` for the fixture), never hand-merge.
2. `cardgen audit --mine`; add any card that is not Full to the table above.
3. Step 5: the baseline is origin/main (build it in a scratch copy, or `git stash`-free: a temporary worktree at
   origin/main, removed after), then this branch, each deck with a changed card on its own tiers:
   `commander-sim --deck X --pool all --games 2000 --seed 500000 --jobs 12` (one at a time, rc=0). Decks: yshtola
   (Venat), kefka (Price of Progress, Glimpse), sauron (Baleful Mastery), cloud (Buster Sword), karn (Treasure
   Vault), urdragon (Ojutai, Bolas), thanos (Orim's Chant, Gemstone Caverns, Tinder Wall; and the t6 pool's City of
   Traitors: run thanos once without and once with commit 5df4413b to separate it). Scripts that did this:
   scratch dir `run-base.sh` / `run-after.sh` (COMMANDER_SIM_DATA points a frozen binary at frozen data).
4. Update `audit/<deck>/FINDINGS.md` for each (model status, the number and why it moved).

## 10-31: no sims (JD) — what moved, for Phase 4's recalibration

JD: no sims of any kind until Phase 4's recalibration, so step 5 is cancelled. The runs that had started (yshtola,
kefka, sauron, urdragon; both arms) were stopped or are void, and no FINDINGS numbers were written. Merged origin/main
19af564c (ghave-deck) on 10-30 and regenerated the data files.

Every card or rule this branch changed, and who plays it (data/decks.json after the merge):

| Change | JD decks (their tiers) | Pool decks |
| --- | --- | --- |
| Venat, Heart of Hydaelyn // Hydaelyn (its card code was dead before) | yshtola (t5) | - |
| Glimpse the Unthinkable (compiled; AI picks who mills) | kefka (t3); seph (t3-t4) until ghave-wire retires it | - |
| Price of Progress | kefka (t3) | - |
| Baleful Mastery ({1}{B} alternative cost) | sauron (t3-t4) | - |
| Orim's Chant (attack lock when kicked; a person's kicker) | thanos (t6) | t6 dack-fayden, kraum-tymna |
| Treasure Vault ({X}{X} ability now used) | karn (t4-t5) | t4 prosper, t5 kinnan-simic |
| Dragonlord Ojutai | urdragon (t5) | - |
| Nicol Bolas, the Ravager (the flip) | urdragon (t5) | - |
| Gemstone Caverns (opening hand) | thanos (t6) | **all six t6 decks** |
| Tinder Wall | thanos (t6) | - |
| Buster Sword | cloud (t1-t3) | - |
| City of Traitors (sacrificed by any played land) | - | t6 dack-fayden, kraum-tymna, rograkh-silas |
| {X} activation cost (Crypt Rats, Fiend Artisan get a real X) | - | t2 meren (Fiend Artisan); Crypt Rats in no current list |
| "target player mills N" target choice (Grinding Station) | kefka via Glimpse | t6 rograkh-silas |
| token multiplier saturates instead of overflowing (repl::token_mult; seen at t5) | any | any |

So the tiers whose games changed: t1-t3 (Buster Sword via cloud; t2 meren), t3-t4 (sauron, kefka), t4-t5 (karn,
prosper, kinnan-simic), t5 (yshtola, urdragon), and t6 (thanos and every t6 deck: Gemstone Caverns, plus Orim's
Chant and City of Traitors).

# Card models, part 2 (10-31): handoff

Brief: `rust/agents/card-models-2.md`. Branch `card-models-2`. Tests: `rust/crates/sim-core/tests/cards_jd_models2.rs`.
No sims were run (JD: cargo tests only).

## Features added (engine / ability language)

- **Abilities as stack objects** (`engine/stack.rs`): `ability_window_ex` puts an ability on the stack with its
  `Ctx` (`target`, and `dsl_ab`: the compiled ability's card and index) and returns `AbilityRes` (resolves, its
  target as it left the stack, the copies that resolved). `dsl::activate` and `dsl::use_loyalty` use it.
  - **Single targets** (`dsl::aim`, `dsl::single_target`): a compiled activated, loyalty or triggered ability whose only
    targeted effect is removal (destroy / exile / bounce / tuck / damage to "target ...") chooses its target as it goes
    on the stack (the AI; a person still picks as it resolves). If that target is gone when it resolves, it does
    nothing (before: it picked a new target at resolution). Triggers are aimed only on the stack path (when someone
    could respond); `dsl::retarget_ok` says whether another permanent is a legal new target.
  - **Copies** (`stack::copyable`, `copy_trigger`): an ability item with `dsl_ab` can be copied like a trigger; the
    copy resolves first and chooses its targets anew.
  - **Counterspells re-aimed**: `cast_counter_spell` resolves the counter against its target as it left the stack.
  - `stack::ai_respond`: an opponent's ability aimed at q's permanent -> `t6_redirects::redirect_ability`; a
    counterspell aimed at q's important spell -> `t6_redirects::redirect_counter`; q's own compiled activated ability
    worth `CAMERA_WORTH` (3, a tutor) -> `t6_copies::camera_copy`. Redirect Lightning joined `ABILITY_ANSWERS_HAND`
    (abilities go on the stack when a player holds it).
- **A permanent becomes a copy and back** (Shifting Woodland): `t6_green2::woodland_become` / `woodland_revert`
  (`DataKey::WoodlandCopy`, `alela::set_face`), a nonland copy moves from `lands` to `perms` and back at the next turn.
- **Vanishing** (`CopyExc::vanishing`, `DataKey::Vanish`, `t6_copies::vanishing_upkeep` from `turn::upkeep`).
- **Search event for Wan Shi Tong** (`tutors::shuffle_library` -> `t6_legends2::searched`, answered in `wan_sba`).
- **Becomes-the-target** for Phantasmal Image (`t6_copies::image_targeted`: `apply_removal` unless edict/wipe, equip).
- **Delayed end-step return** (Mnemonic Betrayal: `Player::betrayed`, `t6_blue::betrayal_return`).
- **Sacrifice-a-land mana** (`mana::Source::SacLand`, Rain of Filth; `plan_pay` spends it last, like Treasure).
- New card code file `impls/card_models2.rs` (Skithiryx, Dreamtide Whale, Voidwing Hybrid, Norn's Decree).

## Cards changed (old -> new status), who plays them

| Card | Old | New | JD decks (tiers) | Pool decks |
| --- | --- | --- | --- | --- |
| Shifting Woodland | Partial | Full | thanos (t6) | t6 rograkh-thrasios |
| Redirect Lightning | Partial | Approximate (hand-written abilities have no stack target) | thanos (t6) | t6 sisay |
| Peter Parker's Camera | Partial | Approximate (hand-written activated abilities can't be copied) | urdragon (t5) | - |
| Wan Shi Tong, Librarian | Partial | Full | - | t6 kinnan-bonder, rograkh-thrasios |
| Flesh Duplicate | Partial | Full | - | t6 kinnan-bonder, rograkh-thrasios |
| Phantasmal Image | Partial | Approximate (a non-removal spell or ability aimed at it doesn't sacrifice it) | - | t6 rograkh-thrasios |
| Mnemonic Betrayal | Partial | Full | - | t6 kraum-tymna, rograkh-silas |
| Rain of Filth | Partial | Full (the AI now casts it) | - | t6 kraum-tymna, rograkh-silas |
| Skithiryx, the Blight Dragon | Partial | Full | - | t3 atraxa-infect |
| Dreamtide Whale | Partial | Full | - | t3 atraxa-infect |
| Voidwing Hybrid | Partial | Full | - | t3 atraxa-infect |
| Norn's Decree | Unmodeled | Full | - | t3 atraxa-infect |

Engine changes that touch every deck (for the recalibration): every compiled single-target removal ability now
fizzles when its target is gone by resolution (all tiers); abilities go on the stack whenever a seat holds Redirect
Lightning (thanos and t6 sisay games). So the tiers whose games changed: t3 (atraxa-infect), t5 (urdragon's Camera),
t6 (thanos and five of the six t6 decks), plus a small all-tier effect from the fizzle rule.

## Left (pool), in rank order

- Hidden Strings (2 t6): cipher (encode on a creature, cast a copy on combat damage) - an engine need.
- Invasion of Ikoria (2 t6): battles - an engine need.
- Magda, Brazen Outlaw (t4 prosper, t6 sisay): a general "becomes tapped" event (tap costs, effects).
- One deck each (Partial unless noted): Aclazotz, Agatha's Soul Cauldron, Ajani Sleeper Agent, All-Fates Stalker,
  Angel's Grace, Astral Cornucopia (Unmodeled: needs an X cast for an artifact and a per-counter rock), Bilbo's
  Gambit, Biomancer's Familiar, Deadpool, Derevi, Exhibition Tidecaller, Ganax, Get Lost, Imposter Mech, Innkeeper's
  Talent, Ixhel, Mandate of Peace, Paladin Class, Petrified Hamlet, Shang-Chi, Soul Partition, Sowing Mycospawn,
  Spider-Woman, Springheart Nantuko, Teferi Who Slows the Sunset, Tezzeret Cruel Captain, The Mycosynth Gardens,
  Training Grounds, Venerated Rotpriest, Vibrance, Volatile Stormdrake. `cardgen audit --deck <key>` has the notes.
- Redirect Lightning / Camera to Full: hand-written card code that calls `ability_window` would have to move to
  `ability_window_ex` (with its target) to be redirectable or copyable.

## Gotchas

- `perl -0pi -e 's|...|...|'` with `|` inside the pattern (closures `|e|`) silently mangles the file: use another
  delimiter or the Edit tool.
- `cast_card` doesn't pay the spell's cost (the caller does).
- Eternal Witness is 2/2 in data/cards.json (Oracle 2/1): not touched.

# Card models, part 3 (10-31): handoff

Brief: `rust/agents/card-models-3.md`. Branch `card-models-3`. Tests: `rust/crates/sim-core/tests/cards_models3.rs`.
Card code: `rust/crates/sim-core/src/impls/card_models3.rs`. No sims were run (cargo tests only).

## Features added (engine / general)

- **Cipher** (`card_models3::cipher_encode` / `cipher_hit`): a cipher spell's resolve ends with `cipher_encode`, which
  exiles it encoded on the creature likeliest to connect (`DataKey::Cipher` on the creature); `cardcode::
  combat_damage_cards` calls `cipher_hit`, which casts a copy free (`cast::cast_spell_copy`, `Ctx::mode` =
  `cipher_copy` so the copy is not encoded again).
- **Battles** (`card_models3::battle_enter`, `battles_for`, `battle_damage`, `protector`; `DataKey::Defense`,
  `DataKey::Protector`): a battle enters as a noncreature permanent with defense counters and a protector (the
  opponent with the fewest creatures; re-chosen if that player leaves). `combat::walker_attacks` sends unblocked
  attackers at the Sieges the defending player protects (before planeswalkers); `resolve_blocks` removes defense; at 0
  it is defeated, exiled, and (Invasion of Ikoria) cast transformed free as Zilortha, Apex of Ikoria (8/8 reach;
  `zilortha_unblocked`: non-Humans assign combat damage as though unblocked).
- **Becomes tapped** (`card_models3::newly_tapped` / `tapped_now`, `DataKey::TapArmed`): a watcher reads, at state
  checks (`hooks::fire_sba`), which of its permanents it saw untapped are tapped now; the mana and attack events
  report at once and disarm. A permanent that entered tapped never triggers it.
- **Becomes the target of a spell** (`card_models3::spell_targeted`, from `cast::cast_card` as any spell is cast).
- **Card timing restriction** (`card_models3::cast_restricted`, read by `cast::castable`): Mandate of Peace.
- **Removal target kind `cep`** (creature, enchantment or planeswalker): engine/removal.rs, human/legal.rs, alela.rs,
  cardgen autotag.
- **Proliferate adds charge counters on nonland permanents** (common::proliferate): Astral Cornucopia, Otherworld
  Atlas (sheoldred plays the Atlas but has no proliferate).
- **Angel's Grace's damage floor** (life::lose_life) and **can't lose against a drain/damage win** (zones::last_chance).
- **Counter doubling by the actor** (repl::counters / vorinclex_mods): Innkeeper's Talent level 3.
- Hand-written walker (common::walker): Ajani, Sleeper Agent; its emblem in rules::emblem_cast.

## Cards changed (old -> new status), who plays them

| Card | Old | New | Decks (tiers) |
| --- | --- | --- | --- |
| Hidden Strings | Partial | Full (cipher) | t6 kinnan-bonder, rograkh-thrasios |
| Invasion of Ikoria // Zilortha | Partial | Approximate (battle + Zilortha; see note) | t6 kinnan-bonder, rograkh-thrasios |
| Magda, Brazen Outlaw | Partial | Full (becomes-tapped event) | t4 prosper, t6 sisay |
| Astral Cornucopia | Unmodeled | Full (X cast as a hand option, charge counters, proliferate) | t3 atraxa-infect |
| Angel's Grace | Partial | Approximate (an opponent's alternate win still ends the game: ai/mod.rs `win` is off limits) | t6 dack-fayden |
| Paladin Class | Partial | Full (level 2 anthem, level-gated; the compiled anthem, on from level 1, is dropped) | t6 dack-fayden |
| Mandate of Peace | Partial | Full (only during combat; the AI casts it against a lethal / half-life attack) | t6 dack-fayden |
| Get Lost | Partial | Approximate (any of the three targets; no Map tokens) | t6 dack-fayden |
| Innkeeper's Talent | Partial | Full (combat counter, levels 2 and 3) | t3 atraxa-infect |
| Venerated Rotpriest | Partial | Full | t3 atraxa-infect |
| Ajani, Sleeper Agent | Partial | Full | t3 atraxa-infect |
| Ixhel, Scion of Atraxa | Partial | Full | t3 atraxa-infect |
| Ganax, Astral Hunter | Partial | Full (status only: Choose a Background does nothing in the 99) | t5 miirym |

No JD deck plays any of these cards. Tiers whose games changed: t3 (atraxa-infect: six cards, charge-counter
proliferate, Innkeeper doubling), t4 (prosper's Magda), t5 (miirym: status only, no behaviour change), t6 (sisay's
Magda, dack-fayden's four, kinnan-bonder and rograkh-thrasios's Hidden Strings and Ikoria). All tiers: a spell aimed
at a creature now checks for Venerated Rotpriest (no effect without it); `castable` checks the Mandate restriction.

## Left (pool), in rank order

- Single-deck, still Partial: Aclazotz (t3 tergrid: the attack discard/draw, the land-discard Bat, the dies ->
  Temple of the Dead land and back), Agatha's Soul Cauldron, All-Fates Stalker (warp from exile), Bilbo's Gambit
  (gift), Biomancer's Familiar / Training Grounds (the discount on hand-written creature abilities), Deadpool (text
  box exchange), Derevi (command-zone ability), Exhibition Tidecaller (mana-spent total), Imposter Mech (vehicles),
  Petrified Hamlet (name ban), Shang-Chi (restricted mana), Soul Partition (play from exile, tax), Sowing Mycospawn
  (kicker), Spider-Woman (Food/Clue enter tapped), Springheart Nantuko (bestow), Teferi Who Slows the Sunset (-7
  emblem), Tezzeret Cruel Captain (-7 emblem), The Mycosynth Gardens (copy an artifact), Vibrance (hybrid cost on a
  normal cast), Volatile Stormdrake (hexproof from abilities only).
- Step 2 of the brief (Redirect Lightning / Peter Parker's Camera: hand-written single-target abilities through
  `ability_window_ex`) was not reached.

## Gotchas

- `cast::cast_card` doesn't pay; `cast_spell_copy` removes the resolved copy from the graveyard/exile it went to.
- A battle is a permanent whose card is a sorcery in the data (types "S"): `is_creature` is false, it sits in `perms`.
- Hooks fire only for hooked permanents: `newly_tapped` and `innkeeper_doublings` bail out when `g.hooks` is empty.
- The 10-31 rules file (rust/agents/RULES.md) is the old porting one; the brief's checks are the ones that apply.
