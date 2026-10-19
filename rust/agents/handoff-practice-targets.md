# Handoff: practice-targets
## Design (done, compiles)
- `human/specs.rs`: per-card `Ask` list (`spec(name)`); `choose_pre` (targets/modes/gy cards/sac/discard, before paying),
  `pay_costs`, `choose_post` (X, Comet Storm kicks+targets, from the pool). Wired in `human/human.rs::cast_inner`
  and `legal::check_cast` (`specs::problem`). Fills Ctx (target/face/x/mode + new fields perms/faces/cards in state.rs).
- Person branches (`crate::human::is_human`) in: sephfs_cards (Extinction Event, Necromantic Selection, Blood on the
  Snow, Rewrite Regrets), kefka (Decorum, Stargaze), alela (Helping Hand, Daydream), t2 Victimize, titania Scout, threats
  Comet Storm, sweepers Inferno Titan (choices::divide_damage), smaug, rules2 Farseek, common Bojuka Bog, sheoldred
  Vault Plunderer; Dark Endurance via engine/cast.rs resolve_inner + tymna_cards::endurance_apply (no hook registered:
  a resolve hook would change pool AI card value).
- Already fine via spec alone: Lethal Throwdown, Presumed Dead, Grab the Prize, Splatter Technique, Cost/Painful/Peer.
- Baseline sim outputs: /tmp/pt-before-{sheoldred,cloud,kefka}.txt (binary /tmp/pt-sim-before); script /tmp/pt-sims.sh
  (BIN=... TAG=after). Use --jobs 3.
## Next
1. tests/play_targets.rs (Throwdown, Splatter draw four, Comet 2 targets, Peer on opponent, Titan 3 split)
2. rustfmt my files; full `cargo test --release -j 3`; after-sims + diff; report-practice-targets.md; final commit msg
   "Practice mode: the person picks targets and modes for resolve-coded spells (N cards)".
