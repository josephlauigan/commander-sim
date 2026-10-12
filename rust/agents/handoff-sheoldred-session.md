# Handoff: the Sheoldred mono-black deck (a separate Claude session)

Written 2026-10-11 by the session working on the Kefka/Veyran trade. You are the main session for this task: you
run on Opus, use Sonnet subagents for card modeling, and check their work as `commander-sim/CLAUDE.md` says.

## Where to work: this worktree only

- **Worktree:** `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/sheoldred`
- **Branch:** `sheoldred-deck`, cut from `deck-swaps` (cc6b3e9).

Do all edits, builds and sims in this worktree. Use its own binaries,
`.claude/worktrees/sheoldred/rust/target/release/...`.

**Don't touch** the main checkout (`commander-sim/`, branch `deck-swaps`), and don't touch the other worktree,
`.claude/worktrees/kefka-veyran` (branch `kefka-veyran-trade`). Another session is working there in parallel.

Your agents' worktrees go in `.claude/worktrees/<name>`, branched from `sheoldred-deck`.

`deck-swaps` holds three commits that aren't on main yet:
- 8064b16, the Sheoldred → Bedevil swap test;
- 1c5d626, the Kefka protection hook;
- cc6b3e9, the Veyran list fix.

Both branches carry those commits. Whichever PR merges first brings them to main.

## Sharing the machine with the other session

- **CPU:** 24 threads. While both sessions are running sims, use `--jobs 11`, not 22. Expect runs to take about
  twice as long as the usual estimates: a 1,000-games-per-tier paired swap test takes about 30 minutes, not 15.
- **Memory:** 62 GB. Keep the usual cap on every sim, test or example:
  `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <command>`. Two capped runs fit.
- **Waiting on builds or runs:** no `pgrep` wait loops (see `rust/agents/OVERNIGHT-RULES.md`). Use background runs
  and notifications.

## Card ownership: no card may end up in two decks

The other session moves cards between Kefka and Veyran:
- **From Veyran to Kefka:** Deflecting Swat, Slip Out the Back, Underworld Breach, Niv-Mizzet, the Firemind,
  Jin-Gitaxias, Progress Tyrant, Steam Vents, Sulfur Falls, Spirebluff Canal and Stormcarved Coast.
- **Back to Veyran:** some of Kefka's cuts, plus basics.
- **Veyran's other refills:** only **blue or red bulk cards**.

To keep the two sessions apart:
- **This session owns the black and colorless cards** in `bulk-cards.txt` and `binder-cards.txt`, plus Bedevil
  (binder) for Sauron's refill.
- **Kefka's cuts aren't yours.** They may go to bulk once the other branch merges, but treat them as unavailable
  for now. Arcane Omens and Kefka's other black cards stay in Kefka.
- **Cards already in a deck stay there.** Lightning Greaves and Swiftfoot Boots are in Kefka; check every deck doc in
  `decklists/JD/` before you list a card. JD's physical lists for Sauron, Sephiroth and Veyran are the txt files in
  the main checkout's root (`commander-sim/sauron.txt` and the others); the repo docs match them.

## What's already known

**Pulling Sheoldred from Sauron costs him 2.1 points.** A paired swap test, Sheoldred → Bedevil, 1,000
look-ahead games per tier:

| Tier | With Sheoldred | With Bedevil | Change |
|---|---|---|---|
| T1 | 28.9% | 27.5% | −1.4 |
| T2 | 34.6% | 30.8% | −3.8 |
| T3 | 25.5% | 22.0% | −3.5 |
| T4 | 24.2% | 22.3% | −1.9 |
| T5 | 22.1% | 22.0% | −0.1 |
| **All** | **27.1%** | **24.9%** | **−2.1** |

The script, raw outputs and README are in `audit/sauron/2026-10-11-sheoldred-bedevil/`.

No card in Sauron is there mainly for Sheoldred, so pulling her leaves no dead cards behind. Jace's Archivist's wheel
is weaker without her, but it is built around Orcish Bowmasters.

For scale, Kefka leaving Sauron cost him 1.8 points.

**JD's status:** the deck was PINNED (paused) on 10/11 and is now being resumed in this session. JD hasn't yet
decided to pull Sheoldred from Sauron. The job is to build the deck, measure it, and show the trade: what the new deck
scores against what Sauron loses.

**The deck idea:** Sheoldred, the Apocalypse leads mono-black.
- **Her engine:** each opponent loses 2 life per card they draw, and you gain 2 per card you draw. Three opponents'
  draw steps drain 6 life a round.
- **The deck's job:** protect her, draw cards and remove threats.
- **Bulk has about 115 black cards** (about 65 creatures):
  - Removal is strong (about 27 cards: Terror, Last Gasp, Vanquish the Weak, Lethal Throwdown, Overkill,
    Sephiroth's Intervention, Soul Enervation, Desert's Due, Disembowel, Invoke Despair and more).
  - Card draw (about 16): Deadly Dispute, Cast Away Doubt, Phyrexian Rager, Seymour Flux, Mask of Memory, Diary of
    Dreams, Tome of Legends and more.
  - Champion's Helm and Darksteel Plate are in bulk.
  - **Gaps:** very few cards make opponents draw (about 3) and no wheels; thin lifegain payoffs; thin ramp.
- **Proposed buys, about $3.50.** JD hasn't approved these; ask before assuming them.

  | Buy | Price | Why |
  |---|---|---|
  | Underworld Dreams | $2.02 | Stacks with Sheoldred: 3 life per opponent draw |
  | Read the Bones | $0.30 | Draw 2 |
  | Syphon Mind | $0.30 | Draw about 3 |
  | Ambition's Cost | $0.34 | Draw 3 |
  | Arcane Signet | $0.50 | Ramp toward a turn-3 Sheoldred |

  Next steps up: Fate Unraveler ($3.41) and Howling Mine ($5.53).

  **Note on the singleton list:** Y'shtola already plays Read the Bones and Arcane Signet. Bought copies are new
  physical cards, so that's allowed, but say so in the doc's buy list.
- **For comparison:** Gix, built from the same black bulk, measured 9.4% (the Zada-candidate batch, dropped). His
  brief and report, `rust/agents/cand2-gix.md` and `report-cand2-gix.md`, show how that build was done.

## Suggested steps (confirm each long run with JD first)

1. **Draft the list.** Build 99 cards from black and colorless bulk and binder cards, plus any buys JD approves.
   Write `decklists/JD/sheoldred-mono-black.md` with a short plan and a pull list by card type (one color, so
   by-color isn't needed), and record the fixture with `cardgen deck sheoldred <list> --log "..."`.
   - Basics over tapped lands.
   - No shared cards.
   - Bracket 3: at most 3 Game Changers. Check Scryfall `is:gamechanger`; `data/scryfall_cache.json` has
     `game_changer`.
2. **Register the deck** (key `sheoldred`). Commit 3c1830e (Titania's registration) is the template; it touched
   these files:
   - `data/my_decks.toml` and `tests/fixtures/my_decks_parsed.json`;
   - sim-cli `decks.rs` (FILES, MINE, test count) and `args.rs` (DECK_KEYS);
   - `ai/brain.rs` (GENERIC_PLAYS and the special_options dispatch), `ai/decks.rs` (deck_prio and protect_response),
     `ai/mod.rs` (MAIN_STYLES);
   - `engine/cast.rs`, `impls/rules.rs` (CTHRESH), `settings.rs`, `impls/mod.rs` and a stub `impls/sheoldred.rs`;
   - practice `catalog.rs` and its tests, and `tests/game.rs` (the played-deck count).
3. **Model the gaps.** Run `cardgen audit --mine` and give every card that isn't Full to a Sonnet agent, with a
   brief in `rust/agents/sheoldred-cards.md` that follows `titania-cards.md` and `OVERNIGHT-RULES.md`.
   - **Check Sheoldred herself:** her trigger must fire for every draw, including opponents' draw steps and extra
     draws.
   - **Her AI plan:** cast her early, protect her (Helm, Plate), and lean on removal.
   - Verify the agent's work per CLAUDE.md: read the diff, run the full `cargo test --release`, check for
     warnings, run `all_decks adaptive 3` and `lookahead 1`, the audit, and spot checks against Oracle text.
4. **Measure.** Pool `all`, look-ahead AI, the same settings as Titania's 22.8% in `decklists/pool/pool-results.md`
   (section 0d has the format). Then compare: the new deck's number against Sauron's −2.1.
5. **Report to JD.** Then, if JD wants it, update `bulk-cards.txt` and `binder-cards.txt` (the Sheoldred deck's cards
   come out; if she leaves Sauron, Bedevil comes out of the binder and goes into Sauron's list and doc), plus
   `pool-results.md`. Then commit and open a PR.

## JD's standing rules (all sessions)

- **Never run Python.** A hook enforces this.
- **Confirm before long sim runs:** give the plan and a time estimate, then wait for "go".
- **Sonnet subagents, checked.** Subagents run on Sonnet; check their work before trusting it.
- **Memory cap** on every run.
- **Push-notify** each sim result and milestone.
- **Standalone commits:** run `git commit` as its own command.
- **Swap tests move functional blocks**, a card plus its support, not one-for-one swaps.
- Memory lives in `~/.claude/projects/-home-joseph-projects-mtg-sims/memory/`. Start the session from
  `/home/joseph/projects/mtg_sims` so that memory loads; `sheoldred-venser-plans.md` covers this deck.
