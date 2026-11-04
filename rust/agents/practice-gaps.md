# Brief: practice-mode gaps (2026-11-04, approved by JD)

Practice mode (`commander-practice`) lets JD play his decks against the AI. Two probes in `crates/sim-core/tests/`
found where it fails the person at the seat:

- `practice_ability_probe.rs`: cards whose activated ability **isn't offered** to the person (or only when the AI's own
  value conditions say so). Example: Garth One-Eye's option hook returns nothing for a person (`impls/garth.rs`, the
  `is_human` check in `options`), and nothing in `human/` replaces it.
- `practice_choice_probe.rs`: cards whose choice **the AI makes for the person** as the card is played, cast or enters.
  Example: fetch lands are cracked at once and `engine/turn.rs::crack_fetch` picks the land with no person branch.

Each probe has a `TO_FIX` list (the gaps) and a `FALSE_ALARMS` list (entry + why). Your job: fix the entries assigned to
you below, and take each fixed entry off `TO_FIX`. The probes fail on a new gap and on a listed entry that is no longer
found, so the lists stay honest.

## Rules (read `rust/agents/RULES.md` first; these come on top)
- Work only in your worktree and branch. Commit there (no `Co-Authored-By` trailer). Don't push, don't open PRs.
- **No Python** (a hook refuses it). **No sims.** Tests only, each under the memory cap:
  `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 cargo test --release ...` (from `rust/`).
- rustfmt only files you create. Don't reformat existing files.
- Check every fix against the card's **Oracle text** (`data/scryfall_cache.json`, lowercase name key). The person must
  get the choice the rules give them, at the time the rules allow, with the AI's code still doing the AI's choices.
- Don't change what the AI does. A fix adds the person's branch (`crate::human::is_human(g, p)`) or a person-side
  ability; the AI path stays as it is.
- When an entry turns out not to be a gap (the probe set-up couldn't reach it, the choice is the opponent's, there is
  only one possible choice by the rules), move it to `FALSE_ALARMS` with a one-line reason instead of changing code.
- When a gap needs more than a person-side branch (a missing engine feature, a large rework), don't build it: leave it
  on `TO_FIX`, and say why in your report.

## How the person is asked and what they can activate
- Questions: `human::choices` (`choose`, `choose_one`, `search`, the pick-n helpers). Look at an existing person branch
  for the pattern, e.g. `impls/karn_engines.rs::fetch_land` (a search), `impls/t45_urdragon.rs` around the Ur-Dragon's
  attack trigger (a choice of permanent), `impls/t0_stella.rs` (Cursed Mirror).
- Activated abilities: `human/human.rs::abilities_of` lists Equip, `human::abilities::permanent_abilities`,
  `human::karn::perm_abilities`, compiled abilities (`dsl::human_abilities`), `human::cards::abilities`, then the card
  hooks' AI options. A person-side ability is an `abilities::ability(label, fn)` entry (see `abilities.rs`, e.g.
  `archivist`): it checks timing, tapping and mana (`tap_ok`, `pay`), then carries the effect out, asking the person.
  It should be listed whenever the rules allow the activation, not only when the AI would use it. Lands: 
  `abilities::land_abilities` and `alt::land_alts`.
- Tests: copy the style of `tests/play_urdragon_human.rs` / `tests/play_common` (a `ScriptHuman` with scripted answers,
  then assert on the game). One test per fixed card at least: it asks the person and the answer is what happens.

## Assignments (entries are `deck: card` as in the probe lists)

**Agent A: Ur-Dragon, Kefka, Sauron, Thanos**
- Ability: kefka: Birgi, God of Storytelling; kefka: Kefka, Court Mage // Kefka, Ruler of Ruin; sauron: Faerie
  Mastermind; thanos: Ranger-Captain of Eos; thanos: Thanos, the Mad Titan; urdragon: Dragon's Hoard; urdragon: Garth
  One-Eye; urdragon: Nicol Bolas, the Ravager; urdragon: Scion of the Ur-Dragon.
- Choice: kefka: Alpharael, Dreaming Acolyte; kefka: Glimpse the Unthinkable; kefka: Seize the Spoils; kefka: Twinned
  Vision; sauron: Chandra's Ignition; sauron: Foreboding Ruins; sauron: Nibelheim Aflame; sauron: Slip Out the Back;
  thanos: Cavern of Souls; urdragon: Cavern of Souls; urdragon: Decimate; urdragon: Kindred Dominance; urdragon:
  Nature's Lore; urdragon: Rampant Growth.
- Priority: Garth One-Eye and Kefka's {8} transform first (a commander and a combo piece).

**Agent B: Cloud, Karn, and the fetch lands of every deck**
- Ability: cloud: Balan, Wandering Knight; cloud: Cid, Freeflier Pilot; cloud: Evolving Wilds; cloud: Giver of Runes;
  cloud: Plaza of Heroes; cloud: Stoneforge Mystic; cloud: Terminal Moraine; cloud: Zack Fair; karn: Expedition Map;
  karn: Urza's Cave; karn: Urza's Saga; karn: Urza's Tome; galadriel: Fabled Passage.
- Choice: cloud: Akroma's Will; cloud: Forge Anew; cloud: Galadriel's Dismissal; cloud: Kor Outfitter; cloud: Winds of
  Abandon; karn: Pithing Needle; karn: Soul-Guide Lantern; karn: Spine of Ish Sah.
- **Fetch lands, every deck** (30 choice entries: Arid Mesa, Bloodstained Mire, Flooded Strand, Marsh Flats, Misty
  Rainforest, Polluted Delta, Scalding Tarn, Verdant Catacombs, Windswept Heath, Wooded Foothills, Evolving Wilds,
  Terminal Moraine, Fabled Passage). For a person: the fetch land is not cracked automatically as it enters; it is an
  ability the person activates when they choose (paying 1 life where the card says so), and the person picks the land
  from the legal ones. The AI keeps cracking at once as today. `engine/turn.rs::crack_fetch` and the line that calls it
  (`play_land_card`, the `Tag::F` check) are the place.

**Agent C: Thrasios, Sheoldred, Y'shtola, Alela, Galadriel**
- Ability: alela: Angelic Page; alela: Cabal Patriarch; alela: Credit Voucher; alela: Crystal Chimes; alela: Emry, Lurker
  of the Loch; alela: Mirage Mirror; alela: Wispdrinker Vampire; sheoldred: Gix, Yawgmoth Praetor; thrasios: Dauthi
  Voidwalker; thrasios: Resplendent Angel; yshtola: Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal.
- Choice: alela: Airlift Chaplain; alela: Pileated Provisioner; alela: Plan for All Outcomes; galadriel: River's Rebuke;
  sheoldred: Perigee Beckoner; sheoldred: Phyrexian Delver; sheoldred: Raise Dead; thrasios: Natural Order; thrasios: The
  Wasp, Winsome Avenger; yshtola: Deep Analysis.

## Done means
- Your entries are fixed (off `TO_FIX`), moved to `FALSE_ALARMS` with a reason, or left with a reason in your report.
- These pass: `cargo test --release -p sim-core --test practice_ability_probe --test practice_choice_probe`, your new
  tests, and `cargo test --release -p practice --test play_bot_games`. Then the whole suite with `--no-fail-fast`;
  report its pass/fail counts and any failure, even if you think it isn't yours.
- Report: `rust/agents/report-practice-gaps-<A|B|C>.md` (per entry: fixed / false alarm / left, and how; the tests you
  added; the suite counts). If the harness refuses the file, put it in your final reply. Final reply: 15 lines or fewer.
