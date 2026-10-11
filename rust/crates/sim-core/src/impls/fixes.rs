//! Python's `cards/impl/fixes.py`: corrections for cards the ability compiler reads wrongly, each a hand-written
//! implementation (the Tier 1 and Sauron cards and Deathrite Shaman for M5, the rest in phase 6: `register_phase6`).

use super::partials::{at_once, best_opp_nonland, first_max, first_min, of_type, on, pack, remove_card, unpack};
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{castable, on_cast};
use crate::engine::hooks::{fire_trigger, hooked};
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{Source, Unit, can_pay, cost_of, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, counter_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library, tutor};
use crate::engine::values::{epow, has_type, pval, threat};
use crate::engine::zones::{Enter, Tokens, bounce, die, enter, landfall, leave, make_tokens, min_by, searchable};
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
    atk: &mut Vec<PermId>,
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
    for &a in atk.iter() {
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
    register_phase6(r, db)
}

// ======================================================== phase 6: the rest of fixes.py
fn opt_ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

// ------------------------------------------------------------------ Acidic Slime
fn slime_pred(g: &Game, x: PermId) -> bool {
    g.perm(x).cd.is_some() && !g.is_creature(x) && (of_type(g, x, Types::ARTIFACT) || of_type(g, x, Types::ENCHANTMENT))
}

/// Full: deathtouch; destroys the best opposing artifact or enchantment, else a nonbasic land
fn slime(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if best_opp_nonland(g, o, slime_pred).is_none() && best_opp_land(g, o).is_none() {
        return Ok(()); // nothing to destroy
    }
    if !trigger_window(g, o, Some(src), "destroy an artifact, enchantment or land", Some(5.0))? {
        return Ok(());
    }
    let t = best_opp_nonland(g, o, slime_pred);
    if let Some(t) = t.filter(|&t| pval(g, t) >= 2.5) {
        apply_removal(g, Some(o), t, "destroy", None)?;
        return Ok(());
    }
    if let Some((q, l)) = best_opp_land(g, o) {
        destroy_land(g, q, l)?;
    } else if let Some(t) = t {
        apply_removal(g, Some(o), t, "destroy", None)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ mana rocks that enter tapped
/// Charcoal Diamond, Fire Diamond, Coldsteel Heart, Worn Powerstone (Full): enter tapped
fn tapped_rock(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.perm_mut(src).tapped = true;
    }
    Ok(())
}

// ------------------------------------------------------------------ Decimate
fn decimate_ok(_g: &Game, _src: Src, _caster: PlayerId, _c: CardId, _zone: Sym) -> bool {
    true
}

type DecimateTargets = (Option<PermId>, Option<PermId>, Option<PermId>, Vec<(PlayerId, LandId)>);

fn decimate_targets(g: &Game, p: PlayerId) -> DecimateTargets {
    let a = best_opp_nonland(g, p, |g, m| of_type(g, m, Types::ARTIFACT));
    let cr = best_opp_nonland(g, p, |g, m| g.is_creature(m));
    let e = best_opp_nonland(g, p, |g, m| of_type(g, m, Types::ENCHANTMENT));
    let lands = g.players.iter().filter(|q| q.alive).flat_map(|q| q.lands.iter().map(move |&l| (q.id, l))).collect();
    (a, cr, e, lands)
}

fn decimate_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let (a, cr, e, lands) = decimate_targets(g, p);
    let (Some(a), Some(cr), Some(e)) = (a, cr, e) else { return 0 };
    if lands.is_empty() {
        return 0;
    }
    if pval(g, a) + pval(g, cr) + pval(g, e) >= 9.0 { 60 } else { 0 }
}

/// Full: needs an artifact, a creature, an enchantment and a land to target; destroys the best of each
fn decimate(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let (a, cr, e, lands) = decimate_targets(g, p);
    for t in [a, cr, e].into_iter().flatten() {
        if g.perm(t).on_bf {
            apply_removal(g, Some(p), t, "destroy", None)?;
        }
    }
    if let Some((q, l)) = best_opp_land(g, p) {
        destroy_land(g, q, l)?;
    } else if let Some(&(q, l)) = first_min(&lands, |(q, _)| (q != p, 0)).as_ref() {
        destroy_land(g, q, l)?;
    }
    Ok("gy")
}

// ------------------------------------------------------------------ Gamble: tutor, then discard at random
/// Full: tutor any card to hand, then discard a card at random
fn gamble(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    tutor(g, p, "any")?;
    let rest: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&x| x != c).collect();
    if let Some(&x) = g.rng.choice(&rest) {
        let pl = g.player_mut(p);
        remove_card(&mut pl.hand, x);
        pl.gy.push(x);
        crate::glog!(g, "    Gamble discards {}", g.db.get(x).name);
        if !g.hooks.is_empty() {
            fire_trigger(g, Event::Discard, Call::Discard { p, c: x })?;
        }
    }
    Ok("gy")
}

// ------------------------------------------------------------------ Goblin Trashmaster: sacrifice a Goblin
/// Full: other Goblins +1/+1; sacrifices a spare Goblin to destroy a valuable artifact
fn trashmaster(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || post.is_none() {
        return Ok(vec![]);
    }
    let t = best_opp_nonland(g, p, |g, m| of_type(g, m, Types::ARTIFACT));
    let fod: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && m != src && has_type(g, m, "goblin") && (g.perm(m).token || pval(g, m) < 2.0))
        .collect();
    let Some(t) = t.filter(|&t| !fod.is_empty() && pval(g, t) >= 3.0) else { return Ok(vec![]) };
    let f = min_by(&fod, |m| pval(g, m)).unwrap();
    Ok(vec![opt_ability(pval(g, t) - 1.5, "Goblin Trashmaster".into(), src, trashmaster_go, pack(t.0, f.0))])
}

fn trashmaster_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (t, f) = unpack(arg);
    let (t, f) = (PermId(t), PermId(f));
    if !on(g, p, f) || !g.perm(t).on_bf {
        return Ok(false);
    }
    die(g, f, "sac")?;
    let label = format!("destroy {}", g.perm(t).name);
    if ability_window(g, p, Some(src), &label, None, Some(t))? && g.perm(t).on_bf {
        apply_removal(g, Some(p), t, "destroy", None)?;
    }
    Ok(true)
}

// ------------------------------------------------------------------ Reshape: X limits the artifact
/// t5.IC_COMBO_ART
const IC_COMBO_ART: [&str; 6] = [
    "Isochron Scepter",
    "Power Artifact",
    "Basalt Monolith",
    "Grim Monolith",
    "Mycosynth Lattice",
    "Rings of Brighthearth",
];

fn reshape_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let spare =
        g.player(p).perms.iter().any(|&m| of_type(g, m, Types::ARTIFACT) && (g.perm(m).token || pval(g, m) < 3.0));
    if total_mana(g, p, false) >= 4 && spare { 55 } else { 0 }
}

/// Full: sacrifices the least valuable artifact; X = spare mana; the best artifact with MV X or less onto the
/// battlefield
fn reshape(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let arts: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| of_type(g, m, Types::ARTIFACT) && !g.perm(m).phased).collect();
    let Some(sac) = min_by(&arts, |m| pval(g, m)) else { return Ok("gy") };
    die(g, sac, "sac")?;
    let x = ctx.x;
    let cs: Vec<CardId> = searchable(g, p)
        .into_iter()
        .filter(|&y| g.db.get(y).types.has(Types::ARTIFACT) && g.db.get(y).cmc as i32 <= x)
        .collect();
    if let Some(y) = first_max(&cs, |y| {
        let d = g.db.get(y);
        (IC_COMBO_ART.contains(&&*d.name), card_worth(g, p, y, false), d.cmc)
    }) {
        remove_card(&mut g.player_mut(p).library, y);
        shuffle_library(g, p);
        enter(g, p, y, Enter::default())?;
    }
    Ok("gy")
}

// ------------------------------------------------------------------ Retrofitter Foundry
/// Full: Servo -> Thopter -> 4/4 Construct chain; untaps for {3} with spare mana
fn foundry(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || post.is_none() {
        return Ok(vec![]);
    }
    if g.perm(src).tapped {
        if !can_pay(g, p, 3, "", false) || total_mana(g, p, false) < 5 {
            return Ok(vec![]);
        }
        return Ok(vec![opt_ability(0.8, "Retrofitter Foundry untap".into(), src, foundry_untap, 0)]);
    }
    let tok = |t: &str| g.player(p).perms.iter().copied().find(|&m| g.perm(m).token && has_type(g, m, t));
    if let Some(th) = tok("thopter") {
        return Ok(vec![opt_ability(
            3.0,
            "Retrofitter Foundry: Construct".into(),
            src,
            foundry_make,
            pack(0, th.0 + 1),
        )]);
    }
    if let Some(sv) = tok("servo")
        && can_pay(g, p, 1, "", false)
    {
        return Ok(vec![opt_ability(2.0, "Retrofitter Foundry: Thopter".into(), src, foundry_make, pack(1, sv.0 + 1))]);
    }
    if can_pay(g, p, 2, "", false) {
        return Ok(vec![opt_ability(1.5, "Retrofitter Foundry: Servo".into(), src, foundry_make, pack(2, 0))]);
    }
    Ok(vec![])
}

fn foundry_untap(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !g.perm(src).tapped || !can_pay(g, p, 3, "", false) {
        return Ok(false);
    }
    pay(g, p, 3, "", false)?;
    if ability_window(g, p, Some(src), "untap", None, None)? {
        g.perm_mut(src).tapped = false;
    }
    Ok(true)
}

/// kind 0: a Construct (sacrifice a Thopter), 1: a Thopter ({1}, sacrifice a Servo), 2: a Servo ({2})
fn foundry_make(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (kind, fod) = unpack(arg);
    let fod = fod.checked_sub(1).map(PermId);
    let cost = kind;
    if g.perm(src).tapped || !can_pay(g, p, cost, "", false) {
        return Ok(false);
    }
    if let Some(f) = fod
        && !on(g, p, f)
    {
        return Ok(false);
    }
    pay(g, p, cost, "", false)?;
    g.perm_mut(src).tapped = true;
    if let Some(f) = fod {
        leave(g, f)?;
    }
    let name = ["construct", "thopter", "servo"][kind as usize];
    if !ability_window(g, p, Some(src), &format!("a {name}"), None, None)? {
        return Ok(true);
    }
    let spec = match kind {
        0 => Tokens { types: vec!["construct", "artifact"], ..Tokens::new(1, 4) },
        1 => Tokens { fly: true, types: vec!["thopter", "artifact"], ..Tokens::new(1, 1) },
        _ => Tokens { types: vec!["servo", "artifact"], ..Tokens::new(1, 1) },
    };
    make_tokens(g, p, spec)?;
    Ok(true)
}

// ------------------------------------------------------------------ Rishkar, Peema Renegade
/// Full: +1/+1 counters on two of your creatures; creatures with counters tap for {G}
fn rishkar(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if !trigger_window(g, o, Some(src), "put +1/+1 counters on two creatures", None)? {
        return Ok(());
    }
    let mut cs: Vec<(bool, f64, PermId)> = g
        .player(o)
        .perms
        .iter()
        .copied()
        .filter(|&x| g.is_creature(x) && !g.perm(x).phased)
        .map(|x| (x == src, -pval(g, x), x))
        .collect();
    // Python sorts by (x is src, -pval) with reverse=True: Rishkar first, then the least valuable (stable)
    cs.sort_by(|a, b| (b.0, b.1).partial_cmp(&(a.0, a.1)).unwrap_or(std::cmp::Ordering::Equal));
    for &(_, _, x) in cs.iter().take(2) {
        g.perm_mut(x).plus += 1;
    }
    Ok(())
}

fn rishkar_mana(g: &Game, _src: Src, p: PlayerId, u: &[Unit]) -> Vec<Unit> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m)
                && x.plus > 0
                && !x.tapped
                && !x.sick
                && !x.phased
                && !u.iter().any(|w| w.src == Source::Perm(m))
                && !x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Dork))
        })
        .map(|m| Unit { src: Source::Perm(m), cols: Colors::from_letters("G"), amt: 1 })
        .collect()
}

// ------------------------------------------------------------------ Crop Rotation
/// fixes._crop_target: (the land Crop Rotation should fetch now, gain): Gaea's Cradle with three or more creatures
/// out; with landfall payoffs out, a fetch land (two landfalls)
fn crop_target(g: &Game, p: PlayerId) -> (Option<CardId>, i32) {
    let lib: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| g.db.get(c).land).collect();
    let n = g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count() as i32;
    let cradle = lib.iter().copied().find(|&c| &*g.db.get(c).name == "Gaea's Cradle");
    if let Some(c) = cradle
        && n >= 3
    {
        return (Some(c), n - 1);
    }
    let payoffs = if g.hooks.is_empty() {
        0
    } else {
        hooked(g, Event::Landfall).iter().filter(|(src, _)| g.perm(*src).owner == p).count() as i32
    };
    let fetch = lib.iter().copied().find(|&c| g.db.get(c).tag(Tag::F));
    if let Some(f) = fetch
        && payoffs != 0
    {
        return (Some(f), payoffs);
    }
    (None, 0)
}

fn crop_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.player(p).lands.is_empty() {
        return 0;
    }
    let (target, gain) = crop_target(g, p);
    if target.is_none() || gain < 2 { 0 } else { 80.min(50 + 6 * gain) }
}

/// Approximate: sacrifices your least useful land (tapped first) for Gaea's Cradle with three or more creatures, or a
/// fetch land when landfall payoffs are out; held otherwise
fn crop_rotation(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if g.player(p).lands.is_empty() {
        return Ok("gy");
    }
    let lands = g.player(p).lands.clone();
    let mut keys = vec![];
    for &l in &lands {
        let d = g.db.get(g.land(l).cd);
        let keep = matches!(&*d.name, "Gaea's Cradle" | "Ancient Tomb");
        let k = (keep, !g.land(l).tapped, d.tags.str(Tag::C).map_or(0, |s| s.chars().count()), g.rng.random());
        keys.push(k);
    }
    let i = first_min(&(0..lands.len()).collect::<Vec<_>>(), |i| keys[i]).unwrap();
    let sac = lands[i];
    let (target, _) = crop_target(g, p);
    crate::engine::turn::remove_land(g, p, sac);
    let cd = g.land(sac).cd;
    g.player_mut(p).gy.push(cd);
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::LandGy, Call::Cards { p, cards: vec![cd] })?;
    }
    let Some(target) = target.filter(|t| g.player(p).library.contains(t)) else { return Ok("gy") };
    remove_card(&mut g.player_mut(p).library, target);
    shuffle_library(g, p);
    g.player_mut(p).stat("tutored", 1);
    let tapped = crate::engine::turn::land_enters_tapped(g, p, target);
    g.add_land(p, target, tapped);
    crate::glog!(
        g,
        "    Crop Rotation: {} sacrifices {} for {}",
        pname(g, p),
        g.db.get(cd).name,
        g.db.get(target).name
    );
    landfall(g, p)?;
    if g.db.get(target).tag(Tag::F) {
        let l = *g.player(p).lands.last().unwrap(); // `p.lands[-1]`, after the landfall triggers
        crate::engine::turn::crack_fetch(g, p, l)?;
    }
    Ok("gy")
}

// ------------------------------------------------------------------ Mana Vault: pay {4} at upkeep to untap it
/// untapping costs 4 and gives back 3 now (1 mana net this turn) and stops the 1 damage each draw step: done when
/// life has started to matter and the rest of the mana still casts the best spell in hand
fn vault_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    let x = g.perm(src);
    if p != o || !x.tapped || x.phased || g.player(o).life > 20 || !can_pay(g, o, 4, "", false) {
        return Ok(());
    }
    let avail = total_mana(g, o, false) as i64;
    let need = g
        .player(o)
        .hand
        .iter()
        .map(|&c| g.db.get(c))
        .filter(|d| !d.land && d.cmc as i64 <= avail)
        .map(|d| d.cmc as i64)
        .max()
        .unwrap_or(0);
    if avail - 1 < need {
        return Ok(());
    }
    pay(g, o, 4, "", false)?;
    g.perm_mut(src).tapped = false;
    crate::glog!(g, "    {} pays {{4}} to untap Mana Vault", pname(g, o));
    Ok(())
}

// ------------------------------------------------------------------ contextual priorities for the outside decks' cards
fn seedborn_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::plans::seedborn_prio(g, p, c)
}

fn drannith_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::plans::drannith_prio(g, p, c)
}

fn register_phase6(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Acidic Slime")?;
    c.etb = Some(slime);
    at_once(c, Event::Etb, false);
    for name in ["Charcoal Diamond", "Fire Diamond", "Coldsteel Heart", "Worn Powerstone"] {
        let c = r.card(db, name)?;
        c.etb = Some(tapped_rock);
        at_once(c, Event::Etb, true);
    }
    // Chord of Calling: t3._x_tutor (fixes.py calls t3's helper; registered in t3.rs)
    let c = r.card(db, "Decimate")?;
    c.can_cast = Some(decimate_ok);
    c.resolve = Some(decimate);
    c.prio = Some(decimate_prio);
    let c = r.card(db, "Gamble")?;
    c.resolve = Some(gamble);
    c.prio = Some(crate::ai::decks::gamble_prio);
    r.card(db, "Goblin Trashmaster")?.options = Some(trashmaster);
    let c = r.card(db, "Reshape")?;
    c.resolve = Some(reshape);
    c.prio = Some(reshape_prio);
    r.card(db, "Retrofitter Foundry")?.options = Some(foundry);
    let c = r.card(db, "Rishkar, Peema Renegade")?;
    c.etb = Some(rishkar);
    at_once(c, Event::Etb, false);
    c.extra_mana = Some(rishkar_mana);
    let c = r.card(db, "Crop Rotation")?;
    c.resolve = Some(crop_rotation);
    c.prio = Some(crop_prio);
    let c = r.card(db, "Mana Vault")?;
    c.upkeep = Some(vault_upkeep);
    at_once(c, Event::Upkeep, true);
    // Tergrid's menace is in the card data (fixes.py edits DB at import; the export has it)
    r.card(db, "Seedborn Muse")?.prio = Some(seedborn_prio);
    r.card(db, "Drannith Magistrate")?.prio = Some(drannith_prio);
    Ok(())
}
