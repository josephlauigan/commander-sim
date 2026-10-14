# Handoff: sephfs (agent 2 stopped after ~65 tool calls)

Branch `sephfs`, worktree .claude/worktrees/sephfs. Baseline binary for same-seed checks: /tmp/claude-1000/-home-joseph-projects-mtg-sims/
291e1f60-a736-4dbc-bd9b-964e999afb88/scratchpad/base-sim (else rebuild from c4fc267).

## Done (agent 2)
- tests/cards_sephfs.rs: 21 tests, all pass (the 10 new cards, Sephiroth flip/emblem/draw, Blood Artist/Zulaport/Butcher, edicts,
  Massacre Girl, Animate Dead/Reanimate from an opponent's yard, Altar, Tower, Yahenni, Rabid Attack option).
- Rabid Attack cast path: `sephfs::options` (brain::special_options arm "sephfs"); sephfs_prio returns 0 for it.
- Animate Dead was cast as a real Aura by pool.rs `reanimate` (never reanimated): for key sephfs it is now the effect only
  (pool.rs, gated like the seph deck's model). Shared-file changes: ai/brain.rs (arm), ai/pool.rs (gated block),
  impls/sheoldred.rs (stat counters only: sf_seph_draws, sf_flips, sf_first_flips, sf_first_flip_round, sf_emblem_drains),
  sim-cli report.rs ("Sephiroth" section, ty collects sf_*), practice/tests/server_fake.rs (11 -> 12 decks).
- Full `cargo test --release` passes (no failures, http_play ok); build --all-targets no warnings; all_decks adaptive 3 and
  lookahead 1 run clean.
- Same-seed (seed 7, 48 games, lookahead, loose) vs base-sim: sauron t3, cloud t5, thrasios t3, garland t3, seph t3 all IDENTICAL.
- 1,000 games (--deck sephfs --pool t3 --profile loose --ai lookahead --analyze --seed 1013 --jobs 16; output in scratchpad
  sephfs-1000.txt): win 31.7%; flips 0.639/game (52.6% of games flip), first flip round 8.0 on average; emblem drains 14.16/game;
  wipes 0.65/game, Pestilence activations 1.72; steals 0.94/game (reanimation spells 0.56, Grave Betrayal 0.74, Slaver 0.02,
  prepared Reanimate 0.05), Ghostform returns 0.17, Rabid draws 0.06; sacrifices 5.18/game; Seph draws 1.64/game; Sephiroth removed 0.75/game.

## Left (blocking: "never cast by the sim" in the 1,000-game run)
Never cast: Animate Dead (the effect-only path in pool.rs `reanimate` does not set `cast_names` / stat spells_cast; copy what
seph.rs ~l.448-455 does), Exsanguinate, Presumed Dead, Rewrite Regrets, Braids's Frightful Return, Call of the Ring.
FOUND: the card_overrides.toml "Full" notes for Rewrite Regrets, Presumed Dead, Braids's Frightful Return, Exsanguinate and Call of the
Ring come from the Yawgmoth candidate; the CODE for most is missing in this lineage (grep shows only the die_rest
`returns_when_dies` mechanism for Presumed Dead's `ret_dies_suspect` grant in sheoldred.rs, `grant_eot`, `yshtola::x_drain_option` /
`x_drain_go` for Exsanguinate (call them from sephfs::options as yshtola's x_option does), and `cardcode::ring_tempt` in turn.rs for
Call of the Ring). To do, in sephfs_cards.rs / sephfs.rs:
 1. Exsanguinate: option via yshtola::x_drain_option (+ Action::Plan to x_drain_go if public) when lethal or late.
 2. Call of the Ring: sephfs_prio ~ 50 (an enchantment: just needs a prio so it is cast; check ring_tempt fires on upkeep for it).
 3. Rewrite Regrets: resolve hook: best creature (cmc<=6) from own graveyard to the battlefield (sheoldred.rs rean_pick / best_gy
    are private: write own using steal_value over own gy); skip Empower Jace or model the Jace token (note it); prio when a good target.
 4. Presumed Dead (instant): option before combat/in response to a wipe: +2/+0 and grant_eot "ret_dies_suspect" on a creature worth >=2
    (grant_eot is private in sheoldred.rs; make it pub or copy). Cast in response to removal if easy.
 5. Braids's Frightful Return (read-ahead saga): chapters I sac+discard, II gy creature to hand, III opponent sacrifices or loses 2 + draw.
    Check whether any Saga machinery exists (`grep -rn lore src`); otherwise model as a one-shot: choose chapter II or III effect
    on entering (document it) so it is cast and has value; update the override note honestly.
 6. Update the override notes of the five cards to say where the code lives. Add tests for each. Rerun the 1,000-game run; no card may be
    "never cast"; Sephiroth flips in 52.6% of games already (no AI fix needed; first flip round 8 is a table round count).
Then: re-run full cargo test --release, same-seed check again (any shared change), write rust/agents/report-sephfs.md (sections:
cards and how modeled, plan, shared-file changes above, numbers above, caveats: Animate Dead effect-only, "black Zombie" clauses
not modeled, Blood on the Snow X = 0, Pestilence applies activations in one action), commit, final 15-line report.

## Gotchas
- Never run Python (the hook blocks even a stray line in a multi-command call).
- Tests: perm() runs ETBs (Plaguecrafter/Fleshbag sacrifice themselves, Sephiroth sacrifices an existing token): order setup accordingly.
- cargo test stops at the first failing target: use --no-fail-fast.

## Agent 3: DONE. All items finished; see report-sephfs.md. Nothing left.
