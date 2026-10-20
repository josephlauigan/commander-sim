# Report: thrasios-ai

Branch thrasios-ai (from thrasios-deck). Files: impls/tymna_partners.rs, tymna_cards.rs, legends.rs (Tymna milestone),
common.rs (Ballista ping), titania.rs (two fns made pub), ai/search.rs, engine/combat.rs, engine/turn.rs (TRACE_HAND
diagnostic), sim-cli/report.rs (plan line, CARD_REPORT_MIN env var), tests/thrasios_ai.rs (12 tests),
audit/thrasios/FINDINGS.md. Shared additions are all key-gated to thrasios / kraum.

## Per gap
1. Blossoming Defense: untagged, so generic prio 0, and only Omnath had a protection path. `tymna_cards::blossom_protect`
   (thrasios seat; commander or pval >= 4; any targeted kind, not edict/wipe/sacrifice) via `titania::cast_protect`.
   Also a combat trick (`tymna_partners::combat_trick`, called in combat.rs after defend_hooks, both seats for Dark
   Endurance): a blocked commander / creature worth 3+ that would die, or a trade-up against a blocker worth 3+.
   200 games: cast in 9% of games (0 before), 40% of the games drawn.
2. Ballista: `common::ballista_prio` held it to 8 mana whenever Rings/Scepter/Power Artifact was anywhere in the deck.
   Thrasios seat now: held (0) only with Basalt AND Rings in hand/out and under 8 mana; else 50 (4-5 mana) / 56 (6+).
   Pings: X/1 creatures worth 1+ or mana dorks, keeping the last counter unless the target is worth 4+ (common.rs, thrasios
   only). Planeswalker loyalty pings not done. 200 games: drops out of the "stuck < 50%" list (20.6% -> 45% at
   MIN=5 list; not listed in the first run's 100-game list).
3. Thrasios: his option was only taken when it was the top-utility option; otherwise the search chose "stop". Now
   `thrasios_sink`: left out of the search, made when the search's best play among the rest is "stop" (search.rs); main-2
   keep covers protection spells as well as counters. Activations stay ~0.5-0.7 a game (base200 0.69, now 0.58, noise):
   the cause is that {4} is rarely spare (hands of 7+ cards; colour-stuck cards), not the choice.
4. Combo: Basalt 82 / 74 any turn, Rings 72 with Basalt (else 38), tutor fetches the missing piece of {Basalt, Rings, outlet}
   once two are in hand/out (`combo_tutor_pick`, Diabolic Tutor and Worldly Tutor). Plan: `tymna` milestone (first Tymna
   draw) + combo; plan_text/plan_turn for thrasios and kraum. Combo share of wins 10.6% -> 14.6% (200 games, noise).
5. Dark Endurance: cast 5.2% because the AI taps out on its own turn and it only answered destroy/damage removal (opponents
   exile, bounce or wipe). Now also a combat trick and cast on a commander against a destroy sweeper (kraum too);
   38.8% of the games drawn. Mana is not held for it (tried via hold_value: the search's hold bonus beat casting two
   creatures with ten lands; reverted).
6. Stuck cards: Avenge (wipe), Austere, Sun/Grave Titan, Elspeth, Sunblast, Dromoka, Sphinx, Hero of Bladehold are 6-7 mana
   or WW/UUBB: traces show 7-10 cards in hand with 8-9 lands of the wrong colours. No priority or tag bug found.
   Fact or Fiction / Diabolic Tutor: ordinary instant/sorcery priorities, cast when mana is spare.
7. Green cards: read against Oracle: Faeburrow Elder (one mana per colour among permanents, all usable in one payment),
   Paradox Surveyor (land or X card), Witherbloom Charm (3 modes), Bite Down (bites.rs), Coiling Oracle, Pilgrim, Dromoka,
   Talismans were already modeled and are cast in the traces. Nothing changed.

## Traces (14 games, TRACE_HAND=1)
Found: the hold-mana regression from my own first attempt (63 stops in 12 games, fixed); Tymna/Thrasios die together to
Rift overload / -X wipes in 7 of 14 games (no answer in the deck); the AI activates Basalt Monolith's untap with nothing to
spend (harmless, not touched); colour-stuck hands.

## For JD
- Win rate was not re-measured (200-game checks: base 25.5%, v1 26.0%, final 24%; noise +-6).
- `CARD_REPORT_MIN=5` lowers the card-report threshold for small diagnostic runs; `TRACE_HAND=1` prints each hand in a trace.
