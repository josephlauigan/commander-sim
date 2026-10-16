# Task: cloud-wide — design option #3: an Equipment deck with several threats (same commander, mono-white)

Read `rust/agents/T0-RULES.md` (Hard rules, How cards are modeled, Keeping context small). Branch cloud-wide (= PR #71).
`-j 16` builds, ≤300-game smoke runs only (no batch sims; JD approves tests). Handoff `rust/agents/handoff-cloud-wide.md`,
report `rust/agents/report-cloud-wide.md`. Any background wait has a hard timeout; leave nothing running.

## Context — read `audit/cloud/FINDINGS.md` first
Cloud, Midgar Mercenary ({W}{W} 2/1; ETB tutor an Equipment; doubles triggers of himself / Equipment attached to him).
Today's list (PR #71) wins ~20.6% T1 / 18.2% T2. Small packages all failed; the limit is that the deck is ONE fragile
threat whose plan costs Cloud + Equipment + equip on turns 2-4, and removal on Cloud resets it. Package D (attack taxes
+ wipes) was the one big gain (+6). JD wants option #3: keep Cloud as commander, but rebuild the 99 so the Equipment
goes on **several bodies** — Cloud becomes a recastable tutor engine instead of the only threat.

## What to produce
1. **A full 100-card list** (Cloud + 99, mono-white, Bracket 3: at most 3 Game Changers — today Smothering Tithe and
   Enlightened Tutor; flag any third), as an import list file `audit/cloud/wide/list.txt` plus a swap file against
   today's list (`audit/cloud/wide/wide.swaps`, one `OUT=>IN` per line) so it can be tested paired.
   Directions: Equipment-payoff bodies (e.g. Kemba, Kha Regent; Puresteel Paladin; Armory Automaton; Sram; Akiri-style
   free equips; Adeline / token makers so there's always a carrier), cheap equip / free-attach (Brass Squire, Sigarda's
   Aid, Lightning Greaves, Shuko-style), Equipment that's good on any body (Swords, Skullclamp, living weapons /
   Batterskull-style germs so the Equipment brings its own body), and **keep Package D's defence** (it was the one big
   win). Keep the mana base and rock count at least as high as today (cutting rocks always lost).
2. **Ownership first:** cards in JD's Cloud list, `bulk-cards.txt`, `binder-cards.txt` that are NOT in another deck
   (grep decklists/JD and decklists/Avery — decks never share cards); then cheap buys (Scryfall batch collection POST
   for prices, never one-by-one). A table: card, owned/price, role.
3. **Model status:** every card Full in `cardgen audit --cards` and no WARNING in the `--swap` pre-flight. List any
   strong candidate that isn't, with what's missing; model small ones (with tests), list big ones for the lead.
4. **AI check (important):** does Cloud's AI spread Equipment onto other bodies well? Read the equip-target logic
   (impls/cloud.rs equip holder choice, `cloud_pick`, dsl equip_options, `keep_cloud_home`), trace ~10 games of the new
   list (`--swap` set) and report where the AI still funnels everything onto Cloud or holds Equipment. Don't fix it;
   describe the fix and where.
5. A smoke run (≤300 games, T1) only to confirm the list loads and plays; claim no result.
6. A test plan for the lead: paired look-ahead vs today's list, 2,000 games per tier, T1 then T2, plus a variant
   without any AI change; time estimates.

Commit (no Co-Authored-By), never push. Report ≤15 lines: line 1 DONE or HANDOFF, line 2 model ID, list size & buys
cost, model gaps, AI findings, proposed test.
