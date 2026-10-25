//! Tier 4 pool deck `yuriko-dimir-ninjas` (EDHREC's bracket-4 average list): the cards the ability language doesn't
//! cover (rust/agents/t45-yuriko.md). Ninjutsu itself is t4.rs's; the new Ninjas' costs are in `NINJUTSU` below,
//! which `t4::ninjutsu_entry` reads after its own table.
//!
//! Ninjas:
//! - Kaito, Bane of Nightmares: ninjutsu {1}{U}{B}; during your turn, with loyalty, a 3/4 Ninja creature with
//!   hexproof; +1 an emblem (Ninjas you control get +1/+1, one per emblem); 0 surveil 2, then draw a card per opponent
//!   who lost life this turn; -2 tap the best opposing creature and put two stun counters on it. Left out: damage
//!   dealt to him while he is a creature does not remove loyalty (it is marked as on any creature).
//! - Nashi, Moon Sage's Scion: ninjutsu {3}{B}; combat damage to a player: exile the top card of each player's
//!   library, cast the best nonland one of them by paying life equal to its mana value. Left out: the card is cast
//!   at once (not later in the turn) and an exiled land is never played.
//! - Ninja of the Deep Hours: ninjutsu {1}{U} (its draw is compiled).
//! - Sakashima's Student: ninjutsu {1}{U}; enters as a copy of the best creature, a Ninja in addition (the clone code
//!   of t6_copies). The ninjutsu choice weighs what it would copy, and skips it with nothing to copy.
//! - Skullsnatcher: ninjutsu {B}; combat damage to a player: exile up to two cards from that player's graveyard (the
//!   two worth most to them).
//! - Satoru, the Infiltrator: a nontoken creature of yours that enters without being cast (ninjutsu, reanimation, Fallen
//!   Shinobi) draws a card. Left out: a creature cast for no mana counts as cast, and creatures entering together draw
//!   one card each instead of one in all.
//!
//! Evasion:
//! - Cover of Darkness: names the most common creature type of the deck as it enters; creatures of that type have
//!   fear (rules::evasion_blocked reads fear).
//! - Shizo, Death's Storehouse: {B},{T}: target legendary creature gains fear until end of turn: the AI uses it in its
//!   first main phase on its best ready legendary attacker that an opposing creature could block.
//! - Mothdust Changeling: at the beginning of combat it taps a summoning-sick creature of yours to gain flying until
//!   end of turn, when an opponent could block it.
//! - Wingcrafter: soulbond (bulk_white's pairing data): pairs with the best creature without flying; both fly.
//!
//! Spells:
//! - Temporal Mastery: an extra turn, exiled; miracle {1}{U} when it was the turn's first card drawn.
//! - Temporal Trespass: delve (partials' delve); an extra turn, exiled.
//! - Scheming Symmetry: you and the least threatening opponent each put a card from the library on top (the tutors'
//!   own choice, Yuriko's reveal list for you).
//! - Shadow of Mortality: costs {1} less per life below 40.
//! - Agadeem's Awakening // Agadeem, the Undercrypt: X picks the best creature card of each mana value up to X in
//!   your graveyard; or played as a land ({B}, 3 life for untapped).
//! - Sea Gate Restoration // Sea Gate, Reborn: draw cards equal to your hand plus one, no maximum hand size for the
//!   game; or played as a land ({U}).
//! - Hydroelectric Specimen // Hydroelectric Laboratory: flashed in at an opponent's instant or sorcery aimed at your
//!   key permanent (t6_counters::stack_response), the spell's target becomes the Specimen; or played as a land ({U}).
//!   Left out: the Specimen spell itself is not on the stack to be countered.
//! - Clearwater Pathway // Murkwater Pathway: played as the face whose colour the deck's lands make less of (the
//!   hand's pips break a tie).

use crate::cards::{CardDb, Colors};
use crate::engine::cast::{cast_card, castable, on_cast};
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::stack::{ability_window_card, trigger_window};
use crate::engine::tutors::{card_worth, tutor_to_top};
use crate::engine::values::{epow, has_type, pval, threat, untargetable};
use crate::engine::zones::{Enter, draw, enter, max_by, min_by};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::impls::common::{WalkerAb, walker};
use crate::impls::partials::{at_once, eot_kw};
use crate::state::{Ctx, DataKey, Game, StackKind, Val};
use crate::sym::Sym;
use crate::tag::Tag;

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn pname(g: &Game, p: PlayerId) -> &'static str {
    g.player(p).name
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn remove_first(v: &mut Vec<CardId>, c: CardId) -> bool {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
        return true;
    }
    false
}

fn named_perm(g: &Game, m: PermId, n: &str) -> bool {
    g.perm(m).cd.is_some_and(|c| cname(g, c) == n)
}

// ======================================================== ninjutsu costs and the ninjutsu choice
/// the new Ninjas' ninjutsu costs (generic, pips), read by `t4::ninjutsu_entry` after t4's own table
pub const NINJUTSU: [(&str, u32, &str); 5] = [
    (KAITO, 1, "UB"),
    (NASHI, 3, "B"),
    ("Ninja of the Deep Hours", 1, "U"),
    (STUDENT, 1, "U"),
    (SKULLSNATCHER, 0, "B"),
];

const KAITO: &str = "Kaito, Bane of Nightmares";
const NASHI: &str = "Nashi, Moon Sage's Scion";
const STUDENT: &str = "Sakashima's Student";
const SKULLSNATCHER: &str = "Skullsnatcher";

/// the worth of putting card c in by ninjutsu, when it isn't its power: Kaito (a 3/4 walker that stays), Sakashima's
/// Student (what it would copy; nothing to copy: never)
pub fn ninja_worth(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    match cname(g, c) {
        KAITO => Some(5),
        STUDENT => Some(crate::impls::t6_copies::best_creature_copy(g, p).map_or(-100, |v| v.round() as i32)),
        _ => None,
    }
}

// ======================================================== Kaito, Bane of Nightmares
const KAITO_EMBLEM: &str = "kaito_bane";

/// during its controller's turn, with a loyalty counter, Kaito is a 3/4 Ninja creature
fn kaito_update(g: &mut Game, src: PermId) {
    let x = g.perm(src);
    let on_ = x.on_bf && g.active == Some(x.owner) && x.loyalty.unwrap_or(0) > 0;
    let x = g.perm_mut(src);
    (x.pow, x.tgh) = (3, 4);
    x.data.set(DataKey::Anim, Val::Bool(on_));
    let has = x.ttypes.contains(&"ninja");
    if on_ && !has {
        x.ttypes.push("ninja");
    } else if !on_ && has {
        x.ttypes.retain(|&t| t != "ninja");
    }
}

fn kaito_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        kaito_update(g, src);
    }
    Ok(())
}

fn kaito_sba(g: &mut Game, src: Src) -> Res {
    kaito_update(g, src);
    Ok(())
}

fn kaito_upkeep(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    kaito_update(g, src);
    Ok(())
}

/// hexproof while he is a creature (your turn)
fn kaito_hexproof(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    m == src && kw == "hexproof" && g.perm(src).data.truthy(DataKey::Anim)
}

/// the emblems' bonus: Ninjas you control get +1/+1 per emblem
fn kaito_pt(g: &Game, m: PermId) -> (i32, i32) {
    let n = g.player(owner(g, m)).emblems.iter().filter(|&&e| e == KAITO_EMBLEM).count() as i32;
    if n == 0 || !has_type(g, m, "ninja") {
        return (0, 0);
    }
    (n, n)
}

fn ninjas_of(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| live(g, m) && g.is_creature(m) && has_type(g, m, "ninja")).count()
}

fn kaito_plus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    Some(1.5 + 0.5 * ninjas_of(g, p) as f64)
}

/// +1: an emblem (several stack: each gives +1/+1)
fn kaito_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    g.player_mut(p).emblems.push(KAITO_EMBLEM);
    g.selfpt = true; // creature_pt is read while selfpt is on
    crate::glog!(g, "    {} gets an emblem: Ninjas they control get +1/+1", pname(g, p));
    Ok(())
}

/// the opponents of p who lost life this turn
fn opps_hurt(g: &Game, p: PlayerId) -> u32 {
    let st = g.turn_stamp();
    g.opps(p).filter(|&q| g.player(q).lost_turn.is_some_and(|x| x.0 == st && x.1 > 0)).count() as u32
}

fn kaito_zero_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    Some(1.2 + 1.2 * opps_hurt(g, p) as f64)
}

/// 0: surveil 2, then draw a card for each opponent who lost life this turn
fn kaito_zero(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    crate::cardcode::scry(g, p, 2, true)?;
    let n = opps_hurt(g, p);
    if n > 0 {
        draw(g, p, n, false)?;
    }
    Ok(())
}

fn stun_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| live(g, m) && g.is_creature(m) && !untargetable(g, m))
        .collect();
    max_by(&cs, |m| pval(g, m) + if g.perm(m).tapped { -1.0 } else { 0.0 })
}

fn kaito_minus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let t = stun_target(g, p)?;
    let v = pval(g, t);
    (v >= 3.0).then_some(0.8 + 0.3 * v.min(10.0))
}

/// -2: tap target creature, two stun counters on it
fn kaito_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let Some(t) = stun_target(g, p) else { return Ok(()) };
    let n = g.perm(t).data.int(DataKey::Stun);
    let x = g.perm_mut(t);
    x.tapped = true;
    x.data.set(DataKey::Stun, Val::Int(n + 2));
    crate::glog!(g, "    {} is tapped with two stun counters", g.perm(t).name);
    Ok(())
}

static KAITO_ABS: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "emblem: Ninjas +1/+1", val: kaito_plus_val, eff: kaito_plus },
    WalkerAb { delta: 0, label: "surveil 2, draw per opponent hurt", val: kaito_zero_val, eff: kaito_zero },
    WalkerAb { delta: -2, label: "tap and stun a creature", val: kaito_minus_val, eff: kaito_minus },
];

// ======================================================== Nashi, Moon Sage's Scion
/// combat damage to a player: exile the top card of each library; cast the best nonland one for life
fn nashi(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if a != src || !trigger_window(g, p, Some(src), "exile the top card of each library", Some(4.0))? {
        return Ok(());
    }
    let mut got: Vec<(CardId, PlayerId)> = vec![];
    let seats: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    for q in seats {
        if let Some(c) = g.player_mut(q).library.pop() {
            g.player_mut(q).exile.push(c);
            got.push((c, q));
        }
    }
    let life = g.player(p).life;
    let ok: Vec<(CardId, PlayerId)> = got
        .into_iter()
        .filter(|&(c, _)| {
            let d = g.db.get(c);
            !d.land && (d.cmc as i32) < life - 8 && castable(g, p, c, "hand")
        })
        .collect();
    let Some(best) = ok.iter().copied().max_by(|x, y| {
        let (a, b) = (card_worth(g, p, x.0, false), card_worth(g, p, y.0, false));
        a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
    }) else {
        return Ok(());
    };
    let (c, q) = best;
    if card_worth(g, p, c, false) < 2.0 * g.db.get(c).cmc as f64 {
        return Ok(());
    }
    let mv = g.db.get(c).cmc as i32;
    remove_first(&mut g.player_mut(q).exile, c);
    lose_life(g, p, mv, Some(p), "other", None)?;
    crate::glog!(g, "    {} casts {} with Nashi for {} life", pname(g, p), cname(g, c), mv);
    g.player_mut(p).hand.push(c);
    let own = if q == p { None } else { Some(q) };
    cast_card(g, p, c, "hand", Ctx { owner: own, ..Ctx::default() })?;
    Ok(())
}

// ======================================================== Skullsnatcher
/// combat damage to a player: exile up to two target cards from that player's graveyard
fn skullsnatcher(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    if a != src || g.player(d).gy.is_empty() {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "exile two cards from a graveyard", None)? {
        return Ok(());
    }
    for _ in 0..2 {
        let gy = g.player(d).gy.clone();
        let Some(c) = max_by(&gy, |c| card_worth(g, d, c, true)) else { break };
        remove_first(&mut g.player_mut(d).gy, c);
        g.player_mut(d).exile.push(c);
        crate::glog!(g, "    Skullsnatcher exiles {} from {}'s graveyard", cname(g, c), pname(g, d));
    }
    Ok(())
}

// ======================================================== Satoru, the Infiltrator
/// a nontoken creature of yours entering without being cast: draw a card
fn satoru(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if p != o || owner(g, m) != o || g.perm(m).token || !g.is_creature(m) || g.last_cast_etb {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "draw a card", None)? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

// ======================================================== Cover of Darkness
fn ctype(g: &Game, m: PermId) -> Option<Sym> {
    match g.perm(m).data.get(DataKey::Ctype) {
        Some(Val::Str(s)) => Some(*s),
        _ => None,
    }
}

fn cover_as(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    if !named_perm(g, m, COVER) {
        return Ok(());
    }
    let t = crate::impls::galadriel::choose_type(g, p, COVER)?;
    g.perm_mut(m).data.set(DataKey::Ctype, Val::Str(t));
    crate::glog!(g, "    Cover of Darkness names {t}");
    Ok(())
}

const COVER: &str = "Cover of Darkness";

/// creatures of the chosen type have fear
fn cover_fear(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "fear" && g.is_creature(m) && ctype(g, src).is_some_and(|t| has_type(g, m, t))
}

// ======================================================== Shizo, Death's Storehouse
fn land_ready(g: &Game, l: LandId, p: PlayerId) -> bool {
    g.player(p).lands.contains(&l) && !g.land(l).tapped
}

/// an opponent has an untapped creature that could block m now and that fear would stop
fn fear_helps(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.opps(p).any(|q| {
        g.player(q).perms.iter().any(|&b| {
            live(g, b)
                && g.is_creature(b)
                && !g.perm(b).tapped
                && crate::engine::combat::can_block(g, b, m)
                && !crate::engine::values::colors_of(g, b).has('B')
                && !crate::engine::values::ptypes(g, b).has(crate::cards::Types::ARTIFACT)
        })
    })
}

fn shizo_options(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.active != Some(p) || !land_ready(g, l, p) {
        return Ok(vec![]);
    }
    if !crate::impls::lands::can_pay_without(g, p, l, 0, "B") {
        return Ok(vec![]);
    }
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            live(g, m)
                && g.is_creature(m)
                && crate::impls::mine::is_legendary(g, m)
                && !x.tapped
                && !x.noatk
                && (!x.sick || crate::engine::combat::has_haste(g, m))
                && !crate::dsl::has_kw(g, m, "fear")
                && fear_helps(g, p, m)
        })
        .collect();
    let Some(best) = max_by(&cands, |m| epow(g, m) as f64 + if g.perm(m).is_cmd { 3.0 } else { 0.0 }) else {
        return Ok(vec![]);
    };
    Ok(vec![Opt {
        utility: 1.0 + 0.2 * epow(g, best) as f64 + if g.perm(best).is_cmd { 1.0 } else { 0.0 },
        label: format!("Shizo: {} gains fear", g.perm(best).name),
        act: Some(Action::Plan { f: shizo_go, arg: l.0 as i64 | (best.0 as i64) << 32 }),
    }])
}

fn shizo_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId((arg & 0xFFFF_FFFF) as u32);
    let m = PermId((arg >> 32) as u32);
    if !land_ready(g, l, p) || !live(g, m) || !crate::impls::lands::pay_without(g, p, l, 0, "B")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    if ability_window_card(g, p, g.land(l).cd, "gains fear", None, Some(m))? && live(g, m) {
        eot_kw(g, m, "fear");
        crate::glog!(g, "    {} gains fear until end of turn (Shizo)", g.perm(m).name);
    }
    Ok(true)
}

// ======================================================== Mothdust Changeling
const MOTHDUST: &str = "Mothdust Changeling";

fn flies(g: &Game, m: PermId) -> bool {
    g.perm(m).fly || crate::dsl::has_kw(g, m, "flying")
}

/// the creature Mothdust taps: an untapped summoning-sick one of its controller's (it can't attack anyway)
fn moth_fodder(g: &Game, src: PermId) -> Option<PermId> {
    let p = owner(g, src);
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            m != src
                && live(g, m)
                && g.is_creature(m)
                && !g.perm(m).tapped
                && (g.perm(m).noatk || (g.perm(m).sick && !crate::engine::combat::has_haste(g, m)))
        })
        .collect();
    min_by(&cs, |m| pval(g, m))
}

/// Mothdust can attack now and gain flying for it
fn moth_ready(g: &Game, src: PermId) -> bool {
    let x = g.perm(src);
    live(g, src) && !x.tapped && (!x.sick || crate::engine::combat::has_haste(g, src)) && !flies(g, src)
}

/// beginning of combat (its controller's first): tap a creature that can't attack for flying until end of turn
fn moth_combat(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if owner(g, src) != p || g.active != Some(p) || !moth_ready(g, src) {
        return Ok(());
    }
    let blockable = g.opps(p).any(|q| crate::engine::combat::blockable(g, q, src));
    if !blockable {
        return Ok(());
    }
    let Some(t) = moth_fodder(g, src) else { return Ok(()) };
    g.perm_mut(t).tapped = true;
    eot_kw(g, src, "flying");
    crate::glog!(g, "    Mothdust Changeling taps {} to gain flying", g.perm(t).name);
    Ok(())
}

/// pool_ai's evasive (the ninjutsu attacker filter, its mana reserve): fear, or a Mothdust Changeling about to fly
pub fn evasive(g: &Game, m: PermId) -> bool {
    if g.hooks.is_empty() {
        return false;
    }
    crate::dsl::has_kw(g, m, "fear") || (named_perm(g, m, MOTHDUST) && moth_ready(g, m) && moth_fodder(g, m).is_some())
}

// ======================================================== Wingcrafter
/// soulbond: pairs as it or another creature enters, with the best creature without flying
fn wing_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    use crate::impls::bulk_white::{soulbond_pair, soulbond_partner};
    let o = owner(g, src);
    if !g.perm(src).on_bf || owner(g, m) != o || !g.is_creature(m) || soulbond_partner(g, src).is_some() {
        return Ok(());
    }
    let pick = if m == src {
        let cs: Vec<PermId> = g
            .player(o)
            .perms
            .iter()
            .copied()
            .filter(|&x| x != src && live(g, x) && g.is_creature(x) && soulbond_partner(g, x).is_none())
            .collect();
        let grounded: Vec<PermId> = cs.iter().copied().filter(|&x| !flies(g, x)).collect();
        max_by(if grounded.is_empty() { &cs } else { &grounded }, |x| pval(g, x))
    } else if soulbond_partner(g, m).is_none() {
        Some(m)
    } else {
        None
    };
    let Some(t) = pick else { return Ok(()) };
    if !crate::engine::stack::may_window(g, o, Some(src), "pair (soulbond)", Some(3.0))? {
        return Ok(());
    }
    soulbond_pair(g, src, t);
    g.dsl_on = true; // the flying grant is read through the keyword checks
    Ok(())
}

/// both creatures of the pair have flying
fn wing_fly(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "flying" && crate::impls::bulk_white::soulbond_partner(g, src).is_some_and(|o| m == src || m == o)
}

// ======================================================== extra turns: Temporal Mastery, Temporal Trespass
fn extra_turn(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    crate::glog!(g, "  {}: {} resolves (an extra turn follows)", pname(g, p), cname(g, c));
    crate::engine::turn::add_extra_turn(g, p, false);
    Ok("exile")
}

/// Temporal Trespass: delve (the least useful graveyard cards), then the extra turn
fn trespass(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let n = crate::impls::partials::delve_count(g, p, c) as usize;
    crate::impls::partials::exile_fodder(g, p, c, n);
    extra_turn(g, p, c, ctx)
}

/// an extra turn is always worth casting: high, below the protection a deck casts first
fn extra_turn_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.extra_turns.iter().any(|&(q, _)| q == p) { 40 } else { 75 }
}

/// miracle {1}{U}: Temporal Mastery was this turn's first card drawn
fn mastery_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let Some((st, mc)) = g.player(p).miracle else { return Ok(vec![]) };
    if post.is_none() || st != g.turn_stamp() || mc != c || !g.player(p).hand.contains(&c) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 1, "U", false) || !castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 6.0,
        label: "Temporal Mastery (miracle)".into(),
        act: Some(Action::Plan { f: mastery_go, arg: c.0 as i64 }),
    }])
}

fn mastery_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "U", false) {
        return Ok(false);
    }
    pay(g, p, 1, "U", false)?;
    g.player_mut(p).miracle = None;
    crate::glog!(g, "  {} casts Temporal Mastery for its miracle cost", pname(g, p));
    cast_card(g, p, c, "hand", Ctx::default())?;
    Ok(true)
}

// ======================================================== Scheming Symmetry
/// you and the opponent who threatens least each put a card from the library on top
fn symmetry(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let other = opps
        .iter()
        .copied()
        .min_by(|&a, &b| threat(g, p, a).partial_cmp(&threat(g, p, b)).unwrap_or(std::cmp::Ordering::Equal));
    tutor_to_top(g, p, "any", 0)?;
    if let Some(q) = other {
        crate::glog!(g, "    {} also searches (Scheming Symmetry)", pname(g, q));
        tutor_to_top(g, q, "any", 0)?;
    }
    Ok("gy")
}

fn prio_55(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    55
}

fn prio_0(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

// ======================================================== Shadow of Mortality
/// {X} less, X: the life below the starting 40
fn mortality_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    -(40 - g.player(p).life).max(0)
}

// ======================================================== spell // land cards
const AGADEEM: &str = "Agadeem's Awakening // Agadeem, the Undercrypt";
const SEA_GATE: &str = "Sea Gate Restoration // Sea Gate, Reborn";
/// Hydroelectric Specimen // Hydroelectric Laboratory (t6_counters::STACK_RESPONSES)
pub const SPECIMEN: &str = "Hydroelectric Specimen // Hydroelectric Laboratory";

fn mdfc_cols(g: &Game, _p: PlayerId, l: LandId) -> Colors {
    Colors::from_letters(if cname(g, g.land(l).cd) == AGADEEM { "B" } else { "U" })
}

/// the land face, when the land drop has no other use (t6_lands' rule)
fn land_face_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    Ok(crate::impls::t6_lands::mdfc_land_opt(g, c, p, post).into_iter().collect())
}

/// the best creature card of each mana value up to x in p's graveyard
fn agadeem_picks(g: &Game, p: PlayerId, x: i32) -> Vec<CardId> {
    let gy = &g.player(p).gy;
    (0..=x.max(0) as u32)
        .filter_map(|mv| {
            let cs: Vec<CardId> =
                gy.iter().copied().filter(|&c| g.db.get(c).creature && g.db.get(c).cmc == mv).collect();
            max_by(&cs, |c| card_worth(g, p, c, false))
        })
        .collect()
}

/// the X that returns the most for the mana: (X, the worth returned)
fn agadeem_plan(g: &Game, p: PlayerId) -> Option<(i32, f64)> {
    let mut best: Option<(i32, f64)> = None;
    for x in 0..=10 {
        if !can_pay(g, p, x as u32, "BBB", false) {
            break;
        }
        let v: f64 = agadeem_picks(g, p, x).iter().map(|&c| card_worth(g, p, c, false)).sum();
        if best.is_none_or(|b| v > b.1 + 0.01) {
            best = Some((x, v));
        }
    }
    best.filter(|b| b.1 >= 8.0)
}

fn agadeem_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut o = land_face_options(g, c, p, post)?;
    if post.is_some()
        && g.active == Some(p)
        && castable(g, p, c, "hand")
        && let Some((x, v)) = agadeem_plan(g, p)
    {
        o.push(Opt {
            utility: 1.0 + 0.15 * v.min(40.0),
            label: format!("Agadeem's Awakening (X = {x})"),
            act: Some(Action::Plan { f: agadeem_go, arg: c.0 as i64 }),
        });
    }
    Ok(o)
}

fn agadeem_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) {
        return Ok(false);
    }
    let Some((x, _)) = agadeem_plan(g, p) else { return Ok(false) };
    let (gn, pips) = cost_of(g, p, c);
    if !can_pay(g, p, gn + x as u32, &pips, false) || !pay(g, p, gn + x as u32, &pips, false)? {
        return Ok(false);
    }
    cast_card(g, p, c, "hand", Ctx { x, ..Ctx::default() })?;
    Ok(true)
}

fn agadeem_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let picks = agadeem_picks(g, p, ctx.x);
    crate::glog!(g, "    Agadeem's Awakening (X = {}) returns {} creature card(s)", ctx.x, picks.len());
    for c in picks {
        if remove_first(&mut g.player_mut(p).gy, c) {
            enter(g, p, c, Enter::default())?;
        }
    }
    Ok("gy")
}

/// draw cards equal to the cards in hand plus one; no maximum hand size for the rest of the game
fn sea_gate_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = g.player(p).hand.len() as u32 + 1;
    g.player_mut(p).t0s.nomax = true;
    draw(g, p, n, false)?;
    Ok("gy")
}

fn sea_gate_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    // the hand after casting it: the more cards, the bigger the draw
    let n = g.player(p).hand.len().saturating_sub(1);
    if n >= 3 { 60 } else { 45 }
}

/// q answers an opponent's instant or sorcery aimed at its key permanent: Hydroelectric Specimen flashes in and the
/// spell's single target becomes it
pub fn specimen_response(g: &mut Game, q: PlayerId, id: u32) -> Res<bool> {
    let Some(i) = g.stack.iter().position(|x| x.id == id) else { return Ok(false) };
    let it = g.stack[i].clone();
    let Some(spell) = it.card else { return Ok(false) };
    let Some(t) = it.ctx.target else { return Ok(false) };
    let d = g.db.get(spell);
    if it.kind != StackKind::Spell || it.controller == q || !(d.instant || d.sorcery) || it.ctx.face.is_some() {
        return Ok(false);
    }
    let harmful = d.tags.str(Tag::Rem).is_some();
    let creature_ok = d.tags.str(Tag::Tgt).is_none_or(|s| s.contains('c') || s == "nl");
    if !harmful || !creature_ok || !g.perm(t).on_bf || owner(g, t) != q || !(pval(g, t) >= 4.0 || g.perm(t).is_cmd) {
        return Ok(false);
    }
    let Some(c) = g.player(q).hand.iter().copied().find(|&x| cname(g, x) == SPECIMEN) else { return Ok(false) };
    if !castable(g, q, c, "hand") {
        return Ok(false);
    }
    let (gn, pips) = cost_of(g, q, c);
    if !can_pay(g, q, gn, &pips, false) || !pay(g, q, gn, &pips, false)? {
        return Ok(false);
    }
    remove_first(&mut g.player_mut(q).hand, c);
    let pl = g.player_mut(q);
    pl.spells_this_turn += 1;
    pl.cast_names.insert(c);
    pl.stat("spells_cast", 1);
    on_cast(g, q, c)?;
    let m = enter(g, q, c, Enter { was_cast: true, ..Enter::default() })?;
    g.stack_pushes += 1;
    if let Some(j) = g.stack.iter().position(|x| x.id == id)
        && g.perm(m).on_bf
    {
        g.stack[j].ctx.target = Some(m);
        crate::glog!(g, "    Hydroelectric Specimen: {} now targets the Specimen", cname(g, spell));
    }
    Ok(true)
}

// ======================================================== Clearwater Pathway // Murkwater Pathway
const PATHWAY: &str = "Clearwater Pathway // Murkwater Pathway";

/// the face p plays: the colour (U or B) its other lands make less of; the hand's pips break a tie
fn pathway_face(g: &Game, p: PlayerId, l: LandId) -> &'static str {
    let pl = g.player(p);
    let src = |ch: char| {
        pl.lands.iter().filter(|&&x| x != l && crate::engine::mana::land_colors_now(g, p, x).has(ch)).count() as i32
    };
    let pips =
        |ch: char| pl.hand.iter().map(|&c| g.db.get(c).pips.chars().filter(|&x| x == ch).count() as i32).sum::<i32>();
    let (u, b) = (src('U') * 10 - pips('U'), src('B') * 10 - pips('B'));
    if b < u { "B" } else { "U" }
}

fn pathway_etb(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    crate::impls::t6_lands::traitors_check(g, p, l)?;
    let face = pathway_face(g, p, l);
    g.land_mut(l).data.set(DataKey::ChosenColor, Val::Str(face));
    crate::glog!(
        g,
        "    {} plays {}",
        pname(g, p),
        if face == "B" { "Murkwater Pathway" } else { "Clearwater Pathway" }
    );
    Ok(())
}

fn pathway_cols(g: &Game, _p: PlayerId, l: LandId) -> Colors {
    match g.land(l).data.get(DataKey::ChosenColor) {
        Some(Val::Str(s)) => Colors::from_letters(s),
        _ => Colors::from_letters("U"),
    }
}

// ======================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    // Kaito, Bane of Nightmares
    walker(r, db, KAITO, &KAITO_ABS)?;
    let c = r.card(db, KAITO)?;
    c.etb = Some(kaito_etb);
    c.sba = Some(kaito_sba);
    c.upkeep = Some(kaito_upkeep);
    c.grant_kw = Some(kaito_hexproof);
    at_once(c, Event::Etb, true);
    at_once(c, Event::Upkeep, true);
    r.creature_pt.push(kaito_pt);
    // the Ninjas' combat damage triggers (Ninja of the Deep Hours' draw is compiled)
    r.card(db, NASHI)?.combat_damage = Some(nashi);
    r.card(db, SKULLSNATCHER)?.combat_damage = Some(skullsnatcher);
    let c = r.card(db, STUDENT)?;
    c.enter_copy = Some(crate::impls::t6_copies::clone_copy);
    c.prio = Some(crate::impls::t6_copies::clone_prio);
    r.card(db, "Satoru, the Infiltrator")?.etb = Some(satoru);
    // evasion
    let c = r.card(db, COVER)?;
    c.as_enters = Some(cover_as);
    c.grant_kw = Some(cover_fear);
    r.card(db, "Shizo, Death's Storehouse")?.land_options = Some(shizo_options);
    let c = r.card(db, MOTHDUST)?;
    c.crew = Some(moth_combat);
    at_once(c, Event::Crew, true);
    let c = r.card(db, "Wingcrafter")?;
    c.etb = Some(wing_etb);
    c.grant_kw = Some(wing_fly);
    // spells
    let c = r.card(db, "Temporal Mastery")?;
    c.resolve = Some(extra_turn);
    c.prio = Some(extra_turn_prio);
    c.hand_options = Some(mastery_options);
    let c = r.card(db, "Temporal Trespass")?;
    c.resolve = Some(trespass);
    c.prio = Some(extra_turn_prio);
    c.self_cost = Some(crate::impls::partials::delve_cost);
    let c = r.card(db, "Scheming Symmetry")?;
    c.resolve = Some(symmetry);
    c.prio = Some(prio_55);
    r.card(db, "Shadow of Mortality")?.self_cost = Some(mortality_cost);
    // spell // land cards
    let c = r.card(db, AGADEEM)?;
    c.land_cols = Some(mdfc_cols);
    c.hand_options = Some(agadeem_options);
    c.resolve = Some(agadeem_resolve);
    c.prio = Some(prio_0);
    let c = r.card(db, SEA_GATE)?;
    c.land_cols = Some(mdfc_cols);
    c.hand_options = Some(land_face_options);
    c.resolve = Some(sea_gate_resolve);
    c.prio = Some(sea_gate_prio);
    let c = r.card(db, SPECIMEN)?;
    c.land_cols = Some(mdfc_cols);
    c.hand_options = Some(land_face_options);
    let c = r.card(db, PATHWAY)?;
    c.land_etb = Some(pathway_etb);
    c.land_cols = Some(pathway_cols);
    Ok(())
}
