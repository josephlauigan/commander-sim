# Task: tymna-fix — fix the AI problems the first thrasios vs kraum run found

Read `rust/agents/TYMNA-RULES.md` first (it points to OVERNIGHT-RULES). Base branch: `tymna-thrasios` (the commit that
added this brief). Your worktree's branch: `tymna-fix`. You are the only agent running: `cargo -j 16`, `--jobs 16`.
Previous work: `rust/agents/report-partner-engine.md`, `report-tymna-cards.md`, `report-tymna-ai.md`. Deck plan
module: `impls/tymna_partners.rs`; card code `impls/tymna_cards.rs`; Tymna in `impls/legends.rs`.

The run (1,000 games per tier, look-ahead, loose; outputs in `audit/tymna/2026-10-13/<deck>-t<N>.txt`, `--analyze`)
gave thrasios 10.4% and kraum 16.6%, but the analysis shows AI problems. Fix each, for **both** decks where it
applies, and say what the cause was:

1. **Never cast, "untagged"** (thrasios): Lagrella, the Magpie; Witherbloom Charm; Shardmage's Rescue. Find why the
   analysis calls them untagged (hand tags in `data/carddb.txt` apply to JD's decks; Lagrella's may have gone with the
   retired jodah deck) and give them what the AI needs to cast them sensibly. Check every other card in both lists
   for the same problem (grep the analysis outputs, and `cardgen`'s tags).
2. **Ultimate Magic: Holy never cast** (reactive): wire both decks into the protect / wipe response dispatch
   (`ai/decks.rs` `protect_response` / `wipe_response`) so Ultimate Magic: Holy answers a wipe or removal on a
   commander or Tymna, and Shardmage's Rescue answers targeted removal on them.
3. **Commanders land late:** Thrasios lands on turn 4.4 on average (2 mana), Tymna on 5.1 (3 mana), Kraum on 6.8 (5).
   Trace games (`--trace`) and find out why: is the deck priority used for commanders in the command zone (the
   partner engine casts them "with the generic commander priority"), are colours missing (four-colour mana base),
   is the second commander delayed behind the first? Fix what is AI; report what is mana (with the numbers).
4. **Thrasios activates only ~0.5 times a game.** He should use all spare mana: at the end of the turn before yours,
   and on your own turn after your main-phase plays. Find out what blocks it (window not offered, utility too low,
   mana spent elsewhere, library check) and fix it.

## Check it works
Same-seed check (other decks identical: `--deck sauron --pool t3`, `--deck cloud --pool t5`, seed 7, 48 games,
lookahead). Before/after on 200 games per deck vs t1 and t3 (`--analyze --seed 1013`): win rate, landing turns,
Thrasios activations, Tymna cards per game, the never-cast list. Numbers in `rust/agents/report-tymna-fix.md`.

## Done when
OVERNIGHT-RULES checks 1-3 pass, the same-seed check is identical, no card in either deck is listed "never cast by the
sim", the report has the before/after numbers.
