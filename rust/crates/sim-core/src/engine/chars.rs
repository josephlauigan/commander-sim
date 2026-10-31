//! The characteristics pass (Phase 3, CR 613): one place works out what a permanent is (its card types, subtypes,
//! colours, abilities, keywords, power and toughness), in the rules' layer order, and every rule reads it from here.
//! Design: `audit/phase3/chars-design.md`.
//!
//! The layers, and where each is fed from:
//! 1. copy: `Perm::cd` holds the copiable values (a copy's `cd` is the copied card), `Perm::exc` the copy's
//!    exceptions (added types, one creature type, new colours, set P/T);
//! 2. control: `Perm::owner` (the controller), not a characteristic here;
//! 3. text: nothing in the pool changes text;
//! 4. type: printed types, animation (`DataKey::Anim`), `NotCreature`, changeling, Maskwood Nexus, "all types";
//! 5. colour: the copy's colours, devoid, the card's or the token's colours;
//! 6. ability: `Perm::neutered` (abilities lost), keywords from the card, the token, Auras, card code, the ability
//!    language's statics, and the effects that remove one (Colossus Hammer);
//! 7. power and toughness: 7a characteristic-defining (Adeline), 7b set base (Wrecking Ball Arm), 7c the changes
//!    (counters, anthems, equipment, pumps, Elesh Norn's -2/-2), then the floor of 0 on power.
//!
//! Speed: these are the engine's hottest reads (`is_creature` alone has ~900 callers, power and keywords are read
//! in every combat and every evaluation). Each layer has its own reader, computed on demand with the old fast paths
//! (no Aura, no hooked card, ability language off); `chars` builds the whole struct for tests, logs and practice.
//! There is no cache: the game has no state version, and permanents are changed in place everywhere.

use crate::cards::{Colors, Types};
use crate::dsl::model::Ability;
use crate::dsl::{self, DslCtx};
use crate::engine::values::{card_tag, equipped, has, norn_vs};
use crate::ids::{PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

/// A permanent's characteristics now, every layer applied.
#[derive(Debug, Clone, PartialEq)]
pub struct Chars {
    /// card types (a token is a creature unless its card says otherwise; a copy's added types)
    pub types: Types,
    /// a creature now (animated, or not a creature any more)
    pub creature: bool,
    /// subtypes, lower case
    pub subtypes: Vec<Sym>,
    /// a creature that is every creature type (Maskwood Nexus, Mirror Entity)
    pub all_creature_types: bool,
    pub colors: Colors,
    /// its abilities are on (not lost to Tishana's Tidebinder, Darksteel Mutation, a face-down 2/2 ...)
    pub abilities_on: bool,
    /// the keywords of `KEYWORDS` it has
    pub keywords: Vec<Sym>,
    pub power: i32,
    pub toughness: i32,
}

/// The keywords the engine asks about (`chars` lists which of these a permanent has).
pub const KEYWORDS: [&str; 18] = [
    "flying",
    "haste",
    "lifelink",
    "first strike",
    "trample",
    "vigilance",
    "hexproof",
    "deathtouch",
    "reach",
    "indestructible",
    "fear",
    "double strike",
    "unblockable",
    "toxic",
    "shroud",
    "menace",
    "cant_block",
    "intimidate",
];

/// Every characteristic of m, in layer order.
pub fn chars(g: &Game, m: PermId) -> Chars {
    let creature = is_creature(g, m);
    Chars {
        types: card_types(g, m),
        creature,
        subtypes: subtypes(g, m),
        all_creature_types: creature && all_creature_types(g, m),
        colors: colors(g, m),
        abilities_on: abilities_on(g, m),
        keywords: KEYWORDS.iter().copied().filter(|k| has_keyword(g, m, k)).collect(),
        power: power(g, m),
        toughness: toughness(g, m),
    }
}

// ================================================================== layer 4: type
/// Is m a creature now (Python's `Perm.creature`): a token, a creature card, or animated.
pub fn is_creature(g: &Game, m: PermId) -> bool {
    let m = g.perm(m);
    ((m.token && m.cd.is_none_or(|c| g.db.get(c).creature)) // a Powerstone token is an artifact, not a creature
        || m.cd.is_some_and(|c| g.db.get(c).creature)
        || m.data.truthy(DataKey::Anim))
        && !m.data.truthy(DataKey::NotCreature) // --- Tiers 4/5 (chulane-kinnan): Enduring Vitality's return
}

/// m's card types, including those a copy has in addition (Copy Artifact's enchantment); a token has creature
pub fn card_types(g: &Game, m: PermId) -> Types {
    let x = g.perm(m);
    let base = x.cd.map_or(Types::CREATURE, |c| g.db.get(c).types);
    Types::union(base, x.exc.add_types)
}

pub const ALL_TYPES: [&str; 19] = [
    "human",
    "warrior",
    "shaman",
    "wizard",
    "elf",
    "goblin",
    "ninja",
    "rogue",
    "zombie",
    "angel",
    "demon",
    "dragon",
    "soldier",
    "knight",
    "cleric",
    "faerie",
    "elemental",
    "druid",
    "spirit",
];
pub const NONCREATURE_TYPES: [&str; 9] =
    ["aura", "equipment", "vehicle", "saga", "treasure", "food", "clue", "curse", "shrine"];

/// m's subtypes (lower case): printed for Scryfall-built cards, set on tokens by their maker (Python's `subtypes`)
pub fn subtypes(g: &Game, m: PermId) -> Vec<Sym> {
    let x = g.perm(m);
    if let Some(s) = x.exc.only_subtype {
        return vec![s]; // a copy that has only this creature type (Preston's Illusions)
    }
    let Some(c) = x.cd else { return x.ttypes.clone() };
    let d = g.db.get(c);
    let mut s: Vec<Sym> = d.subtypes.iter().map(|t| crate::sym::intern(t)).collect();
    if d.has_kw("changeling") {
        s.extend(ALL_TYPES.iter().copied());
    }
    if s.is_empty() {
        for (t, k) in
            [(Tag::Human, "human"), (Tag::Warrior, "warrior"), (Tag::Shaman, "shaman"), (Tag::Wizard, "wizard")]
        {
            if d.tag(t) {
                s.push(k);
            }
        }
    }
    for t in &x.ttypes {
        if !s.contains(t) {
            s.push(t);
        }
    }
    s
}

/// Maskwood Nexus: p's creatures are every creature type
pub fn maskwood(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| !g.perm(m).phased && is_fx(g, m, NamedFx::AllCreatureTypes))
}

/// m (a creature) is every creature type: Maskwood Nexus, or Mirror Entity's "all types" this turn
fn all_creature_types(g: &Game, m: PermId) -> bool {
    maskwood(g, g.perm(m).owner) || g.perm(m).eot_kw.contains(&"all types")
}

/// m has subtype t (Mirror Entity and Maskwood Nexus make creatures every type)
pub fn has_subtype(g: &Game, m: PermId, t: &str) -> bool {
    if subtypes(g, m).contains(&t) {
        return true;
    }
    is_creature(g, m) && !NONCREATURE_TYPES.contains(&t) && all_creature_types(g, m)
}

// ================================================================== layer 5: colour
/// a permanent's colours (cards: coloured mana symbols in the cost; tokens: their token colour)
pub fn colors(g: &Game, m: PermId) -> Colors {
    let x = g.perm(m);
    if let Some(c) = x.exc.colors {
        return c;
    }
    match x.cd {
        Some(c) if g.db.get(c).kws.iter().any(|k| &**k == "devoid") => Colors::NONE, // Spawn-Gang Commander
        Some(c) => g.db.get(c).colors,
        None => x.colors,
    }
}

// ================================================================== layer 6: ability
/// m's abilities are on (not lost to an effect that removes them)
pub fn abilities_on(g: &Game, m: PermId) -> bool {
    !g.perm(m).neutered
}

/// Does m have keyword kw (lower case)? Python's `dsl.has_kw`.
pub fn has_keyword(g: &Game, m: PermId, kw: &str) -> bool {
    let x = g.perm(m);
    if loses_keyword(g, m, kw) {
        return false; // Colossus Hammer
    }
    if x.eot_kw.contains(&kw) {
        return true;
    }
    if kw == "trample" && x.data.get(DataKey::Role) == Some(&Val::Bool(true)) {
        return true; // a Monster Role
    }
    if matches!(kw, "menace" | "cant_block") && x.data.truthy(DataKey::Suspect) {
        return true; // suspected
    }
    if kw == "flying" && is_creature(g, m) && g.player(x.owner).elspeth_emblem {
        return true; // Elspeth's emblem
    }
    match x.cd {
        None => {
            if let Some(Val::List(v)) = x.data.get(DataKey::Kws)
                && v.iter().any(|k| matches!(k, Val::Str(s) if *s == kw))
            {
                return true; // a token made with keywords (Samurai)
            }
        }
        Some(c) => {
            if !x.neutered && g.db.get(c).has_kw(kw) {
                return true;
            }
        }
    }
    if !g.auras.is_empty() && crate::cardcode::attached_kw(g, m, kw) {
        return true;
    }
    if !g.hooks.is_empty() && crate::cardcode::granted_kw(g, m, kw) {
        return true;
    }
    if !dsl::active(g) {
        return false;
    }
    if let Some(cd) = x.cd.filter(|_| !x.neutered) {
        for a in g.db.get(cd).abilities.iter() {
            if let Ability::Static { kind, keyword, .. } = a {
                match kind.as_str() {
                    "unblockable" if kw == "unblockable" => return true,
                    "cant_block" if kw == "cant_block" => return true,
                    "self_keyword" if keyword.as_deref() == Some(kw) => return true,
                    _ => {}
                }
            }
        }
    }
    let cre = is_creature(g, m);
    for (q, src, ab, i) in dsl::statics(g, "keyword") {
        if let Ability::Static { keyword, filter, .. } = &ab[i]
            && keyword.as_deref() == Some(kw)
            && !g.perm(src).neutered // layer 6: a source that lost its abilities grants nothing
            && (cre || dsl::noncreature_ok(filter.as_ref())) // Tier 3 (aurelia): "permanents you control have" (Avacyn)
            && dsl::matches(g, q, m, filter.as_ref(), Some(src))
        {
            return true;
        }
    }
    for (_, src, ab, i) in dsl::statics(g, "equip_keyword") {
        if let Ability::Static { keyword, .. } = &ab[i]
            && keyword.as_deref() == Some(kw)
            && !g.perm(src).neutered
            && g.perm(src).attached == Some(m)
        {
            return true;
        }
    }
    false
}

// ================================================================== layer 7: power and toughness
/// m's power now (Python's `epow`)
pub fn power(g: &Game, m: PermId) -> i32 {
    let x = g.perm(m);
    let p = x.owner;
    let mut v = x.pow + x.plus + static_bonus(g, m);
    if card_tag(g, m, Tag::Adeline) && equipped_base(g, m).is_none() {
        // 7a (Adeline's power) is overridden by a 7b base from an Equipment (Wrecking Ball Arm)
        let n = g.player(p).perms.iter().filter(|&&o| is_creature(g, o) && !g.perm(o).phased).count() as i32;
        v = n + x.plus + static_bonus(g, m);
    }
    if equipped(g, m, Tag::Sword) {
        v += 2;
    }
    if norn_vs(g, p) {
        v -= 2;
    }
    v = v.saturating_add(pt_changes(g, m).0); // --- urdragon: Thrakkus doublings can pass i32
    v -= crate::impls::blue_cards::weakened(g, m); // Mouth of the Storm
    let pl = g.player(p);
    let cre = is_creature(g, m);
    if pl.pump != 0 && cre {
        v = v.max(pl.pump);
    }
    if cre {
        v += pl.pumpadd;
    }
    v.max(0)
}

/// m's toughness now (Python's `etgh`)
pub fn toughness(g: &Game, m: PermId) -> i32 {
    let x = g.perm(m);
    let mut v = x.tgh + x.plus + static_bonus(g, m);
    v += pt_changes(g, m).1;
    if equipped(g, m, Tag::Sword) {
        v += 2;
    }
    if norn_vs(g, x.owner) {
        v -= 2;
    }
    v
}

/// anthems and equipment that change a creature's size (the tagged ones: card code's and auto-tagged)
pub fn static_bonus(g: &Game, m: PermId) -> i32 {
    if !is_creature(g, m) {
        return 0;
    }
    let x = g.perm(m);
    let p = x.owner;
    let tagged = |t: Tag| card_tag(g, m, t);
    let mut b = 0;
    if x.data.truthy(DataKey::Role) {
        b += 1; // a Monster Role
    }
    if has(g, p, Tag::Anthem2) && !tagged(Tag::Anthem2) {
        b += 2; // Elesh Norn, Grand Cenobite
    }
    if has(g, p, Tag::Anthemnh) && !tagged(Tag::Anthemnh) && !tagged(Tag::Human) {
        b += 1; // Mikaeus
    }
    if has(g, p, Tag::Warleader) {
        b += 1; // Warleader's Call
    }
    if x.orig != p && has(g, p, Tag::Garland) {
        b += 2; // Garland: creatures you don't own
    }
    for &o in &g.player(p).perms {
        let y = g.perm(o);
        if o != m
            && !y.phased
            && !y.neutered // layer 6: an anthem that lost its abilities
            && let Some(c) = y.cd
            && let Some(a) = g.db.get(c).tags.int(Tag::Anth)
        {
            // auto-tagged anthems (Commander's Insignia: +1/+1 for each time the commander was cast from the command zone)
            b += if g.db.get(c).named == Some(NamedFx::PerCommandCast) {
                crate::impls::t0_zurgo::command_casts(g, p)
            } else {
                a
            };
        }
    }
    if equipped(g, m, Tag::Flail) {
        b += crate::impls::tymna_cards::colors_among(g, p).count() as i32; // Conqueror's Flail: +1/+1 per colour among your permanents
    }
    if equipped(g, m, Tag::Animist) {
        b += 1;
    }
    if equipped(g, m, Tag::Nim) {
        b += 2; // Nim Deathmantle
    }
    if equipped(g, m, Tag::Helm) {
        b += 2; // Champion's Helm
    }
    b
}

/// Power and toughness changes beyond printed stats and counters: until-end-of-turn pumps, Auras (card code), then
/// the ability language's anthems, equipment bonuses and self-scaling creatures. Python's `dsl.pt`.
pub fn pt_changes(g: &Game, m: PermId) -> (i32, i32) {
    let x = g.perm(m);
    let (mut dp, mut dt) = x.eot_pt;
    if !g.auras.is_empty() || g.selfpt || g.player(x.owner).elspeth_emblem {
        let (a, b) = crate::cardcode::attached_bonus(g, m);
        dp += a;
        dt += b;
    }
    if !dsl::active(g) {
        return (dp, dt);
    }
    let cre = is_creature(g, m);
    // one pass over the statics of the three kinds that change power and toughness (anthem, equip_bonus,
    // bma_anthem); the sums do not depend on the order
    let owner = x.owner;
    for q in g.players.iter().filter(|q| q.alive) {
        for src in q.perms.iter().copied().chain(dsl::land_sources(g, q.id)) {
            let s = g.perm(src);
            let Some(cd) = s.cd else { continue };
            if s.phased || s.owner != q.id || s.neutered {
                continue;
            }
            for a in g.db.get(cd).abilities.iter() {
                let Ability::Static { kind, pow, tgh, filter, per, .. } = a else { continue };
                match kind.as_str() {
                    "anthem" => {
                        if cre && dsl::matches(g, q.id, m, filter.as_ref(), Some(src)) {
                            // --- Tier 3 (marwyn): "+1/+1 for each +1/+1 counter on ~" (Joraga Warcaller)
                            let k = if per.is_some() {
                                dsl::num(g, q.id, per.as_ref(), &DslCtx::base(), Some(src))
                            } else {
                                1
                            };
                            dp += pow.unwrap_or(0) * k;
                            dt += tgh.unwrap_or(0) * k;
                        }
                    }
                    "equip_bonus" => {
                        if s.attached == Some(m) {
                            dp += pow.unwrap_or(0);
                            dt += tgh.unwrap_or(0);
                        }
                    }
                    "bma_anthem" => {
                        if s.plus >= 7 && cre && owner == q.id {
                            dp += 5; // Beastmaster Ascension with 7+ quest counters
                            dt += 5;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    if let Some((bp, bt)) = equipped_base(g, m) {
        dp += bp - x.pow; // Wrecking Ball Arm, Hulkbuster Armor: base power and toughness
        dt += bt - x.tgh;
    }
    let shaped = equipped_lords(g, m); // Auriok Steelshaper
    dp += shaped;
    dt += shaped;
    if let Some(cd) = x.cd.filter(|_| !x.neutered) {
        for a in g.db.get(cd).abilities.iter() {
            if let Ability::Static { kind, per, pow, tgh, base0, .. } = a
                && kind == "self_scaling"
            {
                let k = dsl::num(g, x.owner, per.as_ref(), &DslCtx::base(), Some(m));
                dp += pow.unwrap_or(1) * k;
                dt += tgh.unwrap_or(0) * k;
                if *base0 == Some(true) {
                    dt += k;
                }
            }
        }
    }
    (dp, dt)
}

// ================================================================== the named effects (one table)
/// What a card's presence does to other permanents' characteristics, beyond its compiled abilities and tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedFx {
    /// layer 4: its controller's creatures are every creature type (Maskwood Nexus)
    AllCreatureTypes,
    /// layer 4: every land is this basic land type too (WUBRG letter: Yavimaya G, Urborg B)
    LandsAre(char),
    /// layer 6: the creature it equips loses this keyword (Colossus Hammer: flying)
    EquippedLoses(&'static str),
    /// layer 7b: the creature it equips has this base power and toughness (Wrecking Ball Arm, Hulkbuster Armor)
    EquippedBase(i32, i32),
    /// layer 7c: its auto-tagged anthem is +1/+1 per time its controller cast the commander from the command zone
    /// (Commander's Insignia)
    PerCommandCast,
    /// layer 7c: while it is equipped, its controller's Soldiers and Knights get +1/+1 (Auriok Steelshaper)
    EquippedLord,
}

/// A card name, the layer its effect applies in, and the effect.
#[derive(Debug, Clone, Copy)]
pub struct Named {
    pub name: &'static str,
    pub layer: &'static str,
    pub fx: NamedFx,
}

/// The cards whose characteristic-changing effect is read by name (each was a name check at its use site).
pub const NAMED: [Named; 8] = [
    Named { name: "Maskwood Nexus", layer: "4", fx: NamedFx::AllCreatureTypes },
    Named { name: "Yavimaya, Cradle of Growth", layer: "4", fx: NamedFx::LandsAre('G') },
    Named { name: "Urborg, Tomb of Yawgmoth", layer: "4", fx: NamedFx::LandsAre('B') },
    Named { name: "Colossus Hammer", layer: "6", fx: NamedFx::EquippedLoses("flying") },
    Named { name: "Wrecking Ball Arm", layer: "7b", fx: NamedFx::EquippedBase(7, 7) },
    Named { name: "Hulkbuster Armor", layer: "7b", fx: NamedFx::EquippedBase(9, 9) },
    Named { name: "Commander's Insignia", layer: "7c", fx: NamedFx::PerCommandCast },
    Named { name: "Auriok Steelshaper", layer: "7c", fx: NamedFx::EquippedLord },
];

/// the named effect of a card, if it has one
pub fn named_fx(name: &str) -> Option<NamedFx> {
    NAMED.iter().find(|n| n.name == name).map(|n| n.fx)
}

/// permanent m's named effect is `fx`
fn is_fx(g: &Game, m: PermId, fx: NamedFx) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).named == Some(fx))
}

/// the named effects of the Equipment attached to m (among its controller's permanents)
fn equipment_fx(g: &Game, m: PermId) -> impl Iterator<Item = NamedFx> + '_ {
    g.player(g.perm(m).owner).perms.iter().filter_map(move |&e| {
        let x = g.perm(e);
        if x.attached != Some(m) || !x.on_bf || x.neutered || !crate::impls::cloud::is_equipment(g, e) {
            return None;
        }
        g.db.get(x.cd?).named
    })
}

/// some player's land makes every land this basic type (Yavimaya, Urborg); not one that is a plain Mountain under
/// Blood Moon: it has lost that ability (layer 4 depends on layer 6 here)
pub fn lands_are(g: &Game, ch: char) -> bool {
    g.players
        .iter()
        .filter(|q| q.alive)
        .any(|q| q.lands.iter().any(|&l| is_fx(g, l, NamedFx::LandsAre(ch)) && !crate::engine::mana::moon_land(g, l)))
}

/// layer 4: land l's basic land types (WUBRG letters: Plains, Island, Swamp, Mountain, Forest). Blood Moon makes a
/// nonbasic land a Mountain only (its printed types go); Yavimaya and Urborg add Forest and Swamp to every land.
pub fn land_types(g: &Game, l: PermId) -> Colors {
    if crate::engine::mana::moon_land(g, l) {
        return Colors::from_letters("R");
    }
    let mut t = g.perm(l).cd.map_or(Colors::NONE, |c| g.db.get(c).land_types);
    for (ch, s) in [('G', "G"), ('B', "B")] {
        if lands_are(g, ch) {
            t = t.union(Colors::from_letters(s));
        }
    }
    t
}

/// m loses keyword kw to an Equipment on it (layer 6: Colossus Hammer)
pub fn loses_keyword(g: &Game, m: PermId, kw: &str) -> bool {
    NAMED.iter().any(|n| matches!(n.fx, NamedFx::EquippedLoses(k) if k == kw))
        && equipment_fx(g, m).any(|f| matches!(f, NamedFx::EquippedLoses(k) if k == kw))
}

/// the base power and toughness an Equipment sets on m (layer 7b: Wrecking Ball Arm 7/7, Hulkbuster Armor 9/9), the
/// largest of those on it
pub fn equipped_base(g: &Game, m: PermId) -> Option<(i32, i32)> {
    equipment_fx(g, m)
        .filter_map(|f| match f {
            NamedFx::EquippedBase(p, t) => Some((p, t)),
            _ => None,
        })
        .max()
}

/// layer 7c, Auriok Steelshaper: as long as it is equipped, each Soldier and Knight you control gets +1/+1
fn equipped_lords(g: &Game, m: PermId) -> i32 {
    // count the equipped Steelshapers first (almost always none, a name compare per permanent); only then pay for the
    // subtype lookup of m
    let pl = g.player(g.perm(m).owner);
    let n = pl
        .perms
        .iter()
        .filter(|&&s| {
            is_fx(g, s, NamedFx::EquippedLord)
                && g.perm(s).on_bf
                && !g.perm(s).phased
                && !crate::impls::cloud::equipment_on(g, s).is_empty()
        })
        .count() as i32;
    if n == 0 || !(has_subtype(g, m, "soldier") || has_subtype(g, m, "knight")) {
        return 0;
    }
    n
}
