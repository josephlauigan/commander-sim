# Task: cloud-ai-fix — fix the Cloud AI's equipment play (from the cloud-ai-equip diagnosis)

Read `rust/agents/LASTDECK-RULES.md` first for the hard rules (no Python, memory cap, worktree, commit often,
handoff, short report). Changes for this task:
- Your branch `cloud-ai-fix` starts from `cloud-ai-equip`. Its commit 5533ea6 puts a **test-only** Cloud list (C3) in
  the deck doc: never edit that list; keep your code commits separate from it.
- **No batch sims** (JD paused them): no `--games`, `--analyze` or `all_decks` runs. Single-game traces
  (`--trace N`) and unit tests only. Build and test under `nice -n 19` with `-j 4`.
- Another agent (cloud-equip-cards) models Ardenn, Balan, Armored Skyhunter and Kor Duelist in `impls/bulk_white.rs`
  in parallel. You own `impls/cloud.rs`, `ai/search.rs` and the equip code in `dsl/`; don't edit `bulk_white.rs`
  beyond one-line hooks, and list every shared-file change in your report.

Read `rust/agents/report-cloud-ai-equip.md` (the diagnosis, with traces and file:function pointers) and its probe
tests `tests/cards_cloud_ai_diag.rs`.

## Fix, in this order
1. **Equip abilities any deck can use** for Champion's Helm (equip {1}) and Sword of Feast and Famine (equip {2}):
   today they equip only through Sauron's special options (`ai/decks.rs` sauron_options), Sheoldred's and Y'shtola's own
   code. Give them real equip options the generic equip path offers (card data `equip_cost` / `dsl::equip_options`),
   without changing how Sauron, Sheoldred and Y'shtola play (no double options). Check Magitek Scythe too: its data
   has an `equip_cost`, but the diagnosis says it never gets an equip option; find out why and fix it if so.
2. **`cloud_pick` (Cloud's tutor on entering)**: rank by what the board needs and what the mana allows this turn and
   next: the cheap swords (SoFI, SoFF, SoHH: cast 3, equip 2) and Champion's Helm (protection when Cloud is exposed)
   ahead of Kaldra pieces and Colossus Hammer unless the mana is there to cast and equip them soon. Don't fetch a
   piece that is already in hand or on the battlefield unattached.
3. **Search value of equips** (`cloud::search_bonus`, `search::choose_inner`): equipping Shield/Helm of Kaldra,
   Champion's Helm, Mask of Memory, the swords' protection and triggers should not score below "stop" when the mana
   is otherwise unused. Find why `stop` outscores `equip Shield of Kaldra` (27.8 vs 20.8 in the probe) and fix the
   cause, or add a bonus that fits the existing style (gated to key cloud).
4. **cloud_prio** for Solemn Simulacrum and Court of Ardenvale (ramp and card flow early, before the 4-mana equipment),
   and check the other C3 additions (Smothering Tithe, the rocks, Knight of the White Orchid, Loyal Warhound,
   Enlightened Tutor's target, Grand Abolisher, Skyclave Apparition) get sensible priorities.
5. **`cloud::attack_filter`**: "equipped" must mean equipment that adds power or evasion, not Explorer's Scope alone; no
   zero-damage attacks into blockers.
6. **Recast Cloud before combat** when he can attack or carry equipment this turn (haste from Helm of Kaldra or
   Swiftfoot Boots) and the mana allows.

Each fix gets a unit test (turn the `#[ignore]` demo in `cards_cloud_ai_diag.rs` into a passing test where it applies).

## Done when
The LASTDECK-RULES checks pass, except that the `all_decks` runs and the smoke run are replaced by: `cargo test
--release -j 4` passes (the practice `http_lan` save test fails on the base commit too: note it, don't fix it), and 10
single-game traces at t1 (`--trace 1` .. `--trace 10`) show no cast-but-never-equipped piece while mana sat unused
and no zero-damage attacks. Sauron, Sheoldred and Y'shtola must equip exactly as before: show it with their existing
tests plus one trace each. Report in at most 15 lines (line 1 DONE or HANDOFF, line 2 your model ID); full report in
`rust/agents/report-cloud-ai-fix.md`, committed.
