# Report: cloud-sequencing (diagnosis only), 2026-10-15

Question: are Cloud's cheap ramp and tutors cast late because they are drawn late, or because the AI holds them?
Answer: the AI holds them. Part is the look-ahead search, which gives a mana rock no credit.

## Method
240 traced games (seeds 0-239), `--deck cloud --pool t1 --profile loose`, the five M-package swaps
(`audit/cloud/packages/m-mana.swaps`). A temporary, uncommitted one-line log of Cloud's hand and mana at the start of
each main phase (engine/turn.rs `step_main1`) gave the turn each card first appeared in hand. Reverted; the tree is
clean. A "wait turn" = a turn the card sat in hand while Cloud had mana >= its cost (land already played).
Scripts and traces: the session scratchpad (not committed).

## Numbers (look-ahead, 240 games)
| card | median delay (turns, draw to cast) | waited >= 2 turns |
|---|---|---|
| Sol Ring | 0 | 23% |
| Arcane Signet | 1 | 50% |
| Mind Stone / Fellwar / Thought Vessel | 2 / 2 / 2 | 51 / 53 / 53% |
| Worn Powerstone | 2 | 56% |
| Burnished Hart | 2 (mean 2.9) | 52% |
| Stoneforge Mystic | 2 | 61% |
| Smothering Tithe | 2 | 60% |
Of 399 rock wait-turns with the mana to cast: 159 Cloud himself was cast (commander recast with tax, often after
removal; defensible), 225 a different card was cast, only 15 idle. So it is a choice, not held-up mana.
Examples: Signet and Fellwar both in hand, mana 3-4: Bastion Protector, Baird, Shield of Kaldra cast instead, twice
"stop" chosen over a rock.

## Cause
1. Priorities are fine: rocks/dorks/Lr are `generic_prio` 85 for turns <= 5 (`ai/pool.rs:130`), Sol Ring 88
   (`ai/pool.rs:121`), above every Equipment (<= 84 in `cloud_prio`, `impls/cloud.rs:1005`). Heuristic option
   scores favour the rocks (e.g. "Fellwar 48%, Signet 48%, Bastion Protector 4%").
2. The look-ahead then overrides them. `ai/search.rs:269 choose_inner` plays each candidate, lets the heuristic
   finish the phase, plays on to the end of Cloud's next turn (horizon 1, 6 rollouts) and scores `strength`
   (`search.rs:223`): life, board `pval`, power, 0.9/hand card, 0.6/land. A rock's `pval` is ~0 (`engine/values.rs:409`,
   no mana credit), and casting it costs 0.9 (hand), so it scores below a creature or Equipment (pval + power) and
   sometimes below "stop". The only Cloud-specific bonus (`cloud.rs:1571 search_bonus`) covers Cloud, protection and equips.
3. Control: the heuristic-only AI (`--ai adaptive`, same 240 games) has 238 rock wait-turns vs 399 with look-ahead
   (Sol Ring: 6 vs 37). The remaining delay is mostly Cloud's own casts and 3-4 drops.

## Smallest fix (cloud-only, not applied)
In `search_bonus` (`impls/cloud.rs:1571`) add an arm before the protect arm:
`Some(Action::Cast{card,..})` when `turns <= 6` and the card is tagged Rock/Dork/Lr or is Burnished Hart, Weathered
Wayfarer or Stoneforge Mystic: `=> 6.0`. Tested locally (reverted): rock wait-turns 399 -> 232 (about the adaptive
level), mean delay Sol Ring 0.79->0.52, Signet 1.68->1.19, Mind Stone 1.89->1.11, Powerstone 1.67->1.24, SFM
2.03->0.90, Hart 2.9->2.3; >=2-turn waits Mind Stone 51->40%, SFM 61->30%. Win-rate effect NOT measured (needs JD's
go): expect mana on turn 4 up a little (~+0.2 to 0.4) and plan timing slightly earlier; tune 6 (try 4 and 8).
Same flaw likely hits other decks' rocks under look-ahead (not checked).

## Other sequencing notes
- Cloud's recast turns eat the mana (159 of 399 rock waits); that is the plan, but it makes ramp worth most on turns
  2-3, before Cloud is removed.
- Burnished Hart (4 mana) and Smothering Tithe (3) lose to cheaper plays more often than the 2-mana rocks; the bonus
  helps Hart little (still 62% waited >= 2).
- Equip vs cast was not examined in depth.
