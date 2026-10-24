# Report: cloud-audit-1015 (diagnosis only; model claude-sonnet-5-5)

Evidence: the six 4,000-game analyze files (`audit/pool/2026-10-14-full/cloud-t*.txt`) and 60 traced games (20 each at T1, T2,
T0 via `--any-tier`; seeds 1-20, loose, lookahead). Trace summaries were made with grep/awk over the logs. No code or
decklist changed; no batch sims. Cloud wins in the traces: T1 5/20, T2 1/20, T0 1/20 (matches 13.5 / 12.3 / 7.3).
Limit: the trace has no opening hands and no block lines, so keeps and blocks were NOT audited from traces (blocking code
read instead, see 3).

## 1. Where games are lost
- Plan comes online (equipped Cloud attacking): by T4 27% (T1) / 23% (T2) / 37% (T0); by T6 56 / 55 / 66%; never in ~25-30%
  of games. T3 0-6%. Cloud resolves T2 in 99%+, so the tempo loss is after Cloud: equip + protect + carry.
- Losses are combat races, nearly all: T1 Light-Paws cmdr dmg 17.9%, Isshin 15.4, Tatyova 11.3, Lathril 9.8+6.3 drain, Teysa
  7.5 drain + 5.8; T2 Brago 19.8, Windgrace 16.6, Kaalia 15.9, Sythis 11.1, Meren 8.1; T0 Giada 25.1, Captain America 19.0,
  Anikthea 17.1, Zurgo 14.0+5.8, Stella 7+2+1. Cloud dies at median round ~10 (T1 9.3, T2 11.4, T0 9.2). Traces: Cloud
  eliminated on R5-R15 (T1 median 10, T2 12, T0 9); first 4+ damage hit median R5 (T1), R5 (T2), R5 (T0), none at all in 3/20, 1/20, 6/20 games.
- Cloud (the commander) is removed 0.57/game (T1), 0.68 (T0), 1.35 (T2). In 60 traces 66 removals: exile 28 (Path 6, StP 5,
  Journey 2, O-Ring/Banishing/Cast Out), destroy 11, mutate 3, bounce 2, and 22 by board wipes (21 of 60 games had Cloud's
  side wiped, 77 creatures lost; evac, Deluge-types, Deed, dmg13, minus). Equipment: SoFI is removed 0.31-0.55/game, SoHH
  0.15-0.36 (artifact removal and wipes), so the single carrier plan keeps being reset.
- Mana is NOT the problem: lands at start of R4 3.15 / 3.20 / 2.95, R6 4.3 / 5.2 / 4.7 (on curve); rare screws (t1-6 stuck on 2
  lands all game, died R9). Hand clog IS: Cloud holds 5.5-6.0 cards at R4 and 4.4-4.7 at R8 in all tiers.
- Never cast when drawn (analyze): Unfinished Business 0% (untagged: sim never casts it), Rootborn 3-5%, Formation 2-7%, Holy
  10-18% (all reactive), Beatrix 36-46%, Bronze Guardian 35-44%, Secure the Scene 35-48%, Shield of Kaldra 41-44%, Stormfront
  Riders 42-50%, Austere Command 46% , Exalted Angel 47%, Fate of the Sun-Cryst 43% (T0). Austere cast only 7 times in 60 games.
- Early damage: Cloud's life ~35-37 at R5 start in all tiers, then collapses (t1-3: 23 -> 8 over R5-R6; t1-6: 35 -> 10 in a
  single turn R6-R7). The deck has almost no lifegain, no early blockers, no fog.

## 2. Rules / AI faults found (nothing fixed)
F1. **Look-ahead picks "no attack" with an equipped Cloud.** `ai/search.rs:379-409` (`choose_attack_inner`, candidates
  filtered/all/none per defender, scored only by `evaluate()` after one rollout turn; Cloud has no attack bonus, unlike
  `cloud::search_bonus` cloud.rs:1507 which only covers main-phase options, search.rs:298-310). In 148 Cloud turns with an
  equip, 19 (13%) ended `attack plan: none`; t1-4 R4 and R5 Cloud wore SoFI (+Boots) and sat home both turns (SoFI was then
  destroyed R6 having never connected). Total `none` plans: 34/204 Cloud turns (T1), 23/237 (T2), 26/177 (T0). Expected
  impact: a Sword connect is 4 damage + 2 cards; a missed attack on the one turn the Sword lives is a lost trigger pair.
F2. **Stormfront Riders bounces Cloud himself, with his Equipment, 8 of 14 casts** (t1-4 R6 with Boots on him; t2-9 R8).
  The Equipment falls off and needs re-equip, the tutor re-fetches (value) but costs the turn; Riders is cast 42-50% when
  drawn. Code: Riders' ETB target choice (impls/common/white; grep `Stormfront Riders`); needs a "do not bounce the
  equipped commander unless removal is on the stack / tax>0" rule. Uncertain impact (tutor is real value); test, don't assume.
F3. `keep_cloud_home` (cloud.rs:1482) keeps Cloud home whenever he has no `strong_gear` (cloud.rs:1553) and any untapped
  creature could block him: this is why 28 `keeps N creature(s) home` + the "nothing on him yet" lines appear; fine in
  principle, but it also stops Cloud + a small Sword (Helm, Boots) attacking into a single chump-blocker.
F4. Tutor still returns SoFI/SoHH almost every time (cloud.rs:294 `cloud_pick`; trace reveals t1/t2: SoFI 20+, SoHH 10+, Helm of
  Kaldra / Shield / Sword of Kaldra only when an engine is out). Prior audit's tutor fixes are in; SoFF (+15 win rate in prior
  data) still rarely fetched. Low-medium.
F5. Protection is used rarely: `Cloud protects` 3 times in 60 games (Giver twice) against 66 Cloud removals; Plaza of
  Heroes is modeled now (fix2) but I saw no activation in 60 games. Most removals are instant exile with Cloud tapped/just cast;
  protection is not reachable with the mana spent on development. Mostly a list problem (Giver, Boots, Helm are the only
  pieces; fix2 PROTECT_FIRST delays casting Cloud when risk is high).
F6. Blocking is generic `engine/combat.rs:441 ai_blocks`: blocks only on good/trade blocks, else chump with probability
  `ai::chump_prob(...)`, spare tokens first. No Cloud-specific defence (unlike Y'shtola's special case at combat.rs:~500).
  I did not verify via traces (no block lines). Suggest adding a block log line before judging.
F7. Attack-tax payment (t2-9 R7 "pays 4 to attack", leaving a hand full of Equipment uncast, 4 lands): the AI pays full tax for
  a 6-damage attack with no mana left for development. Probably acceptable, uncertain.
F8. Unfinished Business never cast (analyze: "untagged"). Dead card in the 99 as far as the sim goes (card-model gap,
  `Recommission` cast only 2/60 games).
No wrong card rule found in the traced lines (Cloud tutor, Hero's Blade attach, Kor Outfitter, Puresteel, Riders, Disenchant,
Austere modes "le3, ge4" all behaved per Oracle).

## 3. Weaknesses of the 99 vs what beats it
- Interaction: 6 spot removal (Lay Down Arms, Stand Up, Citizen's Arrest, Secure the Scene, Fate, Disenchant) + Skyclave +
  Gatekeeper + 1 wipe (Austere, cast in 7/60 games). No instant-speed cheap removal (StP, Path), no counter, no fog, no lifegain
  beyond Staff/Steiner. Opponent combos/aggro/voltron are not answered before round 5-6.
- Defence: 29 creatures but mostly 1-3 power utility bodies that are held back only by `keep_cloud_home`; 2 attack taxes none;
  no Ghostly Prison / Propaganda / Archangel of Tithes effects; Cloud lifetime is 9-12 rounds against an opposing clock of 3-4
  attackers by R5.
- Draw/ramp are fine (5 rocks + Warhound/Knight/Solemn/Page; Swords/Mask/Court/Paladin draw).
- Curve: Kaldra set, Hammer (equip 8), Arm (equip 3) are uncastable/uncarriable without Paladin/Ardenn (11 casts of Paladin in
  60 games). Protection is reactive and four of its cards (Rootborn, Formation, Holy, Unfinished Business) are almost dead.
- Package D (`audit/cloud/packages/d-defense.swaps`): addresses combat races (Ghostly Prison, Norn's Annex, Archangel of Tithes,
  Windborn Muse, Baird = attack taxes; Wrath/Day/Fumigate = wipes that leave Equipment). Package R (`r-removal.swaps`):
  addresses spot removal and protection (StP, Path, Council's Judgment, Generous Gift, Mother of Runes, Flawless Maneuver).
  Evidence supports D first: 77 of Cloud's creature deaths are to wipes of the opponent's, and every loss is a combat race;
  R addresses the one thing that already works poorly (66 Cloud removals vs 3 protections), but Cloud's real need is to win
  the race, not to survive.

## 4. Ranked recommendations
AI/model fixes (cheap, small test):
1. F1: give the Cloud attack decision a bonus (or exclude `none`) when Cloud wears at least one Sword/trigger Equipment
   and an attack is not lethal-risky; expected +1 to +2 points (13% of equip turns).
2. F2: do not bounce an equipped commander with Riders unless the tutor target is unavailable; expected under +1.
3. F8: tag Unfinished Business (and check Recommission) so the sim can cast them; +0.3 est.
4. F4: lift SoFF in `cloud_pick` order (data says +15); small.
List changes (all cards are fully modeled per packages README; check with `--cards`):
5. Package D (8 swaps, $23): addresses F1-style tempo not at all, but races. Expected +3 to +6 (guess, no data: earlier Cloud
   deck measured Ghostly Prison +1.2 and Windborn Muse +1.0).
6. Package R (6 swaps, $24): spot removal + Mother of Runes/Maneuver: +2 to +4 (guess).
7. Single cuts if smaller steps preferred: Unfinished Business, Recommission, Rootborn Defenses (reactive, 0-5% cast) for
   Wrath of God / Swords to Plowshares / Ghostly Prison.
8. If D + R + AI fixes still leave Cloud under ~18%, accept that a 2/1 voltron commander can't reach T1/T2 parity in this pool.

## 5. Test plan (lead asks JD first)
All paired, look-ahead, loose, same seed range, vs T1 and T2, 1,000 games per tier, `--jobs 16` (about 7 min per tier per
variant): baseline, D, R, D+R (4 variants x 2 tiers x ~7 min = about 1 hour). AI fixes F1/F2 are code changes: run them on
baseline list first (2 variants x 2 tiers, ~30 min), then stack with the best package. Use 2,000 games per tier for any
difference under 2 points (the 4,000-game noise is about +-0.5 pt, 1,000 games about +-1.1).

Traces used are in the scratchpad only; they can be regenerated with
`commander-sim --deck cloud --pool t1 --profile loose --ai lookahead --trace N` (N=1..20; `--any-tier --pool t0` for T0).
