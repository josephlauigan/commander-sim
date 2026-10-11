# Task: bill-tune — fix Bristly Bill's doubling/attack mismatch

Read `rust/agents/OVERNIGHT-RULES.md` first: its hard rules (never run Python, memory cap, edit only your files,
commit often, handoff, 15-line report) apply here. Where it says "Kefka/Zada", read "Bill".

Base branch: `bill-deck` (3dfe55b plus the measurement). Deck key `bill`, module `impls/bill.rs`, tests
`tests/cards_bill.rs`. The previous agent's notes: `rust/agents/report-bill-cards.md`.

## The problem (from traces against Tier 1)
Reproduce with `./target/release/commander-sim --deck bill --pool t1 --profile loose --games 1 --seed 500000
--ai lookahead --trace N` for N = 0, 1, 2.
- Game 0, round 9: Bill doubles twice "(lethal attack)" (36 -> 72 -> 144 counters), then the attack plan is
  "filtered at Tatyova", keeps 3 creatures home, and **never attacks**. He dies the next turn.
- Game 1, round 14 and game 2, round 11: the same, "doubles ... (lethal attack)" followed by "attack plan: none".
The "lethal attack" test in `bill.rs` (the doubling option before combat) counts power against the lowest life but
ignores blockers: opponents with many creatures (Tatyova's, Isshin's and Teysa's tokens) chump the big non-trampling
attackers. So mana is spent on a doubling that the attack planner then doesn't use.

## What to change (all in `impls/bill.rs` unless a hook is truly needed; gate any shared change to `key == "bill"`)
1. **Lethal check with blockers.** Count only damage that gets through: assume each untapped opposing creature that
   can block chumps one of the biggest attackers it can legally block (respect flying/reach, Silhana Ledgewalker,
   Paladin of Predation's power-2 rule, menace); trample adds the excess over the blocker's toughness. Use the
   engine's own `combat::can_block` / evasion helpers rather than re-deriving rules.
2. **After a lethal doubling, the attack happens.** If the doubling was taken for a lethal attack, the attack plan
   that combat must be all-out at that player (find how a plan is forced: `ai::choose_attack`, the look-ahead's
   forced plans). If it can't be forced cleanly, don't offer the pre-combat lethal doubling at all.
3. **Default timing:** with no lethal attack, double at the end of the opponent's turn just before yours (as now),
   not before combat for a "bigger attack" unless the attack plan will actually attack.
Add tests for 1 and 2 (a board where the raw sum is lethal but chump blockers stop it: no doubling offered; a board
where it's lethal through blockers: the doubling is taken and the attack is made).

## Done when
The OVERNIGHT-RULES checks pass (full `cargo test --release`, no warnings, `all_decks adaptive 3` and `lookahead 1`,
audit `bill: 0 not Full`), the three traces above no longer show a "(lethal attack)" doubling followed by no attack,
and a 240-game check `--deck bill --pool t1 --profile loose --games 240 --seed 500000 --ai lookahead --jobs 22`
finishes (report its win rate; it's 13.0% on 960 games before). Report as the rules say: line 1 DONE or HANDOFF,
line 2 your model ID.
