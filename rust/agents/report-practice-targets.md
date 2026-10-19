# Report: practice-targets
New `human/specs.rs` (per-card cast specs) wired into `human::cast_inner` and `legal::check_cast`; Ctx gained `perms`, `faces`, `cards`.
Specs (cast-time choices): Lethal Throwdown, Presumed Dead, Dark Endurance, Stargaze (X), Grab the Prize (discard), Splatter Technique,
Blood on the Snow (mode), Comet Storm (kicks, X, one target each), Cost of Brilliance / Painful Lesson / Peer into the Abyss /
Decorum Dissertation (player), Rewrite Regrets, Helping Hand, Victimize (graveyard cards + sacrifice), Daydream, Scout the City.
Person branches at resolution: Extinction Event, Necromantic Selection, Blood on the Snow return, Stargaze, Inferno Titan
(choices::divide_damage, one question per point), Smaug (deal_damage), Farseek, Bojuka Bog, Vault Plunderer.
Shared-file changes: state.rs (3 Ctx fields), choices.rs (apply_damage split out of deal_damage, divide_damage), engine/cast.rs
(Dark Endurance block in resolve_inner, only with a person-chosen ctx.target), sheoldred::modified now pub, tymna_cards::endurance_apply.
Not done: Glorious Decay (cut). Dark Endurance has no resolve hook on purpose (it would raise the AI's card value in pool.rs).
Before/after sims (sheoldred t3, cloud t1, kefka t3; 200 games, loose, adaptive, --jobs 3): outputs identical.
