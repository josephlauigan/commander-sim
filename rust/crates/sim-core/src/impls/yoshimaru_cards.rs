//! Cards of the Yoshimaru / Tymna blink deck, modelled before that deck is registered: Yoshimaru, Ever Faithful, Minas
//! Tirith and The Seriema (its enters search is card code). (Kambal, Consul of Allocation is compiled from its Oracle text and needs no code.) Each card's text is
//! in the doc comment above its code.

use crate::cards::CardDb;
use crate::dsl::model::Filter;
use crate::engine::repl;
use crate::engine::stack::{ability_window_card, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::epow;
use crate::engine::zones::{agent_for, agent_take, draw, max_by, searchable};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

/// a legendary permanent: printed, a legendary token, or the Ring-bearer (`legends::legendary`)
fn legendary(g: &Game, m: PermId) -> bool {
    super::legends::legendary(g, m)
}

fn is_land_card(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).land)
}

// ================================================================== Yoshimaru, Ever Faithful
/// **Yoshimaru, Ever Faithful** ({W} 1/1, Partner): "Whenever another legendary permanent you control enters, put a
/// +1/+1 counter on Yoshimaru." A permanent that is not a land comes through the enters event (a stack trigger, so
/// Panharmonicon and Torpor Orb act on it); a land has no single enters event, so it is seen through landfall: every
/// land of Yoshimaru's controller that arrived since he last looked (a higher permanent number than the one he has
/// counted up to, which starts at his own) is checked, once, whatever else landfall does.
fn yoshimaru_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src || !live(g, src) || g.perm(src).owner != p || is_land_card(g, m) || !legendary(g, m) {
        return Ok(());
    }
    counter(g, src, p)
}

/// the counter, after the trigger's window (the table may respond; a countered trigger does nothing)
fn counter(g: &mut Game, src: PermId, p: PlayerId) -> Res {
    if trigger_window(g, p, Some(src), "a +1/+1 counter on Yoshimaru", None)? && live(g, src) {
        repl::put_plus(g, src, 1);
    }
    Ok(())
}

fn yoshimaru_landfall(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if !live(g, src) || g.perm(src).owner != p {
        return Ok(());
    }
    let seen = g.perm(src).data.int(DataKey::Entered).max(src.0 as i64);
    let fresh: Vec<PermId> = g.player(p).lands.iter().copied().filter(|l| l.0 as i64 > seen).collect();
    let Some(top) = fresh.iter().map(|l| l.0 as i64).max() else { return Ok(()) };
    g.perm_mut(src).data.set(DataKey::Entered, Val::Int(top));
    for l in fresh {
        if g.perm(l).on_bf && legendary(g, l) && live(g, src) {
            counter(g, src, p)?;
        }
    }
    Ok(())
}

// ================================================================== Minas Tirith
/// **Minas Tirith** (legendary land): enters tapped unless you control a legendary creature (the engine's enters rule,
/// from the card data). {T}: Add {W} (the `c=W` tag). {1}{W}, {T}: Draw a card. Activate only if you attacked with two
/// or more creatures this turn. The draw has no choice in it: the AI uses it with spare mana after combat, a person
/// through the land's hook options (`human::alt`), paying from their pool.
fn attacked_with_two(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    pl.atk_count.0 == Some(g.turn_stamp()) && pl.atk_count.1 >= 2
}

fn minas_options(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none()
        || g.perm(l).tapped
        || !attacked_with_two(g, p)
        || g.player(p).library.len() < 4
        || !super::lands::can_pay_without(g, p, l, 1, "W")
    {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.5,
        label: "Minas Tirith: draw a card".into(),
        act: Some(Action::Plan { f: minas_go, arg: l.0 as i64 }),
    }])
}

fn minas_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = PermId(arg as u32);
    if !g.player(p).lands.contains(&l)
        || g.perm(l).tapped
        || !attacked_with_two(g, p)
        || !super::lands::pay_without(g, p, l, 1, "W")?
    {
        return Ok(false);
    }
    g.perm_mut(l).tapped = true;
    crate::glog!(g, "  {} uses Minas Tirith", g.player(p).name);
    if ability_window_card(g, p, g.perm(l).card(), "draw a card", None, None)? {
        draw(g, p, 1, false)?;
    }
    Ok(true)
}

// ================================================================== The Seriema
/// **The Seriema** ({1}{W}{W}, legendary artifact, Spacecraft): when it enters, search for a legendary creature card
/// (the compiled `search`; a person picks the card). Station (tap another creature you control: put charge counters
/// equal to its power on it; only as a sorcery). At 7 or more charge counters it is a 5/5 artifact creature with
/// flying, and other tapped legendary creatures you control have indestructible. The AI stations with creatures that
/// can't attack anyway, or when the tap finishes it; a person chooses each creature to tap, one activation at a time.
const SERIEMA: &str = "The Seriema";
const STATION_GOAL: i64 = 7;

/// When The Seriema enters: search for a legendary creature card, reveal it, put it into your hand, then shuffle.
/// (The compiled `search` ignores "legendary" and would take any creature, so the card's override drops it.) A person
/// picks the card, or finds nothing; the AI takes the one its search pick prefers.
fn seriema_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || !live(g, src) || g.perm(src).owner != p {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "search for a legendary creature", None)? {
        return Ok(());
    }
    let legend = |g: &Game, c: CardId| {
        let d = g.db.get(c);
        d.creature && d.tag(Tag::Leg)
    };
    let found = if crate::human::is_human(g, p) {
        crate::human::choices::search(g, p, &legend, 1, "The Seriema: search for a legendary creature card", true)?
            .first()
            .copied()
    } else {
        let cands: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| legend(g, c)).collect();
        let f = Filter { kind: Some("creature".into()), ..Filter::default() };
        let pick = crate::ai::dsl_search_pick(g, p, &f, &cands)
            .or_else(|| max_by(&cands, |c| card_worth(g, p, c, false) * 1000.0 + g.db.get(c).cmc as f64));
        if let Some(c) = pick {
            let lib = &mut g.player_mut(p).library;
            if let Some(i) = lib.iter().position(|&x| x == c) {
                lib.remove(i);
            }
        }
        shuffle_library(g, p);
        pick
    };
    let Some(c) = found else { return Ok(()) };
    if let Some(a) = agent_for(g, p) {
        agent_take(g, a, p, c); // Opposition Agent
        return Ok(());
    }
    g.player_mut(p).hand.push(c);
    g.player_mut(p).seen.insert(c); // revealed
    g.player_mut(p).stat("tutored", 1);
    crate::ai::rules::mark_tutored(g, p, c);
    crate::engine::tutors::log_found(g, p, c, "hand", SERIEMA);
    Ok(())
}

fn charge(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.int(DataKey::Charge)
}

fn seriema_creature(g: &Game, m: PermId) -> bool {
    charge(g, m) >= STATION_GOAL
}

/// the creatures p could tap to station `src`: untapped, another creature p controls
fn station_candidates(g: &Game, p: PlayerId, src: PermId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| m != src && live(g, m) && !g.perm(m).tapped && g.is_creature(m))
        .collect()
}

/// flying at 7+, and indestructible for the other tapped legendary creatures its controller has
fn seriema_grant(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    if !seriema_creature(g, src) {
        return false;
    }
    if m == src {
        return kw == "flying";
    }
    kw == "indestructible"
        && g.perm(m).owner == g.perm(src).owner
        && g.perm(m).tapped
        && g.is_creature(m)
        && legendary(g, m)
}

fn seriema_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.perm(src).owner != p || !live(g, src) || seriema_creature(g, src) {
        return Ok(vec![]);
    }
    let cands = station_candidates(g, p, src);
    if cands.is_empty() {
        return Ok(vec![]);
    }
    if crate::human::is_human(g, p) {
        return Ok(vec![Opt {
            utility: 2.0,
            label: "The Seriema: station (tap another creature)".into(),
            act: Some(Action::Ability { src, f: station_go, arg: -1 }),
        }]);
    }
    // the AI: creatures that can't attack this turn first (the biggest), the rest only when the taps reach 7 at once
    let have = charge(g, src);
    let idle = |g: &Game, m: PermId| g.perm(m).sick || g.perm(m).noatk;
    let mut best: Option<(PermId, f64)> = None;
    let total: i64 = cands.iter().map(|&m| epow(g, m).max(0) as i64).sum();
    let finish = have + total >= STATION_GOAL;
    for &m in &cands {
        let pw = epow(g, m).max(0);
        if pw == 0 || !(idle(g, m) || finish) {
            continue;
        }
        let u = if finish { 4.0 + pw as f64 / 4.0 } else { 1.0 + pw as f64 / 4.0 };
        if best.is_none_or(|b| u > b.1) {
            best = Some((m, u));
        }
    }
    Ok(best
        .map(|(m, u)| Opt {
            utility: u,
            label: format!("The Seriema: station (tap {})", g.perm(m).name),
            act: Some(Action::Ability { src, f: station_go, arg: m.0 as i64 }),
        })
        .into_iter()
        .collect())
}

fn station_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    // only as a sorcery
    if !live(g, src)
        || g.perm(src).owner != p
        || seriema_creature(g, src)
        || crate::human::legal::sorcery_timing(g, p).is_some()
    {
        return Ok(false);
    }
    let cands = station_candidates(g, p, src);
    let t = if arg >= 0 {
        PermId(arg as u32)
    } else {
        let labels: Vec<String> =
            cands.iter().map(|&m| format!("{} (power {})", g.perm(m).name, epow(g, m).max(0))).collect();
        match crate::human::choices::choose(
            g,
            p,
            "choose",
            "The Seriema: tap which creature to put charge counters equal to its power on it?",
            &labels,
            Some("cancel"),
            None,
        )? {
            Some(i) => cands[i],
            None => return Ok(false),
        }
    };
    if !cands.contains(&t) {
        return Ok(false);
    }
    g.perm_mut(t).tapped = true; // the cost
    let n = epow(g, t).max(0);
    crate::glog!(g, "  {} stations The Seriema, tapping {} ({} charge)", g.player(p).name, g.perm(t).name, n);
    crate::engine::stack::ability_run(
        g,
        p,
        Some(src),
        None,
        &format!("station: {n} charge counters"),
        None,
        None,
        |g| {
            if !live(g, src) {
                return Ok(());
            }
            let k = repl::counters(g, p, repl::On::Perm(src), repl::Ctr::Other, n, repl::Path::ALL).max(0);
            let total = charge(g, src) + k as i64;
            g.perm_mut(src).data.set(DataKey::Charge, Val::Int(total));
            if total >= STATION_GOAL && !g.perm(src).data.truthy(DataKey::Anim) {
                let x = g.perm_mut(src);
                x.data.set(DataKey::Anim, Val::Bool(true)); // an artifact creature at 7+
                x.pow = 5;
                x.tgh = 5;
                crate::glog!(g, "    The Seriema is a 5/5 artifact creature");
            }
            Ok(())
        },
    )?;
    Ok(true)
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Yoshimaru, Ever Faithful")?;
    c.etb = Some(yoshimaru_etb);
    c.landfall = Some(yoshimaru_landfall);
    super::partials::at_once(c, crate::hooks::Event::Landfall, true);
    r.card(db, "Minas Tirith")?.land_options = Some(minas_options);
    let c = r.card(db, SERIEMA)?;
    c.etb = Some(seriema_etb);
    c.options = Some(seriema_options);
    c.grant_kw = Some(seriema_grant);
    Ok(())
}
