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
//!   Tainted Pact in hand (Line 1 is cheaper), never at 1 life (it would lose the last point), and not into several
//!   opponents who can stop a spell with no lock in the pile and no counter held. Life is otherwise no bar (JD,
//!   2026-11-06: an all-or-nothing move): no life minimum and no check of the opponents' boards.
//! * **Pile extras** (doomsday-b). Beyond the cards above the search also plays: Thrasios's {4} dig (scry 1, then a land
//!   goes onto the battlefield and anything else is drawn: the library is one card shorter either way; `Act::dig`,
//!   played by `tymna_cards::pile_dig`, the scry decided by `pile_scry` with n = 1); a devotion permanent from the hand
//!   (`K::Dev`: Mockingbird, Mystic Remora, Rhystic Study, Valley Floodcaller: +1 devotion for their blue); Veil of Summer
//!   held in hand with {G} open as the lock (`K::Veil`, `Env::locked`: the engine's `rules::veil_respond` casts it on any
//!   counter aimed at the Oracle, so it is not cast by the line and it is not used when an opponent's deck can hold Mindbreak
//!   Trap, which it does not answer); and a Noxious Revival in hand when the Oracle is cast as a backup
//!   (`Want::backup`, `Built::backup`: a pile without the lock is accepted into two stoppers with it). A Revival in the
//!   library or graveyard takes a pile slot when the line draws it in time. A pile is checked against a forced draw (an
//!   Esper Sentinel of ours: an opponent's first noncreature spell draws us a card) and the draw step's extra cards
//!   (Howling Mine). Borne Upon a Wind as the end-step enabler draws before Doomsday resolves, so the pile is built from
//!   the library after that draw (`ai_pile` runs when Doomsday resolves).
//! * **The table's risk check.** `plan_cast` also asks `t6_table::go_risk_covered` against `go_threshold`, as every Tier 6
//!   line does: the cast waits when the table can stop it and there is no cover (Silence or Chant in hand, a free counter
//!   of ours, the pile's lock, Veil held with {G}).
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
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

pub const DOOMSDAY: &str = "Doomsday";
pub const NAME: &str = "Doomsday + Thassa's Oracle";
const ORACLE: &str = "Thassa's Oracle";
const REVIVAL: &str = "Noxious Revival";
/// the pile is not started (and a Probe is not paid in life) below this much life
const LIFE_FLOOR: i32 = 3;
/// Doomsday is not cast below this much life: at 1 it loses the last point (half, rounded up) and the player dies before
/// the pile is played. Otherwise life is no bar (JD, 2026-11-06: "It's an all or nothing move"): the pile wins or the
/// player loses either way, and the pile search already plays with the life that is left.
const LIFE_NEEDED: i32 = 2;

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
    /// Veil of Summer: not cast by the line; held in hand with {G} open it answers any counter on the Oracle
    /// (`rules::veil_respond`: the Oracle is a combo piece), so it is a lock for the Oracle's cast (not against Mindbreak Trap)
    Veil,
    /// Noxious Revival: not cast by the line; in hand when the Oracle is countered it puts it back on top
    Revival,
    /// a permanent with blue in its cost and nothing else coloured (Mockingbird, Mystic Remora, Rhystic Study, Valley
    /// Floodcaller): cast from hand it adds 1 to the devotion the Oracle counts
    Dev,
    /// a land (Thrasios puts a revealed land onto the battlefield instead of drawing it)
    Land,
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
        "Veil of Summer" => K::Veil,
        REVIVAL => K::Revival,
        "Mockingbird" | "Mystic Remora" | "Rhystic Study" | "Valley Floodcaller" => K::Dev,
        _ => K::Other,
    }
}

fn kd(g: &Game, c: CardId) -> K {
    let d = g.db.get(c);
    match kind_of(&d.name) {
        K::Other if d.land => K::Land,
        k => k,
    }
}

/// a card that is only filler for the pile (never a line card)
fn plain(k: K) -> bool {
    matches!(k, K::Other | K::Land)
}

/// one move of a line: cast this card (a Probe for life when `life`), or Thrasios's {4} dig (`dig`, no card)
#[derive(Clone, Copy, Debug)]
pub struct Act {
    pub card: CardId,
    pub life: bool,
    pub dig: bool,
    /// Lion's Eye Diamond is cracked while this move is paid (the spell is on the stack): 0 not, 1 {U}{U}{U}, 2 {W}{W}{W},
    /// 3 {G}{G}{G}. The rest of the hand is discarded
    pub led: u8,
}

/// the position the search plays from: the hand, the library (top first), the mana the line has spent so far (as
/// colour counts the engine can be asked about), the life left, whether Silence is up and the devotion the line added
#[derive(Clone)]
struct S {
    hand: Vec<CardId>,
    lib: Vec<CardId>,
    u: u8,
    w: u8,
    gn: u8,
    life: i32,
    silence: bool,
    /// devotion to blue added by permanents the line cast (Mockingbird ...)
    dev: u8,
    /// Lion's Eye Diamond is on the battlefield and still uncracked (the line may crack it for a spell: `Act::led`)
    led: bool,
    /// the colour of the three mana a crack made (1 U, 2 W, 3 G; 0 none): it adds to what the lands pay
    bonus: u8,
}

impl S {
    fn new(hand: Vec<CardId>, lib: Vec<CardId>, life: i32) -> S {
        S { hand, lib, u: 0, w: 0, gn: 0, life, silence: false, dev: 0, led: false, bonus: 0 }
    }
}

/// what a line must have when the Oracle is cast
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Want {
    /// Silence up, or Veil of Summer in hand with {G} left
    lock: bool,
    /// a Noxious Revival in hand with its cost left (the Oracle can be put back if it is countered)
    backup: bool,
}

struct Env<'a> {
    g: Cow<'a, Game>,
    p: PlayerId,
    /// {B}s the line still owes before anything else (Doomsday itself, when it is not cast yet)
    b: u8,
    /// devotion to blue without Thassa's Oracle
    dev: usize,
    kinds: HashMap<u16, K>,
    /// the generic mana of a devotion permanent
    dev_gn: HashMap<u16, u8>,
    /// Thrasios is out: the generic cost of his {4} dig
    dig: Option<u8>,
    /// Veil of Summer answers every stopper the table may hold (none can hold Mindbreak Trap or a red blast)
    veil_ok: bool,
    memo: RefCell<HashMap<u32, bool>>,
    /// positions the search has shown to have no line: the most spells left it failed with (a transposition table, so a
    /// position reached by several orders of the same spells, or again at a deeper limit, is searched once)
    fail: RefCell<HashMap<u64, u8>>,
    /// search nodes left (`NODE_BUDGET` at the start); at zero every search answers "no line"
    budget: Cell<u32>,
}

impl<'a> Env<'a> {
    fn new(g: Cow<'a, Game>, p: PlayerId, b: u8, cards: &[CardId]) -> Env<'a> {
        let dev = super::combos::oracle_devotion(&g, p);
        let kinds: HashMap<u16, K> = cards.iter().map(|&c| (c.0, kd(&g, c))).collect();
        let dev_gn = cards
            .iter()
            .filter(|&&c| kinds.get(&c.0) == Some(&K::Dev))
            .map(|&c| (c.0, g.db.get(c).generic.min(9) as u8))
            .collect();
        let dig = g
            .player(p)
            .perms
            .iter()
            .any(|&m| {
                let x = g.perm(m);
                x.on_bf && !x.phased && x.cd.is_some_and(|c| &*g.db.get(c).name == "Thrasios, Triton Hero")
            })
            .then(|| crate::impls::t6_green1::creature_ability_cost(&g, p, 4).min(9) as u8);
        let veil_ok = veil_answers(&g, p);
        Env {
            g,
            p,
            b,
            dev,
            kinds,
            dev_gn,
            dig,
            veil_ok,
            memo: RefCell::new(HashMap::new()),
            fail: RefCell::new(HashMap::new()),
            budget: Cell::new(NODE_BUDGET),
        }
    }
    fn kd(&self, c: CardId) -> K {
        self.kinds.get(&c.0).copied().unwrap_or(K::Other)
    }
    /// can the player pay {B}^b + {U}^u + {W}^w + {G}^gr + generic gn at once
    fn ok(&self, u: u8, w: u8, gr: u8, gn: u8) -> bool {
        self.okb(0, u, w, gr, gn)
    }
    /// `ok` in the position `s`: a crack of Lion's Eye Diamond adds its three mana to what the lands pay
    fn ok_in(&self, s: &S, u: u8, w: u8, gr: u8, gn: u8) -> bool {
        self.okb(s.bonus, u, w, gr, gn)
    }
    /// `ok` with three mana of one colour added (`bonus`: 1 U, 2 W, 3 G, 0 none). The three pay the pips of their colour
    /// first and then generic; what the line has spent is a running total, so a crack's mana counts once
    fn okb(&self, bonus: u8, u: u8, w: u8, gr: u8, gn: u8) -> bool {
        let key = ((((((self.b as u32) * 16 + u as u32) * 16 + w as u32) * 16 + gr as u32) * 32) + gn as u32) * 4
            + bonus as u32;
        if let Some(&r) = self.memo.borrow().get(&key) {
            return r;
        }
        let (mut u, mut w, mut gr, mut gn) = (u, w, gr, gn);
        if bonus != 0 {
            let pips = match bonus {
                1 => &mut u,
                2 => &mut w,
                _ => &mut gr,
            };
            let k = (*pips).min(3);
            *pips -= k;
            gn = gn.saturating_sub(3 - k);
        }
        let pips = format!(
            "{}{}{}{}",
            "B".repeat(self.b as usize),
            "U".repeat(u as usize),
            "W".repeat(w as usize),
            "G".repeat(gr as usize)
        );
        let r = can_pay(&self.g, self.p, gn as u32, &pips, false);
        self.memo.borrow_mut().insert(key, r);
        r
    }
    /// the lock at the Oracle's cast: Silence is up, or Veil of Summer is in hand with {G} left over the Oracle's {U}{U}
    fn locked(&self, s: &S) -> bool {
        s.silence
            || (self.veil_ok && s.hand.iter().any(|&c| self.kd(c) == K::Veil) && self.ok_in(s, s.u + 2, s.w, 1, s.gn))
    }
    /// Noxious Revival is in hand with its {G} (or 2 life) left over the Oracle's {U}{U}
    fn backed(&self, s: &S) -> bool {
        s.hand.iter().any(|&c| self.kd(c) == K::Revival)
            && (self.ok_in(s, s.u + 2, s.w, 1, s.gn) || s.life - 2 >= LIFE_FLOOR)
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
    let mut s = S::new(
        pl.hand.iter().copied().filter(|&c| Some(c) != exclude).collect(),
        pl.library.iter().rev().copied().collect(),
        pl.life,
    );
    s.silence = silence_up(g, p);
    s.led = super::mine::led_on_table(g, p);
    s
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

fn act(card: CardId) -> Act {
    Act { card, life: false, dig: false, led: 0 }
}

fn step_to(e: &Env, t: S, a: Act, left: u8, w: Want, acts: &mut Vec<Act>) -> bool {
    acts.push(a);
    if dfs(e, &t, left - 1, w, acts) {
        return true;
    }
    acts.pop();
    false
}

/// Thrasios's {4}: scry 1, then the top card is revealed; a land goes onto the battlefield (tapped: no mana this turn),
/// anything else is drawn. Either way the library is one card shorter. The ways it can go: the top card kept or put on
/// the bottom first
fn dig_outcomes(e: &Env, s: &S) -> Vec<S> {
    let mut out = vec![];
    for bottom in [false, true] {
        if s.lib.is_empty() || (bottom && s.lib.len() < 2) {
            continue;
        }
        let mut t = s.clone();
        if bottom {
            let top = t.lib.remove(0);
            t.lib.push(top);
        }
        if e.kd(t.lib[0]) == K::Land {
            t.lib.remove(0);
        } else if !draw_n(&mut t, 1) {
            continue;
        }
        out.push(t);
    }
    out
}

/// the search nodes one `Env` may spend (a position the failure table does not cut). A line needs a handful of spells;
/// the budget only stops a pathological position from running for minutes inside a look-ahead playout. At zero every
/// search answers "no line".
const NODE_BUDGET: u32 = 50_000;

/// a key for `s` and `w`: the hand as a set (the order the spells were cast in does not matter), the library in order,
/// what the line has spent
fn state_key(s: &S, w: Want) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    let mut hand: Vec<u16> = s.hand.iter().map(|c| c.0).collect();
    hand.sort_unstable();
    hand.hash(&mut h);
    for c in &s.lib {
        c.0.hash(&mut h);
    }
    (s.u, s.w, s.gn, s.life, s.silence, s.dev, w.lock, w.backup, s.lib.len(), s.led, s.bonus).hash(&mut h);
    h.finish()
}

/// is there a line of at most `left` more spells that ends with Thassa's Oracle on the battlefield and a devotion at
/// least the library (`w.lock`: with Silence up or Veil of Summer held before the Oracle; `w.backup`: with Noxious
/// Revival held)? On success `acts` holds it. A position that failed with at least `left` spells is not searched again
/// (a failure is remembered only while the node budget lasts: after that it may only mean the search was cut short)
fn dfs(e: &Env, s: &S, left: u8, w: Want, acts: &mut Vec<Act>) -> bool {
    if left == 0 {
        return false;
    }
    let budget = e.budget.get();
    if budget == 0 {
        return false;
    }
    e.budget.set(budget - 1);
    // the Oracle needs a devotion of at least the library, and no spell shortens the library by more than one card: with
    // all the devotion permanents of hand and library cast, is the library still too long when the last spell is reached
    let dev_cards = s.hand.iter().chain(&s.lib).filter(|&&c| e.kd(c) == K::Dev).count();
    if s.lib.len() > e.dev + s.dev as usize + dev_cards + 2 + (left as usize - 1) {
        return false;
    }
    let key = state_key(s, w);
    if e.fail.borrow().get(&key).is_some_and(|&l| l >= left) {
        return false;
    }
    if dfs_inner(e, s, left, w, acts) {
        return true;
    }
    if e.budget.get() > 0 {
        let mut f = e.fail.borrow_mut();
        let l = f.entry(key).or_insert(0);
        *l = (*l).max(left);
    }
    false
}

/// what trying to cast one card of the hand came to
#[derive(Clone, Copy, PartialEq, Eq)]
enum Try {
    /// a line was found (it is in `acts`)
    Won,
    /// the card can be cast (or is no move here) and no line follows
    Lost,
    /// the mana for the card is not there: only a crack of Lion's Eye Diamond could pay for it
    Unpayable,
}

/// `s` after Lion's Eye Diamond is cracked for the cast of `keep` (the spell is on the stack; `None`: for an ability): the
/// rest of the hand is discarded (the graveyard is out of the pile's reach), and three mana of colour `col` are added
fn cracked(s: &S, keep: Option<CardId>, col: u8) -> S {
    let mut t = s.clone();
    t.hand = keep.into_iter().collect();
    t.led = false;
    t.bonus = col;
    t
}

fn dfs_inner(e: &Env, s: &S, left: u8, w: Want, acts: &mut Vec<Act>) -> bool {
    for i in 0..s.hand.len() {
        let c = s.hand[i];
        let k = e.kd(c);
        if plain(k) || matches!(k, K::Veil | K::Revival) || s.hand[..i].iter().any(|&x| x == c) {
            continue;
        }
        match try_card(e, s, i, 0, left, w, acts) {
            Try::Won => return true,
            Try::Lost => {}
            // the latest the diamond can be cracked is the first spell the lands cannot pay: cracking earlier only throws
            // the hand away sooner (the mana is a running total). Silence's {W}, the cantrips' {U} or Veil's {G}
            Try::Unpayable => {
                if s.led && s.bonus == 0 {
                    for col in 1..=3 {
                        if try_card(e, &cracked(s, Some(c), col), 0, col, left, w, acts) == Try::Won {
                            return true;
                        }
                    }
                }
            }
        }
    }
    // Thrasios's dig is a draw source of its own
    if let Some(cost) = e.dig {
        if e.ok_in(s, s.u, s.w, 0, s.gn + cost) {
            for mut t in dig_outcomes(e, s) {
                t.gn += cost;
                if step_to(e, t, Act { card: CardId(0), life: false, dig: true, led: 0 }, left, w, acts) {
                    return true;
                }
            }
        } else if s.led && s.bonus == 0 {
            // the diamond pays for the dig when the lands cannot (its cost is an ability's: the whole hand goes)
            for led in 1..=3 {
                let sc = cracked(s, None, led);
                if !e.ok_in(&sc, sc.u, sc.w, 0, sc.gn + cost) {
                    continue;
                }
                for mut t in dig_outcomes(e, &sc) {
                    t.gn += cost;
                    if step_to(e, t, Act { card: CardId(0), life: false, dig: true, led }, left, w, acts) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// cast the card at `s.hand[i]` (with the diamond cracked for it when `led` is not 0: `s` is then the cracked position)
fn try_card(e: &Env, s: &S, i: usize, led: u8, left: u8, w: Want, acts: &mut Vec<Act>) -> Try {
    let c = s.hand[i];
    let k = e.kd(c);
    let mut base = s.clone();
    base.hand.remove(i);
    let a = Act { led, ..act(c) };
    match k {
        K::Silence => {
            if s.silence {
                return Try::Lost;
            }
            if !e.ok_in(s, s.u, s.w + 1, 0, s.gn) {
                return Try::Unpayable;
            }
            base.w += 1;
            base.silence = true;
            if step_to(e, base, a, left, w, acts) {
                return Try::Won;
            }
        }
        K::Oracle => {
            if !e.ok_in(s, s.u + 2, s.w, 0, s.gn) {
                return Try::Unpayable;
            }
            if e.dev + s.dev as usize + 2 >= s.lib.len() && (!w.lock || e.locked(s)) && (!w.backup || e.backed(s)) {
                acts.push(a);
                return Try::Won;
            }
        }
        K::Dev => {
            let gnc = e.dev_gn.get(&c.0).copied().unwrap_or(0);
            if !e.ok_in(s, s.u + 1, s.w, 0, s.gn + gnc) {
                return Try::Unpayable;
            }
            base.u += 1;
            base.gn += gnc;
            base.dev += 1;
            if step_to(e, base, a, left, w, acts) {
                return Try::Won;
            }
        }
        K::Probe => {
            let mut payable = false;
            if e.ok_in(s, s.u + 1, s.w, 0, s.gn) {
                payable = true;
                let mut t = base.clone();
                t.u += 1;
                if draw_n(&mut t, 1) && step_to(e, t, a, left, w, acts) {
                    return Try::Won;
                }
            }
            if s.life - 2 >= LIFE_FLOOR {
                payable = true;
                let mut t = base.clone();
                t.life -= 2;
                if draw_n(&mut t, 1) && step_to(e, t, Act { life: true, ..a }, left, w, acts) {
                    return Try::Won;
                }
            }
            if !payable {
                return Try::Unpayable;
            }
        }
        K::Borne => {
            if !e.ok_in(s, s.u + 1, s.w, 0, s.gn + 1) {
                return Try::Unpayable;
            }
            let mut t = base.clone();
            t.u += 1;
            t.gn += 1;
            if draw_n(&mut t, 1) && step_to(e, t, a, left, w, acts) {
                return Try::Won;
            }
        }
        K::Brainstorm => {
            if !e.ok_in(s, s.u + 1, s.w, 0, s.gn) {
                return Try::Unpayable;
            }
            let mut t = base.clone();
            t.u += 1;
            if !draw_n(&mut t, 3) {
                return Try::Lost;
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
                    if step_to(e, t2, a, left, w, acts) {
                        return Try::Won;
                    }
                }
            }
        }
        K::Preordain => {
            if !e.ok_in(s, s.u + 1, s.w, 0, s.gn) {
                return Try::Unpayable;
            }
            let n = s.lib.len().min(2);
            for (keep, bottom) in scry_variants(&s.lib[..n]) {
                let mut t = base.clone();
                t.u += 1;
                t.lib = keep.into_iter().chain(s.lib[n..].iter().copied()).chain(bottom).collect();
                if draw_n(&mut t, 1) && step_to(e, t, a, left, w, acts) {
                    return Try::Won;
                }
            }
        }
        K::Ponder => {
            if !e.ok_in(s, s.u + 1, s.w, 0, s.gn) {
                return Try::Unpayable;
            }
            let n = s.lib.len().min(3);
            for ord in orders(&s.lib[..n]) {
                let mut t = base.clone();
                t.u += 1;
                t.lib = ord.into_iter().chain(s.lib[n..].iter().copied()).collect();
                if draw_n(&mut t, 1) && step_to(e, t, a, left, w, acts) {
                    return Try::Won;
                }
            }
        }
        K::Veil | K::Revival | K::Land | K::Other => {}
    }
    Try::Lost
}

/// the shortest line from `s` that meets `w`
fn solve(e: &Env, s: &S, w: Want) -> Option<Vec<Act>> {
    solve_to(e, s, w, MAX_SPELLS)
}

/// the most spells a line may have
const MAX_SPELLS: u8 = 8;

/// the shortest line from `s` that meets `w`, of at most `max` spells
fn solve_to(e: &Env, s: &S, w: Want, max: u8) -> Option<Vec<Act>> {
    let has_oracle = s.hand.iter().chain(&s.lib).any(|&c| e.kd(c) == K::Oracle);
    if !has_oracle {
        return None;
    }
    // a lock is Silence cast from the hand or Veil of Summer held: with neither in hand or library there is none
    if w.lock && !s.silence && !s.hand.iter().chain(&s.lib).any(|&c| matches!(e.kd(c), K::Silence | K::Veil)) {
        return None;
    }
    for d in 1..=max {
        let mut acts = vec![];
        if dfs(e, s, d, w, &mut acts) {
            return Some(acts);
        }
    }
    None
}

/// the best line from `s` and what it has: with `lock_first` the lock (and then the backup) are asked for before
/// plain speed; a Noxious Revival is only asked about when one is in hand or in the library
fn ladder(e: &Env, s: &S, lock_first: bool) -> Option<(Want, Vec<Act>)> {
    ladder_over(e, s, lock_first, None)
}

/// `ladder`, asked only for a line that scores above `floor` (the builder's best pile so far, scored as (lock, backup,
/// minus the spells)): a want below the floor's is skipped, and one equal to it must be shorter. The line found is the one
/// `ladder` finds when that scores above the floor, and none otherwise
fn ladder_over(e: &Env, s: &S, lock_first: bool, floor: Option<(i32, i32, i32)>) -> Option<(Want, Vec<Act>)> {
    let revival = s.hand.iter().chain(&s.lib).any(|&c| e.kd(c) == K::Revival);
    let mut tries = vec![];
    if lock_first {
        if revival {
            tries.push(Want { lock: true, backup: true });
        }
        tries.push(Want { lock: true, backup: false });
        if revival {
            tries.push(Want { lock: false, backup: true });
        }
    }
    tries.push(Want { lock: false, backup: false });
    tries.into_iter().find_map(|w| {
        let mut max = MAX_SPELLS;
        if let Some((lock, backup, neg_len)) = floor {
            let rank = (w.lock as i32, w.backup as i32);
            if rank < (lock, backup) {
                return None;
            }
            if rank == (lock, backup) {
                max = max.min((-neg_len - 1).clamp(0, MAX_SPELLS as i32) as u8); // shorter than the floor's line
            }
        }
        solve_to(e, s, w, max).map(|a| (w, a))
    })
}

/// how good a position is: (the line has the lock, minus its length); None when there is no line
fn rank(e: &Env, s: &S) -> Option<(i32, i32)> {
    if let Some(a) = solve(e, s, Want { lock: true, backup: false }) {
        return Some((1, -(a.len() as i32)));
    }
    solve(e, s, Want { lock: false, backup: false }).map(|a| (0, -(a.len() as i32)))
}

/// the line from `s`: with the lock when `want_lock` and one exists
fn line(e: &Env, s: &S, want_lock: bool) -> Option<Vec<Act>> {
    ladder(e, s, want_lock).map(|x| x.1)
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
    /// the line has the lock (Silence up, or Veil of Summer held) when the Oracle is cast
    pub lock: bool,
    /// the line holds a Noxious Revival when the Oracle is cast (a countered Oracle goes back on top)
    pub backup: bool,
    pub life_after: i32,
    /// Doomsday is cast with Lion's Eye Diamond's {B}{B}{B}: the hand is discarded as it goes on the stack (the pile may take
    /// those cards from the graveyard) and the lands are free for the pile
    pub led: bool,
}

thread_local! {
    static CACHE: RefCell<HashMap<u64, Option<Built>>> = RefCell::new(HashMap::new());
}

fn cache_key(g: &Game, p: PlayerId, scn: Scn, b: u8, skip: &[CardId], led_pays: bool) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    let pl = g.player(p);
    (p.0, scn as u8, b, pl.life, g.round, g.step as u8, led_pays).hash(&mut h);
    for c in pl.hand.iter().chain(&pl.library).chain(&pl.gy).chain(skip) {
        c.0.hash(&mut h);
    }
    (pl.hand.len(), pl.library.len(), pl.gy.len(), crate::engine::mana::total_mana(g, p, false)).hash(&mut h);
    let board: Vec<u32> = pl.perms.iter().map(|&m| m.0 as u32).collect();
    board.hash(&mut h);
    silence_up(g, p).hash(&mut h);
    veil_answers(g, p).hash(&mut h);
    forced_draws(g, p).hash(&mut h);
    draw_step_extra(g).hash(&mut h);
    h.finish()
}

/// The best pile for the position, or None when no pile wins. `b` is the {B} Doomsday still has to be paid with (3 when
/// it is not cast yet), `skip` hand cards that will be spent before it (the enabler). The pile is chosen from the
/// cards of the library and graveyard; the hand is what the line starts with.
pub fn build(g: &Game, p: PlayerId, scn: Scn, b: u8, skip: &[CardId]) -> Option<Built> {
    build_with(g, p, scn, b, skip, false)
}

/// `build` for a Doomsday cast with Lion's Eye Diamond's {B}{B}{B} (the diamond is on the battlefield): no {B} is owed, the
/// hand (but the cards in `skip`) is discarded as the spell goes on the stack, and what it held is in the graveyard
/// for the pile to take (unless Necropotence exiles the discards). The line starts with an empty hand
pub fn build_led(g: &Game, p: PlayerId, scn: Scn, skip: &[CardId]) -> Option<Built> {
    build_with(g, p, scn, 0, skip, true)
}

fn build_with(g: &Game, p: PlayerId, scn: Scn, b: u8, skip: &[CardId], led_pays: bool) -> Option<Built> {
    let key = cache_key(g, p, scn, b, skip, led_pays);
    if let Some(r) = CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return r;
    }
    let r = build_uncached(g, p, scn, b, skip, led_pays);
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 256 {
            c.clear();
        }
        c.insert(key, r.clone());
    });
    r
}

fn build_uncached(g: &Game, p: PlayerId, scn: Scn, b: u8, skip: &[CardId], led_pays: bool) -> Option<Built> {
    let pl = g.player(p);
    let kept: Vec<CardId> =
        pl.hand.iter().copied().filter(|&c| &*g.db.get(c).name != DOOMSDAY && !skip.contains(&c)).collect();
    // cracked for Doomsday: the hand is discarded to the graveyard (exiled instead under Necropotence), where the pile can
    // take it; the diamond is sacrificed into the graveyard too
    let exiles_discards = crate::engine::values::has(g, p, crate::tag::Tag::Necro);
    let (hand, discards): (Vec<CardId>, Vec<CardId>) = if led_pays { (vec![], kept) } else { (kept, vec![]) };
    let mut avail: Vec<CardId> = pl.library.iter().chain(&pl.gy).copied().collect();
    if !exiles_discards {
        avail.extend(&discards);
    }
    if avail.is_empty() {
        return None;
    }
    let n_pick = avail.len().min(5);
    let life_after = pl.life - half_up(pl.life);
    let in_hand_oracle = hand.iter().any(|&c| kd(g, c) == K::Oracle);
    if !in_hand_oracle && !avail.iter().any(|&c| kd(g, c) == K::Oracle) {
        return None;
    }
    let necro = pl
        .perms
        .iter()
        .any(|&m| g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == "Necropotence") && !g.perm(m).phased);
    // the draw step takes the first card (and what Howling Mine adds); skipped under Necropotence
    let first_draws = if scn == Scn::EndStep && !necro { 1 + draw_step_extra(g) } else { 0 };
    let forced = forced_draws(g, p);
    // the diamond may still be cracked inside the pile unless it paid for Doomsday
    let led_open = !led_pays && super::mine::led_on_table(g, p);
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
    // two things any winning line needs, checked once for all the piles: the Oracle's {U}{U} is payable after what is owed,
    // and (without Thrasios's dig) there are enough cards that each shorten the library by one, or add a devotion, to leave
    // it no longer than the devotion plus 2 (every spell does one of these, at most)
    if !env.ok(2, 0, 0, 0) {
        return None;
    }
    if env.dig.is_none() {
        let gap = (n_pick - first_draws.min(n_pick)) as i64 - (env.dev as i64 + 2);
        let reducers = all
            .iter()
            .filter(|&&c| matches!(env.kd(c), K::Dev | K::Probe | K::Borne | K::Brainstorm | K::Preordain | K::Ponder))
            .count() as i64;
        if gap > reducers {
            return None;
        }
    }
    // Veil of Summer takes a pile slot only when Silence cannot be the lock (not held or found, or no {W})
    let silence_around = all.iter().any(|&c| env.kd(c) == K::Silence) && env.ok(0, 1, 0, 0);
    let use_veil = env.veil_ok && !silence_around;
    let is_useful = |k: K| match k {
        K::Oracle | K::Silence | K::Probe | K::Brainstorm | K::Preordain | K::Ponder | K::Borne => true,
        K::Veil => use_veil,
        _ => false,
    };
    let useful: Vec<CardId> = avail.iter().copied().filter(|&c| is_useful(env.kd(c))).collect();
    let mut fillers: Vec<CardId> = avail.iter().copied().filter(|&c| !is_useful(env.kd(c))).collect();
    fillers.sort_by(|&x, &y| {
        card_worth(g, p, y, false).partial_cmp(&card_worth(g, p, x, false)).unwrap_or(std::cmp::Ordering::Equal)
    });
    // a pile is good when it wins with the draw step (and a forced draw: an Esper Sentinel of ours) behind it; the
    // score is (the lock, a Noxious Revival to answer a countered Oracle, fewer spells)
    let try_pile = |pile: &[CardId], floor: Option<(i32, i32, i32)>| -> Option<((i32, i32, i32), Want, Vec<Act>)> {
        let mut s = S::new(hand.clone(), pile.to_vec(), life_after);
        s.led = led_open;
        if first_draws > 0 && !draw_n(&mut s, first_draws) {
            return None;
        }
        let (w, plan) = ladder_over(&env, &s, true, floor)?;
        if forced > 0 {
            // a draw we did not plan (our Esper Sentinel, when an opponent casts a spell) must not break the line
            let mut s2 = s.clone();
            if !draw_n(&mut s2, forced) || ladder(&env, &s2, false).is_none() {
                return None;
            }
        }
        Some(((w.lock as i32, w.backup as i32, -(plan.len() as i32)), w, plan))
    };
    let revival_any = all.iter().any(|&c| env.kd(c) == K::Revival);
    // the best score any pile can have: the lock if Silence or Veil of Summer is around, the backup if a Noxious Revival is,
    // and the fewest spells the library allows (one more than the cards that must shorten it)
    let lock_around = (env.veil_ok && all.iter().any(|&c| env.kd(c) == K::Veil))
        || (all.iter().any(|&c| env.kd(c) == K::Silence) && env.ok(0, 1, 0, 0));
    let shortest = (n_pick as i64 - first_draws.min(n_pick) as i64 - (env.dev as i64 + 2) + 1).clamp(1, 8) as i32;
    let ceiling = (lock_around as i32, revival_any as i32, -shortest);
    let mut best: Option<((i32, i32, i32), Built)> = None;
    'piles: for m in 0..=useful.len().min(n_pick) {
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
            let Some((score, w, plan)) = try_pile(&pile, best.as_ref().map(|(sc, _)| *sc)) else { continue };
            if best.as_ref().is_none_or(|(sc, _)| score > *sc) {
                best = Some((score, Built { pile, plan, lock: w.lock, backup: w.backup, life_after, led: led_pays }));
                if score >= ceiling {
                    break 'piles; // nothing can score higher
                }
            }
        }
        if best.as_ref().is_some_and(|(sc, _)| sc.0 == 1 && (sc.1 == 1 || !revival_any)) {
            break; // a pile with the lock (and the backup): more useful cards in it would not make it better
        }
    }
    // a Noxious Revival in the library or graveyard can take a slot of the best pile when the line draws it
    if let Some((sc, bt)) = best.clone()
        && !bt.backup
        && let Some(&rev) = avail.iter().find(|&&c| env.kd(c) == K::Revival)
    {
        for pos in 0..n_pick {
            let mut pile: Vec<CardId> = bt.pile.iter().copied().filter(|&c| c != rev).collect();
            pile.insert(pos.min(pile.len()), rev);
            pile.truncate(n_pick);
            if !pile.contains(&rev) {
                continue;
            }
            let Some((score, w, plan)) = try_pile(&pile, best.as_ref().map(|(sc, _)| *sc)) else { continue };
            if w.backup && score.0 >= sc.0 && best.as_ref().is_some_and(|(b, _)| score > *b) {
                best = Some((score, Built { pile, plan, lock: w.lock, backup: true, life_after, led: led_pays }));
            }
        }
    }
    best.map(|x| x.1)
}

/// extra cards the draw step brings: an untapped Howling Mine (whoever controls it) draws each player one
fn draw_step_extra(g: &Game) -> usize {
    g.players
        .iter()
        .flat_map(|q| q.perms.iter())
        .filter(|&&m| !g.perm(m).phased && g.perm(m).on_bf)
        .map(|&m| match g.perm(m).cd.map(|c| &*g.db.get(c).name) {
            Some("Howling Mine") if !g.perm(m).tapped => 1,
            _ => 0,
        })
        .sum()
}

/// draws p's own cards force on it while the pile is played: each Esper Sentinel of p's draws when an opponent casts its
/// first noncreature spell of the turn (unless it pays {X}); only an opponent with a card in hand can
fn forced_draws(g: &Game, p: PlayerId) -> usize {
    let sentinels = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| {
            !g.perm(m).phased && g.perm(m).on_bf && g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == "Esper Sentinel")
        })
        .count();
    let casters = g.opps(p).filter(|&q| g.player(q).alive && !g.player(q).hand.is_empty()).count();
    (sentinels * casters).min(2)
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
        if e.ok(1, 0, 0, 0) && draw_n(&mut u, 1) { rank(&e, &u) } else { None },
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

/// the scry of a cantrip: Preordain's scry 2 (then a draw), or Thrasios's scry 1 (then the top card is revealed: a land
/// goes onto the battlefield, anything else is drawn). (kept on top in order, put on the bottom)
pub fn pile_scry(g: &Game, p: PlayerId, n: usize) -> Option<(Vec<CardId>, Vec<CardId>)> {
    if !active(g, p) || !(n == 1 || n == 2) {
        return None;
    }
    let s = state_now(g, p, None);
    let e = env_now(g, p, &s);
    let k = s.lib.len().min(n);
    let mut best: Option<((i32, i32), (Vec<CardId>, Vec<CardId>))> = None;
    for (keep, bottom) in scry_variants(&s.lib[..k]) {
        let mut t = s.clone();
        t.lib = keep.iter().copied().chain(s.lib[k..].iter().copied()).chain(bottom.iter().copied()).collect();
        let ok = if n == 1 && t.lib.first().is_some_and(|&c| e.kd(c) == K::Land) {
            t.lib.remove(0);
            true
        } else {
            draw_n(&mut t, 1)
        };
        if !ok {
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

/// the mana the line from `acts` on needs, as (generic, pips): what `pile_dig` leaves untapped
fn rest_cost(g: &Game, p: PlayerId, acts: &[Act]) -> (u32, String) {
    let (mut gn, mut pips) = (0, String::new());
    for a in acts {
        if a.dig {
            gn += crate::impls::t6_green1::creature_ability_cost(g, p, 4);
        } else {
            let (cg, cp) = cost_of(g, p, a.card);
            gn += cg;
            pips.push_str(&cp);
        }
    }
    (gn, pips)
}

/// the mana a crack of the diamond makes for `Act::led`: the pool kind and its name
fn led_kind(led: u8) -> (crate::engine::pool::Kind, &'static str) {
    use crate::engine::pool::Kind;
    match led {
        1 => (Kind::U, "UUU"),
        2 => (Kind::W, "WWW"),
        _ => (Kind::G, "GGG"),
    }
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
        if a.led != 0 {
            // the lands cannot pay for this move: crack Lion's Eye Diamond with it on the stack (the rest of the hand goes)
            let keep: Vec<CardId> = if a.dig { vec![] } else { vec![a.card] };
            let (kind, name) = led_kind(a.led);
            if !crate::impls::mine::crack_led(g, p, &keep, kind)? {
                break;
            }
            crate::glog!(
                g,
                "    {} cracks Lion's Eye Diamond for {} to pay for {}",
                g.player(p).name,
                name,
                if a.dig { "Thrasios's dig".to_string() } else { g.db.get(a.card).name.to_string() }
            );
            g.player_mut(p).stat("led_pile", 1);
        }
        if a.dig {
            crate::glog!(g, "    {} plays the pile: Thrasios's dig", g.player(p).name);
            if !super::tymna_cards::pile_dig(g, p, rest_cost(g, p, &plan[1..]))? {
                failed += 1;
                if failed >= 2 {
                    break;
                }
            }
            continue;
        }
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
/// paid or cast, the life is too low, a board can deal the life that is left, or no pile wins. With Lion's Eye Diamond on
/// the battlefield the cast is also planned with the diamond's {B}{B}{B} (`Built::led`, see `plan_mode`); that plan is
/// taken when the plain cast has no winning pile, or only a worse one (no lock, a longer line)
pub fn plan_cast(g: &Game, p: PlayerId, c: CardId, scn: Scn) -> Option<(Built, Option<CardId>)> {
    plan_cast_why(g, p, c, scn).ok()
}

/// `plan_cast`, with the first check that declined the cast when there is none (--- doomsday-why: the trace-only
/// `dd|` statistics name it; the checks and their order are `plan_cast`'s own). With Lion's Eye Diamond out and no plan
/// either way, the plain cast's reason is given
pub fn plan_cast_why(g: &Game, p: PlayerId, c: CardId, scn: Scn) -> Result<(Built, Option<CardId>), &'static str> {
    let en = pre_checks(g, p, c, scn)?;
    let plain = plan_mode(g, p, c, scn, en, false);
    if !super::mine::led_on_table(g, p) {
        return plain;
    }
    // (the plain plan is the keeper on a tie: the diamond and the hand stay)
    match (plain, plan_mode(g, p, c, scn, en, true)) {
        (Ok(a), Ok(b)) => Ok(if score_of(&b.0) > score_of(&a.0) { b } else { a }),
        (Ok(a), Err(_)) => Ok(a),
        (Err(_), Ok(b)) => Ok(b),
        (Err(e), Err(_)) => Err(e),
    }
}

/// how good a built pile is: (the lock, a Noxious Revival to answer a countered Oracle, minus the spells)
fn score_of(b: &Built) -> (i32, i32, i32) {
    (b.lock as i32, b.backup as i32, -(b.plan.len() as i32))
}

/// is there a Doomsday in hand that Lion's Eye Diamond's mana wins with (as if the diamond were on the battlefield: it is
/// cast first, for nothing): the held-mana statistic and `mine::led_uses` ask
pub fn led_use(g: &Game, p: PlayerId) -> bool {
    if crate::human::is_human(g, p) || active(g, p) {
        return false;
    }
    // (in the main phase the discarded hand can not start a pile, there being no draw: the end step is where the diamond pays)
    g.player(p)
        .hand
        .iter()
        .copied()
        .filter(|&c| &*g.db.get(c).name == DOOMSDAY)
        .any(|c| [Scn::Main, Scn::EndStep].into_iter().any(|scn| led_use_in(g, p, c, scn)))
}

fn led_use_in(g: &Game, p: PlayerId, c: CardId, scn: Scn) -> bool {
    pre_checks(g, p, c, scn).is_ok_and(|en| {
        plan_mode(g, p, c, scn, en, true).is_ok_and(|led| {
            // it is a use when the plain cast has no pile, or the diamond's pile is better
            plan_mode(g, p, c, scn, en, false).ok().is_none_or(|plain| score_of(&led.0) > score_of(&plain.0))
        })
    })
}

/// the checks of a Doomsday cast that do not depend on how it is paid; Ok(the enabler to cast first) when they pass, else
/// the check that declined (`plan_cast_why`)
fn pre_checks(g: &Game, p: PlayerId, c: CardId, scn: Scn) -> Result<Option<CardId>, &'static str> {
    let pl = g.player(p);
    if pl.hand.iter().any(|&h| matches!(&*g.db.get(h).name, "Demonic Consultation" | "Tainted Pact")) {
        return Err("Demonic Consultation or Tainted Pact in hand"); // Line 1 is cheaper
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
        return Err("an opponent's lock (Opposition Agent or a one-spell rule)");
    }
    if pl.life < LIFE_NEEDED {
        return Err("at 1 life");
    }
    if !castable(g, p, c, "hand") {
        return Err("not castable now");
    }
    let flash = grants_flash(g, p, c);
    let en = if scn == Scn::EndStep && !flash {
        Some(enabler(g, p).ok_or("end step: no flash enabler (Teferi, Valley Floodcaller, Borne Upon a Wind)")?)
    } else {
        None
    };
    // the checks that do not need the pile come first: the builder is the expensive part, and a look-ahead asks about a
    // Doomsday in hand at every decision of every playout. No board check (JD, 2026-11-06): a main-phase pile wins this
    // turn, and an end-step cast is in the turn just before the player's own, so no opponent gets a combat first
    // the plan's go-now gate for the cast (a rule in `data/plans/thrasios.toml` naming "Doomsday cast")
    if crate::impls::t6_plans::go_now(g, p, "Doomsday cast") == Some(false) {
        return Err("the plan's go-now gate");
    }
    Ok(en)
}

/// One way to pay for the cast. Plain: the lands (and a Dark Ritual) make {B}{B}{B}, the hand is kept for the line.
/// With `led`: Lion's Eye Diamond makes them (a mana ability, cracked while Doomsday is on the stack: the spell is not
/// discarded), the rest of the hand goes to the graveyard, where Doomsday can take it into the pile, and the lands stay
/// free for the pile. The hand's own cover (Silence, Orim's Chant, a counter of ours, Veil of Summer held) is gone with the
/// hand, so the table's check is made with the hand empty
fn plan_mode(
    g: &Game,
    p: PlayerId,
    c: CardId,
    scn: Scn,
    en: Option<CardId>,
    led: bool,
) -> Result<(Built, Option<CardId>), &'static str> {
    let pl = g.player(p);
    let b = if led {
        // the diamond pays the {B}{B}{B}; a tax on the spell, and the enabler, are paid by the lands
        let (gn, pips) = cost_of(g, p, c);
        if gn > 0 || pips != "BBB" {
            return Err("Lion's Eye Diamond: Doomsday is taxed");
        }
        if let Some(e) = en {
            let (eg, ep) = cost_of(g, p, e);
            if !can_pay(g, p, eg, &ep, false) {
                return Err("Lion's Eye Diamond: the enabler can't be paid");
            }
        }
        0
    } else {
        doomsday_cost(g, p, c, en).ok_or("no {B}{B}{B} (with the enabler) to pay for it")?
    };
    // the table's check below with the most cover a pile can have: if it fails with the lock, it fails without
    let bare;
    let gr: &Game = if led {
        bare = {
            let mut g2 = g.clone();
            g2.player_mut(p).hand.clear();
            g2
        };
        &bare
    } else {
        g
    };
    if !risk_ok(gr, p, true) {
        return Err("the table's risk, even with a lock");
    }
    let built = if led { build_led(g, p, scn, en.as_slice()) } else { build(g, p, scn, b, en.as_slice()) }
        .ok_or("no winning pile from the cards and mana there are")?;
    let n = stoppers(g, p);
    if !built.lock
        && !built.backup
        && n >= 2
        && (led || !pl.hand.iter().any(|&h| g.db.get(h).tag(crate::tag::Tag::Ctr)))
    {
        // into several opponents who can stop a spell, with no lock, no Noxious Revival to put a countered Oracle back and
        // no counter of our own
        return Err("two or more stoppers, and no lock, Noxious Revival or counter");
    }
    // the table's check, as every Tier 6 line makes it (`t6_table::go_ok`): the cast waits when the table can stop it and
    // there is no cover. Cover is Silence or Orim's Chant in hand (`go_risk` counts it), a free counter of our own
    // (counted too), the lock the pile carries, or Veil of Summer held with {G} open
    let veil = !led && pl.hand.iter().any(|&h| &*g.db.get(h).name == "Veil of Summer") && can_pay(g, p, 0, "G", false);
    if !risk_ok(gr, p, built.lock || veil) {
        return Err("the table's risk, with the pile's own cover");
    }
    Ok((built, en))
}

/// can the table stop the Doomsday cast (or the first spell of the pile) by more than the risk Tier 6 accepts,
/// `covered` being cover the risk check does not see
pub fn risk_ok(g: &Game, p: PlayerId, covered: bool) -> bool {
    crate::ai::t6_table::go_risk_covered(g, p, covered) <= crate::ai::t6_table::go_threshold(g, p)
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

/// --- doomsday-why: trace-only statistics (`dd|...` in the player's stats; no decision reads them, they draw no random
/// numbers): each time the AI asks whether to cast a Doomsday in hand, outside the look-ahead, the answer (offered, or
/// the first check that declined) by timing, and once per game that it was asked at all
fn why_stat(g: &mut Game, p: PlayerId, scn: Scn, why: Result<(), &'static str>) {
    if g.in_search {
        return;
    }
    let t = if scn == Scn::EndStep { "end step" } else { "main" };
    let what = match why {
        Ok(()) => "offered",
        Err(e) => e,
    };
    let pl = g.player_mut(p);
    pl.stat(crate::sym::intern(&format!("dd|asked|{t}|{what}")), 1);
    if !pl.stats.contains_key("dd|games asked") {
        pl.stat("dd|games asked", 1);
    }
}

/// --- doomsday-why: at the start of p's own turn (outside the look-ahead), with Doomsday in the library and a tutor in
/// hand: what a tutor would fetch now, and whether Doomsday is on the plan's wish list at all (trace-only, as `why_stat`)
pub fn tutor_stat(g: &mut Game, p: PlayerId) {
    if g.in_search || crate::human::is_human(g, p) || g.player(p).key != "thrasios" {
        return;
    }
    let pl = g.player(p);
    if !pl.library.iter().any(|&c| &*g.db.get(c).name == DOOMSDAY) {
        return;
    }
    let tutor = pl.hand.iter().any(|&c| {
        matches!(
            &*g.db.get(c).name,
            "Demonic Tutor" | "Vampiric Tutor" | "Imperial Seal" | "Grim Tutor" | "Diabolic Intent" | "Mystical Tutor"
        )
    });
    if !tutor {
        return;
    }
    let pick = crate::ai::tutor_pick(g, p, "any").map(|c| g.db.get(c).name.to_string()).unwrap_or("nothing".into());
    let wished = crate::ai::plan_data::derived_wish(g, p).iter().any(|&c| &*g.db.get(c).name == DOOMSDAY);
    let pl = g.player_mut(p);
    pl.stat("dd|tutor turns", 1);
    pl.stat(crate::sym::intern(&format!("dd|tutor would fetch|{pick}")), 1);
    pl.stat(if wished { "dd|tutor: Doomsday on the wish list" } else { "dd|tutor: Doomsday not wished" }, 1);
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
        why_stat(g, p, scn, Err("kept for the end step (a pile wins there)"));
        return Ok(vec![]); // the end step of the turn before is the better time (the guide)
    }
    let why = plan_cast_why(g, p, c, scn);
    why_stat(g, p, scn, why.as_ref().map(|_| ()).map_err(|e| *e));
    let Ok((built, _)) = why else { return Ok(vec![]) };
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

/// cast the rituals that make {B}{B}{B} (or crack Lion's Eye Diamond for them, when the plan is the diamond's), then Doomsday
/// itself
pub fn go_doomsday(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) {
        return Ok(false);
    }
    if !g.in_search {
        g.player_mut(p).stat("dd|chosen (cast begun)", 1); // --- doomsday-why (trace-only)
    }
    let scn = if g.active == Some(p) { Scn::Main } else { Scn::EndStep };
    let Some((built, en)) = plan_cast(g, p, c, scn) else { return Ok(false) };
    if let Some(e) = en {
        crate::glog!(g, "  {} casts {} so Doomsday has flash", g.player(p).name, g.db.get(e).name);
        if !cast_one(g, p, e)? || !grants_flash(g, p, c) {
            return Ok(true); // the enabler was spent (or answered)
        }
    }
    let (gn, pips) = cost_of(g, p, c);
    if built.led {
        // Doomsday goes on the stack, the diamond is cracked for its {B}{B}{B} (the rest of the hand is discarded), and the
        // spell is paid from the pool. A diamond that is gone (answered meanwhile) means the plain way
        crate::glog!(g, "  {} casts Doomsday", g.player(p).name);
        if g.player(p).hand.contains(&c) && super::mine::led_on_table(g, p) {
            super::mine::crack_led(g, p, &[c], crate::engine::pool::Kind::B)?;
            crate::glog!(g, "    {} cracks Lion's Eye Diamond for BBB to pay for Doomsday", g.player(p).name);
            g.player_mut(p).stat("led_doomsday", 1);
            if !cast_one(g, p, c)? {
                return Ok(true);
            }
            if scn == Scn::Main {
                run_pile(g, p)?;
            }
            return Ok(true);
        }
    }
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

/// Noxious Revival's target while a pile is played: Noxious Revival puts the target card of a graveyard on top of its
/// owner's library, and the card the pile needs back is the Thassa's Oracle that was countered into the graveyard
pub fn revival_target(g: &Game, p: PlayerId) -> Option<CardId> {
    if !active(g, p) {
        return None;
    }
    g.player(p).gy.iter().copied().find(|&c| &*g.db.get(c).name == ORACLE)
}

/// does Veil of Summer answer everything the table may hold against the Oracle: it makes our spells uncounterable, but
/// it does not stop Mindbreak Trap (an exile, not a counter), so a table that may hold Mindbreak Trap does not count. Red
/// blasts no longer matter: once Veil resolves no counter works that turn (`rules::veil_respond`, `values::veil_shield`)
fn veil_answers(g: &Game, p: PlayerId) -> bool {
    !g.opps(p).any(|q| crate::ai::brain::prob_holding(g, q, |c| &*g.db.get(c).name == "Mindbreak Trap") > 0.0)
}
