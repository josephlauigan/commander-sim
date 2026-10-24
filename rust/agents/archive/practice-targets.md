# Task: practice-targets, the person picks targets and modes for resolve-coded spells

Read `rust/agents/practice-fixes-common.md` first, with two changes: **build with `-j 3` and `--test-threads=3`**
(another agent runs beside you), and the audit is no longer in a worktree: read it with
`git show origin/practice-human-audit:audit/practice-human/REPORT.md` (and `findings-g1.md` ... `findings-g5.md`,
`jd-cards.tsv` the same way). Worktree: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/practice-targets`,
branch `practice-targets`. JD's decks are in `data/decks.json` (`.mine[]`; `titania` is now `omnath`).

## The problem (REPORT.md cause 3)
`human::legal::spell_targets` gives the person a target only when the card has `Tag::Rem`. Spells with their own
`resolve` hook but no such tag resolve with an empty `Ctx`: they do nothing, or their code picks for the person.
- **Do nothing:** Lethal Throwdown (sheoldred: sacrifice a creature as an additional cost, destroy target creature or
  planeswalker, draw if the sacrificed one was modified), Presumed Dead (sephfs), Stargaze (sephfs: X = 0), Grab the
  Prize (kefka: the discard is never asked, the 2 damage never happens), Dark Endurance (kraum; verify).
- **Mode or target forced:** Splatter Technique (veyran: always the wipe, never draw four), Blood on the Snow (sephfs:
  always creatures, X = 0 for the return), Extinction Event (odd/even), Comet Storm (sauron: AI picks every target and
  ignores the person's), Cost of Brilliance / Painful Lesson / Peer into the Abyss (always target the caster),
  Rewrite Regrets, Daydream, Helping Hand, Necromantic Selection, Victimize, Scout the City, Glorious Decay (now cut),
  Decorum Dissertation, Vault Plunderer, Bojuka Bog, Farseek, Inferno Titan's ETB/attack damage, Smaug's attack.
The findings files give file:line for each.

## Do
1. **Read first** how a person's cast builds its `Ctx` (human/human.rs cast flow, `legal::spell_targets`,
   `choices::pick_modes`, X), and how a resolve hook reads `ctx.target` / `ctx.mode` / `ctx.x`. Write the plan in the
   handoff before coding.
2. Add a general way for a card to declare what the person must choose when casting it (a small spec per card: target
   kind (creature, creature-or-planeswalker, player, any target, card in a graveyard), modes, X, an additional cost
   like "sacrifice a creature" or "discard a card"), read by the person's cast flow, filling the same `Ctx` fields the
   AI's plan fills. Then give each card above its spec. Divided damage (Comet Storm, Inferno Titan) asks one target
   per point or per kick.
3. Card code that picks for the person at resolution (Inferno Titan, Smaug, Bojuka Bog, Farseek) gets a person branch
   using `choices::` (the AI branch unchanged).
4. Never change the AI path.

## Tests
`tests/play_targets.rs`: a person casts Lethal Throwdown (picks the sacrifice and the target; it is destroyed), Splatter
Technique choosing draw four, Comet Storm with two targets, Peer into the Abyss on an opponent, Inferno Titan dividing
3 damage. Plus the common rules' before/after identical sims.

## Files owned
The person's cast flow additions (`human/human.rs`, `human/legal.rs`, `human/choices.rs`), the per-card specs, the
listed cards' person branches, the new test, `rust/agents/{handoff,report}-practice-targets.md`. Commit message for
the last commit: "Practice mode: the person picks targets and modes for resolve-coded spells (N cards)".
