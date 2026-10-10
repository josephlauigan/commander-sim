//! Board reading: what a permanent is (power, toughness, types, keywords, protection) and what it's worth.
//! engine.py's "helpers" and "values / threat" sections, in the same order.

use crate::cardcode;
use crate::cards::Colors;
use crate::dsl;
use crate::ids::{PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

// ------------------------------------------------------------------ tags on the battlefield
/// p controls a permanent with tag t (not phased out, abilities on): Python's `has`
pub fn has(g: &Game, p: PlayerId, t: Tag) -> bool {
    g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        !x.phased && !x.neutered && x.cd.is_some_and(|c| g.db.get(c).tag(t))
    })
}

/// p's permanents with tag t (not phased out): Python's `find`
pub fn find(g: &Game, p: PlayerId, t: Tag) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            !x.phased && x.cd.is_some_and(|c| g.db.get(c).tag(t))
        })
        .collect()
}

pub fn opp_has(g: &Game, p: PlayerId, t: Tag) -> bool {
    g.opps(p).any(|q| has(g, q, t))
}

/// p's Orc Army, if any
pub fn army_of(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).army && !g.perm(m).phased)
}

/// an Equipment with tag t is attached to m (among its controller's permanents)
pub fn equipped(g: &Game, m: PermId, t: Tag) -> bool {
    let owner = g.perm(m).owner;
    g.player(owner).perms.iter().any(|&e| {
        let x = g.perm(e);
        x.attached == Some(m) && x.cd.is_some_and(|c| g.db.get(c).tag(t))
    })
}

/// an opponent has Elesh Norn, Grand Cenobite (your creatures get -2/-2)
pub fn norn_vs(g: &Game, p: PlayerId) -> bool {
    opp_has(g, p, Tag::Normgc)
}

pub fn card_tag(g: &Game, m: PermId, t: Tag) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(t))
}

pub fn card_name(g: &Game, m: PermId) -> Option<&str> {
    g.perm(m).cd.map(|c| &*g.db.get(c).name)
}

// ------------------------------------------------------------------ power and toughness
/// anthems and equipment that change a creature's size
pub fn static_bonus(g: &Game, m: PermId) -> i32 {
    if !g.is_creature(m) {
        return 0;
    }
    let x = g.perm(m);
    let p = x.owner;
    let tagged = |t: Tag| card_tag(g, m, t);
    let mut b = 0;
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
            && let Some(c) = y.cd
            && let Some(a) = g.db.get(c).tags.int(Tag::Anth)
        {
            b += a; // auto-tagged anthems
        }
    }
    if equipped(g, m, Tag::Flail) {
        b += 3; // Conqueror's Flail (~3 colors)
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

/// m's power now (Python's `epow`)
pub fn epow(g: &Game, m: PermId) -> i32 {
    let x = g.perm(m);
    let p = x.owner;
    let mut v = x.pow + x.plus + static_bonus(g, m);
    if card_tag(g, m, Tag::Adeline) {
        let n = g.player(p).perms.iter().filter(|&&o| g.is_creature(o) && !g.perm(o).phased).count() as i32;
        v = n + x.plus + static_bonus(g, m);
    }
    if equipped(g, m, Tag::Sword) {
        v += 2;
    }
    if norn_vs(g, p) {
        v -= 2;
    }
    v += dsl::pt(g, m).0;
    let pl = g.player(p);
    let cre = g.is_creature(m);
    if pl.pump != 0 && cre {
        v = v.max(pl.pump);
    }
    if cre {
        v += pl.pumpadd;
    }
    v.max(0)
}

/// m's toughness now (Python's `etgh`)
pub fn etgh(g: &Game, m: PermId) -> i32 {
    let x = g.perm(m);
    let mut v = x.tgh + x.plus + static_bonus(g, m);
    v += dsl::pt(g, m).1;
    if equipped(g, m, Tag::Sword) {
        v += 2;
    }
    if norn_vs(g, x.owner) {
        v -= 2;
    }
    v
}

pub fn board_power(g: &Game, p: PlayerId) -> i32 {
    g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).map(|&m| epow(g, m)).sum()
}

// ------------------------------------------------------------------ types
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
    g.player(p).perms.iter().any(|&m| !g.perm(m).phased && card_name(g, m) == Some("Maskwood Nexus"))
}

/// m has subtype t (Mirror Entity and Maskwood Nexus make creatures every type)
pub fn has_type(g: &Game, m: PermId, t: &str) -> bool {
    if subtypes(g, m).contains(&t) {
        return true;
    }
    g.is_creature(m)
        && !NONCREATURE_TYPES.contains(&t)
        && (maskwood(g, g.perm(m).owner) || g.perm(m).eot_kw.contains(&"all types"))
}

// ------------------------------------------------------------------ protection and targeting
/// all damage that would be dealt to m is prevented (Cho-Manno, Revolutionary)
pub fn no_damage(g: &Game, m: PermId) -> bool {
    card_tag(g, m, Tag::Nodmg) && !g.perm(m).neutered
}

/// Shadowspear's activation this turn, against a permanent of a player other than the active one
fn spear_on(g: &Game, m: PermId) -> bool {
    g.spear == Some(g.turn_stamp()) && g.active != Some(g.perm(m).owner)
}

pub fn indestructible(g: &Game, m: PermId) -> bool {
    if spear_on(g, m) {
        return false; // Shadowspear
    }
    let x = g.perm(m);
    if x.data.truthy(DataKey::Indestr) {
        return true;
    }
    // (player, their turn count): until that player's next turn
    if let Some(Val::List(v)) = x.data.get(DataKey::IndestrUntil)
        && let [Val::Player(q), Val::Int(t)] = v.as_slice()
        && g.player(*q).turns as i64 == *t
    {
        return true;
    }
    dsl::has_kw(g, m, "indestructible")
}

/// p's legendary creatures and artifacts (Sauron's ward: sacrifice one)
pub fn ward_legends(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            !x.phased
                && x.cd.is_some_and(|c| {
                    let d = g.db.get(c);
                    d.tag(Tag::Leg) && (g.is_creature(m) || d.types.has(crate::cards::Types::ARTIFACT))
                })
        })
        .collect()
}

/// Garland, Royal Kidnapper: creatures you control but don't own can't be sacrificed
pub fn no_sac(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.orig != x.owner && g.is_creature(m) && has(g, x.owner, Tag::Garland)
}

pub fn untargetable(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    if x.phased {
        return true;
    }
    if (dsl::has_kw(g, m, "hexproof") && !spear_on(g, m)) || dsl::has_kw(g, m, "shroud") {
        return true;
    }
    if equipped(g, m, Tag::Helm) && cardcode::is_legendary(g, m) {
        return true; // Champion's Helm
    }
    if equipped(g, m, Tag::Cloak) {
        return true;
    }
    g.player(x.owner).perms.iter().any(|&e| {
        let y = g.perm(e);
        y.attached == Some(m)
            && y.cd.is_some_and(|c| {
                let t = &g.db.get(c).tags;
                t.str(Tag::Prot) == Some("boots") || t.has(Tag::Spider)
            })
    })
}

/// q can't be the target of opponents' spells and abilities (Shalai, Voice of Plenty)
pub fn player_hexproof(g: &Game, q: PlayerId) -> bool {
    crate::engine::hooks::any_player_hexproof(g, q)
}

/// a permanent's colours (cards: coloured mana symbols in the cost; tokens: their token colour)
pub fn colors_of(g: &Game, m: PermId) -> Colors {
    let x = g.perm(m);
    match x.cd {
        Some(c) => g.db.get(c).colors,
        None => x.colors,
    }
}

/// colours m has protection from: Sword of Feast and Famine (B, G), printed protection, equipment and Auras
pub fn prot_colors(g: &Game, m: PermId) -> Colors {
    let mut s = Colors::NONE;
    if equipped(g, m, Tag::Sword) {
        s = s.union(Colors::from_letters("BG"));
    }
    s = s.union(dsl::protection(g, m));
    if let Some(c) = g.perm(m).cd {
        s = s.union(g.db.get(c).protfrom);
    }
    if !g.auras.is_empty() {
        s = s.union(cardcode::attached_prot(g, m));
    }
    s
}

pub fn protected_from(g: &Game, m: PermId, source: Colors) -> bool {
    prot_colors(g, m).intersects(source)
}

/// Disruptor Flute: is a Flute on the battlefield naming this card?
pub fn stopped(g: &Game, name: &str) -> bool {
    g.flutes.iter().any(|&(f, c)| {
        let x = g.perm(f);
        &*g.db.get(c).name == name && x.on_bf && !x.phased
    })
}

/// Conqueror's Flail: while attached, opponents can't cast spells during its controller's turn
pub fn silenced(g: &Game, q: PlayerId) -> bool {
    let Some(a) = g.active else { return false };
    if a == q || !g.player(a).alive {
        return false;
    }
    g.player(a).perms.iter().any(|&e| {
        let x = g.perm(e);
        card_tag(g, e, Tag::Flail) && x.attached.is_some_and(|t| g.perm(t).on_bf && g.perm(t).owner == a)
    })
}

/// true the first time p asks for `key` this turn (Python's `once_per_turn`)
pub fn once_per_turn(g: &mut Game, p: PlayerId, key: Sym) -> bool {
    let stamp = g.turn_stamp();
    let pl = g.player_mut(p);
    if pl.flag_turn.get(key) == Some(&stamp) {
        return false;
    }
    pl.flag_turn.insert(key, stamp);
    true
}

pub fn commander_out(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).is_cmd && !g.perm(m).phased)
}

/// Melira, Sylvok Outcast: p can't get poison counters, and p's creatures can't have -1/-1 counters put on them
pub fn melira(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| card_tag(g, m, Tag::Melira) && !g.perm(m).phased)
}

/// Glacial Chasm
pub fn chasm(g: &Game, p: PlayerId) -> bool {
    g.player(p).lands.iter().any(|&l| g.db.get(g.land(l).cd).tag(Tag::Chasm))
}

/// damage to p from an opponent would be prevented (attack and burn AIs look elsewhere)
pub fn shielded(g: &Game, p: PlayerId) -> bool {
    g.player(p).ring_prot || chasm(g, p)
}

// ------------------------------------------------------------------ what things are worth
/// how much m matters (Python's `pval`): its bomb rating or size, raised for engines and combo pieces
pub fn pval(g: &Game, m: PermId) -> f64 {
    let x = g.perm(m);
    let p = x.owner;
    if x.army {
        let mut v = 2.0 + x.plus as f64 * 0.5;
        if equipped(g, m, Tag::Sword) && has(g, p, Tag::Assault) {
            v += 5.0;
        }
        return v;
    }
    let Some(c) = x.cd else { return 0.3 + epow(g, m) as f64 * 0.25 };
    let cd = g.db.get(c);
    let t = &cd.tags;
    let mut v = cd.bomb as f64;
    if cd.creature && v == 0.0 {
        v = 1.0 + epow(g, m) as f64 * 0.5;
    }
    // a combo half is worth more with its other half out
    let with = |other: Tag, base: f64| if has(g, p, other) { base + 5.0 } else { base };
    if t.has(Tag::Vkitten) {
        v = with(Tag::Vfire, 5.0);
    }
    if t.has(Tag::Vfire) {
        v = with(Tag::Vkitten, 4.0);
    }
    if t.has(Tag::Aether) {
        v = 5.0;
    }
    if t.has(Tag::Ping) {
        v = v.max(4.0);
    }
    if t.has(Tag::Veyran) {
        v = 5.0;
    }
    if t.has(Tag::Dragoncaller) {
        v = 5.0;
    }
    if t.has(Tag::Sauron) {
        v = 7.0;
    }
    if t.has(Tag::Witchking) {
        v = 5.0;
    }
    if t.has(Tag::Rhystic) || t.has(Tag::Tithe) {
        v = 4.0;
    }
    if t.has(Tag::Assault) {
        v = with(Tag::Sword, 3.0);
    }
    if t.has(Tag::Sword) {
        v = with(Tag::Assault, 3.0);
    }
    if t.has(Tag::Cloak) {
        v = 3.0;
    }
    if t.has(Tag::Crusade) {
        v = 6.0;
    }
    if t.has(Tag::Warleader) {
        v = 4.0;
    }
    if t.has(Tag::Mirror) {
        v = 5.0;
    }
    if t.has(Tag::Tokup) || t.has(Tag::Tokatk) {
        v = v.max(3.0);
    }
    if t.has(Tag::Normgc) && g.opps(p).any(|q| g.player(q).key == "najeela") {
        v += 2.0;
    }
    if t.has(Tag::Mikaeus) {
        v = 5.0;
    }
    if t.has(Tag::Eng) {
        v = v.max(2.0);
    }
    if t.has(Tag::Onering) {
        v = v.max(5.0);
    }
    if t.has(Tag::Sphinx) {
        v = v.max(6.0);
    }
    if t.has(Tag::Narset) {
        v = v.max(4.0);
    }
    if t.has(Tag::Panoptic) {
        v = v.max(2.0 + 2.0 * g.imprint.iter().filter(|(pm, _)| *pm == m).count() as f64);
    }
    if x.is_cmd {
        v += 1.0;
    }
    if !g.auras.is_empty() && cd.creature {
        v += 1.5 * cardcode::own_auras_on(g, m) as f64; // removing it takes the Auras too
    }
    if cd.types.has(crate::cards::Types::PLANESWALKER) {
        let loy = x.loyalty.unwrap_or(0) as f64;
        v = v.max(3.0 + 0.4 * loy + if x.is_cmd { 2.0 } else { 0.0 } + 5.0 * cardcode::ult_pressure(g, m).min(1.2));
    }
    v = v.max(cardcode::threat_value(g, m));
    if x.is_cmd {
        v += 1.0; // commanders matter more at a real table
    }
    if !g.auras.is_empty() {
        v *= cardcode::lock_factor(g, m); // Arrest, Kasmina's Transmutation ...
    }
    v
}

/// how threatening q's board is to `me` (Python's `threat`)
pub fn threat(g: &Game, _me: PlayerId, q: PlayerId) -> f64 {
    let pl = g.player(q);
    let mut s: f64 = pl.perms.iter().filter(|&&m| !g.perm(m).phased).map(|&m| pval(g, m)).sum();
    if pl.key == "veyran" && has(g, q, Tag::Vkitten) && has(g, q, Tag::Vfire) {
        s += 10.0;
    }
    if pl.key == "sauron"
        && let Some(a) = army_of(g, q)
        && equipped(g, a, Tag::Sword)
        && has(g, q, Tag::Assault)
    {
        s += 10.0;
    }
    s + (pl.life - 40) as f64 * 0.05
}
