# Report: trigger fixes (practice mode)

Branch practice-trigger-fixes. Tests: `practice_trigger_fixes` (10), `practice_trigger_probe` (activated pass now default), and the
existing files for code touched (practice_gaps_a, practice_followups, practice_ability_probe, practice_urdragon_loops, cards_garth,
cards_karn_engines) all pass. No sims, no full suite. **AI paths unchanged** in every fix (all in `is_human` branches).

## Done
| Card | Fix | Files | Tests |
|---|---|---|---|
| Felidar Retreat | person: trigger window, then "choose a mode": 2/2 Cat, or +1/+1 counter on each creature you control (through `repl::put_plus`) and vigilance until end of turn | `engine/zones.rs` `landfall_once` | `felidar_retreat_person_picks_the_counters_mode`, `..._picks_the_cat` |
| Sun Titan | person: window, then pick one permanent card with MV <= 3 from the graveyard, or "none"; a land enters with `land_enters_tapped` and fires landfall (the AI path still enters it tapped, no landfall). `sun_titan` now takes the source perm (`sun_titan(g, p, src)`; 2 call sites) | `engine/zones.rs`, `engine/combat.rs` | `sun_titan_returns_the_card_the_person_picks`, `..._may_return_nothing`, `..._ai_still_takes_the_best` |
| Mirrorworks | person: no worth filter; window, then yes/no "Mirrorworks: pay {2} to create a token copy of X?" and payment by `pay_tax_ask` (pool first, then taps sources). Before, the person's check `can_pay` (pool only) meant the trigger never showed with an empty pool | `impls/karn_engines.rs` `mirror_etb` | `mirrorworks_asks_the_person_before_paying`, `..._pays_and_copies_when_the_person_says_yes` |
| Vito (cosmetic) | the which-opponent question is headed "Vito, Thorn of the Dusk Rose:" (`t4::bond_target_for`; `bond_target` is the Sanguine Bond wrapper); off FALSE_ALARMS | `impls/t4.rs`, `impls/t45_edgar.rs` | the probe now sees it asked |
| Garth (1) | "you may cast the copy": after the name, "Garth One-Eye: cast the copy of X?" with "don't cast it"; declined -> name stays used, copy ceases to exist, nothing paid | `human/gaps_a.rs` `garth_person` | `garths_copy_is_not_cast_when_the_person_declines`, `..._is_cast_when_the_person_accepts` |
| Garth (2) | Braingeyser's X is capped by `mana::available` (pool + untapped sources) and the "can't pay" label counts sources; the person's payment stays pool-only (`cast_copy_at` -> `pay_from_pool`), so a new `human::mana::tap_to_pay` taps sources into the pool (a source making a missing colour first) just before the cast. Colour fit is greedy, not exact: X is capped by the total, so a colour shortfall makes the copy cease (logged) | `human/mana.rs`, `human/gaps_a.rs` | `garths_braingeyser_x_counts_untapped_lands_and_taps_them` |
| garth.rs doc | stale "Left out: a person's Garth ..." line fixed | `impls/garth.rs` | - |

## Activated-ability pass: made the default
`PROBE_ACTIVATE` is gone; `PROBE_NO_ACTIVATE=1` turns the pass off. Two probe corrections, both reducing false hits: (a) an
activation counts as fired only if "You activate <card>" showed (a refused ability, whose only question was "which ability?",
used to count); (b) for Activate runs P0 also gets Sol Ring, Arcane Signet and Mind Stone (Inventors' Fair and Kuldotha
Forgemaster need three artifacts; both now fire and ask). Result: 201 pairs, 161 fired and asked, 17 flagged (8 TO_FIX, 9 false
alarms), 23 unprobed.

Verdicts (code read for those marked "read"; the rest are from the probe's run only):
- **Real, NOT fixed (TO_FIX with notes in the probe; 8 entries, ~12 abilities)**: Karn Liberated +4/-3 (read: `impls/t3.rs`
  karn_plus removes the worst card of the opponent with most cards, no target player, no exile; karn_minus picks best_opp_nonland),
  Ugin the Ineffable -3, Karn Scion of Urza +1, Karn the Great Creator +1/-2, Teferi Time Raveler -3 (read: `ttr_minus`),
  Wrenn and Six +1/-1, Venser the Sojourner +2, Blightspeaker. Why not fixed: a human entry in `human/cards.rs` `walker(...)`
  replaces the walker's whole generic list, so each fix must re-list every ability (ultimates too) with its choices; I stopped
  at a clean point instead of starting half of them. Suggested shape: Vraska / Ral Zarek in `human/cards.rs` (`Ready::Go` after
  the choice, `pick_perm` / `any_player`).
- **False alarms**: Crystal Chimes (no choice), Judge's Familiar and Tragic Poet (the probe's table lacks a spell to counter / an
  enchantment in the graveyard; not read), Sakura-Tribe Elder in urdragon (the person IS asked in `land_ramp`; urdragon has one
  basic, a Forest the table already moved away).
- **Not flagged any more (ask, or fine)**: Inventors' Fair, Kuldotha Forgemaster, Lin Sivvi (asks X and the search).
- **Unprobed (table lacks the precondition; 15 listed with why)**: Angelic Page, Ballista Squad, Mishra's Foundry/Factory, Cid,
  Plaza of Heroes, Lawbringer, Lightbringer, Ramosian Revivalist, Walking Ballista (x2), Peter Parker's Camera, Dauthi Voidwalker
  (these two ask in `human/urdragon.rs` / `gaps_c.rs`), Maelstrom of the Spirit Dragon, Loran of the Third Path.

## Anything that would change the AI
Nothing intended. `sun_titan`'s signature gained `src`; its AI branch is the old code. `human::mana::tap_to_pay` is new and used
only by the person's Garth.

## For the coordinator
- Next task: the 8 TO_FIX walker/ability entries (remove each key from `TO_FIX` as it is fixed; the test fails on stale ones).
- Sun Titan returns a land to a person untapped unless it enters tapped by its own text, with landfall; the AI keeps entering it tapped (not changed).
- Not rustfmt'd: `practice_trigger_probe.rs` (not mine); new test file is.
