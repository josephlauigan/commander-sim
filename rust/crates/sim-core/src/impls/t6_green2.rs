//! Tier 6 batch `green2`: the green and small utility cards of the Rograkh & Thrasios and Kinnan decks
//! (rust/agents/t6-batches.md). Cards only: no engine file is touched; what needs the engine is listed in
//! rust/agents/engine-needs-green2.md.
//!
//! - Chatterstorm: the Squirrel token is the compiled spell; storm is `t0_stella::storm_copies` on the cast.
//! - Cloud of Faeries: ETB untaps two lands; cycling {2} is a hand option.
//! - Noxious Revival: resolve puts the best of the caster's own graveyard cards on top of its library.
//! - Sowing Mycospawn: the cast trigger fetches a land; the kicked exile is `mycospawn_kicked` (needs engine kicker).

use crate::cards::{CardDb, Colors};
use crate::engine::mana::land_colors_now;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::trigger_window;
use crate::engine::turn::land_enters_tapped;
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::zones::{discard_cards, draw, max_by};
use crate::engine::zones::{landfall, searchable};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn remove_card(v: &mut Vec<CardId>, c: CardId) {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
    }
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

// ================================================================== Chatterstorm
/// Storm: when you cast it, copy it for each spell cast before it this turn
fn chatterstorm_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    crate::impls::t0_stella::storm_copies(g, p, c)
}

// ================================================================== Cloud of Faeries
/// When this creature enters, untap up to two lands: the AI untaps two tapped lands of its own (it always takes the
/// full two when it has them).
fn cloud_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src || !trigger_window(g, o, Some(src), "untap up to two lands", Some(3.0))? {
        return Ok(());
    }
    let tapped: Vec<_> = g.player(o).lands.iter().copied().filter(|&l| g.land(l).tapped).take(2).collect();
    for l in &tapped {
        g.land_mut(*l).tapped = false;
    }
    crate::glog!(g, "    {} untaps {} land(s) (Cloud of Faeries)", pname(g, o), tapped.len());
    Ok(())
}

/// Cycling {2}: the AI cycles it in an opponent's end step or after combat when it has the two mana to spare.
fn cloud_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !g.player(p).hand.contains(&c) || !(post.is_none() && g.active != Some(p) || post == Some(true)) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 0.9,
        label: "cycle Cloud of Faeries".into(),
        act: Some(Action::Plan { f: cloud_cycle, arg: c.0 as i64 }),
    }])
}

fn cloud_cycle(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    crate::glog!(g, "  {} cycles Cloud of Faeries", pname(g, p));
    discard_cards(g, p, &[c])?;
    draw(g, p, 1, false)?;
    Ok(true)
}

// ================================================================== Noxious Revival
/// the nonland card in one of p's own graveyard worth putting back on top: the best by `card_worth`, a creature
/// counting a little more and mana value breaking ties (Oboro, Eternal Witness); None if there is none
pub fn revival_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let gy: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| !g.db.get(c).land).collect();
    max_by(&gy, |c| {
        let d = g.db.get(c);
        card_worth(g, p, c, true) + if d.creature { 5.0 } else { 0.0 } + 0.1 * d.cmc as f64
    })
}

/// {G/P}, instant: put target card from a graveyard on top of its owner's library. The AI aims it at its own best
/// card (the engine's Phyrexian mana lets it pay 2 life); with nothing worth it, it fizzles to the graveyard.
fn revival_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = revival_pick(g, p) {
        let pl = g.player_mut(p);
        remove_card(&mut pl.gy, t);
        pl.library.push(t);
        crate::glog!(g, "    {} puts {} on top of its library (Noxious Revival)", pname(g, p), cname(g, t));
    }
    Ok(intern("gy"))
}

/// cast only to set up a draw: when the graveyard holds a card worth drawing
fn revival_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if revival_pick(g, p).is_some() { 30 } else { 0 }
}

// ================================================================== Sowing Mycospawn
/// the land card the AI fetches: Gaea's Cradle once it has two creatures, else the land adding the most colours it
/// lacks (then the one making the most colours), else the first land
pub fn mycospawn_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let lands: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| g.db.get(c).land).collect();
    let creatures = g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.is_creature(m)).count();
    let have = g.player(p).lands.iter().fold(Colors::NONE, |a, &l| a.union(land_colors_now(g, p, l)));
    max_by(&lands, |c| {
        let d = g.db.get(c);
        let cols = match d.tags.str(Tag::C).unwrap_or("") {
            "A" => g.player(p).ident,
            "C" | "" => Colors::NONE,
            s => Colors::from_letters(s),
        };
        let new_cols = "WUBRG".chars().filter(|&ch| cols.has(ch) && !have.has(ch)).count() as f64;
        let cradle = if &*d.name == "Gaea's Cradle" && creatures >= 2 { 2000.0 } else { 0.0 };
        cradle + new_cols * 100.0 + cols.count() as f64
    })
}

/// When you cast this spell, search your library for a land card, put it onto the battlefield, then shuffle. This is a
/// cast trigger, so it happens even if the spell is countered. The land enters untapped unless it says otherwise.
fn mycospawn_cast(g: &mut Game, p: PlayerId, _c: CardId) -> Res {
    if !trigger_window(g, p, None, "Sowing Mycospawn: search for a land", Some(3.0))? {
        return Ok(());
    }
    if let Some(c) = mycospawn_pick(g, p) {
        let lib = &mut g.player_mut(p).library;
        if let Some(i) = lib.iter().position(|&x| x == c) {
            lib.remove(i);
        }
        let tapped = land_enters_tapped(g, p, c);
        g.add_land(p, c, tapped);
        crate::glog!(g, "    {} fetches {} (Sowing Mycospawn)", pname(g, p), cname(g, c));
        shuffle_library(g, p);
        landfall(g, p)?;
    } else {
        shuffle_library(g, p);
    }
    Ok(())
}

/// The kicked half of the cast trigger: exile target land (the AI takes the best opposing land). Nothing calls this
/// yet: the engine has no kicker (additional cost chosen as a spell is cast, with a flag the cast trigger can read);
/// see rust/agents/engine-needs-green2.md.
pub fn mycospawn_kicked(g: &mut Game, p: PlayerId) -> Res {
    let Some((q, l)) = crate::impls::fixes::best_opp_land(g, p) else { return Ok(()) };
    if !trigger_window(g, p, None, "Sowing Mycospawn: exile target land", Some(4.0))? {
        return Ok(());
    }
    let cd = g.land(l).cd;
    crate::engine::turn::remove_land(g, q, l);
    g.player_mut(q).exile.push(cd);
    crate::glog!(g, "    {} is exiled (Sowing Mycospawn, kicked)", cname(g, cd));
    Ok(())
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Chatterstorm")?.self_cast = Some(chatterstorm_cast);
    let c = r.card(db, "Cloud of Faeries")?;
    c.etb = Some(cloud_etb);
    c.hand_options = Some(cloud_options);
    let c = r.card(db, "Noxious Revival")?;
    c.resolve = Some(revival_resolve);
    c.prio = Some(revival_prio);
    r.card(db, "Sowing Mycospawn")?.self_cast = Some(mycospawn_cast);
    Ok(())
}
