//! The Underworld Breach line (any deck holding Breach and a storm card runs it). Restored 2026-11-06 from the code the
//! hand-AI deletion took out of `impls/mine.rs` (60d9022c, where only Sauron's and Kefka's options reached it); now
//! offered by the generic main-phase options (`ai::brain`), so every deck with the pieces plays it.
//!
//! Dry run first: the whole line played on a copy of the game with the opponents' hands removed; offered only when it
//! finishes every opponent (burned out or milled out). Then cast for real, one spell at a time (each can be countered).
//! Rituals carry their colour: Dark and Cabal Ritual float only {B} and the red rituals (Seething Song, Desperate
//! Ritual, Rite of Flame) only {R} (the pool keeps them as black and red mana), so black ritual mana never pays
//! Grapeshot's {R}, and neither pays Brain Freeze's {U}.

use crate::engine::cast::{castable, cast_card, casts_this_turn};
use crate::engine::life::check_state;
use crate::engine::mana::{Source, mana_units, pay, total_mana};
use crate::engine::pool::Kind;
use crate::engine::repl;
use crate::engine::tutors::card_worth;
use crate::engine::values::{card_name, has};
use crate::engine::zones::{Zone, discard_cards, leave, min_by, to_zone_card};
use crate::flow::Res;
use crate::hooks::{Action, Opt};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::Sym;
use crate::tag::Tag;

const BRAIN_FREEZE: &str = "Brain Freeze";
const GRAPESHOT: &str = "Grapeshot";
const STORM_PIECES: [&str; 2] = [BRAIN_FREEZE, GRAPESHOT];
/// (name, {generic} besides its one coloured pip, mana made, made with threshold (seven cards in your graveyard),
/// the pip and the colour it makes)
const RITUALS: [(&str, u32, u32, u32, &str); 5] = [
    ("Dark Ritual", 0, 3, 3, "B"),
    ("Cabal Ritual", 1, 3, 5, "B"),
    ("Seething Song", 2, 5, 5, "R"),
    ("Desperate Ritual", 1, 3, 3, "R"),
    ("Rite of Flame", 0, 2, 2, "R"),
];
/// {0}: sacrifice it for one mana of any colour
const PETAL: &str = "Lotus Petal";
/// {0}: discard your hand, sacrifice it: three of one colour
const LED: &str = "Lion's Eye Diamond";
/// {R} whenever you cast a spell: with Petal, a Brain Freeze
const BIRGI: &str = "Birgi, God of Storytelling // Harnfel, Horn of Bounty";
/// {2}{R}: {R} for each card in an opponent's hand
const JESKA: &str = "Jeska's Will";
/// in hand: cast first, then the line mills you out and draws to win
const LABMEN: [&str; 2] = ["Laboratory Maniac", "Jace, Wielder of Mysteries"];
const N_LINE: usize = 10;
const LINE_CARDS: [&str; N_LINE] = [
    BRAIN_FREEZE,
    GRAPESHOT,
    "Dark Ritual",
    "Cabal Ritual",
    "Seething Song",
    "Desperate Ritual",
    "Rite of Flame",
    PETAL,
    LED,
    JESKA,
];

/// the floating ritual mana: (black, red, any colour). Rituals float {B} or {R}; a Lotus Petal sacrificed for "any colour"
/// floats as any (counted with the black).
fn ritual_float(g: &Game, p: PlayerId) -> (u32, u32, u32) {
    let m = &g.player(p).mana;
    (m.get(Kind::B), m.get(Kind::R), m.get(Kind::Any))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Wh {
    Hand,
    Gy,
}

/// mine._line_state: the numbers the Breach line decides on. A ritual makes only its own colour ({B} or {R}, `float_r`
/// is the red share): it pays generic costs and pips of its colour, never Brain Freeze's {U}
struct LineState {
    land: u32,
    float: u32,
    /// the red share of `float` (red rituals); the rest is black
    float_r: u32,
    u: u32,
    b: u32,
    r: u32,
    fuel: usize,
    gy: usize,
    storm: i32,
    /// where each of LINE_CARDS is (hand first), None when it's in the graveyard and graveyard casts are locked
    wh: [Option<Wh>; N_LINE],
    breach: bool,
    breach_hand: bool,
    led_bf: bool,
    labman: bool,
    /// Laboratory Maniac / Jace / Birgi in hand: (card, generic, pips)
    setup_hand: Option<(CardId, u32, String)>,
    /// draw spells in hand or graveyard: (card, zone, generic, pip)
    draws: Vec<(CardId, Wh, u32, String)>,
    /// opponents' library sizes and life, in seat order
    lib: Vec<(PlayerId, usize)>,
    life: Vec<(PlayerId, i32)>,
    mylib: usize,
    opp_hand: usize,
}

impl LineState {
    fn w(&self, name: &str) -> Option<Wh> {
        self.wh[LINE_CARDS.iter().position(|&n| n == name).unwrap()]
    }

    /// st[col]: the real sources of a colour
    fn col(&self, col: &str) -> u32 {
        match col {
            "U" => self.u,
            "B" => self.b,
            "R" => self.r,
            _ => 0,
        }
    }

    fn life_of(&self, q: PlayerId) -> i32 {
        self.life.iter().find(|x| x.0 == q).map_or(0, |x| x.1)
    }
}

fn where_card(g: &Game, p: PlayerId, c: Option<CardId>) -> Option<Wh> {
    let c = c?;
    if g.player(p).hand.contains(&c) {
        Some(Wh::Hand)
    } else if g.player(p).gy.contains(&c) {
        Some(Wh::Gy)
    } else {
        None
    }
}

fn is_line_card(g: &Game, c: CardId) -> bool {
    LINE_CARDS.contains(&&*g.db.get(c).name)
}

fn perm_named(g: &Game, p: PlayerId, name: &str, live: bool) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| card_name(g, m) == Some(name) && !(live && g.perm(m).phased))
}

fn line_state(g: &Game, p: PlayerId) -> LineState {
    let ritual_mana = |k: Kind| matches!(k, Kind::B | Kind::R | Kind::Any);
    let real: Vec<_> = mana_units(g, p, false)
        .into_iter()
        .filter(|u| !matches!(u.src, Source::Pool(key) if ritual_mana(key.kind)))
        .collect();
    let (fb, fr, fa) = ritual_float(g, p);
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let bf = g.db.id(BRAIN_FREEZE).unwrap();
    let gy_ok = !g.cast_checks() || castable(g, p, bf, "gy"); // a lock on casting from graveyards stops escapes
    let pl = g.player(p);
    let colour = |c: char| real.iter().filter(|u| u.cols.has(c)).map(|u| u.amt).sum::<u32>();
    let mut wh = [None; N_LINE];
    for (i, n) in LINE_CARDS.iter().enumerate() {
        let w = where_card(g, p, g.db.id(n));
        wh[i] = if gy_ok || w != Some(Wh::Gy) { w } else { None };
    }
    let setup_hand = if g.line_no_setup {
        None
    } else {
        pl.hand
            .iter()
            .find(|&&c| {
                let d = g.db.get(c);
                (LABMEN.contains(&&*d.name) || &*d.name == BIRGI)
                    && d.pips.chars().count() == 1
                    && !pl.perms.iter().any(|&m| g.perm(m).cd.is_some_and(|x| g.db.get(x).name == d.name))
            })
            .map(|&c| (c, g.db.get(c).generic, g.db.get(c).pips.to_string()))
    };
    let mut draws = vec![];
    for (z, cs) in [(Wh::Hand, &pl.hand), (Wh::Gy, &pl.gy)] {
        for &c in cs {
            let d = g.db.get(c);
            if d.tags.int(Tag::Draw).is_some()
                && (d.instant || d.sorcery)
                && d.pips.chars().count() == 1
                && !is_line_card(g, c)
                && (z == Wh::Hand || gy_ok)
            {
                draws.push((c, z, d.generic, d.pips.to_string()));
            }
        }
    }
    LineState {
        land: real.iter().map(|u| u.amt).sum(),
        float: fb + fr + fa,
        float_r: fr,
        u: colour('U'),
        b: colour('B'),
        r: colour('R'),
        fuel: pl.gy.iter().filter(|&&c| !is_line_card(g, c)).count(),
        gy: pl.gy.len(),
        storm: g.players.iter().map(|q| casts_this_turn(g, q.id)).sum(),
        wh,
        breach: has(g, p, Tag::Breach),
        breach_hand: pl.hand.iter().any(|&c| g.db.get(c).tag(Tag::Breach)),
        led_bf: perm_named(g, p, LED, true).is_some(),
        labman: repl::labman(g, pl.id),
        setup_hand,
        draws,
        lib: opps.iter().map(|&q| (q, g.player(q).library.len())).collect(),
        life: opps.iter().map(|&q| (q, g.player(q).life)).collect(),
        mylib: pl.library.len(),
        opp_hand: if g.line_no_setup { 0 } else { opps.iter().map(|&q| g.player(q).hand.len()).max().unwrap_or(0) },
    }
}

/// mine._can: can the line pay {gn} plus one {col}? ({B} can come from black ritual mana, {R} from red)
fn can(st: &LineState, gn: u32, col: &str) -> bool {
    let float_b = st.float - st.float_r;
    let ok = match col {
        "B" => float_b > 0 || st.b > 0,
        "R" => st.float_r > 0 || (st.r > 0 && st.land > 0),
        _ => st.col(col) > 0 && st.land > 0,
    };
    ok && st.float + st.land >= gn + 1
}

fn ready(st: &LineState, name: &str) -> bool {
    let w = st.w(name);
    w == Some(Wh::Hand) || (w == Some(Wh::Gy) && st.fuel >= 3)
}

fn ceil3(n: i64) -> i64 {
    (n + 2).div_euclid(3)
}

/// mine._mill_plan: who each Brain Freeze copy mills (st: after paying for it). The table if the copies finish it.
/// Otherwise yourself first, enough fuel to pay for the next Brain Freeze (Lotus Petal escapes for {U}, rituals for
/// generic), escape it, and recast each ritual twice more for storm; then the opponent closest to an empty library.
/// If nothing can pay the next Brain Freeze's {U}, this is the last one: every copy goes to the table
fn mill_plan(p: PlayerId, st: &LineState, copies: i64) -> Vec<PlayerId> {
    if st.labman {
        return vec![p; copies.max(0) as usize]; // Laboratory Maniac / Jace: mill yourself out, then draw
    }
    let mut lib: Vec<(PlayerId, i64)> = st.lib.iter().map(|&(q, n)| (q, n as i64)).collect();
    let need: i64 = lib.iter().filter(|&&(q, n)| n > 0 && st.life_of(q) > 0).map(|&(_, n)| ceil3(n)).sum();
    let need_u: i64 = if st.u > 0 && st.land > 0 { 0 } else { 1 };
    let need_gen = (2 - st.land as i64 - st.float as i64 - need_u).max(0);
    let petal = st.w(PETAL).is_some();
    let led = st.w(LED).is_some() || st.led_bf;
    let rit = RITUALS.iter().filter(|r| st.w(r.0).is_some()).count() as i64;
    let jeska = if st.w(JESKA).is_some() { 3 } else { 0 }; // fuel for a Jeska's Will escape
    let last = need_u != 0 && !petal && !led;
    let keep = if led {
        // one Lion's Eye Diamond escape pays the whole next Brain Freeze; plus three spare: milled line cards aren't
        // fuel
        6 + if need_u != 0 || need_gen != 0 { 3 } else { 0 } + jeska
    } else {
        let mut esc = if petal { need_u + if rit > 0 { 0 } else { need_gen } } else { 0 };
        esc += if rit > 0 { (need_gen + 1).div_euclid(2) } else { 0 };
        6 + 3 * esc + 6 * rit + jeska // three spare: milled line cards aren't fuel
    };
    let mine = if copies >= need || last {
        0
    } else {
        copies.min(ceil3((keep - st.fuel as i64).max(0))).min(st.mylib as i64 / 3)
    };
    let mut plan = vec![p; mine.max(0) as usize];
    for _ in 0..(copies - mine).max(0) {
        let live: Vec<PlayerId> = lib.iter().filter(|&&(q, n)| n > 0 && st.life_of(q) > 0).map(|&(q, _)| q).collect();
        if live.is_empty() {
            // every opponent is milled out: the spare copy mills one of them (nothing), never yourself. (Python sent it
            // at you, and Sauron decked himself on his own draws before the table drew from empty libraries.)
            plan.push(st.lib.first().map_or(p, |x| x.0));
            continue;
        }
        let q = min_by(&live, |x| lib.iter().find(|y| y.0 == x).unwrap().1 as f64).unwrap();
        lib.iter_mut().find(|y| y.0 == q).unwrap().1 -= 3;
        plan.push(q);
    }
    plan
}

/// mine._draw_ready: a draw spell the line can cast (from hand) or escape (from the graveyard) now
fn draw_ready(st: &LineState) -> Option<(CardId, Wh, u32, String)> {
    st.draws.iter().find(|(_, z, gn, pip)| (*z == Wh::Hand || st.fuel >= 4) && can(st, *gn, pip)).cloned() // (it isn't its own fuel)
}

/// the Breach line's next cast
#[derive(Debug, Clone, Copy, PartialEq)]
enum LineAct {
    Setup,
    Breach,
    Draw,
    Card(&'static str),
}

/// mine._line_step: the next cast of the Breach line, or None. Fuel (graveyard cards) goes first to what the next
/// Brain Freeze needs: its own escape, and a Lotus Petal escape when blue mana runs out; spare fuel becomes ritual
/// escapes, which add storm (and pay generic costs)
fn line_step(st: &LineState) -> Option<LineAct> {
    if let Some((_, gn, pips)) = &st.setup_hand
        && can(st, *gn, pips)
    {
        return Some(LineAct::Setup); // Laboratory Maniac / Birgi first, from hand
    }
    if !st.breach {
        return (st.breach_hand && can(st, 1, "R")).then_some(LineAct::Breach);
    }
    let alive: Vec<PlayerId> = st.lib.iter().filter(|&&(q, n)| n > 0 && st.life_of(q) > 0).map(|&(q, _)| q).collect();
    if alive.is_empty() {
        return None;
    }
    let (hand, gy) = (Some(Wh::Hand), Some(Wh::Gy));
    if st.labman && st.mylib == 0 {
        // library empty: draw a card to win
        if draw_ready(st).is_some() {
            return Some(LineAct::Draw);
        }
        // short of its mana: Petal escapes pay for it
        let res = if st.draws.iter().any(|d| d.1 == Wh::Gy) { 4 } else { 0 };
        if !st.draws.is_empty() && (st.w(PETAL) == hand || (st.w(PETAL) == gy && st.fuel >= 3 + res)) {
            return Some(LineAct::Card(PETAL));
        }
        return None;
    }
    let bf_res = if st.w(BRAIN_FREEZE) == gy { 3 } else { 0 };
    let petal_ready = st.w(PETAL) == hand || (st.w(PETAL) == gy && st.fuel >= 3 + bf_res);
    let petal_res = if st.w(PETAL).is_some() && st.u <= 1 { 3 } else { 0 }; // the next Brain Freeze will want a Petal
    let led_ready = |res: usize| st.led_bf || st.w(LED) == hand || (st.w(LED) == gy && st.fuel >= 3 + res);
    let ritual = |res: usize| {
        for &(n, gn, made, thr, col) in &RITUALS {
            let got = if st.gy >= 7 { thr } else { made };
            if (st.w(n) == hand || (st.w(n) == gy && st.fuel >= 3 + res)) && got > gn + 1 && can(st, gn, col) {
                return Some(LineAct::Card(n));
            }
        }
        None
    };
    let copies = st.storm + 1;
    let gs = st.w(GRAPESHOT).is_some() && ready(st, GRAPESHOT) && can(st, 1, "R");
    if gs && copies >= alive.iter().map(|&q| st.life_of(q)).min().unwrap() {
        return Some(LineAct::Card(GRAPESHOT));
    }
    let gs_or_none = if gs { Some(LineAct::Card(GRAPESHOT)) } else { None };
    let bf = st.w(BRAIN_FREEZE).is_some() && ready(st, BRAIN_FREEZE);
    if st.u == 0 || st.land == 0 {
        // no blue left: a Diamond or a Petal makes it
        if led_ready(bf_res) && bf {
            return Some(LineAct::Card(LED));
        }
        if petal_ready && bf {
            return Some(LineAct::Card(PETAL));
        }
        return gs_or_none;
    }
    if st.land + st.float < 2 {
        // blue, but short of the generic {1}
        if led_ready(bf_res) && bf {
            return Some(LineAct::Card(LED));
        }
        if let Some(r) = ritual(bf_res + petal_res) {
            return Some(r);
        }
        if petal_ready && bf {
            return Some(LineAct::Card(PETAL));
        }
        return gs_or_none;
    }
    if st.opp_hand as i64 - 3 >= 2
        && st.w(JESKA).is_some()
        && (st.w(JESKA) == hand || st.fuel >= 3 + bf_res + petal_res)
        && can(st, 2, "R")
    {
        return Some(LineAct::Card(JESKA)); // Jeska's Will: a big hand is a big mana burst
    }
    if let Some(r) = ritual(bf_res + petal_res) {
        return Some(r); // spare fuel first becomes storm
    }
    if st.w(LED).is_some() && led_ready(bf_res + 3) {
        return Some(LineAct::Card(LED)); // (a Diamond escape: storm and three more mana)
    }
    if bf {
        return Some(LineAct::Card(BRAIN_FREEZE));
    }
    gs_or_none
}

/// mine._pay_line: pay for a spell of the line: the pip from a real source ({B} may use ritual mana), generic from
/// ritual mana first
fn pay_line(g: &mut Game, p: PlayerId, gn: u32, col: &str) -> Res<bool> {
    let (fb, fr, fa) = ritual_float(g, p);
    if col == "B" && fb + fa > 0 {
        g.player_mut(p).mana.take_kind(if fb > 0 { Kind::B } else { Kind::Any }, 1); // black ritual mana
    } else if col == "R" && fr > 0 {
        g.player_mut(p).mana.take_kind(Kind::R, 1); // red ritual mana
    } else if !pay(g, p, 0, col, false)? {
        return Ok(false);
    }
    // generic from ritual mana, black first (red also pays Grapeshot, Breach and the red rituals)
    let mut k = 0;
    for kind in [Kind::B, Kind::Any, Kind::R] {
        let t = g.player(p).mana.get(kind).min(gn - k);
        g.player_mut(p).mana.take_kind(kind, t);
        k += t;
    }
    Ok(gn - k == 0 || pay(g, p, gn - k, "", false)?)
}

/// escape: exile three other graveyard cards (the least useful, line cards and `skip` kept); false if there aren't
/// three
fn escape_fuel(g: &mut Game, p: PlayerId, skip: Option<CardId>) -> bool {
    let mut fuel: Vec<(f64, CardId)> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&x| !is_line_card(g, x) && Some(x) != skip)
        .map(|x| (card_worth(g, p, x, true), x))
        .collect();
    if fuel.len() < 3 {
        return false;
    }
    fuel.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let pl = g.player_mut(p);
    for &(_, x) in &fuel[..3] {
        remove_card(&mut pl.gy, x);
        pl.exile.push(x);
    }
    pl.stat("breach_escapes", 1);
    true
}

fn zone_sym(w: Wh) -> Sym {
    if w == Wh::Gy { "escape" } else { "hand" }
}

/// diagnostics: why a dry run fell short, as a code the report reads (1 mana, 2 blue mana, 3 graveyard fuel, 4 storm
/// count / anything else); a rough first-fit reading of the position, never used to decide anything
fn line_shortfall(st: &LineState) -> i64 {
    let mana = st.land + st.float;
    let no_blue = st.u == 0 && st.w(GRAPESHOT).is_none() && st.w(PETAL).is_none() && !st.led_bf && st.w(LED).is_none();
    if mana < 3 {
        1
    } else if no_blue {
        2
    } else if st.fuel < 3 {
        3
    } else {
        4
    }
}

/// mine._dry_run: would the line finish every opponent if nobody interacts? Played for real on a copy of the game
/// with the opponents' hands removed (so it neither cheats nor fears their counters); cached for the position. Inside
/// a look-ahead playout (itself a copy) it reuses the real game's answer for this turn (the copy carries it).
fn dry_run(g: &mut Game, p: PlayerId) -> Res<bool> {
    let stamp = g.turn_stamp();
    if g.in_search {
        return Ok(matches!(&g.breach_dry, Some((st, _, ok)) if *st == stamp && *ok));
    }
    let pl = g.player(p);
    let mut key = vec![
        total_mana(g, p, false),
        {
            let (b, r, a) = ritual_float(g, p);
            b + r + a
        },
        pl.gy.len() as u32,
        pl.hand.len() as u32,
        pl.library.len() as u32,
        g.players.iter().map(|q| casts_this_turn(g, q.id)).sum::<i32>() as u32,
    ];
    key.extend(g.players.iter().map(|q| q.library.len() as u32));
    if let Some((st, k, ok)) = &g.breach_dry
        && *st == stamp
        && *k == key
    {
        return Ok(*ok);
    }
    let real = !g.in_search;
    let island = g.db.id("Island").unwrap();
    let (mut ok, mut no_setup) = (false, false);
    for ns in [false, true] {
        // with Lab Man / Birgi / Jeska's Will, then the plain line
        no_setup = ns;
        let mut g2 = crate::ai::search::clone(g);
        for q in g2.opps(p).collect::<Vec<_>>() {
            let n = g2.player(q).hand.len(); // no answers in hand, but the same number of cards (Jeska's Will)
            g2.player_mut(q).hand = vec![island; n];
        }
        g2.line_no_setup = ns;
        breach_line(&mut g2, p, true)?;
        ok = g2.winner == Some(p) || g2.opps(p).all(|q| !g2.player(q).alive || g2.player(q).library.is_empty());
        if ok {
            break;
        }
    }
    g.line_no_setup = ok && no_setup; // the real line follows the plan that worked
    if real {
        let code = if ok { 0 } else { line_shortfall(&line_state(g, p)) };
        let st = &mut g.player_mut(p).stats;
        st.insert("bl_dry", 1);
        if ok {
            st.insert("bl_pass", 1);
        } else {
            st.insert("bl_short", code); // the latest failed dry run
        }
    }
    g.breach_dry = Some((stamp, key, ok));
    Ok(ok)
}

/// mine.breach_line: the Underworld Breach line (Sauron's; any deck holding Breach and a storm card runs it). Dry run
/// (execute false): does it finish every opponent (burned to 0 or milled out) if nobody interacts? Execute: cast it for real, one spell at a time (each can be countered).
pub fn breach_line(g: &mut Game, p: PlayerId, execute: bool) -> Res<bool> {
    if g.opps(p).next().is_none() {
        return Ok(false);
    }
    if !crate::ai::decks::breach_pieces(g, p) {
        return Ok(false); // any deck holding the pieces may run the line (cheap check first)
    }
    let st = line_state(g, p);
    if !st.breach && !st.breach_hand {
        return Ok(false);
    }
    if !STORM_PIECES.iter().any(|n| st.w(n).is_some()) {
        return Ok(false);
    }
    if !execute {
        // Rust only (a fix): a line with nothing left to do (every opponent's library already empty, or no first cast
        // it can make) isn't offered. Python's dry run counted empty libraries as the line finishing the table, so the
        // AI kept choosing the 15.0 "Underworld Breach line" option, which did nothing, until the main-phase action
        // cap. The first step is looked at with the setup casts allowed (the line's widest choice).
        let saved = g.line_no_setup;
        g.line_no_setup = false;
        let st0 = line_state(g, p);
        g.line_no_setup = saved;
        let target = st0.lib.iter().any(|&(q, n)| n > 0 && st0.life_of(q) > 0); // (the line's own test of who's left)
        if !target || line_step(&st0).is_none() {
            return Ok(false);
        }
        return dry_run(g, p);
    }
    let turns = g.player(p).turns;
    let pl = g.player_mut(p);
    pl.stat("breach_line", 1);
    pl.milestone.entry("combo").or_insert(turns);
    crate::glog!(g, "  {} goes for the Underworld Breach line", g.player(p).name);
    for _ in 0..300 {
        if g.over || !g.player(p).alive {
            break;
        }
        let st = line_state(g, p);
        let Some(act) = line_step(&st) else { break };
        match act {
            LineAct::Breach => {
                let c = *g.player(p).hand.iter().find(|&&x| g.db.get(x).tag(Tag::Breach)).unwrap();
                if !pay_line(g, p, 1, "R")? {
                    break;
                }
                if !cast_card(g, p, c, "hand", Ctx::default())? || !has(g, p, Tag::Breach) {
                    crate::glog!(g, "    ...Underworld Breach is stopped");
                    break;
                }
            }
            LineAct::Card(LED) => {
                if !line_led(g, p, &st)? {
                    break;
                }
            }
            LineAct::Card(JESKA) => {
                let zone = st.w(JESKA).unwrap();
                let c = g.db.id(JESKA).unwrap();
                if zone == Wh::Gy && !escape_fuel(g, p, None) {
                    break;
                }
                if !pay_line(g, p, 2, "R")? {
                    break;
                }
                g.jeska_mana = true; // in the line it's always the mana mode
                let r = cast_card(g, p, c, zone_sym(zone), Ctx::default());
                g.jeska_mana = false;
                r?;
            }
            LineAct::Setup => {
                let (c, gn, pips) = st.setup_hand.clone().unwrap();
                if !pay_line(g, p, gn, &pips)? {
                    break;
                }
                cast_card(g, p, c, "hand", Ctx::default())?;
                if !g
                    .player(p)
                    .perms
                    .iter()
                    .any(|&m| g.perm(m).cd.is_some_and(|x| g.db.get(x).name == g.db.get(c).name))
                {
                    break; // countered
                }
            }
            LineAct::Draw => {
                // Laboratory Maniac / Jace: the draw that wins
                let (c, zone, gn, pip) = draw_ready(&st).unwrap();
                if zone == Wh::Gy && !escape_fuel(g, p, Some(c)) {
                    break;
                }
                if !pay_line(g, p, gn, &pip)? {
                    break;
                }
                cast_card(g, p, c, zone_sym(zone), Ctx::default())?;
            }
            LineAct::Card(act) => {
                let zone = st.w(act).unwrap();
                let c = g.db.id(act).unwrap();
                let mut ctx = Ctx { storm: Some(st.storm), ..Ctx::default() };
                let pay_for: Option<(u32, &str)> = match RITUALS.iter().find(|r| r.0 == act) {
                    Some(r) => Some((r.1, r.4)),
                    None if act == PETAL => None,
                    None => Some((1, if act == BRAIN_FREEZE { "U" } else { "R" })),
                };
                if zone == Wh::Gy && !escape_fuel(g, p, None) {
                    break;
                }
                if let Some((gn, col)) = pay_for
                    && !pay_line(g, p, gn, col)?
                {
                    break;
                }
                if act == BRAIN_FREEZE {
                    ctx.mill = mill_plan(p, &line_state(g, p), st.storm as i64 + 1); // as paid for
                }
                cast_card(g, p, c, zone_sym(zone), ctx)?; // a ritual floats its own colour (black or red)
                if act == PETAL
                    && let Some(m) = perm_named(g, p, PETAL, false)
                {
                    // sacrifice it at once, so it can be escaped again
                    leave(g, m)?;
                    to_zone_card(g, m, Zone::Gy);
                    let fl = &mut g.player_mut(p).mana;
                    if st.labman && st.mylib == 0 {
                        fl.add(Kind::Any, 1); // any colour, for the winning draw
                    } else if st.u == 0 {
                        fl.add(Kind::U, 1);
                    } else if st.r == 0 {
                        fl.add(Kind::R, 1);
                    } else {
                        fl.add(Kind::Any, 1);
                    }
                }
            }
        }
    }
    check_state(g)?;
    if g.opps(p).all(|q| !g.player(q).alive || g.player(q).library.is_empty()) {
        g.player_mut(p).stats.insert("bl_won", 1); // every opponent dead or decked (--analyze)
    }
    Ok(true)
}

/// mine._line_led: Lion's Eye Diamond in the Breach line: cast it (from hand, or escaped from the graveyard) unless
/// it's already out, then discard your hand and sacrifice it for three mana: blue for Brain Freeze, red when only
/// Grapeshot is left
fn line_led(g: &mut Game, p: PlayerId, st: &LineState) -> Res<bool> {
    if !st.led_bf {
        let zone = st.w(LED).unwrap();
        let c = g.db.id(LED).unwrap();
        if zone == Wh::Gy && !escape_fuel(g, p, None) {
            return Ok(false);
        }
        cast_card(g, p, c, zone_sym(zone), Ctx::default())?;
    }
    let Some(m) = perm_named(g, p, LED, true) else { return Ok(false) }; // countered
    let hand = g.player(p).hand.clone();
    discard_cards(g, p, &hand)?;
    leave(g, m)?;
    to_zone_card(g, m, Zone::Gy);
    let col = if st.r == 0 && st.w(GRAPESHOT).is_some() && st.w(BRAIN_FREEZE).is_none() { 'R' } else { 'U' };
    let pl = g.player_mut(p);
    if col == 'U' {
        pl.mana.add(Kind::U, 3);
    } else {
        pl.mana.add(Kind::R, 3);
    }
    pl.stat("led_used", 1);
    crate::glog!(g, "    Lion's Eye Diamond: {} discards the hand for {}", g.player(p).name, col.to_string().repeat(3));
    Ok(true)
}

/// mine.breach_options: the AI's main-phase option: go for the Breach line when the dry run finishes the table
pub fn breach_options(g: &mut Game, p: PlayerId, _post: bool) -> Res<Vec<Opt>> {
    if g.active == Some(p) && !g.in_search && crate::ai::decks::breach_pieces(g, p) {
        g.player_mut(p).stats.insert("bl_held", 1); // diagnostics only (--analyze)
    }
    if g.active != Some(p) || !breach_line(g, p, false)? {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 15.0,
        label: "Underworld Breach line".into(),
        act: Some(Action::Plan { f: breach_go, arg: 0 }),
    }])
}

fn breach_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    breach_line(g, p, true)
}

fn remove_card(xs: &mut Vec<CardId>, c: CardId) -> bool {
    match xs.iter().position(|&x| x == c) {
        Some(i) => {
            xs.remove(i);
            true
        }
        None => false,
    }
}
