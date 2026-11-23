# Kyodai, Soul of Kamigawa (key `kyodai`, `thanos` until 2026-11-22): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/cali_commander/kyodai-artifact-combo.md`, built 2026-11-13 for Cali Commander (1v1, 30 life) from
the previous Thanos list, the Aragorn deck and cards taken from the other decks. Routes: Basalt Monolith + Rings of
Brighthearth, Staff of Domination + Faeburrow Elder, Tinker into Blightsteel Colossus (Sneak Attack as a second way to
put it in), and Thassa's Oracle with Demonic Consultation or Tainted Pact (Underworld Breach with Brain Freeze as a storm
line). **No sims yet.** The findings of the artifacts-and-Dragons list are in `FINDINGS-artifacts-and-dragons-list.md`,
those of the cEDH list in `FINDINGS-old-list.md`.

## 2026-11-22: Kyodai and Leyline of the Guildpact modelled (Full); hybrid costs fixed; guide rules errors corrected (JD's request, not simulated)
JD asked for both new cards to be fully modelled, the two `mulligan_hints` tests that broke on origin/main to be fixed, the guide's rules errors to be fixed, and Leyline
to be a proxy. Four workers built it on separate branches (Kyodai, Leyline's static abilities and opening-hand clause, hybrid costs, multi-mana lands under Leyline), an Opus
design consult came first, and the full cargo suite passes; no sims were run (the engine has no Cali mode), so nothing here is measured.
**Kyodai** (`impls/kyodai.rs`): the enter trigger is a stack trigger (the table can respond; Elesh Norn blanks it, Panharmonicon doubles it): another target permanent that is
a legal target (no hexproof or shroud, not protected from Kyodai's colours, ward paid) gains indestructible while Kyodai stays on the battlefield under the same controller. A blink or recast is a new object
(the old target loses it, the new trigger makes its own). A person is asked for the target; the AI picks its own engine, then its best permanent, then a land, never an opponent's permanent.
Flash, flying and the {W}{U}{B}{R}{G} +5/+5 pump are compiled and tested. **Leyline** (`engine/chars.rs`, `engine/mana.rs`, `impls/card_models.rs`): nonland permanents are all five colours (layer 5,
before copy exceptions; devotion is unchanged; lands and manlands are excluded), lands are every basic land type (layer 4), so each taps for any one colour; a multi-mana land such as Ancient Tomb gets
`Shape::Alt` (its own {C}{C} with the 2 damage, or one mana of any colour with none, never both); the opening-hand clause (the AI always begins with it; a person is asked yes or no after keeping).
**Hybrid costs** (`engine/hybrid.rs`, the export's `hybrid` field): a card's colours are its pips plus its hybrid pairs, and `cost_of` pays the other colour of a hybrid symbol when the printed first colour cannot be paid,
so Leyline is paid by GGGG, WUBR or any mix. Behaviour is unchanged for non-hybrid cards and for a hybrid card whose printed colours are payable. 31 catalog cards have hybrid symbols; the ones in decks are Leyline (thanos) and,
in the opponent pools, Blade Historian (t3), Deathrite Shaman and Fiend Artisan (t2), Privileged Position and Strength of the Harvest (t2), Steel of the Godhead (t5), Yorion (t2) and Vibrance (t6); only the t3 and t5 ones fall in
JD's tiers, so baselines could shift slightly (nothing has been re-run). Vibrance now reads the colours it was paid with.
**Limits for JD (each is in the card's note):** Blood Moon always beats Leyline on a nonbasic land (the engine has no timestamps; Blood Moon is in no t5 deck); Karoos and Lotus Field get no any-colour alternative under Leyline;
a restricted land (Cavern of Souls, Mishra's Workshop) keeps its restriction on the any-colour mana; a phased-out Kyodai turns the grant off and it returns when he phases back in (rules: it ends for good), a control
change away and back resumes it, and losing Kyodai's abilities stops it (rules: it continues); Ward on an opponent's target is paid by a person only; a hybrid card cast by a path that prices from the printed pips (graveyard casts and some AI look-ahead) is paid at the printed colours;
devotion-style pip counts count a hybrid symbol once; {2/W} and Phyrexian symbols are untouched. The AI has no line for Staff of Domination + Faeburrow Elder or for holding a flash Kyodai to answer removal;
`data/plans/kyodai.toml` only values Leyline (high when the Elder is in hand or out).
**Engine drift to disclose before the next sims** (nothing re-run): hybrid cards now count for both colours in devotion (CR 700.5), so the sythis pool deck's Privileged Position adds +3 white devotion for Heliod and the zur pool deck's Steel of the Godhead adds +2 blue devotion for Thassa's Oracle (zur is a t5 seat, so it touches the baseline of every deck that faces t5); hybrid payment and colours change for Blade Historian (aurelia, t3) and Steel of the Godhead (zur, t5); Iroas with an Altar of the Pantheon now counts the Altar once, not twice; the person's Pyroblast-style target filter and O-Ring read current permanent colours. Re-measure the t3 and t5 baselines before any deck decision.
**Guide corrections** (the guide was wrong; the 11-22 entry below repeats two of them): Urza's {5} plays the card free, so a Ballista found that way has X=0 (use the Staff to draw it); Collector Ouphe and Null Rod stop Lightning
Greaves' equip, so under them Blightsteel attacks a turn later; a Sculpting Steel copy of Chrome Mox may imprint its own card (the 11-22 entry said it could not); The One Ring keeps draining after you stop drawing; Swords to Plowshares only targets
creatures; Mox Opal can work on turn 1 with free rocks; an Oracle in the graveyard can return through Underworld Breach; Esper Sentinel's tax is X (its power). Also fixed in the new text: Ephemerate on Kyodai does not keep the first target
protected, Akroma's Will's second mode gives protection from each colour, Leyline affects nonland permanents only, three (not four) one-mana green dorks, Crucible does not give an extra land drop, Silence costs {W}.
**Tests touched:** `land_hint` and `mulligan_hints` (practice) use the Ur-Dragon list; the Loran test finds the seat by "Kyodai"; Kyodai is off every allow list and `TO_FIX`; `enters_inventory.txt` has a Kyodai row. Added: `cards_kyodai`, `cards_leyline_guildpact`,
`cards_leyline_multimana`, `cards_hybrid_cost` and a practice session test for the opening-hand prompt.

## 2026-11-22: Thanos => Kyodai as commander; Windborn Muse => Leyline of the Guildpact; mana base rebuilt (JD's choice, not simulated)
A commander change, a one-card fix for the Staff of Domination and Faeburrow Elder route, and a land-for-land mana-base rebuild. Nothing was
simulated: the engine has no Cali mode, and it has no hand-written card code for Kyodai or Leyline (`cardgen find --full` after `cardgen export`: Kyodai
is Partial, its flash, flying and +5/+5 compiled from the Oracle text but its enter trigger not; Leyline is Unmodeled, all three abilities unparsed, and its exported
cost is `GGBR`, not the hybrid `{G/W}{G/U}{B/G}{R/G}`; JD took the plan with no engine work). The deck key was renamed from `thanos` to `kyodai` the same day (guide, plan, printings, spellbook, ownership and this folder moved with it); `data/my_decks.toml` names the deck "Kyodai" and its commander
Kyodai, Soul of Kamigawa. The engine still plays the old commander's card code (impls/t6_thanos.rs) for any deck that seats Thanos.
**Kyodai** ({3}{W}, 3/3 flash flier; enter: another permanent gains indestructible while he stays; {W}{U}{B}{R}{G}: +5/+5) takes Thanos's place. Why
JD chose it: one white source and three generic cast him, so Sol Ring, Mana Crypt, Mana Vault and Ancient Tomb help (they could not pay for
Thanos's {R}{W}{B}); flash lets him arrive in response to removal or at the opponent's end step; he still turns on Deflecting Swat, Jeska's Will
and Akroma's Will; and with infinite coloured mana the pump is lethal in the air. What it costs: the power-up wipe (it destroyed each other creature of one mana-value parity, yours too, and the deck has few), deathtouch and
lifelink, and Thanos's R, W and B on the battlefield (Kyodai is white only; WUBRG is his identity). The commander arrives about a turn later on
smooth hands (turn 3 to 4 against turn 2 to 3), and earlier on hands with colourless fast mana or missing red or black.
**Leyline of the Guildpact** ({G/W}{G/U}{B/G}{R/G}, an enchantment, free from the opening hand) takes Windborn Muse's slot. Why: Thanos was the deck's
only black permanent, and without a fifth colour among permanents the Elder loop reaches four colours and nets nothing; with Leyline every nonland
permanent is all colours, so the Elder alone taps for WUBRG. Route A2 now needs Leyline. It also makes lands every basic type (Ancient Tomb, Strip
Mine and Urza's Saga tap for any colour, with no damage from that tap). What it costs: your nonland permanents become blue and red (lands stay colourless), so Pyroblast, Red
Elemental Blast and Hydroblast destroy them (Heroic Intervention is the answer and stays in; Kyodai's indestructible covers one permanent and gives no
hexproof); it does nothing alone; and the Muse's attacker tax and flying blocker go (Kyodai covers part of that). Options B2 (Baleful Strix + Cursed
Mirror), B3 (Mayhem Devil) and B4 (Painter's Servant) were offered; JD chose Leyline. Alternates for the cut were Heroic Intervention (kept for the blast
cards) and Hangarback Walker (kept: it feeds Mox Opal and Tinker).
**Mana base**, land for land (20 out, 20 in; 27 lands, 3 colourless, unchanged): out are the seven painlands, Spirebluff Canal, Sulfur Falls, Sunpetal
Grove, Woodland Cemetery, Haunted Ridge, Stormcarved Coast, Exotic Orchard, Path of Ancestry, Plaza of Heroes, Urborg, Blood Crypt, Overgrown Tomb and
Temple Garden; in are the ten original duals and ten fetchlands, all proxies. Hallowed Fountain, Steam Vents, Watery Grave, Command Tower, Ancient Tomb, Strip
Mine and Urza's Saga stay. Coloured sources among the lands, fetchlands counted as any colour: blue 18 (was 14), white 16 (11), black 16 (12), red 16
(12), green 15 (10). Why: all original duals are untapped and painless, the painlands cost 1 life on each coloured tap and the slow lands often enter tapped,
Plaza made coloured mana only for legendary spells or in the colours of your legendary permanents, Path's scry needs a Dragon or Spirit, Exotic Orchard depends on one opponent's lands, and Urborg is one colour
that fetchlands cannot find. The list has no basic lands, so each fetchland finds an original dual or shockland by its two land types; every pair of colours
has one, so a fetchland is a five-colour land for 1 life (13 targets for 10 fetchlands). Prismatic Vista would find nothing and stays out. Fetch an original
dual, not a shockland (2 more life to enter it untapped). Do not crack a fetchland after Vampiric, Mystical, Enlightened or Worldly Tutor. Life at 30: an average game costs 2 to 3 fetchland
payments, Mana Crypt about 1.5 a turn, The One Ring's burden counters and Ancient Tomb's 2 a tap; the original duals cost nothing, so the base costs less life
than the painland base. The land count stays 27 (26 is arguable with 20 rocks and dorks, untested).
Counts after: creatures 15 (16), enchantments 5 (4), Game Changers unchanged (Kyodai and Leyline are not Game Changers); 15 rocks and rituals, 5 dorks and 25
artifacts unchanged. An Opus consult picked the cut and the land list and found the Leyline blast exposure. Checks: Kyodai, Leyline and the 20 lands are
Commander-legal and fit WUBRG (Scryfall and `cardgen find`); the fetchlands and duals are Full (mana only).
**Open:** (1) superseded the same day: JD then asked for both cards to be fully modelled; see the next entry. (2) `data/ownership.toml` records Kyodai as have (JD bought it; the list JD sent on 2026-11-22 also makes Rhystic Study and Watery Grave proxies in this deck) and Leyline as a proxy, and the 20 lands as proxies; the replaced lands and Thanos and Windborn Muse are in `[spare]`.
(3) `data/printings/kyodai.txt` now holds JD's 100 printings (checked against Scryfall: every set and number exists and matches its card; six are Japanese-language Mystical Archive printings). (4) `data/spellbook/kyodai.json` was re-fetched on 2026-11-23 for the new list (`scripts/spellbook-fetch.sh kyodai`; the commander is Kyodai): the five included
combos are unchanged and the "almost included" list went from 173 to 174; nothing in it was reviewed. (5) Cali ban list: the guide's list (Black Lotus, the five Moxen and the rest) does not include any of the 20 lands, Kyodai or Leyline;
whether proxies are allowed at JD's events is still open. (6) Local blast density (Pyroblast, Red Elemental Blast, Hydroblast) decides how much Leyline
costs; JD knows his table.

## 2026-11-22: Thought Vessel, Fiery Emancipation, Nibelheim Aflame => Mox Opal, Tezzeret, Cruel Captain, Chrome Mox (JD's choice, not simulated)
An artifact package of three pairs, none of them measured (the engine has no Cali mode, so no paired test could judge it).
**Mox Opal** ({0}, legendary, metalcraft: any colour with three or more artifacts) takes Thought Vessel's slot, rock for rock: Thought
Vessel made only {C}, which the deck has many other sources of (Sol Ring, Mana Crypt, Mana Vault, Basalt Monolith, Mind Stone, the Mightstone
and Weakstone, Ancient Tomb, Strip Mine, Urza's Saga, the painlands). An Opus consult picked Thought Vessel because JD did not want Fellwar
Stone cut; the runner-up was Mind Stone, which can cash in for a card. The artifact count is unchanged by this pair. **Tezzeret, Cruel
Captain** ({3}) takes Fiery Emancipation's slot: his 0 untaps the Basalt Monolith (twice with Rings of Brighthearth) and his -3 tutors an
artifact of mana value 1 or less (Sol Ring, Mana Crypt, Mana Vault, Lotus Petal, the Moxen, Esper Sentinel, and Walking Ballista or Hangarback Walker at
mana value 0 in the library). **Chrome Mox** ({0}, a Game Changer) takes Nibelheim Aflame's slot (JD preferred it to Grim Monolith): it
imprints a nonartifact, nonland card, so it costs a card. Fiery Emancipation was the card the earlier notes named as the weakest; Nibelheim Aflame is the other {R}{R} card that left. What it costs: two red cards leave the deck (Thanos stays the red commander, with Chaos Warp, Deflecting Swat, Jeska's
Will, Gamble, Underworld Breach, Sneak Attack and Ragavan still red); the Moxen's colours need a card to imprint, and Mox Opal is dead until
three artifacts are out (the deck has 25 with the five artifact creatures, 24 before). Counts after: 15 rocks and rituals (14), enchantments
4 (5), sorceries 7 (8), one planeswalker, Game Changers +1; lands and land colour sources unchanged. A fourth pair (Windborn Muse => Mishra's
Bauble) was left out (JD took three). Engine status: Mox Opal, Tezzeret and Chrome Mox are Full (`cardgen find --full`). The plan file's
turn<=4 rock priority now lists Mox Opal and Chrome Mox in place of Thought Vessel; Tezzeret has no plan line yet. Sculpting Steel must not copy
Mox Opal (legendary) or Chrome Mox (the copy has nothing imprinted). Staff of Domination stays (a proxy, printing cmr 343). Route A2 is unchanged.
`data/spellbook/kyodai.json` was re-fetched on 2026-11-23 (after the two entries that used Fiery Emancipation had been removed): the five
included combos are unchanged, so the new artifacts add none. Three combos are now "almost included": Tidespout Tyrant + Mox Opal and
Hullbreaker Horror (with any {0} permanent, which Mox Opal and Chrome Mox now supply; both one card away, no result listed), and Time Vault + Tezzeret, Cruel Captain (infinite turns; Time Vault is not Commander-legal,
so Spellbook marks the combo illegal; not in the list).

## 2026-11-15: Triple Triad => Gifts Ungiven (JD's choice, not simulated)
Gifts Ungiven ({3}{U} instant, a Game Changer; the list now has 17 by the cache's flags) takes Triple Triad's slot. Why:
an instant tutor at the opponent's end step with the combo pieces already in the deck. Its piles that hold for every split
(the opponent chooses two of the four for the graveyard) each need one piece already in hand: Thassa's Oracle in hand with
Demonic Consultation, Tainted Pact, Underworld Breach and Demonic Tutor (five of the six splits leave an exile half; if
the opponent bins Consultation and Pact you hold Breach and Demonic Tutor, and Breach has to escape an exile half from the
graveyard, a much bigger turn); Underworld Breach in hand with Oracle and Consultation (both go to the graveyard, then
Breach escapes them). Tinker + Blightsteel + Oracle + Consultation does not hold, and a pile of the Basalt-route pieces
does not either (the opponent bins the Monolith and no tutor finds it in the graveyard). So it adds redundancy and the
Breach pair trick, not a new route, on top of eight 1-3 mana tutors (Demonic, Vampiric, Mystical, Enlightened, Worldly,
Grim, Gamble, Diabolic Intent) and Tinker. What it costs: Triple Triad ({R}{R}{R}) flips few free spells at an average
nonland mana value of 2.53 before the swap, 2.50 after (it went to the Ur-Dragon, where the Dragons cost 5 to 7). Not measured. An Opus consult rated Gifts the weaker of its two
adds for this deck (Mindbreak Trap went to Y'shtola); Fiery Emancipation, the other {R}{R}{R} six-drop, was its next cut (it went out on 2026-11-22, above). In
the simulator the AI builds Gifts piles from tutor value and graveyard worth (`engine/tutors.rs` `pile_tutor`), not from
the combo, and the Oracle line, Tinker and Sneak Attack have no card code or plan lines for `kyodai`, so a sim number
would understate it. Check the blue count when a run exists: 12 land sources make {U}, 14 counting Exotic Orchard (blue
only if an opponent's land makes it) and Plaza of Heroes (which cannot pay for Gifts).

## 2026-11-15: Elesh Norn => Sculpting Steel (JD's choice, not simulated)
Sculpting Steel ({3} artifact, colourless, Commander-legal) takes Elesh Norn, Mother of Machines' slot. Why JD chose it:
a flexible artifact that copies the best artifact already on the battlefield. In this list: a second Rings of Brighthearth
(two Rings let one Basalt Monolith untap be copied twice: {7} spent for three taps of {C}{C}{C}, two extra mana per loop
instead of one), a second Blightsteel Colossus (an artifact creature) after Tinker or Sneak Attack as insurance against a
blocker or removal, or a second colourless rock for Thanos's power-up. What it costs: it needs an artifact already out, so
it finds no combo piece, and it is a plain artifact on an empty board. Elesh Norn doubled the enter triggers of Urza,
Meteor Golem, Primeval Titan and The Mightstone and Weakstone, and stopped the opponent's enter triggers (a defence
against an opposing Thassa's Oracle), so those go too. An Opus consult (Dockside Extortionist, not Steel, was the question
then) ranked Mana Drain, Chrome Mox and Imperial Seal first and advised cutting Fiery Emancipation, the weakest card,
rather than Elesh Norn; Fiery Emancipation went out on 2026-11-22 (above). The engine models Sculpting Steel (Full: it enters as a copy of the best artifact on the
battlefield, `card_overrides.toml`), but `kyodai` has no plan line for it and Tinker, Sneak Attack and the Rings loop are
not modelled, so a sim number would not reflect the Rings and Blightsteel lines. Not measured. Commander
Spellbook lists no combo with Sculpting Steel for this list.

## What the simulator cannot do yet

- **No Cali mode:** the engine plays 4-player pods at 40 life. Cali is 1v1 at 30 life, and the player on the play skips
  their first draw.
- **No Cali ban list:** `cardgen deck` rejects Mana Crypt, Tinker and Primeval Titan as banned in Commander; Cali allows
  them. This list was written by hand and does not pass `cardgen deck` until that is changed.
- **Cards not modelled:** Tinker, Sneak Attack, Mana Crypt, Staff of Domination and the Oracle line (Thassa's Oracle
  with Demonic Consultation or Tainted Pact) need card code or catalogue combos; Haunted Ridge and An Offer You Can't
  Refuse are new.
- **Plan file:** `data/plans/kyodai.toml` still describes the previous list and is rewritten with the engine work.

## Open questions

- Staff of Domination, Watery Grave and Woodland Cemetery are binder-only copies; if any is not in the binder it is a
  proxy.
- Whether proxies are allowed at the Cali events JD plays.
