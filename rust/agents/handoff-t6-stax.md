# Handoff t6-stax
Worktree: .claude/worktrees/t6-cedh/.claude/worktrees/t6-stax (nested), branch t6-stax. cardgen built in rust/target/release.
Cards: Blind Obedience; Chalice of the Void; Damping Sphere; Void Mirror; Pithing Needle; Disruptor Flute; Silence; Orim's Chant; Mandate of Peace; Angel's Grace
Oracle saved: scratchpad oracle.json.
Plan: module impls/t6_stax.rs (create with first code card), tests/cards_t6_stax.rs, engine-needs-stax.md.
Findings: Flute already works in the engine (values::stopped, mana::cost_of +3, flute_pick) -> data+test only.
Stand-ins: Player.hope_lock (q,t) = noncreature-only per-player cast lock (use for Silence/Chant/Mandate); g.fog for Mandate; counters-on-cast (Chalice/Void Mirror) have no hook.
Order: Flute, Silence, Orim's Chant, Mandate, Angel's Grace, Blind Obedience, Damping Sphere, Pithing Needle, Chalice, Void Mirror
Done: Disruptor Flute (Full), Silence, Orim's Chant, Mandate of Peace, Angel's Grace, Blind Obedience, Damping Sphere, Pithing Needle, Chalice of the Void (all Partial but Flute)
Next: Void Mirror, then full cargo test + build all-targets warnings + report-t6-stax.md
