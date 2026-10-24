# cloud-ds report
Engine: double strike was one damage event (2x power): Sword triggers fired 2x (Cloud doubling), not 4x. Fixed in engine/combat.rs resolve_combat_inner: unblocked double strikers now deal two damage steps (post-damage block looped twice), so triggers, commander damage, lifelink happen per step. Blocked/trample/planeswalker paths unchanged.
Shared edits: engine/combat.rs (loop, rustfmt reindent), engine/values.rs (prot_colors: eot_kw pro_colors).
Package audit/cloud/packages/ds-doublestrike.swaps: Secure the Scene->Fireshrieker, Veteran's Powerblade->Duelist's Heritage, Nim Deathmantle->Akroma's Will. None owned; buy.
Same-seed (300 g, adaptive, seed 7, 9 JD decks): kefka, seph, sheoldred, yshtola identical; cloud, sauron, sephfs, titania, veyran moved by <=0.7 pts (double strikers in opposing pools: real rules fix).
Smoke 300 g T1 lookahead loose: 27.3 -> 29.0 (noise +-6.7), no claim.
