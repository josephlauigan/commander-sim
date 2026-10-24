# Task: kraum-godo — model Godo, Bandit Warlord for a Tymna/Kraum swap test

Read `rust/agents/RULES.md` first. Branch `kraum-godo`, worktree `.claude/worktrees/kraum-godo` (from origin/main).
Handoff file: `rust/agents/handoff-kraum-godo.md`. Report (≤15 lines) + details in `rust/agents/report-kraum-godo.md`.
CPU: `cargo -j 6` at most, `nice -n 19`, every test binary under
`systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0`. **Run no sims** (the main session runs the swap test).
No Python. No Co-Authored-By trailer on commits. Commit on `kraum-godo`; do not push.

## Card (Oracle)
**Godo, Bandit Warlord** {5}{R}, Legendary Creature — Human Barbarian, 3/3.
- When Godo enters, you may search your library for an Equipment card and put it onto the battlefield. If you do,
  attach it to Godo, then shuffle.
- Whenever Godo attacks for the first time each turn, untap it and all Samurai you control. After this phase, there
  is an additional combat phase.

Godo is NOT a Samurai himself (he's a Human Barbarian), but the untap includes him by name.

## What to build
- The card in the db (`data/card_overrides.toml` + whatever cardgen needs) and code in
  `rust/crates/sim-core/src/impls/tymna_cards.rs` next to the Gilgamesh/Equipment package (line ~891). Reuse what's
  there: `is_samurai` (Gilgamesh, Samurai's Katana), `equip_holder_value` / `equip_utility_value` to pick which
  Equipment to fetch, and the attach path Gilgamesh uses. The extra combat is `g.player_mut(p).extra_combats += 1`
  (see Combat Celebrant in `impls/t3.rs` / `impls/t1.rs`, and `engine/combat.rs:~1127`).
- "First time each turn": one extra combat per turn from Godo, even when he attacks again in that extra combat.
- ETB fetch AI: choose the Equipment that makes Godo best right now. In the Kraum list, Helm of Kaldra (haste, first
  strike, trample) lets him attack the turn he lands and so use the extra combat at once; Conqueror's Flail and
  Shield of Kaldra are the other strong picks. Score it with the existing helpers; don't hard-code names unless the
  helpers can't see haste's value on an entering creature (then add that to the scoring, generally).
- If the "Job select" Equipment (brings its own Hero token) is fetched, follow the card's rule: still attach to Godo.
- Make sure the second combat actually happens with attackers declared by the AI (untapped Godo and Samurai can
  attack again).
- Tests in `rust/crates/sim-core/tests/cards_tymna.rs`: ETB fetch + attach; no Equipment in library → nothing; first
  attack gives one extra combat and untaps Godo and a Samurai (Gilgamesh); attacking again in the extra combat gives
  no further combat; a game-level test that a hasty Godo with Helm gets two attacks on his turn.
- `cardgen audit` shows Godo **Full**. Full `cargo test --release` in `rust/` passes.
- Kraum list file: do NOT change `decklists/JD/tymna-kraum-partners.md`. The main session swaps
  **Serra Angel → Godo** with `--swap`; check that `commander-sim --deck kraum --swap "Serra Angel=Godo, Bandit Warlord"`
  (or whatever the CLI's swap syntax is — find it) would accept the card, by running only `--help` / a dry parse if one
  exists, not games.
