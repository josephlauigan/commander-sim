# Brief: Phase 3, PRs 4 and 5: a mana pool with restrictions; identity for exiled cards

Two PRs, done one after the other, each on its own branch:
- **PR 4** in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase3-mana`, branch
  `phase3-mana`, stacked on `phase3-repl` (Phase 3 PR 3; PRs 1-3 are not merged yet).
- **PR 5**: when PR 4 is committed and its checks pass, create the worktree yourself:
  `git -C /home/joseph/projects/mtg_sims/commander-sim worktree add -b phase3-exile
  /home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase3-exile phase3-mana`, and work there.

Read `CLAUDE.md` first: memory cap, no Python. You are the only agent: `-j 22` for cargo.

## JD's rules for Phase 3 (2026-10-29)

- **No simulation runs of any kind** until Phase 4: no commander-sim realism / deck / goldfish / trace runs. Checks
  are the cargo test suite (release, all pass, 0 warnings) and the practice crate's tests. Microbenchmarks in
  `rust/crates/sim-core/examples/` are fine.
- Phase 3's gate is "the test suite passes". JD recalibrates once, after Phase 4.

## Background

`documents/realism.md`, Phase 3. Done: one permanent arena (PR 1), the characteristics pass (PR 2,
`engine/chars.rs`), one replacement step (PR 3, `engine/repl.rs`). Read their designs in `audit/phase3/` and follow
the same staging: design, then results identical, then behaviour fixes with tests.

## PR 4: a mana pool with restrictions

Today `engine/mana.rs` models mana as `Unit { src: Source, cols, amt }` units found and paid at cast time, with
floating mana as `Source` variants (`FloatR`, `FloatAny`, `FloatU`, `FloatC`, `FloatG`, `FloatB`, and `FloatArt` for
Karn, Legacy Reforged's "only on artifact spells"), Treasures, Scions, hand cards, and per-card special cases. Find
every restriction or rider on mana today (spend only on creature spells / artifact spells / abilities, "can't be
countered" mana, Powerstone's "can't be spent on nonartifact spells", Food / Gold / mana with riders, floating mana
that empties between steps, Birgi, Kinnan, rituals) and every place mana is added or paid.

1. **Design** (`audit/phase3/mana-design.md`): a real mana pool per player of mana objects (colour or colourless,
   amount, the source kind, an optional restriction and an optional rider), emptied at the end of each step and phase
   (rule 500.4) unless an effect keeps it; how paying a cost picks from the pool and from untapped sources (the
   current search) without making the AI worse or slower; what each current `Float*` variant and special case maps
   to; the speed plan (`clone_cost` and a new `examples/mana_cost` microbenchmark).
2. **Stage A, results identical:** the pool exists and the floating variants become pool entries read through it.
3. **Stage B:** the restriction special cases become pool restrictions; old code deleted. Still identical.
4. **Stage C, behaviour (tests required):** fix what the design found (a restriction not enforced, mana that should
   empty and doesn't, or the reverse). List tiers / JD decks (`data/my_decks.toml`) touched, for the recalibration.

## PR 5: identity for exiled cards

Today `Player::exile` is a `Vec<CardId>`, and effects that care which exiled card is which ("exile ... until ~ leaves
the battlefield" (Static Net, Oath of Teferi, the O-Ring family), "exile the top card; you may play it this turn"
(impulse draw, Laelia), silver counters (Karn, Scion of Urza), foretell, plot, adventure, suspend, cascade-to-exile,
"cards exiled with ~") each track it their own way (`DataKey`s, per-card fields; about 125 sites). Phase 2 listed some
as follow-ups (`rust/agents/report-phase2-doubles.md`, "Follow-ups for the ability language").

1. **Design** (`audit/phase3/exile-design.md`): an exiled card as an object with an id, and its links (exiled by
   which permanent or effect, until when, who may play it and until when, face up or down, counters on it); a card
   that leaves exile and comes back is a new object (rule 400.7); how each current tracking maps onto it.
2. **Stage A, results identical; Stage B, call sites; Stage C, behaviour (tests required)**, as above. Stage C should
   at least make "until ~ leaves" return the right card, and impulse draw's "until end of turn" expire.

## Checks (every stage of both PRs)

- `cargo test --release --no-fail-fast -j 22` passes; `cargo build --release --all-targets -j 22` 0 warnings.
- `cardgen export --offline` + `cardgen validate` pass; `cardgen audit --paths` 0 doubles; `data/paths_known.txt`
  stays empty.
- The practice crate builds and its tests pass (JD plays it): the pool and exile changes may show in its UI; keep
  it working and say what changed there.
- Microbenchmarks before and after: say if it is slower and by how much.

## Rules

- Card data only through `data/card_overrides.toml` and `cardgen export`; no deck list changes.
- Don't seat `omnath` or `godo` in new tests. Format only new files (`rustfmt --edition 2024 <file>`, never
  `cargo fmt`), lines up to 120. Mechanical renames with `sed` are fine; check the diff.
- Commit at each stage (and more often), no Co-Authored-By line, never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase3-mana/` and `.../phase3-exile/`.
- Keep `audit/phase3/mana-handoff.md` and `audit/phase3/exile-handoff.md` current, so the next agent can continue.
- If PR 4 takes everything you have, stop after it cleanly and say so: PR 5 can be a separate agent.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report files from
  subagents), one section per PR: what each stage did, the tests added, the behaviour changes and which tiers / JD
  decks they touch, the microbenchmark numbers, and what doesn't fit yet.
