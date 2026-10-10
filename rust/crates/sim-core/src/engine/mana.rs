//! Mana: what a player can tap, the payment planner, paying, and what a card costs (engine.py's "mana" section).
//!
//! A player's mana is a list of `Unit`s (Python's `[source, colours, amount]` lists): each untapped land and mana
//! permanent, each Treasure, each floating mana. `plan_pay` assigns units to a cost, coloured pips first (the
//! scarcest colour first), then generic mana (Treasures last, painful sources late, without wasting a big source on
//! a small cost, keeping the scarcest colours for later), as Python's does.

use crate::cardcode;
use crate::cards::Colors;
use crate::engine::hooks;
use crate::engine::life::lose_life;
use crate::engine::values::has;
use crate::flow::Res;
use crate::hooks::{Call, Event};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::tag::Tag;

/// Where a unit of mana comes from (Python's first element of a unit).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Land(LandId),
    Perm(PermId),
    /// a Treasure token ('T')
    Treasure,
    /// floating mana: red (Birgi), any colour (rituals), blue, colourless, green, black
    FloatR,
    FloatAny,
    FloatU,
    FloatC,
    FloatG,
    FloatB,
    /// an Eldrazi Scion or Spawn: sacrifice for {C} ('SC')
    Scion(PermId),
    /// a card in hand that makes mana when exiled (Elvish Spirit Guide: 'H')
    HandCard(CardId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unit {
    pub src: Source,
    /// the colours it can make; empty for colourless
    pub cols: Colors,
    pub amt: u32,
}

pub const BASIC_NAMES: [&str; 11] = [
    "Plains",
    "Island",
    "Swamp",
    "Mountain",
    "Forest",
    "Wastes",
    "Snow-Covered Plains",
    "Snow-Covered Island",
    "Snow-Covered Swamp",
    "Snow-Covered Mountain",
    "Snow-Covered Forest",
];

/// colours from a tag value: 'A' is the player's whole identity, 'C' colourless
fn tag_colors(v: &str, ident: Colors) -> Colors {
    match v {
        "A" => ident,
        "C" => Colors::NONE,
        c => Colors::from_letters(c),
    }
}

/// what land L taps for, the board-wide checks (Blood Moon, Dryad of the Ilysian Grove) already made
fn land_cols(g: &Game, p: PlayerId, l: LandId, anyc: bool, moon: bool, dryad: bool) -> Colors {
    let d = g.db.get(g.land(l).cd);
    let ident = g.player(p).ident;
    if moon && !BASIC_NAMES.contains(&&*d.name) {
        return Colors::from_letters("R");
    }
    if anyc || dryad {
        return ident;
    }
    if let Some(c) = cardcode::land_colors(g, p, l) {
        return c; // Plaza of Heroes, Unclaimed Territory, Vivid lands ...
    }
    tag_colors(d.tags.str(Tag::C).unwrap_or("C"), ident)
}

/// the colours p's land L taps for now (Fellwar Stone reads opponents' lands)
pub fn land_colors_now(g: &Game, p: PlayerId, l: LandId) -> Colors {
    let moon = !g.hooks.is_empty() && cardcode::blood_moon(g);
    let dryad = !g.hooks.is_empty() && cardcode::dryad_colors(g, p);
    land_cols(g, p, l, false, moon, dryad)
}

/// every mana unit p can use now (Python's `mana_units`)
pub fn mana_units(g: &Game, p: PlayerId, convoke: bool) -> Vec<Unit> {
    let pl = g.player(p);
    let ident = pl.ident;
    let mut u = vec![];
    let anyc = has(g, p, Tag::Lantern)
        || (pl.lands.iter().any(|&l| g.db.get(g.land(l).cd).tag(Tag::Worldtree)) && pl.lands.len() >= 6);
    let art = g.pay_for.is_some_and(|c| g.db.get(c).types.has(crate::cards::Types::ARTIFACT));
    let hooked = !g.hooks.is_empty();
    let moon = hooked && cardcode::blood_moon(g);
    let dryad = hooked && cardcode::dryad_colors(g, p);
    for &l in &pl.lands {
        let land = g.land(l);
        if land.tapped {
            continue;
        }
        let t = &g.db.get(land.cd).tags;
        if t.has(Tag::Workshop) && !art {
            continue; // Mishra's Workshop: artifact spells only
        }
        if t.has(Tag::Tomb) && pl.life <= 2 {
            continue; // Ancient Tomb's 2 damage would kill you
        }
        let mut amt = t.int(Tag::Amt).unwrap_or(1);
        if let Some(n) = cardcode::dyn_mana_land(g, p, l) {
            amt = n as i32;
        }
        if hooked {
            amt += hooks::total_land_mana(g, p, l);
        }
        u.push(Unit { src: Source::Land(l), cols: land_cols(g, p, l, anyc, moon, dryad), amt: amt.max(0) as u32 });
    }
    let rite = has(g, p, Tag::Rite);
    let no_art_mana = hooked && hooks::total_count(g, Event::NoArtifactMana, p) != 0;
    for &m in &pl.perms {
        let x = g.perm(m);
        if x.tapped || x.phased {
            continue;
        }
        if !g.auras.is_empty() && cardcode::locked(g, m, "noact") {
            continue; // Arrest, Encrust: no mana abilities either
        }
        let Some(c) = x.cd else {
            if rite && !x.sick {
                u.push(Unit { src: Source::Perm(m), cols: ident, amt: 1 });
            } else if let Some(Val::Str(cols)) = x.data.get(DataKey::Manatok).filter(|_| !x.sick) {
                u.push(Unit { src: Source::Perm(m), cols: Colors::from_letters(cols), amt: 1 });
            } else if x.ttypes.contains(&"scion") || x.ttypes.contains(&"spawn") {
                u.push(Unit { src: Source::Scion(m), cols: Colors::NONE, amt: 1 });
            }
            continue;
        };
        let d = g.db.get(c);
        let t = &d.tags;
        if t.has(Tag::Chromemox) {
            // Chrome Mox: one mana of the imprinted card's colours (none if nothing imprinted)
            if !x.colors.is_empty() {
                u.push(Unit { src: Source::Perm(m), cols: x.colors, amt: 1 });
            }
            continue;
        }
        if let Some(rock) = t.str(Tag::Rock) {
            if no_art_mana && d.types.has(crate::cards::Types::ARTIFACT) {
                continue;
            }
            if t.has(Tag::Pstone) && !art {
                continue; // Powerstone: artifact spells and abilities only
            }
            let (a, col) = rock.split_once(':').unwrap_or((rock, "C"));
            let amt = cardcode::dyn_mana_perm(g, p, m).unwrap_or_else(|| a.parse().unwrap_or(1));
            let mut cols = tag_colors(col, ident);
            if &*d.name == "Fellwar Stone" {
                cols = Colors::NONE;
                for q in g.opps(p) {
                    for &l in &g.player(q).lands {
                        cols = cols.union(land_colors_now(g, q, l));
                    }
                }
            }
            if amt > 0 {
                u.push(Unit { src: Source::Perm(m), cols, amt });
            }
        } else if let Some(dork) = t.str(Tag::Dork).filter(|_| !x.sick) {
            if &*d.name == "Hydro-Channeler" && !g.pay_for.is_some_and(|c| g.db.get(c).instant || g.db.get(c).sorcery) {
                continue;
            }
            let amt = cardcode::dyn_mana_perm(g, p, m).unwrap_or(1);
            let cols = if &*d.name == "Delighted Halfling" {
                cardcode::halfling_colors(g, p).unwrap_or(Colors::NONE)
            } else {
                tag_colors(dork, ident)
            };
            if amt > 0 {
                u.push(Unit { src: Source::Perm(m), cols, amt });
            }
        } else if rite && d.creature && !x.sick && x.noatk {
            u.push(Unit { src: Source::Perm(m), cols: ident, amt: 1 });
        }
    }
    let tre = if no_art_mana { 0 } else { pl.treasures };
    let bonus = if hooked { hooks::total_count(g, Event::TreasureBonus, p) } else { 0 };
    for _ in 0..tre {
        u.push(Unit { src: Source::Treasure, cols: ident, amt: (1 + bonus).max(0) as u32 });
    }
    if hooked {
        u = adjust_mana(g, p, u);
    }
    let f = pl.floating;
    let float = |n: u32, src: Source, cols: Colors| (0..n).map(move |_| Unit { src, cols, amt: 1 });
    u.extend(float(f.r, Source::FloatR, Colors::from_letters("R")));
    u.extend(float(f.any, Source::FloatAny, ident));
    u.extend(float(f.u, Source::FloatU, Colors::from_letters("U")));
    u.extend(float(f.c, Source::FloatC, Colors::NONE));
    u.extend(float(f.g, Source::FloatG, Colors::from_letters("G")));
    u.extend(float(f.b, Source::FloatB, Colors::from_letters("B")));
    if convoke {
        // each untapped creature pays for {1} or one coloured pip
        for &m in &pl.perms {
            let x = g.perm(m);
            if g.is_creature(m) && !x.tapped && !x.phased && !u.iter().any(|w| w.src == Source::Perm(m)) {
                u.push(Unit { src: Source::Perm(m), cols: ident, amt: 1 });
            }
        }
    }
    u
}

/// mana locks and bonuses from card code (Karn + Lattice, Collector Ouphe, Cursed Totem, Kinnan, Urza), and
/// Elvish Spirit Guide in hand. Python's `CI.adjust_mana`.
fn adjust_mana(g: &Game, p: PlayerId, mut u: Vec<Unit>) -> Vec<Unit> {
    if hooks::total_count(g, Event::ManaLock, p) != 0 {
        return vec![];
    }
    if hooks::total_count(g, Event::NoCreatureMana, p) != 0 {
        u.retain(|w| !matches!(w.src, Source::Perm(m) if g.is_creature(m)));
    }
    let bonus = hooks::total_count(g, Event::NonlandManaBonus, p);
    if bonus != 0 {
        for w in &mut u {
            if matches!(w.src, Source::Perm(_) | Source::Treasure) {
                w.amt = (w.amt as i32 + bonus).max(0) as u32;
            }
        }
    }
    for (src, imp) in hooks::hooked(g, Event::ExtraMana) {
        if g.perm(src).owner == p {
            let extra = (imp.extra_mana.unwrap())(g, src, p, &u);
            u.extend(extra);
        }
    }
    u.extend(cardcode::hand_mana(g, p)); // Elvish Spirit Guide
    u
}

fn is_pain(g: &Game, src: Source) -> bool {
    match src {
        Source::Land(l) => {
            let t = &g.db.get(g.land(l).cd).tags;
            t.has(Tag::Tomb) || t.has(Tag::Pain)
        }
        Source::Perm(m) => g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Pain)),
        _ => false,
    }
}

const WUBRG: [char; 5] = ['W', 'U', 'B', 'R', 'G'];

/// How many of each unit pay `generic` + `pips` (None if they can't), and which coloured pips each paid for.
/// Python's `plan_pay`, with the same tie-breaking (the first best unit wins).
pub fn plan_pay(g: &Game, u: &[Unit], generic: u32, pips: &str) -> Option<(Vec<u32>, Vec<String>)> {
    let mut rem: Vec<u32> = u.iter().map(|w| w.amt).collect();
    let mut used = vec![0u32; u.len()];
    let mut col = vec![String::new(); u.len()];
    let mut order: Vec<char> = pips.chars().collect();
    order.sort_by_key(|&c| (0..u.len()).filter(|&i| u[i].cols.has(c)).map(|i| rem[i]).sum::<u32>()); // stable
    for c in order {
        let mut best: Option<(usize, (bool, u32))> = None;
        for (i, w) in u.iter().enumerate() {
            if rem[i] > 0 && w.cols.has(c) {
                let k = (w.src == Source::Treasure, w.cols.count());
                if best.is_none_or(|b| k < b.1) {
                    best = Some((i, k));
                }
            }
        }
        let (i, _) = best?;
        rem[i] -= 1;
        used[i] += 1;
        col[i].push(c);
    }
    let mut need = generic;
    while need > 0 {
        let avail: Vec<(char, u32)> = WUBRG
            .iter()
            .map(|&c| (c, (0..u.len()).filter(|&i| u[i].cols.has(c)).map(|i| rem[i]).sum::<u32>()))
            .collect();
        type Key = (bool, bool, u32, u32, i64); // Treasure last, pain late, waste, colours, scarcity
        let mut best: Option<(usize, Key)> = None;
        for (i, w) in u.iter().enumerate() {
            if rem[i] == 0 {
                continue;
            }
            let waste = if used[i] > 0 { 0 } else { rem[i].saturating_sub(need) };
            let scarce = avail.iter().filter(|(c, _)| w.cols.has(*c)).map(|x| x.1).min().unwrap_or(0);
            let k = (w.src == Source::Treasure, is_pain(g, w.src), waste, w.cols.count(), -(scarce as i64));
            if best.is_none_or(|b| k < b.1) {
                best = Some((i, k));
            }
        }
        let (i, _) = best?;
        let take = rem[i].min(need);
        rem[i] -= take;
        used[i] += take;
        need -= take;
    }
    Some((used, col))
}

pub fn can_pay(g: &Game, p: PlayerId, generic: u32, pips: &str, convoke: bool) -> bool {
    // PORT(phase 9): practice mode pays from the person's floating mana pool (play/mana.pool_can_pay)
    plan_pay(g, &mana_units(g, p, convoke), generic, pips).is_some()
}

/// p pays the cost; false (nothing paid) if it can't
pub fn pay(g: &mut Game, p: PlayerId, generic: u32, pips: &str, convoke: bool) -> Res<bool> {
    // PORT(phase 9): practice mode pays from the person's pool (play/mana.pay_from_pool)
    let u = mana_units(g, p, convoke);
    let Some((used, col)) = plan_pay(g, &u, generic, pips) else { return Ok(false) };
    for (i, w) in u.iter().enumerate() {
        if used[i] > 0 {
            spend_unit(g, p, *w, used[i], &col[i])?;
        }
    }
    Ok(true)
}

/// Use unit w for n mana; col: the coloured pips it paid for (painlands hurt only then). Taps or sacrifices the
/// source and runs its side effects: Treasures, floating mana, on-tap triggers, pain. Python's `spend_unit`.
pub fn spend_unit(g: &mut Game, p: PlayerId, w: Unit, n: u32, col: &str) -> Res {
    match w.src {
        Source::Treasure => {
            let st = g.turn_stamp();
            let pl = g.player_mut(p);
            // a trigger during the payment (Gilded Goose's Food sacrificed: Trail of Crumbs pays {1}) may have spent
            // the Treasure this plan counted on: Python's count goes negative, an unsigned one stays at 0
            pl.treasures = pl.treasures.saturating_sub(1);
            pl.left_turn = Some(st);
            if !g.hooks.is_empty() {
                hooks::fire_trigger(
                    g,
                    Event::Sacrifice,
                    Call::Sacrifice { p, what: crate::hooks::Sacrificed::Token("Treasure") },
                )?;
            }
        }
        Source::FloatR => g.player_mut(p).floating.r = g.player(p).floating.r.saturating_sub(1),
        Source::FloatAny => g.player_mut(p).floating.any = g.player(p).floating.any.saturating_sub(1),
        Source::FloatU => g.player_mut(p).floating.u = g.player(p).floating.u.saturating_sub(1),
        Source::FloatC => g.player_mut(p).floating.c = g.player(p).floating.c.saturating_sub(1),
        Source::FloatG => g.player_mut(p).floating.g = g.player(p).floating.g.saturating_sub(1),
        Source::FloatB => g.player_mut(p).floating.b = g.player(p).floating.b.saturating_sub(1),
        Source::Scion(_) | Source::HandCard(_) => cardcode::special_unit_paid(g, p, w)?,
        Source::Land(l) => {
            g.land_mut(l).tapped = true;
            g.tap_cols = Colors::from_letters(col);
            cardcode::on_tap_land(g, p, l, n)?;
            let t = &g.db.get(g.land(l).cd).tags;
            let (tomb, pain) = (t.has(Tag::Tomb), t.has(Tag::Pain));
            if tomb {
                lose_life(g, p, 2, Some(p), "other", Some(true))?; // Ancient Tomb deals 2 damage to you
            }
            if !col.is_empty() && pain {
                lose_life(g, p, 1, Some(p), "other", Some(true))?; // painlands
            }
        }
        Source::Perm(m) => {
            g.perm_mut(m).tapped = true;
            g.tap_cols = Colors::from_letters(col);
            cardcode::on_tap_perm(g, p, m, n)?;
            if !g.hooks.is_empty() {
                for (src, imp) in hooks::hooked(g, Event::ManaTapped) {
                    (imp.mana_tapped.unwrap())(g, src, p, m, n)?;
                }
            }
            if !col.is_empty() && g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Pain)) {
                lose_life(g, p, 1, Some(p), "other", Some(true))?; // Talismans
            }
        }
    }
    Ok(())
}

pub fn total_mana(g: &Game, p: PlayerId, convoke: bool) -> u32 {
    mana_units(g, p, convoke).iter().map(|w| w.amt).sum()
}

/// life a cast of c paid for Phyrexian symbols, given the pips it paid with mana (cost_of)
pub fn phyrexian_life(g: &Game, c: CardId, pips: &str) -> i32 {
    let d = g.db.get(c);
    let mut seen = vec![];
    let mut life = 0;
    for col in d.phyrexian.chars() {
        if seen.contains(&col) {
            continue;
        }
        seen.push(col);
        let phy = d.phyrexian.chars().filter(|&x| x == col).count() as i32;
        let printed = d.pips.chars().filter(|&x| x == col).count() as i32;
        let paid = pips.chars().filter(|&x| x == col).count() as i32;
        life += 2 * phy.min((printed - paid).max(0));
    }
    life
}

/// what card c costs p now: (generic, coloured pips). Python's `cost_of`.
pub fn cost_of(g: &Game, p: PlayerId, c: CardId) -> (u32, String) {
    let d = g.db.get(c);
    let pl = g.player(p);
    let t = &d.tags;
    let mut gn = d.generic as i32;
    let mut pips: String = d.pips.to_string();
    if t.has(Tag::PerCreature) {
        // Blasphemous Act, Vanquish the Horde
        let n = g
            .players
            .iter()
            .filter(|q| q.alive)
            .flat_map(|q| q.perms.iter())
            .filter(|&&m| g.is_creature(m) && !g.perm(m).phased)
            .count() as i32;
        gn = (gn - n).max(0);
    }
    if t.has(Tag::Eris) {
        // -2 per distinct mana value among instants and sorceries in the graveyard
        let mut mvs: Vec<u32> =
            pl.gy.iter().map(|&x| g.db.get(x)).filter(|x| x.instant || x.sorcery).map(|x| x.cmc).collect();
        mvs.sort();
        mvs.dedup();
        gn = (gn - 2 * mvs.len() as i32).max(0);
    }
    if t.has(Tag::Dawning) {
        gn = (gn - pl.gy.iter().filter(|&&x| g.db.get(x).instant || g.db.get(x).sorcery).count() as i32).max(0);
    }
    if t.has(Tag::Spectacle) && pl.hit_turn == pl.turns as i32 {
        gn = 0;
        pips = "R".into();
    }
    for col in d.phyrexian.chars() {
        // {B/P}: 2 life instead of its colour (paid by the caster), when the mana isn't there
        if pl.life > 10 && pips.contains(col) && !can_pay(g, p, gn.max(0) as u32, &pips, false) {
            pips = pips.replacen(col, "", 1);
        }
    }
    gn = (gn + crate::dsl::cost_delta(g, p, c)).max(0);
    if !g.hooks.is_empty() {
        gn = (gn + hooks::total_cost(g, p, c)).max(0);
        let floor = hooks::max_min_cost(g, p, c); // Trinisphere
        gn = gn.max(floor - pips.chars().count() as i32);
    }
    gn = (gn + cardcode::self_cost(g, p, c)).max(0); // Draco's domain, delve
    if pl.emblems.contains(&"tamiyo") && pl.hand.contains(&c) {
        return (0, String::new());
    }
    if crate::engine::values::stopped(g, &d.name) {
        gn += 3; // Disruptor Flute tax
    }
    if c == pl.cmd && !pl.hand.contains(&c) && &*d.name != "Liesa, Shroud of Dusk" {
        gn += pl.tax as i32; // no tax from hand
    }
    if pl.agent_ids.contains(&c) {
        // Opposition Agent: spend mana as though it were mana of any type
        let off: String = pips.chars().filter(|&x| !pl.ident.has(x)).collect();
        if !off.is_empty() {
            gn += off.chars().count() as i32;
            pips = pips.chars().filter(|&x| pl.ident.has(x)).collect();
        }
    }
    (gn.max(0) as u32, pips)
}
