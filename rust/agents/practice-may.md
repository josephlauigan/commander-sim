# Task: practice-may, "you may" triggers ask the person

Read `rust/agents/practice-fixes-common.md` first. Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/practice-may`, branch `practice-may`.

## The problem
`engine::stack::trigger_window` (and `ability_window`) never ask the person whether to use an optional trigger: they
only give priority. So for a person, every "you may" trigger is used (or skipped) by the card code's AI judgment.
The audit (REPORT.md cause 1, and the "chosen + skipped" / "forced may" rows in the findings) lists about 30 cards.

## Do
1. Add one helper for an optional trigger, for example `engine::stack::may_window(g, p, src, name, imp, prompt)`
   (or a `may: Option<&str>` argument on a new function; don't change `trigger_window`'s signature): for a person it
   asks yes/no first (`choices::yes_no`, prompt like "Seymour Flux: pay 1 life and draw a card?") and returns false
   on no; for the AI it behaves exactly like `trigger_window` today.
2. Use it in the "you may" triggers of **cards in JD's decks** (`jd-cards.tsv`). For each card: read its Oracle text
   (`data/scryfall_cache.json`, lowercase name keys, field `oracle_text`; or `curl -s -m 30
   "https://api.scryfall.com/cards/named?exact=<name>"`) and convert only where Oracle says "may". Where the code also
   skips the trigger for the person by an AI value test (`if pval(...) < 2 { return }`, "only if worth it"), give the
   person the choice instead of the AI's test. Where the trigger also has a choice inside (which creature, which card),
   leave that inner choice alone unless it's a one-liner with an existing helper: the per-card choices are a later task.
   Start with the audit's list: Seymour Flux, Karmic Justice, Plan for All Outcomes, Rooftop Assassin, Soul
   Enervation, Phyrexian Delver, Moseo, Airlift Chaplain, Jackdaw Savior, The Wasp, Flickering Hound, Conjurer's
   Closet, Teleportation Circle, Stadium Tidalmage, Jocasta, Helm of the Host, Gix, Silverblade Paladin, Cursed
   Mirror, Ragavan, Boughside Wanderers, Mana Vault (its untap), Curiosity, Gonti, Thief of Sanity, Ral ch. II.
   Then grep `trigger_window(` across `impls/` for the remaining JD cards whose Oracle says "may".
3. Tests: a handful of representative cards (yes applies it, no skips it, and the AI path unchanged), not one per
   card. Then list every converted card in your report with its Oracle "may" phrase.

## Files owned
`engine/stack.rs` (the new helper only), the `impls/*.rs` trigger functions you convert, a new
`tests/play_may_triggers.rs`, `rust/agents/{handoff,report}-practice-may.md`.

Commit message for the last commit: "Practice mode: 'you may' triggers ask the person (may_window), N cards".
