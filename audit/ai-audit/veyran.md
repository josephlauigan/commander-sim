# Veyran, Voice of Duality (veyran) — AI audit, phase 1
Izzet spellslinger: Veyran ({1}{U}{R}) doubles magecraft and "whenever you cast an instant or sorcery" triggers, so cheap
spells feed pingers (Guttersnipe, Kessig, Ral), token makers (Young Pyromancer, Third Path Iconoclast, Talrand,
Murmuring Mystic, Docent, Rite of the Dragoncaller), Storm-Kiln Artist Treasure and Niv-Mizzet/Emeritus draw; a big turn
(Thousand-Year Storm, Fiery Emancipation, Aetherflux, Crackle) ends it. Four tutors: Imperial Recruiter, Fabricate,
Solve the Equation, Mystical Tutor. Weak to creature decks and to Veyran being removed.
Priority function: `veyran_prio`, `ai/veyran.rs` (via `decks.rs::deck_prio`). Tutor wish list: `decks.rs::tutor_pick_named`
arm `"veyran"`. Plan config: milestone "Veyran + a magecraft payoff on the battlefield" (`engine_payoff`); main-phase
options in `veyran::options` (Kitten+Firesinger combo, Aetherflux shot, Boots/Greaves equip, Mizzix's Mastery).

Method: `--deck veyran --pool t2 --analyze --ai adaptive --games 300 --jobs 2` (33.3% conservative / 31.7% loose; Veyran
cast median T4, online by T4 19%, T6 47%, T8 65%; Veyran removed 1.84 times per game; loses 62% of games, mostly to
creature combat) plus 60 play-by-play traces (`--trace 0..59`, conservative, t2) read for casts, tutor choices and
wipes. `--cards`/DSL/code grep for model status (`cardgen audit --deck veyran` is NOT safe, see the end).

## Problems (ranked by likely impact on win rate)
1. Triple Triad is never cast: `veyran_prio` has no branch for it (no tag, no name match), so it returns 0. The card IS
   modeled (`impls/mine.rs`, `triad_upkeep`, upkeep hook registered at mine.rs:4426), so the earlier "untagged" report
   is a valuation gap, not a model gap. Evidence: 0 casts in 60 traces (the only nonland card with no "casts" line in the traces besides Mystic Confluence, which the
   card report says is cast in 28% of the games it is drawn); the `--cards` row shows `(no tags)`. Fix: add a `veyran_prio` branch (e.g. 55-60 at 6+ mana
   with an engine out, so its free spells trigger Veyran).
2. Imperial Recruiter fetches the wrong creature: the "cre2" wish list in `tutor_pick_named` is a leftover of the
   Displacer Kitten combo (Kitten is not in this deck: "Combo attempted 0.0%"). With no Kitten/Firesinger held it returns
   Displacer Kitten, then Blazing Firesinger // Seething Song. Evidence: 18 of 20 Recruiter casts fetched Blazing
   Firesinger // Seething Song (1 Kessig, 1 Guttersnipe). The doc says Guttersnipe/Kessig for damage, Talrand/Pyromancer
   for a board, Storm-Kiln Artist for mana, Archmage Emeritus for cards. Fix: board-state wish list as in the doc.
3. Blazing Firesinger // Seething Song is the top cast priority: `Tag::Vfire` returns 74, above Veyran (70),
   Recruiter (73), pingers (72) and token makers (71). Same Kitten leftover. Evidence: cast 40 times in 60 traces
   (cast in 53% of games, win rate when cast +5.4, the lowest of the reliable cards), as a 2/3 body. Fix: drop to about
   45-50 and let the engine/prepared-spell logic decide (the ritual is a mid-chain card).
4. Lightning Greaves comes after Veyran: prio 50 (`prot=boots`) vs Veyran 70 and engines 71-74, while the doc says
   "before or with Veyran". Evidence: in the 17 traces where Greaves was cast, only 2 had it before or with Veyran, 15
   after; Veyran was removed 1.84 times per game (about 47 destroy, 42 exile, 11 bounce, 7 tuck in 60 games), none of
   those with shroud up. Also Fabricate never fetches Greaves (wish list Aetherflux, Sol Ring, Fellwar Stone; 11/5/3 in
   19 casts). Fix: Greaves at 75 when Veyran is in hand/cast-able this turn or removal_risk is high; add it to Fabricate.
5. Mystical Tutor / Solve the Equation wish list is cantrips and counters only (kind "is" arm: Counterspell, Force of
   Will, Mystic Confluence, Stock Up if no counter in hand; else Stock Up, Flow State, Quick Study, Crackle). Evidence,
   Mystical Tutor (21 casts in 60 games, 8 Stock Up, 5 Counterspell, 3 Mystic Confluence, 2 Flow State, 1 Fabricate):
   it finds Stock Up in most games, never a finisher or a reset. Solve the Equation (13 casts): Counterspell x7, Stock Up
   x4, Mystic Confluence x1; Crackle with Power only appears once (Sanar's Wild Idea). The deck loses 62% of games, mostly
   to creature combat (Windgrace 18%, Kaalia 10%, Brago 10%); Blasphemous Act (11 casts) and Splatter Technique (14) are
   never fetched. Answer for JD: Mystical fetches mostly Stock Up (a four-mana-total draw), else Counterspell. Fix: add
   state-based picks: Crackle with Power/Thousand-Year Storm-support when payoffs + 7+ mana, Blasphemous Act / Splatter /
   River's Rebuke when opponents out-board Veyran, a counter only when holding mana. Solve at 3 mana for Stock Up (3 more)
   is a poor use: require a real target.
6. Docent of Perfection, Eris, Niv-Mizzet, Alania, Harmonic Prodigy, Flux Channeler fall to the generic `d.creature` 40,
   below cantrips (48) and rocks (80 until T5). Evidence: Docent and Eris sit in the "stuck in hand" list (cast in 46% and
   43% of games where drawn); Niv-Mizzet, Parun cast only 9 times in 60 traces, Alania 15, Docent 9, Eris 8. Docent has
   a cast-token engine (a spell-token maker with a Final Iteration payoff), Alania and Prodigy multiply triggers,
   Niv converts draws to pings; none is valued for that. Fix: give them engine-style priorities scaled by spells in
   hand and existing payoffs.
7. Dead Kitten-combo code: `options` "go for the combo", `veyran_try_combo`, `Vkitten` handling, Inventors' Fair option,
   `Mystic Remora`/`Chromox`/`Moxd`/`Narset`/`Breach` branches in `veyran_prio`. Combo attempted in 0.0% of games. Not
   harmful alone but it is why problems 2 and 3 exist.
8. Consecrated Sphinx (new): modeled (Approximate, `impls/t4.rs` draws two per opponent draw, capped by library) and
   valued by `sphinx_prio` (30 + 1.6 x expected cards, 15..75; tag `sphinx`). 10 casts in 60 traces; removed in 3 games by
   Veyran's own/other wipes and 6 by spot removal. No problem found; the draw trigger prints no log line, so I could not
   see the draws in the trace (unknown whether Niv-Mizzet pings are counted).
9. Mystic Confluence and An Offer You Can't Refuse sit in "stuck in hand" (28% / 50% of games where drawn) but are
   flagged reactive; Confluence also has an end-step "draw three" option. Plausible, not a defect.

Checked and fine: Blasphemous Act is cast when the table out-boards Veyran (7 opposing vs 1 own permanent in 17.txt R6);
Propaganda is cast early (prio 52 + opp power, 23 casts); Fiery Emancipation 64 once a pinger or creature is out (20
casts); Aetherflux fires in 5.3% of games; Splatter Technique both modes are chosen sensibly (draw four vs wipe).

## All cards
Legend as in cloud.md; Model: F full, A approximate, P partial, U unmodeled (from tags, DSL rows, code hooks).
Casts = raw counts over 60 traces.

| Card | Model | Valuation | Evidence / note |
|---|---|---|---|
| Veyran, Voice of Duality | F | deck (cmd prio 70) | cast 191 times (recasts); removed 1.84/game; cast median T4 |
| Abrade | F | generic-ok | 20 casts |
| Aetherflux Reservoir | F | deck (66) | 26 casts; cast in 46% of games, +11.9; fired 5.3% |
| Alania, Divergent Storm | F | generic-WRONG (creature 40) | 15 casts; problem 6 |
| An Offer You Can't Refuse | F | generic-ok (reactive) | 8 counters cast; stuck 50% |
| Arcane Signet | F | deck (rock 80/40) | 18 casts |
| Archmage Emeritus | F | deck (spelldraw 58) | 22 casts |
| Banishing Betrayal | F | generic-ok | 23 casts |
| Blasphemous Act | F | generic-ok | 11 casts, sensible (problem list) |
| Blazing Firesinger // Seething Song | F | generic-WRONG (74) | problem 3; 40 casts |
| Burst Lightning | F | generic-ok | 19 casts |
| Chaos Warp | F | generic-ok | 21 casts |
| Consecrated Sphinx | A | deck (`sphinx_prio`) | problem 8; 10 casts |
| Counterspell | F | generic-ok (reactive) | 12 counters; fetched by Mystical/Solve 12 times |
| Crackle with Power | F | generic-ok | 16 casts; only Wild Idea fetched it once |
| Deduce | F | generic-ok (draw spell 48) | 22 casts |
| Docent of Perfection // Final Iteration | A (transform "partly") | generic-WRONG (creature 40) | 9 casts; stuck 54% of draws |
| Dreams of Laguna | F | generic-ok | 32 casts |
| Emeritus of Conflict // Lightning Bolt | A | generic-ok | 18 casts |
| Emeritus of Ideation // Ancestral Recall | A | generic-ok | 13 casts + 13 Ancestral copies |
| Eris, Roar of the Storm | A | generic-WRONG? (creature 40) | 8 casts; stuck 57% of draws; cost reduction needs a full yard |
| Expressive Iteration | F | generic-ok | 24 casts |
| Fabricate | F | deck (tut=art 55, wish list) | 20 casts; Aetherflux 11, Sol Ring 5, Fellwar 3; never Greaves (problem 4) |
| Fellwar Stone | F | deck (rock) | 14 casts |
| Fiery Emancipation | F (kw triple) | deck (64/40) | 20 casts |
| Flashback | F | deck (38 with a draw spell in yard) | 16 casts |
| Flow State | F | generic-ok | 30 casts; cast in 42%, +9.5 |
| Flux Channeler | F | generic (creature 40) | 19 casts; fine as cheap Wizard |
| Guttersnipe | F | deck (ping 72) | 22 casts; +8.9 |
| Harmonic Prodigy | F | generic-WRONG? (creature 40) | 17 casts; a trigger multiplier |
| Imperial Recruiter | F | deck prio 73, **wish list wrong** | problem 2 |
| Kessig Flamebreather | F | deck (ping 72) | 24 casts |
| Light Up the Stage | F | generic-ok | 21 casts |
| Lightning Bolt | F | generic-ok | 18 casts |
| Lightning Greaves | F | deck (50) but too low | problem 4; 17 casts |
| Lightning Strike | F | generic-ok | 13 casts |
| Mizzix's Mastery | F | deck (options, overload/one spell) | 15 casts |
| Murmuring Mystic | F | deck (58) | 18 casts |
| Mystic Confluence | F | generic-ok (reactive, EOT draw) | stuck 72% of draws |
| Mystical Tutor | F (tut=is, top) | deck prio 30, wish list weak | problem 5; 21 casts |
| Niv-Mizzet, Parun | F | generic-WRONG? (creature 40) | 9 casts |
| Old Fat Spider Can't See Me | A | deck (46) | 12 casts |
| Plunder the Trollshaws | F | generic-ok | 35 casts |
| Prismari Charm | F | generic-ok | 18 casts |
| Propaganda | F | deck (52 + opp power) | 23 casts |
| Quick Study | F | generic-ok | 20 casts |
| Ral, Storm Conduit | F | deck (ping 72) | 19 casts |
| Rite of the Dragoncaller | F | deck (60) | 19 casts |
| River's Rebuke | F | generic-ok | 17 casts |
| Sanar, Unfinished Genius // Wild Idea | A | generic (creature 40) | 21 casts + 10 Wild Idea copies; Wild Idea fetches Stock Up/Crackle/Counterspell |
| Sol Ring | F | deck (rock 80/40) | 23 casts; +14.9 |
| Solve the Equation | F | deck (tut=is 30), wish list weak | problem 5; 13 casts |
| Spell Pierce | F | generic-ok (reactive) | 9 counters |
| Splatter Technique | F (modal) | generic-ok | 14 casts |
| Stock Up | F | generic-ok | 28 casts; +13.7 |
| Storm-Kiln Artist | F | deck (kiln 70) | 20 casts |
| Talrand, Sky Summoner | F | deck (spelltok 71) | 22 casts |
| Think Twice | F | generic-ok | 40 casts |
| Third Path Iconoclast | F | deck (spelltok 71) | 19 casts |
| Thor, Asgard's Avenger | A | deck (50) | 13 casts |
| Thousand-Year Storm | F | deck (57) | 13 casts |
| Triple Triad | F (impls/mine.rs upkeep hook) | **generic-WRONG: prio 0** | problem 1; 0 casts |
| Unsummon | F | generic-ok | 24 casts |
| Vibrant Outburst | F | generic-ok | 15 casts |
| Visions of Beyond | A | generic-ok | 26 casts |
| Young Pyromancer | F | deck (spelltok 71) | 25 casts |
| Lands: Ash Barrens, Coastal Peak, Command Tower, Desolate Lighthouse, Evolving Wilds, Exotic Orchard, Mistrise Village, Mystic Sanctuary, Path of Ancestry, Scorched Geyser, Shivan Reef, Sokenzan, Spectacle Summit, Temple of Epiphany, Terramorphic Expanse, Island, Mountain | F | generic-ok | played 12-21 times each in 60 games; Lighthouse looting and surveil options used; no problems seen |

## Tooling warning
`cardgen audit --deck <jdkey>` (and `--md`, `--mine`) OVERWRITES `decklists/JD/<deck>.md` with a model-status table. It clobbered the Cloud doc and then the Y'shtola doc during this survey (both restored with `git checkout`, `git status` is clean except `audit/ai-audit/` and the brief). Do not run it on JD decks. Also: commander-sim refuses every deck while any JD doc lacks an "## Import list".
