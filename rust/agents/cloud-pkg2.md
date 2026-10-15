# Task: cloud-pkg2 — apply Package D, then design three more Cloud packages (no batch sims)

Read `rust/agents/T0-RULES.md` (Hard rules, How cards are modeled, Keeping context small apply). Branch cloud-aifix3
(origin/main + the 10/15 audit + four Cloud AI fixes). Handoff `rust/agents/handoff-cloud-pkg2.md`, report
`rust/agents/report-cloud-pkg2.md`. `-j 12` builds, `--jobs 12` for smoke runs.

## Context (read first)
- `rust/agents/report-cloud-audit-1015.md` (why Cloud loses) and `audit/cloud/2026-10-15/` results, look-ahead,
  2,000 games per tier, paired: AI fixes T1 14.0->15.8, T2 11.9->13.8; **Package D** (audit/cloud/packages/d-defense.swaps)
  +6.4 / +6.3 on top (T1 22.2, T2 20.1); Package R -1.9 / -1.1 (cutting creatures/Equipment for removal spells hurts);
  D+R T1 20.6. JD chose **Package D**.
- Structural limits: Cloud is a 2/1 whose value needs Equipment on him; equipment costs mana twice (cast + equip), the
  plan is online by turn 4 in only 23-37% of games, the hand clogs (~6 cards at round 4), mono-white has little card
  draw, one attacker vs three opponents, Cloud removed 0.6-1.35x a game.

## Part 1: apply Package D to the real list
- `cardgen deck cloud <new import list> --log "<text>"` (see `cardgen deck --help`; it rewrites the doc's Import list
  and records tests/fixtures/my_decks_parsed.json; history goes to decklists/CHANGELOG.md under Cloud's section).
  Update the doc's other sections that name the 8 cut cards (Decklist by type, Strategy mentions) so the doc matches.
- The 8 cut cards go to `bulk-cards.txt` (follow its format). The 8 new cards are buys (all unowned, $23.05 total:
  Ghostly Prison 6.67, Norn's Annex 5.75, Archangel of Tithes 4.39, Wrath of God 3.93, Day of Judgment 0.91,
  Windborn Muse 0.82, Baird 0.31, Fumigate 0.27) — record them where the doc keeps its buys.
- `cardgen export`, `cardgen validate`, full `cargo test --release -j 12 --no-fail-fast` must pass. Commit.

## Part 2: design three packages on top of the D list (each 4-8 swaps, mono-white legal)
- **S — speed:** cheaper/free equipping and cost reducers, so Equipment costs mana once (e.g. Puresteel Paladin,
  Stonehewer Giant, Stoneforge Mystic, Quest for the Holy Relic, Fighter Class, Sigarda's Aid, Brass Squire-style;
  check what's already in the list first: Ardenn, Balan, Kor Outfitter, Auriok Steelshaper are in).
- **C — card draw:** refill the hand in mono-white (e.g. Sram, Senior Edificer; Esper Sentinel; Puresteel Paladin's
  metalcraft draw; Welcoming Vampire; Mentor of the Meek; Skullclamp — JD's copies live in Kefka and Seph and decks
  never share physical cards, so a Skullclamp would be a buy).
- **B — backup carriers:** creatures worth equipping when Cloud is gone: evasion, protection, double strike or
  hexproof bodies (e.g. Fencing Ace, Mirran Crusader, Kor Duelist / Armored Skyhunter if not already in, flying or
  vigilance bodies), cheap enough to come down early.
- Prefer cards JD owns (`bulk-cards.txt`, `binder-cards.txt`, not in another deck), then cheap buys (Scryfall batch
  collection POST for prices; never one-by-one). Cuts: the weakest remaining cards by the audit's data (rarely cast,
  low impact) — never the D cards, never Cloud's core Swords/Kaldra. Package R showed cutting bodies hurts: keep the
  creature count up.
- Every card must be **Full** in `cardgen audit --cards <file>`; if a strong candidate isn't, either model it (with a
  test, fully from Oracle text) or drop it and say so.
- Write `audit/cloud/packages/s-speed.swaps`, `c-draw.swaps`, `b-carriers.swaps` (one `OUT=>IN` per line) plus a
  table in `audit/cloud/packages/README.md` (card, owned or price, why).
- Test plan for the lead to put to JD: paired look-ahead runs on top of the D list, T1 and T2, 2,000 games per tier,
  each package alone and the best combination, with time estimates (one tier at 2,000 games takes ~5 minutes at 22
  threads). **Run no batch sims** — a 100-game smoke run per package to make sure it loads is fine.

Report ≤15 lines: line 1 DONE or HANDOFF, line 2 model ID, commits, the three packages (swaps, cost), test plan.
