//! Tier 6 batch `legends2`: Sisay's legends and interaction (Magda, The Queen of Dale, Tataru Taru, Teferi, Who Slows
//! the Sunset, Spider-Woman, Ertai Resurrected, Legolas's Quick Reflexes, Colossal Skyturtle, Wan Shi Tong). Cards only:
//! no engine file is touched; what needs the engine is listed in rust/agents/engine-needs-legends2.md.

use super::common::{WalkerAb, walker};
use super::partials::{at_once, first_max, on};
use super::t5::is_artifact;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::discard_worst;
use crate::engine::life::gain;
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{card_name, once_per_turn, pval, threat, untargetable};
use crate::engine::zones::{
    Enter, Tokens, add_treasure, agent_for, agent_take, draw, enter, make_tokens, max_by, min_by, searchable,
};
use crate::flow::Res;
use crate::hooks::{Action, Call, Event, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::sym::intern;
use crate::tag::Tag;

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn ability(utility: f64, label: &str, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label: label.into(), act: Some(Action::Ability { src, f, arg }) }
}

fn remove_card(v: &mut Vec<CardId>, c: CardId) {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
    }
}

// ================================================================== Magda, Brazen Outlaw
const MAGDA: &str = "Magda, Brazen Outlaw";

/// a Dwarf creature permanent (a changeling counts)
fn is_dwarf(g: &Game, m: PermId) -> bool {
    g.is_creature(m)
        && g.perm(m).cd.is_some_and(|c| {
            let d = g.db.get(c);
            d.has_subtype("dwarf") || d.has_kw("changeling")
        })
}

/// the Magdas p controls (not phased out), other than `except`
fn magdas(g: &Game, p: PlayerId, except: Option<PermId>) -> usize {
    g.player(p).perms.iter().filter(|&&y| Some(y) != except && live(g, y) && card_name(g, y) == Some(MAGDA)).count()
}

/// other Dwarves you control get +1/+0 (one per Magda), read from `creature_pt` while `g.selfpt` is on
fn magda_pt(g: &Game, m: PermId) -> (i32, i32) {
    if !is_dwarf(g, m) {
        return (0, 0);
    }
    (magdas(g, owner(g, m), Some(m)) as i32, 0)
}

fn magda_as(g: &mut Game, _p: PlayerId, _m: PermId) -> Res {
    g.selfpt = true;
    Ok(())
}

/// whenever a Dwarf you control becomes tapped for mana (the mana-tapped event): a Treasure
fn magda_mana(g: &mut Game, src: Src, p: PlayerId, m: PermId, _n: u32) -> Res {
    if live(g, src) && owner(g, src) == p && owner(g, m) == p && is_dwarf(g, m) {
        add_treasure(g, p, 1)?;
        crate::glog!(g, "    {} makes a Treasure ({} tapped)", g.perm(src).name, g.perm(m).name);
    }
    Ok(())
}

/// whenever a Dwarf you control becomes tapped by attacking: a Treasure each (one trigger window for all of them)
fn magda_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if !live(g, src) || owner(g, src) != p {
        return Ok(vec![]);
    }
    let k = atk.iter().filter(|&&m| owner(g, m) == p && g.perm(m).tapped && is_dwarf(g, m)).count();
    if k > 0 && trigger_window(g, p, Some(src), &format!("create {k} Treasure"), None)? {
        add_treasure(g, p, k as i32)?;
    }
    Ok(vec![])
}

/// the artifact or Dragon card Magda fetches: the best by worth
fn magda_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let ok: Vec<CardId> = searchable(g, p)
        .into_iter()
        .filter(|&c| {
            let d = g.db.get(c);
            d.types.has(Types::ARTIFACT) || d.has_subtype("dragon")
        })
        .collect();
    first_max(&ok, |c| card_worth(g, p, c, false))
}

/// Sacrifice five Treasures: search for an artifact or Dragon card, put it onto the battlefield, shuffle
fn magda_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !live(g, src) || owner(g, src) != p || g.player(p).treasures < 5 {
        return Ok(vec![]);
    }
    let Some(t) = magda_pick(g, p) else { return Ok(vec![]) };
    let u = 8.0f64.min(2.0 + 0.5 * g.db.get(t).cmc as f64);
    Ok(vec![ability(u, &format!("Magda: {}", g.db.get(t).name), src, magda_go, t.0 as i64)])
}

fn magda_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = CardId(arg as u16);
    if !on(g, p, src) || g.player(p).treasures < 5 || !g.player(p).library.contains(&t) {
        return Ok(false);
    }
    for _ in 0..5 {
        g.player_mut(p).treasures -= 1;
        if !g.hooks.is_empty() {
            crate::engine::hooks::fire_trigger(
                g,
                Event::Sacrifice,
                Call::Sacrifice { p, what: crate::hooks::Sacrificed::Token("Treasure") },
            )?;
        }
    }
    crate::glog!(g, "  {} sacrifices five Treasures: Magda searches for {}", g.player(p).name, g.db.get(t).name);
    if !ability_window(g, p, Some(src), &format!("search for {}", g.db.get(t).name), None, None)?
        || !g.player(p).library.contains(&t)
    {
        return Ok(true);
    }
    remove_card(&mut g.player_mut(p).library, t);
    shuffle_library(g, p);
    match agent_for(g, p) {
        Some(a) => agent_take(g, a, p, t),
        None => {
            enter(g, p, t, Enter::default())?;
        }
    }
    Ok(true)
}

// ================================================================== The Queen of Dale
/// Recruit: draw a card, then discard a card (the AI's usual discard); if the discarded card is not a land, create a
/// 1/1 white Human Soldier creature token
pub fn recruit(g: &mut Game, p: PlayerId) -> Res {
    draw(g, p, 1, false)?;
    let before = g.player(p).hand.clone();
    discard_worst(g, p, 1)?;
    let gone: Vec<CardId> = before.into_iter().filter(|c| !g.player(p).hand.contains(c)).collect();
    if gone.first().is_some_and(|&c| !g.db.get(c).land) {
        let spec = Tokens {
            color: Some(Colors::from_letters("W")),
            types: vec!["human".into(), "soldier".into()],
            ..Tokens::new(1, 1)
        };
        make_tokens(g, p, spec)?;
    }
    Ok(())
}

/// **The Queen of Dale** ({1}{W}, 2/1): whenever an opponent casts their first noncreature spell each turn, you recruit
fn queen_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    if !live(g, src) || caster == o || g.db.get(c).creature {
        return Ok(());
    }
    let st = g.turn_stamp();
    let earlier = match &g.player(caster).turn_casts {
        Some((s, cs)) if *s == st => cs.iter().filter(|&&x| x != c && !g.db.get(x).creature).count(),
        _ => 0,
    };
    if earlier == 0 && trigger_window(g, o, Some(src), "recruit", None)? {
        recruit(g, o)?;
    }
    Ok(())
}

// ================================================================== Tataru Taru
/// **Tataru Taru** ({1}{W}, 0/3 Dwarf): when it enters, you draw a card (the engine's `draw` tag on the card does
/// that) and target opponent may draw a card (the AI opponent with the least threat draws, if it has a card to draw).
/// Scions' Secretary: whenever an opponent draws a card, if it isn't that player's turn, create a tapped Treasure
/// token (once each turn). There are no tapped Treasures here (a Treasure is a counter), so the Treasure is kept on
/// Tataru (`DataKey::Charge`) and appears at its controller's next upkeep, which is when a tapped Treasure would
/// untap; if Tataru leaves first, it appears at once.
fn tataru_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src || !live(g, src) {
        return Ok(());
    }
    let o = owner(g, src);
    let opps: Vec<PlayerId> = g.opps(o).filter(|&q| !g.player(q).library.is_empty()).collect();
    let Some(t) = min_by(&opps, |q| threat(g, o, q)) else { return Ok(()) };
    if trigger_window(g, o, Some(src), "target opponent may draw a card", None)? {
        draw(g, t, 1, false)?;
    }
    Ok(())
}

fn tataru_pending(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.get(DataKey::Charge).map_or(0, Val::int)
}

fn tataru_draw(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = owner(g, src);
    if !live(g, src) || p == o || g.active == Some(p) {
        return Ok(());
    }
    if !once_per_turn(g, o, intern(&format!("scions_{}", src.0))) {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "create a tapped Treasure", None)? {
        let n = tataru_pending(g, src) + 1;
        g.perm_mut(src).data.set(DataKey::Charge, Val::Int(n));
    }
    Ok(())
}

fn tataru_release(g: &mut Game, src: Src) -> Res {
    let n = tataru_pending(g, src);
    if n > 0 {
        g.perm_mut(src).data.set(DataKey::Charge, Val::Int(0));
        add_treasure(g, owner(g, src), n as i32)?;
    }
    Ok(())
}

fn tataru_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == owner(g, src) {
        tataru_release(g, src)?;
    }
    Ok(())
}

fn tataru_leaves(g: &mut Game, src: Src, m: PermId) -> Res {
    if m == src {
        tataru_release(g, src)?;
    }
    Ok(())
}

// ================================================================== Teferi, Who Slows the Sunset
/// the permanent +1 aims at in one category: a tapped one of ours to untap, else an untapped one of theirs to tap
enum Aim {
    Untap(PermId),
    Tap(PermId),
}

fn aim_perm(
    g: &Game,
    p: PlayerId,
    pred: impl Fn(&Game, PermId) -> bool,
    worth: impl Fn(&Game, PermId) -> f64,
) -> Option<(Aim, f64)> {
    let own: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.perm(m).tapped && pred(g, m)).collect();
    let opp: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied().collect::<Vec<_>>())
        .filter(|&m| live(g, m) && !g.perm(m).tapped && pred(g, m) && !untargetable(g, m))
        .collect();
    let a = max_by(&own, |m| worth(g, m)).map(|m| (Aim::Untap(m), worth(g, m)));
    let b = max_by(&opp, |m| worth(g, m)).map(|m| (Aim::Tap(m), worth(g, m)));
    match (a, b) {
        (Some(x), Some(y)) => Some(if y.1 > x.1 { y } else { x }),
        (x, y) => x.or(y),
    }
}

fn rock_worth(g: &Game, m: PermId) -> f64 {
    let rock = g.perm(m).cd.is_some_and(|c| g.db.get(c).tags.str(Tag::Rock).is_some());
    if rock { 2.0 } else { 0.3 }
}

/// the three choices of +1 (an artifact, a creature, a land) with what each is worth to the AI
fn teferi_aims(g: &Game, p: PlayerId) -> (Option<(Aim, f64)>, Option<(Aim, f64)>, Option<(LandId, bool)>) {
    let art = aim_perm(g, p, |g, m| is_artifact(g, m), rock_worth);
    let cre = aim_perm(g, p, |g, m| g.is_creature(m), |g, m| pval(g, m) / 3.0);
    let own = g.player(p).lands.iter().copied().find(|&l| g.land(l).on_bf && g.land(l).tapped);
    let opp = g.opps(p).flat_map(|q| g.player(q).lands.clone()).find(|&l| g.land(l).on_bf && !g.land(l).tapped);
    let land = own.map(|l| (l, true)).or(opp.map(|l| (l, false)));
    (art, cre, land)
}

fn teferi_plus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let (a, c, l) = teferi_aims(g, p);
    Some(1.0 + a.map_or(0.0, |x| x.1) + c.map_or(0.0, |x| x.1) + l.map_or(0.0, |(_, own)| if own { 1.0 } else { 0.3 }))
}

/// +1: choose up to one target artifact, creature and land; untap the chosen ones you control, tap the chosen ones you
/// don't; you gain 2 life
fn teferi_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let (a, c, l) = teferi_aims(g, p);
    for aim in [a, c].into_iter().flatten() {
        match aim.0 {
            Aim::Untap(m) => g.perm_mut(m).tapped = false,
            Aim::Tap(m) => g.perm_mut(m).tapped = true,
        }
    }
    if let Some((l, own)) = l {
        g.land_mut(l).tapped = !own;
    }
    gain(g, p, 2)
}

/// -2: look at the top three cards, one into your hand, the rest on the bottom (the one the deck wants most)
fn teferi_minus_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    (g.perm(src).loyalty.unwrap_or(0) >= 4 && g.player(p).library.len() >= 3).then_some(2.2)
}

fn teferi_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let top = crate::ai::topdeck::look(g, p, 3);
    let w: Vec<f64> = top.iter().map(|&x| -crate::ai::topdeck::desire(g, p, x)).collect();
    let mut idx: Vec<usize> = (0..top.len()).collect();
    idx.sort_by(|&a, &b| w[a].partial_cmp(&w[b]).unwrap_or(std::cmp::Ordering::Equal));
    let top: Vec<CardId> = idx.into_iter().map(|i| top[i]).collect();
    let pl = g.player_mut(p);
    if let Some(&x) = top.first() {
        pl.hand.push(x);
        pl.seen.insert(x);
    }
    pl.library.splice(0..0, top.iter().skip(1).copied());
    Ok(())
}

/// -7: the emblem "untap all permanents you control during each opponent's untap step" and "you draw a card during
/// each opponent's draw step". The emblem is recorded but the engine does not act on it (engine-needs-legends2.md),
/// so the AI does not spend 7 loyalty on it (value None).
const TEFERI_EMBLEM: &str = "teferi_sunset";

fn teferi_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    g.player_mut(p).emblems.push(TEFERI_EMBLEM);
    Ok(())
}

static TEFERI: [WalkerAb; 3] = [
    WalkerAb {
        delta: 1,
        label: "untap / tap an artifact, creature and land; gain 2",
        val: teferi_plus_val,
        eff: teferi_plus,
    },
    WalkerAb { delta: -2, label: "look at three, take one", val: teferi_minus_val, eff: teferi_minus },
    WalkerAb { delta: -7, label: "emblem: untap and draw in their steps", val: |_, _, _| None, eff: teferi_ult },
];

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "The Queen of Dale")?;
    c.cast = Some(queen_cast);
    at_once(c, Event::Cast, false);
    let c = r.card(db, "Tataru Taru")?;
    c.etb = Some(tataru_etb);
    at_once(c, Event::Etb, false);
    c.draw = Some(tataru_draw);
    at_once(c, Event::Draw, false);
    c.upkeep = Some(tataru_upkeep);
    at_once(c, Event::Upkeep, true);
    c.leaves = Some(tataru_leaves);
    at_once(c, Event::Leaves, true);
    walker(r, db, "Teferi, Who Slows the Sunset", &TEFERI)?;
    r.creature_pt.push(magda_pt);
    let c = r.card(db, MAGDA)?;
    c.as_enters = Some(magda_as);
    c.mana_tapped = Some(magda_mana);
    c.attack = Some(magda_attack);
    at_once(c, Event::Attack, false);
    c.options = Some(magda_options);
    Ok(())
}
