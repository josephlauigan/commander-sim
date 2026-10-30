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
use crate::engine::pool::{Keep, Key, Kind, Restr, Rider};
use crate::engine::values::has;
use crate::flow::Res;
use crate::hooks::{Call, Event};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::tag::Tag;

/// Where a unit of mana comes from (Python's first element of a unit).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Land(PermId),
    Perm(PermId),
    /// a Treasure token ('T')
    Treasure,
    /// mana in the pool (`engine::pool`): the entry it is taken from
    Pool(Key),
    /// an Eldrazi Scion or Spawn: sacrifice for {C} ('SC')
    Scion(PermId),
    /// a card in hand that makes mana when exiled (Elvish Spirit Guide: 'H')
    HandCard(CardId),
    /// Quirion Ranger (the first) returns a Forest to hand to untap mana creature `dork` (the second): 'QR'
    Quirion(PermId, PermId),
    /// Wirewood Symbiote (the first) returns an Elf (the third) to hand to untap mana creature `dork` (the second): 'WS'
    Symbiote(PermId, PermId, PermId),
}

impl Source {
    /// unrestricted floating mana of `kind` that empties as the step ends (a ritual's, a source's leftover)
    pub const fn float(kind: Kind) -> Source {
        Source::Pool(Key::plain(kind))
    }
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

/// Blood Moon / Magus of the Moon make nonbasic land `l` a plain Mountain: it loses its other land types and every
/// ability (Ancient Tomb's damage, Urza's Saga's chapters, manlands, utility abilities, enters-tapped rules) and
/// taps for {R} only. Cheap: a basic or a game with no hooked card answers at once.
pub fn moon_land(g: &Game, l: PermId) -> bool {
    moon_card(g, g.perm(l).card())
}

/// the same for a land card that is about to enter
pub fn moon_card(g: &Game, cd: CardId) -> bool {
    !g.hooks.is_empty() && !BASIC_NAMES.contains(&&*g.db.get(cd).name) && cardcode::blood_moon(g)
}

/// what land L taps for, the board-wide checks (Blood Moon, Dryad of the Ilysian Grove) already made
fn land_cols(g: &Game, p: PlayerId, l: PermId, anyc: bool, moon: bool, dryad: bool) -> Colors {
    let d = g.db.get(g.perm(l).card());
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
pub fn land_colors_now(g: &Game, p: PlayerId, l: PermId) -> Colors {
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
        || (pl.lands.iter().any(|&l| g.db.get(g.perm(l).card()).tag(Tag::Worldtree)) && pl.lands.len() >= 6);
    let hooked = !g.hooks.is_empty();
    let moon = hooked && cardcode::blood_moon(g);
    let dryad = hooked && cardcode::dryad_colors(g, p);
    for &l in &pl.lands {
        let land = g.perm(l);
        if land.tapped {
            continue;
        }
        let t = &g.db.get(land.card()).tags;
        // Blood Moon: a plain Mountain with no abilities of its own (the board-wide check was made above)
        let plain = moon && !BASIC_NAMES.contains(&&*g.db.get(land.card()).name);
        if !plain && !restr_ok(g, p, g.db.get(land.card()).mana_restr) {
            continue; // Mishra's Workshop: artifact spells only
        }
        if !plain && t.has(Tag::Tomb) && pl.life <= 2 {
            continue; // Ancient Tomb's 2 damage would kill you
        }
        let mut amt = if plain { 1 } else { t.int(Tag::Amt).unwrap_or(1) };
        if !plain && let Some(n) = cardcode::dyn_mana_land(g, p, l) {
            amt = n as i32;
        }
        if hooked {
            amt += hooks::total_land_mana(g, p, l);
        }
        let mut cols = land_cols(g, p, l, anyc, moon, dryad);
        if hooked && amt >= 2 && hooks::land_colorless(g, p, l, amt as u32) {
            cols = Colors::NONE; // Damping Sphere: one {C} instead of any other type and amount
            amt = 1;
        }
        u.push(Unit { src: Source::Land(l), cols, amt: amt.max(0) as u32 });
    }
    let rite = has(g, p, Tag::Rite);
    let no_art_mana = hooked && hooks::total_count(g, Event::NoArtifactMana, p) != 0;
    // Shang-Chi, Tyvar: creature mana abilities work while summoning sick (checked lazily)
    let ahaste = || crate::impls::t6_legends1b::ability_haste(g, p);
    for &m in &pl.perms {
        let x = g.perm(m);
        if x.tapped || x.phased {
            continue;
        }
        if !g.auras.is_empty() && cardcode::locked(g, m, "noact") {
            continue; // Arrest, Encrust: no mana abilities either
        }
        let Some(c) = x.cd else {
            if rite && (!x.sick || ahaste()) {
                u.push(Unit { src: Source::Perm(m), cols: ident, amt: 1 });
            } else if let Some(Val::Str(cols)) = x.data.get(DataKey::Manatok).filter(|_| !x.sick || ahaste()) {
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
            if !restr_ok(g, p, d.mana_restr) {
                continue; // Powerstone, The Mightstone and Weakstone: not for nonartifact spells
            }
            let (a, col) = rock.split_once(':').unwrap_or((rock, "C"));
            let amt = cardcode::dyn_mana_perm(g, p, m).unwrap_or_else(|| a.parse().unwrap_or(1));
            let mut cols = tag_colors(col, ident);
            if &*d.name == "Star Compass" {
                cols = crate::impls::tymna_cards::star_compass_cols(g, p);
                if cols.is_empty() {
                    continue; // no basic land to copy
                }
            }
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
        } else if let Some(dork) = t.str(Tag::Dork).filter(|_| !x.sick || ahaste()) {
            if !restr_ok(g, p, d.mana_restr) {
                continue; // Giada (Angel spells), Hydro-Channeler (instants and sorceries), Rivaz (Dragon creatures)
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
        } else if rite && d.creature && (!x.sick || ahaste()) && x.noatk {
            u.push(Unit { src: Source::Perm(m), cols: ident, amt: 1 });
        }
    }
    let tre = if no_art_mana { 0 } else { pl.live_treasures() };
    let bonus = if hooked { hooks::total_count(g, Event::TreasureBonus, p) } else { 0 };
    // one unit per Treasure, at most 1000: a doubling loop (Miirym t5: Thrakkus copies doubling the power of Old
    // Gnawbone copies) can make billions, and a unit each ran the game out of memory; 1000 mana pays for anything
    for _ in 0..tre.min(1000) {
        u.push(Unit { src: Source::Treasure, cols: ident, amt: (1 + bonus).max(0) as u32 });
    }
    if hooked {
        u = adjust_mana(g, p, u);
    } else {
        u.extend(cardcode::hand_mana(g, p)); // Spirit Guides need no hooked permanent
    }
    // floating mana, at most 1000 of each entry (the same doubling boards float billions); restricted mana only where
    // it may be spent
    for m in pl.mana.iter() {
        if restr_ok(g, p, m.key.restr) {
            let cols = m.key.kind.cols(ident);
            u.extend((0..m.amt.min(1000)).map(|_| Unit { src: Source::Pool(m.key), cols, amt: 1 }));
        }
    }
    if convoke {
        // each untapped creature pays for {1} or one coloured pip
        for &m in &pl.perms {
            let x = g.perm(m);
            if g.is_creature(m) && !x.tapped && !x.phased && !u.iter().any(|w| w.src == Source::Perm(m)) {
                u.push(Unit { src: Source::Perm(m), cols: ident, amt: 1 });
            }
        }
    }
    if !g.keep_untapped.is_empty() {
        // a combo piece that must stay untapped is no mana source, by its own ability or another's (Relic of Legends,
        // Enduring Vitality add theirs through `extra_mana`)
        u.retain(|w| !matches!(w.src, Source::Perm(m) | Source::Scion(m) if g.keep_untapped.contains(&m)));
    }
    u
}

/// May mana with restriction `r` pay for what p is paying for now (rule 106.6)? `Game::pay_for` is the spell being
/// cast; `None` is an ability or another cost (a tax, a ward payment), which only `None` and `NotNonartifact` mana
/// may pay.
#[inline]
pub fn restr_ok(g: &Game, p: PlayerId, r: Restr) -> bool {
    let spell = || g.pay_for.map(|c| g.db.get(c));
    match r {
        Restr::None => true,
        Restr::NotNonartifact => spell().is_none_or(|d| d.types.has(crate::cards::Types::ARTIFACT)),
        Restr::ArtifactSpell => spell().is_some_and(|d| d.types.has(crate::cards::Types::ARTIFACT)),
        Restr::InstantSorcery => spell().is_some_and(|d| d.instant || d.sorcery),
        Restr::Legendary => spell().is_some_and(|d| d.tag(Tag::Leg)),
        Restr::DragonCreature => {
            spell().is_some_and(|d| d.creature && (d.has_subtype("dragon") || d.has_kw("changeling")))
        }
        Restr::Subtype(t) => spell().is_some_and(|d| d.has_subtype(t) || d.has_kw("changeling")),
        Restr::CreatureType(t) => {
            g.pay_for.is_some_and(|c| g.db.get(c).creature && crate::impls::galadriel::card_is(g, p, c, t))
        }
    }
}

/// The restriction on the mana a card's own mana ability makes (`CardDef::mana_restr`, looked up once at load): its
/// tags (Mishra's Workshop, the Powerstones) or its name. Lands that make {C} or restricted coloured mana (Plaza of
/// Heroes, Unclaimed Territory ...) choose their colours by `restr_ok` in their colour code instead.
/// the rider on the mana a card's own mana ability makes (`CardDef::mana_rider`, at load)
pub fn rider_of(name: &str) -> Rider {
    match name {
        "Cavern of Souls" | "Delighted Halfling" => Rider::Uncounterable,
        _ => Rider::None,
    }
}

pub fn restr_of(name: &str, tags: &crate::cards::Tags) -> Restr {
    if tags.has(Tag::Workshop) {
        return Restr::ArtifactSpell;
    }
    if tags.has(Tag::Pstone) {
        return Restr::NotNonartifact;
    }
    match name {
        "Giada, Font of Hope" => Restr::Subtype("angel"),
        "Hydro-Channeler" => Restr::InstantSorcery,
        "Rivaz of the Claw" => Restr::DragonCreature,
        _ => Restr::None,
    }
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
    // --- Tiers 4/5 (chulane-kinnan): Nyxbloom Ancient triples what a tapped permanent makes (before Kinnan's extra)
    let nyx = crate::impls::t45_chulane_kinnan::nyxbloom_factor(g, p);
    let triple = |w: &mut Unit| {
        if nyx > 1 && crate::impls::t45_chulane_kinnan::tapped_source(w.src) {
            w.amt *= nyx;
        }
    };
    u.iter_mut().for_each(triple);
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
            let mut extra = (imp.extra_mana.unwrap())(g, src, p, &u);
            extra.iter_mut().for_each(triple); // --- Tiers 4/5 (chulane-kinnan): Nyxbloom Ancient
            u.extend(extra);
        }
    }
    u.extend(cardcode::hand_mana(g, p)); // Spirit Guides (also added in `units` when nothing is hooked)
    u
}

/// restricted mana in the pool (Karn's): the planner spends it first, while it may
fn restricted_float(src: Source) -> bool {
    matches!(src, Source::Pool(k) if k.restr != Restr::None)
}

fn is_pain(g: &Game, src: Source) -> bool {
    match src {
        Source::Land(l) => {
            let t = &g.db.get(g.perm(l).card()).tags;
            (t.has(Tag::Tomb) || t.has(Tag::Pain)) && !moon_land(g, l)
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
        type Key = (bool, bool, bool, u32, u32, i64); // Karn's mana first, Treasure last, pain late, waste, colours, scarcity
        let mut best: Option<(usize, Key)> = None;
        for (i, w) in u.iter().enumerate() {
            if rem[i] == 0 {
                continue;
            }
            let waste = if used[i] > 0 { 0 } else { rem[i].saturating_sub(need) };
            let scarce = avail.iter().filter(|(c, _)| w.cols.has(*c)).map(|x| x.1).min().unwrap_or(0);
            let k = (!restricted_float(w.src), w.src == Source::Treasure, is_pain(g, w.src), waste, w.cols.count(), -(scarce as i64));
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
    if crate::human::is_human(g, p) {
        return crate::human::mana::pool_can_pay(g, p, generic, pips); // practice mode: from the person's pool
    }
    let u = mana_units(g, p, convoke);
    plan_pay(g, &u, generic, pips).is_some()
}

/// p pays the cost; false (nothing paid) if it can't
pub fn pay(g: &mut Game, p: PlayerId, generic: u32, pips: &str, convoke: bool) -> Res<bool> {
    if crate::human::is_human(g, p) {
        let ok = crate::human::mana::pay_from_pool(g, p, generic, pips).is_none();
        if ok {
            if !g.spent_keep {
                g.mana_spent = 0;
                g.mana_colored = false;
            }
            g.mana_spent += generic + pips.chars().count() as u32; // the pool does not say which mana paid
            g.mana_colored |= !pips.is_empty();
        }
        return Ok(ok);
    }
    g.treasure_paid = 0;
    if !g.spent_keep {
        g.mana_spent = 0;
        g.mana_colored = false;
    }
    let u = mana_units(g, p, convoke);
    let Some((used, col)) = plan_pay(g, &u, generic, pips) else { return Ok(false) };
    for (i, w) in u.iter().enumerate() {
        if used[i] > 0 {
            spend_unit(g, p, *w, used[i], &col[i])?;
            let on_demand = matches!(w.src, Source::Perm(m) if crate::impls::t6_mana::mana_on_demand(g, m));
            if used[i] < w.amt && matches!(w.src, Source::Land(_) | Source::Perm(_)) && !on_demand {
                float_leftover(g, p, w.src, w.cols, w.amt - used[i]);
            }
        }
    }
    Ok(true)
}

/// The mana a land or permanent made beyond what the payment used stays in the pool, as in the rules: Gaea's Cradle
/// tapped for four paid one {G} of Sisay's {W}{U}{B}{R}{G} and the other three were lost, so a line that was
/// affordable stopped after its first step. It empties as the step ends (rule 500.4). One colour floats as that
/// colour, none as {C}, several as any; mana of a restricted source keeps its restriction (Mishra's Workshop's
/// {C}{C} after a one-mana artifact still pays only for artifact spells).
fn float_leftover(g: &mut Game, p: PlayerId, src: Source, cols: Colors, n: u32) {
    let restr = source_restr(g, src);
    g.player_mut(p).mana.add_key(Key { restr, kind: Kind::of_leftover(cols), keep: Keep::Step }, n);
}

/// the restriction on the mana a land or permanent makes by its own ability (none for a land under Blood Moon: a
/// plain Mountain)
fn source_restr(g: &Game, src: Source) -> Restr {
    match src {
        Source::Land(l) if !moon_land(g, l) => g.db.get(g.perm(l).card()).mana_restr,
        Source::Perm(m) => g.perm(m).cd.map_or(Restr::None, |c| g.db.get(c).mana_restr),
        _ => Restr::None,
    }
}

/// Cavern of Souls' and Delighted Halfling's rider: their coloured mana (made only for the spell it may pay for)
/// makes that spell uncounterable (`Game::unc_cast`, read by `stack::counterable`)
fn mana_rider(g: &mut Game, src: Source, w: Unit) {
    let cd = match src {
        Source::Land(l) if !moon_land(g, l) => Some(g.perm(l).card()),
        Source::Perm(m) => g.perm(m).cd,
        _ => None,
    };
    if let Some(cd) = cd
        && g.db.get(cd).mana_rider == Rider::Uncounterable
        && !w.cols.is_empty()
        && let Some(c) = g.pay_for
    {
        g.unc_cast = Some((c, g.turn_stamp()));
    }
}

/// Use unit w for n mana; col: the coloured pips it paid for (painlands hurt only then). Taps or sacrifices the
/// source and runs its side effects: Treasures, floating mana, on-tap triggers, pain. Python's `spend_unit`.
pub fn spend_unit(g: &mut Game, p: PlayerId, w: Unit, n: u32, col: &str) -> Res {
    g.mana_spent += n; // the record of mana spent on the spell being paid for
    g.mana_colored |= !w.cols.is_empty();
    crate::t6stats::flow_spent(g, p, n); // --- ai-flow
    match w.src {
        Source::Treasure => {
            let st = g.turn_stamp();
            g.treasure_paid += 1;
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
        Source::Pool(k) => g.player_mut(p).mana.take(k, n),
        Source::Scion(_) | Source::HandCard(_) | Source::Quirion(..) | Source::Symbiote(..) => {
            cardcode::special_unit_paid(g, p, w)?
        }
        Source::Land(l) => {
            mana_rider(g, w.src, w);
            g.perm_mut(l).tapped = true;
            g.tap_cols = Colors::from_letters(col);
            if !moon_land(g, l) {
                cardcode::on_tap_land(g, p, l, n)?;
            }
            let t = &g.db.get(g.perm(l).card()).tags;
            let plain = moon_land(g, l); // Blood Moon: no Tomb damage, no pain
            let (tomb, pain) = (!plain && t.has(Tag::Tomb), !plain && t.has(Tag::Pain));
            if tomb {
                lose_life(g, p, 2, Some(p), "other", Some(true))?; // Ancient Tomb deals 2 damage to you
            }
            if !col.is_empty() && pain {
                lose_life(g, p, 1, Some(p), "other", Some(true))?; // painlands
            }
        }
        Source::Perm(m) => {
            mana_rider(g, w.src, w);
            g.perm_mut(m).tapped = true;
            crate::impls::t0_captain::became_tapped(g, m); // Captain America, Living Legend
            crate::impls::t3_marwyn::on_tapped(g, m)?; // --- Tier 3 (marwyn): Dionus, Legolas
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
pub fn phyrexian_life(g: &Game, p: PlayerId, c: CardId, pips: &str) -> i32 {
    let d = g.db.get(c);
    let mut seen = vec![];
    let mut life = 0;
    let phys = crate::impls::krrik::life_pips(g, p, c); // K'rrik: every {B} of a black spell too
    for col in phys.chars() {
        if seen.contains(&col) {
            continue;
        }
        seen.push(col);
        let phy = phys.chars().filter(|&x| x == col).count() as i32;
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
    let mut paid_life = 0;
    for col in d.phyrexian.chars() {
        // {B/P}: 2 life instead of its colour (paid by the caster), when the mana isn't there. The life check counts the
        // pips already paid with life (Python checked the starting life for every pip, so three pips at 12 life went to 6)
        if pl.life - paid_life > 10 && pips.contains(col) && !can_pay(g, p, gn.max(0) as u32, &pips, false) {
            pips = pips.replacen(col, "", 1);
            paid_life += 2;
        }
    }
    crate::impls::krrik::drop_black(g, p, c, gn, &mut pips); // K'rrik: 2 life for a {B}
    crate::impls::t45_urdragon::morophon_pips(g, p, c, &mut pips); // --- Tiers 4/5 (urdragon): Morophon
    gn += crate::impls::t45_urdragon::eminence(g, p, c); // --- Tiers 4/5 (urdragon): The Ur-Dragon's eminence
    gn = (gn + crate::dsl::cost_delta(g, p, c)).max(0);
    gn = (gn + crate::impls::t0_stella::kaza_delta(g, p, c)).max(0); // Kaza, Roil Chaser
    if !g.hooks.is_empty() {
        gn = (gn + hooks::total_cost(g, p, c)).max(0);
        if crate::impls::blue_cards::defiler_discount(g, p, c) {
            pips = pips.replacen('U', "", 1); // Defiler of Dreams: pay 2 life for {U} less
        }
        let floor = hooks::max_min_cost(g, p, c); // Trinisphere
        gn = gn.max(floor - pips.chars().count() as i32);
    }
    gn = (gn + cardcode::self_cost(g, p, c)).max(0); // Draco's domain, delve
    if pl.emblems.contains(&"tamiyo") && pl.hand.contains(&c) {
        return (0, String::new());
    }
    if crate::impls::t45_urdragon::free_dragon(g, p, c) {
        // --- Tiers 4/5 (urdragon): Dracogenesis: no mana cost (the commander tax below is still paid)
        gn = 0;
        pips.clear();
    }
    if crate::engine::values::stopped(g, &d.name) {
        gn += 3; // Disruptor Flute tax
    }
    if pl.is_commander(c) && !pl.hand.contains(&c) && &*d.name != "Liesa, Shroud of Dusk" {
        gn += pl.tax_of(c) as i32; // no tax from hand
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

/// what casting these cards one after another costs in all: each one's `cost_of` now, plus {1} per Damping Sphere for
/// every spell of the line cast before it (the Sphere counts the spells cast this turn, and `cost_of` sees only those
/// already cast). The AI checks a combo or a sequence against this, not the printed costs
pub fn seq_cost(g: &Game, p: PlayerId, cards: &[CardId]) -> (u32, String) {
    let spheres = if g.hooks.is_empty() {
        0
    } else {
        hooks::hooked(g, Event::Cost)
            .iter()
            .filter(|(m, _)| g.perm(*m).cd.is_some_and(|c| &*g.db.get(c).name == "Damping Sphere"))
            .count() as u32
    };
    let (mut gn, mut pips, mut before) = (0, String::new(), 0);
    for &c in cards {
        let (cg, cp) = cost_of(g, p, c);
        if !g.db.get(c).land {
            gn += spheres * before;
            before += 1;
        }
        gn += cg;
        pips.push_str(&cp);
    }
    (gn, pips)
}

// --- ai-fix: C: rituals in hand counted as mana
/// The mana ritual `c` adds as it resolves: (amount, colours). A ritual is an instant or sorcery whose only effect adds
/// mana and that needs no additional cost (Culling the Weak's sacrifice is left out), or one of the hand-written ones:
/// Cabal Ritual (five with threshold), Rite of Flame (two, plus one for each in a graveyard), Desperate Ritual. The
/// ability language's rituals add mana of any of p's colours, as `add_mana` does.
pub fn ritual_out(g: &Game, p: PlayerId, c: CardId) -> Option<(u32, Colors)> {
    use crate::dsl::model::{Ability, Num};
    let d = g.db.get(c);
    if d.land || !(d.instant || d.sorcery) {
        return None;
    }
    let red = Colors::from_letters("R");
    match &*d.name {
        "Cabal Ritual" => return Some((if g.player(p).gy.len() >= 7 { 5 } else { 3 }, g.player(p).ident)),
        "Rite of Flame" => {
            let copies = g.players.iter().flat_map(|q| q.gy.iter()).filter(|&&x| &*g.db.get(x).name == "Rite of Flame");
            return Some((2 + copies.count() as u32, red));
        }
        "Desperate Ritual" => return Some((3, red)),
        _ => {}
    }
    if g.registry.get(c).is_some_and(|i| i.resolve.is_some()) {
        return None; // another hand-written spell
    }
    let mut out = None;
    for a in d.abilities.iter() {
        match a {
            Ability::AdditionalCost { .. } => return None,
            Ability::Spell { effects } => {
                if effects.is_empty() || !effects.iter().all(|e| e.kind == "add_mana") {
                    return None;
                }
                let n: i32 = effects.iter().map(|e| if let Some(Num::Int(k)) = &e.n { *k } else { 0 }).sum();
                if n <= 0 {
                    return None;
                }
                out = Some((n as u32, g.player(p).ident));
            }
            _ => {}
        }
    }
    out
}

/// what a ritual nets: the mana it adds less its cost now (0 for a card that is no ritual)
pub fn ritual_net(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let Some((out, _)) = ritual_out(g, p, c) else { return 0 };
    let (gn, pips) = cost_of(g, p, c);
    out as i32 - gn as i32 - pips.chars().count() as i32
}

/// p's mana units once ritual `c` is cast from `u`: its cost paid, its mana added. None: no ritual, or `u` can't pay
/// for it. A source that made more than the cost used keeps the rest (it floats, as `pay` leaves it).
pub fn units_after_ritual(g: &Game, p: PlayerId, c: CardId, u: &[Unit]) -> Option<Vec<Unit>> {
    let (out, cols) = ritual_out(g, p, c)?;
    let (gn, pips) = cost_of(g, p, c);
    let (used, _) = plan_pay(g, u, gn, &pips)?;
    let mut v: Vec<Unit> =
        u.iter().zip(&used).filter(|(w, n)| w.amt > **n).map(|(w, n)| Unit { amt: w.amt - n, ..*w }).collect();
    let src = Source::float(if cols == Colors::from_letters("R") { Kind::R } else { Kind::Any });
    v.extend((0..out).map(|_| Unit { src, cols, amt: 1 }));
    Some(v)
}

/// the rituals in p's hand that net mana and can be cast now, the biggest net first; never one in `skip` (a piece of
/// the line being paid for, or a card it casts)
pub fn rituals_in_hand(g: &Game, p: PlayerId, skip: &[CardId]) -> Vec<CardId> {
    let mut v: Vec<CardId> = g
        .player(p)
        .hand
        .iter()
        .copied()
        .filter(|&c| !skip.contains(&c) && ritual_net(g, p, c) > 0 && crate::engine::cast::castable(g, p, c, "hand"))
        .collect();
    v.sort_by_key(|&c| -ritual_net(g, p, c)); // stable: hand order breaks ties
    v
}

/// can p pay `generic` + `pips` now, casting rituals from its hand first if it must? Each ritual counts once, and only
/// in the order `rituals_in_hand` gives (the one that nets most first). A person's pool is never stretched this way.
pub fn can_pay_with_rituals(g: &Game, p: PlayerId, generic: u32, pips: &str, skip: &[CardId]) -> bool {
    if crate::human::is_human(g, p) {
        return can_pay(g, p, generic, pips, false);
    }
    let mut u = mana_units(g, p, false);
    if plan_pay(g, &u, generic, pips).is_some() {
        return true;
    }
    for c in rituals_in_hand(g, p, skip) {
        if let Some(v) = units_after_ritual(g, p, c, &u) {
            u = v;
            if plan_pay(g, &u, generic, pips).is_some() {
                return true;
            }
        }
    }
    false
}

// --- ai-fix: D
/// p's mana units once card `c` is cast from `u`: its cost paid, and the mana it makes this turn added (a ritual's, or
/// a noncreature rock's: it can tap as it enters; a dork is summoning sick, Chrome Mox's colour is unknown, so neither
/// adds any). None if `u` can't pay for it.
pub fn units_after_cast(g: &Game, p: PlayerId, c: CardId, u: &[Unit]) -> Option<Vec<Unit>> {
    if ritual_out(g, p, c).is_some() {
        return units_after_ritual(g, p, c, u);
    }
    let (gn, pips) = cost_of(g, p, c);
    let (used, _) = plan_pay(g, u, gn, &pips)?;
    let mut v: Vec<Unit> =
        u.iter().zip(&used).filter(|(w, n)| w.amt > **n).map(|(w, n)| Unit { amt: w.amt - n, ..*w }).collect();
    let d = g.db.get(c);
    if !d.creature
        && let Some(rock) = d.tags.str(Tag::Rock)
    {
        let (a, col) = rock.split_once(':').unwrap_or((rock, "C"));
        let amt: u32 = a.parse().unwrap_or(1);
        v.push(Unit { src: Source::float(Kind::Any), cols: tag_colors(col, g.player(p).ident), amt });
    }
    Some(v)
}

/// can units `u` pay `generic` + `pips`, with p's rituals in hand cast first if they must (as `can_pay_with_rituals`)?
pub fn units_pay_with_rituals(g: &Game, p: PlayerId, u: &[Unit], generic: u32, pips: &str, skip: &[CardId]) -> bool {
    if plan_pay(g, u, generic, pips).is_some() {
        return true;
    }
    let mut u = u.to_vec();
    for c in rituals_in_hand(g, p, skip) {
        if let Some(v) = units_after_ritual(g, p, c, &u) {
            u = v;
            if plan_pay(g, &u, generic, pips).is_some() {
                return true;
            }
        }
    }
    false
}

// --- ai-fix: B
/// the mana a land card would add once played, as a unit: its tagged colours (`A`: p's identity) and amount. A guess
/// for a card not yet on the battlefield: no Blood Moon, no enters-tapped rule.
pub fn land_card_unit(g: &Game, p: PlayerId, c: CardId) -> Unit {
    let t = &g.db.get(c).tags;
    let amt = t.int(Tag::Amt).unwrap_or(1).max(0) as u32;
    Unit { src: Source::float(Kind::Any), cols: tag_colors(t.str(Tag::C).unwrap_or("C"), g.player(p).ident), amt }
}

// --- ai-fix: B
/// About how much mana p will have on its next turn: every land and mana permanent untapped (Mana Vault and its kind
/// too), a dork no longer summoning sick, and one more land if it holds one. Colours are not counted.
pub fn next_turn_mana(g: &Game, p: PlayerId) -> u32 {
    let pl = g.player(p);
    let lands: u32 = pl
        .lands
        .iter()
        .filter(|&&l| g.perm(l).on_bf)
        .map(|&l| g.db.get(g.perm(l).card()).tags.int(Tag::Amt).unwrap_or(1).max(0) as u32)
        .sum();
    let perms: u32 = pl
        .perms
        .iter()
        .filter(|&&m| !g.perm(m).phased)
        .filter_map(|&m| g.perm(m).cd)
        .map(|c| {
            let t = &g.db.get(c).tags;
            if let Some(r) = t.str(Tag::Rock) {
                r.split_once(':').map_or(r, |x| x.0).parse().unwrap_or(1)
            } else if t.has(Tag::Dork) || t.has(Tag::Chromemox) {
                1
            } else {
                0
            }
        })
        .sum();
    let drop = u32::from(pl.hand.iter().any(|&c| g.db.get(c).land));
    lands + perms + drop
}
