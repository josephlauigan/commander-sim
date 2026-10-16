# Task: cloud-spread — keep all Kaldra pieces in the wide list, and teach Cloud's AI to spread Equipment

Read `rust/agents/T0-RULES.md` (Hard rules, How cards are modeled, Checks before you finish items 1-3) and
`rust/agents/report-cloud-wide.md` (your predecessor's design and AI findings, with file:line). Branch cloud-wide.
`-j 16` builds, ≤300-game smoke runs (no win-rate runs; the lead runs them). Handoff `rust/agents/handoff-cloud-spread.md`,
report `rust/agents/report-cloud-spread.md`. Every background wait has a hard timeout; leave nothing running.

## 1. List: JD wants ALL three Kaldra pieces (Sword, Shield, Helm of Kaldra) in.
`audit/cloud/wide/wide.swaps` currently cuts Helm of Kaldra (=> Skullclamp) and Shield of Kaldra (=> Lightning Greaves).
Keep Helm and Shield; find two other cuts for Skullclamp and Lightning Greaves using the data (the `--swap` breakdown /
`--analyze` card report: lowest cast%, highest stuck%, least impact). Never cut: lands, mana rocks, Package D cards
(Crawlspace, Norn's Annex, Archon of Absolution, Windborn Muse, Baird, Wrath of God, Day of Judgment, Fumigate), the
Kaldra pieces, Swords, or the new wide-plan cards. Update wide.swaps and audit/cloud/wide/list.txt; say what you cut and why.

## 2. AI: spread Equipment onto the right bodies (gated to the cloud deck)
In the predecessor's traces 62 of 84 equips went onto Cloud; Kemba never wore anything; Kor Outfitter / Ardenn moved
living weapons off their Germs; cloud_pick never aims at the new Equipment; new cards were cast late. The pieces named:
holder_bonus (+20), equip_utility_bonus, holder_score (+50), wants_move, best_holder, cloud_pick (impls/cloud.rs, dsl
equip_options). Fix so that:
- Equipment goes where it does the most: Cloud still gets trigger Equipment (his doubling), but a second Sword, stat
  Equipment, Skullclamp (on 1-toughness bodies) and Kemba (wants Equipment for her Cats) are spread; evasive /
  double-strike / protected bodies (Mirran Crusader, Selfless Spirit, Kor Skyfisher fliers) are valued as carriers.
- Never move a living weapon off its Germ unless the Germ would survive (or a far better use exists).
- cloud_pick (Cloud's ETB tutor) can fetch the new Equipment (Batterskull, Skullclamp, Lightning Greaves) when the
  board needs it.
- Keep the change gated to the cloud deck; today's list should play about the same (the lead measures it).
Tests for each rule. Trace 10 games of the wide list after the fix and report the new spread (equips on Cloud vs others).

## Checks
Full `cargo test --release -j 16 --no-fail-fast`; `--all-targets` build with no warnings; same-seed identity for the other
10 JD decks (build the parent commit's binary in a detached worktree under .claude/worktrees/, removed after;
`--deck <d> --pool all --games 300 --ai adaptive --jobs 16 --seed 7 --brief`). Commit (no Co-Authored-By), never push.
Report ≤15 lines: line 1 DONE/HANDOFF, line 2 model ID, the two new cuts, the AI changes, the new spread numbers.
