//! Python's `cards/impl/t2.py`: the Tier 2 pool decks (Kaalia, Meren, Sythis, Brago, Lord Windgrace) and the staples
//! they share: the blink helpers (`blink`, `blink_value`, `flicker_worth`), `best_target_any` (Tier 3 and 4 cards use
//! it too), evoke and the monarch's Palace Jailer.

use super::common::{WalkerAb, best_opp_creature, best_opp_nonland, death_value, n_ench, oring_exile, oring_return};
use super::partials::{at_once, eot_kw, first_max, first_min, on, pack, remove_card, unpack};
use super::t1::count_type;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{castable, on_cast};
use crate::engine::hooks::fire_trigger;
use crate::engine::life::{gain, lose_life, prevents_damage};
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::turn::{crack_fetch, remove_land};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{epow, etgh, has_type, pval, threat, untargetable};
use crate::engine::zones::{
    Enter, Tokens, die, discard_cards, draw, enter, land_ramp, landfall, leave, make_tokens, max_by, min_by, searchable,
};
use crate::flow::Res;
use crate::hooks::{Action, Assign, Call, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, PermData, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

/// cardimpl._eot: +dp/+dt until end of turn
fn eot(g: &mut Game, m: PermId, dp: i32, dt: i32) {
    let x = &mut g.perm_mut(m).eot_pt;
    *x = (x.0 + dp, x.1 + dt);
}

/// `p.stats[name] += 1`
fn stat(g: &mut Game, p: PlayerId, name: &str) {
    g.player_mut(p).stat(intern(name), 1);
}

/// the card's subtypes include s (Python's `s in c.subtypes`)
fn card_sub(g: &Game, c: CardId, s: &str) -> bool {
    g.db.get(c).has_subtype(s)
}

/// a permanent's card name ("" for a token without a card)
fn cname(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

/// `c.bomb or c.pow`
fn bomb_or_pow(g: &Game, c: CardId) -> i32 {
    let d = g.db.get(c);
    if d.bomb != 0 { d.bomb } else { d.pow }
}

/// Python's `name in CI.HOOKS and 'etb' in CI.HOOKS[name]`: the card has an enters hook
fn has_etb_hook(g: &Game, c: CardId) -> bool {
    g.registry.get(c).is_some_and(|i| i.etb.is_some())
}

/// Python's `name in CI.HOOKS`: the card has code for some event
fn in_hooks(g: &Game, c: CardId) -> bool {
    g.registry.get(c).is_some_and(|i| i.live())
}

/// an activated ability of src for the AI's option list
fn ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

/// a card play for the AI's option list (from hand)
fn plan(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

const NONE_ID: u32 = u32::MAX;

// ------------------------------------------------------------------ shared
/// t2.best_target_any: 'deals n damage to any target': kill the best creature it kills, else the most threatening
/// opponent's face. HUMAN(phase 9): a person picks the target (hc.deal_damage).
pub fn best_target_any(g: &mut Game, p: PlayerId, n: i32) -> Res {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    if opps.is_empty() {
        return Ok(());
    }
    if let Some(&q) = opps.iter().find(|&&q| g.player(q).life <= n) {
        return lose_life(g, q, n, Some(p), "triggers", None);
    }
    let tg: Vec<PermId> = opps
        .iter()
        .flat_map(|&q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !untargetable(g, m) && etgh(g, m) <= n)
        .collect();
    let best = max_by(&tg, |m| pval(g, m));
    match best {
        Some(b) if pval(g, b) >= 3.0 => apply_removal(g, Some(p), b, &format!("dmg{n}"), None),
        _ => {
            let q = max_by(&opps, |q| threat(g, p, q)).unwrap();
            lose_life(g, q, n, Some(p), "triggers", None)
        }
    }
}

/// t2.blink: exile m and return it under its owner's control (ETBs again, untapped, summoning sick); the permanent
/// it came back as, None if it didn't (a token, or a blink chain three deep: those are combos, not loops here)
pub fn blink(g: &mut Game, p: PlayerId, m: PermId) -> Res<Option<PermId>> {
    let x = g.perm(m);
    if x.token || x.cd.is_none() || !x.on_bf {
        return Ok(None);
    }
    if g.blink_depth >= 3 {
        return Ok(None);
    }
    g.blink_depth += 1;
    let r = blink_inner(g, p, m);
    g.blink_depth -= 1;
    r.map(Some)
}

/// t2._blink
fn blink_inner(g: &mut Game, _p: PlayerId, m: PermId) -> Res<PermId> {
    let (cd, orig, cmd) = (g.perm(m).cd.unwrap(), g.perm(m).orig, g.perm(m).is_cmd);
    leave(g, m)?;
    let n = enter(g, orig, cd, Enter { orig: Some(orig), ..Enter::default() })?;
    g.perm_mut(n).is_cmd = cmd;
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::ExiledFromBf, Call::Leaves { m })?;
    }
    crate::glog!(g, "    {} is blinked", g.db.get(cd).name);
    Ok(n)
}

/// t2.blink_value: how much re-entering is worth for p's permanent m
pub fn blink_value(g: &Game, _p: PlayerId, m: PermId) -> i32 {
    let x = g.perm(m);
    let Some(cd) = x.cd else { return 0 };
    if x.token || x.is_cmd {
        return 0;
    }
    let mut v = crate::dsl::etb_value(g, cd);
    if has_etb_hook(g, cd) {
        v += 3;
    }
    if ["Oblivion Ring", "Banishing Light", "Detention Sphere", "Cast Out", "Journey to Nowhere"]
        .contains(&&*g.db.get(cd).name)
    {
        v = 0;
    }
    v
}

/// t2.flicker_worth: what flickering p's creature m is worth to p's AI: Sephiroth's values its own (mine.flicker_worth:
/// the commander Atraxa, Summon: Bahamut's restart, counters and Equipment lost); outside decks use blink_value
pub fn flicker_worth(g: &Game, p: PlayerId, m: PermId) -> f64 {
    if g.player(p).key == "seph" {
        return crate::cardcode::seph_flicker_worth(g, p, m);
    }
    blink_value(g, p, m) as f64
}

/// t2.end_step_flicker: Soulherder, Conjurer's Closet, Teleportation Circle: at the beginning of your end step, you
/// may exile a creature you control, then return it: the one with the best enters-the-battlefield effect. rocks
/// (Teleportation Circle, which takes an artifact too): with no creature worth it, a tapped mana rock, which comes
/// back untapped for the opponents' turns
pub fn end_step_flicker(g: &mut Game, src: PermId, p: PlayerId, other: bool, rocks: bool) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    let sname = cname(g, src).to_string();
    stat(g, p, &format!("flicker_chance {sname}"));
    let mut cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| (m != src || !other) && g.is_creature(m) && flicker_worth(g, p, m) > 0.0)
        .collect();
    if cands.is_empty() && rocks {
        cands = g
            .player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| {
                let x = g.perm(m);
                x.tapped
                    && !x.token
                    && x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Rock))
                    && !g.is_creature(m)
                    && x.orig == p
            })
            .collect();
    }
    if cands.is_empty() {
        return Ok(());
    }
    let what = if g.is_creature(cands[0]) { "blink a creature" } else { "blink a mana rock" };
    if !trigger_window(g, p, Some(src), what, None)? {
        return Ok(());
    }
    cands.retain(|&m| on(g, p, m));
    if cands.is_empty() {
        return Ok(());
    }
    let m = max_by(&cands, |m| if g.is_creature(m) { flicker_worth(g, p, m) } else { pval(g, m) }).unwrap();
    stat(g, p, &format!("flicker {sname}"));
    stat(g, p, &format!("flicker {sname} -> {}", g.perm(m).name));
    blink(g, p, m)?;
    Ok(())
}

// ======================================================== monarch, Palace Jailer
/// monarch (end-step draw, taken by combat damage); exiles a creature until an opponent becomes the monarch
fn jailer(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "become the monarch; exile a creature", Some(5.0))? {
        return Ok(());
    }
    crate::cardcode::become_monarch(g, o)?;
    if on(g, o, src) {
        oring_exile(g, src, o, |g, x| g.is_creature(x), false, false)?;
    }
    Ok(())
}

/// an opponent became the monarch: what the Jailer exiled returns
fn jailer_lose(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) {
        oring_return(g, src)?;
    }
    if !g.perm(src).data.is_empty() {
        g.perm_mut(src).data.set(DataKey::Oring, Val::List(vec![]));
    }
    Ok(())
}

// ======================================================== Brago and blink
/// combat damage: blink every nonland permanent with an ETB worth repeating (and tapped mana rocks)
fn brago(g: &mut Game, src: Src, _p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if a != src {
        return Ok(());
    }
    let o = owner(g, src);
    let ok = |g: &Game, m: PermId| {
        let x = g.perm(m);
        m != src
            && !x.token
            && x.cd.is_some_and(|c| !g.db.get(c).land)
            && (blink_value(g, o, m) > 0 || x.tapped && g.db.get(x.cd.unwrap()).tag(Tag::Rock))
    };
    if !g.player(o).perms.iter().any(|&m| ok(g, m)) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "blink nonland permanents", Some(4.0))? {
        return Ok(());
    }
    let ms: Vec<PermId> = g.player(o).perms.iter().copied().filter(|&m| ok(g, m)).collect();
    for m in ms {
        blink(g, o, m)?;
    }
    Ok(())
}

/// end step: blink the best ETB creature; grows when creatures are exiled
fn soulherder(g: &mut Game, src: Src, p: PlayerId) -> Res {
    end_step_flicker(g, src, p, true, false)
}

fn soulherder_grow(g: &mut Game, src: Src, m: PermId) -> Res {
    let o = owner(g, src);
    if g.is_creature(m) && trigger_window(g, o, Some(src), "a +1/+1 counter", Some(1.0))? {
        g.perm_mut(src).plus += 1;
    }
    Ok(())
}

/// end step: blink the best ETB creature
fn closet(g: &mut Game, src: Src, p: PlayerId) -> Res {
    end_step_flicker(g, src, p, false, false)
}

/// end step: blink the best ETB creature, else untap a tapped mana rock by blinking it
fn teleport_circle(g: &mut Game, src: Src, p: PlayerId) -> Res {
    end_step_flicker(g, src, p, false, true)
}

/// Whenever you cast a creature spell, exile up to one other target creature you control, then return that card to
/// the battlefield under its owner's control: the one with the best enter effect (the spell itself isn't on the
/// battlefield yet)
fn flickering_hound(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let p = owner(g, src);
    if caster != p || !g.db.get(c).creature || !on(g, p, src) || g.perm(src).phased {
        return Ok(());
    }
    stat(g, p, "flicker_chance Flickering Hound");
    let mut cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| m != src && g.is_creature(m) && flicker_worth(g, p, m) > 0.0)
        .collect();
    if cands.is_empty() || !trigger_window(g, p, Some(src), "blink a creature", None)? {
        return Ok(());
    }
    cands.retain(|&m| on(g, p, m));
    if cands.is_empty() {
        return Ok(());
    }
    let m = max_by(&cands, |m| flicker_worth(g, p, m)).unwrap();
    stat(g, p, "flicker Flickering Hound");
    stat(g, p, &format!("flicker Flickering Hound -> {}", g.perm(m).name));
    blink(g, p, m)?;
    Ok(())
}

/// t2._blink_option's costs and rule, by card: (generic, pips, other)
fn blink_option_spec(name: &str) -> (u32, &'static str, bool) {
    match name {
        "Eldrazi Displacer" => (3, "", true),
        _ => (1, "U", false), // Deadeye Navigator
    }
}

/// t2._blink_option: pay to blink your best ETB creature (Deadeye Navigator; Eldrazi Displacer's is rules2's)
fn blink_option(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let name = cname(g, src).to_string();
    let (cg, cp, other) = blink_option_spec(&name);
    if post.is_none() && !can_pay(g, p, cg, cp, false) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, cg, cp, false) {
        return Ok(vec![]);
    }
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && (m != src || !other) && blink_value(g, p, m) >= 3)
        .collect();
    let Some(t) = max_by(&cands, |m| blink_value(g, p, m) as f64) else { return Ok(vec![]) };
    let label = format!("{name}: blink {}", g.perm(t).name);
    Ok(vec![ability(0.5 + 0.5 * blink_value(g, p, t) as f64, label, src, blink_option_go, t.0 as i64)])
}

fn blink_option_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    let (cg, cp, _) = blink_option_spec(cname(g, src));
    if !on(g, p, t) || !can_pay(g, p, cg, cp, false) {
        return Ok(false);
    }
    pay(g, p, cg, cp, false)?;
    let label = format!("blink {}", g.perm(t).name);
    if ability_window(g, p, Some(src), &label, None, Some(t))? && on(g, p, t) {
        blink(g, p, t)?;
    }
    Ok(true)
}

// ======================================================== Kaalia of the Vast
const ADD: [&str; 3] = ["angel", "demon", "dragon"];

fn kaalia_ok(g: &Game, c: CardId) -> bool {
    g.db.get(c).creature && ADD.iter().any(|t| card_sub(g, c, t))
}

/// attacks: best Angel/Demon/Dragon from hand onto the battlefield attacking (ETBs fire)
fn kaalia(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if !atk.contains(&src) || owner(g, src) != p {
        return Ok(vec![]);
    }
    if !g.player(p).hand.iter().any(|&c| kaalia_ok(g, c)) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "put an Angel, Demon or Dragon onto the battlefield attacking", Some(5.0))? {
        return Ok(vec![]);
    }
    let cands: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| kaalia_ok(g, c)).collect();
    let Some(c) = first_max(&cands, |c| {
        let d = g.db.get(c);
        (d.bomb, d.pow, d.cmc)
    }) else {
        return Ok(vec![]);
    };
    remove_card(&mut g.player_mut(p).hand, c);
    let m = enter(g, p, c, Enter::default())?;
    let x = g.perm_mut(m);
    x.tapped = true;
    x.sick = false;
    crate::glog!(g, "    Kaalia puts {} onto the battlefield attacking", g.db.get(c).name);
    stat(g, p, "kaalia_cheats");
    Ok(vec![m])
}

/// t2.kaalia_prio: hold Angels, Demons and Dragons for Kaalia when she's ready to attack
pub fn kaalia_prio(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    let pl = g.player(p);
    let kaalia = pl.perms.iter().any(|&m| g.perm(m).is_cmd);
    let d = g.db.get(c);
    if d.creature && ADD.iter().any(|t| d.has_subtype(t)) && d.cmc >= 5 && (kaalia || pl.cmd_in_zone && pl.tax <= 2) {
        return Some(0);
    }
    None
}

/// damage equal to power when another creature enters; the targeting life tax is ignored
fn terror(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if m != src
        && owner(g, m) == o
        && g.is_creature(m)
        && epow(g, m) > 0
        && trigger_window(g, o, Some(src), &format!("{} damage to any target", epow(g, m)), Some(5.0))?
    {
        let n = epow(g, m);
        best_target_any(g, o, n)?;
    }
    Ok(())
}

/// Dragon ETB damage; firebreathing not used
fn valkas(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if owner(g, m) == o
        && (m == src || has_type(g, m, "dragon"))
        && trigger_window(g, o, Some(src), "damage to any target per Dragon", Some(5.0))?
    {
        let n = count_type(g, o, "dragon", false);
        best_target_any(g, o, n)?;
    }
    Ok(())
}

/// flying creatures gain haste; Dragons deal damage per Dragon
fn tempest(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if owner(g, m) != o || !g.is_creature(m) {
        return Ok(());
    }
    let fly = g.perm(m).fly || crate::dsl::has_kw(g, m, "flying");
    let drag = has_type(g, m, "dragon");
    if !(fly || drag) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "haste / damage per Dragon", Some(4.0))? {
        return Ok(());
    }
    if fly {
        g.perm_mut(m).sick = false;
    }
    if drag {
        let n = count_type(g, o, "dragon", false);
        best_target_any(g, o, n)?;
    }
    Ok(())
}

/// 4 + 3 + 3 damage split by the any-target heuristic
fn drakuseth(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src) && trigger_window(g, p, Some(src), "4, 3 and 3 damage to any targets", Some(6.0))? {
        best_target_any(g, p, 4)?;
        best_target_any(g, p, 3)?;
        best_target_any(g, p, 3)?;
    }
    Ok(vec![])
}

/// combat damage to a player: that much damage to each creature they control
fn balefire(g: &mut Game, src: Src, _p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    if a == src {
        let o = owner(g, src);
        let label = format!("{dmg} damage to each creature {} controls", pname(g, d));
        if trigger_window(g, o, Some(src), &label, Some(6.0))? {
            for m in g.player(d).perms.clone() {
                if g.is_creature(m) && etgh(g, m) <= dmg {
                    die(g, m, "destroy")?;
                }
            }
        }
    }
    Ok(())
}

/// attacking alone unblocked sets life to 1; the attack-alone rule is not forced on the AI
fn moc(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId, assign: &mut Assign) -> Res {
    if atk.contains(&src)
        && atk.len() == 1
        && !assign.iter().any(|&(a, _)| a == src)
        && g.player(d).life > 1
        && !prevents_damage(g, d, Some(p))
    {
        let label = format!("{}'s life total becomes 1", pname(g, d));
        if !trigger_window(g, p, Some(src), &label, Some(7.0))? || g.player(d).life <= 1 {
            return Ok(());
        }
        let n = g.player(d).life - 1;
        lose_life(g, d, n, Some(p), "triggers", None)?;
        let pw = epow(g, src);
        eot(g, src, -pw, 0);
    }
    Ok(())
}

/// landfall: exile a nonland permanent until it leaves
fn admonition(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == owner(g, src) && trigger_window(g, p, Some(src), "exile a nonland permanent", Some(5.0))? && on(g, p, src) {
        oring_exile(g, src, p, |_, m| m != src, false, false)?;
    }
    Ok(())
}

fn admonition_leave(g: &mut Game, src: Src, _m: PermId) -> Res {
    oring_return(g, src)
}

/// exiles up to three opposing creatures (graveyard targets not used); they return to hand when it leaves
fn serenity(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "exile up to three creatures", Some(6.0))? || !on(g, o, src) {
        return Ok(());
    }
    let src_cd = g.perm(src).cd;
    for _ in 0..3 {
        let cands: Vec<PermId> = g
            .opps(o)
            .flat_map(|q| g.player(q).perms.iter().copied())
            .filter(|&x| g.is_creature(x) && !untargetable(g, x))
            .collect();
        let Some(x) = max_by(&cands, |x| pval(g, x)) else { break };
        if pval(g, x) < 2.0 {
            break;
        }
        let xo = owner(g, x);
        apply_removal(g, Some(o), x, "exile", src_cd)?;
        let y = g.perm(x);
        if !on(g, xo, x) && !y.token && y.cd.is_some_and(|cd| g.player(y.orig).exile.contains(&cd)) {
            let entry = Val::List(vec![Val::Card(y.cd.unwrap()), Val::Player(y.orig)]);
            let data = &mut g.perm_mut(src).data;
            match data.get_mut(DataKey::Serenity) {
                Some(Val::List(v)) => v.push(entry),
                _ => data.set(DataKey::Serenity, Val::List(vec![entry])),
            }
        }
    }
    Ok(())
}

fn serenity_leave(g: &mut Game, src: Src, _m: PermId) -> Res {
    let Some(Val::List(v)) = g.perm(src).data.get(DataKey::Serenity).cloned() else { return Ok(()) };
    for e in v {
        let Val::List(pair) = e else { continue };
        let (Val::Card(cd), Val::Player(q)) = (&pair[0], &pair[1]) else { continue };
        let (cd, q) = (*cd, *q);
        let pl = g.player_mut(q);
        if remove_card(&mut pl.exile, cd) {
            pl.hand.push(cd);
        }
    }
    Ok(())
}

/// flash, ETB indestructible; the transform is not modeled
fn avacyn(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if m == src && trigger_window(g, o, Some(src), "your creatures gain indestructible", Some(5.0))? {
        for x in g.player(o).perms.clone() {
            if g.is_creature(x) {
                eot_kw(g, x, "indestructible");
            }
        }
    }
    Ok(())
}

/// opponents can't gain life
fn despair_nolife(g: &Game, src: Src, p: PlayerId) -> bool {
    p != owner(g, src)
}

/// each end step: opponents lose life equal to the life they lost this turn
fn despair(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    let st = g.turn_stamp();
    let o = owner(g, src);
    let lost = |g: &Game, q: PlayerId| g.player(q).lost_turn.is_some_and(|lt| lt.0 == st && lt.1 > 0);
    if !g.opps(o).any(|q| lost(g, q)) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "opponents lose life equal to life lost this turn", Some(5.0))? {
        return Ok(());
    }
    for q in g.opps(o).collect::<Vec<_>>() {
        if lost(g, q) {
            let n = g.player(q).lost_turn.unwrap().1;
            lose_life(g, q, n, Some(o), "drain", None)?;
        }
    }
    Ok(())
}

/// every spell costs its caster 2 life; the recast-for-life clause is ignored
fn liesa(g: &mut Game, src: Src, caster: PlayerId, _c: CardId) -> Res {
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), &format!("{} loses 2 life", pname(g, caster)), None)? {
        return Ok(());
    }
    lose_life(g, caster, 2, Some(if caster != o { o } else { caster }), "drain", None)
}

/// exerts for 4 damage to a creature every other attack
fn glorybringer(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let turns = g.player(p).turns as i64;
    if !atk.contains(&src) || matches!(g.perm(src).data.get(DataKey::Exerted), Some(Val::Int(t)) if *t == turns - 1) {
        return Ok(vec![]);
    }
    let cands: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !has_type(g, m, "dragon") && !untargetable(g, m) && etgh(g, m) <= 4)
        .collect();
    if let Some(best) = max_by(&cands, |m| pval(g, m))
        && pval(g, best) >= 2.0
    {
        let ok = trigger_window(g, p, Some(src), &format!("4 damage to {}", g.perm(best).name), Some(5.0))?;
        let mut data = PermData::default();
        data.set(DataKey::Exerted, Val::Int(turns));
        g.perm_mut(src).data = data;
        if ok && on(g, owner(g, best), best) {
            let cd = g.perm(src).cd;
            apply_removal(g, Some(p), best, "dmg4", cd)?;
        }
    }
    Ok(vec![])
}

/// 1 damage to each opposing flier, tapping them
fn thundermaw(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "1 damage to each opposing flier and tap them", Some(5.0))? {
        return Ok(());
    }
    for q in g.opps(o).collect::<Vec<_>>() {
        for x in g.player(q).perms.clone() {
            if g.is_creature(x) && (g.perm(x).fly || crate::dsl::has_kw(g, x, "flying")) {
                g.perm_mut(x).tapped = true;
                if etgh(g, x) <= 1 {
                    die(g, x, "destroy")?;
                }
            }
        }
    }
    Ok(())
}

/// mentor; +2/+0, trample and vigilance on the best attacker each combat
fn aurelia_ex(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    let ready = |g: &Game, m: PermId| g.is_creature(m) && !g.perm(m).noatk;
    if !g.player(p).perms.iter().any(|&m| ready(g, m)) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "a creature gets +2/+0, trample and vigilance", None)? {
        return Ok(());
    }
    let cr: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| ready(g, m)).collect();
    if let Some(t) = first_max(&cr, |m| (g.perm(m).is_cmd, epow(g, m))) {
        eot(g, t, 2, 0);
        eot_kw(g, t, "trample");
        eot_kw(g, t, "vigilance");
    }
    Ok(())
}

/// upkeep: draw a card, lose 1 life (above 10 life)
fn bloodgift(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == owner(g, src) && g.player(p).life > 10 && trigger_window(g, p, Some(src), "draw a card, lose 1 life", None)?
    {
        draw(g, p, 1, false)?;
        lose_life(g, p, 1, Some(p), "other", None)?;
    }
    Ok(())
}

/// nonland cards to hand until one with MV < 4 (the exiled lands are skipped)
fn belzenlok(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "put nonland cards into your hand", Some(4.0))? {
        return Ok(());
    }
    for _ in 0..4 {
        let Some(nl) = g.player(o).library.iter().rev().copied().find(|&c| !g.db.get(c).land) else { break };
        let pl = g.player_mut(o);
        remove_card(&mut pl.library, nl);
        pl.hand.push(nl);
        lose_life(g, o, 1, Some(o), "other", None)?;
        if g.db.get(nl).cmc < 4 {
            break;
        }
    }
    Ok(())
}

/// 4/4 Angel on 5+ life gained in a turn; the pump is not used
fn resplendent(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    let o = owner(g, src);
    let st = g.turn_stamp();
    if g.player(o).gained_turn.is_some_and(|gt| gt.0 == st && gt.1 >= 5)
        && trigger_window(g, o, Some(src), "create a 4/4 Angel", None)?
    {
        let spec =
            Tokens { fly: true, color: Some(Colors::from_letters("W")), types: vec!["angel"], ..Tokens::new(1, 4) };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

// ======================================================== Lord Windgrace (lands)
/// +2: discard a card (a land when it pays), draw one (two for a land)
fn wg_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if g.player(p).hand.is_empty() {
        return draw(g, p, 1, false);
    }
    let hand = g.player(p).hand.clone();
    let lands: Vec<CardId> = hand.iter().copied().filter(|&c| g.db.get(c).land).collect();
    let c = if !lands.is_empty()
        && (lands.len() >= 2 || g.player(p).gy.iter().any(|&x| g.db.get(x).land) || g.player(p).lands.len() >= 5)
    {
        lands[0]
    } else {
        min_by(&hand, |c| card_worth(g, p, c, false)).unwrap()
    };
    discard_cards(g, p, &[c])?;
    let land = g.db.get(c).land;
    draw(g, p, if land { 2 } else { 1 }, false)?;
    if !g.hooks.is_empty() && land {
        fire_trigger(g, Event::LandGy, Call::Cards { p, cards: vec![c] })?;
    }
    Ok(())
}

/// -3: two land cards from your graveyard onto the battlefield (the most colours first, then fetch lands)
fn wg_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let mut ls: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).land).collect();
    let key = |c: CardId| {
        let t = &g.db.get(c).tags;
        (-(t.str(Tag::C).map_or(0, |s| s.chars().count()) as i64), !t.has(Tag::F))
    };
    ls.sort_by_key(|&c| key(c));
    ls.truncate(2);
    for c in ls {
        remove_card(&mut g.player_mut(p).gy, c);
        let l = g.add_land(p, c, false);
        landfall(g, p)?;
        if g.db.get(c).tag(Tag::F) && g.player(p).lands.last() == Some(&l) {
            crack_fetch(g, p, l)?;
        }
    }
    Ok(())
}

/// -11: destroy up to six nonland permanents, six 2/2 Cat Warriors
fn wg_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    for _ in 0..6 {
        let Some(t) = best_opp_nonland(g, p, |_| true) else { break };
        apply_removal(g, Some(p), t, "destroy", None)?;
    }
    let spec = Tokens { color: Some(Colors::from_letters("G")), types: vec!["cat", "warrior"], ..Tokens::new(6, 2) };
    make_tokens(g, p, spec)?;
    Ok(())
}

fn gy_lands(g: &Game, p: PlayerId) -> usize {
    g.player(p).gy.iter().filter(|&&c| g.db.get(c).land).count()
}

/// loyalty abilities and ultimate (forestwalk on the Cats ignored)
pub const WINDGRACE: [WalkerAb; 3] = [
    WalkerAb { delta: 2, label: "discard, draw", val: |_, _, _| Some(2.5), eff: wg_plus },
    WalkerAb {
        delta: -3,
        label: "two lands back",
        val: |g, p, _| {
            let n = gy_lands(g, p);
            if n >= 1 { Some(1.5 + 1.5 * n.min(2) as f64) } else { None }
        },
        eff: wg_minus,
    },
    WalkerAb { delta: -11, label: "ultimate", val: |_, _, _| Some(12.0), eff: wg_ult },
];
static WINDGRACE_ABS: [WalkerAb; 3] = WINDGRACE;

/// main-phase landfall: an additional combat (the per-attack +1/+0 is not modeled)
fn moraug(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if owner(g, src) == p
        && g.active == Some(p)
        && g.player(p).combat_no < 1
        && g.player(p).extra_combats < 3
        && trigger_window(g, p, Some(src), "an additional combat phase", Some(5.0))?
        && g.player(p).extra_combats < 3
    {
        g.player_mut(p).extra_combats += 1;
    }
    Ok(())
}

/// landfall: a 5/5 Elemental
fn omnath(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if owner(g, src) == p && trigger_window(g, p, Some(src), "create a 5/5 Elemental", None)? {
        let spec = Tokens { color: Some(Colors::from_letters("RG")), types: vec!["elemental"], ..Tokens::new(1, 5) };
        make_tokens(g, p, spec)?;
    }
    Ok(())
}

/// an Elemental of yours dies (or Omnath): 3 damage to any target
fn omnath_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, src);
    if (owner(g, m) == o && g.is_creature(m) && has_type(g, m, "elemental") || m == src)
        && trigger_window(g, o, Some(src), "3 damage to any target", Some(4.0))?
    {
        best_target_any(g, o, 3)?;
    }
    Ok(())
}

/// +1/+1 per land on the battlefield and in the graveyard (common.SELF_PT)
fn multani_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    let k = g.player(p).lands.len() as i32 + gy_lands(g, p) as i32;
    (k, k)
}

/// its own power/toughness rule is on
fn selfpt_on(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
    }
    Ok(())
}

/// upkeep: sacrifice it unless you sacrifice a land (a tapped one, the fewest colours)
fn gitrog_up(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "sacrifice it unless you sacrifice a land", None)? || !on(g, p, src) {
        return Ok(());
    }
    if g.player(p).lands.len() >= 4 {
        let lands = g.player(p).lands.clone();
        let l = first_min(&lands, |l| {
            let land = g.land(l);
            (!land.tapped, g.db.get(land.cd).tags.str(Tag::C).map_or(0, |s| s.chars().count()))
        })
        .unwrap();
        let cd = g.land(l).cd;
        remove_land(g, p, l);
        g.player_mut(p).gy.push(cd);
        fire_trigger(g, Event::LandGy, Call::Cards { p, cards: vec![cd] })?;
    } else {
        die(g, src, "sac")?;
    }
    Ok(())
}

/// a land card of yours went to the graveyard: draw
fn gitrog_draw(g: &mut Game, src: Src, p: PlayerId, _cd: CardId) -> Res {
    if p == owner(g, src) && g.player(p).library.len() > 8 && trigger_window(g, p, Some(src), "draw a card", None)? {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

/// an extra land drop
fn own_one(g: &Game, src: Src, p: PlayerId) -> i32 {
    (p == owner(g, src)) as i32
}

/// you discarded a land: draw
fn gitrog_discard(g: &mut Game, src: Src, q: PlayerId, c: CardId) -> Res {
    if q == owner(g, src)
        && g.db.get(c).land
        && g.player(q).library.len() > 8
        && trigger_window(g, q, Some(src), "draw a card", None)?
    {
        draw(g, q, 1, false)?;
    }
    Ok(())
}

/// landfall: gain 1 life
fn courser_life(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == owner(g, src) && trigger_window(g, p, Some(src), "gain 1 life", Some(1.0))? {
        gain(g, p, 1)?;
    }
    Ok(())
}

/// Mulch: the top four, lands to hand, the rest to the graveyard
fn mulch(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = 4.min(g.player(p).library.len());
    let pl = g.player_mut(p);
    let top: Vec<CardId> = (0..n).map(|_| pl.library.pop().unwrap()).collect();
    for x in top {
        if g.db.get(x).land {
            g.player_mut(p).hand.push(x);
        } else {
            g.player_mut(p).gy.push(x);
        }
    }
    Ok("gy")
}

fn mulch_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    40
}

/// kicked when {2} more is available
fn grow(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = if can_pay(g, p, 2, "", false) { 2 } else { 1 };
    if n == 2 {
        pay(g, p, 2, "", false)?;
    }
    land_ramp(g, p, n, false)?;
    Ok("gy")
}

fn grow_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.player(p).turns <= 6 { 60 } else { 30 }
}

/// two basic-typed lands tapped; the Desert Zombies are not modeled
fn hour(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    land_ramp(g, p, 2, true)?;
    Ok("gy")
}

fn hour_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.player(p).turns <= 8 { 55 } else { 30 }
}

// ======================================================== Meren of Clan Nel Toth (recursion)
/// another creature of yours dies: an experience counter
fn meren_xp(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, src);
    if owner(g, m) == o
        && m != src
        && g.is_creature(m)
        && trigger_window(g, o, Some(src), "get an experience counter", Some(2.0))?
    {
        g.player_mut(o).experience += 1;
    }
    Ok(())
}

/// end step: best creature back (battlefield if its MV fits the experience, else hand)
fn meren_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    if !g.player(p).gy.iter().any(|&c| g.db.get(c).creature) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "return a creature card from your graveyard", Some(4.0))? {
        return Ok(());
    }
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    if cs.is_empty() {
        return Ok(());
    }
    let xp = g.player(p).experience;
    let ok: Vec<CardId> = cs.iter().copied().filter(|&c| g.db.get(c).cmc as i32 <= xp).collect();
    if !ok.is_empty() {
        let c = first_max(&ok, |c| {
            let v = crate::dsl::etb_value(g, c) + bomb_or_pow(g, c) + if in_hooks(g, c) { 3 } else { 0 };
            (v, g.db.get(c).cmc)
        })
        .unwrap();
        remove_card(&mut g.player_mut(p).gy, c);
        enter(g, p, c, Enter::default())?;
        crate::glog!(g, "    Meren returns {} to the battlefield", g.db.get(c).name);
    } else {
        let c = max_by(&cs, |c| card_worth(g, p, c, false)).unwrap();
        let pl = g.player_mut(p);
        remove_card(&mut pl.gy, c);
        pl.hand.push(c);
    }
    Ok(())
}

/// t2.EVOKE: the evoke cost of each card with one
fn evoke_cost(name: &str) -> Option<(u32, &'static str)> {
    match name {
        "Shriekmaw" => Some((1, "B")),
        "Mulldrifter" => Some((2, "U")),
        _ => None,
    }
}

/// t2.evoke_options: evoke Shriekmaw or Mulldrifter when the full cost is out of reach (the outside decks' card plays)
pub fn evoke_options(g: &mut Game, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut o = vec![];
    if post.is_none() {
        return Ok(o);
    }
    for c in g.player(p).hand.clone() {
        let name = g.db.get(c).name.to_string();
        let Some((gen_, pips)) = evoke_cost(&name) else { continue };
        if !castable(g, p, c, "hand") {
            continue;
        }
        let (fg, fp) = cost_of(g, p, c);
        if can_pay(g, p, fg, &fp, false) || !can_pay(g, p, gen_, pips, false) {
            continue;
        }
        let u = if name == "Shriekmaw" {
            let t = best_opp_creature(g, p, |m| {
                !g.perm(m).cd.is_some_and(|cd| {
                    let d = g.db.get(cd);
                    d.types.has(Types::ARTIFACT) || d.pips.contains('B')
                })
            });
            match t {
                Some(t) if pval(g, t) >= 3.0 => pval(g, t) - 2.5,
                _ => continue,
            }
        } else {
            2.5
        };
        o.push(plan(u, format!("evoke {name}"), evoke_go, c.0 as i64));
    }
    Ok(o)
}

fn evoke_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let (gen_, pips) = evoke_cost(&g.db.get(c).name).unwrap();
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, gen_, pips, false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, gen_, pips, false)?;
    g.player_mut(p).spells_this_turn += 1;
    on_cast(g, p, c)?;
    crate::glog!(g, "  {} evokes {}", pname(g, p), g.db.get(c).name);
    let m = enter(g, p, c, Enter { was_cast: true, ..Enter::default() })?;
    if on(g, p, m) {
        die(g, m, "sac")?;
    }
    Ok(true)
}

/// discards the worst creature (or one Meren can return) for the best one
fn survival(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !can_pay(g, p, 0, "G", false) {
        return Ok(vec![]);
    }
    let disc: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    let lib: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| g.db.get(c).creature).collect();
    if disc.is_empty() || lib.is_empty() {
        return Ok(vec![]);
    }
    let mut d = min_by(&disc, |c| card_worth(g, p, c, false)).unwrap();
    let reanim = g.player(p).perms.iter().any(|&m| cname(g, m) == "Meren of Clan Nel Toth");
    if reanim {
        let xp = g.player(p).experience as i64;
        d = first_max(&disc, |c| {
            let cmc = g.db.get(c).cmc as i64;
            if cmc <= xp { cmc } else { -1 }
        })
        .unwrap();
    }
    let name = crate::ai::tutor_pick(g, p, "cre"); // the wish list first, then impact
    let want = match lib.iter().copied().find(|&c| Some(c) == name) {
        Some(c) => c,
        None => max_by(&lib, |c| crate::ai::tutor_value(g, p, c)).unwrap(),
    };
    let u = 1.5 + card_worth(g, p, want, false) / 30.0;
    Ok(vec![ability(u, "Survival of the Fittest".into(), src, survival_go, pack(d.0 as u32, want.0 as u32))])
}

fn survival_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (d, want) = unpack(arg);
    let (d, want) = (CardId(d as u16), CardId(want as u16));
    if !g.player(p).hand.contains(&d) || !can_pay(g, p, 0, "G", false) {
        return Ok(false);
    }
    pay(g, p, 0, "G", false)?;
    discard_cards(g, p, &[d])?;
    if !ability_window(g, p, Some(src), "search for a creature", None, None)? {
        return Ok(true);
    }
    if g.player(p).library.contains(&want) {
        let pl = g.player_mut(p);
        remove_card(&mut pl.library, want);
        pl.hand.push(want);
        shuffle_library(g, p);
        g.player_mut(p).stat("tutored", 1);
    }
    Ok(true)
}

/// sacrifice a creature for one with MV one higher, when it upgrades
fn pod(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if post.is_none() || x.tapped || x.sick || !can_pay(g, p, 1, "G", false) {
        return Ok(vec![]);
    }
    let fod: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && !g.perm(m).token && g.perm(m).cd.is_some() && !g.perm(m).is_cmd)
        .collect();
    let lib = searchable(g, p);
    let mut best: Option<(f64, PermId, CardId)> = None;
    for m in fod {
        let cmc = g.db.get(g.perm(m).cd.unwrap()).cmc;
        let up: Vec<CardId> =
            lib.iter().copied().filter(|&c| g.db.get(c).creature && g.db.get(c).cmc == cmc + 1).collect();
        if let Some(c) = max_by(&up, |c| card_worth(g, p, c, false)) {
            let gain_ = card_worth(g, p, c, false) / 10.0 - pval(g, m) + 1.0;
            if best.is_none_or(|b| gain_ > b.0) {
                best = Some((gain_, m, c));
            }
        }
    }
    let Some((v, m, c)) = best.filter(|b| b.0 >= 0.5) else { return Ok(vec![]) };
    let label = format!("Birthing Pod {} -> {}", g.perm(m).name, g.db.get(c).name);
    Ok(vec![ability(1.5 + v, label, src, pod_go, pack(m.0, c.0 as u32))])
}

fn pod_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (m, c) = unpack(arg);
    let (m, c) = (PermId(m), CardId(c as u16));
    if !on(g, p, m) || !g.player(p).library.contains(&c) || g.perm(src).tapped || !can_pay(g, p, 1, "G", false) {
        return Ok(false);
    }
    pay(g, p, 1, "G", false)?;
    g.perm_mut(src).tapped = true;
    die(g, m, "sac")?;
    let label = format!("search for {}", g.db.get(c).name);
    if !ability_window(g, p, Some(src), &label, Some(6.0), None)? || !g.player(p).library.contains(&c) {
        return Ok(true);
    }
    remove_card(&mut g.player_mut(p).library, c);
    shuffle_library(g, p);
    enter(g, p, c, Enter::default())?;
    Ok(true)
}

fn leap_fodder(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && (g.perm(m).token || pval(g, m) < 2.0) && !g.perm(m).is_cmd)
        .collect()
}

/// sacrifice spare creatures for the next creature card
fn leap(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !can_pay(g, p, 0, "G", false) {
        return Ok(vec![]);
    }
    let fod = leap_fodder(g, p);
    if fod.is_empty() || !g.player(p).library.iter().any(|&c| g.db.get(c).creature) {
        return Ok(vec![]);
    }
    let m = min_by(&fod, |x| pval(g, x)).unwrap();
    let u = 2.0 + death_value(g, p, None) / 2.0;
    Ok(vec![ability(u, "Evolutionary Leap".into(), src, leap_go, m.0 as i64)])
}

fn leap_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    if !on(g, p, m) || !can_pay(g, p, 0, "G", false) {
        return Ok(false);
    }
    pay(g, p, 0, "G", false)?;
    die(g, m, "sac")?;
    if !ability_window(g, p, Some(src), "reveal until a creature", None, None)? {
        return Ok(true);
    }
    let lib = &g.player(p).library;
    if let Some(i) = (0..lib.len()).rev().find(|&i| g.db.get(lib[i]).creature) {
        let pl = g.player_mut(p);
        let c = pl.library.remove(i);
        pl.hand.push(c);
    }
    Ok(true)
}

/// sacrifice a creature: the two best creature cards return tapped
fn victimize(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let fod: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).is_cmd).collect();
    let Some(f) = min_by(&fod, |m| pval(g, m)) else { return Ok("gy") };
    die(g, f, "sac")?;
    let mut cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&x| g.db.get(x).creature).collect();
    let key = |x: CardId| -(bomb_or_pow(g, x) as f64 + g.db.get(x).cmc as f64 * 0.1);
    cs.sort_by(|a, b| key(*a).partial_cmp(&key(*b)).unwrap_or(std::cmp::Ordering::Equal));
    cs.truncate(2);
    for x in cs {
        remove_card(&mut g.player_mut(p).gy, x);
        let m = enter(g, p, x, Enter::default())?;
        g.perm_mut(m).tapped = true;
    }
    Ok("gy")
}

fn victimize_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let pl = g.player(p);
    let big = pl.gy.iter().filter(|&&x| g.db.get(x).creature && bomb_or_pow(g, x) >= 4).count() >= 1;
    let fodder = pl.perms.iter().any(|&m| g.is_creature(m) && (g.perm(m).token || pval(g, m) < 2.5));
    if big && fodder { 58 } else { 0 }
}

// ======================================================== Sythis, Harvest's Hand (enchantress)
/// t2._constellation: an enchantment entering under your control draws a card
fn constellation(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if owner(g, m) == o
        && g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ENCHANTMENT))
        && trigger_window(g, o, Some(src), "constellation: draw a card", None)?
    {
        if cname(g, src) == "Setessan Champion" {
            g.perm_mut(src).plus += 1; // and a +1/+1 counter
        }
        draw(g, o, 1, false)?;
    }
    Ok(())
}

/// Spirit Cleric per enchantment cast (P/T = Spirits you control)
fn haunting(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster == owner(g, src) && g.db.get(c).types.has(Types::ENCHANTMENT) {
        g.selfpt = true;
        if trigger_window(g, caster, Some(src), "create a Spirit Cleric", None)? {
            let spec =
                Tokens { color: Some(Colors::from_letters("W")), types: vec!["spirit", "cleric"], ..Tokens::new(1, 1) };
            make_tokens(g, caster, spec)?;
        }
    }
    Ok(())
}

/// flying and vigilance at seven enchantments
fn haunting_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    let o = owner(g, src);
    (kw == "flying" || kw == "vigilance") && owner(g, m) == o && g.is_creature(m) && n_ench(g, o) >= 7
}

/// returns an enchantment each upkeep; animating enchantments at five is not modeled
fn starfield(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    let ok = |g: &Game, c: CardId| g.db.get(c).types.has(Types::ENCHANTMENT) && !card_sub(g, c, "aura");
    if !g.player(p).gy.iter().any(|&c| ok(g, c))
        || !trigger_window(g, p, Some(src), "return an enchantment from your graveyard", Some(4.0))?
    {
        return Ok(());
    }
    let es: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| ok(g, c)).collect();
    if let Some(c) = max_by(&es, |c| card_worth(g, p, c, false)) {
        remove_card(&mut g.player_mut(p).gy, c);
        enter(g, p, c, Enter::default())?;
    }
    Ok(())
}

/// mana equal to an opponent's Islands once each turn (precombat)
fn carpet(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    let x = g
        .opps(p)
        .map(|q| {
            g.player(q)
                .lands
                .iter()
                .filter(|&&l| {
                    let d = g.db.get(g.land(l).cd);
                    d.has_subtype("island") || &*d.name == "Island"
                })
                .count()
        })
        .max()
        .unwrap_or(0) as u32;
    if x != 0 && trigger_window(g, p, Some(src), &format!("add {x} mana"), Some(2.0))? {
        g.player_mut(p).floating.any += x;
    }
    Ok(())
}

/// blinks your best ETB creature (immediately, not at end step), else 3 life.
/// Not registered: in Python rules2.py's enters hook replaces this one, so it never runs.
#[allow(dead_code)]
fn prince(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "blink a creature or gain 3 life", None)? {
        return Ok(());
    }
    let cands: Vec<PermId> = g
        .player(o)
        .perms
        .iter()
        .copied()
        .filter(|&x| x != src && g.is_creature(x) && blink_value(g, o, x) >= 3)
        .collect();
    match max_by(&cands, |x| blink_value(g, o, x) as f64) {
        Some(t) => {
            blink(g, o, t)?;
        }
        None => gain(g, o, 3)?,
    }
    Ok(())
}

// ======================================================== value blinks: Ephemerate, Restoration Angel (pool decks)
/// t2.STYLE_KEYS: your decks with their own blink play (they cast these by their own AI)
const STYLE_KEYS: [&str; 4] = ["seph", "veyran", "sauron", "najeela"];

fn value_blink_cost(name: &str) -> (u32, &'static str) {
    if name == "Ephemerate" { (0, "W") } else { (3, "W") }
}

/// t2._value_blink: cast at the end of an opponent's turn for value: blink your best enters-the-battlefield creature
/// (Ephemerate rebounds; Restoration Angel also arrives as a 3/4 flash flier). Kept for protection when nothing is
/// worth it. Registered for Restoration Angel; Ephemerate's hand option is mine.py's (it replaces this one).
fn value_blink(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let name = g.db.get(c).name.to_string();
    let (gen_, pips) = value_blink_cost(&name);
    let resto = name == "Restoration Angel";
    if post.is_some()
        || g.active == Some(p)
        || !g.player(p).hand.contains(&c)
        || STYLE_KEYS.contains(&g.player(p).key)
        || !can_pay(g, p, gen_, pips, false)
    {
        return Ok(vec![]);
    }
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            g.is_creature(m) && !g.perm(m).token && blink_value(g, p, m) >= 3 && !(resto && has_type(g, m, "angel"))
        })
        .collect();
    let t = max_by(&cands, |m| blink_value(g, p, m) as f64);
    if t.is_none() && !resto {
        return Ok(vec![]);
    }
    let v = t.map_or(0, |t| blink_value(g, p, t)) as f64 + if resto { 2.5 } else { 0.0 };
    if v < 3.0 {
        return Ok(vec![]);
    }
    let arg = pack(c.0 as u32, t.map_or(NONE_ID, |t| t.0));
    Ok(vec![plan(v - 1.0, format!("{name} (end of turn)"), value_blink_go, arg)])
}

fn value_blink_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, t) = unpack(arg);
    let c = CardId(c as u16);
    let t = (t != NONE_ID).then_some(PermId(t));
    let name = g.db.get(c).name.to_string();
    let (gen_, pips) = value_blink_cost(&name);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, gen_, pips, false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, gen_, pips, false)?;
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.cast_names.insert(c);
    on_cast(g, p, c)?;
    if name == "Ephemerate" {
        let pl = g.player_mut(p);
        pl.exile.push(c);
        pl.rebound.push(c);
        if let Some(t) = t
            && on(g, p, t)
        {
            blink(g, p, t)?;
        }
    } else {
        g.resto_target = t; // its enters trigger blinks this one
        let r = enter(g, p, c, Enter::default());
        g.resto_target = None;
        r?;
    }
    match t {
        Some(t) => crate::glog!(g, "  {} casts {name} at end of turn: blinks {}", pname(g, p), g.perm(t).name),
        None => crate::glog!(g, "  {} casts {name} at end of turn", pname(g, p)),
    }
    Ok(true)
}

/// when it enters: you may exile target non-Angel creature you control, then return it. HUMAN(phase 9): a person
/// picks the creature (or none).
fn resto_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let ok = |g: &Game, x: PermId| g.is_creature(x) && !g.perm(x).phased && x != src && !has_type(g, x, "angel");
    if !g.player(o).perms.iter().any(|&x| ok(g, x)) {
        return Ok(());
    }
    // the AI only puts it on the stack with a worthwhile blink
    let t = g.resto_target;
    if t.is_none_or(|t| !ok(g, t) || !on(g, o, t)) {
        let best = g
            .player(o)
            .perms
            .iter()
            .filter(|&&x| ok(g, x))
            .map(|&x| flicker_worth(g, o, x))
            .fold(None, |a: Option<f64>, v| Some(a.map_or(v, |a| if v > a { v } else { a })))
            .unwrap_or(0.0);
        if best < 3.0 {
            return Ok(());
        }
    }
    if !trigger_window(g, o, Some(src), "blink a non-Angel creature", Some(4.0))? {
        return Ok(());
    }
    let cands: Vec<PermId> = g.player(o).perms.iter().copied().filter(|&x| ok(g, x)).collect();
    if cands.is_empty() {
        return Ok(());
    }
    let mut t = g.resto_target;
    if t.is_none_or(|t| !cands.contains(&t)) {
        let best = max_by(&cands, |x| flicker_worth(g, o, x));
        t = best.filter(|&b| flicker_worth(g, o, b) >= 3.0);
    }
    if let Some(t) = t
        && on(g, o, t)
    {
        stat(g, o, "flicker Restoration Angel");
        stat(g, o, &format!("flicker Restoration Angel -> {}", g.perm(t).name));
        if g.perm(t).token {
            leave(g, t)?;
            crate::glog!(g, "    {} (a token) is exiled for good", g.perm(t).name);
        } else {
            blink(g, o, t)?;
        }
    }
    Ok(())
}

// ======================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    // monarch, Palace Jailer
    let c = r.card(db, "Palace Jailer")?;
    c.etb = Some(jailer);
    c.monarch = Some(jailer_lose);
    // Brago and blink
    r.card(db, "Brago, King Eternal")?.combat_damage = Some(brago);
    let c = r.card(db, "Soulherder")?;
    c.end_step = Some(soulherder);
    at_once(c, Event::EndStep, true); // the hook itself has no trigger_window (end_step_flicker's is its own round)
    c.exiled_from_bf = Some(soulherder_grow);
    let c = r.card(db, "Conjurer's Closet")?;
    c.end_step = Some(closet);
    at_once(c, Event::EndStep, true);
    let c = r.card(db, "Teleportation Circle")?;
    c.end_step = Some(teleport_circle);
    at_once(c, Event::EndStep, true);
    r.card(db, "Flickering Hound")?.cast = Some(flickering_hound);
    // Eldrazi Displacer: rules2's options hook replaces t2's (blink_option)
    r.card(db, "Deadeye Navigator")?.options = Some(blink_option);
    // Kaalia of the Vast
    r.card(db, "Kaalia of the Vast")?.attack = Some(kaalia);
    r.card(db, "Terror of the Peaks")?.etb = Some(terror);
    r.card(db, "Scourge of Valkas")?.etb = Some(valkas);
    r.card(db, "Dragon Tempest")?.etb = Some(tempest);
    r.card(db, "Drakuseth, Maw of Flames")?.attack = Some(drakuseth);
    r.card(db, "Balefire Dragon")?.combat_damage = Some(balefire);
    r.card(db, "Master of Cruelties")?.blocks = Some(moc);
    let c = r.card(db, "Admonition Angel")?;
    c.landfall = Some(admonition);
    c.leaves = Some(admonition_leave);
    at_once(c, Event::Leaves, true);
    let c = r.card(db, "Angel of Serenity")?;
    c.etb = Some(serenity);
    c.leaves = Some(serenity_leave);
    at_once(c, Event::Leaves, true);
    r.card(db, "Archangel Avacyn // Avacyn, the Purifier")?.etb = Some(avacyn);
    let c = r.card(db, "Archfiend of Despair")?;
    c.no_lifegain = Some(despair_nolife);
    c.end_step = Some(despair);
    r.card(db, "Liesa, Shroud of Dusk")?.cast = Some(liesa);
    r.card(db, "Glorybringer")?.attack = Some(glorybringer);
    r.card(db, "Thundermaw Hellkite")?.etb = Some(thundermaw);
    r.card(db, "Aurelia, Exemplar of Justice")?.upkeep = Some(aurelia_ex);
    r.card(db, "Bloodgift Demon")?.upkeep = Some(bloodgift);
    r.card(db, "Demonlord Belzenlok")?.etb = Some(belzenlok);
    r.card(db, "Resplendent Angel")?.end_step = Some(resplendent);
    // Lord Windgrace (lands)
    super::common::walker(r, db, "Lord Windgrace", &WINDGRACE_ABS)?;
    r.card(db, "Moraug, Fury of Akoum")?.landfall = Some(moraug);
    let c = r.card(db, "Omnath, Locus of Rage")?;
    c.landfall = Some(omnath);
    c.dies = Some(omnath_dies);
    let c = r.card(db, "Multani, Yavimaya's Avatar")?;
    c.self_pt = Some(multani_pt);
    c.etb = Some(selfpt_on);
    at_once(c, Event::Etb, true);
    let c = r.card(db, "The Gitrog Monster")?;
    c.upkeep = Some(gitrog_up);
    c.land_gy = Some(gitrog_draw);
    c.extra_lands = Some(own_one);
    c.discard = Some(gitrog_discard);
    let c = r.card(db, "Courser of Kruphix")?;
    c.lands_from_top = Some(own_one);
    c.landfall = Some(courser_life);
    r.card(db, "Conduit of Worlds")?.lands_from_gy = Some(own_one);
    let c = r.card(db, "Mulch")?;
    c.resolve = Some(mulch);
    c.prio = Some(mulch_prio);
    let c = r.card(db, "Grow from the Ashes")?;
    c.resolve = Some(grow);
    c.prio = Some(grow_prio);
    let c = r.card(db, "Hour of Promise")?;
    c.resolve = Some(hour);
    c.prio = Some(hour_prio);
    // Meren of Clan Nel Toth (recursion)
    let c = r.card(db, "Meren of Clan Nel Toth")?;
    c.dies = Some(meren_xp);
    c.end_step = Some(meren_end);
    r.card(db, "Survival of the Fittest")?.options = Some(survival);
    r.card(db, "Birthing Pod")?.options = Some(pod);
    r.card(db, "Evolutionary Leap")?.options = Some(leap);
    let c = r.card(db, "Victimize")?;
    c.resolve = Some(victimize);
    c.prio = Some(victimize_prio);
    // Sythis, Harvest's Hand (enchantress)
    r.card(db, "Setessan Champion")?.etb = Some(constellation);
    r.card(db, "Eidolon of Blossoms")?.etb = Some(constellation);
    let c = r.card(db, "Hallowed Haunting")?;
    c.cast = Some(haunting);
    c.grant_kw = Some(haunting_kw);
    r.card(db, "Starfield of Nyx")?.upkeep = Some(starfield);
    r.card(db, "Carpet of Flowers")?.upkeep = Some(carpet);
    // Charming Prince: rules2's enters hook replaces t2's (prince)
    // value blinks: Ephemerate's hand option is mine.py's (it replaces t2's value_blink)
    let c = r.card(db, "Restoration Angel")?;
    c.etb = Some(resto_etb);
    c.hand_options = Some(value_blink);
    Ok(())
}
