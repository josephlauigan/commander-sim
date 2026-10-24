# Task: gandalf-ai — make the AI use Gandalf's instant-speed sorceries

Read `rust/agents/LASTDECK-RULES.md` first (you are the only agent running: `-j 16`, `--jobs 20`; the branch is
`tier0-precons`, not `elrond-cloud-decks`). Gandalf, Friend of the Shire: *Flash. You may cast sorcery spells as though
they had flash. Whenever the Ring tempts you, if you chose a creature other than Gandalf as your Ring-bearer, draw a
card.* The rules side works (`Tag::Gandalf`, `human/cards.rs`, `impls/mine.rs`); the AI barely uses it.

## What exists
`ai/brain.rs` `end_of_turn_window` (the end of the turn before yours) already offers sorceries when `has(g, p,
Tag::Gandalf)`, but skips `Tag::Rem`, `Tag::Wipe` and `Tag::Prot`, and the main phase casts most sorceries anyway, so
few are left for it.

## What to add (every deck, only while p controls Gandalf; small, gated changes in `ai/brain.rs`, `ai/decks.rs`,
`cardcode.rs` defend hooks, or `impls/elrond.rs` helpers)
1. **Hold flexible sorceries in the main phase:** card draw, tutors, token makers and targeted removal with no
   main-phase reason to cast now (nothing they enable this turn, no lethal) wait for the end-of-turn window, keeping
   mana open for instants and flash. Don't hold ramp, permanents' enablers, or anything needed this turn.
2. **Removal sorceries at instant speed:** in the end-of-turn window, allow `Tag::Rem` sorceries on the best opposing
   threat; in the defend step (an opponent attacks you), a removal sorcery on the biggest attacker when that saves a
   lot of damage or a key creature.
3. **Wipes after attackers:** in the defend step, a `Tag::Wipe` sorcery when it kills more opposing value (attackers
   included) than yours and the attack would cost you a lot.
Find the existing instant-speed paths first (`end_of_turn_window`, `cardcode::defend_hooks`, `hand_defend`, how
Icy Reception is cast in `impls/bulk_blue.rs`) and reuse them.

## Tests and measurement
A test for each of 1-3 (`tests/cards_elrond.rs` is fine). Then measure, look-ahead, loose, 1,000 games per tier,
Tiers 1-5, the same seeds, before and after your change:
- Sauron with `--swap "Night's Whisper=>Gandalf, Friend of the Shire"` (the delta before vs after tells what the AI
  change is worth for Gandalf);
- the elrond deck as is (`--deck elrond`), to check nothing got worse.
Put both tables in your report.

## Done when
The LASTDECK-RULES checks pass (no Python; memory cap; commit often; handoff; 15-line report, line 1 DONE or HANDOFF,
line 2 model ID). Full report in `rust/agents/report-gandalf-ai.md`.
