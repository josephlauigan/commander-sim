//! The huplan_plain(self.m, generic, pips).map(|m| Left { m, k: None, art })an seat's mana pool: tap sources yourself, pay costs from the pool, and the pool empties between steps
//! (Python's `play/mana.py`).
//!
//! The AI seats keep the engine's automatic payment (`engine::mana::pay`). A tapped source goes through
//! `engine::mana::spend_unit`, so pain lands, Treasures and on-tap triggers behave exactly as they do for the AI.

use crate::engine::mana::{Source, Unit, mana_units, spend_unit};
use crate::engine::values::has_type;
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

pub const COLOURS: &str = "WUBRGC";
/// any colour, but only for creature spells / creatures of a named type
pub const RESTRICTED_LANDS: [&str; 1] = ["Secluded Courtyard"];

fn idx(c: char) -> Option<usize> {
    COLOURS.find(c)
}

fn colour_name(c: char) -> String {
    match c {
        'W' => "white",
        'U' => "blue",
        'B' => "black",
        'R' => "red",
        'G' => "green",
        'C' => "colourless",
        _ => return c.to_string(),
    }
    .to_string()
}

/// What is being paid for now: a card, or a creature whose ability it is (restricted mana's test; Python's
/// `mana.SPENDING`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spending {
    Card(CardId),
    Perm(PermId),
}

/// Mana you have made and not spent. `m[6]` is mana of any colour the engine adds for you (rituals and similar).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ManaPool {
    /// W U B R G C, then any
    pub m: [u32; 7],
    /// restricted mana (Secluded Courtyard), by colour: for creature spells of type `ktype`
    pub k: [u32; 6],
    pub ktype: Option<Sym>,
    /// colourless from Karn, Legacy Reforged: not for nonartifact spells, and it stays all turn (`empty` leaves it)
    pub art: u32,
    pub spending: Option<Spending>,
}

/// the pool left after a payment
struct Left {
    m: [u32; 7],
    k: Option<[u32; 6]>,
    art: u32,
}

fn plan_plain(mut m: [u32; 7], generic: u32, pips: &str) -> Option<[u32; 7]> {
    for c in pips.chars() {
        match idx(c) {
            Some(i) if m[i] > 0 => m[i] -= 1,
            _ if m[6] > 0 => m[6] -= 1,
            _ => return None,
        }
    }
    let mut need = generic;
    let mut order: Vec<usize> = vec![5];
    let mut wubrg: Vec<usize> = (0..5).collect();
    wubrg.sort_by_key(|&i| std::cmp::Reverse(m[i])); // stable, like Python's sorted(key=-m[c])
    order.extend(wubrg);
    order.push(6);
    for i in order {
        let k = m[i].min(need);
        m[i] -= k;
        need -= k;
        if need == 0 {
            break;
        }
    }
    (need == 0).then_some(m)
}

impl ManaPool {
    pub fn add(&mut self, colour: char, n: u32) {
        match idx(colour) {
            Some(i) => self.m[i] += n,
            None => self.m[6] += n, // 'A'
        }
    }

    pub fn add_restricted(&mut self, colour: char, n: u32, ctype: Sym) {
        self.k[idx(colour).unwrap()] += n;
        self.ktype = Some(ctype);
    }

    pub fn total(&self) -> u32 {
        self.m.iter().sum::<u32>() + self.k.iter().sum::<u32>() + self.art
    }

    pub fn empty(&mut self) {
        self.m = [0; 7];
        self.k = [0; 6];
    }

    /// {B}{B}{C} style: coloured mana, then colourless, then any-colour ({3} would read as generic mana)
    pub fn text(&self) -> String {
        let mut out = String::new();
        for (i, c) in COLOURS.chars().enumerate() {
            for _ in 0..self.m[i] {
                out.push_str(&format!("{{{c}}}"));
            }
        }
        if self.m[6] > 0 {
            out.push_str(&format!(" +{} any", self.m[6]));
        }
        if self.art > 0 {
            out.push_str(&format!(" +{} for artifact spells and abilities", self.art));
        }
        if self.k.iter().any(|&x| x > 0) {
            out.push_str(" +");
            for (i, c) in COLOURS.chars().enumerate() {
                for _ in 0..self.k[i] {
                    out.push_str(&format!("{{{c}}}"));
                }
            }
            out.push_str(&format!(" for {} creature spells", super::legal::title(self.ktype.unwrap_or("the named"))));
        }
        if out.is_empty() { "empty".into() } else { out }
    }

    /// may the restricted mana pay for what is being paid for now
    fn k_ok(&self, g: &Game, owner: PlayerId) -> bool {
        let (Some(x), Some(t)) = (self.spending, self.ktype) else { return false };
        if !self.k.iter().any(|&n| n > 0) {
            return false;
        }
        match x {
            Spending::Perm(m) => g.is_creature(m) && has_type(g, m, t),
            Spending::Card(c) => g.db.get(c).creature && crate::impls::galadriel::card_is(g, owner, c, t),
        }
    }

    /// may the artifact-only mana (Karn) pay for what is being paid for now: anything but a nonartifact spell
    fn art_ok(&self, g: &Game) -> bool {
        match self.spending {
            Some(Spending::Card(c)) => g.db.get(c).types.has(crate::cards::Types::ARTIFACT),
            _ => true,
        }
    }

    /// The pool left after paying {generic}+pips, or None if it can't: each coloured pip from its own colour
    /// (any-colour mana last); generic from colourless first, then the most plentiful colour, then any-colour.
    /// Restricted mana (Secluded Courtyard) is spent first when it may be.
    fn plan(&self, g: &Game, owner: PlayerId, generic: u32, pips: &str) -> Option<Left> {
        // Karn's mana first (it is gone at the end of the turn): it pays generic mana only
        let used_art = if self.art_ok(g) { self.art.min(generic) } else { 0 };
        let art = self.art - used_art;
        let generic = generic - used_art;
        if self.k_ok(g, owner) {
            let mut k = self.k;
            let mut rest = String::new();
            for c in pips.chars() {
                match idx(c) {
                    Some(i) if k[i] > 0 => k[i] -= 1,
                    _ => rest.push(c),
                }
            }
            let mut need = generic;
            for i in 0..6 {
                let t = k[i].min(need);
                k[i] -= t;
                need -= t;
            }
            let m = plan_plain(self.m, need, &rest)?;
            return Some(Left { m, k: Some(k), art });
        }
        plan_plain(self.m, generic, pips).map(|m| Left { m, k: None, art })
    }

    pub fn can_pay(&self, g: &Game, owner: PlayerId, generic: u32, pips: &str) -> bool {
        self.plan(g, owner, generic, pips).is_some()
    }

    pub fn pay(&mut self, g: &Game, owner: PlayerId, generic: u32, pips: &str) -> bool {
        match self.plan(g, owner, generic, pips) {
            None => false,
            Some(l) => {
                self.m = l.m;
                self.art = l.art;
                if let Some(k) = l.k {
                    self.k = k;
                }
                true
            }
        }
    }

    /// in words, what the pool lacks for this cost (for the error message)
    pub fn missing(&self, generic: u32, pips: &str) -> String {
        let mut m = self.m;
        let mut short = vec![];
        for c in pips.chars() {
            match idx(c) {
                Some(i) if m[i] > 0 => m[i] -= 1,
                _ if m[6] > 0 => m[6] -= 1,
                _ => short.push(format!("{{{c}}}")),
            }
        }
        let left: u32 = m.iter().sum();
        if generic > left {
            short.insert(0, format!("{{{}}}", generic - left));
        }
        short.concat()
    }

    /// 'is empty' / 'has {B}{B}', for messages
    pub fn has_text(&self) -> String {
        if self.total() == 0 { "is empty".into() } else { format!("has {}", self.text()) }
    }
}

/// The pool with the engine's floating mana (rituals, Birgi, Lion's Eye Diamond) added, without changing the game.
/// Python's `_gather_floating` moves it into the pool for good; `gather_floating` does that.
fn gathered(g: &Game, p: PlayerId) -> ManaPool {
    let pl = g.player(p);
    let mut pool = pl.pool.clone();
    let f = pl.floating;
    pool.art += f.art;
    for (n, c) in [(f.r, 'R'), (f.u, 'U'), (f.c, 'C'), (f.any, 'A'), (f.g, 'G'), (f.b, 'B')] {
        pool.add(c, n);
    }
    pool
}

pub fn gather_floating(g: &mut Game, p: PlayerId) {
    let pool = gathered(g, p);
    let pl = g.player_mut(p);
    pl.pool = pool;
    let w = pl.floating.w; // Python has no floatW: white floating mana stays where it is
    pl.floating = Default::default();
    pl.floating.w = w;
}

pub fn pool_can_pay(g: &Game, p: PlayerId, generic: u32, pips: &str) -> bool {
    gathered(g, p).can_pay(g, p, generic, pips)
}

pub fn cost_text(generic: u32, pips: &str) -> String {
    let mut s = String::new();
    if generic > 0 {
        s.push_str(&format!("{{{generic}}}"));
    }
    for c in pips.chars() {
        s.push_str(&format!("{{{c}}}"));
    }
    if s.is_empty() { "{0}".into() } else { s }
}

fn problem(pool: &ManaPool, generic: u32, pips: &str) -> String {
    format!(
        "It costs {}; your mana pool {}. Missing {}.",
        cost_text(generic, pips),
        pool.has_text(),
        pool.missing(generic, pips)
    )
}

/// None if p's pool can pay this now, else the reason (nothing is spent).
pub fn cost_problem(g: &Game, p: PlayerId, generic: u32, pips: &str) -> Option<String> {
    let pool = gathered(g, p);
    if pool.can_pay(g, p, generic, pips) {
        return None;
    }
    Some(problem(&pool, generic, pips))
}

/// Pay a cost from p's pool and the engine's floating mana (rituals). Returns None, or the reason it can't.
pub fn pay_from_pool(g: &mut Game, p: PlayerId, generic: u32, pips: &str) -> Option<String> {
    gather_floating(g, p);
    let mut pool = std::mem::take(&mut g.player_mut(p).pool);
    let ok = pool.pay(g, p, generic, pips);
    let why = if ok { None } else { Some(problem(&pool, generic, pips)) };
    g.player_mut(p).pool = pool;
    why
}

/// a step ends: every mana pool empties
pub fn empty_pools(g: &mut Game) {
    for p in g.players.iter_mut() {
        p.pool.empty();
    }
}

// ------------------------------------------------------------------ sources
/// One thing p can tap (or sacrifice) for mana now. `id` indexes the engine's mana units and is only valid until the
/// next change to the board.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceInfo {
    pub id: usize,
    pub name: String,
    pub colours: String,
    pub amount: u32,
    /// Secluded Courtyard: the creature type its mana is restricted to
    pub restricted: Option<Sym>,
    pub treasure: bool,
    /// the index among p's lands / permanents, for the browser
    pub land: Option<usize>,
    pub perm: Option<usize>,
}

/// engine units that are already floating mana, not sources to tap (Python's FLOATS: it leaves out the black one)
fn is_float(s: Source) -> bool {
    matches!(s, Source::FloatR | Source::FloatAny | Source::FloatU | Source::FloatC | Source::FloatG | Source::FloatArt)
}

/// the colours unit u can make; painlands and Talismans can also make colourless, without the pain
fn unit_colours(g: &Game, u: &Unit) -> String {
    let mut cols: String = "WUBRG".chars().filter(|&c| u.cols.has(c)).collect();
    if cols.is_empty() {
        cols.push('C');
    }
    let cd = match u.src {
        Source::Land(l) => Some(g.land(l).cd),
        Source::Perm(m) => g.perm(m).cd,
        _ => None,
    };
    if let Some(cd) = cd
        && g.db.get(cd).tag(Tag::Pain)
        && !cols.contains('C')
    {
        cols.push('C');
    }
    cols
}

fn restricted_type(g: &Game, src: Source) -> Option<Sym> {
    let Source::Land(l) = src else { return None };
    let land = g.land(l);
    if !RESTRICTED_LANDS.contains(&&*g.db.get(land.cd).name) {
        return None;
    }
    match land.data.get(DataKey::Ctype) {
        Some(Val::Str(s)) => Some(s),
        _ => None,
    }
}

fn source_name(g: &Game, src: Source) -> String {
    match src {
        Source::Treasure => "a Treasure".into(),
        Source::Land(l) => g.db.get(g.land(l).cd).name.to_string(),
        Source::Perm(m) | Source::Scion(m) | Source::Quirion(_, m) | Source::Symbiote(_, m, _) => {
            g.perm(m).name.to_string()
        }
        Source::HandCard(c) => g.db.get(c).name.to_string(),
        _ => "a mana source".into(),
    }
}

/// The untapped things p can tap (or sacrifice) for mana now (Python's `sources`).
pub fn sources(g: &Game, p: PlayerId) -> Vec<SourceInfo> {
    let pl = g.player(p);
    let mut out = vec![];
    for (i, u) in mana_units(g, p, false).iter().enumerate() {
        if is_float(u.src) {
            continue;
        }
        let name = match u.src {
            Source::Treasure => "Treasure".to_string(),
            Source::FloatB => "FB".to_string(),
            s => source_name(g, s),
        };
        let mut d = SourceInfo {
            id: i,
            name,
            colours: unit_colours(g, u),
            amount: u.amt,
            restricted: None,
            treasure: false,
            land: None,
            perm: None,
        };
        if let Some(kt) = restricted_type(g, u.src) {
            d.colours = format!("C{}", "WUBRG".chars().filter(|&c| pl.ident.has(c)).collect::<String>());
            d.restricted = Some(kt);
        }
        match u.src {
            Source::Treasure => d.treasure = true,
            Source::Land(l) => d.land = pl.lands.iter().position(|&x| x == l),
            Source::Perm(m) | Source::Scion(m) => d.perm = pl.perms.iter().position(|&x| x == m),
            _ => {}
        }
        out.push(d);
    }
    out
}

/// Tap (or sacrifice) source `source_id` from `sources()` for its mana, into p's pool; `colour`: which colour a
/// multi-colour source makes (default: its first). Returns an error string, or None when it worked.
pub fn tap(g: &mut Game, p: PlayerId, source_id: i64, colour: Option<char>) -> Res<Option<String>> {
    let units = mana_units(g, p, false);
    let Some(u) = (source_id >= 0).then(|| units.get(source_id as usize)).flatten().filter(|u| !is_float(u.src)) else {
        return Ok(Some("That source is gone or already tapped.".into()));
    };
    let u = *u;
    let name = g.player(p).name;
    if let Some(kt) = restricted_type(g, u.src)
        && let Some(col) = colour.filter(|&c| c != 'C')
    {
        if !g.player(p).ident.has(col) {
            return Ok(Some(format!("That source can't make {} mana.", colour_name(col))));
        }
        spend_unit(g, p, u, 1, "")?;
        g.player_mut(p).pool.add_restricted(col, 1, kt);
        let sn = unit_label(g, u.src);
        crate::glog!(
            g,
            "  {} taps {} for {{{}}} (only for {} creature spells)",
            name,
            sn,
            col,
            super::legal::title(kt)
        );
        return Ok(None);
    }
    let cols = unit_colours(g, &u);
    let colour = colour.unwrap_or_else(|| cols.chars().next().unwrap());
    if !cols.contains(colour) {
        return Ok(Some(format!("That source can't make {} mana.", colour_name(colour))));
    }
    let paid = if colour != 'C' { colour.to_string() } else { String::new() };
    spend_unit(g, p, u, u.amt, &paid)?;
    g.player_mut(p).pool.add(colour, u.amt);
    let sn = unit_label(g, u.src);
    let mana: String = (0..u.amt).map(|_| format!("{{{colour}}}")).collect();
    crate::glog!(g, "  {} taps {} for {}", name, sn, mana);
    Ok(None)
}

/// Python's `sources_name`
fn unit_label(g: &Game, src: Source) -> String {
    match src {
        Source::Treasure => "a Treasure".into(),
        Source::Land(_) | Source::Perm(_) => source_name(g, src),
        _ => "a mana source".into(),
    }
}

/// `choices::pay_tax`'s body: an opponent's tax (Rhystic Study, Smothering Tithe, Mystic Remora): ask p whether to
/// pay {n}. Paid from p's floating mana first, then by tapping p's sources (as you would at the table).
pub fn pay_tax(g: &mut Game, p: PlayerId, n: u32, what: &str) -> Res<bool> {
    let name = g.player(p).name;
    let can = gathered(g, p).total() + sources(g, p).iter().map(|s| s.amount).sum::<u32>();
    if can < n {
        crate::glog!(g, "    {} can't pay {{{}}} for {}", name, n, what);
        return Ok(false);
    }
    if !super::choices::yes_no(g, p, &format!("{what}: pay {{{n}}}?"))? {
        return Ok(false);
    }
    gather_floating(g, p);
    while g.player(p).pool.total() < n {
        let Some(s) = sources(g, p).first().map(|s| s.id) else { break };
        tap(g, p, s as i64, None)?;
    }
    let mut pool = std::mem::take(&mut g.player_mut(p).pool);
    let ok = pool.pay(g, p, n, "");
    g.player_mut(p).pool = pool;
    if !ok {
        return Ok(false);
    }
    crate::glog!(g, "    {} pays {{{}}} for {}", name, n, what);
    Ok(true)
}
