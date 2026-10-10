//! The turn (Python's `ais.py` "turn structure" section and the turn loop): lands entering tapped or not, playing
//! lands, upkeep, the end step, the five steps of a turn, mulligans, and a whole game.

use crate::ai;
use crate::cardcode;
use crate::cards::{EnterRule, Types};
use crate::engine::combat::{blocked, combat};
use crate::engine::hooks::fire_trigger;
use crate::engine::life::{check_state, lose_life};
use crate::engine::mana::{can_pay, land_colors_now, pay, total_mana};
use crate::engine::stack::{step_priority, trigger_window};
use crate::engine::values::*;
use crate::engine::zones::*;
use crate::flow::{Res, Stop};
use crate::hooks::{Call, Event};
use crate::ids::{CardId, LandId, PlayerId};
use crate::state::{Game, Step};
use crate::sym::Sym;
use crate::tag::Tag;

// ------------------------------------------------------------------ lands
fn cards_named(g: &Game, p: PlayerId, names: &[&str]) -> bool {
    g.opps(p).any(|q| {
        g.player(q).perms.iter().any(|&m| !g.perm(m).phased && card_name(g, m).is_some_and(|n| names.contains(&n)))
    })
}

/// does land cd enter tapped for p? Python's `land_enters_tapped`
pub fn land_enters_tapped(g: &Game, p: PlayerId, cd: CardId) -> bool {
    let d = g.db.get(cd);
    let t = &d.tags;
    if t.has(Tag::F) {
        return true;
    }
    if !matches!(&*d.name, "Plains" | "Island" | "Swamp" | "Mountain" | "Forest" | "Wastes")
        && cards_named(g, p, &["Thalia, Heretic Cathar", "Archon of Emeria"])
    {
        return true;
    }
    if let Some(rule) = d.enters_rule {
        return !untapped_by_rule(g, p, cd, rule); // check lands, fast and slow lands, battle lands, snarls ...
    }
    if t.has(Tag::T) {
        return true;
    }
    if t.has(Tag::Ck) {
        return g.player(p).lands.len() < 2;
    }
    false
}

fn untapped_by_rule(g: &Game, p: PlayerId, cd: CardId, rule: EnterRule) -> bool {
    let pl = g.player(p);
    let lt = |l: LandId| g.db.get(g.land(l).cd);
    match rule {
        EnterRule::Control(ts) => pl.lands.iter().any(|&l| lt(l).land_types.intersects(ts)),
        EnterRule::Fewer(n) => pl.lands.len() as u32 <= n,
        EnterRule::More(n) => pl.lands.len() as u32 >= n,
        EnterRule::Basics(n) => pl.lands.iter().filter(|&&l| lt(l).basic).count() as u32 >= n,
        EnterRule::Opponents(n) => g.opps(p).count() as u32 >= n,
        EnterRule::TypesMore(t, n) => pl.lands.iter().filter(|&&l| lt(l).land_types.intersects(t)).count() as u32 >= n,
        EnterRule::Reveal(ts) => {
            pl.hand.iter().any(|&c| c != cd && g.db.get(c).land && g.db.get(c).land_types.intersects(ts))
        }
        EnterRule::Legendary => pl.perms.iter().any(|&m| g.is_creature(m) && card_tag(g, m, Tag::Leg)),
        EnterRule::Pay(n) => pays_for_land(g, p, cd, n),
        EnterRule::Planeswalker => pl
            .perms
            .iter()
            .any(|&m| !g.perm(m).phased && g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER))),
    }
}

/// the AI: pay n life so land cd enters untapped? Yes when the extra mana casts something this turn (or it's an
/// early turn) and life is comfortable
pub fn pays_for_land(g: &Game, p: PlayerId, cd: CardId, n: i32) -> bool {
    let pl = g.player(p);
    if pl.life <= n + 10 {
        return false;
    }
    let mana_after = total_mana(g, p, false) + 1;
    let fits = pl.hand.iter().filter(|&&c| !g.db.get(c).land && c != cd).any(|&c| {
        let (gn, pips) = crate::engine::mana::cost_of(g, p, c);
        gn + pips.chars().count() as u32 == mana_after
    });
    fits || (pl.turns <= 3 && pl.life >= 30)
}

/// the colours p's lands make (for land choice: which new colours a land adds)
fn colors_have(g: &Game, p: PlayerId, except: Option<LandId>) -> crate::cards::Colors {
    g.player(p)
        .lands
        .iter()
        .filter(|&&l| Some(l) != except)
        .fold(crate::cards::Colors::NONE, |a, &l| a.union(land_colors_now(g, p, l)))
}

fn tag_cols(g: &Game, p: PlayerId, cd: CardId) -> crate::cards::Colors {
    match g.db.get(cd).tags.str(Tag::C).unwrap_or("") {
        "A" => g.player(p).ident,
        "C" | "" => crate::cards::Colors::NONE,
        c => crate::cards::Colors::from_letters(c),
    }
}

/// p plays its best land from hand (or the graveyard with Yawgmoth's Will / Muldrotha). Python's `play_land`.
pub fn play_land(g: &mut Game, p: PlayerId) -> Res {
    let pl = g.player(p);
    let mut lands: Vec<CardId> = pl.hand.iter().copied().filter(|&c| g.db.get(c).land).collect();
    if pl.yawg {
        // Yawgmoth's Will: lands from the graveyard too
        lands.extend(pl.gy.iter().copied().filter(|&c| g.db.get(c).land && pl.yawg_gy.contains(&c)));
    }
    let muld = cardcode::muldrotha_on(g, p);
    if muld {
        let more: Vec<CardId> = pl
            .gy
            .iter()
            .copied()
            .filter(|&c| g.db.get(c).land && !lands.contains(&c) && cardcode::muld_types(g, p, c))
            .collect();
        lands.extend(more); // Muldrotha
    }
    let chasm_card = |c: CardId| g.db.get(c).tag(Tag::Chasm);
    if lands.iter().any(|&c| chasm_card(c)) && !(g.player(p).lands.len() >= 4 && chasm_threatened(g, p) && !chasm(g, p))
    {
        lands.retain(|&c| !chasm_card(c)); // hold Glacial Chasm until it's needed
    }
    if lands.is_empty() {
        return Ok(());
    }
    let have = colors_have(g, p, None);
    // each land's score, plus a small random tie-break drawn in order as Python's max(lands, key=score) does
    let mut scored: Vec<(CardId, f64)> = vec![];
    for &c in &lands {
        let base = land_score(g, p, c, have);
        scored.push((c, base + g.rng.random() * 0.1));
    }
    let c = scored
        .iter()
        .copied()
        .fold(None, |b: Option<(CardId, f64)>, x| if b.is_none_or(|b| x.1 > b.1) { Some(x) } else { b })
        .unwrap()
        .0;
    let pl = g.player_mut(p);
    if let Some(i) = pl.hand.iter().position(|&x| x == c) {
        pl.hand.remove(i);
    } else {
        let from_yawg = pl.yawg && pl.yawg_gy.contains(&c);
        if muld && !from_yawg {
            cardcode::muld_mark(g, p, "L")?;
        }
        let gy = &mut g.player_mut(p).gy;
        let i = gy.iter().position(|&x| x == c).unwrap();
        gy.remove(i);
    }
    play_land_card(g, p, c, "plays")
}

fn land_score(g: &Game, p: PlayerId, c: CardId, have: crate::cards::Colors) -> f64 {
    let t = &g.db.get(c).tags;
    let life = g.player(p).life;
    let cols = tag_cols(g, p, c);
    let new_cols = "WUBRG".chars().filter(|&ch| cols.has(ch) && !have.has(ch)).count() as f64;
    let mut s = new_cols * 2.0 + if land_enters_tapped(g, p, c) { 0.0 } else { 3.0 };
    if t.has(Tag::Chasm) {
        s += 10.0;
    }
    let amt = t.int(Tag::Amt).unwrap_or(1).max(1);
    if amt > 1 && !t.has(Tag::Workshop) {
        s += 1.5 * (amt - 1) as f64
            - if life > 15 {
                0.0
            } else if life > 6 {
                1.5
            } else {
                6.0
            };
    }
    if t.has(Tag::Workshop) {
        let arts =
            g.player(p).hand.iter().filter(|&&x| g.db.get(x).types.has(Types::ARTIFACT) && !g.db.get(x).land).count();
        s += 2.0 * arts.min(2) as f64 - 2.0;
    }
    if &*g.db.get(c).name == "Gaea's Cradle" {
        s += g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count() as f64 - 1.0;
    }
    if t.has(Tag::Tabernacle) {
        let mine = g.player(p).perms.iter().filter(|&&m| g.is_creature(m)).count();
        let theirs =
            g.opps(p).map(|q| g.player(q).perms.iter().filter(|&&m| g.is_creature(m)).count()).max().unwrap_or(0);
        s += if theirs >= mine + 3 { 4.0 } else { -6.0 };
    }
    s
}

/// put land card c (already taken from its zone) onto the battlefield as p's land drop. Python's `play_land_card`.
pub fn play_land_card(g: &mut Game, p: PlayerId, c: CardId, how: &str) -> Res {
    let rule = g.db.get(c).enters_rule;
    let l = match rule {
        Some(EnterRule::Pay(n))
            if !(!g.hooks.is_empty() && cards_named(g, p, &["Thalia, Heretic Cathar", "Archon of Emeria"])) =>
        {
            // HUMAN(phase 9): the person chooses (hc.yes_no)
            let paid = pays_for_land(g, p, c, n);
            if paid {
                lose_life(g, p, n, Some(p), "other", None)?;
                crate::glog!(g, "  {} pays {} life for an untapped {}", g.player(p).name, n, g.db.get(c).name);
            }
            g.add_land(p, c, !paid)
        }
        _ => {
            let tapped = land_enters_tapped(g, p, c);
            g.add_land(p, c, tapped)
        }
    };
    let pl = g.player_mut(p);
    pl.land_turn = pl.turns as i32;
    cardcode::land_etb(g, p, l)?; // CI.LAND_ETB (Bojuka Bog ...)
    g.player_mut(p).lands_played += 1;
    crate::glog!(g, "  {} {} {}", g.player(p).name, how, g.db.get(c).name);
    let t = g.db.get(c).tags.clone();
    if t.has(Tag::Chasm) {
        // Glacial Chasm: when it enters, sacrifice a land
        g.player_mut(p).chasm_age = 0;
        let others: Vec<LandId> = g.player(p).lands.iter().copied().filter(|&x| g.land(x).cd != c).collect();
        let x = min_by(&others, |x| (!g.land(x).tapped) as i32 as f64 * 10.0 + land_colors_now(g, p, x).count() as f64)
            .unwrap_or(l);
        remove_land(g, p, x);
        let cd = g.land(x).cd;
        g.player_mut(p).gy.push(cd);
        crate::glog!(g, "    sacrifices {}", g.db.get(cd).name);
    }
    if t.has(Tag::Bounceland) {
        // Izzet Boilerworks returns another land
        let others: Vec<LandId> = g
            .player(p)
            .lands
            .iter()
            .copied()
            .filter(|&x| g.land(x).cd != c && !g.db.get(g.land(x).cd).tag(Tag::Bounceland))
            .collect();
        if let Some(x) = min_by(&others, |x| land_colors_now(g, p, x).count() as f64) {
            remove_land(g, p, x);
            let cd = g.land(x).cd;
            g.player_mut(p).hand.push(cd);
        }
    }
    landfall(g, p)?;
    if t.has(Tag::F) && g.player(p).lands.last() == Some(&l) {
        crack_fetch(g, p, l)?;
    }
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::LandPlay, Call::Cards { p, cards: vec![c] })?;
    }
    Ok(())
}

pub fn remove_land(g: &mut Game, p: PlayerId, l: LandId) {
    g.player_mut(p).lands.retain(|&x| x != l);
    g.land_mut(l).on_bf = false;
}

/// fetch lands' basic land types (Prismatic Vista fetches a basic)
fn true_fetch(name: &str) -> Option<&'static str> {
    Some(match name {
        "Polluted Delta" => "UB",
        "Flooded Strand" => "WU",
        "Bloodstained Mire" => "BR",
        "Wooded Foothills" => "RG",
        "Windswept Heath" => "GW",
        "Marsh Flats" => "WB",
        "Scalding Tarn" => "UR",
        "Verdant Catacombs" => "BG",
        "Arid Mesa" => "RW",
        "Misty Rainforest" => "GU",
        "Prismatic Vista" => "basic",
        _ => return None,
    })
}

/// pool games: a fetch land is sacrificed at once for a land (a second landfall, a land in the graveyard)
pub fn crack_fetch(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    let name = g.db.get(g.land(l).cd).name.to_string();
    let kinds = true_fetch(&name);
    let basics = ["Forest", "Island", "Plains", "Swamp", "Mountain", "Wastes"];
    let cands: Vec<CardId> = match kinds {
        Some(k) if k != "basic" => {
            let ts = crate::cards::Colors::from_letters(k);
            searchable(g, p)
                .into_iter()
                .filter(|&c| g.db.get(c).land && g.db.get(c).land_types.intersects(ts))
                .collect()
        }
        _ => {
            searchable(g, p).into_iter().filter(|&c| g.db.get(c).land && basics.contains(&&*g.db.get(c).name)).collect()
        }
    };
    if cands.is_empty() {
        return Ok(());
    }
    let have = colors_have(g, p, Some(l));
    let mut scored = vec![];
    for &c in &cands {
        let cols = tag_cols(g, p, c);
        let new_cols = "WUBRG".chars().filter(|&ch| cols.has(ch) && !have.has(ch)).count() as f64;
        let ncols = g.db.get(c).tags.str(Tag::C).map_or(0, |s| s.chars().count()) as f64;
        scored.push((c, new_cols * 1000.0 + ncols * 10.0 + g.rng.random()));
    }
    let c = scored
        .iter()
        .copied()
        .fold(None, |b: Option<(CardId, f64)>, x| if b.is_none_or(|b| x.1 > b.1) { Some(x) } else { b })
        .unwrap()
        .0;
    remove_land(g, p, l);
    let fetch = g.land(l).cd;
    g.player_mut(p).gy.push(fetch);
    let lib = &mut g.player_mut(p).library;
    let i = lib.iter().position(|&x| x == c).unwrap();
    lib.remove(i);
    crate::engine::tutors::shuffle_library(g, p);
    if let Some(a) = agent_for(g, p) {
        agent_take(g, a, p, c);
        if !g.hooks.is_empty() {
            fire_trigger(g, Event::LandGy, Call::Cards { p, cards: vec![fetch] })?;
        }
        return Ok(());
    }
    let tapped = kinds.is_none() && !(name == "Fabled Passage" && g.player(p).lands.len() >= 4);
    if kinds.is_some() {
        lose_life(g, p, 1, Some(p), "other", None)?;
    }
    let t = g.db.get(c).tag(Tag::T);
    g.add_land(p, c, tapped || t);
    crate::glog!(g, "    cracks {} for {}", name, g.db.get(c).name);
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::LandGy, Call::Cards { p, cards: vec![fetch] })?;
    }
    landfall(g, p)
}

/// Life from the Loam (dredge 3): mill three and return it instead of drawing, when lands are short
pub fn loam_dredge(g: &mut Game, p: PlayerId) -> Res<bool> {
    let pl = g.player(p);
    let Some(c) = pl.gy.iter().copied().find(|&x| g.db.get(x).tag(Tag::Loam)) else { return Ok(false) };
    if pl.library.len() < 25 || pl.hand.iter().filter(|&&x| g.db.get(x).land).count() >= 2 {
        return Ok(false);
    }
    mill(g, p, 3)?;
    let pl = g.player_mut(p);
    let i = pl.gy.iter().position(|&x| x == c).unwrap();
    pl.gy.remove(i);
    pl.hand.push(c);
    crate::glog!(g, "  {} dredges Life from the Loam", g.player(p).name);
    Ok(true)
}

/// cards p draws per land entering (Tatyova, Aesi ...), counting landfall copies
pub fn landfall_draws(g: &Game, p: PlayerId) -> i32 {
    let n = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| matches!(card_name(g, m), Some("Tatyova, Benthic Druid" | "Aesi, Tyrant of Gyre Strait")))
        .count() as i32;
    if n == 0 { 0 } else { n * (1 + crate::engine::hooks::total_trigger_copies(g, p, "landfall", None)) }
}

/// additional land drops (Exploration, Azusa ...) from hand, the top of the library (Oracle of Mul Daya) or the
/// graveyard (Crucible of Worlds, Ramunap Excavator)
pub fn more_lands(g: &mut Game, p: PlayerId) -> Res {
    let allowed = 1 + cardcode::extra_lands(g, p) + g.player(p).extra_land_now;
    let top_ok = cardcode::lands_from_top(g, p);
    let gy_ok = cardcode::lands_from_gy(g, p);
    let safe = 12 + 4 * landfall_draws(g, p) as usize; // optional land drops stop before they deck you
    while g.player(p).lands_played < allowed && !g.over && g.player(p).library.len() > safe {
        let pl = g.player(p);
        if top_ok && pl.library.last().is_some_and(|&c| g.db.get(c).land) {
            let c = g.player_mut(p).library.pop().unwrap();
            play_land_card(g, p, c, "plays from the top")?;
        } else if pl.hand.iter().any(|&c| g.db.get(c).land) {
            let n = pl.lands_played;
            play_land(g, p)?;
            if g.player(p).lands_played == n {
                break;
            }
        } else if gy_ok && pl.gy.iter().any(|&c| g.db.get(c).land) {
            let gl: Vec<CardId> = pl.gy.iter().copied().filter(|&c| g.db.get(c).land).collect();
            let c = max_by(&gl, |c| {
                let t = &g.db.get(c).tags;
                t.has(Tag::F) as i32 as f64 * 100.0 + t.str(Tag::C).map_or(0, |s| s.chars().count()) as f64
            })
            .unwrap();
            let gy = &mut g.player_mut(p).gy;
            let i = gy.iter().position(|&x| x == c).unwrap();
            gy.remove(i);
            play_land_card(g, p, c, "plays from the graveyard")?;
        } else {
            break;
        }
    }
    Ok(())
}

pub fn chasm_threatened(g: &Game, p: PlayerId) -> bool {
    g.opps(p).map(|q| board_power(g, q)).sum::<i32>() as f64 >= g.player(p).life as f64 * 0.5 || g.player(p).life <= 15
}

// ------------------------------------------------------------------ upkeep and end step
/// cards whose activated ability Disruptor Flute names stop
pub const ACTIVATED_ENGINES: [&str; 5] = [
    "Erebos, God of the Dead",
    "Vraska, Betrayal's Sting",
    "Ral Zarek, Guest Lecturer",
    "Nicol Bolas, Dragon-God",
    "Rhys the Redeemed",
];

/// p's upkeep. Python's `ais.upkeep`.
pub fn upkeep(g: &mut Game, p: PlayerId) -> Res {
    cardcode::turn_start(g, p)?;
    if g.player(p).ring_prot {
        g.player_mut(p).ring_prot = false; // The One Ring's protection ends as its controller's turn starts
        crate::glog!(g, "  {} no longer has protection from everything", g.player(p).name);
    }
    g.player_mut(p).life_locked = false; // Teferi's Protection
    if !g.player(p).suspended.is_empty() {
        cardcode::suspend_upkeep(g, p)?; // suspend (Profane Tutor)
    }
    let tabernacle = g
        .players
        .iter()
        .filter(|q| q.alive)
        .any(|q| q.lands.iter().any(|&l| g.db.get(g.land(l).cd).tag(Tag::Tabernacle)));
    if tabernacle {
        // The Tabernacle at Pendrell Vale: each creature is destroyed unless its controller pays {1}
        let mut cr: Vec<_> =
            g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
        cr.sort_by(|&a, &b| pval(g, b).total_cmp(&pval(g, a)));
        for m in cr {
            if pval(g, m) >= 0.5 && can_pay(g, p, 1, "", false) {
                pay(g, p, 1, "", false)?;
            } else {
                die(g, m, "destroy")?;
                g.player_mut(p).stat("tabernacle_lost", 1);
            }
        }
    }
    if chasm(g, p) {
        // Glacial Chasm: cumulative upkeep, pay 2 life per age counter
        g.player_mut(p).chasm_age += 1;
        let cost = 2 * g.player(p).chasm_age;
        if g.player(p).life - cost >= 10 && chasm_threatened(g, p) {
            lose_life(g, p, cost, Some(p), "other", None)?;
            crate::glog!(g, "  {} pays {} life for Glacial Chasm", g.player(p).name, cost);
        } else {
            let l = g.player(p).lands.iter().copied().find(|&l| g.db.get(g.land(l).cd).tag(Tag::Chasm)).unwrap();
            remove_land(g, p, l);
            let cd = g.land(l).cd;
            g.player_mut(p).gy.push(cd);
            g.player_mut(p).chasm_age = 0;
            crate::glog!(g, "  {} lets Glacial Chasm go", g.player(p).name);
        }
    }
    if !find(g, p, Tag::Panoptic).is_empty() {
        cardcode::mirror_upkeep(g, p)?;
    }
    if g.players.iter().any(|q| q.alive && has(g, q.id, Tag::Braids)) {
        cardcode::braids_sacrifice(g, p)?;
    }
    if g.over || !g.player(p).alive {
        return Ok(());
    }
    if g.dsl_on {
        crate::dsl::fire(g, "upkeep", crate::dsl::Fired { player: Some(p), ..Default::default() })?;
    }
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::Upkeep, Call::Player { p })?;
    }
    for m in g.player(p).perms.clone() {
        let x = g.perm(m);
        let Some(c) = x.cd else { continue };
        if x.phased || !x.on_bf || x.owner != p {
            continue;
        }
        let d = g.db.get(c);
        let t = d.tags.clone();
        let name = d.name.to_string();
        if ACTIVATED_ENGINES.contains(&&*name) && blocked(g, p, &name) {
            continue; // Disruptor Flute
        }
        if t.has(Tag::Eng) {
            if cardcode::replaced_tag_engine(&name) {
                continue;
            }
            if t.has(Tag::Remora) && g.perm(m).age > 4 {
                leave(g, m)?;
                g.player_mut(p).gy.push(c);
                continue;
            }
            if name == "Sylvan Library" && g.player(p).life <= 20 {
                continue;
            }
            if !trigger_window(g, p, Some(m), "draw a card", None)? {
                continue;
            }
            draw(g, p, 1, false)?;
            let life_cost = match &*name {
                "Phyrexian Arena" => 1,
                "Sylvan Library" => 4,
                "Call of the Ring" => 2,
                _ => 0,
            };
            if life_cost > 0 {
                lose_life(g, p, life_cost, Some(p), "other", None)?;
            }
            if t.has(Tag::Upkprolif) {
                if let Some(a) = army_of(g, p) {
                    g.perm_mut(a).plus += 1;
                }
                lose_life(g, p, 1, Some(p), "other", None)?;
            }
        }
        if t.has(Tag::Callring) {
            cardcode::ring_tempt(g, p)?; // Call of the Ring: the Ring tempts you
        }
        if t.has(Tag::Spider) {
            // the saga's chapters III and IV draw, then it's sacrificed
            let age = g.perm(m).age;
            if age == 1 {
                cardcode::spider_chapter2(g, p, m)?; // chapter II: prevent a creature's damage
            }
            if age == 2 || age == 3 {
                draw(g, p, 1, false)?;
            }
            if age >= 3 {
                leave(g, m)?;
                g.player_mut(p).gy.push(c);
                continue;
            }
        }
        if t.has(Tag::Bolas) && g.perm(m).age <= 5 {
            // +1 each turn: draw; each opponent loses a card
            draw(g, p, 1, false)?;
            random_discards(g, p)?;
        }
        if t.has(Tag::Pwdiscard) && g.perm(m).age <= 3 {
            random_discards(g, p)?; // Ral Zarek -1: each opponent discards
        }
        if t.has(Tag::Mycoloth) && g.perm(m).plus > 0 && trigger_window(g, p, Some(m), "Saprolings", None)? {
            let n = g.perm(m).plus as u32;
            make_tokens(g, p, Tokens { color: Some(crate::cards::Colors::from_letters("G")), ..Tokens::new(n, 1) })?;
        }
        if let Some(n) = t.int(Tag::Tokup)
            && trigger_window(g, p, Some(m), "tokens", None)?
        {
            make_tokens(g, p, Tokens { warrior: t.has(Tag::Warrior), ..Tokens::new(n.max(0) as u32, 1) })?;
        }
        if t.has(Tag::SheoW) && trigger_window(g, p, Some(m), "return a creature; each opponent sacrifices", Some(6.0))?
        {
            // HUMAN(phase 9): hc.pick_cards
            let cr: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&x| g.db.get(x).creature).collect();
            if let Some(b) = max_by(&cr, |x| ai::seph_bval(g, p, x)) {
                let gy = &mut g.player_mut(p).gy;
                let i = gy.iter().position(|&x| x == b).unwrap();
                gy.remove(i);
                let was = g.player(p).removed_bombs.contains(&b);
                enter(g, p, b, Enter::default())?;
                ai::note_bomb(g, p, b, was);
            }
            for q in g.opps(p).collect::<Vec<_>>() {
                edict(g, q, false)?;
            }
        }
    }
    check_state(g)
}

fn random_discards(g: &mut Game, p: PlayerId) -> Res {
    for q in g.opps(p).collect::<Vec<_>>() {
        let n = g.player(q).hand.len() as u64;
        if n > 0 {
            let i = g.rng.below(n) as usize;
            discard_index(g, q, i)?;
        }
    }
    Ok(())
}

/// Erebos: {1}{B}, pay 2 life: draw a card. Only with spare mana and a comfortable life total.
pub fn erebos_draw(g: &mut Game, p: PlayerId) -> Res<bool> {
    if !has(g, p, Tag::Erebos) || blocked(g, p, "Erebos, God of the Dead") {
        return Ok(false);
    }
    let st = g.turn_stamp();
    if g.player(p).life < 16 || g.player(p).erebos_t == Some(st) || !can_pay(g, p, 1, "B", false) {
        return Ok(false);
    }
    pay(g, p, 1, "B", false)?;
    lose_life(g, p, 2, Some(p), "other", None)?;
    draw(g, p, 1, false)?;
    g.player_mut(p).erebos_t = Some(st);
    g.player_mut(p).stat("erebos_draws", 1);
    Ok(true)
}

/// Yawgmoth's Will: cards that reached your graveyard this turn were exiled instead
pub fn yawg_cleanup(g: &mut Game, p: PlayerId) {
    let pl = g.player_mut(p);
    let mut left = pl.yawg_gy.clone();
    let (mut keep, mut new) = (vec![], vec![]);
    for &c in &pl.gy {
        if let Some(i) = left.iter().position(|&x| x == c) {
            left.remove(i);
            keep.push(c);
        } else {
            new.push(c);
        }
    }
    if !new.is_empty() {
        pl.gy = keep;
        pl.exile.extend(new);
    }
}

/// p's end step. Python's `ais.end_step`.
pub fn end_step(g: &mut Game, p: PlayerId) -> Res {
    cardcode::necro_deliver(g, p)?;
    cardcode::delayed_end_step(g, p)?; // Marchesa, The Eternal Wanderer, Eerie Interlude, Memory Jar
    if g.player(p).yawg {
        yawg_cleanup(g, p);
    }
    if g.dsl_on {
        crate::dsl::fire(g, "end_step", crate::dsl::Fired { player: Some(p), ..Default::default() })?;
    }
    for m in std::mem::take(&mut g.player_mut(p).borrowed) {
        // Zealous Conscripts: control returns
        let orig = g.perm(m).orig;
        if g.perm(m).on_bf && g.perm(m).owner == p && g.player(orig).alive {
            g.player_mut(p).perms.retain(|&x| x != m);
            g.perm_mut(m).owner = orig;
            g.player_mut(orig).perms.push(m);
            g.bf_ver += 1;
            if g.perm_mut(m).data.remove(crate::state::DataKey::TapOnReturn).is_some_and(|v| v.truthy()) {
                g.perm_mut(m).tapped = true; // Ray of Command
            }
        }
    }
    if g.monarch == Some(p) {
        draw(g, p, 1, false)?; // the monarch draws
    }
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::EndStep, Call::Player { p })?;
    }
    for m in find(g, p, Tag::Breach) {
        // Underworld Breach: sacrifice it at the beginning of the end step
        if trigger_window(g, p, Some(m), "sacrifice it", None)? && g.perm(m).on_bf {
            die(g, m, "sac")?;
        }
    }
    let turns = g.player(p).turns;
    let il = std::mem::take(&mut g.player_mut(p).impulse_long);
    let mut keep = vec![];
    for (c, t) in il {
        // Prosper / Reckless Impulse: until the end of your next turn
        let pl = g.player_mut(p);
        if pl.hand.contains(&c) && t <= turns {
            let i = pl.hand.iter().position(|&x| x == c).unwrap();
            pl.hand.remove(i);
            pl.exile.push(c);
        } else if pl.hand.contains(&c) {
            keep.push((c, t));
        }
    }
    g.player_mut(p).impulse_long = keep;
    for c in std::mem::take(&mut g.player_mut(p).impulse) {
        // Jeska's Will: unplayed exiled cards stay in exile
        let pl = g.player_mut(p);
        if let Some(i) = pl.hand.iter().position(|&x| x == c) {
            pl.hand.remove(i);
            pl.exile.push(c);
        }
    }
    if has(g, p, Tag::Necro) && g.humans.get(p).is_none() {
        cardcode::necro_pay(g, p)?; // the person pays with the ability
    }
    erebos_draw(g, p)?; // leftover mana at your own end step
    for m in g.player(p).perms.clone() {
        if g.perm(m).temp {
            leave(g, m)?;
        }
    }
    if has(g, p, Tag::Pvprolif) {
        // Atraxa, Praetors' Voice
        let src = find(g, p, Tag::Pvprolif)[0];
        if trigger_window(g, p, Some(src), "proliferate", None)? {
            for m in g.player(p).perms.clone() {
                if g.perm(m).plus > 0 {
                    g.perm_mut(m).plus += 1;
                }
            }
        }
    }
    // HUMAN(phase 9): a person discards to hand size themselves (hc.discard_to_hand_size)
    let nomax = |g: &Game| {
        has(g, p, Tag::Nomax)
            || g.player(p).lands.iter().any(|&l| g.db.get(g.land(l).cd).tag(Tag::Nomax))
            || g.player(p).nomax_turn == Some(g.player(p).turns)
    };
    let held = |g: &Game| -> Vec<CardId> {
        let pl = g.player(p);
        pl.hand.iter().copied().filter(|c| !pl.stolen.contains_key(c) && !pl.agent_ids.contains(c)).collect()
    };
    while held(g).len() > 7 && !nomax(g) {
        if g.player(p).key == "seph" {
            let bombs: Vec<CardId> =
                g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).creature && g.db.get(c).bomb >= 6).collect();
            if let Some(c) = max_by(&bombs, |c| g.db.get(c).bomb as f64) {
                discard_cards(g, p, &[c])?;
                continue;
            }
        }
        g.tick()?; // each discard can set off triggers (Tergrid): count it
        let h = held(g);
        let lands: Vec<CardId> = h.iter().copied().filter(|&c| g.db.get(c).land).collect();
        let c = if lands.len() >= 2 { lands[0] } else { max_by(&h, |c| g.db.get(c).cmc as f64).unwrap() };
        discard_cards(g, p, &[c])?;
    }
    g.player_mut(p).last_turn_end = g.enter_no; // Premature Burial: what entered since this turn ended
    if g.player(p).key == "seph" {
        ai::seph_end_milestones(g, p);
    }
    Ok(())
}

// ------------------------------------------------------------------ the turn
pub const STEPS: [Step; 5] = [Step::Start, Step::Main1, Step::Combat, Step::Main2, Step::End];

pub fn take_turn(g: &mut Game, p: PlayerId) -> Res {
    continue_turn(g, p, Step::Start)
}

/// play p's turn from `step` on (a copied game resumes mid-turn from here)
pub fn continue_turn(g: &mut Game, p: PlayerId, step: Step) -> Res {
    let i = STEPS.iter().position(|&s| s == step).unwrap();
    for &st in &STEPS[i..] {
        g.step = st;
        // PORT(phase 9): practice mode empties the mana pools between steps
        match st {
            Step::Start => step_start(g, p)?,
            Step::Main1 => step_main1(g, p)?,
            Step::Combat => {
                combat(g, p)?;
                g.player_mut(p).pump = 0;
            }
            Step::Main2 => step_main2(g, p)?,
            Step::End => step_end(g, p)?,
        }
        if g.over || !g.player(p).alive {
            return Ok(());
        }
    }
    Ok(())
}

/// the start of p's turn: untap, upkeep, draw, the land drop (Python's `_step_start`)
pub fn step_start(g: &mut Game, p: PlayerId) -> Res {
    g.active = Some(p);
    for m in &mut g.perms {
        m.eot_pt = (0, 0);
        m.eot_kw.clear();
    }
    for q in &mut g.players {
        q.gy_start = q.gy.clone(); // what was there before this turn
        q.floating.any = 0;
        q.floating.r = 0; // floating mana empties between turns
        q.floating.u = 0;
        q.floating.c = 0;
        q.floating.g = 0;
        q.floating.b = 0;
    }
    let pl = g.player_mut(p);
    pl.extra_combats = 0;
    pl.combat_no = 0;
    pl.turns += 1;
    for l in g.player(p).lands.clone() {
        g.land_mut(l).tapped = false;
    }
    let crack = cardcode::crackdown_on(g); // Crackdown: big nonwhite creatures stay tapped
    for m in g.player(p).perms.clone() {
        let x = g.perm(m);
        if x.data.int(crate::state::DataKey::Frozen) > 0 {
            // Frost Titan, Tamiyo
            let f = x.data.int(crate::state::DataKey::Frozen);
            g.perm_mut(m).data.set(crate::state::DataKey::Frozen, crate::state::Val::Int(f - 1));
            continue;
        }
        if crack && x.tapped && cardcode::crackdown_holds(g, m) {
            let y = g.perm_mut(m);
            y.sick = false;
            y.phased = false;
            y.age += 1;
            continue;
        }
        let nountap = card_tag(g, m, Tag::Nountap) || (!g.auras.is_empty() && cardcode::locked(g, m, "frozen"));
        let y = g.perm_mut(m);
        if !nountap {
            y.tapped = false; // Grim Monolith, Mana Vault; Encrust
        }
        y.sick = false;
        y.phased = false;
        y.age += 1;
    }
    let pl = g.player_mut(p);
    pl.spells_this_turn = 0;
    pl.yawg = false;
    pl.pump = 0;
    pl.pumpadd = 0;
    pl.trample = false;
    pl.combo_tried = false;
    pl.haste_all = false;
    crate::glog!(
        g,
        "--- {} turn {}: life {}, {} cards in hand, {} lands",
        g.player(p).name,
        g.player(p).turns,
        g.player(p).life,
        g.player(p).hand.len(),
        g.player(p).lands.len()
    );
    upkeep(g, p)?;
    if g.over || !g.player(p).alive {
        return Ok(());
    }
    step_priority(g, "upkeep", None, &[])?;
    if g.over || !g.player(p).alive {
        return Ok(());
    }
    for m in find(g, p, Tag::Vaultping) {
        // Mana Vault: at the beginning of your draw step, 1 damage if tapped
        if g.perm(m).tapped {
            lose_life(g, p, 1, Some(p), "other", Some(true))?;
        }
    }
    if has(g, p, Tag::Necro) {
        // Necropotence: skip your draw step
    } else if loam_dredge(g, p)? || cardcode::dakmor_dredge(g, p)? || cardcode::skip_draw(g, p) {
        // Life from the Loam, Dakmor Salvage dredged; Solitary Confinement
    } else if !(g.player(p).key == "seph" && ai::seph_dredge(g, p)?) {
        // HUMAN(phase 9): a person may dredge (play/cards.dredge)
        draw(g, p, 1, true)?;
    }
    check_state(g)?;
    if g.over || !g.player(p).alive {
        return Ok(());
    }
    step_priority(g, "draw", None, &[])?;
    if g.over || !g.player(p).alive {
        return Ok(());
    }
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::Main1, Call::Player { p })?; // at the beginning of your precombat main phase (Sagas)
    }
    if g.over || !g.player(p).alive {
        return Ok(());
    }
    let nl = g.player(p).lands.len();
    let pl = g.player_mut(p);
    pl.lands_played = 0;
    pl.extra_land_now = 0;
    // HUMAN(phase 9): a person plays lands in the main phase
    play_land(g, p)?;
    if !g.hooks.is_empty() {
        more_lands(g, p)?;
    }
    let pl = g.player_mut(p);
    if pl.lands.len() == nl && pl.turns <= 5 {
        pl.stat("land_miss", 1);
    }
    if pl.turns == 4 || pl.turns == 6 {
        let mana = total_mana(g, p, false) as i64;
        let pl = g.player_mut(p);
        let t = pl.turns;
        pl.stats.insert(crate::sym::intern(&format!("mana_T{t}")), mana);
        pl.stats.insert(crate::sym::intern(&format!("had_T{t}")), 1);
    }
    Ok(())
}

fn step_main1(g: &mut Game, p: PlayerId) -> Res {
    // HUMAN(phase 9): a person's main phase (play/human.human_main)
    ai::main(g, p, false)?;
    if g.over || !g.player(p).alive {
        return Ok(());
    }
    if !g.hooks.is_empty() {
        more_lands(g, p)?;
    }
    Ok(())
}

fn step_main2(g: &mut Game, p: PlayerId) -> Res {
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::Main2, Call::Player { p })?; // at the beginning of your postcombat main phase (Tymna)
    }
    if g.over || !g.player(p).alive {
        return Ok(());
    }
    ai::main(g, p, true)?;
    if g.player(p).key == "veyran" && has(g, p, Tag::Veyran) && ai::engine_payoff(g, p) {
        let t = g.player(p).turns;
        g.player_mut(p).milestone.entry("engine").or_insert(t);
    }
    Ok(())
}

fn step_end(g: &mut Game, p: PlayerId) -> Res {
    end_step(g, p)?;
    check_state(g)?;
    if !g.over && g.player(p).alive {
        step_priority(g, "end", None, &[])?;
    }
    Ok(())
}

// ------------------------------------------------------------------ a game
/// Python's `mulligan`: keep 2 to 5 lands; after two mulligans, bottom one or two cards.
pub fn mulligan(g: &mut Game, p: PlayerId, rng: &mut crate::rng::Rng) {
    let draw7 = |g: &mut Game, rng: &mut crate::rng::Rng| {
        let pl = g.player_mut(p);
        let hand = std::mem::take(&mut pl.hand);
        pl.library.extend(hand);
        rng.shuffle(&mut pl.library);
        for _ in 0..7 {
            let c = pl.library.pop().unwrap();
            pl.hand.push(c);
        }
    };
    let ok = |g: &Game| {
        let n = g.player(p).hand.iter().filter(|&&c| g.db.get(c).land).count();
        (2..=5).contains(&n)
    };
    draw7(g, rng);
    if ok(g) {
        return;
    }
    draw7(g, rng);
    if ok(g) {
        return;
    }
    for bottom in [1, 2] {
        draw7(g, rng);
        if ok(g) || bottom == 2 {
            for _ in 0..bottom {
                let hand = g.player(p).hand.clone();
                let lands: Vec<CardId> = hand.iter().copied().filter(|&c| g.db.get(c).land).collect();
                let spells: Vec<CardId> = hand.iter().copied().filter(|&c| !g.db.get(c).land).collect();
                let c = if lands.len() > 4 || spells.is_empty() {
                    lands[0]
                } else {
                    max_by(&spells, |c| g.db.get(c).cmc as f64).unwrap()
                };
                let pl = g.player_mut(p);
                let i = pl.hand.iter().position(|&x| x == c).unwrap();
                pl.hand.remove(i);
                pl.library.insert(0, c);
            }
            g.player_mut(p).stats.insert("mulls", bottom + 1);
            return;
        }
    }
}

/// a game seated in this order, shuffled and mulliganed, before turn one. Each seat shuffles from its own stream
/// (`lib:{seed}:{deck}`) and play decisions use `play:{seed}`, so changing one deck's list leaves every other seat's
/// library and hand the same. Python's `setup_pool_game`.
pub fn setup_game(
    db: std::sync::Arc<crate::cards::CardDb>,
    registry: std::sync::Arc<crate::hooks::Registry>,
    settings: std::sync::Arc<crate::settings::Settings>,
    seed: u64,
    seats: &[Seat],
    trace: bool,
) -> Game {
    let players = seats
        .iter()
        .enumerate()
        .map(|(i, s)| {
            crate::state::Player::new(PlayerId(i as u8), s.key, s.name, s.ident, s.commander, s.cards.clone())
        })
        .collect();
    let mut g = Game::new(db, registry, settings, players, crate::rng::Rng::named(&format!("play:{seed}")));
    if trace {
        let names: Vec<&str> = g.players.iter().map(|q| q.name).collect();
        g.log = Some(vec![format!("Seat order: {}", names.join(", "))]);
    }
    for i in 0..g.players.len() {
        let p = PlayerId(i as u8);
        let mut r = crate::rng::Rng::named(&format!("lib:{seed}:{}", g.player(p).key));
        let mut lib = std::mem::take(&mut g.player_mut(p).library);
        r.shuffle(&mut lib);
        g.player_mut(p).library = lib;
        mulligan(&mut g, p, &mut r);
        let hand = g.player(p).hand.clone();
        g.player_mut(p).seen.extend(hand);
    }
    g
}

/// One seat: the deck key, its display name, the commander and the full list (commander included).
#[derive(Debug, Clone)]
pub struct Seat {
    pub key: Sym,
    pub name: Sym,
    pub commander: CardId,
    /// the colour identity (Python's IDENT / SEATS): Command Tower, Arcane Signet and Chromatic Lantern make these
    pub ident: crate::cards::Colors,
    pub cards: std::sync::Arc<[CardId]>,
}

/// play rounds until someone wins or max_rounds pass; a game still going then is won by the strongest player
/// (life + 2 x board power). Python's `_run_rounds`.
pub fn run_rounds(g: &mut Game, max_rounds: u32) {
    let r = (|| -> Res {
        for r in 1..=max_rounds {
            g.round = r;
            for i in 0..g.players.len() {
                let p = PlayerId(i as u8);
                if g.player(p).alive && !g.over && g.player(p).skip_turns > 0 {
                    g.player_mut(p).skip_turns -= 1;
                    crate::glog!(g, "  {} skips a turn", g.player(p).name);
                    continue;
                }
                if g.player(p).alive && !g.over {
                    if r > 1 {
                        ai::end_of_turn_window(g, p)?; // HUMAN(phase 9): not for a person's seat
                    }
                    if g.player(p).alive && !g.over {
                        take_turn(g, p)?;
                    }
                }
            }
            if g.over {
                break;
            }
        }
        Ok(())
    })();
    match r {
        Ok(()) => {}
        Err(Stop::OutOfWork) => {
            // a runaway loop: the game ends as a timeout
            g.stopped = true;
        }
        Err(Stop::Probe(pr)) => panic!("a trigger probe escaped to the turn loop: {pr:?}"),
    }
    if !g.over {
        let alive: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
        g.winner = max_by(&alive, |q| (g.player(q).life + board_power(g, q) * 2) as f64);
        g.wintype = Some("timeout");
    }
}
