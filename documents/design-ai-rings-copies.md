> Not built. Opus's design for the AI's Rings of Brighthearth copying loyalty and other abilities. JD decided on 2026-11-24 not to build it: the AI never copying is an accepted limit (the Arzakon note and FINDINGS limit 1). Written 2026-11-24 against main 2079d835.

# Opus design consult: the AI's Rings of Brighthearth on loyalty and other non-mana abilities

Tree read: `/home/joseph/projects/commander-sim-wt-gatebase` at `2079d835` (read-only; nothing built, run or changed).
All paths below are under `rust/crates/sim-core/src/` unless they start with `data/`, `decklists/`, `audit/` or `scripts/`.

## Summary (6 lines)

1. Today the AI's Rings never copies: `stack::rings_of` returns nothing for a non-person (engine/stack.rs:745-748), and `ability_window_ex` / `ability_window_card_ex` skip the stack when nobody can respond (689, 874), then pass `rings = is_human` (697, 879). Route A exists only as `combos::power_artifact`'s result (impls/combos.rs:1260-1283).
2. Design: keep one decision site, `rings_offer` (reached from `window_item`), and give it an AI branch. The AI takes the stack path only when a pure pre-check says it will copy, so a game with no AI copy plays exactly as today. Each ability's copy value is set by its own card code in a new `Ctx::rings_worth` field; compiled abilities use a filtered `dsl::ability_worth`. One shared check prices what the {2} costs.
3. What v1 copies: Arzakon's +2 (with a new target, and it can now see lethal through the copy) and -3 (a second Lotus token), Tezzeret, Cruel Captain's -3 and -7, Karn Liberated's +4 and -3, Karn, the Great Creator's -2, Urza's {5}, and compiled draw abilities with no target (Staff's {5},{T}; Aether Spellbomb's {1} draw in the Urza pool deck). The AI never copies anything else; the list is under "Status and docs".
4. Route A is not touched. The rule never copies untap, tap or mana effects. It makes no copy short of a lethal one while a catalogued combo is ready, or while Basalt Monolith and Rings are both out and paying {2} would leave less than the loop's {5}.
5. Play changes only in games whose players include a Rings deck: the arzakon seat, or urza-mono-blue-artifacts among the opponents. The Urza pool deck changes through Urza's {5} and Spellbomb's draw, which moves every JD t5 baseline (engine drift to disclose). Every other L2 record must be SAME, and an `AI_RINGS=0` switch reproduces today's play.
6. About 6-8 Sonnet worker hours and about 22 tests. Rings stays Full, because the audit-mine gate requires it. Its note is rewritten, and the limits it names need JD's approval.

## Where the decision lives

**One decision site, with each ability supplying its own copy value.** The decision belongs in the stack's activation path. Valuing the copy and choosing its target belong to each ability's card code, the same split the Camera uses.

What exists:
- **The person's path.** `window_item(g, p, it, rings)` (stack.rs:1068-1085) pushes the item, flushes queued triggers, then calls `rings_offer` (764-787) before the round of priority. `rings_offer` runs once for each of the person's Rings:
  - It asks through `human::mana::pay_tax_ask` (human/mana.rs:651), or takes the preset answer in the thread-local `RINGS_AUTO` (724-742).
  - It puts the Rings trigger on the stack with `trigger_window`.
  - If the trigger and the original both survive, `copy_item(g, p, id, true)` (600-620) clones the item. The clone gets a new id, the name "copy of …", and `trigger` / `dsl_ab` / `copy_worth` / `target` cleared, so it cannot be copied again. It goes on the stack through `stack_window`, so players may respond. When it resolves, the original's id is pushed to `g.trig_copies`.
  - The caller counts the copies (`ability_copies`, 564-570) and runs the effect once per copy, before the original: `ability_run_w` (890-916), `dsl::activate_ability` (dsl/mod.rs:2642-2650), `dsl::use_loyalty` (2686-2693).
  - The effect runs again with `is_copy = true` (or `DslCtx { target: None }`) and chooses its targets anew.
- **Everything that can be copied for anyone goes through `ability_window_ex` / `ability_window_card_ex`.** That covers DSL activated and loyalty abilities and every `ability_run` / `ability_run_w` caller, including the walker framework (`walker_pay_run`, impls/common.rs:1818) and Urza's {5} (impls/t5.rs:133). Abilities on the plain `ability_window` / `ability_window_card` (631-656, 705-721) cannot be copied by anyone's Rings. The doc at 626-630 says so, and the AI's Walking Ballista ping is one of them (impls/common.rs:3463).
- **The Camera precedent, for the AI.** `ai_respond` (1126-1138) copies the AI's own ability on top of the stack when `dsl::ability_worth >= CAMERA_WORTH` or `ctx.copy_worth >= CAMERA_WORTH`. To get there, `abilities_answered` returns true when `camera_ready` (524), which forces every such activation through priority.

Why not copy the Camera's placement:
- Rings is a trigger on activation, not a priority action.
- Adding a "Rings ready" clause to `abilities_answered` would push every activation by a Rings controller through rounds of priority, and flush queued triggers earlier, even when no copy is made. That changes games in which nothing is copied.

Can the AI reuse `window_item`? Yes, with four adjustments:
1. Payment. `pay_tax_ask` is the person's (it asks, and pays through their pool). The AI branch uses `mana::can_pay` then `mana::pay(g, p, 2, "", false)`, as the Camera does (impls/t6_copies.rs:452-455).
2. `RINGS_AUTO` is the person's preset; the AI branch must not read it.
3. `human::copies_priority` already does nothing for the AI (human/human.rs:185-187).
4. The AI's item must actually reach `window_item`. Today it doesn't when `!abilities_answered` (689 / 874). So add a pure pre-check, `rings_ai::wants(g, p, &ctx)`, before that early return. The early return then happens only when there is no copy to make, and the call becomes `window_item(g, p, it, is_human || ai_copy)`. Inside `rings_offer` the AI branch calls `wants` again: queued triggers may have resolved in between, so the second answer may differ, and that is harmless.

How a copy is represented, and the loyalty and once-per-turn bookkeeping: a copy is a new stack item whose effect the activating code runs again. Costs belong to the activation, so a copy pays nothing and counts as no use (CR 707.10: a copy of an activated ability is not activated). That is already how the engine works:
- **Walker framework.** `walker_pay_run` sets `loyalty_used` and `LoyaltyN` and changes loyalty before the window (common.rs:1801-1809). The copy runs only `eff`.
- **Compiled activated abilities.** `act_uses` goes up once (dsl/mod.rs:2629-2631).
- **Compiled loyalty abilities.** `use_loyalty` sets `loyalty_used` once (2679-2681).
- **A walker that dies paying its cost** (Arzakon's -3 from 3) leaves before the window (common.rs:1811-1815). A Rings copy still resolves, which the rules allow: the trigger is on the activation, not the source.
- **A copy cannot loop.** A copy is never activated, so it triggers no Rings; and `copy_item` clears what makes it copyable. Recursion is impossible.

Concrete changes at the decision site:
- `state.rs` `Ctx` (1119-1188): add `pub rings_worth: Option<f64>`. It is the AI's net worth of a Rings copy, computed by card code at activation; `None` means the generic DSL rule applies or there is no copy. `copy_worth` stays exactly the Camera's, so no Camera decision moves. `copy_item` also sets `cp.ctx.rings_worth = None`.
- `engine/stack.rs`:
  - Factor `ability_run_w` into a private `ability_run_ctx(…, ctx, eff)`.
  - Add `pub fn ability_run_rings(g, p, src, cd, name, imp, target, rings_worth: Option<f64>, eff)`. It builds `Ctx { target, copy_worth: Some(0.0), rings_worth, .. }`. With `None` it produces the same `Ctx` as `ability_run` today.
  - `ability_window_ex` and `ability_window_card_ex` get the pre-check.
  - `rings_offer` gets the AI branch. In v1 that means the first AI Rings only, and the same `trigger_window` + `copy_item`.
  - Log line text: `"    {name} pays {2}: Rings of Brighthearth"`. The existing `"    Rings of Brighthearth copies {}"` (782) is kept, so traces can be grepped.
  - `rings_of`, `pay_tax_ask`, `RINGS_AUTO` and every human file stay byte-identical.
- New `impls/rings_ai.rs` (about 200 lines), holding:
  - `ai_rings(g, p)`: the Rings on the battlefield, not phased or neutered, for a non-person; the same filter as `rings_of`, with no person check.
  - `ready(g, p)`: AI, at least one Rings, `can_pay(2)`, and the switch on.
  - `wants(g, p, &Ctx)`, `spend_cost`, `dsl_gross`, and the constants.
- `impls/common.rs` walker framework:
  - Add `walker_pay_run_c(…, eff: &mut dyn FnMut(&mut Game, bool) -> Res)` and keep `walker_pay_run` as a wrapper (`|g, _| eff(g)`), so `theorist_go` and `vondam_elspeth` don't change.
  - Inside, `rings_worth = if rings_ai::ready(g, p) { hook(g, p, src, i, target) } else { None }`. Then call `ability_run_rings` in place of `ability_run`.
  - The hook is a new `CardImpl` field (hooks.rs:540, `#[derive(Default)]`): `walker_copy: Option<fn(&Game, PlayerId, PermId, usize, Option<PermId>) -> Option<f64>>`. It returns a net worth; `None` means never.
- `switch!(force_rings, rings_on, FORCE_RINGS, "AI_RINGS", "the AI's Rings of Brighthearth copies")`, in ai/table_fixes.rs (26-47), or a sibling using that macro. `ready()` returns false when it is off, so `AI_RINGS=0` gives today's play byte for byte. It also gives a same-binary paired run, on the same seeds, for measuring.

## The rule per ability class

**Shared pieces.** All are pure `&Game` functions, so they consume no RNG.

- `mana::units_minus(g, u, n) -> Option<Vec<Unit>>`: `plan_pay(g, u, n, "")` then `units_left` (mana.rs:626, 851; `units_left` is private today).
- `brain::greedy_value(g, p, s, u)`: the hand's nonland, non-ritual cards and the command-zone cards with `castable` and `card_utility(g, p, s, x) = Some(v)`, taken by `v` descending. Each is paid from what remains with `plan_pay` + `units_left`, and the result is the sum of `v` over the cards paid. It reuses card_utility (brain.rs:354), cost_of, plan_pay and castable, the same inputs as `extra_mana_play` (481-517).
- `brain::spend_cost(g, p, s, before, after)`, the larger of:
  - `greedy_value(before) - greedy_value(after)`: the marginal plays this turn that the mana stops paying for. This is the AI's own opportunity cost of the {2}.
  - `hold_value`'s held card (brain.rs:239-271), if `before` pays it and `after` does not: `0.7 * hold_v`, or `8.0` when `combo_near`, the factors of `reserve_penalty` (325-335).
- **Combo guard.** No copy unless its net worth is lethal (at least `LETHAL = 20`) when either:
  - `combos::combo_ready(g, p)` (combos.rs:936): a ready line goes first; or
  - Basalt Monolith and Rings are both on p's battlefield and `total_mana - 2 < 5`.

  This guard is needed because Route A's start test lives inside `power_artifact` (1270-1276), while its `mana` field is `(0, "")` (176-182). So `mana_need` and `line_enabled_by` (899-933) can't see that the {2} strands the loop.
- **The decision**, `wants`:
  - If `ctx.rings_worth = Some(n)`: copy when `n >= RINGS_MARGIN`.
  - Else, if `ctx.dsl_ab = Some((cd, idx))`: let `gross = dsl_gross(...)`. Copy when `gross >= RINGS_DSL_MIN` and `gross - spend_cost(u, units_minus(u, 2)) >= RINGS_MARGIN`.
  - Else: never copy.
  - Always also require `can_pay(2)`, a passed combo guard, and the first AI Rings only.
- **Net worth from a walker hook**, generic form: `gross - spend_cost(u, units_minus(u, 2))`, with `u = mana_units(g, p, false)` read at activation (the ability's own cost is already paid).

**Constants. All are provisional; no sim has tuned any of them.**

| const | value | anchor |
|---|---|---|
| `RINGS_MARGIN` | 0.5 | the net utility a copy must clear |
| `RINGS_DSL_MIN` | 1.0 | below a compiled draw (`eff_value("draw") = 1.3`, dsl/mod.rs:2082) and above untap or counters (0.3, 0.4) |
| `LOTUS_BANK` | 2.5 | a second Black Lotus token kept for later (lotus_val banks the -3 at 3.6, but that is weighed against the +2, arzakon.rs:203-212) |
| `URZA_COPY` | 2.5 | a free card off the top; Urza's own option is 1.5 / 0.8 (t5.rs:119), a mana sink |
| `TUTOR_COPY` | 3.0 | = `CAMERA_WORTH` ("a tutor is 3", stack.rs:538-540) |
| `EMBLEM_COPY` | 6.0 | 2/3 of `tez_ult_val` 9.0 (t6_godo.rs:528-531) |
| `KARN_HAND_COPY`, `KARN_WISH_COPY` | 2.0 each | the abilities' own values (2.5, 2.0: t3.rs:1171, t5.rs:205) |
| `LETHAL` | 20 | the +2's lethal score (arzakon.rs:100) |

Name check against the newconsts gate: no `NO_`, `SKIP`, `FIX` or similar.

**Per ability.** "Today" is the AI's path now.

| ability | today | the copy's effect and target | gross worth (v1) | v1 |
|---|---|---|---|---|
| Arzakon +2 | `arz_plus_go` → `walker_pay_run`, target fixed on activation (arzakon.rs:148-158) | new `copy_plus_target(g, p, src, t)`, run again when the copy resolves (see below) | 20 if the pair is lethal; the kill score of `plus_plan` (pval, +2 for a commander or combo piece) for a second kill; 2.5 or 3.0 to the face; 0 if the original alone is lethal | yes |
| Arzakon -3 | `walker_go` → `lotus_token` | a second Lotus token | `base = u + L` (the original's token), `more = units_minus(u, 2) + L + L`; net = `max(LOTUS_BANK, extra_mana_play(g, p, s, me, &base, &more) - RITUAL_DISCOUNT) - spend_cost(base, more)` | yes |
| Arzakon -7 | `wheel_eff` | — | never (a second wheel undoes the first) | no |
| Tezzeret CC 0 | `tez_zero` picks again when it resolves (t6_godo.rs:477-485) | — | never in v1: the copy resolves first and untaps the best tapped rock, and the original untaps the next one. The engine taps for mana only when paying, so it can't tap the Monolith between them (the guide's "untap twice" needs that). The net is +1 mana only with two tapped 3-mana rocks, and Basalt + Rings is the combo anyway | no |
| Tezzeret CC -3 | `tez_three` picks again (499-509) | a second tutor (the copy gets the best card, the original the next) | `TUTOR_COPY` if at least 2 distinct qualifying cards are in the library (count, not `tez_finds`'s any) | yes |
| Tezzeret CC -7 | `tez_ult` | a second emblem (`tez_emblem` already loops over n emblems, 544-583) | `EMBLEM_COPY` if an artifact exists | yes |
| Karn Liberated +4 | `karn_plus` picks again (t3.rs:1147-1155) | the opponent loses a second card | `KARN_HAND_COPY` if that opponent holds 2 or more cards | yes |
| Karn Liberated -3 | `karn_minus` picks again (1162-1167) | exiles the second-best permanent | `karn_minus_val`'s rule on the second best: `common::best_opp_nonland(g, p, \|m\| m != best)`, pval at least 4, minus 2 | yes |
| Karn Liberated -14 | read as a win | — | never | no |
| Karn GC +1 | its effect is a no-op (t5.rs:201) | — | never | no |
| Karn GC -2 | `karn_minus` (t5.rs:189-196) | a second artifact from exile | `KARN_WISH_COPY` if 2 or more artifact cards are in exile | yes |
| Urza {5} | `urza_go` → `urza_top` → `ability_run` (t5.rs:122-163) | shuffle and play the top card free again (the closure already works as a copy) | `URZA_COPY` if the library holds 2 or more cards | yes (JD decides: moves t5) |
| Staff {5},{T}: draw; Aether Spellbomb {1}, sac: draw (urza) (both compiled) | `dsl::activate`; both are taken as filler above "stop" (-3.0) | draws again | `ability_worth` 1.3 through `dsl_gross` | yes |
| Other compiled abilities | `dsl::activate` / `use_loyalty` | — | `dsl_gross` returns 0 unless every effect kind is in `{draw}` and the ability has no target (`ctx.target == None`) | draw only |
| Walking Ballista ping | plain `ability_window` (common.rs:3463) | — | can't be copied today | no (option B1 below) |
| Tezzeret the Seeker, Narset (urza pool) | walker framework, no hook | — | never | no |
| The One Ring, Lightning Greaves equip, Voltaic Key, Mystic Forge, Inventors' Fair, Urza's Saga Construct, Sneak Attack, Strip Mine | `ability_run` or plain windows, worth 0 | — | never | no |
| Fetchlands | no stack at all (`turn::crack_fetch`, engine/turn.rs:558; excluded from `ability_options`, dsl/mod.rs:2479-2486) | — | never | no |

**`copy_plus_target` (Arzakon +2).** The original's 3 damage to `t` is pending, and the copy resolves first.
- If `t` is a player at 4 to 6 life, aim at `t`: worth 20, because the pair is lethal.
- If `t` is a player at 3 or less, there is no copy: the original is lethal alone.
- If the original alone finishes `t` (a creature with toughness 3 or less that takes damage, or a planeswalker at 3 loyalty or less), aim at `plus_plan` with `t` excluded.
- If `t` is a planeswalker at 4 to 6 loyalty, aim at `t` (the pair kills it).
- Otherwise aim at `plus_plan`'s choice, which is the face again for a face hit.

`plus_plan` gains an `exclude: Option<PermId>` argument. The copy works this out again as it resolves, and `ping_legal` decides (the target may have left). In the same change, `plus_plan`'s lethal test (arzakon.rs:99) becomes `life <= DAMAGE * (1 + rings_ai::ready(g, p) as i32)`. That way the original is aimed at the face when the copy makes it lethal; it is v1's only change to which ability the AI activates, and it applies only when Rings is ready. `arz_plus_go` passes `|g, is_copy| …` through `walker_pay_run_c`.

**New targets in general.**
- **Walker effects that pick as they resolve** (Tezzeret, Karn, the Lotus) get "new targets" for free. The copy resolves first and takes the best; the original takes the next best. Only a countered original differs from real targeting.
- **Compiled abilities with a target are excluded.** A compiled copy picks its target as it resolves (`DslCtx { target: None }`, dsl/mod.rs:2644), and it picks the same best target the original was aimed at on activation (2640). If the copy bounces or kills that target, the original does nothing (2654-2657). Paying {2} for that is waste. The Camera has the same problem today with a targeted compiled ability worth 3 or more (pre-existing; no deck has a Camera).

**Option B1 (not in v1; for JD to accept as a limit or ask for): Walking Ballista's reach.**
- Change `ballista_go` to `ability_run_rings`. Its copy aims at the same lethal player, or at the next X/1 creature.
- `ballista_ping`'s lethal test (common.rs:3431) becomes `life <= plus + min(plus, total_mana / 2)` when Rings is ready, with worth 20 on the lethal path.
- About 40 lines and 3 tests. It changes the urza pool deck too, since it runs Ballista and Rings.

## Interaction with the loop

- **The AI never runs the loop step by step.**
  - `combos::power_artifact` (combos.rs:1260-1283) declares it ready when Basalt Monolith and Rings are out, there are {5} (1270-1276), and a mana sink exists; the combo framework then resolves the result.
  - `karn_engines::infinite_mana` (1066-1070) only gates Staff's draw-down shortcut (`staff_options` 1101-1109, `staff_go` with `ability_run`, worth 0).
  - Neither puts the Monolith's untap through an ability window, so a rule in the activation path never sees the loop and cannot double count it.
- **A general "copy whatever is worth X" rule would break two things.** That is why v1 has the exclusions and the guard:
  1. **Untap copies are wasted in this engine.** Basalt's compiled `{3}: untap self` (data/cards.json) is offered to the AI at `0.3 - 0.75 = -0.45` (dsl/mod.rs:2455-2461). That still beats the main loop's "stop" option at -3.0 (ai/brain.rs:1526), so the AI can activate it as filler, up to 3 times a turn (2432-2440). Copies of it, of Tezzeret's 0 and of Voltaic Key would each cost {2} for nothing: the copy resolves first, and the original then finds the permanent untapped. Fix: `dsl_gross` allows only `draw`; Tezzeret 0 and the Key get no hook.
  2. **Spending into the loop.** The {2} can take the AI below the {5} that starts the loop, and `line_enabled_by` / `mana_need` can't see it (above). Fix: the combo guard.
- **`extra_mana_play`.** `lotus_val` uses it, comparing `u` with `u + L`, to choose the -3 (arzakon.rs:190-202), and v1 leaves that choice unchanged. The copy's worth compares `u + L` with `u - 2 + 2L`, so the play the original's token enables is in both sides and is not counted twice. The second Lotus is v1's only copy that pays for itself (+3 of one colour for {2} generic). Its colour shape can strand a coloured spell, which `spend_cost(base, more)` catches.
- **Route A is unchanged** on every board where it was ready: no new code runs in `power_artifact`, and the guard forbids non-lethal copies while it is ready. Test 15 proves it.

## Effects on the sim

**Decks whose play changes.**
- **arzakon** (JD, Cali 1v1, t5/t6). Every hooked ability and Staff's draw. In the old list's 4-player baseline Rings was cast in 29.5% of games (audit/arzakon/FINDINGS.md:74), and the commander comes most games. So copies are plausible in roughly 10-25% of games, mostly the +2 (6 to the face a turn at 30 life, or lethal at 6 or less) and the -3. The effect on its win rate is unmeasured; my guess is 0 to +2 points, possibly inside the noise.
- **urza-mono-blue-artifacts** (pool t5), through Urza's {5} and Aether Spellbomb's `{1}, sacrifice: draw`:
  - The Spellbomb's utility is `1.3 - 0.25 - 1.5 = -0.45` (the sac-self cost at dsl/mod.rs:2457-2459). That beats "stop" at -3.0, so it is cracked as filler, and the generic draw rule then copies it when {2} is spare.
  - Tezzeret the Seeker and Narset get no hook.
  - Its other compiled abilities are the Spellbomb's bounce (targeted: excluded), Basalt's untap (excluded) and Otawara's channel (targeted).
  - Its Ballista is on the plain window.
  - Its full list of non-mana activated abilities: Urza {5}; Tezzeret the Seeker +1/-X; Narset -2; Basalt untap; Isochron Scepter; Sensei's Top (two); Mystic Forge; Manifold Key (two); Aether Spellbomb (two); Mishra's Bauble; Spellskite; Tormod's Crypt; Welding Jar; Walking Ballista (two); Lightning Greaves equip; The One Ring; Emry; Sai; The Reality Chip reconfigure; Uthros station; Inventors' Fair; Academy Ruins; Buried Ruin; Otawara channel (from hand); Urza's Saga's Construct ability; Forensic Gadgeteer's Clues.
- **Any seat whose clone copies a Rings** (Phyrexian Metamorph in urza itself, or another t5 deck's clone at a table with urza). Rare, and covered by "the table contains a Rings deck" below.
- **So every JD t5 baseline moves**: yshtola t5, urdragon t5, yoshimaru t5 and arzakon t5. That is engine drift to disclose, as FINDINGS limit 10 did on 11-23.
  - Leaving Urza out of v1 is not enough to avoid it: the generic draw rule still copies the Spellbomb.
  - Keeping the pool decks unchanged would need no Urza hook and no generic compiled rule, with Staff's draw allowed by card name instead. That is a deck-shaped rule, and the arzakon deck's Urza wouldn't copy either.
  - **I recommend the generic version.** AGENTS.md wants deck-specific AI rules only when a generic one won't do, and a real Urza pilot copies the {5}. Re-measure the t5 baselines when JD approves that run.

**L2 traces that may differ.** `scripts/trace-record.sh --mode l2`, 20 seeds:
- all `arzakon/t5` and `arzakon/t6` records;
- `yshtola/t5`, `urdragon/t5`, `yoshimaru/t5`, and the t5 games among the pool decks alone, but only games whose players include urza-mono-blue-artifacts.

Everything else must be SAME: `sheoldred/t3-t4`, `yoshimaru/t4`, the games among the pool decks alone at t1-t4 and t6, and every t5 game without urza. (Kinnan and Dack have Basalt but no Rings.)

Why a game with urza can differ before any real copy: the look-ahead plays rollouts with re-dealt hands (ai/search.rs:1-11). A rollout in which urza holds Rings may copy there and change another player's decision. So the first differing line is not always a Rings line. It is either a Rings line or a look-ahead decision or play of a player at that table.

**How to prove nothing else moved:**
1. `trace-compare.py base branch --allow arzakon`. Then a small coordinator check for each non-SAME record: its players' deck list contains urza-mono-blue-artifacts, or it is the arzakon seat. Any other non-SAME record fails.
2. For each non-SAME record, re-run that game with the branch binary and `AI_RINGS=0`, by hand: `env -i PATH HOME COMMANDER_SIM_ROOT=<tree> AI_RINGS=0 <bin> --deck D --pool T --seed 500000 --trace k` (trace-record's `env -i` drops the variable, so it can't do this itself). Its log must equal the base log byte for byte. This attributes every difference to the switch.
3. L3 (200 seeds per deck-tier): no new panics, timeouts or games with no winner. This is the "never loop forever" check.
4. Count the `Rings of Brighthearth copies` lines per deck in the branch L2 logs: more than 0 for arzakon and urza, 0 everywhere else.

**Determinism.** Every decision function takes `&Game`, so none can touch `g.rng`. RNG streams change only after a real copy whose effect shuffles or tutors (Urza's shuffle, Tezzeret's tutor). Same seed, same game; test 20 checks that.

**Look-ahead cost.** The cost only applies while the AI has a Rings on the battlefield:
- one scan of p's permanents for each AI activation;
- one `Situation` plus a greedy plan over the hand for each candidate copy;
- one extra effect run for each copy.

I expect under 1% of run time. Compare the `meta.json` timings of the two L2 recordings, and treat more than 5% as a finding. The per-game `game_search_work` budget can run out earlier in games where copies happen, which pushes later decisions onto the heuristic. That is one more source of differences, still only in those games.

## Status and docs

**Is Rings Full for the AI after v1? Not by the letter of the status table.** documents/cards.md:42-48: Full means hand-verified against Oracle, and "Partial: some text … ignored". The AI would still never copy:
- Ballista;
- targeted compiled abilities;
- fetchlands (no stack);
- the abilities with no hook (list above);
- more than one copy per activation (two Rings, through Sculpting Steel);

and it would still pay the {2} before the trigger goes on the stack, as the person's path does (stack.rs:775-779; Oracle pays on resolution).

The card is rated Full today with a stricter gap ("the loop itself resolves as its result"), and the L1 `audit-mine` gate needs 0 cards below Full in JD's decks (scripts/GATES.md). So: **keep Full, rewrite the note, and have JD approve the named limits.** If JD prefers the stricter reading, the honest alternative is Approximate, and the gate would then need JD's waiver for arzakon.

**Lines to change** (notes are not exported, so `cardgen export` should leave `git status` clean; run the export gate anyway):

1. `data/card_overrides.toml:4077-4079`, Rings. Replace the note with:
   - the loop as a combo result;
   - a person is offered every copy;
   - the AI pays {2} for one copy per activation of: Arzakon +2/-3, Tezzeret CC -3/-7, Karn Liberated +4/-3, Karn GC -2, Urza's {5}, compiled draw abilities with no target. The value is the play the {2} costs (`impls/rings_ai.rs`), and there is no copy while a combo is ready;
   - never: the list above.
2. `data/card_overrides.toml:267`, Arzakon. In "Accepted limits", replace "Rings of Brighthearth does not copy its abilities for the AI (a person's copy asks again)" with the new behaviour: +2 and -3 copied, -7 never; the +2 copy takes a new target and the original aims at the face when the pair is lethal.
3. `data/card_overrides.toml:8881` (Tezzeret, Cruel Captain), `2618` (Karn Liberated), `2629` (Karn, the Great Creator), `5402` (Urza), `9469-9472` (Staff): one clause each on what the AI's Rings copies. Staff: the draw, never the untaps.
4. `impls/arzakon.rs:37-38`, the "Left out" paragraph of the module doc; and lines 18-22 for the +2's lethal through a copy.
5. `engine/stack.rs` docs:
   - 723: the section title "(practice mode)" becomes "(a person; the AI's decision is `rings_ai::wants`)";
   - 744 (`rings_of`, person-only);
   - 760-763 (`rings_offer`);
   - 697 (the window comment);
   - 844-848 (`ability_run`: "the AI never does");
   - 884-888 (`ability_run_w`: `worth` is the Camera's);
   - 931-933 (`equip_ability`: still true, Rings included).
6. `state.rs:1185-1187`: the `copy_worth` doc says Camera only; add the doc for `rings_worth`.
7. `decklists/cali_commander/arzakon-artifact-combo.md:20`. Replace "The AI never copies loyalty abilities with Rings of Brighthearth; a person at a real table can." with what the AI copies and never copies. Do not add history (the deck-docs rule). Lines 52-53 and 191-192 are pilot advice and stay.
8. `audit/arzakon/FINDINGS.md`. Add a new dated entry above the 11-23 section (the engine change, the limits JD approves, and the drift). Mark limit 1 (line 29) "superseded <date>: see …" rather than deleting it.
9. The PR body's limits list, for JD: the "never" list, one copy per activation, payment before the trigger, provisional constants, and the t5 drift.

## Implementation plan and tests

**One Sonnet worker, one branch, in this order.** Each step compiles before the next.

1. `state.rs` `Ctx.rings_worth`; `copy_item` clears it. `cargo build`.
2. `engine/mana.rs` `units_minus`; `ai/brain.rs` `greedy_value` and `spend_cost` (next to `extra_mana_play`).
3. The `AI_RINGS` switch (table_fixes `switch!`). New `impls/rings_ai.rs`: `ai_rings`, `ready`, `wants`, `dsl_gross`, the combo guard, the constants (each marked `PROVISIONAL`). Register the module.
4. `engine/stack.rs`: `ability_run_ctx` / `ability_run_rings`; the pre-check in `ability_window_ex` and `ability_window_card_ex`; the AI branch of `rings_offer`; doc lines.
5. `hooks.rs` `CardImpl.walker_copy`. `impls/common.rs`: `walker_pay_run_c`, the hook lookup, `ability_run_rings`.
6. Card hooks:
   - `impls/arzakon.rs`: `plus_plan(exclude)`, the lethal-through-copy test, `copy_plus_target`, the +2 and -3 hooks, `arz_plus_go` through `_c`;
   - `impls/t6_godo.rs`: Tezzeret -3 and -7;
   - `impls/t3.rs`: Karn Liberated +4 and -3;
   - `impls/t5.rs`: Karn GC -2, and Urza's `urza_top` through `ability_run_rings`.
7. Docs and data notes (the list above).
8. Tests (below), rustfmt on the changed files, then the full suite.

**Red-green tests, new file `tests/ai_rings.rs`.** Each test that is a fix must fail on base by assertion: scripts/red-green.sh. Unless a test says otherwise, the setup is AI P0 with Rings and spare mana.

Abilities that are copied:
1. Arzakon +2, opponent at 5 life: the original aims at the face, {2} is paid, the copy kills (game over).
2. Arzakon +2, two opposing 2/2s worth 3 or more: both die (the copy takes the other).
3. Arzakon +2, a 2/2 and nothing else worth 3: the 2/2 dies, the copy hits the face.
4. Arzakon -3 with {2} spare: two Lotus tokens; loyalty 4 → 1 once; `loyalty_used` set once; `uses() == 1`.
5. Arzakon at 3 loyalty, -3: Arzakon dies, both tokens are still made.
6. Arzakon -7: never copied.
7. Tezzeret CC -3 with 2 or more targets: two cards to hand. With 1 target: no copy, mana unchanged.
8. Tezzeret CC 0: never copied.
9. Karn Liberated -3 with two opposing permanents of pval 4 or more: both exiled. With one: no copy.
10. Karn Liberated +4, opponent with 3 cards: it loses 2.
11. Karn GC +1: no copy.
12. Urza {5} with 7 mana: two free cards. With 5 mana: one card, no payment.
13. Staff {5},{T}: draw, through `dsl::activate`, with {2} spare: two cards. Aether Spellbomb's {1} draw: two cards. Staff {1} untap, Staff {3} untap a creature, and Basalt's {3} untap: never.

When the AI declines:

14. No copy when the {2} would leave a castable card in hand (greedy value) unpaid, or a held counterspell (`hold_value`) unpayable.
15. Basalt + Rings + Ballista + {5}: `power_artifact` ready and resolved exactly as before, with no `Rings of Brighthearth copies` line. Basalt + Rings out with total mana 6: a non-lethal +2 is not copied.
16. No Rings, or 1 mana: no copy, `stack_pushes` unchanged, no Rings line, `trig_copies` empty.

Other seats and edge cases:

17. Two Rings (Sculpting Steel as Rings): exactly one AI copy per activation (v1 limit); no copy is made of a copy.
18. Tishana's Tidebinder in an opponent's hand: the trigger or the copy can be answered; if it is, no copy resolves (the payment timing is documented).
19. `AI_RINGS=0` (`force_rings(Some(false))`): tests 1, 4 and 12 play as on base.
20. Same seed twice, with copies: identical logs.
21. A person's path is unchanged: `practice_rings_probe.rs` and `cards_arzakon.rs::a_rings_of_brighthearth_copy_of_the_persons_plus_two_asks_for_a_new_target` pass untouched; `EXCEPTIONS` unchanged (frozen allow-list).
22. Walking Ballista ping with Rings: no copy (v1). Turn this into B1's tests if JD asks for B1.

**Checks before review.** All under the memory cap, in the worktree's own target dir:
- `cargo test --release -p sim-core`, then `-p practice` (an AI with Rings now copies in front of a person), then the workspace.
- `scripts/stage-gate.sh` L0/L1: suite, ignored, names, diffcheck, fmt, audit-mine, audit-paths, validate, export, allowlists, newconsts.
- The coordinator runs L2 and L3 base vs branch, with the checks under "How to prove nothing else moved".

Expected diff: about 450-600 lines of code and docs across state.rs, mana.rs, brain.rs, table_fixes.rs, rings_ai.rs (new), stack.rs, hooks.rs, common.rs, arzakon.rs, t6_godo.rs, t3.rs and t5.rs; about 350 lines of tests; 6 doc and data files.

Existing tests that put an AI Rings on the battlefield may change (cards_karn_verify.rs `staff_table`, 422-429, together with any compiled Staff draw). Each must be re-justified against Oracle, never rewritten to match.

**Effort.** 6-8 Sonnet worker hours, plus 2-3 h of adversarial review and an Opus final review. B1 adds about 1.5 h.

**What the independent adversarial reviewer should attack:**
- **Unbounded loops.** Two Rings; Sculpting Steel copying Rings; a copy of an ability that untaps Basalt (Voltaic Key, Tezzeret 0) with Rings; Urza's {5} with a deck of all-free artifacts; L3 timeouts.
- **Mana.**
  - Does `pay` tap sources other than the ones `plan_pay` assumed, stranding a coloured spell?
  - Is a banked Lotus token sacrificed to pay {2}, wasting a mana?
  - Pool mana and Mana Vault / Basalt left floating.
  - The combo guard at exactly {5} and {7}.
- **Bookkeeping.** Loyalty paid once; `uses`, `act_uses` and `loyalty_used` after a copy; a walker at 0 whose copy still resolves; Tezzeret's artifact-enters loyalty trigger from two Lotus tokens.
- **Targets.** The +2 copy never hits Arzakon, the AI's own face, a hexproof or protected permanent, or a target that has left; behaviour when the original is countered; the lethal-through-copy aim with exactly 1 mana.
- **Isolation.**
  - Byte-identity where there is no Rings: L2 SAME outside tables with a Rings deck, and the `AI_RINGS=0` re-runs.
  - The Camera's decisions unchanged (`copy_worth` untouched).
  - The person's path, and practice mode with an AI Rings opposite a person.
- **Gates.** newconsts names; the frozen `EXCEPTIONS`; no test deleted.

## Risks

1. **The look-ahead spreads the change to every game where a Rings deck sits at the table.** Rollouts with re-dealt hands can copy, which changes evaluations and decisions even when no copy is made in the real game. Line-by-line attribution is impossible. Only "SAME outside those tables" and the `AI_RINGS=0` re-runs prove isolation.
2. **t5 baselines move for yshtola, urdragon, yoshimaru and arzakon** (the Urza pool deck's {5} and Spellbomb copies). Any paired comparison against an older t5 baseline mixes this drift in.
3. **Every constant is a guess** (margin, DSL minimum, bank value, Urza, tutor, emblem, Karn), and so is the greedy "what {2} costs".
   - The greedy sum ignores mana sinks among activated abilities (Urza's {5} again, Staff draws, Ballista's {4}), next-turn plans, and the opponents' turns, except for counters and protection held through `hold_value`.
   - Copies can starve those plays, or be skipped where a person would copy.
4. **`spend_cost` prices with `plan_pay`, but `pay` taps what it chooses.** A copy judged safe can strand a coloured spell. The Lotus token "spent like Lotus Petal" (accepted limit 6) may be burned on a {2}.
5. **Timing deviation, as on the person's path:** the {2} is paid before the Rings trigger goes on the stack. A countered trigger loses the {2}, while Oracle pays on resolution.
6. **One copy per activation for the AI.** A second Rings (Sculpting Steel, which the guide names as Route A's first Steel target) adds nothing outside the loop.
7. **The +2's target choice anticipates the copy only for a lethal pair.** A 4-6 toughness creature that the pair would kill is never chosen.
8. **Tests encode today's behaviour.** Any existing test with an AI Rings on the battlefield and a compiled draw or a hooked ability will turn red. The risk is a worker "fixing" the test instead of checking it against Oracle.
9. **Practice mode changes visibly:** an AI opponent with Rings now puts a trigger and a copy on the stack, and the person gets priority on them. UI and practice tests may need updates.
10. **Arzakon's own AI rules are already provisional and untested** (FINDINGS limit 7). The copy multiplies whatever the +2 and -3 rules get wrong (more Lotus tokens wasted, more face damage instead of a kill).
11. **A real but pre-existing issue surfaced here:** the Camera copying a targeted compiled ability wastes it, because the copy takes the original's target. Not in any deck today.

## What I could not determine

- How often copies happen, and the effect on any win rate: nothing was simulated, and the old 4-player baseline is a different format and list. The 10-25% and 0 to +2 point figures are judgement, not measurement.
- Whether `mana::pay` (mana.rs:739) picks the same sources as `plan_pay` for a {2} on these boards. I did not trace its source ordering.
- Which existing tests go red. I did not run the suite. The candidates are cards_karn_verify.rs (422-445 area), cards_arzakon.rs and thrasios_ai.rs.
- How often the AI actually takes the filler abilities (Basalt's untap, the Spellbomb's draw, Staff's five abilities) in real games. Their options beat "stop" (-3.0, ai/brain.rs:1526; `rules::filter_options` filters only casts, ai/rules.rs:94-123), but I did not count them in any trace. If they are frequent, the Spellbomb's draw copies are a larger share of the Urza deck's change than Urza's {5}.
- Exactly which L2 seeds put urza at a t5 table: that is known only from the recordings.
- The look-ahead cost in numbers: the under-1% figure is an estimate.
- Whether trace-record could carry `AI_RINGS`. It uses `env -i`, so the off-switch proof is a manual re-run of the non-SAME games, not a second recording, unless the recorder gets a pass-through flag. That is a separate change to `scripts/`, and it needs JD.
