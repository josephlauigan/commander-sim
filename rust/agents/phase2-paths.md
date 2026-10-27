# Brief: Phase 2, PR 1: `cardgen audit --paths` (report only)

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase2` on branch `phase2-one-model`, which is based on main. Run every command from
there. Read `CLAUDE.md` first: memory cap, no Python. Do not use `rust/agents/RULES.md`.

## Why

Phase 2 of the realism plan is "one card model". A card's behaviour reaches the sim by up to seven paths, and nothing
stops two of them from running the same ability:
1. the ability language (DSL) compiled from Oracle text (`CardDef.abilities`, run by `sim-core/src/dsl/mod.rs`);
2. regex autotags (`cardgen/src/autotag.rs`), used only when nothing compiles;
3. hand tags and overrides (`data/carddb.txt`, `data/card_overrides.toml`). Some tags *execute* effects:
   `engine/cast.rs` around lines 1062-1120, `engine/zones.rs` `do_etb` and `ETB_TRIGGER_TAGS`, and
   `engine/mana.rs`;
4. hand-written card code in the Registry (`sim-core/src/hooks.rs` `CardImpl`, filled by `impls/*::register`);
5. card-name special-cases in `engine/`, `ai/` and `cardcode.rs`;
6. load-time data fixes (`cards.rs` `data_fixes`);
7. the combo catalogue and T6 plans.

Known doubles:
- Ruby, Jet and Pearl Medallion and Starfield Mystic: a DSL `static:cost` plus a registry `cost` hook.
- Laelia, the Blade Reforged: a compiled draw plus a code trigger.
- Oath of Teferi.
- Hexing Squelcher: uncounterable twice.

JD's direction (2026-10-26): the long-term goal is to **remove hand-written rule code** wherever the ability language
does the card correctly. Hand-written *AI* code (`prio`, `options`, `hand_options`, `pval`, and so on) is
not rules; it stays for now and belongs to a later phase.

## What to build

`cardgen audit --paths [--deck KEY] [--md FILE]`: a report only. **No change to game behaviour.**

1. **In-memory build.** Build the CardDb and the Registry the way `audit::hooks_report` does
   (`cardgen/src/audit.rs`, around lines 477-480). Use the real registry, not grep, so const-named, looped and
   helper-registered cards (`common::walker`, `common::aura`) are all counted.
2. **`CardImpl::slots()`** in `hooks.rs`: the names of the filled slots. Split them into **rule slots** (events
   such as etb, dies, attack, cast, upkeep, end_step, landfall, draw, combat_damage, gain_life; plus resolve, cost,
   self_cost, walker, aura, land_*, uncounterable and other rule effects) and **AI slots** (prio, options,
   hand_options, pval and other choices or values). Document each slot's class in one place.
3. **Executing tags.** A table of the tags that execute an effect (a tag-to-executor list, e.g. `Tag::exec` in
   `tag.rs`, or a const list next to the code that runs them), separate from AI-only tags.
4. **Per card, its paths:** DSL abilities by type (`triggered:E`, `static:cost`, `static:toxic`, activated,
   spell, loyalty, uncounterable); executing tags; registry rule slots; registry AI slots.
5. **The double classes:**
   - **hard**: the same game action from two executors for one event (DSL `triggered:E` plus a registry slot for
     E; DSL `static:cost` plus registry cost/self_cost; DSL toxic plus a registry combat_damage that gives toxic;
     DSL loyalty plus walker; an executing enter tag plus registry etb; uncounterable twice);
   - **competing**: a DSL activated ability plus a registry `options` for the same ability;
   - **shadowed**: a DSL spell under a registry `resolve` (the DSL never runs);
   - **split**: two paths for different parts of the card. Treat this as a hard or competing double unless an
     override declares it. Add an optional override key `paths_ok = "why"` to `data/card_overrides.toml`'s
     schema: it is read and reported here, nothing else yet.
6. **The rule-code sort.** For each card with registry rule slots, put it in one group:
   - **dsl-covers**: the card compiles fully (Full-auto: every Oracle line compiled, nothing unparsed, no dropped
     keyword) and the DSL has an ability for each event the rule slots handle. The hand code is a deletion candidate,
     to verify by hand later.
   - **dsl-partial**: some Oracle text compiles but not all, or the DSL covers some events but not others.
   - **dsl-none**: nothing compiles, or the DSL is removed (`dsl = []`).
   
   Use the audit's existing compile information, the source and `unparsed` fields.
7. **Output:**
   - a summary: counts per double class and per rule-code group, overall and for `--deck`;
   - a table of every hard, competing and shadowed card with its paths;
   - the dsl-covers list.
   
   `--md FILE` writes the same as Markdown.
8. **Inventory, path 5:** a count of card-name string literals in `engine/`, `ai/` and `cardcode.rs` that are
   real card names. Inventory only.

## Checks

- Tests in cardgen:
  - the Medallions and Starfield Mystic show as hard (cost);
  - Laelia shows as hard (trigger);
  - a card with `paths_ok` shows as split, not hard;
  - an AI-only registry card (only `prio`) shows no double;
  - `slots()` covers every `CardImpl` field. Add a test that fails when a new field is added without a class,
    for example by destructuring the struct.
- Run the report on the committed data. Put the full Markdown in `audit/phase2/paths-2026-10-26.md`, and the
  summary numbers in your report.
- `cargo test --release --no-fail-fast` passes, and `cargo build --release --all-targets` has no warnings. No
  sim output may change. Prove it: the same seeds give byte-identical
  `commander-sim --realism --pool t3 --games 200 --jobs 16` output before and after your branch.

## Rules

- **Commands.** Run every build, test and sim from `rust/`, under
  `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 <command>`.
- **Thread limits.** Pass `-j 16` to cargo and `--jobs 16` to sims. Run one command at a time, in the background.
- **No Python**, anywhere in a command line.
- **No game behaviour change.** Don't edit card data except to add the `paths_ok` schema key, and add no values
  for it unless the data already proves the split. Don't fix any doubles in this PR.
- **Formatting.** Format only your files (`rustfmt --edition 2024 <file>`), never `cargo fmt`. Lines go up to 120
  characters.
- **Commits.** Commit often, with no Co-Authored-By line, and never push.
- **Scratch files.** Put them in `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase2/` only.
- **Report** in your final reply: a summary, the counts, the hard-double list, the size of the dsl-covers list, the
  files you changed, and anything that surprised you (cards the static map got wrong).
- **Waiting.** Don't end your turn while your own background command is still running: wait for it.
