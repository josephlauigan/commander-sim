//! Python's `cards/impl/fixes.py`: corrections for cards the ability compiler reads wrongly, each a hand-written
//! implementation (the Tier 1 and Sauron cards so far, and Deathrite Shaman; the rest come with phase 6).

use super::partials::{best_opp_nonland, first_max, of_type, on, pack, remove_card, unpack};
use crate::cards::{CardDb, Types};
use crate::engine::cast::{castable, on_cast};
use crate::engine::hooks::fire_trigger;
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, counter_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{epow, has_type, pval, threat};
use crate::engine::zones::{Enter, bounce, die, enter, leave};
use crate::flow::Res;
use crate::hooks::{Action, Call, Event, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::Sym;
use crate::tag::Tag;

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

/// fixes.destroy_land: q's land is destroyed (its card to the graveyard)
pub fn destroy_land(g: &mut Game, q: PlayerId, l: LandId) -> Res {
    if !g.player(q).lands.contains(&l) {
        return Ok(());
    }
    crate::engine::turn::remove_land(g, q, l);
    let cd = g.land(l).cd;
    g.player_mut(q).gy.push(cd);
    crate::glog!(g, "    {} ({}) is destroyed", g.db.get(cd).name, pname(g, q));
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::LandGy, Call::Cards { p: q, cards: vec![cd] })?;
    }
    Ok(())
}

/// fixes.best_opp_land: the most valuable opposing land: nonbasics that make several colours or big mana, of the
/// leading player
pub fn best_opp_land(g: &Game, p: PlayerId) -> Option<(PlayerId, LandId)> {
    let lands: Vec<(PlayerId, LandId)> = g
        .opps(p)
        .flat_map(|q| g.player(q).lands.iter().map(move |&l| (q, l)))
        .filter(|&(_, l)| {
            !matches!(&*g.db.get(g.land(l).cd).name, "Plains" | "Island" | "Swamp" | "Mountain" | "Forest")
        })
        .collect();
    first_max(&lands, |(q, l)| {
        let t = &g.db.get(g.land(l).cd).tags;
        (t.int(Tag::Amt).unwrap_or(1), t.str(Tag::C).map_or(0, |s| s.chars().count()), threat(g, p, q))
    })
}

// ------------------------------------------------------------------ Aetherize
/// d is attacked by p: return all attacking creatures when the attack is big (at least a third of d's life, or it
/// holds tokens that would simply vanish)
fn aetherize(
    g: &mut Game,
    c: CardId,
    d: PlayerId,
    p: PlayerId,
    atk: &[PermId],
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if !g.player(d).hand.contains(&c) || !castable(g, d, c, "hand") {
        return Ok(());
    }
    let (gn, pips) = cost_of(g, d, c);
    if !can_pay(g, d, gn, &pips, false) {
        return Ok(());
    }
    let power: i32 =
        atk.iter().filter(|&&a| on(g, p, a) && !assign.iter().any(|x| x.0 == a)).map(|&a| epow(g, a)).sum();
    let toks = atk.iter().filter(|&&a| g.perm(a).token).count();
    if (power as f64) < (8.0f64).max(g.player(d).life as f64 / 3.0) && toks < 4 {
        return Ok(());
    }
    pay(g, d, gn, &pips, false)?;
    remove_card(&mut g.player_mut(d).hand, c);
    crate::glog!(g, "  {} casts Aetherize", pname(g, d));
    on_cast(g, d, c)?;
    if !counter_window(g, d, c, 5.0, vec![])? {
        g.player_mut(d).gy.push(c);
        return Ok(());
    }
    for &a in atk {
        if g.perm(a).on_bf {
            if g.perm(a).token {
                leave(g, a)?;
            } else {
                bounce(g, a)?;
            }
        }
    }
    g.player_mut(d).gy.push(c);
    Ok(())
}

// ------------------------------------------------------------------ Casualties of War: every mode with a target
fn casualties_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let n = g.opps(p).flat_map(|q| g.player(q).perms.iter()).filter(|&&m| pval(g, m) >= 2.0).count();
    if n >= 2 { 58 } else { 0 }
}

/// destroys the best opposing artifact, creature, enchantment, planeswalker and nonbasic land (each mode that has a
/// target)
fn casualties(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for kind in [Types::ARTIFACT, Types::CREATURE, Types::ENCHANTMENT, Types::PLANESWALKER] {
        let t = if kind == Types::CREATURE {
            best_opp_nonland(g, p, |g, m| g.is_creature(m))
        } else {
            best_opp_nonland(g, p, |g, m| of_type(g, m, kind))
        };
        if let Some(t) = t.filter(|&t| g.perm(t).on_bf) {
            apply_removal(g, Some(p), t, "destroy", None)?;
        }
    }
    if let Some((q, l)) = best_opp_land(g, p) {
        destroy_land(g, q, l)?;
    }
    Ok("gy")
}

// ------------------------------------------------------------------ Crackling Doom: the greatest power
fn doom_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.opps(p).any(|q| g.player(q).perms.iter().any(|&m| g.is_creature(m))) { 55 } else { 0 }
}

/// 2 damage to each opponent; each opponent sacrifices its greatest-power creature
fn doom(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for q in g.opps(p).collect::<Vec<_>>() {
        lose_life(g, q, 2, Some(p), "burn", None)?;
    }
    for q in g.opps(p).collect::<Vec<_>>() {
        let cs: Vec<PermId> =
            g.player(q).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
        if let Some(m) = first_max(&cs, |m| (epow(g, m), -pval(g, m))) {
            crate::glog!(g, "    {} sacrifices {}", pname(g, q), g.perm(m).name);
            die(g, m, "sac")?;
        }
    }
    Ok("gy")
}

// ------------------------------------------------------------------ Deathrite Shaman
/// mana while a land card is in a graveyard
fn drs_mana(g: &Game, _p: PlayerId, _m: PermId) -> u32 {
    g.players.iter().any(|q| q.gy.iter().any(|&x| g.db.get(x).land)) as u32
}

/// the land card it exiles for that mana (an opponent's first)
fn drs_tap(g: &mut Game, p: PlayerId, _m: PermId, _used: u32) -> Res {
    let order: Vec<PlayerId> = g.players.iter().map(|q| q.id).filter(|&q| q != p).chain([p]).collect();
    for q in order {
        if let Some(x) = g.player(q).gy.iter().copied().find(|&x| g.db.get(x).land) {
            remove_card(&mut g.player_mut(q).gy, x);
            g.player_mut(q).exile.push(x);
            return Ok(());
        }
    }
    Ok(())
}

/// {B},{T}: exile an instant or sorcery card from a graveyard, each opponent loses 2; {G},{T}: exile a creature card
/// from a graveyard, you gain 2 (prefers an opponent's reanimation target)
fn drs_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if x.owner != p || x.tapped || x.sick || post.is_none() {
        return Ok(vec![]);
    }
    let mut o = vec![];
    let in_gys = |f: &dyn Fn(CardId) -> bool| -> Vec<(PlayerId, CardId)> {
        g.players.iter().flat_map(|q| q.gy.iter().filter(|&&c| f(c)).map(move |&c| (q.id, c))).collect()
    };
    let spell = in_gys(&|c| g.db.get(c).instant || g.db.get(c).sorcery);
    let cre = in_gys(&|c| g.db.get(c).creature);
    if !spell.is_empty() && can_pay(g, p, 0, "B", false) {
        let (q, c) = first_max(&spell, |(q, c)| (q != p, card_worth(g, q, c, true))).unwrap();
        o.push(Opt {
            utility: 2.2,
            label: "Deathrite Shaman drain".into(),
            act: Some(Action::Ability { src, f: drs_drain, arg: pack(q.0 as u32, c.0 as u32) }),
        });
    }
    if !cre.is_empty() && can_pay(g, p, 0, "G", false) {
        let (q, c) = first_max(&cre, |(q, c)| (q != p, g.db.get(c).bomb, g.db.get(c).cmc)).unwrap();
        if q != p {
            o.push(Opt {
                utility: 1.2 + 0.4 * g.db.get(c).bomb as f64,
                label: "Deathrite Shaman exile".into(),
                act: Some(Action::Ability { src, f: drs_hate, arg: pack(q.0 as u32, c.0 as u32) }),
            });
        }
    }
    Ok(o)
}

/// {T} is part of the cost, so Deathrite can't also tap for the mana; the card leaves the graveyard before paying
fn drs_pay(g: &mut Game, src: PermId, p: PlayerId, q: PlayerId, x: CardId, pip: &str) -> Res<bool> {
    if g.perm(src).tapped || !g.player(q).gy.contains(&x) {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if !can_pay(g, p, 0, pip, false) {
        g.perm_mut(src).tapped = false;
        return Ok(false);
    }
    remove_card(&mut g.player_mut(q).gy, x);
    pay(g, p, 0, pip, false)?;
    g.player_mut(q).exile.push(x);
    Ok(true)
}

fn drs_drain(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (q, x) = unpack(arg);
    let (q, x) = (PlayerId(q as u8), CardId(x as u16));
    if !drs_pay(g, src, p, q, x, "B")? {
        return Ok(false);
    }
    if !ability_window(g, p, Some(src), "each opponent loses 2 life", None, None)? {
        return Ok(true);
    }
    for o in g.opps(p).collect::<Vec<_>>() {
        lose_life(g, o, 2, Some(p), "triggers", None)?;
    }
    crate::glog!(g, "  {} uses Deathrite Shaman: exiles {}, each opponent loses 2", pname(g, p), g.db.get(x).name);
    Ok(true)
}

fn drs_hate(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (q, x) = unpack(arg);
    let (q, x) = (PlayerId(q as u8), CardId(x as u16));
    if !drs_pay(g, src, p, q, x, "G")? {
        return Ok(false);
    }
    if ability_window(g, p, Some(src), "you gain 2 life", None, None)? {
        gain(g, p, 2)?;
    }
    crate::glog!(g, "  {} uses Deathrite Shaman: exiles {}", pname(g, p), g.db.get(x).name);
    Ok(true)
}

// ------------------------------------------------------------------ Elvish Clancaller
/// {4}{G}{G},{T}: a card named Elvish Clancaller onto the battlefield
fn clancaller(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if x.owner != p || x.tapped || x.sick || post.is_none() || !can_pay(g, p, 4, "GG", false) {
        return Ok(vec![]);
    }
    if !g.player(p).library.iter().any(|&c| &*g.db.get(c).name == "Elvish Clancaller") {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.0,
        label: "Elvish Clancaller search".into(),
        act: Some(Action::Ability { src, f: clancaller_go, arg: 0 }),
    }])
}

fn clancaller_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let find = |g: &Game| g.player(p).library.iter().copied().find(|&c| &*g.db.get(c).name == "Elvish Clancaller");
    if g.perm(src).tapped || find(g).is_none() || !can_pay(g, p, 4, "GG", false) {
        return Ok(false);
    }
    pay(g, p, 4, "GG", false)?;
    g.perm_mut(src).tapped = true;
    if !ability_window(g, p, Some(src), "search for Elvish Clancaller", None, None)? {
        return Ok(true);
    }
    let Some(c) = find(g) else { return Ok(true) };
    remove_card(&mut g.player_mut(p).library, c);
    shuffle_library(g, p);
    enter(g, p, c, Enter::default())?;
    crate::glog!(g, "  {} fetches another Elvish Clancaller", pname(g, p));
    Ok(true)
}

// ------------------------------------------------------------------ Elvish Champion: forestwalk
fn forestwalk(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "forestwalk" && m != src && has_type(g, m, "elf")
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Aetherize")?.hand_defend = Some(aetherize);
    let c = r.card(db, "Casualties of War")?;
    c.resolve = Some(casualties);
    c.prio = Some(casualties_prio);
    let c = r.card(db, "Crackling Doom")?;
    c.resolve = Some(doom);
    c.prio = Some(doom_prio);
    let c = r.card(db, "Deathrite Shaman")?;
    c.dyn_mana_perm = Some(drs_mana);
    c.on_tap_perm = Some(drs_tap);
    c.options = Some(drs_options);
    r.card(db, "Elvish Clancaller")?.options = Some(clancaller);
    r.card(db, "Elvish Champion")?.grant_kw = Some(forestwalk);
    Ok(())
}
