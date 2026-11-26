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
    /// --- card-models-2: Rain of Filth's "Sacrifice this land: Add {B}" (a land of p's, tapped or not), used last
    SacLand(PermId),
}

impl Source {
    /// unrestricted floating mana of `kind` that empties as the step ends (a ritual's, a source's leftover)
    pub const fn float(kind: Kind) -> Source {
        Source::Pool(Key::plain(kind))
    }
}

/// How the `amt` mana of one unit divide among the colours in `cols` (one permanent that makes several mana).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shape {
    /// each mana is any one of `cols`, chosen separately (a dork, a Treasure, one mana of a land)
    #[default]
    Free,
    /// all `amt` mana are ONE colour of `cols`, chosen as the source is tapped: "three mana of any one color" (Gilded
    /// Lotus, Lotus Field, Black Lotus, Sanctum Weaver; the card tag `oneclr`)
    One,
    /// `amt / cols.count()` whole sets of one mana of EACH colour in `cols` (Karoos {B}{G}, Signets, Faeburrow Elder,
    /// Bloom Tender); the few mana beyond whole sets (Kinnan's extra) are `Free`
    Each,
    /// a land with two alternative mana abilities, only one of which is used per tap: its own (`amt` mana of the
    /// colours in the payload, none = {C}: Ancient Tomb's {C}{C}, Cabal Coffers' {B} per Swamp) or, under Leyline of
    /// the Guildpact (a basic land type: "{T}: add one mana of any color"), ONE mana of any colour. `cols` is
    /// WUBRG then, `amt` the own ability's. Which ability a payment used is read from its amount and pips
    /// (`alt_uses_own`): the own one makes Ancient Tomb's damage.
    Alt(Colors),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unit {
    pub src: Source,
    /// the colours it can make; empty for colourless
    pub cols: Colors,
    pub amt: u32,
    /// how the mana split among `cols` (`Free` unless it is several mana from one source)
    pub shape: Shape,
}

impl Unit {
    /// `amt` mana, each any one of `cols`
    pub const fn free(src: Source, cols: Colors, amt: u32) -> Unit {
        Unit { src, cols, amt, shape: Shape::Free }
    }

    /// whole sets of one mana of each colour an `Each` unit makes
    fn sets(&self) -> u32 {
        match self.shape {
            Shape::Each => self.amt / self.cols.count().max(1),
            _ => 0,
        }
    }
}

/// The shape of the several mana a permanent makes. `printed` is the amount its ability prints (before Nyxbloom,
/// Kinnan or a count), `letters` its colour tag: an explicit set that prints as many mana as it has colours is one of
/// each ({B}{G}); "any one color" (`oneclr`) is all one colour; anything else is free.
fn shape_of(t: &crate::cards::Tags, letters: &str, cols: Colors, printed: u32, amt: u32, rock_or_land: bool) -> Shape {
    if amt < 2 || cols.count() < 2 {
        Shape::Free
    } else if t.has(Tag::Oneclr) {
        Shape::One
    } else if rock_or_land && letters != "A" && printed >= 2 && cols.count() == printed {
        Shape::Each
    } else {
        Shape::Free
    }
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

/// Urborg, Tomb of Yawgmoth / Yavimaya, Cradle of Growth: every land is also a Swamp / Forest, so it has that basic
/// type's mana ability too ({T}: add {B} / {G}). None under Blood Moon (`lands_are` checks it). Read once per call
/// by the callers, not per land. Only a land that makes one mana gets it: a unit is one colour set and one amount, so
/// adding {B} to Ancient Tomb's {C}{C} or Cabal Coffers' count would let it make that much {B}; those keep their
/// bigger ability.
fn granted_cols(g: &Game) -> Colors {
    let mut c = Colors::NONE;
    for (ch, s) in [('B', "B"), ('G', "G")] {
        if crate::engine::chars::lands_are(g, ch) {
            c = c.union(Colors::from_letters(s));
        }
    }
    c
}

/// `granted_cols` plus what p's own Leyline of the Guildpact gives its lands: every basic land type, so one mana of any
/// colour. Not to a land that is a plain Mountain under Blood Moon (`plain`: the engine has no timestamps, so the moon
/// wins; `granted_cols` is empty then, as `lands_are` skips such lands).
fn granted_cols_for(g: &Game, p: PlayerId, granted: Colors, plain: bool) -> Colors {
    if !plain && !g.hooks.is_empty() && crate::engine::chars::leyline(g, p) {
        granted.union(Colors::from_letters("WUBRG"))
    } else {
        granted
    }
}

/// the colours p's land L taps for now (Fellwar Stone reads opponents' lands)
pub fn land_colors_now(g: &Game, p: PlayerId, l: PermId) -> Colors {
    let moon = !g.hooks.is_empty() && cardcode::blood_moon(g);
    let dryad = !g.hooks.is_empty() && cardcode::dryad_colors(g, p);
    let cols = land_cols(g, p, l, false, moon, dryad);
    if g.db.get(g.perm(l).card()).tags.int(Tag::Amt).unwrap_or(1) == 1 {
        cols.union(granted_cols_for(g, p, granted_cols(g), moon_land(g, l)))
    } else if cols.count() <= 1 {
        cols.union(granted_cols_for(g, p, Colors::NONE, moon_land(g, l))) // Leyline: the alternative ability (`Shape::Alt`)
    } else {
        cols
    }
}

/// every mana unit p can use now (Python's `mana_units`)
pub fn mana_units(g: &Game, p: PlayerId, convoke: bool) -> Vec<Unit> {
    units(g, p, convoke, false, paying(g))
}

/// Manual sources include Rivaz outside spell payment; the person's pool keeps its restriction.
pub fn human_sources(g: &Game, p: PlayerId) -> Vec<Unit> {
    units(g, p, false, true, paying(g))
}

/// p's lands have a free "{T}: add one mana of any colour" of their own: Chromatic Lantern, The World Tree
pub fn lands_any_colour(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    has(g, p, Tag::Lantern)
        || (pl.lands.iter().any(|&l| g.db.get(g.perm(l).card()).tag(Tag::Worldtree)) && pl.lands.len() >= 6)
}

fn units(g: &Game, p: PlayerId, convoke: bool, manual: bool, pay_for: Option<CardId>) -> Vec<Unit> {
    let pl = g.player(p);
    let ident = pl.ident;
    let mut u = vec![];
    let anyc = lands_any_colour(g, p);
    let hooked = !g.hooks.is_empty();
    let moon = hooked && cardcode::blood_moon(g);
    let dryad = hooked && cardcode::dryad_colors(g, p);
    let granted = granted_cols(g); // Urborg / Yavimaya
    for &l in &pl.lands {
        let land = g.perm(l);
        if land.tapped {
            continue;
        }
        if crate::engine::combat::tap_sick(g, l) {
            continue; // a creature land (Astral Dragon's copy) is summoning sick: no {T} ability, its mana included
        }
        let t = &g.db.get(land.card()).tags;
        // Blood Moon: a plain Mountain with no abilities of its own (the board-wide check was made above)
        let plain = moon && !BASIC_NAMES.contains(&&*g.db.get(land.card()).name);
        if !plain && !restr_ok_for(g, p, pay_for, g.db.get(land.card()).mana_restr) {
            continue; // Mishra's Workshop: artifact spells only
        }
        let leyline = !plain && !granted_cols_for(g, p, Colors::NONE, plain).is_empty();
        // a granted "{T}: add one mana of any colour" (Leyline's basic types, a Lantern) that deals no damage and costs no life
        let granted_any = leyline || (anyc && !plain);
        if !plain && t.has(Tag::Tomb) && pl.life <= 2 {
            if granted_any {
                // Ancient Tomb's 2 damage would kill you, but its Leyline ability ({T}: one mana of any colour) deals none
                u.push(Unit {
                    src: Source::Land(l),
                    cols: Colors::from_letters("WUBRG"),
                    amt: 1,
                    shape: Shape::Alt(Colors::NONE),
                });
            }
            continue;
        }
        let mut amt = if plain { 1 } else { t.int(Tag::Amt).unwrap_or(1) };
        if !plain && let Some(n) = cardcode::dyn_mana_land(g, p, l) {
            amt = n as i32;
        }
        let one = amt == 1; // its own ability makes one mana (before Crypt Ghast's extra and the like)
        if hooked {
            amt += hooks::total_land_mana(g, p, l);
        }
        // a Lantern recolours a land that makes one mana; a land that makes several (Ancient Tomb, the Urza lands) keeps its
        // own ability and gets the Lantern's one mana of any colour as an alternative, like Leyline's (`Shape::Alt` below)
        let mut cols = land_cols(g, p, l, anyc && one, moon, dryad);
        if one {
            // Urborg / Yavimaya: also {B} / {G} (see granted_cols); Leyline of the Guildpact: any colour
            cols = cols.union(granted_cols_for(g, p, granted, plain));
        }
        if hooked && amt >= 2 && hooks::land_colorless(g, p, l, amt as u32) {
            cols = Colors::NONE; // Damping Sphere: one {C} instead of any other type and amount
            amt = 1;
        }
        let amt = amt.max(0) as u32;
        // several mana from one land: all one colour (Lotus Field), or one of each colour of its pair (a Karoo). Only
        // its own tagged colours: a land turned any-colour by a Lantern, Dryad or a card's rule stays free
        let mut shape = if amt >= 2 && !dryad && !plain && cardcode::land_colors(g, p, l).is_none() {
            let printed = t.int(Tag::Amt).unwrap_or(1).max(0) as u32;
            shape_of(t, t.str(Tag::C).unwrap_or("C"), cols, printed, amt, true)
        } else {
            Shape::Free
        };
        if granted_any && !one && !dryad && amt >= 2 && shape == Shape::Free && cols.count() <= 1 {
            // Leyline of the Guildpact: Ancient Tomb, Cabal Coffers, Gaea's Cradle, Mishra's Workshop also have the
            // basic types' ability, one mana of any colour: an alternative to their own, never both in one tap
            if cardcode::land_colors(g, p, l).is_none() {
                shape = Shape::Alt(cols);
                cols = Colors::from_letters("WUBRG");
            }
        }
        u.push(Unit { src: Source::Land(l), cols, amt, shape });
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
            if rite && !crate::engine::combat::tap_sick(g, m) {
                u.push(Unit::free(Source::Perm(m), ident, 1));
            } else if let Some(Val::Str(cols)) =
                x.data.get(DataKey::Manatok).filter(|_| !crate::engine::combat::tap_sick(g, m))
            {
                u.push(Unit::free(Source::Perm(m), Colors::from_letters(cols), 1));
            } else if x.ttypes.contains(&"scion") || x.ttypes.contains(&"spawn") {
                u.push(Unit::free(Source::Scion(m), Colors::NONE, 1));
            }
            continue;
        };
        let d = g.db.get(c);
        let t = &d.tags;
        if t.has(Tag::Chromemox) {
            // Chrome Mox: one mana of the imprinted card's colours (none if nothing imprinted)
            if !x.colors.is_empty() {
                u.push(Unit::free(Source::Perm(m), x.colors, 1));
            }
            continue;
        }
        if let Some(rock) = t.str(Tag::Rock) {
            if no_art_mana && d.types.has(crate::cards::Types::ARTIFACT) {
                continue;
            }
            if !restr_ok_for(g, p, pay_for, d.mana_restr) && !(manual && d.mana_restr == Restr::CreatureSpell) {
                continue; // Powerstone, The Mightstone and Weakstone: not for nonartifact spells; Herd Heirloom: creature spells
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
                // Gilded Lotus: all of it one colour; a Signet: one of each colour of its pair
                let shape = shape_of(t, col, cols, a.parse().unwrap_or(1), amt, true);
                u.push(Unit { src: Source::Perm(m), cols, amt, shape });
            }
        } else if let Some(dork) = t.str(Tag::Dork).filter(|_| !crate::engine::combat::tap_sick(g, m)) {
            if d.mana_restr == Restr::DragonCreature && x.neutered {
                continue;
            }
            if !restr_ok_for(g, p, pay_for, d.mana_restr) && !(manual && d.mana_restr == Restr::DragonCreature) {
                continue; // Giada (Angel spells), Hydro-Channeler (instants and sorceries), Rivaz (Dragon creatures)
            }
            let amt = cardcode::dyn_mana_perm(g, p, m).unwrap_or(1);
            let mut each = false;
            let cols = if &*d.name == "Delighted Halfling" {
                cardcode::halfling_colors(g, p).unwrap_or(Colors::NONE)
            } else if &*d.name == "Bloom Tender" {
                // one mana of each colour among your permanents (never more than those colours: `amt` counts them)
                each = true;
                let c = crate::impls::t5::bloom_tender_colours(g, p);
                if c.is_empty() { tag_colors(dork, ident) } else { c }
            } else {
                tag_colors(dork, ident)
            };
            if amt > 0 {
                // Sanctum Weaver, Wirewood Channeler: X mana of any one colour
                let shape = if each && cols.count() == amt && amt >= 2 {
                    Shape::Each
                } else {
                    shape_of(t, dork, cols, 1, amt, false)
                };
                u.push(Unit { src: Source::Perm(m), cols, amt, shape });
            }
        } else if &*d.name == crate::impls::newcards_b::MARVIN && !crate::engine::combat::tap_sick(g, m) {
            if let Some(unit) = marvin_mana(g, p, m) {
                u.push(unit);
            }
        } else if rite && d.creature && x.noatk && !crate::engine::combat::tap_sick(g, m) {
            u.push(Unit::free(Source::Perm(m), ident, 1));
        }
    }
    if crate::impls::t6_blue::rain_of_filth_active(g, p) {
        // --- card-models-2: Rain of Filth: every land p controls, tapped or not, can be sacrificed for {B}
        for &l in &pl.lands {
            u.push(Unit::free(Source::SacLand(l), Colors::from_letters("B"), 1));
        }
    }
    let tre = if no_art_mana { 0 } else { pl.live_treasures() };
    let bonus = if hooked { hooks::total_count(g, Event::TreasureBonus, p) } else { 0 };
    // one unit per Treasure, at most 1000: a doubling loop (Miirym t5: Thrakkus copies doubling the power of Old
    // Gnawbone copies) can make billions, and a unit each ran the game out of memory; 1000 mana pays for anything
    let tre_amt = (1 + bonus).max(0) as u32;
    // Goldspan Dragon: "two mana of any one color", not two single mana
    let tre_shape = if tre_amt >= 2 && ident.count() >= 2 { Shape::One } else { Shape::Free };
    for _ in 0..tre.min(1000) {
        u.push(Unit { src: Source::Treasure, cols: ident, amt: tre_amt, shape: tre_shape });
    }
    if hooked {
        u = adjust_mana(g, p, u);
    } else {
        u.extend(cardcode::hand_mana(g, p)); // Spirit Guides need no hooked permanent
    }
    // floating mana, at most 1000 of each entry (the same doubling boards float billions); restricted mana only where
    // it may be spent
    for m in pl.mana.iter() {
        if restr_ok_for(g, p, pay_for, m.key.restr) {
            let cols = m.key.kind.cols(ident);
            u.extend((0..m.amt.min(1000)).map(|_| Unit::free(Source::Pool(m.key), cols, 1)));
        }
    }
    if convoke {
        // each untapped creature pays for {1} or one coloured pip
        for &m in &pl.perms {
            let x = g.perm(m);
            if g.is_creature(m) && !x.tapped && !x.phased && !u.iter().any(|w| w.src == Source::Perm(m)) {
                u.push(Unit::free(Source::Perm(m), ident, 1));
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
    restr_ok_for(g, p, paying(g), r)
}

thread_local! {
    /// the spell a read-only affordability check (`can_pay_spell`, `total_mana_spell`) asks about: the colour hooks of
    /// lands and creatures (Plaza of Heroes, Cavern of Souls, Haven of the Spirit Dragon ...) read it through
    /// `paying`, so the check sees the same colours `pay_spell` will (which sets `Game::pay_for` itself)
    static PAY_PEEK: std::cell::Cell<Option<CardId>> = const { std::cell::Cell::new(None) };
}

/// puts the peeked spell back when the check ends, even by a panic
struct PeekGuard(Option<CardId>);

impl Drop for PeekGuard {
    fn drop(&mut self) {
        PAY_PEEK.with(|c| c.set(self.0));
    }
}

fn peek(c: CardId) -> PeekGuard {
    PeekGuard(PAY_PEEK.with(|x| x.replace(Some(c))))
}

/// The spell being paid for, as the mana colour hooks see it: the one a read-only check is asking about, else
/// `Game::pay_for`.
#[inline]
pub fn paying(g: &Game) -> Option<CardId> {
    PAY_PEEK.with(|c| c.get()).or(g.pay_for)
}

fn restr_ok_for(g: &Game, p: PlayerId, pay_for: Option<CardId>, r: Restr) -> bool {
    let spell = || pay_for.map(|c| g.db.get(c));
    match r {
        Restr::None => true,
        Restr::SpellOnly => pay_for.is_some(),
        Restr::NotNonartifact => spell().is_none_or(|d| d.types.has(crate::cards::Types::ARTIFACT)),
        Restr::ArtifactSpell => spell().is_some_and(|d| d.types.has(crate::cards::Types::ARTIFACT)),
        Restr::InstantSorcery => spell().is_some_and(|d| d.instant || d.sorcery),
        Restr::Legendary => spell().is_some_and(|d| d.tag(Tag::Leg)),
        Restr::DragonCreature => {
            spell().is_some_and(|d| d.creature && (d.has_subtype("dragon") || d.has_kw("changeling")))
        }
        Restr::Subtype(t) => spell().is_some_and(|d| d.has_subtype(t) || d.has_kw("changeling")),
        Restr::CreatureType(t) => {
            pay_for.is_some_and(|c| g.db.get(c).creature && crate::impls::galadriel::card_is(g, p, c, t))
        }
        Restr::CreatureSpell => spell().is_some_and(|d| d.creature),
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
        "Herd Heirloom" => Restr::CreatureSpell,
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
    // --- model-gaps-a3: Marvin, Murderous Mimic has Quirion Ranger's and Wirewood Symbiote's untap abilities (their
    // units, with Marvin as the source: it is Marvin that may use it once each turn)
    for &m in &g.player(p).perms {
        let x = g.perm(m);
        if x.on_bf
            && !x.phased
            && !x.neutered
            && x.cd.is_some_and(|c| &*g.db.get(c).name == crate::impls::newcards_b::MARVIN)
        {
            for donor in ["Quirion Ranger", "Wirewood Symbiote"] {
                let cd = g.player(p).perms.iter().copied().find_map(|d| {
                    let y = g.perm(d);
                    (y.on_bf && !y.phased && !y.neutered && g.is_creature(d) && y.name == donor)
                        .then_some(y.cd)
                        .flatten()
                });
                if let Some(f) = cd.and_then(|c| g.registry.get(c)).and_then(|c| c.extra_mana) {
                    let mut extra = f(g, m, p, &u);
                    extra.iter_mut().for_each(triple);
                    u.extend(extra);
                }
            }
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
            let d = g.db.get(g.perm(l).card());
            // Mana Confluence (1 life per tap) and City of Brass (1 damage per tap) hurt like a painland: tapped late
            let hurts = d.tags.has(Tag::Tomb)
                || d.tags.has(Tag::Pain)
                || matches!(&*d.name, "Mana Confluence" | "City of Brass");
            hurts && !moon_land(g, l)
        }
        Source::Perm(m) => mana_card(g, m).is_some_and(|c| g.db.get(c).tag(Tag::Pain)),
        _ => false,
    }
}

const WUBRG: [char; 5] = ['W', 'U', 'B', 'R', 'G'];

/// How many of each unit pay `generic` + `pips` (None if they can't), and which coloured pips each paid for.
/// Python's `plan_pay`, with the same tie-breaking (the first best unit wins).
/// a unit paid with only when nothing else can: a Treasure, a land sacrificed (Rain of Filth)
fn last_resort(s: Source) -> bool {
    matches!(s, Source::Treasure | Source::SacLand(_))
}

/// May a unit that is `Shape::One` or `Each`, which has paid pips `col` already, pay one more pip of colour `c`
/// (which it can make)? `One`: only the colour it is paying. `Each`: at most one of each colour per whole set, the few
/// mana beyond whole sets standing in for any of its colours.
/// `Shape::Alt(a)`: did a payment that took `n` of the unit's mana for the coloured pips `col` (and generic mana) use
/// the land's own ability (`a`, `amt` mana) and not Leyline's one mana of any colour? Pips its own ability can make
/// ({C} for a colourless one, its colour) mean its own; any other coloured pip means Leyline's. With no pips: one mana
/// is Leyline's (no damage) unless the own ability makes a colour (a free choice, the leftover floats), two or more
/// are its own.
fn alt_uses_own(a: Colors, n: u32, col: &str) -> bool {
    if col.is_empty() { n >= 2 || !a.is_empty() } else { col.chars().all(|x| x == 'C' || a.has(x)) }
}

/// May an `Alt(a)` unit, which has paid pips `col` already, pay one more pip `c`? A fresh unit pays any colour (Leyline's
/// ability: one mana), or {C} when its own ability is colourless and makes more than one; once it has paid, only its
/// own ability goes on (more of its colours, more {C}).
fn alt_fits(w: &Unit, a: Colors, col: &str, c: char) -> bool {
    let own = if c == 'C' { a.is_empty() && w.amt >= 2 } else { a.has(c) };
    if col.is_empty() { c != 'C' || own } else { own && alt_uses_own(a, 0, col) }
}

fn fits_shape(w: &Unit, col: &str, c: char) -> bool {
    match w.shape {
        Shape::Free => true,
        Shape::One => col.is_empty() || col.starts_with(c),
        Shape::Alt(a) => alt_fits(w, a, col, c),
        Shape::Each => {
            let (k, spare) = (w.sets(), w.amt - w.sets() * w.cols.count());
            let over: u32 = WUBRG
                .iter()
                .filter(|&&x| w.cols.has(x))
                .map(|&x| (col.chars().filter(|&y| y == x).count() as u32 + u32::from(x == c)).saturating_sub(k))
                .sum();
            over <= spare
        }
    }
}

pub fn plan_pay(g: &Game, u: &[Unit], generic: u32, pips: &str) -> Option<(Vec<u32>, Vec<String>)> {
    let mut rem: Vec<u32> = u.iter().map(|w| w.amt).collect();
    let mut used = vec![0u32; u.len()];
    let mut col = vec![String::new(); u.len()];
    let mut order: Vec<char> = pips.chars().collect();
    // --- thanos: a {C} pip (colourless mana, not generic: Thanos's power-up) is paid only by a colourless unit
    let fits = |w: &Unit, c: char| match w.shape {
        Shape::Alt(a) => c != 'C' || (a.is_empty() && w.amt >= 2), // Leyline's any colour, or its own {C}{C}
        _ if c == 'C' => w.cols.is_empty(),
        _ => w.cols.has(c),
    };
    order.sort_by_key(|&c| (0..u.len()).filter(|&i| fits(&u[i], c)).map(|i| rem[i]).sum::<u32>()); // stable
    for c in order {
        let mut best: Option<(usize, (bool, bool, u32, bool))> = None;
        for (i, w) in u.iter().enumerate() {
            // a unit of several mana pays only the colours it can make together (`Shape`): one colour for a Lotus, one
            // of each for a Karoo; the plain case (the common one) is just `fits`
            if rem[i] > 0 && fits(w, c) && (w.shape == Shape::Free || fits_shape(w, &col[i], c)) {
                // a Lotus already paying a colour is that colour only
                let colours = if w.shape == Shape::One && !col[i].is_empty() { 1 } else { w.cols.count() };
                // on a tie, a land's Leyline ability (one mana, the land's own is lost) goes after a plain source
                let alt_one = matches!(w.shape, Shape::Alt(a) if c != 'C' && !a.has(c));
                let k = (!restricted_float(w.src), last_resort(w.src), colours, alt_one);
                if best.is_none_or(|b| k < b.1) {
                    best = Some((i, k));
                }
            }
        }
        let (i, _) = best?;
        rem[i] -= 1;
        used[i] += 1;
        col[i].push(c);
        if let Shape::Alt(a) = u[i].shape
            && c != 'C'
            && !a.has(c)
        {
            rem[i] = 0; // Leyline's ability made one mana for this pip; the land's own ability was not used
        }
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
            // an untouched Alt land with a colourless own ability pays one generic mana with Leyline's ability: no waste, no damage
            let leyline_one = need == 1 && used[i] == 0 && matches!(w.shape, Shape::Alt(a) if a.is_empty());
            let waste = if used[i] > 0 || leyline_one { 0 } else { rem[i].saturating_sub(need) };
            let scarce = avail.iter().filter(|(c, _)| w.cols.has(*c)).map(|x| x.1).min().unwrap_or(0);
            let k = (
                !restricted_float(w.src),
                last_resort(w.src),
                is_pain(g, w.src) && !leyline_one,
                waste,
                w.cols.count(),
                -(scarce as i64),
            );
            if best.is_none_or(|b| k < b.1) {
                best = Some((i, k));
            }
        }
        let (i, _) = best?;
        let take = rem[i].min(need);
        rem[i] -= take;
        used[i] += take;
        need -= take;
        if take == 1 && used[i] == 1 && col[i].is_empty() && matches!(u[i].shape, Shape::Alt(a) if a.is_empty()) {
            rem[i] = 0; // paid with Leyline's one mana: the own {C}{C} was not made
        }
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

/// Affordability for a spell, including sources restricted to casting it and the colours its mana hooks give for it.
pub fn can_pay_spell(g: &Game, p: PlayerId, c: CardId, generic: u32, pips: &str, convoke: bool) -> bool {
    if crate::human::is_human(g, p) {
        return crate::human::mana::card_cost_problem(g, p, c, generic, pips).is_none();
    }
    let _peek = peek(c);
    plan_pay(g, &units(g, p, convoke, false, Some(c)), generic, pips).is_some()
}

/// Pay a spell's cost, restoring both payment contexts even if payment is interrupted.
pub fn pay_spell(g: &mut Game, p: PlayerId, c: CardId, generic: u32, pips: &str, convoke: bool) -> Res<bool> {
    let prev = g.pay_for.replace(c);
    let spending = g.player_mut(p).pool.spending.replace(crate::human::mana::Spending::Card(c));
    let result = pay(g, p, generic, pips, convoke);
    g.player_mut(p).pool.spending = spending;
    g.pay_for = prev;
    result
}

/// Mana usable for this spell, for X and other cast-affordability estimates.
pub fn total_mana_spell(g: &Game, p: PlayerId, c: CardId, convoke: bool) -> u32 {
    let _peek = peek(c);
    units(g, p, convoke, false, Some(c)).iter().map(|w| w.amt).sum()
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
                float_leftover(g, p, &u, &used, i, &col[i]);
            }
        }
    }
    Ok(true)
}

/// The mana a land or permanent made beyond what the payment used stays in the pool, as in the rules: Gaea's Cradle
/// tapped for four paid one {G} of Sisay's {W}{U}{B}{R}{G} and the other three were lost, so a line that was
/// affordable stopped after its first step. It empties as the step ends (rule 500.4). One colour floats as that
/// colour, none as {C}, several as any; mana of a restricted source keeps its restriction (Mishra's Workshop's
/// {C}{C} after a one-mana artifact still pays only for artifact spells). A source of several mana floats what it could
/// really make: a Lotus tapped for {U}{U}{U} floats blue, a Karoo that paid its {B} floats the {G} (`leftover_groups`).
fn float_leftover(g: &mut Game, p: PlayerId, u: &[Unit], used: &[u32], i: usize, pips: &str) {
    let w = &u[i];
    let restr = source_restr(g, w.src);
    // how much of each colour the player still has from the other units: a Lotus's free choice keeps the scarcer colour
    let elsewhere = |c: char| -> u32 {
        (0..u.len()).filter(|&j| j != i && u[j].cols.has(c)).map(|j| u[j].amt - used[j].min(u[j].amt)).sum()
    };
    for (cols, n) in leftover_groups(w, used[i], pips, elsewhere) {
        g.player_mut(p).mana.add_key(Key { restr, kind: Kind::of_leftover(cols), keep: Keep::Step }, n);
    }
}

/// What unit `w` has left once `n` of its mana paid the coloured `pips` and generic mana, as groups of mana each
/// of which can make `Colors` (a group of one colour is that colour; several colours, any of them): for a unit
/// of several mana only what it could make. `elsewhere(c)`: how much mana of colour `c` the player has from other
/// units, to choose among colours the payment left open.
fn leftover_groups(w: &Unit, n: u32, pips: &str, elsewhere: impl Fn(char) -> u32) -> Vec<(Colors, u32)> {
    let left = w.amt.saturating_sub(n);
    if left == 0 {
        return vec![];
    }
    let one = |c: char| Colors::from_letters(&c.to_string());
    match w.shape {
        Shape::Free => vec![(w.cols, left)],
        // the own ability's mana left over (a colourless one, or its colour); Leyline's one mana has none
        Shape::Alt(a) => {
            if alt_uses_own(a, n, pips) {
                vec![(a, left)]
            } else {
                vec![]
            }
        }
        Shape::One => {
            // the colour it was tapped for: the colour of the pips it paid, else the scarcest the player has elsewhere
            let c = pips
                .chars()
                .next()
                .filter(|&c| c != 'C')
                .or_else(|| WUBRG.iter().copied().filter(|&c| w.cols.has(c)).min_by_key(|&c| elsewhere(c)));
            vec![(c.map_or(Colors::NONE, one), left)]
        }
        Shape::Each => {
            let sets = w.sets();
            let mut spare = w.amt - sets * w.cols.count(); // mana beyond whole sets: any of the colours
            let mut rem: Vec<(char, u32)> =
                WUBRG.iter().copied().filter(|&c| w.cols.has(c)).map(|c| (c, sets)).collect();
            for pip in pips.chars() {
                if let Some(r) = rem.iter_mut().find(|r| r.0 == pip && r.1 > 0) {
                    r.1 -= 1;
                } else {
                    spare = spare.saturating_sub(1); // a pip past its colour's set was paid by a spare mana
                }
            }
            // generic mana took the spare first, then colours the player has most of elsewhere
            let mut generic = n.saturating_sub(pips.chars().count() as u32);
            let took = generic.min(spare);
            spare -= took;
            generic -= took;
            while generic > 0 {
                let Some(r) = rem.iter_mut().filter(|r| r.1 > 0).max_by_key(|r| elsewhere(r.0)) else { break };
                r.1 -= 1;
                generic -= 1;
            }
            let mut out: Vec<(Colors, u32)> = rem.into_iter().filter(|r| r.1 > 0).map(|r| (one(r.0), r.1)).collect();
            if spare > 0 {
                out.push((w.cols, spare));
            }
            out
        }
    }
}

/// The units left once a payment `plan_pay` planned is made (for a look ahead; `pay` floats the same mana): a unit it
/// did not touch stays as it is, one it used keeps what it could still make (`leftover_groups`).
fn units_left(u: &[Unit], used: &[u32], col: &[String]) -> Vec<Unit> {
    let mut v = Vec::with_capacity(u.len());
    for (i, w) in u.iter().enumerate() {
        if used[i] == 0 {
            v.push(*w);
            continue;
        }
        let elsewhere = |c: char| -> u32 {
            (0..u.len()).filter(|&j| j != i && u[j].cols.has(c)).map(|j| u[j].amt - used[j].min(u[j].amt)).sum()
        };
        for (cols, amt) in leftover_groups(w, used[i], &col[i], elsewhere) {
            v.push(Unit { src: w.src, cols, amt, shape: Shape::Free });
        }
    }
    v
}

/// the restriction on the mana a land or permanent makes by its own ability (none for a land under Blood Moon: a
/// plain Mountain)
fn source_restr(g: &Game, src: Source) -> Restr {
    match src {
        Source::Land(l) if !moon_land(g, l) => g.db.get(g.perm(l).card()).mana_restr,
        Source::Perm(m) => mana_card(g, m).map_or(Restr::None, |c| g.db.get(c).mana_restr),
        _ => Restr::None,
    }
}

/// Cavern of Souls' and Delighted Halfling's rider: their coloured mana (made only for the spell it may pay for)
/// makes that spell uncounterable (`Game::unc_cast`, read by `stack::counterable`)
fn mana_rider(g: &mut Game, src: Source, w: Unit) {
    let cd = match src {
        Source::Land(l) if !moon_land(g, l) => Some(g.perm(l).card()),
        Source::Perm(m) => mana_card(g, m),
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
    crate::holdstats::spent(g, p, n);
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
        Source::SacLand(l) => {
            if g.player(p).lands.contains(&l) {
                crate::glog!(g, "    {} sacrifices {} for {{B}} (Rain of Filth)", g.player(p).name, g.perm(l).name);
                crate::impls::lands::sac_land(g, p, l)?;
            }
        }
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
            // Leyline's one mana of any colour from Ancient Tomb deals no damage (`Shape::Alt`)
            let own = !matches!(w.shape, Shape::Alt(a) if !alt_uses_own(a, n, col));
            let (tomb, pain) = (!plain && own && t.has(Tag::Tomb), !plain && t.has(Tag::Pain));
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
            if !col.is_empty() && mana_card(g, m).is_some_and(|c| g.db.get(c).tag(Tag::Pain)) {
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

/// what card c costs p now: (generic, coloured pips). Python's `cost_of`: `cost_of_x` at x = 0, except a card whose X the
/// AI picks itself when it prices the cast (`CardImpl::auto_x`: Hangarback Walker, Chalice of the Void, {X}{X}): that card
/// is priced at that X, so what `cost_of` says is what the cast pays. A card without a usable X on an AI seat gets 99
/// more generic mana (uncastable) unless its cost is the same whatever X is (Tamiyo's emblem, `x_free`). A person's seat
/// is priced at x = 0: practice mode asks for X and charges it as it is cast (`human::cast_inner`).
pub fn cost_of(g: &Game, p: PlayerId, c: CardId) -> (u32, String) {
    let xcount = g.db.get(c).xcount;
    if xcount > 0 && !crate::human::is_human(g, p) {
        // (one field read for every other card; the slot is looked up only for an {X} card)
        match cardcode::auto_x(g, p, c) {
            Some(Some(x)) => return cost_of_x(g, p, c, x.max(0).saturating_mul(xcount as i32)),
            Some(None) if !x_free(g, p, c) => {
                let (gn, pips) = cost_of_x(g, p, c, 0);
                return (gn.saturating_add(99), pips);
            }
            _ => {}
        }
    }
    cost_of_x(g, p, c, 0)
}

/// does the cast of c cost the same whatever X is (Tamiyo's emblem from the hand, Dracogenesis: no mana cost)? Its X is
/// then 0 (107.3b). A price that reducers brought down to nothing at one X is not this: it differs at a large X. Probed
/// the way `brain::max_x` probes it
pub fn x_free(g: &Game, p: PlayerId, c: CardId) -> bool {
    cost_of_x(g, p, c, 0) == cost_of_x(g, p, c, 1 << 16)
}

/// what card c costs p now when cast with X = x: the printed cost with x added to the generic part, then every
/// reduction, tax, floor and minimum `cost_of` applies, in the same order (a generic reducer such as Jet Medallion
/// takes its {1} off the generic part that includes X, and off a tax; every increase and reduction is summed and the
/// generic cost floored at 0 once, see `cost_of_base`). The Trinisphere minimum comes after every reduction and tax and counts every symbol (rule 601.2f); delve, a way of
/// paying, and the choices to pay life for a Phyrexian or K'rrik {B} come after it. `cost_of` is this at x = 0 (for a
/// card whose own code picks X, see there). Never reads `CardImpl::auto_x`: it is the price at the X it is given.
/// K'rrik's life payment stays with `phyrexian_life`.
///
/// x is added to the generic part exactly once, whatever the card. A card without {X} must be given x = 0 (`cost_of`
/// does), and a card with several {X} symbols needs the caller to pass x times the symbol count. A negative x counts
/// as 0. The arithmetic is done in i64, so any i32 x is safe; the result is the true generic cost, saturating at
/// `u32::MAX`.
pub fn cost_of_x(g: &Game, p: PlayerId, c: CardId, x: i32) -> (u32, String) {
    cost_of_with(g, p, c, x, &g.db.get(c).pips)
}

/// `cost_of_x` for a cast whose coloured part is not the printed one: an alternative cost (harmonize's {X}{G}{G}{G}{G}) or
/// the printed cost with symbols added (strive's {R} for each target beyond the first). The base generic part is still the
/// printed `generic`, so any difference from it (strive's {2} per extra target, an alternative cost's own generic) must
/// be passed in x. Every step that reads the cost's symbols works on `pips`, not the printed ones: the Trinisphere
/// minimum counts them, Opposition Agent, hybrid and Morophon see them, and so do the Phyrexian and K'rrik choices (those
/// two look at the printed card's own Phyrexian symbols, `d.phyrexian`, and at its black pips in the string).
pub fn cost_of_with(g: &Game, p: PlayerId, c: CardId, x: i32, pips: &str) -> (u32, String) {
    let hybrid = &g.db.get(c).hybrid;
    if !hybrid.is_empty() && hybrid.contains('2') {
        return crate::engine::hybrid::mono_cost(g, p, c, x, pips); // {2/W}: the {2} is chosen first (engine::hybrid)
    }
    cost_of_base(g, p, c, x, pips)
}

/// `cost_of_with` for the printed symbols as they stand: the pipeline itself (`engine::hybrid::mono_cost` runs it once per
/// choice of {2} for a card with {2/X} symbols).
///
/// Rule 601.2f: the total cost is the mana cost or an alternative cost, plus every increase (the commander tax, Disruptor
/// Flute's {3}, Thalia-style taxes), minus every reduction, never below {0}; only then do effects such as Trinisphere's
/// minimum apply. So the pipeline picks one base (Tamiyo's emblem, Dracogenesis and spectacle are alternative costs of
/// {0}, {0} and {R}; otherwise the printed generic part plus X), sums every increase and reduction of the generic part
/// into ONE signed running total with no floor between steps, and floors that once at 0. A reduction larger than the
/// generic part therefore takes from the commander tax and from Disruptor Flute's {3} as from any generic mana.
///
/// Temporary shadow probe: with the environment variable `PRICE_SHADOW=1` every call also runs `cost_of_base_legacy`
/// (the previous body, which floored at 0 after each step) and prints each distinct (deck key, card, old, new) to stderr
/// once per thread, only where the two differ. The coordinator records that table; a later commit deletes the legacy body
/// and this switch. Without the variable the legacy body never runs and the switch costs one cached atomic load.
pub(crate) fn cost_of_base(g: &Game, p: PlayerId, c: CardId, x: i32, pips: &str) -> (u32, String) {
    cost_of_base_probed(g, p, c, x, pips, price_shadow_on())
}

/// `cost_of_base` with the shadow probe switched by `shadow` (the environment switch is read once, so a test passes it
/// here). The probe must not change the game: the legacy body runs FIRST and the colours `hybrid::resolve` records in
/// `Game::hybrid_paid` are put back afterwards, so the real run's hybrid choice is the new body's. Temporary, see
/// `cost_of_base`.
#[doc(hidden)]
pub fn cost_of_base_probed(g: &Game, p: PlayerId, c: CardId, x: i32, pips: &str, shadow: bool) -> (u32, String) {
    let old = shadow.then(|| {
        let kept = g.hybrid_paid.borrow().clone();
        let old = cost_of_base_legacy(g, p, c, x, pips);
        *g.hybrid_paid.borrow_mut() = kept;
        old
    });
    let new = cost_of_pipeline(g, p, c, x, pips, None);
    if let Some(old) = old
        && old != new
    {
        price_shadow_report(g, p, c, &old, &new);
    }
    new
}

/// what a card cast from the hand under Tamiyo's emblem costs when the cast also has an additional cost of `generic`
/// and `pips` (Twinflame's strive: {2}{R} for each target beyond the first). The emblem's alternative cost is {0}, the
/// additional cost is added to it, and every increase and reduction, the Trinisphere minimum (counting these symbols)
/// and the rest apply as to any cost. Only for a card cast from the hand under Tamiyo's emblem (the caller checks); it prices
/// the emblem's {0} base whatever the card's own cost is.
pub fn cost_of_free_with_extra(g: &Game, p: PlayerId, c: CardId, generic: u32, pips: &str) -> (u32, String) {
    cost_of_pipeline(g, p, c, 0, "", Some((generic as i64, pips)))
}

/// `PRICE_SHADOW=1` (read once): compare `cost_of_base` with `cost_of_base_legacy`. Temporary, see `cost_of_base`.
fn price_shadow_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PRICE_SHADOW").is_ok_and(|v| v == "1"))
}

/// print one `PRICE_SHADOW` line per distinct (deck key or seat index, card, old, new) on this thread. Temporary.
fn price_shadow_report(g: &Game, p: PlayerId, c: CardId, old: &(u32, String), new: &(u32, String)) {
    thread_local! {
        static SEEN: std::cell::RefCell<std::collections::HashSet<String>> = std::cell::RefCell::new(Default::default());
    }
    let key = g.player(p).key.to_string();
    let who = if key.is_empty() { format!("seat{}", p.0) } else { key };
    let line = format!(
        "PRICE_SHADOW deck={} card={} old=({}, {:?}) new=({}, {:?})",
        who,
        g.db.get(c).name,
        old.0,
        old.1,
        new.0,
        new.1
    );
    if SEEN.with(|s| s.borrow_mut().insert(line.clone())) {
        eprintln!("{line}");
    }
}

/// the pipeline of rule 601.2f (see `cost_of_base`): one base, one signed running total of every increase and reduction
/// of the generic part, one floor at 0, then the Trinisphere minimum, delve, Phyrexian life, K'rrik, Opposition Agent and
/// hybrid, all unchanged
///
/// `extra` is an additional cost (generic, coloured symbols) added to Tamiyo's emblem's {0} base and to nothing else
/// (`cost_of_free_with_extra`); every other caller passes `None`
fn cost_of_pipeline(g: &Game, p: PlayerId, c: CardId, x: i32, pips: &str, extra: Option<(i64, &str)>) -> (u32, String) {
    let d = g.db.get(c);
    let pl = g.player(p);
    let t = &d.tags;
    // 1. the mana cost or an alternative cost (601.2b)
    let (mut gn, mut pips): (i64, String) = if pl.emblems.contains(&"tamiyo") && pl.hand.contains(&c) {
        // Tamiyo's emblem: cast without paying its mana cost (an additional cost is still paid, 118.9d)
        extra.map_or((0, String::new()), |(n, s)| (n, s.to_string()))
    } else if crate::impls::t45_urdragon::free_dragon(g, p, c) {
        (0, String::new()) // --- Tiers 4/5 (urdragon): Dracogenesis: no mana cost (the commander tax is still paid)
    } else if t.has(Tag::Spectacle) && pl.hit_turn == pl.turns as i32 {
        // spectacle's cost (X is not added, as before). Moving this base before the card's own reductions is equivalent to
        // the old order only because no spectacle card has PerCreature, Eris or Dawning (only Light Up the Stage has spectacle)
        (0, "R".into())
    } else {
        (d.generic as i64 + x.max(0) as i64, pips.to_string())
    };
    // 2. every increase (+) and reduction (-) of the generic part, summed with no floor between them (601.2f)
    let mut delta: i64 = 0;
    if t.has(Tag::PerCreature) {
        // Blasphemous Act, Vanquish the Horde
        let n = g
            .players
            .iter()
            .filter(|q| q.alive)
            .flat_map(|q| q.perms.iter())
            .filter(|&&m| g.is_creature(m) && !g.perm(m).phased)
            .count() as i64;
        delta -= n;
    }
    if t.has(Tag::Eris) {
        // -2 per distinct mana value among instants and sorceries in the graveyard
        let mut mvs: Vec<u32> =
            pl.gy.iter().map(|&x| g.db.get(x)).filter(|x| x.instant || x.sorcery).map(|x| x.cmc).collect();
        mvs.sort();
        mvs.dedup();
        delta -= 2 * mvs.len() as i64;
    }
    if t.has(Tag::Dawning) {
        delta -= pl.gy.iter().filter(|&&x| g.db.get(x).instant || g.db.get(x).sorcery).count() as i64;
    }
    crate::impls::t45_urdragon::morophon_pips(g, p, c, &mut pips); // --- Tiers 4/5 (urdragon): Morophon (coloured)
    delta += crate::impls::t45_urdragon::eminence(g, p, c) as i64; // --- Tiers 4/5 (urdragon): The Ur-Dragon's eminence
    delta += crate::dsl::cost_delta(g, p, c) as i64; // Medallions, The Fire Crystal, compiled taxes and reductions
    delta += crate::impls::t0_stella::kaza_delta(g, p, c) as i64; // Kaza, Roil Chaser
    if !g.hooks.is_empty() {
        delta += hooks::total_cost(g, p, c) as i64;
        if crate::impls::blue_cards::defiler_discount(g, p, c) {
            pips = pips.replacen('U', "", 1); // Defiler of Dreams: pay 2 life for {U} less
        }
    }
    // a card's own reduction (Draco's domain, Emry's affinity, Shadow of Mortality...); delve is a way of paying the total,
    // not a reduction of it (702.66a), so it comes after the minimum
    let delve = crate::impls::partials::is_delve_card(&d.name);
    if !delve {
        delta += cardcode::self_cost(g, p, c) as i64;
    }
    if crate::engine::values::stopped(g, &d.name) {
        delta += 3; // Disruptor Flute tax
    }
    if pl.is_commander(c) && !pl.hand.contains(&c) && &*d.name != "Liesa, Shroud of Dusk" {
        delta += pl.tax_of(c) as i64; // no tax from hand
    }
    // 3. the one floor
    gn = (gn + delta).max(0);
    // 4. after the floor: Trinisphere's minimum, delve, Phyrexian life, K'rrik, Opposition Agent, hybrid (as before)
    if !g.hooks.is_empty() {
        // Trinisphere: the minimum applies to the total cost, every symbol counted (the pips a player may later pay with
        // life are part of it), after every reduction and tax (601.2f)
        let floor = hooks::max_min_cost(g, p, c);
        gn = gn.max(floor as i64 - pips.chars().count() as i64);
    }
    if delve {
        gn = (gn + cardcode::self_cost(g, p, c) as i64).max(0);
    }
    // Paying life for a coloured pip is decided against the generic cost as it now stands, after every reduction and tax
    // above (a reducer such as Jet Medallion may make the mana enough, so no life is paid) and before Opposition Agent
    let mut paid_life = 0;
    for col in d.phyrexian.chars() {
        // {B/P}: 2 life instead of its colour (paid by the caster), when the mana isn't there. The life check counts the
        // pips already paid with life (Python checked the starting life for every pip, so three pips at 12 life went to 6)
        if pl.life - paid_life > 10 && pips.contains(col) && !can_pay_spell(g, p, c, to_u32(gn), &pips, false) {
            pips = pips.replacen(col, "", 1);
            paid_life += 2;
        }
    }
    crate::impls::krrik::drop_black(g, p, c, gn.clamp(i32::MIN as i64, i32::MAX as i64) as i32, &mut pips); // K'rrik: 2 life for a {B}
    if pl.agent_ids.contains(&c) {
        // Opposition Agent: spend mana as though it were mana of any type (a pip turned into generic keeps the total
        // the same, so the minimum above is unaffected)
        let off: String = pips.chars().filter(|&x| !pl.ident.has(x)).collect();
        if !off.is_empty() {
            gn += off.chars().count() as i64;
            pips = pips.chars().filter(|&x| pl.ident.has(x)).collect();
        }
    }
    crate::engine::hybrid::resolve(g, p, c, to_u32(gn), &mut pips); // {G/W}: the other colour when the first can't be paid
    (to_u32(gn), pips)
}

/// The body `cost_of_base` had before the 601.2f restructure, verbatim: every step floored the generic part at 0 before the
/// next (so a reduction larger than the generic part was lost before the commander tax or Disruptor Flute was added).
/// Kept only for the `PRICE_SHADOW=1` probe (see `cost_of_base`); a later commit deletes it and the switch.
fn cost_of_base_legacy(g: &Game, p: PlayerId, c: CardId, x: i32, pips: &str) -> (u32, String) {
    let d = g.db.get(c);
    let pl = g.player(p);
    let t = &d.tags;
    let mut gn = d.generic as i64 + x.max(0) as i64;
    let mut pips: String = pips.to_string();
    if t.has(Tag::PerCreature) {
        // Blasphemous Act, Vanquish the Horde
        let n = g
            .players
            .iter()
            .filter(|q| q.alive)
            .flat_map(|q| q.perms.iter())
            .filter(|&&m| g.is_creature(m) && !g.perm(m).phased)
            .count() as i64;
        gn = (gn - n).max(0);
    }
    if t.has(Tag::Eris) {
        // -2 per distinct mana value among instants and sorceries in the graveyard
        let mut mvs: Vec<u32> =
            pl.gy.iter().map(|&x| g.db.get(x)).filter(|x| x.instant || x.sorcery).map(|x| x.cmc).collect();
        mvs.sort();
        mvs.dedup();
        gn = (gn - 2 * mvs.len() as i64).max(0);
    }
    if t.has(Tag::Dawning) {
        gn = (gn - pl.gy.iter().filter(|&&x| g.db.get(x).instant || g.db.get(x).sorcery).count() as i64).max(0);
    }
    if t.has(Tag::Spectacle) && pl.hit_turn == pl.turns as i32 {
        gn = 0;
        pips = "R".into();
    }
    crate::impls::t45_urdragon::morophon_pips(g, p, c, &mut pips); // --- Tiers 4/5 (urdragon): Morophon
    gn += crate::impls::t45_urdragon::eminence(g, p, c) as i64; // --- Tiers 4/5 (urdragon): The Ur-Dragon's eminence
    gn = (gn + crate::dsl::cost_delta(g, p, c) as i64).max(0);
    gn = (gn + crate::impls::t0_stella::kaza_delta(g, p, c) as i64).max(0); // Kaza, Roil Chaser
    if !g.hooks.is_empty() {
        gn = (gn + hooks::total_cost(g, p, c) as i64).max(0);
        if crate::impls::blue_cards::defiler_discount(g, p, c) {
            pips = pips.replacen('U', "", 1); // Defiler of Dreams: pay 2 life for {U} less
        }
    }
    // a card's own reduction (Draco's domain, Emry's affinity, Shadow of Mortality...) comes before the minimum; delve is a way
    // of paying the total, not a reduction of it (702.66a), so it comes after the minimum
    let delve = crate::impls::partials::is_delve_card(&d.name);
    if !delve {
        gn = (gn + cardcode::self_cost(g, p, c) as i64).max(0);
    }
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
        gn += pl.tax_of(c) as i64; // no tax from hand
    }
    if !g.hooks.is_empty() {
        // Trinisphere: the minimum applies to the total cost, every symbol counted (the pips a player may later pay with
        // life are part of it), after every reduction and tax (601.2f)
        let floor = hooks::max_min_cost(g, p, c);
        gn = gn.max(floor as i64 - pips.chars().count() as i64);
    }
    if delve {
        gn = (gn + cardcode::self_cost(g, p, c) as i64).max(0);
    }
    // Paying life for a coloured pip is decided against the generic cost as it now stands, after every reduction and tax
    // above (a reducer such as Jet Medallion may make the mana enough, so no life is paid) and before Opposition Agent
    let mut paid_life = 0;
    for col in d.phyrexian.chars() {
        // {B/P}: 2 life instead of its colour (paid by the caster), when the mana isn't there. The life check counts the
        // pips already paid with life (Python checked the starting life for every pip, so three pips at 12 life went to 6)
        if pl.life - paid_life > 10 && pips.contains(col) && !can_pay_spell(g, p, c, to_u32(gn), &pips, false) {
            pips = pips.replacen(col, "", 1);
            paid_life += 2;
        }
    }
    crate::impls::krrik::drop_black(g, p, c, gn.clamp(i32::MIN as i64, i32::MAX as i64) as i32, &mut pips); // K'rrik: 2 life for a {B}
    if pl.agent_ids.contains(&c) {
        // Opposition Agent: spend mana as though it were mana of any type (a pip turned into generic keeps the total
        // the same, so the minimum above is unaffected)
        let off: String = pips.chars().filter(|&x| !pl.ident.has(x)).collect();
        if !off.is_empty() {
            gn += off.chars().count() as i64;
            pips = pips.chars().filter(|&x| pl.ident.has(x)).collect();
        }
    }
    crate::engine::hybrid::resolve(g, p, c, to_u32(gn), &mut pips); // {G/W}: the other colour when the first can't be paid
    (to_u32(gn), pips)
}

/// a working generic cost as the u32 the callers pay with: 0 below zero, `u32::MAX` above
fn to_u32(gn: i64) -> u32 {
    gn.clamp(0, u32::MAX as i64) as u32
}

/// what casting these cards one after another costs in all: each one's `cost_of` now, with the Damping Sphere's {1} for
/// every spell cast before it in the line priced through the pipeline (see `seq_cost_x`). The AI checks a combo or a
/// sequence against this, not the printed costs
pub fn seq_cost(g: &Game, p: PlayerId, cards: &[CardId]) -> (u32, String) {
    seq_cost_x(g, p, &cards.iter().map(|&c| (c, 0)).collect::<Vec<_>>())
}

/// `seq_cost` with the X each card is cast for (0: `cost_of`, whatever the card's own X choice; above 0: `cost_of_x` with
/// that X, already times the card's {X} symbol count as `cost_of_x` wants it).
///
/// Damping Sphere counts the spells cast this turn, and `cost_of` sees only those already cast. When a Sphere is out,
/// every card after a spell of the line is priced on a copy of the game that has the line's earlier spells in this turn's
/// casts, so the Sphere's tax meets the reducers, floors and the Trinisphere minimum in the engine's own order (the
/// same copy `combos::casts_castable` makes). With no Sphere out nothing is copied.
pub fn seq_cost_x(g: &Game, p: PlayerId, cards: &[(CardId, i32)]) -> (u32, String) {
    let sphere = !g.hooks.is_empty()
        && hooks::hooked(g, Event::Cost)
            .iter()
            .any(|(m, _)| g.perm(*m).cd.is_some_and(|c| &*g.db.get(c).name == "Damping Sphere"));
    let price = |g: &Game, c: CardId, x: i32| if x > 0 { cost_of_x(g, p, c, x) } else { cost_of(g, p, c) };
    let (mut gn, mut pips) = (0, String::new());
    for (i, &(c, x)) in cards.iter().enumerate() {
        let before: Vec<CardId> = cards[..i].iter().map(|&(b, _)| b).filter(|&b| !g.db.get(b).land).collect();
        let (cg, cp) = if sphere && !before.is_empty() {
            let mut g2 = g.clone();
            let st = g2.turn_stamp();
            let mut done: Vec<CardId> = match &g2.player(p).turn_casts {
                Some((s, cs)) if *s == st => cs.clone(),
                _ => vec![],
            };
            done.extend(before);
            g2.player_mut(p).turn_casts = Some((st, done));
            price(&g2, c, x)
        } else {
            price(g, c, x)
        };
        gn += cg;
        pips.push_str(&cp);
    }
    (gn, pips)
}

// --- ai-fix: C: rituals in hand counted as mana
/// The mana ritual `c` adds as it resolves, as the kinds it floats (an amount of each): the colours the card prints. A
/// ritual is an instant or sorcery whose only effect adds mana and that needs no additional cost (Culling the Weak's
/// sacrifice is left out), or one of the hand-written ones: Cabal Ritual ({B}{B}{B}, five with threshold), Rite of Flame
/// (two {R}, plus one for each in a graveyard), Desperate Ritual ({R}{R}{R}). Dark Ritual's mana is black: it pays {B}
/// and generic costs, never {U}. `Kind::Any` is only what a card lets the player choose (none of these does).
pub fn ritual_mana(g: &Game, p: PlayerId, c: CardId) -> Option<Vec<(Kind, u32)>> {
    use crate::dsl::model::{Ability, Num};
    let d = g.db.get(c);
    if d.land || !(d.instant || d.sorcery) {
        return None;
    }
    match &*d.name {
        "Cabal Ritual" => return Some(vec![(Kind::B, if g.player(p).gy.len() >= 7 { 5 } else { 3 })]),
        "Rite of Flame" => {
            let copies = g.players.iter().flat_map(|q| q.gy.iter()).filter(|&&x| &*g.db.get(x).name == "Rite of Flame");
            return Some(vec![(Kind::R, 2 + copies.count() as u32)]);
        }
        "Desperate Ritual" => return Some(vec![(Kind::R, 3)]),
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
                let mut v: Vec<(Kind, u32)> = vec![];
                for e in effects {
                    let k = if let Some(Num::Int(k)) = &e.n { *k } else { 0 };
                    if k > 0 {
                        v.extend(Kind::printed(e.color.as_deref(), k as u32));
                    }
                }
                if v.is_empty() {
                    return None;
                }
                out = Some(v);
            }
            _ => {}
        }
    }
    out
}

/// The mana ritual `c` adds as it resolves: (amount, the colours it can pay as). See `ritual_mana`.
pub fn ritual_out(g: &Game, p: PlayerId, c: CardId) -> Option<(u32, Colors)> {
    let v = ritual_mana(g, p, c)?;
    let ident = g.player(p).ident;
    Some((v.iter().map(|x| x.1).sum(), v.iter().fold(Colors::NONE, |a, x| a.union(x.0.cols(ident)))))
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
    let made = ritual_mana(g, p, c)?;
    let (gn, pips) = cost_of(g, p, c);
    let (used, col) = plan_pay(g, u, gn, &pips)?;
    let mut v: Vec<Unit> = units_left(u, &used, &col);
    // each kind floats as that colour (the same units `mana_units` makes of the pool)
    let ident = g.player(p).ident;
    for (kind, k) in made {
        v.extend((0..k).map(|_| Unit::free(Source::float(kind), kind.cols(ident), 1)));
    }
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
/// adds any). None if `u` can't pay for it. A dork adds none even where haste would let it tap (The Fire Crystal, Lightning
/// Greaves): the creature is not on the battlefield yet, so `combat::tap_sick` has nothing to read, and the AI's same-turn
/// "cast a dork, then pay with it" planning stays conservative (it under-counts, never over-counts).
pub fn units_after_cast(g: &Game, p: PlayerId, c: CardId, u: &[Unit]) -> Option<Vec<Unit>> {
    if ritual_out(g, p, c).is_some() {
        return units_after_ritual(g, p, c, u);
    }
    let (gn, pips) = cost_of(g, p, c);
    let (used, col) = plan_pay(g, u, gn, &pips)?;
    let mut v: Vec<Unit> = units_left(u, &used, &col);
    let d = g.db.get(c);
    if !d.creature
        && let Some(rock) = d.tags.str(Tag::Rock)
    {
        let (a, col) = rock.split_once(':').unwrap_or((rock, "C"));
        let amt: u32 = a.parse().unwrap_or(1);
        let cols = tag_colors(col, g.player(p).ident);
        v.push(Unit { src: Source::float(Kind::Any), cols, amt, shape: shape_of(&d.tags, col, cols, amt, amt, true) });
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

/// The one colour p picks for "add N mana of any one color" (Carpet of Flowers, The Aetherspark): all N mana are that
/// colour, never split. The colour of p's identity that makes the most cards in hand (and its commander) payable that
/// can't be paid now, then the one the unpayable cards ask for most, then the first colour of the identity. A colourless
/// identity makes {C}. This is the AI's choice; a person's seat goes through `pick_one_colour`, which asks.
pub fn choose_one_colour(g: &Game, p: PlayerId, n: u32) -> Kind {
    let ident = g.player(p).ident;
    if ident.is_empty() {
        return Kind::C;
    }
    let u = mana_units(g, p, false);
    let pl = g.player(p);
    let cards: Vec<CardId> = pl.hand.iter().copied().filter(|&c| !g.db.get(c).land).chain(pl.cmds_in_zone()).collect();
    let want: Vec<(u32, String, bool)> = cards
        .iter()
        .map(|&c| {
            let (gn, pips) = cost_of(g, p, c);
            let now = plan_pay(g, &u, gn, &pips).is_some();
            (gn, pips, now)
        })
        .collect();
    let mut best: Option<(Kind, (u32, u32))> = None;
    for ch in WUBRG {
        if !ident.has(ch) {
            continue;
        }
        let k = Kind::of_letter(ch);
        let mut v = u.clone();
        v.extend((0..n.min(60)).map(|_| Unit::free(Source::float(k), k.cols(ident), 1)));
        let (mut enabled, mut asked) = (0, 0);
        for (gn, pips, now) in &want {
            if *now {
                continue;
            }
            if plan_pay(g, &v, *gn, pips).is_some() {
                enabled += 1;
            }
            asked += pips.chars().filter(|&x| x == ch).count() as u32;
        }
        if best.is_none_or(|b| (enabled, asked) > b.1) {
            best = Some((k, (enabled, asked)));
        }
    }
    best.map_or(Kind::C, |b| b.0)
}

/// The colour of "add N mana of any one color" (Carpet of Flowers, The Aetherspark, Photon) for the seat that makes it:
/// the AI's choice (`choose_one_colour`), or, in practice mode, the person's own, asked here among the colours of their
/// identity (no question when they have one colour or none). All N mana are then that colour.
pub fn pick_one_colour(g: &mut Game, p: PlayerId, n: u32) -> Res<Kind> {
    if !crate::human::is_human(g, p) {
        return Ok(choose_one_colour(g, p, n));
    }
    let ident = g.player(p).ident;
    let cols: Vec<char> = WUBRG.iter().copied().filter(|&c| ident.has(c)).collect();
    match cols.len() {
        0 => return Ok(Kind::C),
        1 => return Ok(Kind::of_letter(cols[0])),
        _ => {}
    }
    let name = |c: char| match c {
        'W' => "white",
        'U' => "blue",
        'B' => "black",
        'R' => "red",
        _ => "green",
    };
    let labels: Vec<String> = cols.iter().map(|&c| name(c).to_string()).collect();
    let prompt = format!("Add {n} mana of one colour: which colour?");
    let i = crate::human::choices::choose_one(g, p, "choose", &prompt, &labels)?;
    Ok(Kind::of_letter(cols[i.min(cols.len() - 1)]))
}

// --- ai-fix: B
/// the mana a land card would add once played, as a unit: its tagged colours (`A`: p's identity) and amount. A guess
/// for a card not yet on the battlefield: no Blood Moon, no enters-tapped rule.
pub fn land_card_unit(g: &Game, p: PlayerId, c: CardId) -> Unit {
    let t = &g.db.get(c).tags;
    let amt = t.int(Tag::Amt).unwrap_or(1).max(0) as u32;
    let letters = t.str(Tag::C).unwrap_or("C");
    let mut cols = tag_colors(letters, g.player(p).ident);
    let leyline = granted_cols_for(g, p, Colors::NONE, false);
    if amt == 1 {
        cols = cols.union(leyline); // Leyline of the Guildpact: any colour
    } else if amt >= 2 && cols.count() <= 1 && !leyline.is_empty() {
        return Unit { src: Source::float(Kind::Any), cols: leyline, amt, shape: Shape::Alt(cols) }; // `Shape::Alt`
    }
    Unit { src: Source::float(Kind::Any), cols, amt, shape: shape_of(t, letters, cols, amt, amt, true) }
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

/// The mana units of p's mana permanents that `mana_units` leaves out now but p has on its next turn: a tapped Rock,
/// Dork or Chrome Mox (it untaps) and an untapped Dork still summoning sick without haste (it will not be). Colours and amounts as
/// `units` makes them. A permanent with a special rule (Star Compass, Fellwar Stone, Bloom Tender, Delighted Halfling,
/// restricted mana, a lock, no mana from artifacts) is left out, and so is a tapped permanent that will not untap
/// (`nountap`: Mana Vault, Grim Monolith; a "frozen" lock), so the list errs towards less mana. Not handled, so it can
/// err the other way: a stun counter, Winter Orb, Static Orb and the like keeping a permanent tapped. A pure read.
pub fn next_turn_perm_units(g: &Game, p: PlayerId) -> Vec<Unit> {
    let pl = g.player(p);
    let ident = pl.ident;
    let hooked = !g.hooks.is_empty();
    let no_art_mana = hooked && hooks::total_count(g, Event::NoArtifactMana, p) != 0;
    let mut u = vec![];
    for &m in &pl.perms {
        let x = g.perm(m);
        let Some(c) = x.cd else { continue };
        if x.phased || (!g.auras.is_empty() && cardcode::locked(g, m, "noact")) {
            continue;
        }
        let d = g.db.get(c);
        let t = &d.tags;
        // the untap step's own test (engine/turn.rs): a tapped Mana Vault or a frozen permanent stays tapped
        if x.tapped && (d.tag(Tag::Nountap) || (!g.auras.is_empty() && cardcode::locked(g, m, "frozen"))) {
            continue;
        }
        if t.has(Tag::Chromemox) {
            if x.tapped && !x.colors.is_empty() {
                u.push(Unit::free(Source::Perm(m), x.colors, 1));
            }
        } else if let Some(rock) = t.str(Tag::Rock) {
            let special = matches!(&*d.name, "Star Compass" | "Fellwar Stone");
            if !x.tapped
                || special
                || d.mana_restr != Restr::None
                || (no_art_mana && d.types.has(crate::cards::Types::ARTIFACT))
            {
                continue;
            }
            let (a, col) = rock.split_once(':').unwrap_or((rock, "C"));
            let amt = cardcode::dyn_mana_perm(g, p, m).unwrap_or_else(|| a.parse().unwrap_or(1));
            let cols = tag_colors(col, ident);
            if amt > 0 {
                let shape = shape_of(t, col, cols, a.parse().unwrap_or(1), amt, true);
                u.push(Unit { src: Source::Perm(m), cols, amt, shape });
            }
        } else if let Some(dork) = t.str(Tag::Dork) {
            let counted_now = !x.tapped && !crate::engine::combat::tap_sick(g, m);
            let special = matches!(&*d.name, "Delighted Halfling" | "Bloom Tender");
            if counted_now || special || d.mana_restr != Restr::None {
                continue;
            }
            let amt = cardcode::dyn_mana_perm(g, p, m).unwrap_or(1);
            let cols = tag_colors(dork, ident);
            if amt > 0 {
                u.push(Unit { src: Source::Perm(m), cols, amt, shape: shape_of(t, dork, cols, 1, amt, false) });
            }
        }
    }
    u
}

/// --- model-gaps-a3: Marvin, Murderous Mimic has the mana abilities of the mana creatures its controller has under
/// another name ("{T}: Add {G}" ...): it taps for the best one (the most mana, then the most colours), as that
/// creature would make it (`dyn_mana_perm`: the mana of Priest of Titania counts Elves ...). None if there is none.
fn marvin_mana(g: &Game, p: PlayerId, marvin: PermId) -> Option<Unit> {
    marvin_donor(g, p, marvin).map(|b| b.0)
}

/// the unit Marvin taps for and the card whose mana ability it is (its restriction, rider and pain count)
fn marvin_donor(g: &Game, p: PlayerId, marvin: PermId) -> Option<(Unit, crate::ids::CardId)> {
    let ident = g.player(p).ident;
    let me = g.perm(marvin).name;
    let mut best: Option<(Unit, crate::ids::CardId)> = None;
    for &c in &g.player(p).perms {
        let y = g.perm(c);
        let Some(cd) = y.cd else { continue };
        if c == marvin || y.name == me || !y.on_bf || y.phased || y.neutered || !g.is_creature(c) {
            continue;
        }
        let d = g.db.get(cd);
        let Some(dork) = d.tags.str(Tag::Dork) else { continue };
        if !restr_ok(g, p, d.mana_restr) || crate::engine::values::stopped(g, &d.name) {
            continue;
        }
        let amt = cardcode::dyn_mana_perm(g, p, c).unwrap_or(1);
        let cols = if &*d.name == "Delighted Halfling" {
            cardcode::halfling_colors(g, p).unwrap_or(Colors::NONE)
        } else {
            tag_colors(dork, ident)
        };
        let better = best.as_ref().is_none_or(|b| (amt, cols.count()) > (b.0.amt, b.0.cols.count()));
        if amt > 0 && better {
            best = Some((Unit::free(Source::Perm(marvin), cols, amt), cd));
        }
    }
    best
}

/// the card whose mana ability permanent m taps for: its own, or (Marvin) the creature whose ability it uses
fn mana_card(g: &Game, m: PermId) -> Option<crate::ids::CardId> {
    let cd = g.perm(m).cd?;
    if &*g.db.get(cd).name != crate::impls::newcards_b::MARVIN {
        return Some(cd);
    }
    let p = g.players.iter().find(|q| q.perms.contains(&m))?.id;
    Some(marvin_donor(g, p, m).map_or(cd, |b| b.1))
}
