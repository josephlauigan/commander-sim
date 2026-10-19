# Task: new-cards-a — model Primeval Titan, Oppression, Andúril, Narsil Reforged

JD bought new cards and wants to know they aren't big downgrades in his decks. Your job is only to model the cards
(and give each deck's AI a sane way to play them); the main session runs the swap tests afterwards.

Worktree: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/new-cards-a` (branch `new-cards-a`, off
`new-cards-1018`). Work only there. Read `rust/agents/RULES.md`, but these rules override the old parts of it:
- **No Python, ever** (not even a version check); a hook refuses it. Ignore every Python / difftest / `rust-port`
  merge instruction in RULES.md. Don't merge any branch.
- Heavy commands: `systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 nice -n 19 <command>`, cargo with
  `-j 4`. The machine is shared.
- No sims beyond the preflight check below (no `--games` runs, no `all_decks`).
- Commit messages: no Co-Authored-By line.

A second agent (new-cards-b) is modeling other cards in parallel and also edits `data/carddb.txt`,
`data/card_overrides.toml` and `data/cards.json`: add your lines in one block, keep changes to shared files small.
The main session merges both branches and re-exports.

## Cards (Oracle text; model all of it, status Full unless you note a gap)
1. **Primeval Titan** {4}{G}{G}, Creature — Giant 6/6. *Trample. Whenever this creature enters or attacks, you may
   search your library for up to two land cards, put them onto the battlefield tapped, then shuffle.*
   Deck: **Titania** (key `titania`, tiers t1-t3). Titania, Protector of Argoth makes a 5/3 Elemental whenever a land
   goes to her owner's graveyard from the battlefield, and the deck has landfall payoffs. The land choice matters:
   in this deck prefer fetch lands / lands that sacrifice themselves (each one is a landfall now and an Elemental
   with Titania out), then utility lands, then basics. Reuse the engine's land search and fetch-land code
   (`engine/zones.rs` land_ramp, `engine/turn.rs` crack_fetch, `impls/titania.rs`). It enters tapped lands, both
   triggers (enter and attack) count as landfall.
2. **Oppression** {1}{B}{B}, Enchantment. *Whenever a player casts a spell, that player discards a card.*
   Deck: **Kefka** (key `kefka`, tier t3), a Grixis discard deck (see `audit/kefka/FINDINGS.md`, `impls/kefka.rs`,
   `ai/` for Kefka). It is symmetric: Kefka also casts spells. AI: cast it when the opponents hold more cards than
   Kefka does or when the deck profits from discards; don't cast it while holding a combo turn's worth of spells.
   Reuse existing discard machinery (`discard_cards`) and the deck's discard payoffs, if any trigger on opponents'
   discards.
3. **Andúril, Narsil Reforged** {2}, Legendary Artifact — Equipment. *Ascend (If you control ten or more permanents,
   you get the city's blessing for the rest of the game.) Whenever equipped creature attacks, put a +1/+1 counter
   on each creature you control. If you have the city's blessing, put two +1/+1 counters on each creature you
   control instead. Equip {3}.* Deck: **Cloud** (key `cloud`, mono-white Equipment voltron, tiers t1-t3; AI in
   `ai/` for Cloud, Equipment handling shared with other Equipment). The city's blessing already exists in the
   engine (grep `blessing` / `ascend` in `impls/t1.rs`, `t4.rs`, `alela.rs`): reuse it. Cloud's AI should value
   equipping Andúril on the creature that attacks (Cloud), like its other Equipment.

## For each card
- Line in `data/carddb.txt` (see `crates/cardgen/README.md`, "Add a card by hand"), an override with `status` and
  `note` in `data/card_overrides.toml`, then `cargo run --release -p cardgen -- export` (from `rust/`; it fetches
  the card from Scryfall if missing). Card code in the deck's module or `impls/common.rs`, registered like its
  neighbours. Pattern to copy for a recent small card: The Mind Stone / The Soul Stone in `impls/common.rs`
  (commit `2e93440`).
- A test per card in the deck's test file (`tests/cards_titania.rs`, `tests/cards_kefka.rs`, `tests/cards_cloud.rs`)
  that checks the Oracle behaviour (doc comment quoting the Oracle text, like `mind_stone_harness_and_end_step_blink`
  in `tests/cards_seph.rs`).
- The deck's AI must cast it: run the preflight from the repo root after building `sim-cli`:
  `rust/target/release/commander-sim --deck titania --pool all --swap "Ghalta the Unstoppable=>Primeval Titan" --preflight-only --jobs 4`
  `... --deck kefka --swap "Feed the Swarm=>Oppression" ...`, `... --deck cloud --swap "Hero's Blade=>Andúril, Narsil Reforged" ...`
  It must pass (the card is cast in a fair share of the games it's drawn). If it doesn't, fix the deck's priority.

## Checks before you finish
`cargo test --release` passes (whole workspace); `cargo build --release --all-targets` has no warnings; the three
preflights pass; `cardgen audit --deck titania|kefka|cloud` shows your cards with the status you set.
Handoff: `rust/agents/handoff-new-cards-a.md`; full report `rust/agents/report-new-cards-a.md`; final reply ≤15 lines.
