# Phase 4, PR 1: deck plans as data (design)

`documents/realism.md`, Phase 4 ("Generic AI"); gate: each deck level with its hand code in a paired test. JD's plan
document, AI point 1: deck intent as data, not code; a new deck needs a plan file and no Rust.

## Where the plans live and how they load

- One TOML file per deck: `data/plans/<deck-key>.toml`. The file name must be the deck key (`deck = ...` repeats it;
  the loader refuses a mismatch).
- `rust/crates/sim-core/build.rs` embeds every `data/plans/*.toml` (`include_str!`, regenerated when the directory
  changes), so sims and tests need no data path, and a new file is picked up by the next build with no Rust change.
- `ai/plan_data.rs` parses them once (`plan_data::all()`, serde + `toml`, `deny_unknown_fields`): unknown keys,
  unknown conditions, a combo name that isn't in the catalogue, a prio rule that selects nothing, or a bad knob value
  refuse the file with its name and the reason (a panic at first use; `tests/plans.rs` exercises each refusal).
- `tests/plans.rs::every_plan_file_fits_its_deck`: every card a plan names (key cards, line pieces, engines, prio
  rule cards, `out:` conditions, holds) must be in its deck's list in `data/decks.json` (commander included), or for
  a `retired = true` deck in the import list of `decklists/pool/retired/<deck>.md`; every group of every line (a
  catalogue combo's too) must have a card in the deck; a line citing a Spellbook variant (`spellbook = "id"`) must
  list every card of that variant in `data/spellbook/<deck>.json`. A list change that strands a plan line fails
  `cargo test`.

## Schema

```toml
deck = "chulane-bant-value-combo"   # the deck key (= the file name)
retired = false                      # a retired pool deck (test seat): checked against its retired list
default = "hand"                     # which arm plays the deck when AI_PLAN_DATA is unset: "hand" | "plan"

[style]                              # the heuristic AI's style (moved verbatim; JD refits after Phase 4)
temp = 1.0
aggression = 0.5
caution = 0.7

[commander]
prio = 88                            # cast priority (default 75; 85 for a commander with card code)
turn = 1                             # earliest turn (default 2)

[knobs]
aura_host = "commander"              # where Auras go
aura_prio = 70                       # an Aura's cast priority (default 55)
top_pref = "mv"                      # stack the library top by mana value once the commander can connect
hooked_prio = 55                     # a card-code permanent's default priority

[key_cards]                          # name -> cast priority, over every other rule; also "worth protecting"
"Craterhoof Behemoth" = 80

[[line]]                             # win lines, best first (the order breaks ties)
name = "Aluren + Chulane + a bouncing creature"
combo = "Aluren + Chulane + a bouncing creature"   # a catalogue combo (impls::combos::all() name): its groups

[[line]]
name = "hoof"
kind = "combat"                      # combo | combat | drain | commander-damage | lock | value (documentation)
pieces = [["Craterhoof Behemoth"]]   # groups: one card of each group is needed
when = ["creatures>=6"]              # live only when this holds; not live, the line counts one step further
ordered = false                      # true: the groups are steps; tutors want the first missing one only
spellbook = "1234-5678"              # optional: the Spellbook variant these pieces are (the test checks the cards)

engines = ["Rhystic Study"]          # card engines tutors want once no line wants a card

[[prio]]                             # cast-priority rules: the first rule selecting the card decides
cards = ["Craterhoof Behemoth"]      # selectors (all given must match): cards, commander, creature, flying,
                                     # subtypes, min_mv, max_mv, next_of (the next step of an ordered line)
when = ["creatures>=6"]
value = 80
else = 20                            # when `when` fails (absent: the next rule)
plus_mv = 8                          # value + min(mana value, plus_mv)

[[hold]]                             # a protection card and what it protects
card = "Teferi's Protection"
protects = ["commander", "Craterhoof Behemoth"]
mana = true                          # keep its mana up (not read yet)

[mulligan]
lands = [2, 5]                       # keep with this many lands (the generic rule)
pieces = 1                           # and at least this many line pieces or engines
fixing = true                        # a source of each colour (not read yet)
```

Conditions (`when`): a list of alternatives (any), each `a & b & ...` (all). Atoms: `commander-out`,
`commander-zone`, `ninjutsu-window` (a combat with a ninjutsu window ahead), `drain-payoff` (a death-drain payoff
out), `out:<card>` (that permanent on the battlefield), `not <atom>`, and comparisons of `creatures`, `lands`, `tax`,
`turn`, `opp-power`, `life` with `>=`, `<=`, `>`, `<`, `=`.

## How the generic policy reads it

`plans::config(key)` is the one entry point the AI already used (`pool::generic_prio`, `pool::worth_protecting`,
`ai::base_style`, `common` aura host, `topdeck::mv_mode`, the tutors through `plans::wish_raw`). For a deck that plays
from its plan, it returns the `DeckConfig` the file stands for:

- the static knobs and key cards straight from the file;
- `wish`: the **derived tutor priority** when the file has lines or engines (`plan_data::derived_wish`):
  1. the mana a line complete but for its mana needs (`combos::mana_for_lines`, as before);
  2. line by line, from the closest: distance = the line's missing groups (a group is held on the battlefield, in
     hand, as a land in play, as a commander in the command zone, or on the library top where the player put it),
     plus one when its `when` fails; ties in file order. Each line gives the library cards of its missing groups (an
     ordered line: of its first missing group only). A line with a group the deck no longer plays is skipped;
  3. then the engines not in hand or play.
  The whole ranked list goes out (a tutor restricted to creatures, say, still finds the next line's piece), so the
  wish list can't go stale when the list changes: it is computed from the current pieces;
- `prio_fn`: the file's `[[prio]]` rules (`plan_data::rule_prio`) when it has any;
- where the file has no lines (no prio rules), the hand wish function (priority function) still runs (Stage A).

Holds: `pool::worth_protecting` also protects what a plan's holds name (the commander already was). Mulligan: with a
`[mulligan]` section, `turn::keeps_hand` uses it after the Cloud and Tier 6 rules. Both are inert for the decks
converted so far (none needed them to match its hand code).

## The A/B switch

Env `AI_PLAN_DATA`: `1` every deck with a plan file plays from it; `0` every deck that still has hand code plays from
that; a comma-separated list of deck keys plays only those from their plans (the paired test of one deck with the
rest of its tier unchanged); unset, each file's `default`. A deck whose hand code is deleted always plays from its
plan.

## Stages

- A (results identical): a file for every `DeckConfig` deck with its knobs; `AI_PLAN_DATA=1` reproduces the hand
  choices byte for byte (`--realism --pool tN --games 300`).
- B: lines, engines and prio rules replace the hand `wish` and `prio_fn`, deck by deck, each with a paired test
  (`--realism --pool <tier> --games 2000`, `AI_PLAN_DATA=0` against `AI_PLAN_DATA=<deck>`, same seeds). Results
  and what was deleted: `audit/phase4/plans-handoff.md`.
