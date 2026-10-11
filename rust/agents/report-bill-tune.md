# Report: bill-tune (branch models-bill-tune, base bill-deck)

Result: DONE. Changes: impls/bill.rs, ai/mod.rs (shared, 5 lines), tests/cards_bill.rs.
- `damage_through` / `lethal_victim` (bill.rs): the pre-combat "(lethal attack)" doubling now counts only damage that gets
  through blockers: each untapped opposing creature that `combat::can_block` allows chumps one of the biggest attackers
  (menace takes two blockers, trample passes the excess over toughness). Greedy assignment, smallest-toughness blockers first.
- `double_lethal` sets a per-turn flag naming the victim; `ai::choose_attack` (gated to key "bill") returns (victim, "all")
  when the flag is set, so the attack after a lethal doubling is all-out and the search/filter cannot cancel it.
- The "bigger attack" pre-combat doubling was removed: default timing is the end of the opponent's turn only (item 3).
- Tests: `lethal_doubling_not_offered_when_chump_blockers_stop_the_attack`, `lethal_doubling_taken_through_blockers_forces_the_attack`.
- Traces (seed 500000, lookahead, games 0,1,2): no "(lethal attack)" doubling followed by no attack. Games 1 and 2 are now won.
- 240 games t1/loose/lookahead seed 500000: win rate 10.0% (95% CI 6.8-14.4%); 13.0% on 960 earlier. Inside noise, not a clear gain.
  The change removes wasted mana but also the "bigger attack" option.
- Checks: cargo test --release all green; build --all-targets no warnings; all_decks adaptive 3 and lookahead 1 exit 0;
  audit `bill: 67 unique cards, 0 not Full`.
