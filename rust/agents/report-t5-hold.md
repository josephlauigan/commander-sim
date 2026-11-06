# Report: t5-hold (measure Tier 5 counter-mana holding; no behaviour change)

## Built
- `sim-core/src/holdstats.rs`: trace-only `hd|...` counters in each AI seat's `stats`. At the end of the second main phase the turn
  is put in a group: a card held by `brain::hold_value` (counter / protection / instant removal / Ad Nauseam / other), or `idle`
  (nothing held, 2+ mana unused, a castable nonland card that is not a held kind). The group is followed to the holder's next turn
  start and ends as: used (a same-kind card hit something), cast with no hit, mana spent elsewhere, or nothing spent. Hits are
  classed combo piece or anything cast mid combo attempt / commander / creature / other permanent / removal spell / other spell /
  ability. Also kept: turns, "2+ unused and any castable card" (the C3 measure), the same split into held vs not held, unused
  mana, hold value, mana spent and other instants cast in the window. Each is kept for own turns 5-10 and for all own turns.
- Hooks: `engine/turn.rs` (snapshot after `mana_held`; outcome at turn start), `engine/cast.rs::on_cast`,
  `engine/mana.rs::spend_unit`, `t6stats::counter_hit`, `engine/removal.rs` (`apply_removal_rest`, the "protects" branch).
  `Player::hw` holds the snapshot; `Settings::hold_trace` (default on) exists so the identity test can switch it off.
  Skipped for people's seats and look-ahead copies. No RNG, no log lines, no decision reads a counter.
- `sim-cli/src/holdreport.rs`, printed by `--realism` after the flow tables: pooled table and per-deck, for turns 5-10 and all turns.
- `tests/t5_hold.rs`: three seeded Tier 5 pods (two adaptive, one look-ahead) play identically (winner, rounds, full log, every
  non-`hd|` counter) with the counters off; and a scripted held Counterspell that counters Sheoldred is a `used` hold with hit
  class `creature`, and a turn with nothing cast is `unused`.
- `rust/agents/t5-hold-queue.sh`: build, then `--realism --pool t5` and `--pool t3`, 2000 games, seed 900000, adaptive, to
  `audit/realism/t5-hold-2026-11-06/{t5,t3}-adaptive-2000.txt`, then `ALL DONE`. Not run by me.

## Checks run
`cargo build --release --all-targets`: no warnings. Passing: t5_hold, c3_casual_speed, ai, ai_flow, ai_eval, cards_yshtola,
cards_thanos, cards_sauron, cards_sauron_speed, cards_sauron_sweepers, cards_sauron_threats, cards_sheoldred, cards_sheoldred_sac,
and sim-cli's holdreport unit test. No sims were run, so there are no measured numbers yet.

## Limits (see NEEDS JD in the reply)
- Counter target: the topmost uncountered stack item of the countered player (not a stored target id).
- Removal and protection hits are classed by the permanent, found at removal time; a removal spell that is countered or fizzles
  is "cast, no hit". Protection saved by a sacrifice-style response (no mana) still counts as a hit, so protection "used" may be
  slightly high.
- Mana "spent elsewhere" counts any mana spent by the holder in the window (abilities included), not only instants.
- `--ai adaptive` (the queue) does not use `search::hold_bonus` / `hold_credit`; Tier 5 pool seats under the look-ahead hold with
  those extra terms, so a look-ahead run could differ from the adaptive one.

## Proposed hold rules, by outcome (not implemented)
All change `ai/brain.rs`; the held-mana value `v` is made in `hold_of` (line ~241) and applied by `reserve_penalty` (~258, called at
~1192 in `card_utility`); the look-ahead adds it in `search::hold_bonus` (~465), `hold_credit` (~660) and the `evdiag` hold (~749).
1. Most holds wasted (used well under half of held turns, mostly "unused"): shrink the outside-deck floor in `hold_of`, i.e. the
   `v = v.max(1.5 + 2.5 * caution * n.min(2))` line for `!is_main(...)` decks, e.g. scale it by the measured use rate, or
   drop it to 0 unless `s.max_threat` is high.
2. Holds used mostly against low-value targets (spells, ability, engine) but seldom against combo or commanders: hold only when an
   opponent is close to winning: gate the floor on `s.combo_near || s.cmdr_combo || s.max_threat >= X`, keeping the
   `3.0 * caution * threat` term.
3. Hold used but only the cheapest answer needed while more mana sits idle (high "unused mana" with a castable card, low
   mana per hold): cap the reserve to the held card's cost: `reserve_penalty` already uses `seq_cost(c, h)`; the idle cases come
   from the main-2 stop in `main_options`, so offer castable cards when the reserve is still payable after them.
4. Most holds used (counters hit combo pieces / commanders): leave the rule; a Tier 5 table of counter decks then really does
   stonewall, and the Tier 3 comparison shows whether it is the same at Tier 3.
5. Mixed by kind (e.g. counters wasted, protection used): split the floor by kind: the `n` count in `hold_of` is of `Tag::Ctr`
   cards only; give removal and protection their own terms.

---

# Part B (t5-hold-b): protection hold shrunk, idle turns diagnosed

Commit on branch t5-hold. No sims were run (the coordinator runs them); only single traced games.

## 1. Protection hold (changed)
`ai/brain.rs` `hold_of`: for a **pool deck** whose held cards include no counter (so protection only), the outside floor
(`1.5 + 2.5 * caution * n`) no longer applies. New `protection_hold`: mana is held only if the player has a commander, a
combo piece (`is_combo_piece`) or a plan file's key card / protect row on the battlefield (`pool::worth_holding_for`: the
`worth_protecting` test without its "value 4 or more" clause) **and** `removal_risk` (an opponent that may hold instant
removal with two mana open, from `prob_holding`) is at least 0.25. The value is then `1.0 + 3.0 * caution * risk` (turn 1-2: -2).
Otherwise `hold_of` returns `(None, 0.0)`, so "stop" is a plain stop (-3) and the look-ahead (`hold_bonus`, `hold_credit`, evdiag,
all read `hold_value`) gets no hold credit: no separate edit was needed there.
- Counters keep their term exactly: when the held list has a counter, the code is the old line (test `a_held_counter_is_unchanged`,
  with a protection spell added the value is the same).
- **Decks touched**: Tier 1-5 pool decks (every outside generic deck: not `is_main`, not `jd_deck`, not a Tier 6 key, not `pl.t6`).
  **Not touched**: JD's and Avery's decks (sauron, yshtola, kefka, sheoldred, cloud, alela, galadriel, omnath, najeela, sephfs),
  Thrasios, Thanos and every other Tier 6 key; they keep the old floor (test `jd_and_tier_six_decks_keep_the_old_floor`).
  Note JD's hand-deleted decks read `hand_key` "" and so already used the outside floor; that is unchanged.
- Expected effect (not measured): the 2.2% of Tier 5 turns with protection held (Miirym 10% of its turns, 4.8 mana, used 5%) drop
  to the turns where something worth saving is out and an opponent plausibly has instant removal. Those mana go to casting
  cards instead. The "protection held" line should fall toward 0.2-0.5%; check its used% rises. The risk gate (0.25) needs about
  four unseen instant removal spells in an opponent's library with a full hand and two open mana, so it is rarely met: if the
  coordinator sees protection spells never held and Miirym losing the commander to removal more, lower `PROT_RISK_MIN`.
- Not modelled (stated): the gate is instant-speed removal only, as the brief says. A sorcery-speed removal spell or a wipe on the
  opponent's own turn is not counted, although Heroic Intervention / Teferi's Protection would answer them too.

## 2. Idle turns: diagnosis
Traces: 50 Tier 5 games (`--pool t5 --trace 1..50`, seed 900000, adaptive) and 50 Tier 3 games, with a new trace-only
diagnostic: `STOP_DIAG=1` makes `stop_diag` (brain.rs) print the "[X stops in main 2 (...) ... payable: card: why]" line for every
seat, not only Tier 6 (off by default; no effect without a trace log). I took each end-of-main-2 stop with nothing held, 2+ mana
left and a payable nonland card that is not a held kind (the `holdstats` idle test): **134 idle turns in Tier 5** (8.6% of 1550
traced own turns, 4.5% in Tier 3 on 84 of 1846; the sim runs gave 11.2% and 4.9%).
- In 133 of 134 the card was "no utility" (`card_utility` None): its priority is 0 (119 turns) or it is a tutor that waits for the
  end of the opponent's turn (14 turns, prio 43-72). Never: a utility below "stop" (0 turns), a main-2 stop outvoting a card
  (0), a sorcery-speed tax (0), a reserve kept for a held card (1 turn: Diabolic Intent, no sacrifice available).
- Attribution of the 134 turns (a turn counts once, under the first cause that fits, in this order):

| Cause | Turns | Verdict |
|---|---|---|
| A permanent the deck never casts: Garruk's Uprising (Miirym; game 5 R4/R9, 9 R3, 19 R7-R14, 23 R5, 36 R8) and Welding Jar (Urza; games 2, 4, 5, 8, 15, 46, mostly R1-R8) | 39 | **mistake, fixed** |
| Reactive card kept for a response: Deflecting Swat, Force of Will, Malakir Rebirth, Imp's Mischief | 18 | legit: cast by the response logic (free or instant redirect/protection) |
| Walking Ballista (rule: 4+ mana), Nature's Rhythm, Green Sun's Zenith, Chord of Calling, Everflowing Chalice with 2-3 mana | 19 | legit: X spells wait for more mana (`ballista_prio`, Xtutor 0) |
| Waits for a precondition: Animate Dead / Victimize (no target), Auras with no creature, Copy Enchantment / Estrid's Invocation, Chrome Mox (imprint), Thassa's Oracle, Windfall, a commander on its first turn | 21 | legit |
| Isochron Scepter / Dramatic Reversal held until the partner is in hand (prio 62 once both are) | 15 | legit (combo hold) |
| Howling Mine, Winter / Static Orb, Back to Basics, Unwinding Clock, Solemnity, Phyrexian Unlife, Diplomatic Immunity (symmetrical or lock permanents the deck keeps) | 15 | legit by design; real players are split too |
| Toxic Deluge (wipe kept until the board needs it) | 1 | legit |
| Tutors (Worldly, Vampiric, Mystical) wait for the opponent's end step | 6 | legit: they are cast in `end_of_turn_window` (e.g. game 13 R4 Worldly Tutor) |

Tier 3 has the same shape (reactive 16, X spells 15, tutors 7, preconditions 4) with none of the two mistakes; its remaining
"other" turns (Hardened Scales, Vandalblast, Reanimate, Finale of Devastation...) are conditional cards of the same kind.

## 3. Idle turns: fixes
1. **Garruk's Uprising** (`pool.rs` `generic_prio_base`, a general rule on `Tag::Uprising`): the derived worth could not see the
   engine, so priority 0 for 8 straight turns with 14 mana spare. Now 60 with a power-4+ creature out, 50 with two in hand to
   come, else 0.
2. **Welding Jar** (`t45_urza_yawgmoth.rs`, `jar_prio`, registered as the card's `prio`): it costs 0 and regenerates only from the
   battlefield, but its derived worth was 0. Now 40 once an artifact with `pval` 1.5 or more (the regeneration's own test) is out.
   Note this removes the idle count, not wasted mana: the Jar costs nothing.
Tests: `tests/t5_hold_b.rs` (new). Expected effect: the idle share of Tier 5 falls by up to a quarter of its count (39 of 134
traced turns), most of that in Urza and Miirym; Zur (19%) is mostly X-spell and precondition holds, which are legitimate, so
it will fall little. Tier 3 cannot rise: neither rule applies to a Tier 3 card unless Garruk's Uprising / Welding Jar is in its
list, and both only add a cast.

## Limits
- The 0.25 risk gate and the 60/50/40 priorities are my choices, not measured; the coordinator's Tier 5 run decides them.
- I did not run `rustfmt` on `brain.rs`, `pool.rs`, `t45_urza_yawgmoth.rs`: they already differ from rustfmt output in 12, 14
  and 1 places before my change, so formatting them would bury the diff. The new test file is formatted.
