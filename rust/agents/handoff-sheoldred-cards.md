# Handoff: sheoldred-cards (agent 1 stopped at ~85 tool calls, 10/11)

## Key finding
The audit's "Full" verdicts for many deck cards were STALE: the candidate decks (glissa, gix, yawgmoth) were retired
and their card code deleted, but data/card_overrides.toml still says Full, so `cardgen audit --mine` hid cards that did
nothing in a game. Found by a probe (`registry.get(card).is_some()` + a cast test). FIXED for 18 cards in commit
ced9058: Cost of Brilliance, Vault Plunderer, Melancholic Poet, Foolish Fate, Canyon Crawler, Mabel, Seymour Flux
(old glissa.rs); Al Bhed Salvagers, Necrosquito, Callous Inspector, Apex Witchstalker, Gravblade Heavy, Ulna Alley
Shopkeep, Restless Bloodseeker, End of the Hunt (old gix.rs); Phyrexian Delver, Soul Enervation, Perigee Beckoner (old
yawgmoth.rs). All ported into impls/sheoldred.rs with 25 tests in tests/cards_sheoldred.rs, all passing, whole
`cargo test --release` passes. Old code: `git show 6fc094b^:rust/crates/sim-core/src/impls/glissa.rs` (also 9c289b5^ gix.rs,
fdfb1c6^ yawgmoth.rs). In zsh write `"${rev}:path"`, never `$rev:path` (`:r` modifier).

STILL TO CHECK (impl-less in the registry; I only judged by Oracle text, not by cast tests): Hullcarver (deathtouch
vanilla, probably fine), Multiply by Zero (tags rem,tgt; "base P/T 0/0"), Depressurize and Overkill (tags rem; old
gix.rs said `legal_targets` filters Depressurize by name and Overkill is `rem=shrink9999`: verify they work),
Lash of the Balrog (sacor4), Deadly Dispute (draw,fill), Dark Endurance (prot), Blood Artist (bartist tag),
Accursed Marauder/Burglar Rat/Invoke Despair/Screeching Soulbreaker/Champion's Helm (have impls). The probe test
(a throwaway tests/probe_tmp.rs listing names from /tmp/claude-1000/shcards.txt) printed impl=false abilities=0 for these;
write a cast test per card (one line of effect each) and fix any that do nothing. Also tell the lead: other decks
that were rebuilt from retired candidates may have the same stale-Full problem.

## Shared-file changes so far (list in the report)
- state.rs: `Player::bloods: u32` (Blood token count; 3 lines)
- engine/zones.rs die_rest: 3-line hook `impls::sheoldred::returns_when_dies` (Perigee Beckoner's grant)
- impls/rules.rs `replaced_tag_engine`: added "Seymour Flux" (the engine's generic `eng` upkeep draw doubled it)

## Sheoldred herself: verified OK with tests
Opponent draws (draw step via zones::draw(step=true), extra draws, wheel) drain 2 each; her controller gains 2 per own
draw. engine/zones.rs `draw` handles it; no fix needed. (`has(g,q,Tag::SheoA)` is one trigger per player, fine.)

## Part 2 done so far
- Cost of Brilliance and Vault Plunderer aim at the lowest-life opponent while Sheoldred is out (`drain_target`),
  tests included.

## Next, in order
1. The brief's 8 cards (none done). Design notes:
   - **Locust Spray**: resolve slot (-1/-1 eot via `shrink` helper in sheoldred.rs), `hand_options`: cast at an opposing
     creature with etgh<=1 (legal_targets(g,p,"destroy","c",false,Some(c)) then filter), else cycle {B} (pay, discard_cards,
     draw 1); prio hook = zero_prio. Pattern: cost_options/cost_plan in sheoldred.rs and bite_plan in bites.rs.
   - **Ribskiff**: Vehicle 4/4, crew 3. `etb` (m==src) draws 1; `crew` slot (Event::Crew, mark `at_once(c, Event::Crew, true)`
     as alela.rs does) copying alela.rs `skycoach_crew` / rules2.rs `crew` (Esika's Chariot): tap creatures that are sick/noatk
     or worth <Sheoldred (never crew with her), set DataKey::Anim + DataKey::Crewed(Val::Stamp) and `pow=4,tgh=4`, not when
     `x.sick`; `lands::open_attack`. Toxic 2: `combat_damage` slot like bites.rs `paladin_damage` (poison += 2 unless melira).
   - **Tome of Legends** (carddb tag `x`, source manual): page counters in DataKey::Counters (kefka.rs scamp pattern for the
     counter + tap ability via Action::Ability). Enters with 1 (etb m==src); other permanent entering that is your commander
     (`g.perm(m).is_cmd`) -> +1 (etb slot sees every entering perm); `attack` slot with commander in `atk` -> +1; ability
     {1},{T}, remove a counter: draw (options; utility higher with Sheoldred). Give it a `prio` (~38) so it is cast.
   - **Diary of Dreams**: override `dsl = []` in card_overrides.toml so the compiled {5},{T} draw goes away; `cast` slot (caster ==
     owner, instant/sorcery) +1 page counter; options: {5-counters},{T}: draw.
   - **Leech Collector // Bloodletting**: `gain_life` slot (LifeFn, fires on every gain; use once_per_turn key) sets DataKey::Prepared;
     `options` slot at sorcery timing (post.is_some()) pays {B} and calls `impls::mine::cast_copy_with(g,p,None,None,false,imp,&mut |g| ...)`
     (name None skips the counter window and the need for a "Bloodletting" card in the db; or add it to cards.rs engine_made) with
     each opponent losing 2; then Prepared=false. The card's name in the db is "Leech Collector // Bloodletting".
   - **Lethal Throwdown**: now tags rem=destroy tgt=cp + a dsl additional_cost note; `removal_options`/`cast_removal` (brain.rs)
     would cast it WITHOUT paying the sacrifice. Override tags="" and dsl=[] in card_overrides.toml, then write hand_options +
     resolve: plan = pick best target (legal_targets), pick fodder (prefer a modified creature: plus != 0, or attached equipment/aura;
     else the cheapest by `sac_worth`), `die(g, fodder, "sac")`, `do_cast_with(g,p,c,None,Ctx{target, x: modified as i32,..})`; resolve:
     apply_removal(g, Some(p), t, "destroy", Some(c)); if ctx.x == 1 draw 1. Check the test that the sacrifice is really paid.
   - **Moseo, Vein's New Dean**: end_step slot: if gained_life(p) (bites::gained_life; amount in `gained_turn.1`), return the best
     creature card with cmc <= life gained this turn from the graveyard (`rean_pick`-like helper `best_gy`/`reanimate` in
     sheoldred.rs, ignore the "pay life" part). Its Pest token (tags tok/tokp) already works.
   - **Restless Bloodseeker**: ported; override status entry is under the wrong key ("Restless Bloodseeker"), the card is
     "Restless Bloodseeker // Bloodsoaked Reveler".
   After each: override entry (status = "Full" + note) in data/card_overrides.toml for the card's exact name, `cardgen export`,
   test in tests/cards_sheoldred.rs.
2. Part 2 rest: Memory Jar (jodah.rs `jar`: when sheoldred_out(g,p) crack whenever active+post==Some(false)+total_mana>=3 or even no mana
   (it costs only {T}; check cost), else Jodah's rule; tiny edit in jodah.rs, list it), Toxic Deluge (find the X choice in
   engine/removal.rs ~line 375: with Sheoldred, X <= 4 so toughness 5 survives; verify X life paid), Bitter Triumph and Korlash tests
   with the sheoldred seat (Korlash = P/T equals Swamps; regenerate {1}{B}).
3. AI plan in `sheoldred_prio`/`sheoldred_special`/`sheoldred_protect` (stubs still generic): Sheoldred early, rocks first, equip Helm/Plate on her
   before other plays, Masterful Flourish / Battle-Rage Blessing / Dark Endurance as responses to destroy removal aimed at her
   (check `kind`), hold one protection instant while she is out, removal at the biggest threat. See ai/decks.rs ~1258 for the
   dispatch and other decks' `*_protect` (kefka.rs, zur.rs) for the pattern.
4. cardgen: the audit counts `Full` from overrides. Run `./target/release/cardgen audit --mine` -> `sheoldred: ... 0 not Full`; run
   `cardgen export` after card_overrides changes (rewrites data/cards.json + decks.json; commit them).
5. Final checks from OVERNIGHT-RULES (all tests, no warnings with --all-targets, all_decks adaptive 3 and lookahead 1, 48-game smoke
   `--deck sheoldred --pool t3 --profile loose --games 48 --ai lookahead --jobs 6 --brief`), report-sheoldred-cards.md, final 15-line report.
