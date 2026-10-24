# Task: silverblade — model Silverblade Paladin (soulbond) and pair it with Cloud

Read `rust/agents/T0-RULES.md` (Hard rules, How cards are modeled, Checks before you finish items 1-3). Branch
cloud-kaldra (Cloud's current list + JD's next swaps in audit/cloud/packages/kaldra-out.swaps). `-j 16`. ~20 min;
JD is waiting. No win-rate runs.

Silverblade Paladin ({1}{W}{W} 2/2 Human Knight): "Soulbond (You may pair this creature with another unpaired creature
when either enters. They remain paired for as long as you control both of them.) As long as Silverblade Paladin is
paired with another creature, both creatures have double strike." Today it is Partial (body only).
- Model soulbond generically if small (pairing on either's entry, unpair when either leaves or changes control; a
  `DataKey` pair link), else for this card; grep first (soulbond may exist for another card).
- Double strike for both while paired must use the engine's double-strike path (two damage steps since 10-15).
- AI: pair with the best creature — an equipped Cloud first (his Sword triggers then fire 4 times), else the creature
  carrying the most Equipment; when Cloud enters while the Paladin is unpaired, pair them.
- Tests: pairing on the Paladin's entry and on a later creature's entry; both get double strike; unpair when one
  leaves; Cloud with a Sword paired = 4 Sword triggers per attack.
- Mark Full in data/card_overrides.toml. `cardgen export`, `cardgen validate`, full `cargo test --release -j 16
  --no-fail-fast`, `--all-targets` no warnings. If you touch shared engine code, a same-seed identity check of the
  other 10 JD decks (parent binary in a detached worktree under .claude/worktrees/, removed after; `--deck <d> --pool
  all --games 300 --ai adaptive --jobs 16 --seed 7 --brief`). Commit (no Co-Authored-By), never push.
Report ≤8 lines: line 1 DONE/HANDOFF, line 2 model ID.
