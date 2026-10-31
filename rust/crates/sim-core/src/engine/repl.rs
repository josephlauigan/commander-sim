//! The replacement step (Phase 3, CR 614-616): each kind of event that can be replaced (counters placed, a permanent
//! entering, tokens made, damage dealt, life gained, a card going to a graveyard, a card drawn, a trigger) asks one
//! function here, which collects the replacement and prohibition effects that apply from one table and applies them
//! in order. Design: `audit/phase3/repl-design.md`.
//!
//! Where the effects come from:
//! - `NAMED`: the cards the engine knew by name or by tag (Hardened Scales, Vorinclex, Solemnity, Melira ...), looked
//!   up once per card at load (`CardDef::repl`), so a step compares one enum per permanent, never a name;
//! - the ability language's compiled replacements and statics (`tokens`, `counters`, `lifegain` multipliers,
//!   `lifegain_plus`, `no_lifegain`, `enters_tapped`), read through `dsl::statics` while it is on;
//! - card code's replacement hook slots (`no_graveyard`, `prevent_damage`, `no_lifegain`, `enters_tapped`,
//!   `trigger_copies` ...), read through `engine::hooks` while a hooked card is out.
//!
//! Order (616.1): prohibitions first ("can't": Solemnity, Melira); the event's own self-replacements (614.15) are
//! built into the amount the caller hands in; the rest are a short list of modifications (`Mod`) applied in order.

use crate::cards::Tags;
use crate::dsl::model::Ability;
use crate::engine::hooks;
use crate::hooks::Event;
use crate::ids::{PermId, PlayerId};
use crate::state::Game;
use crate::sym::Sym;
use crate::tag::Tag;

// ================================================================== event kinds
/// the kinds of event that go through the step
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ev {
    Enter,
    Counters,
    Tokens,
    Damage,
    LifeGain,
    LifeLoss,
    Graveyard,
    Draw,
    Trigger,
}

// ================================================================== the table
/// a replacement (or prohibition) effect the engine knows by the card's name or tag
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplFx {
    /// Hardened Scales: one more +1/+1 counter on a creature you control
    Scales,
    /// Vorinclex, Monstrous Raider: twice the counters you put, half (rounded down) the ones an opponent puts
    Vorinclex,
    /// Solemnity: players can't get counters; no counters on artifacts, creatures, enchantments or lands
    Solemnity,
    /// Melira, Sylvok Outcast (tag `melira`): you can't get poison counters, no -1/-1 counters on your creatures
    Melira,
    /// Mauhúr, Uruk-hai Captain (tag `mauhur`): one more +1/+1 counter on your Army, Goblin or Orc
    Mauhur,
    /// Cho-Manno, Revolutionary (tag `nodmg`): damage to it is prevented
    NoDamage,
    /// Tajic, Legion's Edge: noncombat damage to your other creatures is prevented
    Tajic,
    /// Academy Manufactor: a Treasure, Food or Clue you'd make is one of each instead
    Manufactor,
    /// Hushbringer: creatures entering or dying don't trigger abilities
    Hushbringer,
    /// Laboratory Maniac, Jace, Wielder of Mysteries: a draw from an empty library wins instead
    LabMan,
    /// Spelunking: lands you control enter untapped
    Spelunking,
}

pub struct Named {
    pub name: &'static str,
    pub ev: Ev,
    pub fx: ReplFx,
}

/// the named replacement effects: one entry per card the engine knew by name
pub const NAMED: [Named; 10] = [
    Named { name: "Hardened Scales", ev: Ev::Counters, fx: ReplFx::Scales },
    Named { name: "Vorinclex, Monstrous Raider", ev: Ev::Counters, fx: ReplFx::Vorinclex },
    Named { name: "Solemnity", ev: Ev::Counters, fx: ReplFx::Solemnity },
    Named { name: "Tajic, Legion's Edge", ev: Ev::Damage, fx: ReplFx::Tajic },
    Named { name: "Academy Manufactor", ev: Ev::Tokens, fx: ReplFx::Manufactor },
    Named { name: "Hushbringer", ev: Ev::Trigger, fx: ReplFx::Hushbringer },
    Named { name: "Laboratory Maniac", ev: Ev::Draw, fx: ReplFx::LabMan },
    Named { name: "Jace, Wielder of Mysteries", ev: Ev::Draw, fx: ReplFx::LabMan },
    Named { name: "Spelunking", ev: Ev::Enter, fx: ReplFx::Spelunking },
    Named { name: "Melira, Sylvok Outcast", ev: Ev::Counters, fx: ReplFx::Melira },
];

/// the tags that are replacement or prohibition effects
pub const TAGGED: [(Tag, Ev, ReplFx); 3] = [
    (Tag::Melira, Ev::Counters, ReplFx::Melira),
    (Tag::Mauhur, Ev::Counters, ReplFx::Mauhur),
    (Tag::Nodmg, Ev::Damage, ReplFx::NoDamage),
];

/// a card's replacement effect, looked up once at load (`CardDef::repl`): its tag first, then its name
pub fn fx_of(name: &str, tags: &Tags) -> Option<ReplFx> {
    TAGGED.iter().find(|t| tags.has(t.0)).map(|t| t.2).or_else(|| NAMED.iter().find(|n| n.name == name).map(|n| n.fx))
}

/// the event kind an effect replaces
pub fn ev_of(fx: ReplFx) -> Ev {
    NAMED.iter().find(|n| n.fx == fx).map(|n| n.ev).or_else(|| TAGGED.iter().find(|t| t.2 == fx).map(|t| t.1)).unwrap()
}

/// permanent m's replacement effect (its copiable values' card)
#[inline]
pub fn fx(g: &Game, m: PermId) -> Option<ReplFx> {
    g.perm(m).cd.and_then(|c| g.db.get(c).repl)
}

#[inline]
fn is(g: &Game, m: PermId, f: ReplFx) -> bool {
    fx(g, m) == Some(f)
}

// ================================================================== the effects' readers
/// a permanent's replacement effect applies: on the battlefield, not phased out, its abilities on (614: a static
/// ability; Stage C made this one rule for every entry)
#[inline]
fn src_on(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased && !x.neutered
}

/// p controls a permanent with this effect, applying
fn controls(g: &Game, p: PlayerId, f: ReplFx) -> bool {
    g.player(p).perms.iter().any(|&m| is(g, m, f) && src_on(g, m))
}

/// Hardened Scales: how many extra +1/+1 counters each placement on one of p's creatures gets (one per Scales)
pub fn scales(g: &Game, p: PlayerId) -> i32 {
    g.player(p).perms.iter().filter(|&&y| is(g, y, ReplFx::Scales) && src_on(g, y)).count() as i32
}

/// Vorinclex: `n` counters (negative: -1/-1) `actor` would put. Each Vorinclex its controller runs doubles them;
/// each one an opponent runs halves them, rounded down (in seat order; `counters` lets the controller order them)
pub fn vorinclex(g: &Game, actor: PlayerId, n: i32) -> i32 {
    let mut mods = Mods::new();
    vorinclex_mods(g, actor, &mut mods);
    mods.apply(n)
}

fn vorinclex_mods(g: &Game, actor: PlayerId, mods: &mut Mods) {
    for pl in &g.players {
        for &y in &pl.perms {
            if is(g, y, ReplFx::Vorinclex) && src_on(g, y) {
                if pl.id == actor {
                    mods.push(Mod::Double, "Vorinclex (yours)");
                } else {
                    mods.push(Mod::Half, "Vorinclex (an opponent's)");
                }
            }
        }
    }
}

/// Solemnity is out (any player's, abilities on): players can't get counters, and counters can't be put on
/// artifacts, creatures, enchantments or lands. Loyalty counters still go on a planeswalker; lore counters (Sagas)
/// are left out
pub fn solemnity(g: &Game) -> bool {
    g.players.iter().any(|q| q.perms.iter().any(|&m| is(g, m, ReplFx::Solemnity) && src_on(g, m)))
}

/// Melira: p can't get poison counters, and p's creatures can't have -1/-1 counters put on them (Solemnity too)
pub fn melira(g: &Game, p: PlayerId) -> bool {
    controls(g, p, ReplFx::Melira) || solemnity(g)
}

/// Mauhúr: p controls one
pub fn mauhur(g: &Game, p: PlayerId) -> bool {
    controls(g, p, ReplFx::Mauhur)
}

/// the product of a card's compiled `counters` replacements' multipliers (`CardDef::counters_mult`, at load)
pub fn counters_mult_of(abilities: &[Ability]) -> i32 {
    abilities
        .iter()
        .map(|a| match a {
            Ability::Replacement { replace, multiplier } if replace == "counters" => multiplier.unwrap_or(2),
            _ => 1,
        })
        .product()
}

/// the counter doublers of p (Doubling Season's compiled `counters` replacement; abilities on)
pub fn counter_mult(g: &Game, p: PlayerId) -> i32 {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| src_on(g, m))
        .filter_map(|&m| g.perm(m).cd)
        .map(|c| g.db.get(c).counters_mult)
        .product()
}

// ================================================================== counters
/// where counters go
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum On {
    Perm(PermId),
    Player(PlayerId),
    /// counters the engine keeps on player p for one of p's artifacts (the Incubator token, The Ozolith)
    Kept(PlayerId),
}

/// which counters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ctr {
    /// +1/+1 (a negative amount: -1/-1)
    Plus,
    Loyalty,
    Poison,
    /// other counters on a permanent (charge counters, a land's kept counters)
    Other,
}

/// A modification of an amount (a replacement effect that changes how many)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mod {
    Add(i32),
    Mul(i32),
    /// Vorinclex for its controller
    Double,
    /// Vorinclex for an opponent: half, rounded down (toward zero for -1/-1 counters)
    Half,
}

impl Mod {
    #[inline]
    pub fn apply(self, k: i32) -> i32 {
        match self {
            Mod::Add(a) => k + a,
            Mod::Mul(x) => k * x,
            Mod::Double => k * 2,
            Mod::Half => k / 2,
        }
    }
}

/// the modifications that apply to one event, in order, with the effect each comes from (a fixed array: no
/// allocation; past 12 the rest are dropped, a table no game reaches)
#[derive(Debug, Clone, Copy)]
pub struct Mods {
    v: [Mod; 12],
    src: [&'static str; 12],
    n: usize,
}

impl Default for Mods {
    fn default() -> Self {
        Self::new()
    }
}

impl Mods {
    pub const fn new() -> Mods {
        Mods { v: [Mod::Add(0); 12], src: [""; 12], n: 0 }
    }
    #[inline]
    pub fn push(&mut self, m: Mod, src: &'static str) {
        if self.n < self.v.len() {
            self.v[self.n] = m;
            self.src[self.n] = src;
            self.n += 1;
        }
    }
    pub fn as_slice(&self) -> &[Mod] {
        &self.v[..self.n]
    }
    pub fn len(&self) -> usize {
        self.n
    }
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
    /// apply them in order
    pub fn apply(&self, n: i32) -> i32 {
        self.as_slice().iter().fold(n, |k, m| m.apply(k))
    }
    /// the effects in this order ("Hardened Scales, then a counter doubler")
    pub fn label(&self) -> String {
        self.src[..self.n].join(", then ")
    }
}

/// which effects a counters path applies. Stage A kept each old site's own subset; since Stage C every placement
/// by an effect applies `ALL`, and a loyalty ability's cost (not an effect: Doubling Season stays out) `VORINCLEX`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Path(pub u8);

impl Path {
    /// no effect
    pub const NONE: Path = Path(0);
    /// Solemnity stops them
    pub const SOLEMN: Path = Path(1);
    /// Vorinclex, by the actor
    pub const VORINCLEX: Path = Path(2);
    /// Hardened Scales (+1/+1 on a creature)
    pub const SCALES: Path = Path(4);
    /// Mauhúr (+1/+1 on an Army, Goblin or Orc)
    pub const MAUHUR: Path = Path(8);
    /// the counter doublers of the permanent's controller (Doubling Season)
    pub const MULT: Path = Path(16);
    /// Melira (and Solemnity) stop poison and -1/-1 counters
    pub const MELIRA: Path = Path(32);
    /// every effect: a placement by an effect
    pub const ALL: Path = Path(1 | 2 | 4 | 8 | 16 | 32);
    /// poison: Melira (Solemnity), Vorinclex
    pub const POISON: Path = Path(2 | 32);

    #[inline]
    pub fn has(self, f: Path) -> bool {
        self.0 & f.0 != 0
    }
}

impl std::ops::BitOr for Path {
    type Output = Path;
    fn bitor(self, o: Path) -> Path {
        Path(self.0 | o.0)
    }
}

/// the controller of what the counters go on
fn controller(g: &Game, on: On) -> PlayerId {
    match on {
        On::Perm(m) => g.perm(m).owner,
        On::Player(q) | On::Kept(q) => q,
    }
}

/// an Army, Goblin or Orc (Mauhúr)
fn orcish(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.army
        || (x.cd.is_some()
            && (crate::engine::values::has_type(g, m, "orc") || crate::engine::values::has_type(g, m, "goblin")))
}

/// Solemnity stops this placement: every counter on a player, and every counter on an artifact, creature,
/// enchantment or land (a planeswalker's loyalty goes on)
fn solemn_stops(g: &Game, on: On, kind: Ctr) -> bool {
    match (on, kind) {
        (On::Perm(m), Ctr::Loyalty) => {
            let types = g.perm(m).cd.map(|c| g.db.get(c).types);
            g.is_creature(m)
                || types.is_some_and(|t| {
                    t.has(crate::cards::Types::ARTIFACT)
                        || t.has(crate::cards::Types::ENCHANTMENT)
                        || t.has(crate::cards::Types::LAND)
                })
        }
        _ => true,
    }
}

/// what one pass over the battlefield finds for a counters event on o's side: the effects that apply
struct Seen {
    solemnity: bool,
    melira: bool,
    mauhur: bool,
    scales: i32,
    mult: i32,
}

/// one pass over every permanent (one byte and one small number per card, no name), collecting the counters
/// effects; Vorinclex goes straight onto `mods` in seat order (applied after the others when nobody orders them)
fn gather(g: &Game, o: PlayerId, actor: PlayerId, vor: &mut Mods) -> Seen {
    let mut s = Seen { solemnity: false, melira: false, mauhur: false, scales: 0, mult: 1 };
    for pl in &g.players {
        for &y in &pl.perms {
            let Some(c) = g.perm(y).cd else { continue };
            let d = g.db.get(c);
            if (d.repl.is_none() && d.counters_mult == 1) || !src_on(g, y) {
                continue;
            }
            let mine = pl.id == o;
            if mine {
                s.mult *= d.counters_mult;
            }
            match d.repl {
                Some(ReplFx::Solemnity) => s.solemnity = true,
                Some(ReplFx::Melira) if mine => s.melira = true,
                Some(ReplFx::Mauhur) if mine => s.mauhur = true,
                Some(ReplFx::Scales) if mine => s.scales += 1,
                Some(ReplFx::Vorinclex) => {
                    if pl.id == actor {
                        vor.push(Mod::Double, "Vorinclex (yours)");
                    } else {
                        vor.push(Mod::Half, "Vorinclex (an opponent's)");
                    }
                }
                _ => {}
            }
        }
    }
    s
}

/// The counters step: how many `kind` counters `actor` really puts `on` it when it would put `n` (negative +1/+1:
/// -1/-1 counters), through the effects `path` applies. Prohibitions first (Solemnity, Melira); then the
/// modifications (Mauhúr and Hardened Scales add, the controller's doublers multiply, Vorinclex doubles or halves),
/// in the order the affected controller picks (616.1, `ordered`).
pub fn counters(g: &Game, actor: PlayerId, on: On, kind: Ctr, n: i32, path: Path) -> i32 {
    if n == 0 {
        return 0;
    }
    let o = controller(g, on);
    let mut vor = Mods::new();
    let s = gather(g, o, actor, &mut vor);
    if path.has(Path::SOLEMN) && s.solemnity && solemn_stops(g, on, kind) {
        return 0;
    }
    if path.has(Path::MELIRA) && (kind == Ctr::Poison || (kind == Ctr::Plus && n < 0)) && s.melira {
        return 0;
    }
    let mut mods = Mods::new();
    if let On::Perm(m) = on
        && kind == Ctr::Plus
        && n > 0
    {
        if path.has(Path::MAUHUR) && s.mauhur && orcish(g, m) {
            mods.push(Mod::Add(1), "Mauhúr");
        }
        if path.has(Path::SCALES) && s.scales > 0 && g.is_creature(m) {
            for _ in 0..s.scales {
                mods.push(Mod::Add(1), "Hardened Scales");
            }
        }
    }
    if path.has(Path::MULT) && s.mult != 1 && matches!(on, On::Perm(_) | On::Kept(_)) {
        mods.push(Mod::Mul(s.mult), "Doubling Season");
    }
    if path.has(Path::VORINCLEX) {
        for (m, src) in vor.as_slice().iter().zip(vor.src.iter()) {
            mods.push(*m, src);
        }
    }
    ordered(g, o, kind, n, &mods)
}

// ================================================================== the order (616.1)
/// the most effects the step orders (6: 720 orders); past it they apply in the order found
const MAX_ORDERED: usize = 6;

/// the affected player wants the most (+1/+1 counters, loyalty, its permanents' other counters) or the fewest
/// (-1/-1 and poison counters)
fn wants_more(kind: Ctr, n: i32) -> bool {
    match kind {
        Ctr::Poison => false,
        Ctr::Plus => n > 0,
        Ctr::Loyalty | Ctr::Other => true,
    }
}

/// every order of the modifications (Heap's algorithm) with its result; the first is the order found
fn orders(n: i32, mods: &Mods) -> Vec<(i32, Mods)> {
    let mut out = vec![];
    let mut m = *mods;
    let k = m.n;
    let mut c = [0usize; MAX_ORDERED];
    out.push((m.apply(n), m));
    let mut i = 0;
    while i < k {
        if c[i] < i {
            let j = if i % 2 == 0 { 0 } else { c[i] };
            m.v.swap(j, i);
            m.src.swap(j, i);
            out.push((m.apply(n), m));
            c[i] += 1;
            i = 0;
        } else {
            c[i] = 0;
            i += 1;
        }
    }
    out
}

/// 616.1: the affected object's controller (or the affected player) `o` orders the replacement effects. The AI
/// takes the best result for itself; a person (practice mode) picks when two orders give different results.
pub fn ordered(g: &Game, o: PlayerId, kind: Ctr, n: i32, mods: &Mods) -> i32 {
    let first = mods.apply(n);
    if mods.n < 2 || mods.n > MAX_ORDERED || mods.as_slice().iter().all(|m| matches!(m, Mod::Add(_))) {
        return first; // additions alone commute
    }
    let mut distinct: Vec<(i32, Mods)> = vec![];
    for (r, m) in orders(n, mods) {
        if !distinct.iter().any(|d| d.0 == r) {
            distinct.push((r, m));
        }
    }
    if distinct.len() == 1 {
        return first;
    }
    if wants_more(kind, n) {
        distinct.sort_by(|a, b| b.0.cmp(&a.0));
    } else {
        distinct.sort_by_key(|a| a.0.abs());
    }
    if crate::human::is_human(g, o) {
        let what = match kind {
            Ctr::Plus if n < 0 => "-1/-1 counters",
            Ctr::Plus => "+1/+1 counters",
            Ctr::Loyalty => "loyalty counters",
            Ctr::Poison => "poison counters",
            Ctr::Other => "counters",
        };
        let labels: Vec<String> = distinct.iter().map(|(r, m)| format!("{}: {}", m.label(), r.abs())).collect();
        let prompt = format!("Order the replacement effects ({} {what} would be put)", n.abs());
        // a cancelled prompt (the page closed) keeps the best order; the next prompt stops the game
        let i = crate::human::choices::choose_one(g, o, "replacement_order", &prompt, &labels).unwrap_or(0);
        return distinct.get(i).map_or(distinct[0].0, |d| d.0);
    }
    distinct[0].0
}

/// put `n` +1/+1 counters (negative: -1/-1) on m as `actor` would, through every effect; returns how many went on
pub fn put_counters(g: &mut Game, actor: PlayerId, m: PermId, n: i32) -> i32 {
    let k = counters(g, actor, On::Perm(m), Ctr::Plus, n, Path::ALL);
    if k != 0 {
        g.perm_mut(m).plus += k;
        crate::impls::t0_captain::counters_put(g, m, k); // Captain Marvel copies counters
    }
    k
}

/// put `n` +1/+1 counters (negative: -1/-1) on m as its owner would (`put_counters`)
pub fn put_plus(g: &mut Game, m: PermId, n: i32) {
    put_counters(g, g.perm(m).owner, m, n);
}

/// p can get poison counters (not under Melira or Solemnity)
pub fn poison_ok(g: &Game, p: PlayerId) -> bool {
    !melira(g, p)
}

/// -1/-1 counters can be put on m (not on Melira's side, not under Solemnity)
pub fn minus_ok(g: &Game, m: PermId) -> bool {
    !melira(g, g.perm(m).owner)
}

/// The poison step: `actor` gives q `n` poison counters through the effects `path` applies; returns how many q got
pub fn give_poison(g: &mut Game, actor: PlayerId, q: PlayerId, n: i32, path: Path) -> i32 {
    let k = counters(g, actor, On::Player(q), Ctr::Poison, n, path);
    if k > 0 {
        g.player_mut(q).poison += k as u32;
    }
    k
}

/// The enter step's counters for m, entering under p's control: (its loyalty, undying's +1/+1 counter, the +1/+1
/// counters it enters with), from its starting loyalty and the counters it would enter with (`plus`; persist -1).
/// Each is a placement by an effect (614.1c): Doubling Season, Hardened Scales, Vorinclex, Solemnity, Melira.
pub fn enter_counters(g: &Game, p: PlayerId, m: PermId, loyalty: Option<i32>, plus: i32) -> (i32, i32, i32) {
    let c = |kind: Ctr, n: i32| counters(g, p, On::Perm(m), kind, n, Path::ALL);
    (loyalty.map_or(0, |l| c(Ctr::Loyalty, l)), c(Ctr::Plus, 1), c(Ctr::Plus, plus))
}

// ================================================================== tokens
/// Tokens: how many tokens p makes when it would make n (Kaya, Geist Hunter; the compiled doublers)
pub fn token_mult(g: &Game, p: PlayerId) -> u32 {
    let kaya = crate::impls::t0_zurgo::kaya_mult(g, p); // Kaya, Geist Hunter -2: twice as many this turn
    if !crate::dsl::active(g) {
        return kaya;
    }
    let mut k = kaya;
    for (q, src, ab, i) in crate::dsl::statics(g, "tokens") {
        if let Ability::Replacement { multiplier, .. } = &ab[i]
            && q == p
            && !g.perm(src).neutered
        {
            k = k.saturating_mul(multiplier.unwrap_or(2).max(1) as u32); // (many copies of a doubler: no overflow)
        }
    }
    k
}

/// the tokens step: n tokens p would make
pub fn tokens(g: &Game, p: PlayerId, n: u32) -> u32 {
    n * token_mult(g, p)
}

/// Academy Manufactor: p controls one
pub fn manufactor(g: &Game, p: PlayerId) -> bool {
    controls(g, p, ReplFx::Manufactor)
}

// ================================================================== damage
/// all damage that would be dealt to m is prevented (Cho-Manno, Revolutionary; Iroas, God of Victory's attackers)
pub fn no_damage(g: &Game, m: PermId) -> bool {
    ((is(g, m, ReplFx::NoDamage) && !g.perm(m).neutered) || crate::impls::t3_aurelia::iroas_shield(g, m))
        && !g.unpreventable()
}

/// all damage m would deal is prevented (Old Fat Spider's chapter II; Arachnogenesis and Galadhrim Ambush: combat
/// damage by non-Spiders / non-Elves this turn)
pub fn source_prevented(g: &Game, m: PermId) -> bool {
    !g.unpreventable()
        && (crate::impls::mine::damage_prevented(g, m)
            || crate::impls::titania::spider_fog(g, m)
            || crate::impls::t3_marwyn::elf_fog(g, m))
}

/// Tajic prevents noncombat damage to its controller's other creatures
pub fn tajic(g: &Game, m: PermId) -> bool {
    let o = g.perm(m).owner;
    g.player(o).perms.iter().any(|&x| x != m && is(g, x, ReplFx::Tajic) && src_on(g, x))
}

/// Tajic's prevention, unless damage can't be prevented (Impractical Joke): noncombat damage to m
pub fn tajic_prevents(g: &Game, m: PermId) -> bool {
    !g.unpreventable() && tajic(g, m)
}

/// Dovin, Hand of Control's -1: damage to and from m is prevented
pub fn dovin(g: &Game, m: PermId) -> bool {
    !g.unpreventable() && crate::impls::rules::dovin_blocked(g, m)
}

// ================================================================== life
/// the ability language's life gain modifications (Cleric Class's +1, The Wind Crystal's doubling)
pub fn lifegain_mod(g: &Game, p: PlayerId, n: i32) -> i32 {
    if n <= 0 || !crate::dsl::active(g) {
        return n;
    }
    let mut n = n;
    for (q, _, ab, i) in crate::dsl::statics(g, "lifegain_plus") {
        if let Ability::Static { amount, .. } = &ab[i]
            && q == p
        {
            n += amount.unwrap_or(1);
        }
    }
    for (q, _, ab, i) in crate::dsl::statics(g, "lifegain") {
        if let Ability::Replacement { multiplier, .. } = &ab[i]
            && q == p
        {
            n *= multiplier.unwrap_or(2).max(1);
        }
    }
    n
}

/// The life gain step: the life p really gains when it would gain n; None when p can't gain life (Erebos, the
/// compiled and card-code "can't gain life" statics). Angel of Vitality's +1, then the compiled ones.
pub fn life_gain(g: &Game, p: PlayerId, n: i32) -> Option<i32> {
    if g.opps(p).any(|q| crate::engine::values::has(g, q, Tag::Erebos)) {
        return None; // Erebos: opponents can't gain life
    }
    if crate::dsl::no_lifegain(g, p) || (!g.hooks.is_empty() && hooks::any_no_lifegain(g, p)) {
        return None;
    }
    let n = n + crate::impls::t0_giada::gain_bonus(g, p, n); // Angel of Vitality: plus 1
    Some(lifegain_mod(g, p, n)) // --- Tiers 4/5 (heliod-zur): Cleric Class, The Wind Crystal
}

// ================================================================== graveyard
/// Rest in Peace and kin: a card of p's that would go to a graveyard is exiled instead
pub fn no_graveyard(g: &Game, p: PlayerId) -> bool {
    !g.hooks.is_empty() && hooks::total_player(g, Event::NoGraveyard, p) != 0
}

// ================================================================== draw
/// Laboratory Maniac / Jace: p controls one: a draw from an empty library wins
pub fn labman(g: &Game, p: PlayerId) -> bool {
    controls(g, p, ReplFx::LabMan)
}

// ================================================================== triggers
/// Hushbringer: creatures entering or dying don't cause abilities to trigger
pub fn hushed(g: &Game) -> bool {
    g.players.iter().filter(|q| q.alive).flat_map(|q| q.perms.iter()).any(|&m| {
        let x = g.perm(m);
        !x.phased && !x.neutered && is(g, m, ReplFx::Hushbringer)
    })
}

/// extra copies of p's triggered abilities of this kind ('etb', 'dies', 'landfall', 'attack'; x the permanent
/// that set them off): Panharmonicon, Teysa, Isshin, Ancient Greenwarden, Drivnod
pub fn trigger_copies(g: &Game, p: PlayerId, kind: Sym, x: Option<PermId>) -> i32 {
    hooks::total_trigger_copies(g, p, kind, x)
}

// ================================================================== enter
/// "enters tapped" replacements: does a permanent entering under p's control enter tapped (m None: a Treasure,
/// Food or Clue)?
pub fn enters_tapped(g: &Game, p: PlayerId, m: Option<PermId>) -> bool {
    crate::dsl::opp_enters_tapped(g, p, m) // --- Tiers 4/5 (heliod-zur): Authority of the Consuls (compiled)
        || (!g.hooks.is_empty()
            && hooks::hooked(g, Event::EntersTapped)
                .iter()
                .any(|(src, imp)| (imp.enters_tapped.unwrap())(g, *src, p, m)))
}

/// Spelunking: p's lands enter untapped
pub fn spelunking(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        x.on_bf && !x.phased && !x.neutered && is(g, m, ReplFx::Spelunking)
    })
}
