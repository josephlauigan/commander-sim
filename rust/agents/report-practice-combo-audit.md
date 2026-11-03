# Report: practice-combo-audit (11/03) — can a person play JD's infinite combos in practice mode?

Code reading only (no tests run): every Broken/Unsure needs a practice-mode test before fixing. Paths under
`rust/crates/sim-core/src/`.

## How a person's turn reaches a card
- Activations come from `human/human.rs:522` `abilities_of`: Equip at printed cost (`human.rs:594`),
  `human/abilities.rs`, `human/karn.rs` (17 cards, any deck), DSL abilities, a short named list in
  `human/cards.rs:850`, else the AI's `Event::Options` hook (with the AI's own fodder/target/timing filters). A card
  with none of these has no activation.
- A spell with no DSL, no `resolve` hook and no tags does nothing (`engine/cast.rs:1016`).
- Triggers are automatic for the person; priority on their own trigger only with auto-pass "all" (`human.rs:145`).
  Different simultaneous triggers prompt "which resolves first?" each time (`human.rs:1022`).
- Creature damage is "kill if toughness <= n, else nothing", never marked (`human/cards.rs:206`,
  `engine/removal.rs:278-310`).
- The only loop shortcut is Sephiroth's (`human/abilities.rs:162`, gated on `key == "seph"`); `mine::loops_now`
  (`impls/mine.rs:3391`) is already deck-agnostic.

## No infinite or game-ending loop in the current list
Cloud, Galadriel, Alela, Sauron (write-ups say so; no bot combo code keyed to them).

## Urdragon
| combo | status | detail |
|---|---|---|
| Godo + Helm of the Host | Works by hand (tedious) | Helm token at beginning of combat for a person (`engine/combat.rs:1066-1074`, `impls/t3.rs:948`); Godo ETB fetch `impls/t6_godo.rs:89-102`; first attack untaps + extra combat (`t6_godo.rs:126`, `combat.rs:1309`). ~10-15 combats = 30-50 prompts (attackers + target each combat, `human/combat.rs:79-130`), plus a search per Godo token. Person's Godo fetch limited to the AI's wanted card (`t6_godo.rs:62`). Equip discounts (Puresteel, Steelshaper) ignored for a person (`human/legal.rs:397` vs `impls/cloud.rs:184`). Hammer (`t6_godo.rs:164`), Sigarda's Aid (`impls/partials.rs:1036`), Brass Squire (`impls/cloud.rs:513-548`, main 1 only) choose the attach target for the person. |
| Combat Celebrant + Helm | Works by hand (tedious) | Same chain; token Celebrant exerts automatically (`impls/t3.rs:834`). |
| Dualcaster Mage + Twinflame | Broken | Twinflame's effect exists only in the AI hand option (`impls/mine.rs:3147-3207`); cast normally it does nothing, so Dualcaster's copy (`engine/cast.rs:1389`) copies nothing. |
| Camera / Jaxis backups | Broken | Camera has no person activation (`impls/t6_copies.rs:412`, called only from `engine/stack.rs:720`). Jaxis's copy ability: not found outside the abstract steps; test it. |

## Sheoldred
| combo | status | detail |
|---|---|---|
| Mikaeus + Triskelion | Broken | Triskelion self-pings never kill (no marked damage). `engine/zones.rs:906` (`!x.undying`) blocks a second undying return. `zones.rs:913` sets `plus = 1` after `enter`, maybe overwriting Triskelion's three counters (test). Bots: abstract win (`impls/mikaeus_trisk.rs`). |
| Yawgmoth + Mikaeus undying + drain payoff | Broken | No person path: AI hook `impls/t5.rs:717` offers one option with the AI's cheapest fodder, main phase only, life > 8; fodder limited to tokens/cheap/printed-undying; the -1/-1 counter only goes to an opposing toughness-1 creature. |
| Altar / Seer / Tower outlets | Works, not infinite | `human/cards.rs:694`, `:2481`. |

## Thanos
| combo | status | detail |
|---|---|---|
| Thassa's Oracle + Demonic Consultation / Tainted Pact | Broken | Consultation and Pact have no card code, so they do nothing (`engine/cast.rs:1016`). The Oracle trigger resolves at once (`impls/combos.rs:1300-1316`): no "in response". Oracle ETB also auto-reorders the top cards. |
| Breach + LED + Brain Freeze + Oracle | Broken | Breach escape works (`human/legal.rs:531`, `human.rs:679`, `:935`). LED has no person activation (`led_options` only from `ai/brain.rs:1181`). Brain Freeze always mills the lowest-library opponent (`mine.rs:931-945`). Grapeshot targets AI-chosen. |
| Ad Nauseam into a win | Broken (no control) | `engine/cast.rs:1150` calls the AI's `ad_nauseam` (`engine/tutors.rs:539`): stops at the AI's life floor, 15-card cap or `combo_ready`; the person is never asked. |
| Necropotence | Works | `human/cards.rs:2404`. |

## Kefka
| combo | status | detail |
|---|---|---|
| Psychosis Crawler + flipped Kefka | Works by hand (test stack depth/prompts) | `mine.rs:875`, compiled Crawler DSL; `{8}` flip is an AI hook paying from the person's pool (`mine.rs:782`). |
| Niv-Mizzet Firemind / Parun + Kefka | Needs a shortcut | Every draw prompts for a 1-damage target (`dsl/mod.rs:631` → `human/choices.rs:433`), plus trigger order with two draw triggers: 40+ rounds. |
| Breach + storm finish | Works, not a loop | Storm copy targets AI-chosen. |

## Karn and Thrasios
- Basalt Monolith + Rings (+ Ballista / Staff): Broken, being fixed in `practice-rings`.
- No other Karn loop (Grim Monolith + Rings nets 0; Hangarback, Skullclamp, Self-Replicator, Mirrorworks finite).
- Thrasios's {4} dig is an AI hook (`impls/tymna_cards.rs:74-108`): offered only in main 2 or at instant speed, and
  in main 2 only when untapped mana (not the pool) covers cost plus a held counterspell (`:100`). Needs a person path.

## Yshtola
No infinite combo. X drain + Y'shtola + Sanguine Bond + Blight-Priest works by hand (X asked `human.rs:805-904`,
Bond target `impls/t4.rs:1332`); Displacer Kitten asks what to flicker (`mine.rs:1407`).

## Ordered fix list
1. Basalt + Rings (practice-rings). 2. Thanos Oracle + Consultation / Pact. 3. Sheoldred Mikaeus + Triskelion and
Yawgmoth. 4. Thanos Breach line (LED, Brain Freeze target, Ad Nauseam stop). 5. Urdragon Helm loops (extra-combat
shortcut, attach-target picks, equip discount, free Godo fetch). 6. Urdragon Dualcaster + Twinflame, Camera, Jaxis.
7. Kefka Niv loops (standing ping target). 8. Thrasios dig in main 1 and from the pool.

## Needs a practice-mode test
Kefka Crawler loop with a 45-card library; counters on the returned Triskelion; the person's window on their own
Twinflame; Jaxis's copy ability under `use`; Thrasios's dig with pool mana.
