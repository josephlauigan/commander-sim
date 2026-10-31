//! Thanos, the Mad Titan's power-up, for JD's `thanos` deck (rust/agents/thanos-urdragon.md). Oracle text
//! (data/scryfall_cache.json):
//!
//! - **Thanos, the Mad Titan** ({R}{W}{B}, 4/4 Legendary Creature — Eternal Villain): "Deathtouch, lifelink.
//!   Power-up — {C}{W}{U}{B}{R}{G}: Put two +1/+1 counters on Thanos. Choose odd or even. Destroy each other creature
//!   with mana value of the chosen quality. (Activate each power-up ability only once. Reduce the cost by his mana cost
//!   if he entered this turn. Zero is even.)"
//!
//! Deathtouch and lifelink are the card's keywords. The power-up is an activated ability with no timing restriction:
//! the AI uses it in its main phase or at the end-of-turn window, once per Thanos object: a Thanos that leaves and
//! comes back (a blink, a recast) is a new object and may power up again (rule 113.7). On the turn Thanos entered the cost is {C}{U}{G}; {C} is
//! colourless mana (Sol Ring, Ancient Tomb), not generic (`mana::plan_pay`). The parity is chosen as the ability
//! resolves: the one whose destroyed creatures are worth the most to the opponents against what they cost its
//! controller (`parity_swing`), nothing hard-coded. Left out: a token's mana value is 0 (a token copy really has the
//! copied card's), so every token is even.

use crate::cards::CardDb;
use crate::engine::life::check_state;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::ability_window;
use crate::engine::values::{indestructible, pval};
use crate::engine::zones::die;
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{PermId, PlayerId};
use crate::state::Game;
use crate::sym::intern;

pub const THANOS: &str = "Thanos, the Mad Titan";

/// the owner's stat (one per permanent id) holding the `born` stamp of the Thanos object that powered up
const USED: &str = "thanos|powerup";

fn used_key(m: PermId) -> crate::sym::Sym {
    intern(&format!("{USED}|{m:?}"))
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

/// a permanent's mana value: its card's, 0 for a token
pub fn mv(g: &Game, m: PermId) -> u32 {
    g.perm(m).cd.map_or(0, |c| g.db.get(c).cmc)
}

/// Thanos entered the battlefield this turn (the cost drops by his mana cost)
pub fn entered_this_turn(g: &Game, m: PermId) -> bool {
    g.perm(m).born > g.turn_enter
}

/// the power-up's cost now: (generic, pips). {C}{W}{U}{B}{R}{G}, or {C}{U}{G} the turn he entered
pub fn cost(g: &Game, m: PermId) -> (u32, &'static str) {
    if entered_this_turn(g, m) { (0, "CUG") } else { (0, "CWUBRG") }
}

/// this Thanos object (permanent m since it last entered) has used its power-up
pub fn used(g: &Game, m: PermId) -> bool {
    let p = g.perm(m).owner;
    g.player(p).stats.get(&used_key(m)).copied() == Some(g.perm(m).born as i64)
}

/// the creatures the power-up destroys for this parity (`even`: mana value 0, 2, 4, ...), Thanos left out
pub fn victims(g: &Game, src: PermId, even: bool) -> Vec<(PlayerId, PermId)> {
    g.players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().map(move |&m| (q.id, m)))
        .filter(|&(_, m)| m != src && live(g, m) && g.is_creature(m) && (mv(g, m) % 2 == 0) == even)
        .collect()
}

/// what destroying this parity's creatures is worth to p: the opponents' creatures lost (their board value and
/// threat) minus 1.2 times p's own, as the AI rates a board wipe (`brain::wipe_options`); indestructible ones survive
pub fn parity_swing(g: &Game, p: PlayerId, src: PermId, even: bool) -> f64 {
    let (mut ol, mut ml) = (0.0, 0.0);
    for (q, m) in victims(g, src, even) {
        if indestructible(g, m) {
            continue;
        }
        let v = pval(g, m).max(if g.perm(m).token { 0.3 } else { 0.6 + 0.2 * mv(g, m).min(6) as f64 });
        if q == p {
            ml += v;
        } else {
            ol += v;
        }
    }
    ol - 1.2 * ml
}

/// the better parity for p and its swing (a tie keeps even)
pub fn best_parity(g: &Game, p: PlayerId, src: PermId) -> (bool, f64) {
    let (e, o) = (parity_swing(g, p, src, true), parity_swing(g, p, src, false));
    if o > e { (false, o) } else { (true, e) }
}

/// the AI's power-up: once per Thanos object, with its cost payable, when the better parity destroys more of the opponents'
/// value than of its own (in the main phase, or at the end-of-turn window when Thanos didn't enter this turn)
fn thanos_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if crate::human::is_human(g, p) || g.perm(src).owner != p || !live(g, src) || used(g, src) {
        return Ok(vec![]);
    }
    let fresh = entered_this_turn(g, src);
    if post.is_none() && fresh {
        return Ok(vec![]); // the cheap cost is the main phase's (the end step is the same turn: no extra value)
    }
    let (gn, pips) = cost(g, src);
    if !can_pay(g, p, gn, pips, false) {
        return Ok(vec![]);
    }
    let (even, swing) = best_parity(g, p, src);
    if swing <= 1.0 {
        return Ok(vec![]);
    }
    // the wipe's utility on the scale `brain::wipe_options` uses, plus the two counters; the full cost spends six
    // mana a turbo deck wants for its combo, so it is worth less than on the turn he entered
    let u = swing / 2.2 - 2.5 + 0.5 - if fresh { 0.0 } else { 1.0 };
    Ok(vec![Opt {
        utility: u,
        label: format!("Thanos, the Mad Titan: power-up ({})", if even { "even" } else { "odd" }),
        act: Some(Action::Ability { src, f: thanos_go, arg: 0 }),
    }])
}

/// pay and activate; the parity is chosen as it resolves
pub fn thanos_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || used(g, src) {
        return Ok(false);
    }
    let (gn, pips) = cost(g, src);
    if !can_pay(g, p, gn, pips, false) || !pay(g, p, gn, pips, false)? {
        return Ok(false);
    }
    let born = g.perm(src).born as i64;
    let o = g.perm(src).owner;
    g.player_mut(o).stats.insert(used_key(src), born);
    if !ability_window(g, p, Some(src), "power-up", None, None)? {
        return Ok(true);
    }
    let (even, _) = best_parity(g, p, src);
    resolve(g, p, src, even)?;
    Ok(true)
}

/// the power-up resolves: two +1/+1 counters on Thanos (if he is still here), then each other creature of the chosen
/// parity is destroyed, all at once (each victim may answer it as it would a board wipe)
pub fn resolve(g: &mut Game, p: PlayerId, src: PermId, even: bool) -> Res {
    if live(g, src) {
        crate::engine::repl::put_counters(g, p, src, 2);
    }
    crate::glog!(g, "    Thanos, the Mad Titan's power-up: {} mana values", if even { "even" } else { "odd" });
    g.player_mut(p).stat(intern(if even { "thanos|even" } else { "thanos|odd" }), 1);
    let cd = g.perm(src).cd;
    crate::t6stats::wipe(g, p, cd);
    g.batch += 1; // destroyed together: they die simultaneously
    let prev_batch = g.cur_batch.replace(g.batch);
    let prev = g.destroyer.replace(p);
    let kind = intern("destroy");
    let mut doomed = vec![];
    let players: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    for q in players {
        if g.over {
            break;
        }
        let prot = crate::ai::wipe_response(g, q, kind, p)?;
        if prot == Some("all") {
            continue;
        }
        for (o, m) in victims(g, src, even) {
            if o == q && !(prot == Some("indes")) {
                doomed.push(m);
            }
        }
    }
    g.resolving += 1;
    let mut r = Ok(());
    for m in doomed {
        if r.is_ok() && g.perm(m).on_bf {
            r = die(g, m, kind);
        }
    }
    g.cur_batch = prev_batch;
    g.destroyer = prev;
    g.resolving -= 1;
    r?;
    if g.resolving == 0 && !g.trig_queue.is_empty() {
        crate::engine::stack::flush_triggers(g)?;
    }
    check_state(g)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if db.id(THANOS).is_some() {
        r.card(db, THANOS)?.options = Some(thanos_options);
    }
    Ok(())
}
