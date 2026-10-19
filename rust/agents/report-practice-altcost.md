# Report: practice-altcost

Generic: `human/alt.rs` offers the `hand_options` / `gy_options` / `land_options` hook options (label = the hook's) to
the person: cast from hand asks "cast normally / other ways"; graveyard cards with a hook are listed ("alt") and asked
the same way; `use land` lists them beside the land's own abilities. `can_pay`/`pay` already read the person's pool, so
the person taps mana first. Run = `ai::act::perform` (AI-chosen inside). Timing: sorcery speed unless instant/flash.
When a hook offers nothing (AI judgement gate) a small table (`forced_*` in alt.rs) offers the play's own `*_go`.

Own person paths (the person chooses): Everflowing Chalice (kicks, records Kicks, taps for 2), Ragavan dash, Phyrexian
Tower (creature), Ghost Quarter (land), Demonic Embrace from gy (discard + Aura target), Otherworld Atlas (both abilities),
face-down turn-up for Exalted Angel / Forum Familiar / Unyielding Gatekeeper / Exit Specialist.
Forced AI-go (AI-chosen inside): Susurian Voidborn and Perigee Beckoner warp, Exalted Angel / Familiar / Gatekeeper / Exit
face down, Scarlet Spider mayhem, Reassembling Skeleton, Command Beacon, Karn's Bastion, Geier Reach, Mikokoro, Abstergo.
Via hooks (AI-chosen inside): Woe Strider escape, Twinned Vision flashback (AI picks the discard), Profane Tutor suspend,
Reconstructed Thopter unearth, Muddle transmute, Jaxis blitz, Greenbelt Radical, Splatter Technique, Stargaze.
Probe test (`probe_jd_cards`, ignored) lists every JD card with a hook and how many ways it gets.

Not done: web-slinging / plot / foretell / landcycling (not hooks in the three kinds; unchecked), instant-speed use
of gy abilities on others' turns (Skeleton), Twinned Vision / Voidborn person-chosen discard. Hooks that are only "when to
flash" (Restoration Angel, Notion Thief ...) stay normal casts.

Engine/shared edits: `*_go` fns made `pub(crate)` in impls (legends, lands, sheoldred, t6_lands, kefka, t3, rules, rules2,
bulk_white); `human/mod.rs` (+alt); `human/human.rs` (gy + land menus); practice `view.rs` seam (gy list "alt",
`land_has_ability` was a stub returning false: now true for lands with abilities/hooks).
Checks: cargo test --release all pass, no build warnings; sims (sheoldred t3, cloud t1, kefka t3, 200 games) IDENTICAL.
