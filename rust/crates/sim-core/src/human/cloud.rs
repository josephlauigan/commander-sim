//! Practice mode: the activated abilities of the Cloud deck's cards that were only the AI's card code (an option the
//! AI takes when its own conditions say so), or not modeled at all (Giver of Runes, Plaza of Heroes). The person
//! decides for themselves, so each is listed whenever the rules allow the activation, its cost is paid from the pool,
//! and the choices (targets, a colour, a card from the hand) are asked of the person (JD, 2026-11-04).
//! Each is for the person's seat only; the AI's hooks for these cards are unchanged.

use super::abilities::{LandAbility, PermAbility, Why, ability};
use super::{cards, choices, mana};
use crate::engine::stack::{ability_window, ability_window_card};
use crate::engine::zones::{self, Enter};
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;
use crate::sym::intern;

const PERMS: [&str; 5] =
    ["Balan, Wandering Knight", "Cid, Freeflier Pilot", "Giver of Runes", "Stoneforge Mystic", "Zack Fair"];

/// A permanent whose abilities are all listed here (the AI's hook options for it are left out, so nothing shows twice).
pub fn owns_perm(g: &Game, m: PermId) -> bool {
    PERMS.contains(&&*g.perm(m).name)
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn need(g: &Game, p: PlayerId, gn: u32, pips: &str, what: &str) -> Why {
    mana::cost_problem(g, p, gn, pips).map(|why| format!("Can't activate {what}. {why}"))
}

/// a {T} ability of a creature: tapped, or still summoning sick
fn tap_ok(g: &Game, m: PermId) -> Why {
    let x = g.perm(m);
    if x.tapped {
        return Some(format!("{} is tapped.", x.name));
    }
    if x.sick {
        return Some(format!("{} has summoning sickness (it came under your control this turn).", x.name));
    }
    None
}

pub fn perm_abilities(g: &Game, _p: PlayerId, m: PermId) -> Vec<PermAbility> {
    let x = g.perm(m);
    if !x.on_bf || x.phased || x.cd.is_none() {
        return vec![];
    }
    match &*x.name {
        "Balan, Wandering Knight" => vec![ability("{1}{W}: attach all Equipment you control to Balan", balan)],
        "Cid, Freeflier Pilot" => {
            vec![ability("{2}, {T}: return target Equipment or Vehicle card from your graveyard to your hand", cid)]
        }
        "Giver of Runes" => vec![ability(
            "{T}: another target creature you control gains protection from colorless or from a color of your choice until end of turn",
            giver,
        )],
        "Stoneforge Mystic" => {
            vec![ability("{1}{W}, {T}: you may put an Equipment card from your hand onto the battlefield", stoneforge)]
        }
        "Zack Fair" => vec![ability(
            "{1}, sacrifice: target creature you control gains indestructible; Zack's counters and Equipment move to it",
            zack,
        )],
        _ => vec![],
    }
}

pub fn land_abilities(g: &Game, l: PermId) -> Vec<LandAbility> {
    match &*g.db.get(g.perm(l).card()).name {
        "Plaza of Heroes" if !crate::engine::mana::moon_land(g, l) => vec![ability(
            "{3}, {T}, exile it: target legendary creature gains hexproof and indestructible until end of turn",
            plaza,
        )],
        _ => vec![],
    }
}

/// Balan: {1}{W}: attach all Equipment you control to Balan (any time; no {T})
fn balan(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = need(g, p, 1, "W", "Balan") {
        return Ok(Some(why));
    }
    mana::pay_from_pool(g, p, 1, "W");
    crate::glog!(g, "  {} activates Balan", g.player(p).name);
    if ability_window(g, p, Some(m), "attach all Equipment", Some(5.0), Some(m))? && live(g, m) {
        for e in crate::impls::cloud::my_equipment(g, p) {
            if g.perm(e).attached != Some(m) {
                crate::impls::cloud::attach(g, e, m);
            }
        }
    }
    Ok(None)
}

/// Cid: {2}, {T}: return target Equipment or Vehicle card from your graveyard to your hand
fn cid(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m).or_else(|| need(g, p, 2, "", "Cid")) {
        return Ok(Some(why));
    }
    let cs: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| g.db.get(c).has_subtype("equipment") || g.db.get(c).has_subtype("vehicle"))
        .collect();
    if cs.is_empty() {
        return Ok(Some("There is no Equipment or Vehicle card in your graveyard.".into()));
    }
    let labels: Vec<String> = cs.iter().map(|&c| choices::card_label(g, c)).collect();
    let Some(k) =
        choices::choose(g, p, "target", "Cid: return which Equipment or Vehicle card?", &labels, Some("cancel"), None)?
    else {
        return Ok(None); // targets first, then the costs
    };
    let c = cs[k];
    mana::pay_from_pool(g, p, 2, "");
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "  {} activates Cid", g.player(p).name);
    if ability_window(g, p, Some(m), &format!("return {} to hand", g.db.get(c).name), None, None)?
        && let Some(i) = g.player(p).gy.iter().position(|&x| x == c)
    {
        g.player_mut(p).gy.remove(i);
        g.player_mut(p).hand.push(c);
        g.player_mut(p).seen.insert(c);
        crate::glog!(g, "    Cid returns {}", g.db.get(c).name);
    }
    Ok(None)
}

/// Giver of Runes: {T}: another target creature you control gains protection from colorless or from the color of
/// your choice until end of turn
fn giver(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m) {
        return Ok(Some(why));
    }
    let tg: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&x| x != m && live(g, x) && g.is_creature(x)).collect();
    if tg.is_empty() {
        return Ok(Some("You control no other creature to target.".into()));
    }
    let Some(t) = cards::pick_perm(
        g,
        p,
        "target",
        "Giver of Runes: which other creature gains protection?",
        &tg,
        Some("cancel"),
    )?
    else {
        return Ok(None);
    };
    let opts = [
        ("colorless", "pro_colorless"),
        ("white", "pro_W"),
        ("blue", "pro_U"),
        ("black", "pro_B"),
        ("red", "pro_R"),
        ("green", "pro_G"),
    ];
    let labels: Vec<String> = opts.iter().map(|o| format!("protection from {}", o.0)).collect();
    let Some(k) =
        choices::choose(g, p, "choose", "Giver of Runes: protection from what?", &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "  {} activates Giver of Runes", g.player(p).name);
    if ability_window(g, p, Some(m), &format!("{} gains protection from {}", g.perm(t).name, opts[k].0), None, Some(t))?
        && live(g, t)
    {
        crate::impls::partials::eot_kw(g, t, intern(opts[k].1));
        crate::glog!(g, "    {} gains protection from {} until end of turn", g.perm(t).name, opts[k].0);
    }
    Ok(None)
}

/// Stoneforge Mystic: {1}{W}, {T}: you may put an Equipment card from your hand onto the battlefield
fn stoneforge(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m).or_else(|| need(g, p, 1, "W", "Stoneforge Mystic")) {
        return Ok(Some(why));
    }
    let eq: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).has_subtype("equipment")).collect();
    if eq.is_empty() {
        return Ok(Some("You have no Equipment card in your hand.".into()));
    }
    mana::pay_from_pool(g, p, 1, "W");
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "  {} activates Stoneforge Mystic", g.player(p).name);
    if !ability_window(g, p, Some(m), "put an Equipment card onto the battlefield", Some(5.0), None)? {
        return Ok(None);
    }
    // chosen as it resolves, from what is in the hand then
    let eq: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).has_subtype("equipment")).collect();
    if eq.is_empty() {
        return Ok(None);
    }
    let labels: Vec<String> = eq.iter().map(|&c| choices::card_label(g, c)).collect();
    let Some(k) = choices::choose(
        g,
        p,
        "choose",
        "Stoneforge Mystic: put which Equipment card onto the battlefield?",
        &labels,
        Some("none"),
        None,
    )?
    else {
        return Ok(None);
    };
    let c = eq[k];
    crate::impls::partials::remove_card(&mut g.player_mut(p).hand, c);
    zones::enter(g, p, c, Enter::default())?;
    crate::glog!(g, "  {} puts {} onto the battlefield with Stoneforge Mystic", g.player(p).name, g.db.get(c).name);
    Ok(None)
}

/// Zack Fair: {1}, sacrifice: target creature you control gains indestructible until end of turn; put Zack's counters
/// on that creature and attach an Equipment that was attached to Zack to that creature
fn zack(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = need(g, p, 1, "", "Zack Fair") {
        return Ok(Some(why));
    }
    let tg: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&x| x != m && live(g, x) && g.is_creature(x)).collect();
    if tg.is_empty() {
        return Ok(Some("You control no other creature to target.".into()));
    }
    let Some(t) =
        cards::pick_perm(g, p, "target", "Zack Fair: which creature gains indestructible?", &tg, Some("cancel"))?
    else {
        return Ok(None);
    };
    crate::glog!(g, "  {} sacrifices Zack Fair", g.player(p).name);
    if !crate::impls::cloud::zack_sacrifice(g, m, p, t.0 as i64)? {
        return Ok(Some("Can't activate Zack Fair.".into()));
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

/// Plaza of Heroes: {3}, {T}, exile it: target legendary creature gains hexproof and indestructible until end of turn
fn plaza(g: &mut Game, p: PlayerId, l: PermId) -> Res<Why> {
    if g.perm(l).tapped {
        return Ok(Some("Plaza of Heroes is tapped.".into()));
    }
    if let Some(why) = need(g, p, 3, "", "Plaza of Heroes") {
        return Ok(Some(why));
    }
    let tg: Vec<PermId> =
        cards::creatures(g, p, false).into_iter().filter(|&x| crate::cardcode::is_legendary(g, x)).collect();
    if tg.is_empty() {
        return Ok(Some("There is no legendary creature to target.".into()));
    }
    let Some(t) = cards::pick_perm(g, p, "target", "Plaza of Heroes: which legendary creature?", &tg, Some("cancel"))?
    else {
        return Ok(None);
    };
    mana::pay_from_pool(g, p, 3, "");
    g.perm_mut(l).tapped = true;
    let cd = g.perm(l).real_card();
    crate::engine::turn::remove_land(g, p, l);
    g.player_mut(p).exile.push(cd);
    crate::glog!(g, "  {} exiles Plaza of Heroes", g.player(p).name);
    if ability_window_card(g, p, cd, &format!("{} gains hexproof and indestructible", g.perm(t).name), None, Some(t))?
        && live(g, t)
    {
        crate::impls::partials::eot_kw(g, t, intern("hexproof"));
        crate::impls::partials::eot_kw(g, t, intern("indestructible"));
        crate::glog!(g, "    {} gains hexproof and indestructible until end of turn", g.perm(t).name);
    }
    Ok(None)
}

// ------------------------------------------------------------------ spells the person resolves
/// The person's Akroma's Will, Winds of Abandon and Galadriel's Dismissal as they resolve, by what they chose as they
/// cast them (`specs`: `ctx.mode`, `ctx.target`, `ctx.face`). True if handled here (the caller puts the card away).
pub fn resolve_spell(g: &mut Game, p: PlayerId, c: CardId, ctx: &crate::state::Ctx) -> Res<bool> {
    match &*g.db.get(c).name {
        "Akroma's Will" => will(g, p, ctx),
        "Winds of Abandon" => winds(g, p, c, ctx)?,
        "Galadriel's Dismissal" => dismissal(g, p, ctx),
        _ => return Ok(false),
    }
    Ok(true)
}

/// Akroma's Will: flying, vigilance and double strike (mode one); lifelink, indestructible and protection from each
/// color (mode two); both with a commander
fn will(g: &mut Game, p: PlayerId, ctx: &crate::state::Ctx) {
    let is = |k: &str| ctx.mode.is_some_and(|m| m == k);
    let (one, two) = if is("protect") {
        (false, true)
    } else if is("both") {
        (true, true)
    } else {
        (true, false)
    };
    for m in g.player(p).perms.clone() {
        if !live(g, m) || !g.is_creature(m) {
            continue;
        }
        if one {
            for k in ["flying", "vigilance", "double strike"] {
                crate::impls::partials::eot_kw(g, m, intern(k));
            }
        }
        if two {
            for k in ["lifelink", "indestructible", "pro_colors"] {
                crate::impls::partials::eot_kw(g, m, intern(k));
            }
        }
    }
    crate::glog!(g, "    {}'s creatures gain the mode(s) of Akroma's Will", g.player(p).name);
}

/// Winds of Abandon: exile target creature you don't control (or each, overloaded); for each creature exiled this way,
/// its controller searches for a basic land card and puts it onto the battlefield tapped
fn winds(g: &mut Game, p: PlayerId, c: CardId, ctx: &crate::state::Ctx) -> Res {
    if ctx.mode.is_some_and(|m| m == "overload") {
        for q in g.opps(p).collect::<Vec<_>>() {
            let mut n = 0;
            for m in g.player(q).perms.clone() {
                if g.perm(m).on_bf && g.is_creature(m) && !g.perm(m).phased {
                    zones::exile_perm(g, m)?;
                    n += (!g.perm(m).on_bf) as u32;
                }
            }
            if n > 0 {
                zones::land_ramp(g, q, n, true)?;
            }
        }
        return Ok(());
    }
    if let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) {
        let q = g.perm(t).owner;
        crate::engine::removal::apply_removal(g, Some(p), t, "exile", Some(c))?;
        if !g.perm(t).on_bf {
            zones::land_ramp(g, q, 1, true)?;
        }
    }
    Ok(())
}

/// phase out m and what is attached to it
fn phase_out(g: &mut Game, m: PermId) {
    g.perm_mut(m).phased = true;
    let all: Vec<PermId> = g.players.iter().flat_map(|q| q.perms.clone()).collect();
    for x in all {
        if g.perm(x).attached == Some(m) {
            g.perm_mut(x).phased = true;
        }
    }
}

/// Galadriel's Dismissal: target creature phases out; kicked, each creature target player controls phases out instead
fn dismissal(g: &mut Game, _p: PlayerId, ctx: &crate::state::Ctx) {
    if ctx.mode.is_some_and(|m| m == "kicked") {
        let Some(q) = ctx.face else { return };
        for m in g.player(q).perms.clone() {
            if live(g, m) && g.is_creature(m) {
                phase_out(g, m);
            }
        }
        crate::glog!(g, "    each creature {} controls phases out", g.player(q).name);
    } else if let Some(t) = ctx.target.filter(|&t| live(g, t)) {
        phase_out(g, t);
        crate::glog!(g, "    {} phases out", g.perm(t).name);
    }
}

// ------------------------------------------------------------------ enter triggers the person answers
/// Kor Outfitter: when it enters, you may attach target Equipment you control to target creature you control
pub fn outfitter(g: &mut Game, src: PermId) -> Res {
    let o = g.perm(src).owner;
    let eq = crate::impls::cloud::my_equipment(g, o);
    let cre: Vec<PermId> = g.player(o).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect();
    if eq.is_empty() || cre.is_empty() {
        return Ok(());
    }
    let Some(e) = cards::pick_perm(
        g,
        o,
        "target",
        "Kor Outfitter: attach which Equipment you control?",
        &eq,
        Some("don't attach"),
    )?
    else {
        return Ok(());
    };
    let Some(t) = cards::pick_perm(
        g,
        o,
        "target",
        "Kor Outfitter: attach it to which creature you control?",
        &cre,
        Some("don't attach"),
    )?
    else {
        return Ok(());
    };
    if crate::engine::stack::may_window(g, o, Some(src), "attach Equipment", Some(5.0))? && live(g, e) && live(g, t) {
        crate::impls::cloud::attach(g, e, t);
    }
    Ok(())
}

/// Forge Anew: when it enters, return target Equipment card from your graveyard to the battlefield
pub fn forge_anew(g: &mut Game, src: PermId) -> Res {
    let o = g.perm(src).owner;
    if crate::impls::t45_winota::jailed(g) {
        return Ok(()); // Soulless Jailer: permanent cards in graveyards can't enter
    }
    let eq: Vec<CardId> = g.player(o).gy.iter().copied().filter(|&c| g.db.get(c).has_subtype("equipment")).collect();
    if eq.is_empty() {
        return Ok(());
    }
    // the question comes after the trigger's window: the engine runs a trigger once up to its first window as a
    // probe (`stack::resolve_batch`), so asking before it asked twice
    if !crate::engine::stack::trigger_window(g, o, Some(src), "return an Equipment from the graveyard", Some(6.0))? {
        return Ok(());
    }
    let eq: Vec<CardId> = g.player(o).gy.iter().copied().filter(|&c| g.db.get(c).has_subtype("equipment")).collect();
    if eq.is_empty() {
        return Ok(());
    }
    let labels: Vec<String> = eq.iter().map(|&c| choices::card_label(g, c)).collect();
    let k =
        choices::choose_one(g, o, "target", "Forge Anew: return which Equipment card to the battlefield?", &labels)?;
    let c = eq[k];
    if crate::impls::partials::remove_card(&mut g.player_mut(o).gy, c) {
        zones::enter(g, o, c, Enter::default())?;
        crate::glog!(g, "    {} returns to the battlefield", g.db.get(c).name);
    }
    Ok(())
}
