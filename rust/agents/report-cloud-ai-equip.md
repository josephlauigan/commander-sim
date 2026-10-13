# Report: cloud-ai-equip (diagnosis only; no engine or AI code changed)

Model: claude-sonnet-5-5. Evidence: 15 single-game traces (`commander-sim --deck cloud --pool t1 --profile loose --ai
lookahead --trace N`, N = 1..15; game N = seed 500000+N) and the probe tests in
`rust/crates/sim-core/tests/cards_cloud_ai_diag.rs` (`cargo test --release -p sim-core --test cards_cloud_ai_diag --
--nocapture --test-threads 1`; they print every main-phase option with its heuristic utility and the look-ahead score;
one `#[ignore]`d test demonstrates problem 1). No batch sims were run.

How a main-phase play is chosen (needed to read the evidence): `brain::main_options` lists options with a heuristic
utility (cast = `cloud_prio/10`, equip = `dsl::equip_options`); `search::choose_inner` keeps the top 5 by utility plus
"stop", plays each out (`playout_main`: the option, the rest of the phase heuristically, then the opponents' turns until
the end of our next turn, 6 rollouts) and scores `search::evaluate` (board `pval` + 0.35/power + 0.9/card in hand + 0.6/land
+ life, against the opponents). The trace's percentages are a softmax of the HEURISTIC utilities, not the search scores;
the search pick is the last item (`-> X [search]`).

## Problems, ranked by expected impact

### 1. Three Equipment of the C3 list can never be equipped (Champion's Helm, Sword of Feast and Famine, Magitek Scythe)
Probe `probe_k`: with each Equipment loose on the battlefield and 9 lands, the main options contain an "equip" option for
every Equipment of the deck except these three (`#[ignore]`d test `bug_helm_feast_and_famine_and_scythe_have_no_equip_option`
fails on Champion's Helm first). After casting Champion's Helm with 1 land spare (probe_j) the only option is "stop".
- Code: `impls/cloud.rs` / `dsl::equip_options` (dsl/mod.rs ~1819) only offer equip for cards whose compiled data has an
  `equip_cost` static. Champion's Helm (data/cards.json: `dsl: null`, tag `helm`) and Sword of Feast and Famine (tag `sword`)
  are hand-tagged for the Sauron deck: their equip plans are `decks::sauron_options` (`helm_equip`, `sauron_equip`), which only
  runs for key `sauron`. `pool::special_options` (run for `cloud` through `GENERIC_PLAYS`) handles only Greaves, Boots, Cloak, Clamp.
  Magitek Scythe's Equip {2} is not modeled (it attaches on entering only, `scythe_etb`).
- Effect: this is why Champion's Helm is "cast 43% of the games drawn": it is a dead card. Its cast utility is 5.5 (pval 3.0,
  no equip), and when cast it adds nothing but `pval`. Same for SoFF (a good card, protection + untap lands).
- Fix (highest value, small): give all three real equip abilities (compile "Equip {N}" for them in data/card_overrides.toml
  or `cardgen`, with the +2/+2 / +2/+2 pro-B-G / +2/+1 bonuses and Helm's hexproof-for-legends), or extend
  `dsl::equip_options` to the hand-tagged ones. Also give them `cloud_prio` entries and `equip_utility_bonus` weights.

### 2. Cloud's tutor ignores the new cards and fetches slow Equipment (`cloud::cloud_pick`)
Reveals over the 15 traces: Mask of Memory 14, Wrecking Ball Arm 5, Colossus Hammer 9, Sword/Shield/Helm of Kaldra 7+7+4, never
Sword of Fire and Ice / Feast and Famine / Hearth and Home / Champion's Helm. The `order` and `REST` lists (cloud.rs ~286-345)
predate the C3 list, so those four are found only by the last-resort `lib.iter().find(...)`. The tutor also looks only at
"what the deck wants", not at mana: Colossus Hammer (cast 1, equip {8}) is fetched 9 times, Kaldra pieces (cast 4 + equip 4)
at 3-5 lands. In game 1 (R6, R7: 4 lands) the deck casts Shield of Kaldra, then Sword of Kaldra, with nothing ever attached
(all mana used to cast); in game 3 R5 Helm of Kaldra (4) is cast on 4 lands with nothing left to equip it.
Each recast of Cloud (2.4 casts per game on average in the 15 traces; removed 11 times) also adds one more Equipment to a hand
that cannot spend its mana on all of them, which is the "drawn but left in hand" pattern.
- Fix: put Sword of Fire and Ice, Sword of Hearth and Home, Sword of Feast and Famine (once equippable) first in the lists (equip
  {2}, cast 3), demote Colossus Hammer (equip {8}) and the Kaldra pieces unless lands >= 6 or Puresteel/Cid is out, and make the
  pick depend on `total_mana` (cast + equip cost <= mana next turn).

### 3. The position score cannot see what several Equipment do, so the look-ahead picks "stop" or a worse play
Probes `probe_m` / `probe_l` (4 lands, Cloud, loose Shield of Kaldra, hand Sword of Kaldra, Helm of Kaldra, Puresteel Paladin):
"equip Shield of Kaldra" (heuristic 5.8) changes the strength from 23.70 to 23.70 (indestructible/protection are worth nothing
to `search::strength`), costs all 4 mana, and its look-ahead score is 20.83 against 27.82 for "stop" (stable over 40 seeds,
sd ~0.5, `probe_h` style tally). The same scoring explains traces where the AI passes with mana open: game 4 R5/R6 (4-5 lands,
7 cards in hand: Paladin 48% / Sword of Kaldra 29% / Mask 19%, pick `stop` twice, though Cloud was an Elk from Kenrith's
Transformation that turn, so not the cleanest case), and game 4 R6 main 1 `stop` then Cloud cast after combat.
`strength` counts only `pval` + 0.35/power: Shield of Kaldra, Helm (hexproof), Mask (draw 4), SoFI/SoHH triggers, doubled
triggers have no value there; only +N/+N is seen. `search_bonus` (cloud.rs) adds +10 only for casting Cloud himself.
- Fix: extend `cloud::search_bonus` to the other plan actions: equip of a non-stat Equipment on Cloud (Shield, Helm, Mask,
  Scope, Shuriken, Kitesail), casting an Equipment while Cloud is on the board and mana remains to equip it, and a penalty on
  "stop" when mana >= 3 is unspent and a cast/equip option exists. (Not changed: engine-wide `strength`.)

### 4. Equip scoring against casting is roughly even, which is fine; the bad cases are mana, not scores
Probe `probe_a`/`probe_b`/`probe_l2` (Cloud out, loose Arm or SoFI): the equip option wins the search in B (equip Arm 25.97 vs
cast 22.9-23.9), L2 (equip SoFI 29.81 vs Cid 29.47) and ties in A. So with Cloud on the board and a loose piece whose equip
cost is affordable, it is equipped. Heuristic equip utilities for Cloud: `equip_utility_bonus` +4.5 (+1.5 for Arm/Armor/Hammer,
+1.5 for a trigger); resulting values 5.8 (Shield) to 11.1 (Hammer, which costs {8}: this value is wrong, it scores highest
and is never payable; top-5 truncation therefore rarely wastes a slot on it only because `can_pay` filters it). Unattached
pieces left on the battlefield get destroyed first (game 3: SoFI cast R3, destroyed R4 before an equip turn; game 4 R6,
game 1 R5 SoHH exiled by Light-Paws's aura). Casting Equipment on turn 3 when there is no mana to equip it the same turn
exposes it for a full opponent turn.
- Fix: in `cloud_prio`, raise a loose-equipment-in-play-and-Cloud-present equip over casting another piece (e.g. count
  `loose` pieces and penalise casting a third), and prefer a cast+equip pair that fits the mana (cost + equip <= lands).

### 5. Equip targets without Cloud work, at low weight (question 3)
Probe `probe_c` (no Cloud, 6 lands, Bronze Guardian + Exalted Angel, loose Arm and SoHH): "equip Sword of Hearth and Home" is
offered at 4.9 (no `holder_bonus`/`equip_utility_bonus`), Arm is not offered (equip {8} for a non-legend), and the search chose
casting Solemn (45.21) over the equip (43.07). Trace evidence of suiting up others: game 5 R6 equips Scope and Mask to Knight of
the White Orchid while Cloud is dead; when Cloud returns (game 2 R7, game 5 R8 through Beatrix's pull, game 6 R9) the equipment
is moved to him (`wants_move`: equip Arm to Cloud {3}, SoHH/Mask 2). Probe `probe_d`: Cloud back, Arm and SoHH on Bronze
Guardian, 6 lands: both "equip X to Cloud" options are offered (10.9 / 7.6); the search scores in that probe are 100.00 for some
options (the opponent dies in the playout: seph table with an empty library), ignore them. Minor: nothing evaluates paying {3}
to move the Arm for no net stats (7/7 base on a 2/1 Cloud is a gain, so this is correct).

### 6. Champion's Helm, Solemn Simulacrum, Court of Ardenvale priorities (question 2)
- Helm: see problem 1 (dead card). Cast utility 5.5.
- Solemn Simulacrum: no `cloud_prio` entry, so `generic_prio` rates it as a creature: 42 + 2*pow = 46 (utility 4.6) against 7.0-8.2
  for the Equipment and the Equipment creatures. It costs 4, the same as a Kaldra piece or Cid + something, so with 4-5 lands there is
  always a higher option; the look-ahead does pick it when it is the only 4-mana play (probe F: 21.12 vs stop 18.86), so it is only
  skipped because the hand is full. A ramp creature should be cast on turns 3-4 when the hand holds expensive Equipment.
- Court of Ardenvale: utility 5.0, six mana; probe F2: it wins the search (20.75) over Shield of Kaldra (19.83) when it is the only
  castable thing, so it is simply low in the queue.
- Fix: `cloud_prio` entries: Solemn 66 on turns <= 5 (ramp pays for the equip costs), Court 60 when lands >= 6 and the graveyard holds
  an Equipment or Cloud (it returns permanents with mana value <= 3: SoFI, SoHH, Arm... check the card text), Helm only after problem 1.

### 7. Other turn wasters (question 5)
- Zero-damage attacks: 15 traces show 14 `Cloud attacks ...: 0 damage` lines (games 1, 4, 5, 6, 7, 8, 10, 13, 15). Game 1 R5: Cloud
  wearing only Explorer's Scope attacks Isshin's board for 0 (`cloud::attack_filter` keeps him home only when he wears nothing, so
  a Scope counts as "equipped"). Game 7 R4/R5: Cloud (bare, or with nothing relevant) attacks "all at Tatyova" for 0 twice.
  Game 4 R6: the pre-combat `stop` then the creatures attack and Cloud (Elk) dies, recast after combat (the search seems to use
  attacking as a way to recast him, which is arguably correct).
- Fix: `attack_filter` should require the wearer to have real stats or evasion (`epow >= 4` or an unblockable piece), not "any
  Equipment".
- Cloud is recast after combat in main 2 more often than before it (game 4 R6, game 6 R7): the heuristic cast priority is 130 only on
  `pl.turns >= 2`, but from hand (after dying) the `cmd_stays_in_hand` rule means he is only an ordinary hand option in main 1
  and the search at times prefers `stop`. Casting him pre-combat would put the tutored Equipment in hand a phase earlier. Low impact.
- Cloud loses to Kenrith's Transformation / bounce (Stormfront Riders returned him to hand in game 2 R8 while he wore the Arm).

## Answers in one line each
1. Unattached because: (a) three cards have no equip option (problem 1), (b) the tutor and the casting order buy 4+4 and 3+8 pieces
   that cannot be cast and equipped the same turn (2, 4), (c) the search values several pieces at 0 (3). Equip is not scored too low
   in general (4); the equip option is not missing for the rest.
2. Helm: no equip. Solemn / Court: low heuristic priority (4.6 / 5.0) with no `cloud_prio` entry (6).
3. Yes, suits up others at 4.9; when Cloud returns he gets the equipment back at its equip cost (5).
4. Tutor lists predate the C3 swords; Hammer and Kaldra pieces are fetched too often for the mana (2).
5. See 7.

## Proposed fix order
1. Equip abilities for Champion's Helm, SoFF, Magitek Scythe (card data / dsl). 2. `cloud_pick` order with mana-aware filtering,
SoFI/SoHH/SoFF first. 3. `search_bonus` for equips and casts of non-stat Equipment, anti-"stop" penalty. 4. `cloud_prio` for
Solemn / Court and a "cast + equip must fit this turn's mana" rule. 5. `attack_filter` threshold. Re-measure after each
(batch sims are stopped by JD for now).

## Files touched
Only `rust/crates/sim-core/tests/cards_cloud_ai_diag.rs` (new, probes; 13 pass, 1 ignored) and this report. No shared files.
