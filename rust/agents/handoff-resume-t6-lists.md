# Tier 6 lists handoff

Worktree `/tmp/commander-resume-t6`, branch `resume-t6-lists`. Full per-deck verification, source and model limitations: `audit/t6-lists/implementation.md`. The Kinnan/Basalt readiness gap has been escalated to the coordinator; no engine or AI changes were made. The Queen of Dale's Oracle text and model are card filtering and tokens, not mana production; inherited prose was corrected.

Current cardgen reports all 14 incoming cards Full; cache legality and identity, and origin/main cuts, were checked. Export regenerated, three Spellbook files forcibly refreshed. Coordinator runs Spellbook guard, full suite and the fixed-seed, loose lookahead queue `/tmp/resume-t6-queue.sh` with its immutable binary, and decides how to handle the existing Kinnan gap. No simulations or tests were run here. User-approved AGENTS.md and CLAUDE.md changes are included in the commit.
