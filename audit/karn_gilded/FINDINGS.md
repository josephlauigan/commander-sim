# Karn, Gilded Guardian (key `karn_gilded`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/karn-gilded-guardian.md`, registered 2026-11-24 (Karn, Gilded Guardian, WUBRG; 64 spells and 35 lands, no basics;
15 Game Changers, Bracket 4). **Tiers:** t4 and t5 (`data/my_decks.toml`). **No sims yet.** The deck replaces the Y'shtola and Yoshimaru/Tymna
decks, whose findings are in `audit/retired/yshtola/` and `audit/retired/yoshimaru/`.

## Plan (what the deck is meant to do)
- **Engine:** Karn draws a card for each colour among the other artifacts we control, up to five; coloured artifacts (14 in the list; the
  Transguild Courier and Infinite Guideline Station are five colours alone) and blink effects (Ephemerate, Cloudshift, Restoration Angel,
  Flickerwisp, Teleportation Circle, the harnessed The Mind Stone) enter him again; Gandalf the White and Elesh Norn,
  Mother of Machines add triggers.
- **Wins:** Thassa's Oracle with Demonic Consultation or Doomsday; Exquisite Blood with Sanguine Bond or Vito (Haliya and The Wind Crystal
  as the starters); Exsanguinate off Cabal Coffers and Urborg; combat with Karn and Buster Sword or Sword of Feast and Famine.

## Strengths (by design, not measured)
- Many ways to draw and to find a line: seven tutors and the Karn draw.
- Counter and stop suite for the win turn: Force of Will, Counterspell, Silence, Teferi's Protection, Teferi, Time Raveler, Ranger-Captain of Eos.
- Four independent finishes.

## Weaknesses and limits (by design and from the engine work; none measured)
- **Karn is slow and fragile:** a real cost of 5 to 10 and {2} more per recast; an opponent's Flickerwisp or any removal sends him to the command zone.
- **Hand size at our own end step:** Teleportation Circle and the harnessed The Mind Stone return Karn in our end step, he draws before cleanup and
  the excess is discarded unless Thought Vessel or Reliquary Tower is out (they give no maximum hand size).
- **The draw is mandatory:** the library can empty; Thassa's Oracle is the outlet, `death_kind decked` is the stat to watch.
- **Blood Moon effects:** no basic lands.
- **Mana:** nine shocks, five fetches, three pain lands, City of Brass and Mana Confluence cost life next to Necropotence and Doomsday; red and green sources are thin (17 and 16).
- **Engine limits (JD-accepted or open):** The Mind Stone and Shorikai are Approximate (the stone never picks Infinite Guideline Station or The One
  Ring; Crew is not modelled); Teferi's Protection is Approximate (JD waived the Full requirement 2026-11-24); an opponent's Flickerwisp on
  Karn goes to the command zone without asking; Cursed Mirror never copies a commander; the AI does not cast Karn at instant speed under Gandalf;
  Breya's 3-damage mode takes a player only; the AI numbers for Station, Breya, Herd Heirloom, Twitching Doll and Karn's own priority are
  provisional and untuned. More engine limits: in a rare spot the Oracle exception checks that the Oracle is castable before the blink itself is
  paid, so the AI can draw its library to zero and lose; a main-phase blink (Flickerwisp worst) costs Karn's attack and the AI does not weigh it;
  Flickerwisp keeps only the primary commander in exile; the Restoration Angel protector double-blinks Karn whenever it saves any creature, which
  flatters Karn's draw numbers in a baseline; The Reaper misses the case where a wipe kills it together with the creature, and a person is never
  offered its "may" (the card is always taken); for a person, Station, Breya, Herd Heirloom and Twitching Doll go through the card's own option hook;
  and the `st(...)` settings rows and `HAND_DELETED` in `ai/mod.rs`, `CTHRESH` in `impls/rules.rs` and the key list at `settings.rs:197` are keyed on the retired decks, so Karn gets the generic values.
- **Doomsday:** its pile builder and its play are card code that runs for every deck (only the `tutor_stat` trace stat is keyed on `thrasios`); whether it ever finds a winning pile with this list is unmeasured.
- **Deck-key gates that do not run for this key:** Solitude's pitch spares line pieces (`rules2.rs`), equipment holder scoring for Buster Sword and
  the Sword (`bulk_white.rs`, `cloud.rs`, `tymna_cards.rs`) and commander importance 7.0 (`cast.rs`) are keyed on the retired decks; Karn gets the generic paths.

## To check in the first runs
Goldfish first (`trace-record.sh --mode goldfish`), then `--analyze` at t4 and t5: the turn Karn is cast and how many colours he counts on entry;
`flicker <card> -> Karn, Gilded Guardian` in the stats; `death_kind decked`; `protection_used`; the Oracle and Consultation line's go rate; hand size at
cleanup after an end-step blink; how often Akroma's Will has both modes (a commander on the battlefield); what Ranger-Captain of Eos fetches (Yoshimaru, Giver of Runes or Esper Sentinel), how big Yoshimaru gets, how often Esper Sentinel draws (cast rate, tax paid or not), and how often Giver of Runes protects Karn; the life lost to the lands; whether Doomsday ever starts (`dd|asked|...` and `dd|chosen (cast begun)` in the stats).

## List change 2026-11-24 (JD's choice, not simulated)
Out: Deflecting Swat, Generous Gift. In: Yoshimaru, Ever Faithful, Giver of Runes. The two are the deck's only creatures with mana value 1 or less, so Ranger-Captain of Eos (whose search found nothing) now has two targets; Giver of Runes protects Karn from one colour (or colourless) each turn, and Yoshimaru gets a +1/+1 counter for each other legendary permanent that enters. Job lost: Swat's free redirect (it needs a commander on the battlefield, so it was mostly a hard cast at {2}{R}), Generous Gift's answer to a noncreature permanent (Anguished Unmaking, Skyclave Apparition and Farewell remain). Counts after (before): creatures 21 (19) without Karn; instants 14 (16); curve 1: 14 (12), 3: 16 (18), average 2.86 (2.92); pips W38 U17 B23 R3 G3 (W37, R4); Game Changers 15 (15); land sources unchanged. Both new cards are Full.

## List change 2026-11-24 (JD's choice, not simulated): Felidar Guardian => Esper Sentinel
JD's decision: Esper Sentinel (his owned copy, moved from the Arzakon deck; `data/ownership.toml`) takes Felidar Guardian's slot (a proxy). Not slot-for-slot by the packages rule: a {3}{W} blink creature out, a {W} tax body in (mana value 4 for 1); it is a package change JD chose, with the loss below.
- **Job lost:** a four-mana 1/4 body with a "may" blink on entering, and its two infinite lines: Restoration Angel + Felidar Guardian (infinite blink; with Elesh Norn out and Karn as a target, an infinite draw) and Felidar Guardian + Cursed Mirror (the Mirror copies the Guardian). Both lines are gone from the guide's disclosure list. Blink effects are now six (Ephemerate, Cloudshift, Restoration Angel, Flickerwisp, Teleportation Circle, the harnessed The Mind Stone). The two lines were never modelled as a combo (Spellbook `KNOWN_GAPS` in `rust/crates/sim-core/tests/spellbook.rs` still lists both for `karn_gilded`; the coordinator updates it after the next Spellbook fetch).
- **Job gained:** a one-mana white artifact creature (1/1): it is the 14th coloured artifact (white, so it counts for Karn's draw; white was already supplied by Shorikai, Breya, Thopteryx, The Mind Stone and The Wind Crystal, so it adds no new colour), a tax body (draw a card unless the opponent pays {1}, its power) and a third Ranger-Captain of Eos target (creatures with mana value 1 or less: Yoshimaru, Giver of Runes, Esper Sentinel).
- **Counts after (before):** creatures 21 (21) without Karn; artifact creatures 7 (6), other creatures 14 (15), artifacts in all 22 (21); curve 1: 15 (14), 4: 9 (10); average mana value 2.81 (2.86), 180 over 64 (183); pips unchanged (W38: Felidar and Sentinel each have one {W}); blink effects 6 (7); Game Changers 15 (15); coloured artifacts 14 (13); land sources unchanged. Esper Sentinel is Full (`data/card_overrides.toml`; the note reads: opponents pay X for their first noncreature spell only when they can spare it, otherwise you draw).
- **What the old numbers say, and why they do not transfer:** Felidar Guardian has no number in this deck (no sims). The only recorded numbers are from the retired Yoshimaru/Tymna list at t5: cast in 9.7% of games, win rate when cast 45.5% (1,500 games, `audit/stage3-baseline-2026-11-23/yoshimaru-analyze.txt`) and cast in 10.0%, win rate when cast 27.4% (3,000 games, `audit/stage3-paired-2026-11-23/big-armC-yoshimaru-analyze.txt`). Both are correlation only and for a different deck (the Yoshimaru list ran it with Preston-style blink value, not Karn). The brief's figures (16.7% and 12.0%) were not found in the repository.
- **To check in the first run:** Esper Sentinel's cast turn and cast rate, how often the table pays the {1} (the sim's opponents pay when they can spare it, per the card's note), what Ranger-Captain fetches now, whether Karn's colour count on entry moves (it should not: white is covered), and whether the missing blink shows in the Karn re-entry count per game.

## Tested
No sims yet.
