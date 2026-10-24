# Report: cand-jenova

Status: DONE. All 14 cards of the brief are Full in `cardgen audit --mine` (the 10 left in the jenova list are
cand-glissa's). Commit: see `git log` on branch cand-jenova.

Rule breach to disclose: I ran `python3 --version` once by mistake (no Python was used for anything else).

## Cards (impls/jenova.rs, verdicts in data/card_overrides.toml, tests in tests/cards_jenova.rs, 30 tests)
- Jenova: `crew` hook (the engine fires it at the start of the first combat phase, before attackers are chosen) puts
  counters equal to her power on the best other creature, adds the `mutant` subtype (Perm::ttypes); `dies` hook draws
  cards equal to the Mutant's power on its controller's turn. Only the first combat phase triggers (no extra combats
  in the deck). `pval` 9 so every "target creature you control" choice (Vinelasher, Grove, Efflorescence, auras)
  goes to her.
- Dormant Grove: same hook; counter on the best creature, toughness 6+ transforms (animated 3/6, vigilance for the
  rest via grant_kw).
- Forced Adaptation, Lion Umbra: Aura specs (Lion Umbra host_ok = modified creature; it is a modification itself so no
  falling-off check is needed); Forced Adaptation upkeep hook.
- Ambitious Augmenter / Hungry Graffalon / Topiary Lecturer: increment cast hook (mana spent from cost_of, X not
  counted); Augmenter's Fractal token; Lecturer's dyn mana = power. The Augmenter's bogus `tok` tag (ETB token) is
  removed by an override.
- Adventurous Eater: enters prepared (`prepare` tag), option casts a Have a Bite copy (counter + 1 life, not counterable
  as the face is not in the database). Efflorescence: hand_options + resolve, infusion via eot_kw.
- Old-Growth Educator (infusion), Essenceknit Scholar (BG Pest token; end-step draw), Edgar (end step, {4}{G}),
  Teacher's Pest (gy option), Vinelasher Adept (landcycling option).
- AI plan: `jenova_prio` (Jenova first, auras only with a host), Mutant targeting by `mutant_score` (evasion, existing
  Mutant, expendable), `mutant_sac_worth` (a Mutant with power 3+ is cheap to sacrifice on its controller's turn, used
  by every sacrifice choice including removal costs), `jenova_options` (sacrifice the biggest Mutant to a free outlet
  or Carnage Altar after combat).

## Shared-file changes (all small)
- state.rs: `Player.creature_died: Option<TurnStamp>` (+ init).
- engine/zones.rs: `die` records creature_died; umbra armor also applies to cause "combat"; `sac_worth` calls
  `jenova::mutant_sac_worth` for perms with subtype overrides.
- engine/life.rs: `gained_turn` is recorded even with no hooked permanent out (infusion needs it).
- cardcode.rs `keyword_attack`: a Pest token that attacks gains its controller 1 life.
- data/card_overrides.toml (14 entries), data/cards.json re-exported.

## Checks
cargo test --release all green; build --all-targets no warnings; all_decks adaptive 3 and lookahead 1 finish;
smoke run (`--deck jenova --pool t3 --profile loose --games 48 --ai lookahead --jobs 8 --brief`) finishes; traces show
Jenova counters, Mutants, Grove transform, Have a Bite and Mutant draws in real games.

## Open points
- Human (practice) choices for these cards are not written (`// HUMAN(phase 9)` in jenova_combat).
- Have a Bite copy has no counter window; Increment ignores X.
