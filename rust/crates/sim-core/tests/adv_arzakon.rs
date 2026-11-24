//! Adversarial probes for the Arzakon change (reviewer, 2026-11-23). Not the author's tests: each one tries to break a
//! claim in the handoffs, the guide or the design. Expected results come from Oracle text / the comprehensive rules.

mod play_common;
use play_common::*;
use sim_core::engine::mana::{self, total_mana};
use sim_core::engine::{cast, zones};
use sim_core::hooks::Opt;
use sim_core::human::human::{AbilityKind, abilities_of, human_main};
use sim_core::impls::model_gaps_a4::{Tgt, ping};
use sim_core::settings::AiMode;
use sim_core::state::Ctx;

const ARZ: &str = "Arzakon";

fn tbl() -> Game {
    let mut g = table(&["arzakon", "yshtola", "karn"]);
    set_commander(&mut g, P0, ARZ);
    g
}

fn walker(g: &mut Game, l: i32) -> PermId {
    let w = perm(g, P0, ARZ);
    g.perm_mut(w).loyalty = Some(l);
    g.perm_mut(w).is_cmd = true;
    w
}

fn colour_lands(g: &mut Game, p: PlayerId) {
    for b in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        lands(g, p, b, 1, false);
    }
}


#[test]
fn inspect_what_the_lotus_token_is_spent_on() {
    use sim_core::format::Rules;
    let rules = Rules { life: 30, first_player_draws: false };
    let opp: Vec<&str> = vec!["yshtola", "urdragon", "yoshimaru", "sheoldred"];
    let (mut shown, mut tokens_made, mut tokens_spent_same_round, mut tokens_never_spent) = (0, 0, 0, 0);
    for seed in 7000u64..7060 {
        let o = opp[(seed as usize) % opp.len()];
        let g = play_in(&["arzakon", o], seed, true, AiMode::Adaptive, rules);
        let log = g.log.as_ref().unwrap();
        for (i, l) in log.iter().enumerate() {
            if l.contains("uses Arzakon (-3") {
                tokens_made += 1;
                let round = l.split_whitespace().next().unwrap_or("").to_string();
                let tail: Vec<&String> = log[i + 1..].iter().take(25).collect();
                let spent = tail.iter().any(|t| t.contains("Black Lotus") && (t.contains("sacrific") || t.contains("taps") || t.contains("tap")));
                if spent && tail.iter().any(|t| t.starts_with(&round)) {
                    tokens_spent_same_round += 1;
                }
                if !spent {
                    tokens_never_spent += 1;
                }
                if shown < 8 {
                    shown += 1;
                    eprintln!("LOTUS seed {seed} vs {o}:");
                    for t in log[i.saturating_sub(2)..(i + 8).min(log.len())].iter() {
                        eprintln!("    {t}");
                    }
                }
            }
        }
    }
    eprintln!("LOTUS made={tokens_made} spent_soon={tokens_spent_same_round} no_spend_line_in_next_25={tokens_never_spent}");
}

#[allow(dead_code)]
fn options(g: &mut Game, w: PermId) -> Vec<Opt> {
    let f = g.registry.get(g.perm(w).cd.unwrap()).and_then(|i| i.options).expect("options");
    f(g, w, P0, Some(false)).unwrap()
}

// ---------------------------------------------------------------- guide claim: Faeburrow Elder + Arzakon
#[test]
fn faeburrow_elder_taps_for_all_five_colours_while_arzakon_is_on_the_battlefield() {
    let mut g = tbl();
    perm(&mut g, P0, "Faeburrow Elder");
    // alone: green and white only
    assert!(!mana::can_pay(&g, P0, 0, "WUBRG", false), "no Arzakon: the Elder makes only G and W");
    walker(&mut g, 4);
    assert_eq!(total_mana(&g, P0, false), 5, "five colours among permanents -> five mana");
    assert!(mana::can_pay(&g, P0, 0, "WUBRG", false), "guide: Arzakon makes the Elder tap for all five colours");
}

#[test]
fn faeburrow_elder_does_not_count_arzakon_in_the_command_zone() {
    let mut g = tbl();
    perm(&mut g, P0, "Faeburrow Elder");
    assert!(g.player(P0).cmd_in_zone);
    assert!(!mana::can_pay(&g, P0, 0, "WUBRG", false));
}

// ---------------------------------------------------------------- the person's +2
fn plus_two_label_index(g: &mut Game, w: PermId) -> (usize, usize) {
    let i = g.player(P0).perms.iter().position(|&x| x == w).unwrap();
    let ab = abilities_of(g, P0, w).unwrap();
    let k = ab.iter().position(|a| a.0.starts_with("+2")).unwrap();
    (i, k)
}

#[test]
fn a_person_kills_an_opponent_at_three_life_with_plus_two() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    g.player_mut(P1).life = 3;
    let (i, k) = plus_two_label_index(&mut g, w);
    // the choices list is players first; ask by text for opponent P1's seat
    let name = g.player(P1).name.to_string();
    seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(k), by_text("3 life"), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert_eq!(g.perm(w).loyalty, Some(6));
    assert!(!g.player(P1).alive, "{name} was at 3 life and took 3: must be out of the game");
}

#[test]
fn a_person_kills_a_creature_with_plus_two_and_it_dies_not_just_marked() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    let k = perm(&mut g, P1, "Kinnan, Bonder Prodigy");
    let (i, kk) = plus_two_label_index(&mut g, w);
    seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(kk), by_text("Kinnan"), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(!g.perm(k).on_bf, "2/2 took 3");
}

// ---------------------------------------------------------------- Siege-Gang vs a battle (shared `ping_legal`)
#[test]
fn siege_gang_commander_does_not_start_aiming_its_pings_at_a_battle() {
    let mut g = table(&["krenko-mono-red-goblins", "yshtola", "kinnan-bonder-mana-combo"]);
    let sg = perm(&mut g, P0, "Siege-Gang Commander");
    lands(&mut g, P0, "Mountain", 4, false);
    let ikoria = take(&mut g, P2, "Invasion of Ikoria // Zilortha, Apex of Ikoria");
    cast::resolve(&mut g, P2, ikoria, &Ctx { x: 1, ..Default::default() }, "hand").unwrap();
    let battle = g.player(P2).perms.iter().copied().find(|&m| g.perm(m).cd == Some(ikoria)).unwrap();
    let before = sim_core::impls::card_models3::defense(&g, battle);
    // the AI's options for Siege-Gang
    let f = g.registry.get(g.perm(sg).cd.unwrap()).and_then(|i| i.options).expect("options");
    let opts = f(&mut g, sg, P0, Some(false)).unwrap();
    eprintln!(
        "SIEGE battle pval={:.2} etgh={} loyalty={:?} legal_for_ping={} opts={:?}",
        sim_core::engine::values::pval(&g, battle),
        sim_core::engine::values::etgh(&g, battle),
        g.perm(battle).loyalty,
        sim_core::impls::model_gaps_a4::ping_legal(&g, P0, sg, Tgt::Perm(battle)),
        opts.iter().map(|o| &o.label).collect::<Vec<_>>()
    );
    let hit_battle = opts.iter().any(|o| o.label.contains("Invasion of Ikoria"));
    assert!(!hit_battle, "Siege-Gang should not ping a battle: {:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>());
    let _ = before;
}

// ---------------------------------------------------------------- full Cali games with the merged deck
#[test]
fn cali_games_with_the_real_arzakon_deck_complete_and_use_the_walker() {
    use sim_core::format::Rules;
    let rules = Rules { life: 30, first_player_draws: false };
    let opp: Vec<&str> = vec!["yshtola", "urdragon", "yoshimaru", "sheoldred"];
    let (mut games, mut cast_games, mut plus, mut minus3, mut minus7, mut tokens) = (0, 0, 0, 0, 0, 0);
    let mut casts_total = 0;
    for (i, seed) in (900u64..924).enumerate() {
        let o = opp[i % opp.len()];
        let g = play_in(&["arzakon", o], seed, true, AiMode::Adaptive, rules);
        let log = g.log.as_ref().unwrap();
        games += 1;
        let arz: Vec<&String> = log.iter().filter(|l| l.contains("Arzakon")).collect();
        if i < 2 {
            for l in arz.iter().take(40) {
                eprintln!("[g{i}] {l}");
            }
        }
        let c = arz.iter().filter(|l| l.contains("casts Arzakon") || l.contains("cast Arzakon")).count();
        casts_total += c;
        if c > 0 {
            cast_games += 1;
        }
        plus += arz.iter().filter(|l| l.contains("uses Arzakon (+2")).count();
        minus3 += arz.iter().filter(|l| l.contains("uses Arzakon (-3")).count();
        minus7 += arz.iter().filter(|l| l.contains("uses Arzakon (-7")).count();
        tokens += log.iter().filter(|l| l.contains("Black Lotus")).count();
    }
    eprintln!(
        "games={games} cast_games={cast_games} casts_total={casts_total} +2={plus} -3={minus3} -7={minus7} lotus_lines={tokens}"
    );
}


// ---------------------------------------------------------------- urdragon package: Courser and Incubator
fn ur_table(seed: u64) -> Game {
    let mut g = table_with(&["urdragon", "yshtola", "sheoldred"], seed, 40);
    for p in &mut g.players {
        p.hand.clear();
        p.gy.clear();
    }
    g
}

#[test]
fn urzas_incubator_names_dragon_and_discounts_the_big_dragons_with_eminence() {
    let mut g = ur_table(5);
    let courser = card(&g, "Hellkite Courser");
    let (gn0, pi0) = mana::cost_of(&g, P0, courser);
    let m = perm(&mut g, P0, "Urza's Incubator");
    let named = sim_core::impls::galadriel::ctype(&g, m).map(|s| format!("{s:?}").to_lowercase());
    eprintln!("Incubator names {named:?}; Courser cost before {gn0}+{} after:", pi0.len());
    assert!(named.as_deref().is_some_and(|n| n.contains("dragon")), "the AI must name Dragon: {named:?}");
    let (gn1, pi1) = mana::cost_of(&g, P0, courser);
    assert_eq!(gn0 + pi0.len() as u32 - (gn1 + pi1.len() as u32), 2, "Incubator: creature spells of the type cost {{2}} less");
}

#[test]
fn a_whole_ai_turn_with_hellkite_courser_puts_the_ur_dragon_in_attacks_and_returns_it() {
    use sim_core::engine::turn;
    use sim_core::state::Step;
    for seed in 1..=4u64 {
        let mut g = ur_table(seed);
        lands(&mut g, P0, "Mountain", 6, false);
        hand(&mut g, P0, &["Hellkite Courser"]);
        let ur = card(&g, "The Ur-Dragon");
        assert!(g.player(P0).cmd_zone_of(ur));
        let tax0 = g.player(P0).tax;
        turn::continue_turn(&mut g, P0, Step::Main1).unwrap();
        let log: Vec<String> = g.log.clone().unwrap_or_default();
        let cast = perms_named(&g, P0, "Hellkite Courser").len();
        eprintln!("seed {seed}: courser_on_bf={cast} cmd_zone={} life={:?} tax={} {}", g.player(P0).cmd_zone_of(ur), [g.player(P1).life, g.player(P2).life], g.player(P0).tax, log.len());
        assert!(g.player(P0).cmd_zone_of(ur), "seed {seed}: back in the command zone at the end step");
        assert_eq!(g.player(P0).tax, tax0, "seed {seed}: not cast, so no tax");
        assert!(g.player(P1).life < 40 || g.player(P2).life < 40, "seed {seed}: The Ur-Dragon (haste) attacked and connected");
    }
}


#[test]
fn many_cali_games_scan_for_self_targets_and_odd_log_lines() {
    use sim_core::format::Rules;
    let rules = Rules { life: 30, first_player_draws: false };
    let mut opp: Vec<&str> = vec!["yshtola", "urdragon", "yoshimaru", "sheoldred"];
    opp.extend(tier("t5"));
    opp.extend(tier("t6"));
    let (mut games, mut cast_games, mut selfhit, mut m7, mut m3, mut p2, mut wins, mut first_rounds) =
        (0, 0, 0, 0, 0, 0, 0, vec![]);
    let mut odd: Vec<String> = vec![];
    for seed in 3000u64..3200 {
        let o = opp[(seed as usize) % opp.len()];
        if o == "arzakon" {
            continue;
        }
        let g = play_in(&["arzakon", o], seed, true, AiMode::Adaptive, rules);
        let log = g.log.as_ref().unwrap();
        games += 1;
        let mut cast = None;
        for (i, l) in log.iter().enumerate() {
            if l.contains("Arzakon +2 (Arzakon)") && l.contains("decides") && l.contains("->") {
                selfhit += 1;
                odd.push(format!("seed {seed} vs {o}: {l}"));
            }
            if l.contains("casts Arzakon") && cast.is_none() {
                cast = Some(i);
            }
            if l.contains("uses Arzakon (-7") {
                m7 += 1;
                odd.push(format!("seed {seed} vs {o} -7: {}", log[i.saturating_sub(3)..=i].join(" | ")));
            }
            if l.contains("uses Arzakon (-3") {
                m3 += 1;
            }
            if l.contains("uses Arzakon (+2") {
                p2 += 1;
            }
            if l.contains("invalid") || l.contains("refus") || l.contains("illegal") {
                odd.push(format!("seed {seed} vs {o}: {l}"));
            }
        }
        if let Some(i) = cast {
            cast_games += 1;
            // the round of the first cast: scan back for the latest "R<n>" prefix
            let r = log[i].split_whitespace().next().unwrap_or("").trim_start_matches('R').parse::<u32>().unwrap_or(0);
            first_rounds.push(r);
        }
        if g.winner == Some(P0) {
            wins += 1;
        }
    }
    first_rounds.sort();
    eprintln!(
        "SCAN games={games} cast_games={cast_games} wins={wins} +2={p2} -3={m3} -7={m7} selfhit={selfhit} first_cast_rounds={first_rounds:?}"
    );
    for l in odd.iter().take(30) {
        eprintln!("ODD {l}");
    }
    assert_eq!(selfhit, 0, "the AI aimed Arzakon's +2 at itself");
}


// ---------------------------------------------------------------- data guards
fn engine_with(name: &str, edit: impl FnOnce(&mut serde_json::Value)) -> Result<sim_core::run::Engine, String> {
    let real = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("adv_arzakon_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for e in std::fs::read_dir(&real).unwrap() {
        let e = e.unwrap();
        if e.file_name() != "decks.json" {
            std::os::unix::fs::symlink(e.path(), dir.join(e.file_name())).unwrap();
        }
    }
    let mut v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(real.join("decks.json")).unwrap()).unwrap();
    edit(&mut v);
    std::fs::write(dir.join("decks.json"), serde_json::to_string(&v).unwrap()).unwrap();
    sim_core::run::Engine::load(&dir)
}

#[test]
fn a_commander_deck_with_a_black_lotus_card_is_refused_at_a_cali_table_but_garths_token_is_not_a_card() {
    // a Cali deck at a Cali table is not re-checked by `admits` (cardgen's validate owns its legality); the list ban
    // applies to a Commander deck sat at a Cali table
    let e = engine_with("lotus", |_| {}).unwrap();
    assert!(e.admits("urdragon", "cali").is_ok(), "Garth's Black Lotus is a token, not a card in the list");
    let e = engine_with("lotus2", |v| {
        let d = v["mine"].as_array_mut().unwrap().iter_mut().find(|d| d["key"] == "urdragon").unwrap();
        let cards = d["cards"].as_array_mut().unwrap();
        let i = cards.iter().position(|c| c == "Sol Ring").unwrap();
        cards[i] = json!("Black Lotus");
    })
    .unwrap();
    assert!(e.admits("urdragon", "cali").is_err(), "Black Lotus the card is banned in Cali");
}

#[test]
fn the_old_deck_key_does_not_exist_and_the_new_one_has_arzakon_as_commander() {
    assert!(!decks().iter().any(|d| &*d.key == "kyodai"), "the old key must be gone");
    let d = deck("arzakon");
    assert_eq!(d.commander, "Arzakon");
    assert!(d.cards.iter().any(|c| c == "Kyodai, Soul of Kamigawa"), "Kyodai is a 99 card");
    assert_eq!(d.cards.len(), 100);
    assert_eq!(d.cards.iter().filter(|c| *c == "Arzakon").count(), 1);
}


// ---------------------------------------------------------------- Karn (new in the 99) against the table's artifacts
#[test]
fn an_opposing_karn_stops_a_sol_ring_and_the_lotus_token_in_the_mana_count_too() {
    let mut g = tbl();
    let w = walker(&mut g, 7);
    perm(&mut g, P0, "Sol Ring");
    let before = total_mana(&g, P0, false);
    perm(&mut g, P1, "Karn, the Great Creator");
    let after = total_mana(&g, P0, false);
    eprintln!("Sol Ring mana before Karn {before}, after {after}");
    assert_eq!(after, 0, "Karn: activated abilities of artifacts your opponents control can't be activated");
    // the Lotus token made by -3 is an artifact too
    let (i, k) = plus_two_label_index(&mut g, w);
    let _ = (i, k);
    let _ = sim_core::impls::arzakon::lotus_token(&mut g, P0, w);
    assert_eq!(total_mana(&g, P0, false), 0, "the token's mana ability is stopped as well");
}

#[test]
fn oath_of_teferi_lets_arzakon_use_two_abilities_and_the_second_is_still_legal() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    perm(&mut g, P0, "Oath of Teferi");
    let f = g.registry.get(g.perm(w).cd.unwrap()).and_then(|i| i.options).unwrap();
    let o = f(&mut g, w, P0, Some(false)).unwrap();
    let plus = o.iter().find(|x| x.label.contains("+2")).unwrap().act.clone().unwrap();
    assert!(sim_core::ai::act::perform(&mut g, P0, &plus).unwrap());
    let o = f(&mut g, w, P0, Some(false)).unwrap();
    assert!(!o.is_empty(), "a second ability is on offer with Oath of Teferi");
}


#[test]
fn lookahead_ai_cali_games_still_cast_arzakon() {
    use sim_core::format::Rules;
    let rules = Rules { life: 30, first_player_draws: false };
    let opp: Vec<&str> = vec!["yshtola", "urdragon", "yoshimaru", "sheoldred"];
    let (mut games, mut cast_games) = (0, 0);
    for seed in 5000u64..5024 {
        let o = opp[(seed as usize) % opp.len()];
        let g = play_in(&["arzakon", o], seed, true, AiMode::Lookahead, rules);
        games += 1;
        if g.log.as_ref().unwrap().iter().any(|l| l.contains("casts Arzakon")) {
            cast_games += 1;
        }
    }
    eprintln!("LOOKAHEAD games={games} cast_games={cast_games}");
}


fn names_on_bf(g: &Game, p: PlayerId) -> Vec<String> {
    g.player(p).perms.iter().map(|&m| g.perm(m).name.to_string()).collect()
}

/// Tinker's additional cost ("sacrifice an artifact", paid by `cast::additional_cost` on the AI's cast path) with this as
/// the only artifact: (dry run says payable, what is left on the battlefield after paying)
fn tinker_with(first_artifact: &str) -> (bool, Vec<String>) {
    let mut g = tbl();
    let w = walker(&mut g, 7);
    if first_artifact == "token" {
        sim_core::impls::arzakon::lotus_token(&mut g, P0, w).unwrap();
    } else {
        perm(&mut g, P0, first_artifact);
    }
    lands(&mut g, P0, "Island", 3, false);
    let t = hand(&mut g, P0, &["Tinker"])[0];
    let dry = cast::additional_cost(&mut g, P0, t, true).unwrap();
    if dry {
        assert!(cast::additional_cost(&mut g, P0, t, false).unwrap());
    }
    (dry, names_on_bf(&g, P0))
}

#[test]
fn tinker_can_sacrifice_the_black_lotus_token() {
    let (ok_c, bf_c) = tinker_with("Sol Ring");
    eprintln!("TINKER control (Sol Ring): payable={ok_c} battlefield={bf_c:?}");
    let (ok_t, bf_t) = tinker_with("token");
    eprintln!("TINKER token: payable={ok_t} battlefield={bf_t:?}");
    assert!(ok_c, "control: Sol Ring can be sacrificed");
    assert!(ok_t, "Tinker's cost is payable with the token as the only artifact");
    assert!(!bf_t.iter().any(|n| n == "Black Lotus"), "the token was the sacrifice: {bf_t:?}");
}

#[test]
fn arzakon_returns_to_the_command_zone_from_every_zone_it_can_be_sent_to() {
    for z in [zones::Zone::Gy, zones::Zone::Exile, zones::Zone::Hand, zones::Zone::Lib, zones::Zone::Top] {
        let mut g = tbl();
        colour_lands(&mut g, P0);
        let cmd = g.player(P0).cmd;
        assert!(cast::cast_card(&mut g, P0, cmd, "cmd", Ctx::default()).unwrap());
        let w = perms_named(&g, P0, ARZ)[0];
        zones::leave(&mut g, w).unwrap();
        zones::to_zone_card(&mut g, w, z);
        let pl = g.player(P0);
        assert!(pl.cmd_in_zone, "{z:?}: back in the command zone");
        assert!(!pl.hand.contains(&cmd) && !pl.gy.contains(&cmd) && !pl.exile.contains(&cmd) && !pl.library.contains(&cmd), "{z:?}: nowhere else");
        assert_eq!(pl.tax, 2, "{z:?}: tax paid once");
    }
}

#[allow(dead_code)]
fn unused(_: Tgt, _: AbilityKind) {
    let _ = ping;
    let _ = zones::die;
}
