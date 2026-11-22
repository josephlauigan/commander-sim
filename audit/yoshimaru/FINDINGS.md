# Yoshimaru and Tymna (key `yoshimaru`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/yoshimaru-tymna-blink.md` (Yoshimaru, Ever Faithful and Tymna the Weaver, plain Partner, white-black blink and
enter-effect midrange; 100 cards). **Tiers:** t4 and t5 (`data/my_decks.toml`). **No sims yet.**
The findings of the retired Tymna/Thrasios Doomsday deck are in `audit/retired/thrasios/`.

## What the simulator cannot do yet
- Every card is Full or Full-auto except Delney, Streetwise Lookout (Approximate: the extra instance for leaves that run at once or inline in the
  engine is left out; see her note in `data/audit.toml`). Full-auto means compiled from Oracle text, not hand-written card code: Stroke of Midnight and
  Sword of Hearth and Home. **Steelshaper's Gift is rated Full-auto but is wrong** (defect 9 below).
- `data/plans/yoshimaru.toml` is provisional: priorities come from the guide, unmeasured; the keep rule is the generic lands 2 to 5.
  It leaves out the Archangel of Thune + Walking Ballista line and the guide's Restoration Angel + Felidar Guardian line (Spellbook
  1090-2781, listed in `KNOWN_GAPS` in `tests/spellbook.rs`).

## 2026-11-21: the Equipment package (22 swaps, not simulated)
What changed and why is in `decklists/CHANGELOG.md`; the guide's "The Equipment package" describes the cards. **Nothing has been simulated:
every statement below is a claim to test, not a result.**

### What to check in the first runs
- **Where the AI puts Equipment.** `data/plans/yoshimaru.toml` has Cloud's Equipment rows (priorities, `hand:equipment` conditions) but not Cloud's
  `[equip] holder = "commander"`, which would load the Swords on Yoshimaru or Tymna. The guide wants fliers (Restoration Angel, Flickerwisp, Archangel of Thune,
  Sunpearl Kirin) and only Swiftfoot Boots on Tymna, and Sword of Hearth and Home never on Walking Ballista. Look at the traces for who wears what and
  whether Tymna ends up with +power (Delney's doubling and evasion switch off at power 3).
- **Flawless Maneuver in response to our own Wrath of God.** The guide's one-sided wipe needs it; the generic policy may not do it.
- **Court of Ardenvale and Walking Ballista.** Court returns a permanent card of mana value 3 or less to the battlefield while we are the monarch; Ballista
  would enter with X = 0 and die. Check what the AI returns.
- **Mana.** JD's report (11/21) was 3 to 4 mana for most of the game with the old list. The outside goldfish estimate for 35 lands, 6 rocks and Wayfarer's Bauble is a mean of
  4.1 mana on turn 4, 4.8 on turn 5, 5.3 on turn 6 and 6.3 on turn 8 (on the play; not the simulator). Compare with the simulator's turn-by-turn mana and with
  `--goldfish yoshimaru,t5`.
- **Delney conflicts.** Cloud, Stoneforge Mystic, Kor Outfitter and Puresteel Paladin have power 2 or less, so Delney doubles their enter and combat
  triggers (Zack Fair has no triggered ability); any Sword on them removes that. A Sword on Cloud (equipped, so his own doubling applies) is the intended exception.
- **Protection from white.** Sword of Hearth and Home's carrier cannot be targeted by our white spells and abilities (Ephemerate, Restoration Angel, Felidar Guardian,
  Flickerwisp, Teleportation Circle, Eerie Interlude, Giver of Runes, Heliod's lifelink, Zack Fair's sacrifice, Kor Outfitter's and Ardenn's attach); Equipment's own
  equip abilities, Puresteel Paladin's equip {0}, Forge Anew and Mithril Coat's attach trigger still work.

### Costs accepted by JD with the list (11/21)
Combo stops are down to Silence and Ranger-Captain of Eos; answers to noncreature permanents are Stroke of Midnight, Skyclave Apparition and Witch Enchanter; Line C is
a variant of Line B or A (Sorin is out); Line D's payoffs are Elas il-Kor, Haliya, Vito and Archangel of Thune; Wrath of God cannot touch opposing indestructible
creatures; Stroke of Midnight gives the opponent a 1/1 Human; the deck's black is oversupplied (no card costs {B}{B}).

### Open engine defects (found by the interaction tests; NOT fixed, NOT accepted by JD)
Twelve tests of the package's card interactions fail on purpose against the engine (108 tests written, 96 pass). They are not in this change: the test file
`cards_yoshimaru_equipment.rs` lands with the engine-fix change, after JD's decision, so main never carries ignored tests. Until then no sim of the Yoshimaru
deck means anything for the cards below. Each line: the card, what the engine does, what the Oracle text says.
1. Court of Ardenvale never returns a land card (the filter drops lands); a land card is a permanent card with mana value 0.
2. Zack Fair's sacrifice moves every Equipment attached to him; the text attaches "an Equipment".
3. Zack Fair's sacrifice may target a creature with protection from white (a Sword of Hearth and Home carrier), though Zack is white.
4. Kor Outfitter's enter trigger attaches an Equipment to the carrier with protection from white, though Kor Outfitter is white.
5. Ardenn's beginning-of-combat attach does the same.
6. Restoration Angel's enter trigger blinks the protected carrier, though the Angel is white.
7. Felidar Guardian's enter trigger blinks the protected carrier, though Felidar is white.
8. The AI offers Ephemerate on the protected carrier, though Ephemerate is white (Flickerwisp is correct).
9. **Steelshaper's Gift is compiled with an empty search filter**: it puts the best card of any type into the hand (it found Sol Ring) instead of only an
   Equipment card, so it is a {W} tutor for anything. It is in the list.
10. Stroke of Midnight's token is a plain 1/1 of the controller's colour, not a white Human.
11. Wrath of God has no "can't be regenerated" tag, so a regeneration shield saves a creature.
12. Metalcraft counts only artifact permanents; Treasure tokens (kept as a counter on the player) are not counted, and Lotho makes Treasures.
Opus's triage: fix 9 and 3 to 8 (one shared protection check, `protected_from`, at six call sites; Outfitter and Ardenn need a source-aware variant of
`best_holder`) before any sim, and 1, 2, 10, 11 and 12 in the same change; none needs to be accepted as a limit. `best_holder`, `zack_sacrifice`, `metalcraft`
and `resto_etb` are shared with the Cloud deck, so Cloud's numbers may shift.
