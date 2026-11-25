# Karn, Gilded Guardian (key `karn_gilded`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/karn-gilded-guardian.md`, registered 2026-11-24 (Karn, Gilded Guardian, WUBRG; 64 spells and 35 lands, no basics;
15 Game Changers, Bracket 4). **Tiers:** t4 and t5 (`data/my_decks.toml`). **No sims yet.** The deck replaces the Y'shtola and Yoshimaru/Tymna
decks, whose findings are in `audit/retired/yshtola/` and `audit/retired/yoshimaru/`.

## Plan (what the deck is meant to do)
- **Engine:** Karn draws a card for each colour among the other artifacts we control, up to five; coloured artifacts (13 in the list; the
  Transguild Courier and Infinite Guideline Station are five colours alone) and blink effects (Ephemerate, Cloudshift, Restoration Angel,
  Flickerwisp, Felidar Guardian, Teleportation Circle, the harnessed The Mind Stone) enter him again; Gandalf the White and Elesh Norn,
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
cleanup after an end-step blink; how often Akroma's Will has both modes (a commander on the battlefield); what Ranger-Captain of Eos fetches (Yoshimaru or Giver of Runes), how big Yoshimaru gets, and how often Giver of Runes protects Karn; the life lost to the lands; whether Doomsday ever starts (`dd|asked|...` and `dd|chosen (cast begun)` in the stats).

## List change 2026-11-24 (JD's choice, not simulated)
Out: Deflecting Swat, Generous Gift. In: Yoshimaru, Ever Faithful, Giver of Runes. The two are the deck's only creatures with mana value 1 or less, so Ranger-Captain of Eos (whose search found nothing) now has two targets; Giver of Runes protects Karn from one colour (or colourless) each turn, and Yoshimaru gets a +1/+1 counter for each other legendary permanent that enters. Job lost: Swat's free redirect (it needs a commander on the battlefield, so it was mostly a hard cast at {2}{R}), Generous Gift's answer to a noncreature permanent (Anguished Unmaking, Skyclave Apparition and Farewell remain). Counts after (before): creatures 21 (19) without Karn; instants 14 (16); curve 1: 14 (12), 3: 16 (18), average 2.86 (2.92); pips W38 U17 B23 R3 G3 (W37, R4); Game Changers 15 (15); land sources unchanged. Both new cards are Full.

## Tested
No sims yet.
