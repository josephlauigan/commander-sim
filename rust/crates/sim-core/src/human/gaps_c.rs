//! Practice-mode gaps (2026-11-04, Agent C): activated abilities of the Alela, Sheoldred, Thrasios and Y'shtola decks'
//! cards that the AI uses through card-code option hooks (and only when its own value conditions say so), listed here
//! for a person whenever the rules allow them. Each is an `abilities::ability` entry: it checks timing, tapping and
//! mana from the person's pool, asks the person for the targets and the costs' choices, then carries the effect out
//! (reusing the AI play's `*_go` where nothing is chosen by the AI).
//!
//! `cards::more_abilities` falls through to `abilities` for the names listed here.

use super::abilities::{PermAbility, Why, ability};
use super::{Target, cards, choices, legal, mana};
use crate::cards::Types;
use crate::engine::stack::ability_window;
use crate::engine::{self, values, zones};
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Step, Val};

fn pname(g: &Game, p: PlayerId) -> String {
    g.player(p).name.to_string()
}

fn cname(g: &Game, c: CardId) -> String {
    g.db.get(c).name.to_string()
}

fn alive(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

/// why a {T} ability of m can't be used now (tapped; a creature also summoning sick)
fn tap_ok(g: &Game, m: PermId, creature: bool) -> Why {
    let x = g.perm(m);
    if x.tapped {
        return Some(format!("{} is tapped.", x.name));
    }
    if creature && x.sick {
        return Some(format!("{} has summoning sickness (it came under your control this turn).", x.name));
    }
    None
}

/// why the mana of an ability can't be paid from the pool now
fn cost_ok(g: &Game, p: PlayerId, gn: u32, pips: &str, what: &str) -> Why {
    mana::cost_problem(g, p, gn, pips).map(|why| format!("Can't activate {what}. {why}"))
}

/// the abilities of p's permanent m (card name `nm`), or None when this file doesn't list the card
pub fn abilities(_g: &Game, _p: PlayerId, _m: PermId, nm: &str) -> Option<Vec<PermAbility>> {
    Some(match nm {
        "Angelic Page" => {
            vec![ability("{T}: target attacking or blocking creature gets +1/+1 until end of turn", angelic_page)]
        }
        "Cabal Patriarch" => vec![
            ability("{2}{B}, sacrifice a creature: target creature gets -2/-2 until end of turn", |g, p, m| {
                patriarch(g, p, m, true)
            }),
            ability(
                "{2}{B}, exile a creature card from your graveyard: target creature gets -2/-2 until end of turn",
                |g, p, m| patriarch(g, p, m, false),
            ),
        ],
        "Credit Voucher" => vec![ability(
            "{2}, {T}, sacrifice it: shuffle any number of cards from your hand into your library, then draw that many",
            voucher,
        )],
        "Crystal Chimes" => vec![ability(
            "{3}, {T}, sacrifice it: return all enchantment cards from your graveyard to your hand",
            chimes,
        )],
        "Emry, Lurker of the Loch" => {
            vec![ability("{T}: choose an artifact card in your graveyard; you may cast it this turn", emry)]
        }
        "Mirage Mirror" => vec![ability(
            "{2}: becomes a copy of target artifact, creature or enchantment until end of turn",
            mirage_mirror,
        )],
        "Wispdrinker Vampire" => vec![ability(
            "{5}{W}{B}: creatures you control with power 2 or less gain deathtouch and lifelink until end of turn",
            wispdrinker,
        )],
        "Gix, Yawgmoth Praetor" => vec![ability(
            "{4}{B}{B}{B}, discard X cards: exile the top X cards of an opponent's library; you may play lands and cast spells from among them for free",
            gix,
        )],
        "Harnfel, Horn of Bounty" => vec![ability(
            "Discard a card: exile the top two cards of your library; you may play them this turn",
            harnfel,
        )],
        "Dauthi Voidwalker" => vec![ability(
            "{T}, sacrifice it: play an exiled card with a void counter without paying its mana cost",
            dauthi,
        )],
        "Resplendent Angel" => {
            vec![ability("{3}{W}{W}{W}: +2/+2 and lifelink until end of turn", resplendent)]
        }
        "Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal" => vec![ability(
            "Hero's Sundering, {7}, {T}: exile target nonland permanent, then transform Venat (sorcery speed)",
            sundering,
        )],
        _ => return None,
    })
}

// ------------------------------------------------------------------ Angelic Page
/// {T}: target attacking or blocking creature gets +1/+1 until end of turn
fn angelic_page(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, true) {
        return Ok(Some(why));
    }
    let white = crate::cards::Colors::from_letters("W");
    let mut fighting: Vec<PermId> = g.in_combat.clone();
    for &x in &g.blocking {
        if !fighting.contains(&x) {
            fighting.push(x);
        }
    }
    let cre: Vec<PermId> = fighting
        .into_iter()
        .filter(|&x| {
            g.perm(x).on_bf
                && g.player(g.perm(x).owner).perms.contains(&x)
                && !(g.perm(x).owner != p && values::untargetable(g, x))
                && !values::protected_from(g, x, white)
        })
        .collect();
    if cre.is_empty() {
        return Ok(Some("Angelic Page needs an attacking or blocking creature to target.".into()));
    }
    let Some(t) = cards::pick_perm(
        g,
        p,
        "target",
        "Angelic Page: which attacking or blocking creature gets +1/+1?",
        &cre,
        Some("cancel"),
    )?
    else {
        return Ok(None);
    };
    g.perm_mut(m).tapped = true;
    let tn = g.perm(t).name;
    crate::glog!(g, "  {} activates Angelic Page: {} gets +1/+1", pname(g, p), tn);
    if ability_window(g, p, Some(m), &format!("+1/+1 on {tn}"), None, Some(t))? && alive(g, t) {
        let y = g.perm_mut(t);
        y.eot_pt = (y.eot_pt.0 + 1, y.eot_pt.1 + 1);
    }
    Ok(None)
}

// ------------------------------------------------------------------ Cabal Patriarch
/// -n/-n until end of turn on t (a lethal one goes through the removal code)
fn shrink(g: &mut Game, p: PlayerId, t: PermId, n: i32) -> Res {
    if !g.perm(t).on_bf {
        return Ok(());
    }
    if values::etgh(g, t) <= n {
        return engine::removal::apply_removal(g, Some(p), t, &format!("shrink{n}"), None);
    }
    let x = g.perm_mut(t);
    x.eot_pt = (x.eot_pt.0 - n, x.eot_pt.1 - n);
    Ok(())
}

/// {2}{B}, sacrifice a creature (`sac`) or exile a creature card from your graveyard: target creature gets -2/-2
fn patriarch(g: &mut Game, p: PlayerId, m: PermId, sac: bool) -> Res<Why> {
    if let Some(why) = cost_ok(g, p, 2, "B", "Cabal Patriarch") {
        return Ok(Some(why));
    }
    let bodies: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&x| g.is_creature(x) && !g.perm(x).phased && !values::no_sac(g, x))
        .collect();
    let yard: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    if sac && bodies.is_empty() {
        return Ok(Some("You have no creature to sacrifice.".into()));
    }
    if !sac && yard.is_empty() {
        return Ok(Some("There is no creature card in your graveyard to exile.".into()));
    }
    let black = crate::cards::Colors::from_letters("B");
    let tg: Vec<PermId> = cards::creatures(g, p, false)
        .into_iter()
        .filter(|&x| !(g.perm(x).owner != p && values::protected_from(g, x, black)))
        .collect();
    if tg.is_empty() {
        return Ok(Some("There is no creature to target.".into()));
    }
    let Some(t) = cards::pick_perm(g, p, "target", "Cabal Patriarch: -2/-2 on which creature?", &tg, Some("cancel"))?
    else {
        return Ok(None);
    };
    enum Cost {
        Sac(PermId),
        Exile(CardId),
    }
    let cost = if sac {
        let Some(x) =
            cards::pick_perm(g, p, "choose", "Cabal Patriarch: sacrifice which creature?", &bodies, Some("cancel"))?
        else {
            return Ok(None);
        };
        Cost::Sac(x)
    } else {
        let labels: Vec<String> = yard.iter().map(|&c| choices::card_label(g, c)).collect();
        let Some(k) = choices::choose(
            g,
            p,
            "choose",
            "Cabal Patriarch: exile which creature card from your graveyard?",
            &labels,
            Some("cancel"),
            None,
        )?
        else {
            return Ok(None);
        };
        Cost::Exile(yard[k])
    };
    mana::pay_from_pool(g, p, 2, "B");
    match cost {
        Cost::Sac(x) => zones::die(g, x, "sac")?,
        Cost::Exile(c) => {
            let pl = g.player_mut(p);
            if let Some(i) = pl.gy.iter().position(|&y| y == c) {
                pl.gy.remove(i);
                pl.exile.push(c);
            }
        }
    }
    let tn = g.perm(t).name;
    crate::glog!(g, "  {} activates Cabal Patriarch: -2/-2 on {}", pname(g, p), tn);
    if ability_window(g, p, Some(m), &format!("-2/-2 on {tn}"), None, Some(t))? {
        shrink(g, p, t, 2)?;
    }
    Ok(None)
}

// ------------------------------------------------------------------ Credit Voucher
/// {2}, {T}, sacrifice: shuffle any number of cards from your hand into your library, then draw that many
fn voucher(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false).or_else(|| cost_ok(g, p, 2, "", "Credit Voucher")) {
        return Ok(Some(why));
    }
    let mut left: Vec<CardId> = g.player(p).hand.clone();
    if left.is_empty() {
        return Ok(Some("You have no card in hand to shuffle away.".into()));
    }
    let mut picked: Vec<CardId> = vec![];
    while !left.is_empty() {
        let labels: Vec<String> = left.iter().map(|&c| choices::card_label(g, c)).collect();
        let done = if picked.is_empty() { "cancel" } else { "done" };
        let prompt = format!("Credit Voucher: shuffle which card into your library? ({} chosen so far)", picked.len());
        let Some(k) = choices::choose(g, p, "choose", &prompt, &labels, Some(done), None)? else {
            if picked.is_empty() {
                return Ok(None);
            }
            break;
        };
        picked.push(left.remove(k));
    }
    mana::pay_from_pool(g, p, 2, "");
    g.perm_mut(m).tapped = true;
    zones::die(g, m, "sac")?; // the sacrifice is a cost
    crate::glog!(g, "  {} activates Credit Voucher", pname(g, p));
    if !ability_window(g, p, Some(m), "shuffle cards from hand into the library, draw that many", None, None)? {
        return Ok(None);
    }
    let picked: Vec<CardId> = picked.into_iter().filter(|c| g.player(p).hand.contains(c)).collect();
    for &c in &picked {
        let pl = g.player_mut(p);
        if let Some(i) = pl.hand.iter().position(|&x| x == c) {
            pl.hand.remove(i);
        }
        pl.library.push(c);
    }
    engine::tutors::shuffle_library(g, p);
    crate::glog!(g, "    shuffles {} card(s) into the library and draws {}", picked.len(), picked.len());
    zones::draw(g, p, picked.len() as u32, false)?;
    Ok(None)
}

// ------------------------------------------------------------------ Crystal Chimes
/// {3}, {T}, sacrifice: return all enchantment cards from your graveyard to your hand
fn chimes(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false).or_else(|| cost_ok(g, p, 3, "", "Crystal Chimes")) {
        return Ok(Some(why));
    }
    crate::impls::alela::chimes_go(g, m, p, 0)?;
    Ok(None)
}

// ------------------------------------------------------------------ Emry, Lurker of the Loch
/// {T}: choose target artifact card in your graveyard; you may cast it this turn. The card is cast at once (the mana
/// for it must be in the pool), as the AI's play does
fn emry(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, true) {
        return Ok(Some(why));
    }
    let arts: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| {
            g.db.get(c).types.has(Types::ARTIFACT) && !g.db.get(c).land && engine::cast::castable(g, p, c, "gy")
        })
        .collect();
    if arts.is_empty() {
        return Ok(Some("There is no artifact card in your graveyard that you could cast.".into()));
    }
    let labels: Vec<String> = arts
        .iter()
        .map(|&c| {
            let (gn, pips) = engine::mana::cost_of(g, p, c);
            format!("{} (costs {})", choices::card_label(g, c), mana::cost_text(gn, &pips))
        })
        .collect();
    let Some(k) = choices::choose(
        g,
        p,
        "target",
        "Emry: choose target artifact card in your graveyard (you may cast it this turn)",
        &labels,
        Some("cancel"),
        None,
    )?
    else {
        return Ok(None);
    };
    let c = arts[k];
    // "Choose target artifact card in your graveyard. You may cast that card this turn.": the permission lasts the
    // turn; the person casts it from the graveyard (paying its cost) whenever they like, or not at all
    g.perm_mut(m).tapped = true;
    let what = format!("you may cast {} from your graveyard this turn", cname(g, c));
    if !engine::stack::ability_window(g, p, Some(m), &what, None, None)? || !g.player(p).gy.contains(&c) {
        return Ok(None); // countered, or the card left the graveyard in response
    }
    let st = g.turn_stamp();
    let mut list = match g.perm(m).data.get(DataKey::Emry) {
        Some(Val::List(v)) if v.first() == Some(&Val::Stamp(st)) => v.clone(),
        _ => vec![Val::Stamp(st)],
    };
    list.push(Val::Card(c));
    g.perm_mut(m).data.set(DataKey::Emry, Val::List(list));
    crate::glog!(g, "    {}: {what}", g.perm(m).name);
    Ok(None)
}

/// a person's Emry, Lurker of the Loch let p cast card c from the graveyard this turn
pub fn emry_permits(g: &Game, p: PlayerId, c: CardId) -> bool {
    // every Emry p has had (the permanents' arena keeps one that left the battlefield: the permission outlives it)
    let st = g.turn_stamp();
    g.perms.iter().any(|x| {
        x.owner == p
            && matches!(x.data.get(DataKey::Emry),
                Some(Val::List(v)) if v.first() == Some(&Val::Stamp(st)) && v.contains(&Val::Card(c)))
    })
}

// ------------------------------------------------------------------ Harnfel, Horn of Bounty
/// Discard a card: exile the top two cards of your library; you may play those cards this turn. The exiled cards are
/// held in the hand for the turn (`Player::impulse`: the end step sends the unplayed ones back to exile).
fn harnfel(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    let hand = g.player(p).hand.clone();
    if hand.is_empty() {
        return Ok(Some("You have no card to discard.".into()));
    }
    let Some(&d) = choices::pick_cards(g, p, &hand, 1, "Harnfel: discard which card?", "choose")?.first() else {
        return Ok(None);
    };
    zones::discard_cards(g, p, &[d])?;
    crate::glog!(g, "  {} activates Harnfel: discards {}", pname(g, p), cname(g, d));
    if !ability_window(g, p, Some(m), "exile the top two cards of your library; you may play them this turn", None, None)? {
        return Ok(None);
    }
    for _ in 0..2 {
        let Some(c) = g.player_mut(p).library.pop() else { break };
        let pl = g.player_mut(p);
        pl.hand.push(c);
        pl.impulse.push(c);
        pl.seen.insert(c);
        crate::glog!(g, "    exiles {} (Harnfel): you may play it this turn", cname(g, c));
    }
    Ok(None)
}

// ------------------------------------------------------------------ Mirage Mirror
/// {2}: becomes a copy of target artifact, creature, enchantment or land until end of turn. Not offered: Auras (a copy
/// of an Aura would be attached to nothing and go to the graveyard) and Mirage Mirrors that are already copies. Any land
/// can be copied, abilities and all (a fetch land can be cracked, a bounceland taps for its two mana, Command Beacon or
/// Strip Mine can be sacrificed ...): the Mirror moves to its controller's land list until `alela::mirror_revert`, and
/// the card that leaves the battlefield is the Mirror's own (`Perm::real_card`), never the copied land's card. Your
/// own legendary land can be copied too: the legend rule (`life::sba_legend`) then has you keep one of the two.
fn mirage_mirror(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = cost_ok(g, p, 2, "", "Mirage Mirror") {
        return Ok(Some(why));
    }
    let mut tg: Vec<PermId> = vec![];
    for q in g.players.iter().filter(|q| q.alive) {
        for &t in q.perms.iter().chain(q.lands.iter()) {
            let x = g.perm(t);
            let Some(cd) = x.cd else { continue };
            // a token copy of a card counts (its copiable values are the card's); your own legend too (the legend
            // rule then has you keep one of the two). Not offered: an Aura (the copy would be attached to nothing)
            if t == m || x.phased || (x.owner != p && values::untargetable(g, t)) {
                continue;
            }
            let d = g.db.get(cd);
            if x.data.truthy(DataKey::MirageMirror) || d.has_subtype("aura") {
                continue;
            }
            if d.land {
                // any land (its abilities work as the copy; see the doc above)
            } else if !(g.is_creature(t) || d.types.has(Types::ARTIFACT) || d.types.has(Types::ENCHANTMENT)) {
                continue;
            }
            tg.push(t);
        }
    }
    if tg.is_empty() {
        return Ok(Some("There is no artifact, creature, enchantment or land to copy.".into()));
    }
    let Some(t) =
        cards::pick_perm(g, p, "target", "Mirage Mirror: become a copy of which permanent?", &tg, Some("cancel"))?
    else {
        return Ok(None);
    };
    mana::pay_from_pool(g, p, 2, "");
    let tn = g.perm(t).name;
    crate::glog!(g, "  {} activates Mirage Mirror: a copy of {}", pname(g, p), tn);
    if !ability_window(g, p, Some(m), &format!("becomes a copy of {tn}"), None, Some(t))? {
        return Ok(None);
    }
    let Some(tcd) = g.perm(t).cd.filter(|_| g.perm(t).on_bf && g.perm(m).on_bf) else { return Ok(None) };
    // already a copy this turn (a second activation): the Mirror's own card is the one it was before
    let own = if g.perm(m).data.truthy(DataKey::MirageMirror) { g.perm(m).phys } else { g.perm(m).cd };
    crate::impls::alela::set_face(g, m, tcd);
    let x = g.perm_mut(m);
    x.phys = own;
    x.data.set(DataKey::MirageMirror, Val::Bool(true));
    crate::impls::alela::mirror_lists(g, m, g.db.get(tcd).land);
    crate::engine::life::check_state(g)?; // a copy of your own legend: the legend rule (you keep one)
    Ok(None)
}

// ------------------------------------------------------------------ Wispdrinker Vampire
/// {5}{W}{B}: creatures you control with power 2 or less gain deathtouch and lifelink until end of turn
fn wispdrinker(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = cost_ok(g, p, 5, "WB", "Wispdrinker Vampire") {
        return Ok(Some(why));
    }
    mana::pay_from_pool(g, p, 5, "WB");
    crate::glog!(g, "  {} activates Wispdrinker Vampire", pname(g, p));
    if !ability_window(g, p, Some(m), "small creatures gain deathtouch and lifelink", None, None)? {
        return Ok(None);
    }
    let mut n = 0;
    for x in g.player(p).perms.clone() {
        if g.is_creature(x) && !g.perm(x).phased && values::epow(g, x) <= 2 {
            crate::impls::partials::eot_kw(g, x, "deathtouch");
            crate::impls::partials::eot_kw(g, x, "lifelink");
            n += 1;
        }
    }
    crate::glog!(g, "    {n} creature(s) gain deathtouch and lifelink until end of turn");
    Ok(None)
}

// ------------------------------------------------------------------ Gix, Yawgmoth Praetor
/// {4}{B}{B}{B}, discard X cards: exile the top X cards of target opponent's library. You may play lands and cast
/// spells from among the cards exiled this way without paying their mana costs: asked one card at a time as the
/// ability resolves (the cards you don't play stay exiled)
fn gix(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = cost_ok(g, p, 4, "BBB", "Gix") {
        return Ok(Some(why));
    }
    let opps: Vec<PlayerId> = g.opps(p).filter(|&q| g.player(q).alive && legal::player_targetable(g, q)).collect();
    if opps.is_empty() {
        return Ok(Some("There is no opponent to target.".into()));
    }
    let q = if opps.len() == 1 {
        opps[0]
    } else {
        let labels: Vec<String> = opps.iter().map(|&o| legal::describe_target(g, p, Target::Player(o))).collect();
        let Some(k) = choices::choose(
            g,
            p,
            "target",
            "Gix: exile cards from which opponent's library?",
            &labels,
            Some("cancel"),
            None,
        )?
        else {
            return Ok(None);
        };
        opps[k]
    };
    let hand = g.player(p).hand.clone();
    let xs: Vec<usize> = (0..=hand.len()).collect();
    let labels: Vec<String> =
        xs.iter().map(|x| format!("X = {x} (discard {x} card{})", if *x == 1 { "" } else { "s" })).collect();
    let Some(k) = choices::choose(g, p, "choose", "Gix: X = ?", &labels, Some("cancel"), None)? else {
        return Ok(None);
    };
    let x = xs[k];
    let discards = choices::pick_cards(g, p, &hand, x, "Gix: discard which card?", "choose")?;
    mana::pay_from_pool(g, p, 4, "BBB");
    zones::discard_cards(g, p, &discards)?;
    crate::glog!(g, "  {} activates Gix: discards {} card(s)", pname(g, p), discards.len());
    let label = format!("exile the top {x} of {}'s library", pname(g, q));
    if !ability_window(g, p, Some(m), &label, Some(6.0), None)? {
        return Ok(None);
    }
    let mut top: Vec<CardId> = vec![];
    for _ in 0..x.min(g.player(q).library.len()) {
        top.push(g.player_mut(q).library.pop().unwrap());
    }
    crate::glog!(g, "    exiles {} card(s) of {}", top.len(), pname(g, q));
    for c in top {
        if g.over || !g.player(p).alive {
            break;
        }
        let d = g.db.get(c);
        let my_main = g.active == Some(p) && matches!(g.step, Step::Main1 | Step::Main2);
        let land_ok = d.land && my_main && (g.player(p).lands_played as i32) < legal::land_drops(g, p);
        let spell_ok = !d.land && engine::cast::castable(g, p, c, "hand");
        let play = (land_ok || spell_ok)
            && choices::yes_no(
                g,
                p,
                &format!("Gix: {} {} for free?", if land_ok { "play" } else { "cast" }, choices::card_label(g, c)),
            )?;
        if play && land_ok {
            engine::turn::play_land_card(g, p, c, "plays (Gix)")?;
        } else if play {
            engine::cast::cast_card(g, p, c, "lib".into(), crate::state::Ctx { owner: Some(q), ..Default::default() })?;
        } else {
            g.player_mut(q).exile.push(c);
        }
    }
    Ok(None)
}

// ------------------------------------------------------------------ Dauthi Voidwalker
/// {T}, sacrifice it: choose an exiled card an opponent owns with a void counter; you may play it this turn without
/// paying its mana cost (a land is played at once as the land drop; a spell is a permission for the turn)
fn dauthi(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, true) {
        return Ok(Some(why));
    }
    let mut void = crate::impls::common::dauthi_void(g, m);
    // "you may play it": an exiled land too, as your land drop (your main phase, the stack empty, a drop left)
    let lands = dauthi_void_lands(g, m);
    let land_ok = legal::sorcery_timing(g, p).is_none() && (g.player(p).lands_played as i32) < legal::land_drops(g, p);
    if land_ok {
        void.extend(lands.iter().copied());
    }
    if void.is_empty() {
        return Ok(Some(if lands.is_empty() {
            "No card is exiled with a void counter.".into()
        } else {
            "Only a land is exiled with a void counter, and you can't play a land now.".into()
        }));
    }
    let labels: Vec<String> =
        void.iter().map(|&(c, q)| format!("{} ({}'s)", choices::card_label(g, c), pname(g, q))).collect();
    let Some(k) = choices::choose(
        g,
        p,
        "target",
        "Dauthi Voidwalker: play which exiled card for free?",
        &labels,
        Some("cancel"),
        None,
    )?
    else {
        return Ok(None);
    };
    let (c, q) = void[k];
    crate::glog!(g, "  {} activates Dauthi Voidwalker", pname(g, p));
    if g.db.get(c).land {
        // {T}, sacrifice it; then the land is played from exile as the land drop
        let Some(cd) = g.perm(m).cd else { return Ok(None) };
        zones::die(g, m, "sac")?;
        let label = format!("play {}", cname(g, c));
        if engine::stack::ability_window_card(g, p, cd, &label, Some(5.0), None)? && g.player(q).exile.contains(&c) {
            g.player_mut(q).exile.take(c);
            crate::glog!(g, "  Dauthi Voidwalker: {} plays {} from exile", pname(g, p), cname(g, c));
            engine::turn::play_land_card(g, p, c, "plays (Dauthi Voidwalker)")?;
        }
        engine::life::check_state(g)?;
        return Ok(None);
    }
    // a spell: "you may play it this turn without paying its mana cost". The Voidwalker is sacrificed as a cost; the
    // permission lasts the turn and the person casts the card (from exile, for free) whenever they like, or not at
    // all (`dauthi_permits`, `legal::check_cast_exile`). The AI's play (`common::dauthi_go`) casts it at once.
    let Some(cd) = g.perm(m).cd else { return Ok(None) };
    zones::die(g, m, "sac")?;
    let what = format!("you may play {} this turn", cname(g, c));
    if engine::stack::ability_window_card(g, p, cd, &what, Some(5.0), None)? && g.player(q).exile.contains(&c) {
        let st = g.turn_stamp();
        let mut list = match g.perm(m).data.get(DataKey::Dauthi) {
            Some(Val::List(v)) if v.first() == Some(&Val::Stamp(st)) => v.clone(),
            _ => vec![Val::Stamp(st)],
        };
        list.push(Val::Card(c));
        g.perm_mut(m).data.set(DataKey::Dauthi, Val::List(list));
        crate::glog!(g, "    Dauthi Voidwalker: {what}");
    }
    engine::life::check_state(g)?;
    Ok(None)
}

/// p may play the exiled card c this turn without paying its mana cost, by a Dauthi Voidwalker or by Urza's {5} / Triple
/// Triad (`Player::free_play`): whose exile it is in
pub fn exile_permits(g: &Game, p: PlayerId, c: CardId) -> Option<PlayerId> {
    if g.player(p).free_play.contains(&c)
        && let Some(q) = g.players.iter().find(|q| q.exile.contains(&c)).map(|q| q.id)
    {
        return Some(q);
    }
    dauthi_permits(g, p, c)
}

/// the exiled cards p may play this turn without paying their mana costs: (card, whose exile)
pub fn exile_playable(g: &Game, p: PlayerId) -> Vec<(CardId, PlayerId)> {
    let mut out = dauthi_playable(g, p);
    for &c in &g.player(p).free_play {
        if let Some(q) = exile_permits(g, p, c)
            && !out.contains(&(c, q))
        {
            out.push((c, q));
        }
    }
    out
}

/// a person's Dauthi Voidwalker let p play the exiled card c this turn: whose exile it is in
pub fn dauthi_permits(g: &Game, p: PlayerId, c: CardId) -> Option<PlayerId> {
    // every Voidwalker p has had (the permanents' arena keeps the sacrificed one: the permission outlives it)
    let st = g.turn_stamp();
    let held = g.perms.iter().any(|x| {
        x.owner == p
            && matches!(x.data.get(DataKey::Dauthi),
                Some(Val::List(v)) if v.first() == Some(&Val::Stamp(st)) && v.contains(&Val::Card(c)))
    });
    if !held {
        return None;
    }
    g.players.iter().find(|q| q.exile.contains(&c)).map(|q| q.id)
}

/// the exiled cards p may play this turn: (card, whose exile)
pub fn dauthi_playable(g: &Game, p: PlayerId) -> Vec<(CardId, PlayerId)> {
    let st = g.turn_stamp();
    let mut out = vec![];
    for x in g.perms.iter().filter(|x| x.owner == p) {
        if let Some(Val::List(v)) = x.data.get(DataKey::Dauthi)
            && v.first() == Some(&Val::Stamp(st))
        {
            for e in v.iter().skip(1) {
                if let Val::Card(c) = e
                    && let Some(q) = dauthi_permits(g, p, *c)
                    && !out.contains(&(*c, q))
                {
                    out.push((*c, q));
                }
            }
        }
    }
    out
}

/// the lands among Dauthi Voidwalker's void cards (`common::dauthi_void` leaves them out: the AI plays only spells)
fn dauthi_void_lands(g: &Game, m: PermId) -> Vec<(CardId, PlayerId)> {
    let Some(Val::List(v)) = g.perm(m).data.get(DataKey::Void) else { return vec![] };
    v.iter()
        .filter_map(|e| match e {
            Val::List(x) => match (&x[0], &x[1]) {
                (Val::Card(c), Val::Player(q)) => Some((*c, *q)),
                _ => None,
            },
            _ => None,
        })
        .filter(|&(c, q)| g.player(q).exile.contains(&c) && g.db.get(c).land)
        .collect()
}

// ------------------------------------------------------------------ Resplendent Angel
/// {3}{W}{W}{W}: until end of turn, +2/+2 and lifelink
fn resplendent(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = cost_ok(g, p, 3, "WWW", "Resplendent Angel") {
        return Ok(Some(why));
    }
    mana::pay_from_pool(g, p, 3, "WWW");
    crate::glog!(g, "  {} activates Resplendent Angel", pname(g, p));
    if ability_window(g, p, Some(m), "+2/+2 and lifelink", None, None)? && alive(g, m) {
        crate::impls::t3::eot(g, m, 2, 2);
        crate::impls::partials::eot_kw(g, m, "lifelink");
    }
    Ok(None)
}

// ------------------------------------------------------------------ Venat, Heart of Hydaelyn
/// Hero's Sundering, {7}, {T}, sorcery speed: exile target nonland permanent, then transform Venat
fn sundering(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if g.perm(m).data.truthy(DataKey::Hydaelyn) {
        return Ok(Some("Venat has already transformed into Hydaelyn.".into()));
    }
    if let Some(why) = legal::sorcery_timing(g, p) {
        return Ok(Some(why.replace("do that", "activate Hero's Sundering")));
    }
    if let Some(why) = tap_ok(g, m, true).or_else(|| cost_ok(g, p, 7, "", "Hero's Sundering")) {
        return Ok(Some(why));
    }
    let tg: Vec<PermId> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&t| {
            let x = g.perm(t);
            x.cd.is_some_and(|c| !g.db.get(c).land) && !x.phased && !(x.owner != p && values::untargetable(g, t))
        })
        .collect();
    if tg.is_empty() {
        return Ok(Some("There is no nonland permanent to target.".into()));
    }
    let Some(t) =
        cards::pick_perm(g, p, "target", "Hero's Sundering: exile which nonland permanent?", &tg, Some("cancel"))?
    else {
        return Ok(None);
    };
    crate::impls::legends::sundering_go(g, m, p, t.0 as i64)?;
    Ok(None)
}
