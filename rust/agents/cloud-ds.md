# Task: cloud-ds — double strike on Cloud: verify the engine, design a 2-3 card package (no batch sims)

Read `rust/agents/T0-RULES.md` (Hard rules, How cards are modeled, Checks before you finish items 1-3) and
`audit/cloud/FINDINGS.md`. Branch cloud-ds (= PR #73: Cloud with Package D2). `-j 16` builds, ≤300-game smoke runs only.
Report `rust/agents/report-cloud-ds.md`. Every background wait has a hard timeout; leave nothing running.

## Idea
Cloud, Midgar Mercenary doubles the triggers of himself and of Equipment attached to him. A Sword's "deals combat
damage to a player" trigger fires once per combat-damage step, so with **double strike** an equipped Cloud should get
each Sword trigger **4 times per attack** (2 steps x Cloud's doubling), and twice the commander damage. Cloud wins
almost only by damage; defense (D, D2) gave him time — double strike should make the turns he connects count more.

## 1. Verify the engine first (most important)
Write tests (tests/cards_cloud_ds.rs) that put Cloud with a Sword (e.g. Sword of Fire and Ice) and a double-strike
source on the battlefield, attack an open player, and count: (a) combat damage dealt (and commander damage) — 2x;
(b) Sword trigger resolutions — expect 4. Check first-strike/regular damage steps in engine/combat.rs and how Cloud's
doubling (impls/cloud.rs) hooks combat-damage triggers. If anything is wrong (triggers only once per combat, doubling
skipped on the first-strike step, commander damage not doubled), FIX it with the smallest change, with tests, and list
the shared-file edits. Check that other decks are unchanged (same-seed identity: build the parent commit's binary in a
detached worktree under .claude/worktrees/, removed after; `--deck <d> --pool all --games 300 --ai adaptive --jobs 16
--seed 7 --brief` for the other 10 JD decks — identical, unless the fix is a real rules fix, then report what moved).

## 2. Design the package (2-3 swaps, mono-white legal, Bracket 3 — no new Game Changers)
Double-strike sources for Cloud: e.g. Fireshrieker (equip {2}: double strike), Duelist's Heritage, Akroma's Will,
Silverblade Paladin, True Conviction, Ancestral Blade-style, anything better you find. Check `cardgen audit --cards`
(Full only; model small gaps with tests), ownership (bulk-cards.txt / binder-cards.txt, not in another deck — grep
decklists/JD and decklists/Avery), prices (Scryfall batch collection POST). Make sure the AI actually uses them on Cloud
(equips Fireshrieker to Cloud, casts Heritage/Akroma's Will when Cloud attacks with a Sword) — trace a few games.
**Cuts:** only the weak non-creature slots: Secure the Scene (stuck 50%), Veteran's Powerblade (stuck 36%), then the
next weakest non-creature card by the data. Never cut creatures, rocks, lands, Package D/D2 cards, Swords or Kaldra.
Write `audit/cloud/packages/ds-doublestrike.swaps` and a README table (card, owned/price, why).

## 3. Smoke + plan
One ≤300-game smoke run (`--swap` set, T1) to confirm it loads and the pre-flight is clean; claim no result. Propose a
quick test (1,000 games, T1, paired) and a confirmation (2,000, T1 + T2).

Commit (no Co-Authored-By), never push. Report ≤15 lines: line 1 DONE/HANDOFF, line 2 model ID, engine verdict (and any
fix), the package (swaps, cost), AI usage seen.
