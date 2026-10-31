//! --- phase4-rules: generic rule checks in the policy (`audit/phase4/rules-design.md`). Three guards, each one check
//! that covers every card with its pattern:
//!
//! - **followup**: a spell is not offered when a payment it commits p to can't be made: a pact's upkeep cost (all the
//!   upkeep debts together, against the mana p will have at its next upkeep), a discard or sacrifice cost;
//! - **tutored**: a card a tutor found this turn or last (to hand or to the top of the library) is not discarded or
//!   binned by p's own choice while another choice exists;
//! - **combo**: a spent piece of one of p's lines (an instant or sorcery that is gone once it resolves and does nothing
//!   alone: no draw, no mana, no search) is not cast while another piece of the line is neither held nor findable by a
//!   tutor p holds and can cast now.
//!
//! The switch `AI_RULES`: unset, empty or `1` all three for every AI player (the default since the stage 1 paired
//! tests: all ten halves t1-t5 and all five JD decks tried level, `audit/recal/pairs-2026-10-31.md` section 3), `0`
//! none (the old play), or a list of guard names (`followup,tutored,combo`); `AI_RULES_DECKS=a,b` limits them to those
//! decks' players (a paired test's half). With a guard off its calls return before they read the game, change it or
//! draw a random number.

use crate::hooks::{Action, Opt};
use crate::ids::{CardId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;
use std::sync::OnceLock;

// ------------------------------------------------------------------ the switch
/// one guard (a bit of the switch's mask)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Guard {
    Followup = 1,
    Tutored = 2,
    Combo = 4,
}

pub const ALL: u8 = 7;

/// `AI_RULES` read as a mask of guards: unset, empty or `1` all (the default); `0` none; else guard names separated by
/// commas
pub fn parse_mode(v: Option<&str>) -> u8 {
    match v.map(str::trim) {
        Some("0") => 0,
        None | Some("") | Some("1") => ALL,
        Some(list) => list.split(',').map(str::trim).filter(|s| !s.is_empty()).fold(0, |m, s| {
            m | match s {
                "followup" => Guard::Followup as u8,
                "tutored" => Guard::Tutored as u8,
                "combo" => Guard::Combo as u8,
                _ => panic!("bad AI_RULES guard {s:?} (followup, tutored, combo)"),
            }
        }),
    }
}

fn env_mode() -> u8 {
    static M: OnceLock<u8> = OnceLock::new();
    *M.get_or_init(|| parse_mode(std::env::var("AI_RULES").ok().as_deref()))
}

thread_local! {
    /// a test's override on this thread: the mask of guards on, whatever `AI_RULES` says
    static FORCE: std::cell::Cell<Option<u8>> = const { std::cell::Cell::new(None) };
}

/// for tests: these guards on (a mask of `Guard`s, `ALL`, or 0) on this thread; None: back to `AI_RULES`
pub fn force(mask: Option<u8>) {
    FORCE.with(|f| f.set(mask));
}

/// `AI_RULES_DECKS` (for the paired tests' halves): unset or empty, every deck; else the deck keys, separated by
/// commas, whose players the guards apply to
fn env_decks() -> &'static Option<Vec<String>> {
    static D: OnceLock<Option<Vec<String>>> = OnceLock::new();
    D.get_or_init(|| {
        let v = std::env::var("AI_RULES_DECKS").ok()?;
        let keys: Vec<String> = v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        (!keys.is_empty()).then_some(keys)
    })
}

/// is this guard on for p? The mask is read first, so with the guard off nothing of the game is read.
pub fn on(g: &Game, p: PlayerId, guard: Guard) -> bool {
    let forced = FORCE.with(|f| f.get());
    let mask = forced.unwrap_or_else(env_mode);
    if mask & guard as u8 == 0 || crate::human::is_human(g, p) {
        return false;
    }
    match (forced, env_decks()) {
        (None, Some(keys)) => keys.iter().any(|k| k == g.player(p).key),
        _ => true,
    }
}

// ------------------------------------------------------------------ the policy's options
/// The cast options p's menu offers (`brain::main_options`, the end-of-turn window): those the follow-up or the combo
/// guard refuses are dropped (counted in the `rules|followup` and `rules|combo` stats).
pub fn filter_options(g: &mut Game, p: PlayerId, opts: &mut Vec<Opt>) -> crate::flow::Res {
    let (fu, cb) = (on(g, p, Guard::Followup), on(g, p, Guard::Combo));
    if !fu && !cb {
        return Ok(());
    }
    let mut keep = Vec::with_capacity(opts.len());
    for o in std::mem::take(opts) {
        let card = match &o.act {
            Some(Action::Cast { card, .. })
            | Some(Action::Removal { card, .. })
            | Some(Action::Wipe { card, .. })
            | Some(Action::Face { card, .. })
            | Some(Action::Crackle { card, .. }) => Some(*card),
            _ => None,
        };
        if let Some(c) = card {
            if fu && !(followup_ok(g, p, c) && compiled_cost_ok(g, p, c)?) {
                g.player_mut(p).stat("rules|followup", 1);
                continue;
            }
            if cb && !combo_ok(g, p, c) {
                g.player_mut(p).stat("rules|combo", 1);
                continue;
            }
        }
        keep.push(o);
    }
    *opts = keep;
    Ok(())
}

// ------------------------------------------------------------------ follow-up payments
/// the upkeep payment casting c commits p to (generic, pips): a `pactpay` cost (Slaughter Pact) or a pact
/// (Pact of Negation: "At the beginning of your next upkeep, pay {3}{U}{U}. If you don't, you lose the game.")
pub fn upkeep_due(g: &Game, c: CardId) -> Option<(u32, String)> {
    let t = &g.db.get(c).tags;
    if let Some(cost) = t.str(Tag::Pactpay) {
        return Some(crate::engine::cast::parse_cost(cost));
    }
    t.has(Tag::Pact).then(|| (3, "UU".to_string()))
}

/// Can p pay `due` on top of every upkeep debt it already owes (each Pact of Negation's {3}{U}{U}, each pact debt) at
/// its next upkeep? The mana estimate: p's mana sources as they will be then: every land and permanent of p's untapped
/// and no longer summoning-sick, its Treasures, nothing else (no land drop: the upkeep comes first; no mana from cards
/// in hand). A copy of the game is untapped, so this game is not touched.
pub fn next_upkeep_affordable(g: &Game, p: PlayerId, due: (u32, &str)) -> bool {
    let (mut gn, mut pips) = (due.0, due.1.to_string());
    let pl = g.player(p);
    for (dg, dp) in &pl.pact_debts {
        gn += dg;
        pips.push_str(dp);
    }
    for _ in 0..pl.pacts {
        gn += 3;
        pips.push_str("UU");
    }
    let mut g2 = g.clone();
    for l in g2.player(p).lands.clone() {
        g2.perm_mut(l).tapped = false;
    }
    for m in g2.player(p).perms.clone() {
        let x = g2.perm_mut(m);
        x.tapped = false;
        x.sick = false;
    }
    crate::engine::mana::can_pay(&g2, p, gn, &pips, false)
}

/// The follow-up guard on card c (anything of p's that can be cast with only the game to read: the counter pick uses
/// it too): its upkeep payment can be made at the next upkeep, a tagged discard cost has a card to discard, a tagged
/// sacrifice cost has a creature. True when the guard is off.
pub fn followup_ok(g: &Game, p: PlayerId, c: CardId) -> bool {
    if !on(g, p, Guard::Followup) {
        return true;
    }
    if let Some((gn, pips)) = upkeep_due(g, c)
        && !next_upkeep_affordable(g, p, (gn, &pips))
    {
        return false;
    }
    let d = g.db.get(c);
    if d.tag(Tag::Discard1) && !g.player(p).hand.iter().any(|&x| x != c) {
        return false; // Big Score: "As an additional cost to cast this spell, discard a card."
    }
    if d.tag(Tag::Needsac) && super::brain::spare_creature(g, p).is_none() {
        return false; // Diabolic Intent: "sacrifice a creature"
    }
    true
}

/// a compiled additional cost ("As an additional cost to cast this spell, sacrifice ... / discard a card") can be
/// paid: the engine's own dry run
fn compiled_cost_ok(g: &mut Game, p: PlayerId, c: CardId) -> crate::flow::Res<bool> {
    let d = g.db.get(c);
    if !d.has_dsl() || d.land {
        return Ok(true);
    }
    crate::engine::cast::additional_cost(g, p, c, true)
}

// ------------------------------------------------------------------ tutored cards
/// the turns taken so far in the game (every player's, extra turns too): "this turn or last" is a difference of 1
fn turn_no(g: &Game) -> u32 {
    g.players.iter().map(|pl| pl.turns).sum()
}

/// A tutor of p's has just put card c in p's hand or on top of p's library: mark it (marks older than last turn are
/// dropped). Nothing happens with the guard off.
pub fn mark_tutored(g: &mut Game, p: PlayerId, c: CardId) {
    if !on(g, p, Guard::Tutored) {
        return;
    }
    let now = turn_no(g);
    let marks = &mut g.player_mut(p).tutored;
    marks.retain(|&(x, t)| x != c && now.saturating_sub(t) <= 1);
    marks.push((c, now));
}

/// did a tutor of p's find c this turn or last (and is the guard on)?
pub fn tutored(g: &Game, p: PlayerId, c: CardId) -> bool {
    if !on(g, p, Guard::Tutored) {
        return false;
    }
    let now = turn_no(g);
    g.player(p).tutored.iter().any(|&(x, t)| x == c && now.saturating_sub(t) <= 1)
}

/// The cards p may choose among for a discard (or a self-mill) once the tutored ones are set aside: Some(the others)
/// when some of `cands` are freshly tutored and some are not; None (choose as today) otherwise.
pub fn spare_tutored(g: &Game, p: PlayerId, cands: &[CardId]) -> Option<Vec<CardId>> {
    if !on(g, p, Guard::Tutored) {
        return None;
    }
    let rest: Vec<CardId> = cands.iter().copied().filter(|&c| !tutored(g, p, c)).collect();
    (!rest.is_empty() && rest.len() < cands.len()).then_some(rest)
}

// ------------------------------------------------------------------ combo starts
fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// a spent piece: an instant or sorcery (gone once it resolves) that does nothing toward the line alone: it draws no
/// card, makes no mana and searches for nothing (those dig for the missing piece themselves)
pub fn spent_piece(g: &Game, p: PlayerId, c: CardId) -> bool {
    let d = g.db.get(c);
    !d.perm
        && !d.land
        && !super::brain::draws_cards(g, c)
        && crate::engine::mana::ritual_net(g, p, c) <= 0
        && super::planner::tutor_rule(g, c).is_none()
}

/// is a card of this piece group held by p apart from `except`: in hand, on the battlefield (lands too), a
/// commander in the command zone, or a card p put on top of its library?
fn group_held(g: &Game, p: PlayerId, grp: &[&str], except: CardId) -> bool {
    let pl = g.player(p);
    let named = |c: CardId| grp.contains(&cname(g, c));
    pl.hand.iter().any(|&c| c != except && named(c))
        || pl.perms.iter().filter(|&&m| !g.perm(m).phased).filter_map(|&m| g.perm(m).cd).any(named)
        || pl.lands.iter().filter(|&&l| g.perm(l).on_bf).any(|&l| named(g.perm(l).card()))
        || pl.cmds_in_zone().any(named)
        || crate::engine::tutors::known_top(g, p).into_iter().any(named)
}

/// the tutors in p's hand (not `except`) that p can pay for now, each with the library cards it may take
fn castable_tutors(g: &Game, p: PlayerId, except: CardId) -> Vec<(CardId, Vec<CardId>)> {
    let mut out = vec![];
    for &t in &g.player(p).hand {
        if t == except {
            continue;
        }
        let (gn, pips) = crate::engine::mana::cost_of(g, p, t);
        if !crate::engine::mana::can_pay(g, p, gn, &pips, false) {
            continue;
        }
        if let Some((_, finds)) = super::planner::tutor_targets(g, p, t) {
            out.push((t, finds));
        }
    }
    out
}

/// can p finish this line once c is cast: every other group held, the rest each found by a different tutor in hand
fn line_reachable(g: &Game, p: PlayerId, c: CardId, groups: &[&[&str]], tutors: &[(CardId, Vec<CardId>)]) -> bool {
    let name = cname(g, c);
    let mut used = vec![false; tutors.len()];
    for grp in groups {
        if grp.contains(&name) || group_held(g, p, grp, c) {
            continue;
        }
        let Some(i) = (0..tutors.len()).find(|&i| !used[i] && tutors[i].1.iter().any(|&x| grp.contains(&cname(g, x))))
        else {
            return false;
        };
        used[i] = true;
    }
    true
}

/// The combo guard on card c: a spent piece of p's lines is cast only when one of its lines can be finished (every
/// other piece held, or found by a tutor p holds and can cast now). True for any other card, or with the guard off.
pub fn combo_ok(g: &Game, p: PlayerId, c: CardId) -> bool {
    if !on(g, p, Guard::Combo) {
        return true;
    }
    let lines = crate::impls::combos::deck_lines_with(g, p, cname(g, c));
    if lines.is_empty() || !spent_piece(g, p, c) {
        return true;
    }
    let tutors = castable_tutors(g, p, c);
    lines.iter().any(|groups| line_reachable(g, p, c, groups, &tutors))
}
