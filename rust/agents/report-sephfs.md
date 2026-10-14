# Report: sephfs (Sephiroth, Fabled SOLDIER deck; three agents)

Branch `sephfs`. Deck: `decklists/JD/sephiroth-fs-wipe-steal.md`, key `sephfs`, card list `rust/agents/sephfs-cards.list` (68 names).

## Cards and how they are modeled
- Card code: `impls/sephfs_cards.rs` (Extinction Event, Necromantic Selection, Blood on the Snow, Living Death, Grave Betrayal,
  Dread Slaver, Kaya's Ghostform, Rabid Attack, Pestilence, Grave Researcher // Reanimate, and, from agent 3: Rewrite Regrets,
  Presumed Dead, Braids's Frightful Return). Sephiroth himself is in `impls/sheoldred.rs` (shared with the sheoldred deck).
- Agent 3 additions: **Rewrite Regrets** (best creature, else planeswalker, mana value <= 6 from our own graveyard, then Empower Jace 2
  through `alela::empower`; prio 64 only with a target worth >= 3). **Presumed Dead** (+2/+0 and `ret_dies_suspect`, run by
  `sheoldred::returns_when_dies`; cast before combat on our best non-token attacker worth >= 2.5 when opponents have creatures, and in
  response to destroy / damage / shrink / sacrifice removal on Sephiroth or a creature worth >= 2 via `presumed_protect` from
  `ai/decks.rs protect_main`). **Braids's Frightful Return** (read-ahead Saga with Lore data and etb + upkeep hooks: starts on II with a
  creature worth returning, on I with spare fodder and cards in an opponent's hand, else III; lore counters at upkeep, not after the
  draw step; no proliferate; the opponent sacrifices only a mana value 0-1 permanent, else loses 2 and we draw). **Call of the Ring**
  (prio 50; the upkeep tempt is the engine's `ring_tempt`). **Exsanguinate** (option in `sephfs::options` through
  `yshtola::x_drain_option` / `x_drain_go`: all spare mana, when `x_drain_option` scores a kill or X >= 4 with opposing creature power
  under a third of our life). **Zombify** (new in the deck, compiled Full-auto, own graveyard only; prio 64 with a target worth >= 3).
  **Animate Dead**: still the effect only (as the seph deck models it), now counted by the cast stat and card report.
- Caveats: "black Zombie in addition" clauses (Betrayal, Slaver, Necromantic Selection) not modeled; Blood on the Snow X = 0 (no snow
  mana); Pestilence applies activations in one action; Empower Jace's token only soaks attacks.

## Plan (`impls/sephfs.rs`, gated to key sephfs)
Cast order (Sephiroth first, outlets and payoffs with him, edicts only when they reach the fourth drain of the turn), wipe valuation
that spares an unflipped Sephiroth, steals from any graveyard, Rabid Attack ahead of a sweep, plus the agent-3 options above.
`--analyze` has a "Sephiroth" section (flips, first flip round, emblem drains, wipes, steals by source, sacrifices, Sephiroth draws).

## Deck change during agent 3 (user's request)
Reanimate and Phyrexian Arena out; Zombify and Callous Inspector in; Animate Dead is JD's own. Header: 30 buys, about $62.50.

## Shared-file changes
- `ai/brain.rs` (special_options arm "sephfs"), `ai/pool.rs` (gated reanimate block for sephfs; now also sets spells_cast / cast_names),
  `ai/decks.rs` (protect_main: sephfs arm calling `presumed_protect`), `impls/sheoldred.rs` (stat counters sf_*; `grant_eot` made pub),
  `impls/yshtola.rs` (`x_drain_go` made pub), `sim-cli/src/report.rs` ("Sephiroth" section), `practice/tests/server_fake.rs` (12 decks),
  `data/card_overrides.toml` (notes), `tests/fixtures/my_decks_parsed.json` (new list).
- Cards shared with other decks: Rewrite Regrets (an Avery Alela deck), Call of the Ring and Exsanguinate (sauron, sheoldred, yshtola),
  Callous Inspector (sheoldred). Registration of Rewrite Regrets / Presumed Dead / Braids is by card, so any deck casting them now gets the
  effect (before they had none). Same-seed games unchanged for sauron, sheoldred, yshtola, alela (t3).

## Checks
- `cargo test --release --no-fail-fast`: all pass (cards_sephfs: 31 tests); `cargo build --release --all-targets`: no warnings.
- Same-seed (seed 7, 48 games, lookahead, loose) vs base-sim (c4fc267): sauron t3, cloud t5, thrasios t3, garland t3, seph t3, plus alela,
  sheoldred, yshtola t3: identical.
- 1,000 games `--deck sephfs --pool t3 --profile loose --ai lookahead --analyze --seed 1013 --jobs 16`:
  | | before (agent 2, old list) | after (agent 3, new list) |
  |---|---|---|
  | win rate | 31.7% | 36.6% |
  | games with a flip | 52.6% | 56.3% |
  | flips per game | 0.639 | 0.686 |
  | emblem drains per game | 14.16 | 14.06 |
  | Sephiroth draws per game | 1.64 | 1.74 |
  | sacrifices per game | 5.18 | 5.58 |
  | wipes per game | 0.65 | 0.60 |
  | creatures stolen per game | 0.94 | 0.91 |
  No card is "never cast". Cast rates: Exsanguinate 11.4% of games, Presumed Dead 12.3%, Rewrite Regrets 17.6%. The win-rate gain mixes
  the new cards with the list change (Zombify, Callous Inspector for Reanimate, Phyrexian Arena); it is not split by cause.
