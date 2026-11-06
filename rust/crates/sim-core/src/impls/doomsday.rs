//! Doomsday ({B}{B}{B} sorcery): the card, the AI's pile builder and the play of the pile (JD's Tymna/Thrasios list,
//! `decklists/JD/tymna-thrasios-doomsday.md`, "Line 2: Doomsday").
//!
//! * **The card.** "Search your library and graveyard for five cards and exile the rest. Put the chosen cards on top of
//!   your library in any order. You lose half your life, rounded up." `resolve` takes the five (fewer if there are
//!   fewer) from library and graveyard together, exiles every other card of both, stacks the five in the chosen order
//!   (top first) and loses ceil(life / 2) (nothing at 0 life or less). A person picks the five and their order
//!   (`human::choices`); the AI's pile comes from `build`.
//! * **The pile builder (AI).** The guide's Piles A, B and C, and the Thrasios (devotion 3) variants, are not
//!   written out: `solve` is a small exact search over the position after Doomsday (the library is known: it is the
//!   pile). Its moves are the cards of the guide: Silence, Gitaxian Probe (for {U} or 2 life), Brainstorm (and which two
//!   cards go back), Preordain (scry 2), Ponder (the order of the top three), Borne Upon a Wind and Thassa's Oracle,
//!   paid from the mana the player will really have (`Env::ok` asks the engine's `can_pay`) and the life it will have
//!   left. `build` tries ordered choices of the useful cards that are in the library or graveyard, the best filler
//!   under them, in the two timings the guide gives (the end step of the turn before, with the draw step still to
//!   come, or the player's own main phase), and keeps the pile whose line has the lock (Silence) first and is the
//!   shortest. A pile with Pile A's cards is Pile A, and so on; a devotion of 3 or more makes the shorter lines win.
//! * **Playing the pile.** `run_pile` re-solves the position before every spell and casts the first move (every
//!   spell goes through the table's counter window as any cast does); a cantrip that is countered just stalls the
//!   line, and a Thassa's Oracle that is countered into the graveyard is put back on top with Noxious Revival. The
//!   mid-resolution choices of the cantrips (`pile_putback`, `pile_scry`, `pile_ponder`, `pile_probe_life`) ask the same
//!   search, so Brainstorm puts back the right two and Preordain / Ponder keep Silence. The catalogue combo
//!   "Doomsday + Thassa's Oracle" is the pile line of a later turn (the end-step cast leaves it to the next turn's main
//!   phase).
//! * **When** (`doomsday_options`, the card's `hand_options`): at the end step of the turn before the player's own
//!   with an instant-speed enabler (the card's own flash from Teferi's +1, Valley Floodcaller or a Borne Upon a Wind
//!   cast first), else in the main phase when the pile starts from the hand; never with Demonic Consultation or
//!   Tainted Pact in hand (Line 1 is cheaper), never below 16 life or into a board that can deal the life that is
//!   left, and not into several opponents who can stop a spell with no lock in the pile and no counter held.
//! * **Teferi, Time Raveler +1**: "until your next turn you may cast sorcery spells as though they had flash"
//!   (`teferi_plus`, read by `cast::grants_flash`).

use super::combos::{Combo, Ready, Step, StepKind, no_keys, no_src};
use crate::engine::cast::{cast_card, castable, grants_flash};
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, cost_of, pay, rituals_in_hand};
use crate::engine::tutors::card_worth;
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game, Step as TurnStep};
use crate::sym::Sym;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;

pub const DOOMSDAY: &str = "Doomsday";
pub const NAME: &str = "Doomsday + Thassa's Oracle";
const ORACLE: &str = "Thassa's Oracle";
const REVIVAL: &str = "Noxious Revival";
/// the pile is not started (and a Probe is not paid in life) below this much life
const LIFE_FLOOR: i32 = 3;
/// Doomsday is not cast below this much life (the guide: "Don't plan a Doomsday turn below about 16 life")
const LIFE_NEEDED: i32 = 16;

// ------------------------------------------------------------------ Teferi, Time Raveler +1
/// +1: until your next turn you may cast sorcery spells as though they had flash
pub fn teferi_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let turns = g.player(p).turns;
    g.player_mut(p).sorc_flash = Some(turns);
    Ok(())
}

/// is p's Teferi grant in force: used on this turn or since it, and p's next turn has not begun
pub fn teferi_flash(g: &Game, p: PlayerId) -> bool {
    g.player(p).sorc_flash == Some(g.player(p).turns)
}

/// the AI's value of +1: high with a Doomsday in hand (it makes the end-step cast possible), otherwise the old 1.5
pub fn teferi_plus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let doom = g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == DOOMSDAY);
    Some(if doom && !teferi_flash(g, p) { 6.0 } else { 1.5 })
}

// ------------------------------------------------------------------ the position
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum K {
    Oracle,
    Silence,
    Probe,
    Brainstorm,
    Preordain,
    Ponder,
    Borne,
    Other,
}

fn kind_of(n: &str) -> K {
    match n {
        ORACLE => K::Oracle,
        "Silence" => K::Silence,
        "Gitaxian Probe" => K::Probe,
        "Brainstorm" => K::Brainstorm,
        "Preordain" => K::Preordain,
        "Ponder" => K::Ponder,
        "Borne Upon a Wind" => K::Borne,
        _ => K::Other,
    }
}

fn kd(g: &Game, c: CardId) -> K {
    kind_of(&g.db.get(c).name)
}

/// one move of a line: cast this card (a Probe for life when `life`)
#[derive(Clone, Copy, Debug)]
pub struct Act {
    pub card: CardId,
    pub life: bool,
}

/// the position the search plays from: the hand, the library (top first), the mana the line has spent so far (as
/// colour counts the engine can be asked about), the life left and whether Silence is up
#[derive(Clone)]
struct S {
    hand: Vec<CardId>,
    lib: Vec<CardId>,
    u: u8,
    w: u8,
    gn: u8,
    life: i32,
    silence: bool,
}

struct Env<'a> {
    g: Cow<'a, Game>,
    p: PlayerId,
    /// {B}s the line still owes before anything else (Doomsday itself, when it is not cast yet)
    b: u8,
    /// devotion to blue without Thassa's Oracle
    dev: usize,
    kinds: HashMap<u16, K>,
    memo: RefCell<HashMap<u32, bool>>,
}

impl<'a> Env<'a> {
    fn new(g: Cow<'a, Game>, p: PlayerId, b: u8, cards: &[CardId]) -> Env<'a> {
        let dev = super::combos::oracle_devotion(&g, p);
        let kinds = cards.iter().map(|&c| (c.0, kd(&g, c))).collect();
        Env { g, p, b, dev, kinds, memo: RefCell::new(HashMap::new()) }
    }
    fn kd(&self, c: CardId) -> K {
        self.kinds.get(&c.0).copied().unwrap_or(K::Other)
    }
    /// can the player pay {B}^b + {U}^u + {W}^w + generic gn at once
    fn ok(&self, u: u8, w: u8, gn: u8) -> bool {
        let key = ((((self.b as u32) * 16 + u as u32) * 16 + w as u32) * 16) + gn as u32;
        if let Some(&r) = self.memo.borrow().get(&key) {
            return r;
        }
        let pips = format!("{}{}{}", "B".repeat(self.b as usize), "U".repeat(u as usize), "W".repeat(w as usize));
        let r = can_pay(&self.g, self.p, gn as u32, &pips, false);
        self.memo.borrow_mut().insert(key, r);
        r
    }
}

/// is a lock in force on every opponent this turn: Silence's cast lock (`cast_locks`, no opponent can cast a spell), or
/// Ranger-Captain of Eos's
pub fn silence_up(g: &Game, p: PlayerId) -> bool {
    if g.silence == Some((g.turn_stamp(), p)) {
        return true;
    }
    let Some(probe) = g.db.id("Silence") else { return false };
    let mut any = false;
    for q in g.opps(p) {
        if !g.player(q).alive {
            continue;
        }
        any = true;
        if !crate::engine::cast::spell_locked(g, q, probe) {
            return false;
        }
    }
    any
}

fn half_up(life: i32) -> i32 {
    if life <= 0 { 0 } else { (life + 1) / 2 }
}

/// the position now (a cantrip resolving, or the main phase of a pile turn); `exclude` is the spell that is resolving
fn state_now(g: &Game, p: PlayerId, exclude: Option<CardId>) -> S {
    let pl = g.player(p);
    S {
        hand: pl.hand.iter().copied().filter(|&c| Some(c) != exclude).collect(),
        lib: pl.library.iter().rev().copied().collect(),
        u: 0,
        w: 0,
        gn: 0,
        life: pl.life,
        silence: silence_up(g, p),
    }
}

fn env_now<'a>(g: &'a Game, p: PlayerId, s: &S) -> Env<'a> {
    let mut cards = s.hand.clone();
    cards.extend(&s.lib);
    Env::new(Cow::Borrowed(g), p, 0, &cards)
}

fn draw_n(t: &mut S, n: usize) -> bool {
    if t.lib.len() < n {
        return false; // drawing from an empty library loses
    }
    let drawn: Vec<CardId> = t.lib.drain(..n).collect();
    t.hand.extend(drawn);
    true
}

fn orders(cards: &[CardId]) -> Vec<Vec<CardId>> {
    match cards.len() {
        0 => vec![vec![]],
        1 => vec![cards.to_vec()],
        _ => {
            let mut out = vec![];
            for i in 0..cards.len() {
                let mut rest = cards.to_vec();
                let c = rest.remove(i);
                for mut tail in orders(&rest) {
                    tail.insert(0, c);
                    out.push(tail);
                }
            }
            out
        }
    }
}

/// the ways to scry the top cards: (kept on top in order, put on the bottom)
fn scry_variants(top: &[CardId]) -> Vec<(Vec<CardId>, Vec<CardId>)> {
    match top.len() {
        0 => vec![(vec![], vec![])],
        1 => vec![(vec![top[0]], vec![]), (vec![], vec![top[0]])],
        _ => {
            let (a, b) = (top[0], top[1]);
            vec![
                (vec![a, b], vec![]),
                (vec![b, a], vec![]),
                (vec![a], vec![b]),
                (vec![b], vec![a]),
                (vec![], vec![a, b]),
                (vec![], vec![b, a]),
            ]
        }
    }
}

fn step_to(e: &Env, t: S, a: Act, left: u8, lock: bool, acts: &mut Vec<Act>) -> bool {
    acts.push(a);
    if dfs(e, &t, left - 1, lock, acts) {
        return true;
    }
    acts.pop();
    false
}

/// is there a line of at most `left` more spells that ends with Thassa's Oracle on the battlefield and a devotion at
/// least the library (`lock`: with Silence up before the Oracle)? On success `acts` holds it
fn dfs(e: &Env, s: &S, left: u8, lock: bool, acts: &mut Vec<Act>) -> bool {
    if left == 0 {
        return false;
    }
    for i in 0..s.hand.len() {
        let c = s.hand[i];
        let k = e.kd(c);
        if k == K::Other || s.hand[..i].iter().any(|&x| x == c) {
            continue;
        }
        let mut base = s.clone();
        base.hand.remove(i);
        let a = Act { card: c, life: false };
        match k {
            K::Silence => {
                if s.silence || !e.ok(s.u, s.w + 1, s.gn) {
                    continue;
                }
                base.w += 1;
                base.silence = true;
                if step_to(e, base, a, left, lock, acts) {
                    return true;
                }
            }
            K::Oracle => {
                if e.ok(s.u + 2, s.w, s.gn) && e.dev + 2 >= s.lib.len() && (!lock || s.silence) {
                    acts.push(a);
                    return true;
                }
            }
            K::Probe => {
                if e.ok(s.u + 1, s.w, s.gn) {
                    let mut t = base.clone();
                    t.u += 1;
                    if draw_n(&mut t, 1) && step_to(e, t, a, left, lock, acts) {
                        return true;
                    }
                }
                if s.life - 2 >= LIFE_FLOOR {
                    let mut t = base.clone();
                    t.life -= 2;
                    if draw_n(&mut t, 1) && step_to(e, t, Act { life: true, ..a }, left, lock, acts) {
                        return true;
                    }
                }
            }
            K::Borne => {
                if e.ok(s.u + 1, s.w, s.gn + 1) {
                    let mut t = base.clone();
                    t.u += 1;
                    t.gn += 1;
                    if draw_n(&mut t, 1) && step_to(e, t, a, left, lock, acts) {
                        return true;
                    }
                }
            }
            K::Brainstorm => {
                if !e.ok(s.u + 1, s.w, s.gn) {
                    continue;
                }
                let mut t = base.clone();
                t.u += 1;
                if !draw_n(&mut t, 3) {
                    continue;
                }
                let mut seen: Vec<(K, K)> = vec![];
                for x in 0..t.hand.len() {
                    for y in 0..t.hand.len() {
                        if x == y {
                            continue;
                        }
                        let (cx, cy) = (t.hand[x], t.hand[y]);
                        let key = (e.kd(cx), e.kd(cy));
                        if seen.contains(&key) {
                            continue;
                        }
                        seen.push(key);
                        // cx goes back first, cy on top of it
                        let mut t2 = t.clone();
                        t2.hand.retain(|&h| h != cx && h != cy);
                        t2.lib.insert(0, cx);
                        t2.lib.insert(0, cy);
                        if step_to(e, t2, a, left, lock, acts) {
                            return true;
                        }
                    }
                }
            }
            K::Preordain => {
                if !e.ok(s.u + 1, s.w, s.gn) {
                    continue;
                }
                let n = s.lib.len().min(2);
                for (keep, bottom) in scry_variants(&s.lib[..n]) {
                    let mut t = base.clone();
                    t.u += 1;
                    t.lib = keep.into_iter().chain(s.lib[n..].iter().copied()).chain(bottom).collect();
                    if draw_n(&mut t, 1) && step_to(e, t, a, left, lock, acts) {
                        return true;
                    }
                }
            }
            K::Ponder => {
                if !e.ok(s.u + 1, s.w, s.gn) {
                    continue;
                }
                let n = s.lib.len().min(3);
                for ord in orders(&s.lib[..n]) {
                    let mut t = base.clone();
                    t.u += 1;
                    t.lib = ord.into_iter().chain(s.lib[n..].iter().copied()).collect();
                    if draw_n(&mut t, 1) && step_to(e, t, a, left, lock, acts) {
                        return true;
                    }
                }
            }
            K::Other => {}
        }
    }
    false
}

/// the shortest line from `s`, with the lock first when `lock`
fn solve(e: &Env, s: &S, lock: bool) -> Option<Vec<Act>> {
    let has_oracle = s.hand.iter().chain(&s.lib).any(|&c| e.kd(c) == K::Oracle);
    if !has_oracle {
        return None;
    }
    for d in 1..=7 {
        let mut acts = vec![];
        if dfs(e, s, d, lock, &mut acts) {
            return Some(acts);
        }
    }
    None
}

/// how good a position is: (the line has the lock, minus its length); None when there is no line
fn rank(e: &Env, s: &S) -> Option<(i32, i32)> {
    if let Some(a) = solve(e, s, true) {
        return Some((1, -(a.len() as i32)));
    }
    solve(e, s, false).map(|a| (0, -(a.len() as i32)))
}

/// the line from `s`: with the lock when `want_lock` and one exists
fn line(e: &Env, s: &S, want_lock: bool) -> Option<Vec<Act>> {
    if want_lock {
        if let Some(a) = solve(e, s, true) {
            return Some(a);
        }
    }
    solve(e, s, false)
}

/// how many opponents could stop one spell of ours right now
fn stoppers(g: &Game, p: PlayerId) -> usize {
    g.opps(p).filter(|&q| crate::ai::t6_table::stop_chance(g, q) >= 0.15).count()
}

// ------------------------------------------------------------------ the pile builder
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scn {
    /// the end step of the turn before the player's own: the draw step takes the first card of the pile
    EndStep,
    /// the player's own main phase
    Main,
}

#[derive(Clone, Debug)]
pub struct Built {
    /// the five cards, top first
    pub pile: Vec<CardId>,
    pub plan: Vec<Act>,
    pub lock: bool,
    pub life_after: i32,
}

thread_local! {
    static CACHE: RefCell<HashMap<u64, Option<Built>>> = RefCell::new(HashMap::new());
}

fn cache_key(g: &Game, p: PlayerId, scn: Scn, b: u8, skip: &[CardId]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    let pl = g.player(p);
    (p.0, scn as u8, b, pl.life, g.round, g.step as u8).hash(&mut h);
    for c in pl.hand.iter().chain(&pl.library).chain(&pl.gy).chain(skip) {
        c.0.hash(&mut h);
    }
    (pl.hand.len(), pl.library.len(), pl.gy.len(), crate::engine::mana::total_mana(g, p, false)).hash(&mut h);
    let board: Vec<u32> = pl.perms.iter().map(|&m| m.0 as u32).collect();
    board.hash(&mut h);
    silence_up(g, p).hash(&mut h);
    h.finish()
}

/// The best pile for the position, or None when no pile wins. `b` is the {B} Doomsday still has to be paid with (3 when
/// it is not cast yet), `skip` hand cards that will be spent before it (the enabler). The pile is chosen from the
/// cards of the library and graveyard; the hand is what the line starts with.
pub fn build(g: &Game, p: PlayerId, scn: Scn, b: u8, skip: &[CardId]) -> Option<Built> {
    let key = cache_key(g, p, scn, b, skip);
    if let Some(r) = CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return r;
    }
    let r = build_uncached(g, p, scn, b, skip);
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 256 {
            c.clear();
        }
        c.insert(key, r.clone());
    });
    r
}

fn build_uncached(g: &Game, p: PlayerId, scn: Scn, b: u8, skip: &[CardId]) -> Option<Built> {
    let pl = g.player(p);
    let hand: Vec<CardId> =
        pl.hand.iter().copied().filter(|&c| &*g.db.get(c).name != DOOMSDAY && !skip.contains(&c)).collect();
    let avail: Vec<CardId> = pl.library.iter().chain(&pl.gy).copied().collect();
    if avail.is_empty() {
        return None;
    }
    let n_pick = avail.len().min(5);
    let life_after = pl.life - half_up(pl.life);
    let in_hand_oracle = hand.iter().any(|&c| kd(g, c) == K::Oracle);
    if !in_hand_oracle && !avail.iter().any(|&c| kd(g, c) == K::Oracle) {
        return None;
    }
    let draws = scn == Scn::EndStep
        && !pl
            .perms
            .iter()
            .any(|&m| g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == "Necropotence") && !g.perm(m).phased);
    let mana: Cow<Game> = if scn == Scn::EndStep {
        // the line is played on the player's own turn: everything untaps first
        let mut g2 = g.clone();
        for l in g2.player(p).lands.clone() {
            g2.perm_mut(l).tapped = false;
        }
        for m in g2.player(p).perms.clone() {
            g2.perm_mut(m).tapped = false;
            g2.perm_mut(m).sick = false;
        }
        Cow::Owned(g2)
    } else {
        Cow::Borrowed(g)
    };
    let mut all = hand.clone();
    all.extend(&avail);
    let env = Env::new(mana, p, if scn == Scn::EndStep { 0 } else { b }, &all);
    let useful: Vec<CardId> = avail.iter().copied().filter(|&c| env.kd(c) != K::Other).collect();
    let mut fillers: Vec<CardId> = avail.iter().copied().filter(|&c| env.kd(c) == K::Other).collect();
    fillers.sort_by(|&x, &y| {
        card_worth(g, p, y, false).partial_cmp(&card_worth(g, p, x, false)).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut best: Option<((i32, i32), Built)> = None;
    for m in 0..=useful.len().min(n_pick) {
        let mut seqs: Vec<Vec<CardId>> = vec![];
        gen_seqs(&useful, m, &mut vec![], &mut seqs);
        for seq in seqs {
            let mut pile = seq.clone();
            for &f in &fillers {
                if pile.len() >= n_pick {
                    break;
                }
                pile.push(f);
            }
            for &u in &useful {
                if pile.len() >= n_pick {
                    break;
                }
                if !pile.contains(&u) {
                    pile.push(u);
                }
            }
            let mut s =
                S { hand: hand.clone(), lib: pile.clone(), u: 0, w: 0, gn: 0, life: life_after, silence: false };
            if draws && !draw_n(&mut s, 1) {
                continue;
            }
            let (lock, plan) = match solve(&env, &s, true) {
                Some(a) => (true, a),
                None => match solve(&env, &s, false) {
                    Some(a) => (false, a),
                    None => continue,
                },
            };
            let score = (lock as i32, -(plan.len() as i32));
            if best.as_ref().is_none_or(|(sc, _)| score > *sc) {
                best = Some((score, Built { pile, plan, lock, life_after }));
            }
        }
        if best.as_ref().is_some_and(|(sc, _)| sc.0 == 1) {
            break; // a pile with the lock: more useful cards in it would not make it better
        }
    }
    best.map(|x| x.1)
}

fn gen_seqs(pool: &[CardId], m: usize, cur: &mut Vec<CardId>, out: &mut Vec<Vec<CardId>>) {
    if cur.len() == m {
        out.push(cur.clone());
        return;
    }
    for &c in pool {
        if !cur.contains(&c) {
            cur.push(c);
            gen_seqs(pool, m, cur, out);
            cur.pop();
        }
    }
}

// ------------------------------------------------------------------ the card
fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0 // cast only through `doomsday_options`
}

/// Doomsday resolves: the five chosen cards on top (top first), everything else of library and graveyard exiled, then
/// half the life (rounded up) is lost
fn resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let chosen = if crate::human::is_human(g, p) { human_pile(g, p)? } else { ai_pile(g, p) };
    apply(g, p, &chosen)?;
    Ok("gy")
}

fn ai_pile(g: &Game, p: PlayerId) -> Vec<CardId> {
    let scn = if g.active == Some(p) { Scn::Main } else { Scn::EndStep };
    if let Some(b) = build(g, p, scn, 0, &[]) {
        return b.pile;
    }
    // no pile wins (the cast was forced, or the position changed): the cards worth most, the line's cards first
    let pl = g.player(p);
    let mut avail: Vec<CardId> = pl.library.iter().chain(&pl.gy).copied().collect();
    avail.sort_by(|&x, &y| {
        let key = |c: CardId| (kd(g, c) != K::Other, card_worth(g, p, c, false));
        key(y).partial_cmp(&key(x)).unwrap_or(std::cmp::Ordering::Equal)
    });
    avail.truncate(5);
    avail
}

fn human_pile(g: &Game, p: PlayerId) -> Res<Vec<CardId>> {
    let pl = g.player(p);
    let pool: Vec<CardId> = pl.library.iter().rev().chain(&pl.gy).copied().collect();
    let n = pool.len().min(5);
    let picked = crate::human::choices::pick_cards(
        g,
        p,
        &pool,
        n,
        "Doomsday: choose five cards from your library and graveyard (the rest are exiled)",
        "choose",
    )?;
    if picked.len() < 2 {
        return Ok(picked);
    }
    let mut order = crate::human::choices::pick_cards(
        g,
        p,
        &picked,
        picked.len() - 1,
        "Doomsday: put the cards on top of your library, the top card first",
        "choose",
    )?;
    if let Some(&last) = picked.iter().find(|c| !order.contains(c)) {
        order.push(last);
    }
    Ok(order)
}

/// the effect: `chosen` (top first) on top of the library, the rest of library and graveyard exiled, half the life lost
fn apply(g: &mut Game, p: PlayerId, chosen: &[CardId]) -> Res {
    let pl = g.player_mut(p);
    let mut all: Vec<CardId> = pl.library.drain(..).collect();
    all.extend(pl.gy.drain(..));
    let mut keep: Vec<CardId> = vec![];
    for &c in chosen.iter().take(5) {
        if let Some(i) = all.iter().position(|&x| x == c) {
            all.remove(i);
            keep.push(c);
        }
    }
    let n_exiled = all.len();
    pl.exile.extend(all);
    pl.library = keep.iter().rev().copied().collect();
    pl.stat("doomsday", 1);
    let life = pl.life;
    let names: Vec<String> = keep.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    crate::glog!(
        g,
        "    Doomsday: {} exiles {} cards and keeps {} on top",
        g.player(p).name,
        n_exiled,
        names.join(", ")
    );
    lose_life(g, p, half_up(life), Some(p), "other", None)
}

// ------------------------------------------------------------------ the pile is in play
/// has p's Doomsday resolved (the AI plays the library as a pile from then on)
pub fn active(g: &Game, p: PlayerId) -> bool {
    !crate::human::is_human(g, p) && g.player(p).stats.get("doomsday").copied().unwrap_or(0) > 0
}

/// Gitaxian Probe's payment while the pile is played: Some(true) pay the 2 life, Some(false) pay {U}
pub fn pile_probe_life(g: &Game, p: PlayerId) -> Option<bool> {
    if !active(g, p) {
        return None;
    }
    let s = state_now(g, p, None);
    let e = env_now(g, p, &s);
    let mut u = s.clone();
    u.u = 1;
    let mut l = s.clone();
    l.life -= 2;
    let (ru, rl) = (
        if e.ok(1, 0, 0) && draw_n(&mut u, 1) { rank(&e, &u) } else { None },
        if s.life - 2 >= LIFE_FLOOR && draw_n(&mut l, 1) { rank(&e, &l) } else { None },
    );
    match (ru, rl) {
        (Some(a), Some(b)) => Some(b > a),
        (None, Some(_)) => Some(true),
        (Some(_), None) => Some(false),
        (None, None) => None,
    }
}

/// Brainstorm's two cards to put back (the first goes back first, the second ends on top), by the line that wins
pub fn pile_putback(g: &Game, p: PlayerId, n: usize, exclude: Option<CardId>) -> Option<Vec<CardId>> {
    if !active(g, p) || n != 2 {
        return None;
    }
    let s = state_now(g, p, exclude);
    let e = env_now(g, p, &s);
    let mut best: Option<((i32, i32), Vec<CardId>)> = None;
    for &x in &s.hand {
        for &y in &s.hand {
            if x == y {
                continue;
            }
            let mut t = s.clone();
            t.hand.retain(|&h| h != x && h != y);
            t.lib.insert(0, x);
            t.lib.insert(0, y);
            if let Some(r) = rank(&e, &t)
                && best.as_ref().is_none_or(|(b, _)| r > *b)
            {
                best = Some((r, vec![x, y]));
            }
        }
    }
    best.map(|x| x.1)
}

/// Preordain's scry 2: (kept on top in order, put on the bottom)
pub fn pile_scry(g: &Game, p: PlayerId, n: usize) -> Option<(Vec<CardId>, Vec<CardId>)> {
    if !active(g, p) || n != 2 {
        return None;
    }
    let s = state_now(g, p, None);
    let e = env_now(g, p, &s);
    let k = s.lib.len().min(2);
    let mut best: Option<((i32, i32), (Vec<CardId>, Vec<CardId>))> = None;
    for (keep, bottom) in scry_variants(&s.lib[..k]) {
        let mut t = s.clone();
        t.lib = keep.iter().copied().chain(s.lib[k..].iter().copied()).chain(bottom.iter().copied()).collect();
        if !draw_n(&mut t, 1) {
            continue;
        }
        if let Some(r) = rank(&e, &t)
            && best.as_ref().is_none_or(|(b, _)| r > *b)
        {
            best = Some((r, (keep, bottom)));
        }
    }
    best.map(|x| x.1)
}

/// Ponder: the order of the top three, top first (never shuffled)
pub fn pile_ponder(g: &Game, p: PlayerId) -> Option<Vec<CardId>> {
    if !active(g, p) {
        return None;
    }
    let s = state_now(g, p, None);
    let e = env_now(g, p, &s);
    let k = s.lib.len().min(3);
    let mut best: Option<((i32, i32), Vec<CardId>)> = None;
    for ord in orders(&s.lib[..k]) {
        let mut t = s.clone();
        t.lib = ord.iter().copied().chain(s.lib[k..].iter().copied()).collect();
        if !draw_n(&mut t, 1) {
            continue;
        }
        if let Some(r) = rank(&e, &t)
            && best.as_ref().is_none_or(|(b, _)| r > *b)
        {
            best = Some((r, ord));
        }
    }
    best.map(|x| x.1)
}

/// Thassa's Oracle waits for the lock while the pile is played and Silence can still be cast
pub fn hold_oracle(g: &Game, p: PlayerId) -> bool {
    if !active(g, p) || silence_up(g, p) || stoppers(g, p) == 0 {
        return false;
    }
    let Some(&sil) = g.player(p).hand.iter().find(|&&c| kd(g, c) == K::Silence) else { return false };
    castable(g, p, sil, "hand") && can_pay(g, p, 0, "W", false)
}

/// the line from the current position, with the lock when opponents can stop a spell
fn plan_now(g: &Game, p: PlayerId) -> Option<Vec<Act>> {
    let s = state_now(g, p, None);
    let e = env_now(g, p, &s);
    line(&e, &s, stoppers(g, p) > 0)
}

fn cast_one(g: &mut Game, p: PlayerId, c: CardId) -> Res<bool> {
    if !g.player(p).hand.contains(&c) || !castable(g, p, c, "hand") {
        return Ok(false);
    }
    let (gn, pips) = cost_of(g, p, c);
    if !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn, &pips, false)?;
    cast_card(g, p, c, "hand", Ctx::default())
}

/// Play the pile: re-solve before every spell, cast the first move of the line, stop when there is no line, a spell
/// cannot be cast, or the game is over. Thassa's Oracle in the graveyard comes back with Noxious Revival when there is
/// no line. True when the game was won.
pub fn run_pile(g: &mut Game, p: PlayerId) -> Res<bool> {
    let mut failed = 0;
    for _ in 0..12 {
        if g.over || !g.player(p).alive {
            break;
        }
        let Some(plan) = plan_now(g, p) else {
            if !revive_oracle(g, p)? {
                break;
            }
            continue;
        };
        let a = plan[0];
        crate::glog!(g, "    {} plays the pile: {}", g.player(p).name, g.db.get(a.card).name);
        if !cast_one(g, p, a.card)? {
            failed += 1;
            if failed >= 2 {
                break;
            }
            continue;
        }
    }
    Ok(g.over && g.winner == Some(p))
}

/// Noxious Revival on a Thassa's Oracle that was countered into the graveyard (paid with {G}, or 2 life)
fn revive_oracle(g: &mut Game, p: PlayerId) -> Res<bool> {
    let pl = g.player(p);
    let Some(&rev) = pl.hand.iter().find(|&&c| &*g.db.get(c).name == REVIVAL) else { return Ok(false) };
    if !pl.gy.iter().any(|&c| &*g.db.get(c).name == ORACLE) || pl.life <= LIFE_FLOOR + 2 {
        return Ok(false);
    }
    crate::glog!(g, "    {} puts Thassa's Oracle back on top with Noxious Revival", g.player(p).name);
    cast_one(g, p, rev)
}

// ------------------------------------------------------------------ when to cast it
fn opp_power(g: &Game, q: PlayerId) -> i32 {
    g.player(q)
        .perms
        .iter()
        .filter(|&&m| !g.perm(m).phased && g.is_creature(m))
        .map(|&m| crate::engine::values::epow(g, m).max(0))
        .sum()
}

/// the instant-speed enabler to cast first (Valley Floodcaller, else Borne Upon a Wind), when nothing grants flash now
fn enabler(g: &Game, p: PlayerId) -> Option<CardId> {
    let hand = &g.player(p).hand;
    ["Valley Floodcaller", "Borne Upon a Wind"]
        .iter()
        .find_map(|n| hand.iter().copied().find(|&c| &*g.db.get(c).name == *n))
}

/// can the player pay Doomsday (with the cost of `extra` more, an enabler) now: Some(b) is the {B} still owed when a
/// Dark Ritual makes the rest
fn doomsday_cost(g: &Game, p: PlayerId, c: CardId, extra: Option<CardId>) -> Option<u8> {
    let (mut gn, mut pips) = cost_of(g, p, c);
    if let Some(e) = extra {
        let (eg, ep) = cost_of(g, p, e);
        gn += eg;
        pips.push_str(&ep);
    }
    if can_pay(g, p, gn, &pips, false) {
        return Some(3);
    }
    // a Dark Ritual ({B}: add {B}{B}{B}) pays the {B}{B}{B}
    let ritual = rituals_in_hand(g, p, &[]).into_iter().any(|r| &*g.db.get(r).name == "Dark Ritual");
    let rest = pips.replacen("BBB", "B", 1);
    (ritual && rest != pips && can_pay(g, p, gn, &rest, false)).then_some(1)
}

/// the pile to build for a Doomsday in hand now, or None: Doomsday is not the line (Line 1 is in hand), it cannot be
/// paid or cast, the life is too low, a board can deal the life that is left, or no pile wins
pub fn plan_cast(g: &Game, p: PlayerId, c: CardId, scn: Scn) -> Option<(Built, Option<CardId>)> {
    let pl = g.player(p);
    if pl.hand.iter().any(|&h| matches!(&*g.db.get(h).name, "Demonic Consultation" | "Tainted Pact")) {
        return None; // Line 1 is cheaper
    }
    // the guide: never cast it under Opposition Agent (its controller would choose the pile), and one spell per turn
    // (Archon of Emeria, Deafening Silence, Rule of Law, Ethersworn Canonist) kills a pile that needs several spells
    let under_lock = g.opps(p).any(|q| {
        g.player(q).perms.iter().any(|&m| {
            !g.perm(m).phased
                && g.perm(m).cd.is_some_and(|x| {
                    matches!(
                        &*g.db.get(x).name,
                        "Opposition Agent"
                            | "Archon of Emeria"
                            | "Deafening Silence"
                            | "Rule of Law"
                            | "Ethersworn Canonist"
                    )
                })
        })
    });
    if under_lock {
        return None;
    }
    if pl.life < LIFE_NEEDED || !castable(g, p, c, "hand") {
        return None;
    }
    let flash = grants_flash(g, p, c);
    let en = if scn == Scn::EndStep && !flash { Some(enabler(g, p)?) } else { None };
    let b = doomsday_cost(g, p, c, en)?;
    let built = build(g, p, scn, b, en.as_slice())?;
    if g.opps(p).any(|q| g.player(q).alive && opp_power(g, q) >= built.life_after - 2) {
        return None; // a board that can deal what is left
    }
    let n = stoppers(g, p);
    if !built.lock && n >= 2 && !pl.hand.iter().any(|&h| g.db.get(h).tag(crate::tag::Tag::Ctr)) {
        return None; // into several opponents who can stop a spell, with no lock and no counter of our own
    }
    // the plan's go-now gate for the cast (a rule in `data/plans/thrasios.toml` naming "Doomsday cast"); the table's own
    // "can it be stopped" check is not asked: a countered Doomsday costs the card and {B}{B}{B} only (library and graveyard
    // are untouched), and the dangerous spells come after it, behind the lock the pile carries
    if crate::impls::t6_plans::go_now(g, p, "Doomsday cast") == Some(false) {
        return None;
    }
    Some((built, en))
}

/// is an end-step line available (or being made available) for a Doomsday in hand: flash granted, Teferi on the
/// battlefield or in the grant, Valley Floodcaller out, or an enabler in hand
fn enabler_around(g: &Game, p: PlayerId, c: CardId) -> bool {
    grants_flash(g, p, c)
        || enabler(g, p).is_some()
        || g.player(p).perms.iter().any(|&m| {
            !g.perm(m).phased
                && g.perm(m)
                    .cd
                    .is_some_and(|x| matches!(&*g.db.get(x).name, "Teferi, Time Raveler" | "Valley Floodcaller"))
        })
}

pub fn doomsday_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if crate::human::is_human(g, p) || !g.player(p).alive || active(g, p) {
        return Ok(vec![]);
    }
    let scn = match post {
        None if g.active != Some(p) => Scn::EndStep,
        Some(_) if g.active == Some(p) && g.stack.is_empty() && matches!(g.step, TurnStep::Main1 | TurnStep::Main2) => {
            Scn::Main
        }
        _ => return Ok(vec![]),
    };
    if scn == Scn::Main && enabler_around(g, p, c) && plan_cast(g, p, c, Scn::EndStep).is_some() {
        return Ok(vec![]); // the end step of the turn before is the better time (the guide)
    }
    let Some((built, _)) = plan_cast(g, p, c, scn) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 16.0,
        label: format!(
            "Doomsday ({} line, {} spells)",
            if built.lock { "locked" } else { "unlocked" },
            built.plan.len()
        ),
        act: Some(Action::Plan { f: go_doomsday, arg: c.0 as i64 }),
    }])
}

/// cast the rituals that make {B}{B}{B}, then Doomsday itself
pub fn go_doomsday(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) {
        return Ok(false);
    }
    let scn = if g.active == Some(p) { Scn::Main } else { Scn::EndStep };
    let Some((_, en)) = plan_cast(g, p, c, scn) else { return Ok(false) };
    if let Some(e) = en {
        crate::glog!(g, "  {} casts {} so Doomsday has flash", g.player(p).name, g.db.get(e).name);
        if !cast_one(g, p, e)? || !grants_flash(g, p, c) {
            return Ok(true); // the enabler was spent (or answered)
        }
    }
    let (gn, pips) = cost_of(g, p, c);
    while !can_pay(g, p, gn, &pips, false) {
        let Some(&r) = rituals_in_hand(g, p, &[]).first() else { return Ok(true) };
        if !crate::ai::brain::do_cast(g, p, r, None)? {
            return Ok(true);
        }
    }
    crate::glog!(g, "  {} casts Doomsday", g.player(p).name);
    if !cast_one(g, p, c)? {
        return Ok(true);
    }
    if scn == Scn::Main {
        run_pile(g, p)?;
    }
    Ok(true)
}

// ------------------------------------------------------------------ the catalogue combo (the pile on a later turn)
fn pile_ready(g: &Game, p: PlayerId) -> Ready {
    let no = (false, vec![], vec![]);
    if !active(g, p)
        || g.active != Some(p)
        || !g.stack.is_empty()
        || !matches!(g.step, TurnStep::Main1 | TurnStep::Main2)
    {
        return no;
    }
    (plan_now(g, p).is_some(), vec![], vec![])
}

fn play_pile(g: &mut Game, p: PlayerId) -> Res<bool> {
    run_pile(g, p)
}

fn finish(_g: &mut Game, _p: PlayerId) -> Res {
    Ok(()) // the pile was played for real: Thassa's Oracle's own trigger won (or did not)
}

static STEPS: [Step; 1] = [Step {
    label: "play the pile to Thassa's Oracle",
    kind: StepKind::Engine,
    src: no_src,
    pay: (0, ""),
    run: play_pile,
    keys: no_keys,
}];

pub static COMBOS: [Combo; 1] = [Combo {
    name: NAME,
    groups: &[&[DOOMSDAY], &[ORACLE]],
    ready: pile_ready,
    mana: (0, ""),
    finish: Some(finish),
    xcost: &[],
    steps: &STEPS,
}];

pub fn register(r: &mut Registry, db: &crate::cards::CardDb) -> Result<(), String> {
    let c = r.card(db, DOOMSDAY)?;
    c.resolve = Some(resolve);
    c.prio = Some(zero_prio);
    c.hand_options = Some(doomsday_options);
    Ok(())
}
