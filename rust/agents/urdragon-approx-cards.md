# Task: bring Ur-Dragon's five approximate cards up to Full (JD's Ur-Dragon deck)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/urdragon-approx-cards`, branch
`urdragon-approx-cards`, based on origin/main (e6243581). `cd` into it for every command. Commit there; do not push,
do not open a PR.

## Which rules apply
Read the repo's `CLAUDE.md` and `AGENTS.md`, then `rust/agents/RULES.md` only for its conventions (impls layout,
hooks, registration, `rustfmt` on your files only, commit often, handoff file, short report). **Ignore the parts of
RULES.md that are out of date:** there is no Python reference and no `rust-port` branch, so don't merge `rust-port`
and don't run `difftest`. Never run Python, not even a version check: a hook blocks it. This task is already approved
by JD; work through it without asking.
- Memory cap on every cargo/binary command: `systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0 <cmd>`.
  Exit 137 means the cap was hit: treat it as a memory bug. The shell is zsh: write the cap out in full, don't store
  it in a variable.
- **Run no simulations** (no `commander-sim` runs, no `all_decks`). The coordinator runs the sims afterwards.
- No `Co-Authored-By:` line in commits.
- **Other agents work in sibling worktrees.** One models Garth One-Eye, in a new `impls/garth.rs`. The coordinator
  is changing Scion of the Ur-Dragon (`scion_options` in `impls/t45_urdragon.rs`, about line 818, and
  `impls/t6_scion.rs`) and the plan files. **Don't touch Scion, Garth or `data/plans/*`.**

## Context
JD's `urdragon` deck was rebuilt around Dragon entries, token copies and Treasure. It plays the generic policy plus
its plan file. Run `cardgen audit --mine` inside the worktree and read the `== urdragon:` block for the full current
notes on each card below (their code is in `impls/t45_urdragon.rs`, `impls/t45_miirym.rs` and
`impls/card_models.rs`, plus wherever the notes point). All five are `Approximate`:

| Card | Oracle text |
|---|---|
| Astral Dragon {6}{U}{U}, Creature — Dragon 4/4 | Flying. Project Image — When this creature enters, create two tokens that are copies of target noncreature permanent, except they're 3/3 Dragon creatures in addition to their other types, and they have flying. |
| Cursed Mirror {2}{R}, Artifact | {T}: Add {R}. As this artifact enters, you may have it become a copy of any creature on the battlefield until end of turn, except it has haste. |
| Nicol Bolas, the Ravager // Nicol Bolas, the Arisen | Ravager ({1}{U}{B}{R}, 4/4): Flying. When Nicol Bolas enters, each opponent discards a card. {4}{U}{B}{R}: Exile Nicol Bolas, then return him to the battlefield transformed under his owner's control. Activate only as a sorcery. Arisen (planeswalker, loyalty 7): +2: Draw two cards. −3: 10 damage to target creature or planeswalker. −4: Put target creature or planeswalker card from a graveyard onto the battlefield under your control. −12: Exile all but the bottom card of target player's library. |
| The Ur-Dragon {4}{W}{U}{B}{R}{G}, Legendary Creature — Dragon Avatar 10/10 | Eminence — As long as The Ur-Dragon is in the command zone or on the battlefield, other Dragon spells you cast cost {1} less to cast. Flying. Whenever one or more Dragons you control attack, draw that many cards, then you may put a permanent card from your hand onto the battlefield. |
| Wrenn and Six {R}{G}, Legendary Planeswalker (loyalty 3) | +1: Return up to one target land card from your graveyard to your hand. −1: 1 damage to any target. −7: You get an emblem with "Instant and sorcery cards in your graveyard have retrace." |

## Work
For each card:
1. **Find the gaps.** List exactly where the current model differs from the Oracle text: the audit note, the code,
   and what the AI does with it. Pay particular attention to these suspects, but verify them, don't assume:
   - **Astral Dragon:** are the tokens 3/3 flying Dragons, so that Dragon triggers like Dragon Tempest and Miirym see
     them?
   - **Cursed Mirror:** does it copy the creature's abilities and enter triggers, or only its stats? Is it a
     non-creature artifact again at end of turn?
   - **Bolas:** does the transform activation work, with all of the Arisen's loyalty abilities?
   - **The Ur-Dragon:** does the attack trigger draw one card per attacking Dragon, and then let you put a permanent
     card from hand onto the battlefield?
   - **Wrenn and Six:** the −7 emblem's retrace.
2. **Close the gaps that change play,** with the AI using the new behaviour sensibly. A gap may be left only if it
   almost never matters in this deck (e.g. Bolas's −12), and then the audit note says exactly what's missing and the
   status stays `Approximate`. Otherwise set status `Full`, with a one-line note, in `data/card_overrides.toml` (or
   wherever that card's status lives).
3. **Reuse shared machinery** (copy, token, planeswalker and transform paths) rather than copying it. Shared engine
   files: smallest additive change, listed in your report.
4. **Tests**, in a new `rust/crates/sim-core/tests/cards_urdragon_approx.rs`: at least one per closed gap, checking the
   Oracle behaviour and, where there's an AI choice, that the AI makes a reasonable one.
5. After any card-data change: rebuild cardgen, run `cardgen export`, and commit the generated JSON.

## Finish
- **Checks:** `cargo test --release` (from `rust/`, under the cap) passes, except the known
  `every_plan_file_fits_its_deck` failure for `urdragon` (fixed on another branch). `cargo build --release
  --all-targets` has no warnings.
- **Commits:** everything committed, with `rust/agents/handoff-urdragon-approx-cards.md` kept current while you work.
- **Full report** in `rust/agents/report-urdragon-approx-cards.md`: per card, the gaps found, what was fixed, what was
  left and why; every shared file touched.
- **Final message:** at most 15 lines (commit hash, per-card status, test result, anything the coordinator must act
  on). Then stop.
