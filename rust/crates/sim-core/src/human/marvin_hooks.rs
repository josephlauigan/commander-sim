//! Practice mode (model-gaps-a4, item 2): the person's own path for the activated abilities that were only the AI's
//! `options` hooks until now (the AI picks the target, the sacrifice, the card). A person using such an ability
//! chooses what the Oracle text lets them choose, with the cost paid from the pool. Each function takes the source
//! `m`: the creature itself, or Marvin, Murderous Mimic using it (`model_gaps_a4::marvin_abilities` lists them for
//! Marvin with the creature's own list, so the questions are the same).
//!
//! Creatures whose hook offers no choice (Allosaurus Shepherd, Battle Cry Goblin, Bristly Bill ...) stay hooks.

use super::abilities::{PermAbility, Why, ability};
use super::cards::{creatures, pick_perm};
use super::{Target, choices, legal, mana};
use crate::engine::values::{epow, etgh, has_type, untargetable};
use crate::engine::{removal::apply_removal, stack, zones};
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};

// ------------------------------------------------------------------ helpers
pub(super) fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn say(s: impl Into<String>) -> Res<Why> {
    Ok(Some(s.into()))
}

fn cost(g: &Game, p: PlayerId, gn: u32, pips: &str, what: &str) -> Why {
    mana::cost_problem(g, p, gn, pips).map(|why| format!("Can't activate {what}. {why}"))
}

fn pay(g: &mut Game, p: PlayerId, gn: u32, pips: &str, what: &str) -> Why {
    mana::pay_from_pool(g, p, gn, pips).map(|why| format!("Can't activate {what}. {why}"))
}

/// why m can't pay a {T} cost now (tapped, or summoning sick without haste)
fn tap_problem(g: &Game, p: PlayerId, m: PermId) -> Why {
    let x = g.perm(m);
    if !live(g, m) {
        return Some(format!("{} is not on the battlefield.", x.name));
    }
    if x.tapped {
        return Some(format!("{} is tapped.", x.name));
    }
    if x.sick && !crate::engine::combat::has_haste(g, m) && !crate::impls::t45_krenko::elixir(g, p) {
        return Some(format!("{} has summoning sickness (it came under your control this turn).", x.name));
    }
    None
}

fn tap(g: &mut Game, m: PermId) {
    g.perm_mut(m).tapped = true;
    crate::impls::t0_captain::became_tapped(g, m);
}

fn sorcery(g: &Game, p: PlayerId) -> Why {
    legal::sorcery_timing(g, p)
}

fn targetable(g: &Game, p: PlayerId, m: PermId) -> bool {
    live(g, m) && !(g.perm(m).owner != p && untargetable(g, m))
}

/// the permanents p controls (lands too) that pass `f`
fn mine(g: &Game, p: PlayerId, f: impl Fn(&Game, PermId) -> bool) -> Vec<PermId> {
    let pl = g.player(p);
    let mut v: Vec<PermId> = pl.perms.iter().chain(pl.lands.iter()).copied().filter(|&m| live(g, m)).collect();
    v.dedup();
    v.retain(|&m| f(g, m));
    v
}

fn my_creatures(g: &Game, p: PlayerId, except: Option<PermId>) -> Vec<PermId> {
    mine(g, p, |g, m| g.is_creature(m) && Some(m) != except)
}

/// the permanents of every player that are legal targets for p
fn all_targets(g: &Game, p: PlayerId, f: impl Fn(&Game, PermId) -> bool) -> Vec<PermId> {
    let mut v = vec![];
    for q in g.players.iter().filter(|q| q.alive) {
        for &m in q.perms.iter().chain(q.lands.iter()) {
            if targetable(g, p, m) && !v.contains(&m) && f(g, m) {
                v.push(m);
            }
        }
    }
    v
}

fn pick_target(g: &Game, p: PlayerId, prompt: &str, cands: &[PermId]) -> Res<Option<PermId>> {
    pick_perm(g, p, "target", prompt, cands, Some("cancel"))
}

fn pick_cost(g: &Game, p: PlayerId, prompt: &str, cands: &[PermId]) -> Res<Option<PermId>> {
    pick_perm(g, p, "choose", prompt, cands, Some("cancel"))
}

/// `n` of `cands` for a cost ("tap five untapped Vampires", "sacrifice two artifacts"): None when the person cancels
fn pick_costs(g: &Game, p: PlayerId, prompt: &str, cands: &[PermId], n: usize) -> Res<Option<Vec<PermId>>> {
    let mut left = cands.to_vec();
    let mut got = vec![];
    for i in 0..n {
        let pr = if n > 1 { format!("{prompt} ({} of {n})", i + 1) } else { prompt.to_string() };
        let Some(x) = pick_perm(g, p, "choose", &pr, &left, if i == 0 { Some("cancel") } else { None })? else {
            return Ok(None);
        };
        left.retain(|&y| y != x);
        got.push(x);
    }
    Ok(Some(got))
}

fn sacrifice(g: &mut Game, p: PlayerId, what: PermId) -> Res {
    crate::glog!(g, "  {} sacrifices {}", g.player(p).name, g.perm(what).name);
    zones::die(g, what, "sac")
}

/// the ability (its cost paid) on the stack with one chosen permanent target; a copy (Peter Parker's Camera, Rings)
/// chooses a target anew from `cands`. `eff` runs on the target while it is still there. `gone`: the source was
/// sacrificed as the cost, so the ability's source is its card.
#[allow(clippy::too_many_arguments)]
fn aimed(
    g: &mut Game,
    p: PlayerId,
    m: PermId,
    gone: bool,
    what: &str,
    t: PermId,
    cands: Vec<PermId>,
    eff: impl Fn(&mut Game, PermId) -> Res,
) -> Res<Why> {
    let name = format!("{what}: {}", g.perm(t).name);
    let cd = g.perm(m).cd;
    let (src, card) = if gone { (None, cd) } else { (Some(m), None) };
    stack::ability_run_w(g, p, src, card, &name, None, Some(t), 0.0, |g, is_copy| {
        let mut t = t;
        if is_copy {
            let c: Vec<PermId> = cands.iter().copied().filter(|&x| live(g, x)).collect();
            let prompt = format!("The copy of {}'s ability: which target?", g.perm(m).name);
            if !c.is_empty()
                && let Some(x) = pick_perm(g, p, "target", &prompt, &c, Some("keep the target"))?
            {
                t = x;
            }
        }
        if live(g, t) {
            eff(g, t)?;
        }
        Ok(())
    })?;
    crate::engine::life::check_state(g)?;
    Ok(None)
}

/// the ability without a permanent target
fn plain(
    g: &mut Game,
    p: PlayerId,
    m: PermId,
    gone: bool,
    what: &str,
    mut eff: impl FnMut(&mut Game) -> Res,
) -> Res<Why> {
    let cd = g.perm(m).cd;
    let (src, card) = if gone { (None, cd) } else { (Some(m), None) };
    stack::ability_run_w(g, p, src, card, what, None, None, 0.0, |g, _| eff(g))?;
    crate::engine::life::check_state(g)?;
    Ok(None)
}

fn shrink(g: &mut Game, p: PlayerId, t: PermId, n: i32) -> Res {
    if !g.perm(t).on_bf {
        return Ok(());
    }
    if etgh(g, t) <= n {
        return apply_removal(g, Some(p), t, &format!("shrink{n}"), None);
    }
    let x = g.perm_mut(t);
    x.eot_pt = (x.eot_pt.0 - n, x.eot_pt.1 - n);
    Ok(())
}

fn opponents(g: &Game, p: PlayerId) -> Vec<PlayerId> {
    g.opps(p).filter(|&q| g.player(q).alive).collect()
}

/// the person picks a player to target (None: they cancel)
fn pick_opp(g: &Game, p: PlayerId, prompt: &str) -> Res<Option<PlayerId>> {
    let opps: Vec<PlayerId> = opponents(g, p).into_iter().filter(|&q| legal::player_targetable(g, q)).collect();
    if opps.is_empty() {
        return Ok(None);
    }
    let labels: Vec<String> = opps.iter().map(|&q| g.player(q).name.to_string()).collect();
    Ok(choices::choose(g, p, "target", prompt, &labels, Some("cancel"), None)?.map(|k| opps[k]))
}

// ------------------------------------------------------------------ the abilities
/// the person's ability list for creature `nm` (p's permanent m), or None when its hook has no choice to ask
pub fn abilities(nm: &str) -> Option<Vec<PermAbility>> {
    Some(match nm {
        "Amphin Pathmage" => vec![ability("{2}{U}: target creature can't be blocked this turn", pathmage)],
        "Chittering Witch" => {
            vec![ability("{1}{B}, sacrifice a creature: target creature gets -2/-2 until end of turn", witch)]
        }
        "Cankerbloom" => vec![ability(
            "{1}, sacrifice it: destroy target artifact, destroy target enchantment, or proliferate",
            canker,
        )],
        "Captivating Vampire" => {
            vec![ability("tap five untapped Vampires you control: gain control of target creature", vampire)]
        }
        "Goblin Cratermaker" => vec![ability(
            "{1}, sacrifice it: 2 damage to target creature, or destroy target colorless nonland permanent",
            cratermaker,
        )],
        "Goblin Trashmaster" => vec![ability("sacrifice a Goblin: destroy target artifact", trashmaster)],
        "Grim Hireling" => vec![ability(
            "{B}, sacrifice X Treasures: target creature gets -X/-X until end of turn (sorcery speed)",
            hireling,
        )],
        "Hawkeye, Avenging Archer" => vec![ability("{T}: 1 damage to any target", hawkeye)],
        "Hope of Ghirapur" => vec![ability(
            "sacrifice it: a player it dealt combat damage this turn can't cast noncreature spells until your next turn",
            hope,
        )],
        "Jagged-Scar Archers" => {
            vec![ability("{T}: damage equal to its power to target creature with flying", archers)]
        }
        "Loran of the Third Path" => vec![ability("{T}: you and target opponent each draw a card", loran)],
        "Mogg Fanatic" => vec![ability("sacrifice it: 1 damage to any target", fanatic)],
        "Pashalik Mons" => vec![ability("{3}{R}, sacrifice a Goblin: two 1/1 red Goblin tokens", mons)],
        "Pia Nalaar" => vec![
            ability("{1}{R}: target artifact creature gets +1/+0 until end of turn", pia_pump),
            ability("{1}, sacrifice an artifact: target creature can't block this turn", pia_block),
        ],
        "Sai, Master Thopterist" => vec![ability("{1}{U}, sacrifice two artifacts: draw a card", sai)],
        "Temur Sabertooth" => vec![ability(
            "{1}{G}: you may return another creature you control to its owner's hand; if you do, it gains indestructible",
            sabertooth,
        )],
        "Timberwatch Elf" => vec![ability(
            "{T}: target creature gets +X/+X until end of turn (X = Elves on the battlefield)",
            timberwatch,
        )],
        "Vilis, Broker of Blood" => {
            vec![ability("{B}, pay 2 life: target creature gets -1/-1 until end of turn", vilis)]
        }
        "Weathered Wayfarer" => {
            vec![ability("{W}, {T}: search for a land card (an opponent has more lands)", wayfarer)]
        }
        "Destiny Spinner" => vec![ability(
            "{3}{G}: target land you control becomes an X/X Elemental with trample and haste until end of turn",
            spinner,
        )],
        "Eldrazi Displacer" => vec![ability("{2}{C}: exile another target creature, then return it tapped", displacer)],
        "Formidable Speaker" => vec![ability("{1}, {T}: untap another target permanent", speaker)],
        "Patriot, Shield Wielder" => vec![ability(
            "{2}, {T}: another target creature you control gets +2/+0 and hexproof until end of turn",
            patriot,
        )],
        "Deathrite Shaman" => vec![
            ability(
                "{B}, {T}: exile target instant or sorcery card from a graveyard; each opponent loses 2 life",
                deathrite_drain,
            ),
            ability("{G}, {T}: exile target creature card from a graveyard; you gain 2 life", deathrite_gain),
        ],
        _ => return more(nm),
    })
}

/// the creature names this file lists abilities for
pub const NAMES: &[&str] = &[
    "Amphin Pathmage",
    "Chittering Witch",
    "Cankerbloom",
    "Captivating Vampire",
    "Goblin Cratermaker",
    "Goblin Trashmaster",
    "Grim Hireling",
    "Hawkeye, Avenging Archer",
    "Hope of Ghirapur",
    "Jagged-Scar Archers",
    "Loran of the Third Path",
    "Mogg Fanatic",
    "Pashalik Mons",
    "Pia Nalaar",
    "Sai, Master Thopterist",
    "Temur Sabertooth",
    "Timberwatch Elf",
    "Vilis, Broker of Blood",
    "Weathered Wayfarer",
    "Destiny Spinner",
    "Eldrazi Displacer",
    "Formidable Speaker",
    "Patriot, Shield Wielder",
    "Deathrite Shaman",
    "Ioreth of the Healing House",
    "Goblin Engineer",
    "Goblin Welder",
    "High Perfect Morcant",
    "Kinnan, Bonder Prodigy",
    "Loot, Exuberant Explorer",
    "Magda, Brazen Outlaw",
    "Magus of the Candelabra",
    "Mina and Denn, Wildborn",
    "Nullmage Shepherd",
    "Nyx Weaver",
    "Oboro Breezecaller",
    "Preston, the Vanisher",
    "Priest of Forgotten Gods",
    "Recruitment Officer",
    "Ruthless Technomancer",
    "Sisay, Weatherlight Captain",
    "Skrelv, Defector Mite",
    "Tavern Scoundrel",
    "Teysa, Orzhov Scion",
    "Urza, Powerstone Prodigy",
    "Eladamri, Korvecdal",
    "Drivnod, Carnage Dominus",
    "Elvish Reclaimer",
    "Brutal Hordechief",
    "Scourge of Valkas",
    "Lathril, Blade of the Elves",
    "Tekuthal, Inquiry Dominus",
];

fn pathmage(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Amphin Pathmage";
    if let Some(w) = cost(g, p, 2, "U", W) {
        return say(w);
    }
    let cands = creatures(g, p, false);
    let Some(t) = pick_target(g, p, "Amphin Pathmage: which creature can't be blocked this turn?", &cands)? else {
        return Ok(None);
    };
    if let Some(w) = pay(g, p, 2, "U", W) {
        return say(w);
    }
    aimed(g, p, m, false, "can't be blocked", t, cands, |g, t| {
        let st = g.turn_stamp();
        g.perm_mut(t).data.set(DataKey::Unbl, Val::Stamp(st));
        crate::glog!(g, "    {} can't be blocked this turn", g.perm(t).name);
        Ok(())
    })
}

fn witch(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Chittering Witch";
    if let Some(w) = cost(g, p, 1, "B", W) {
        return say(w);
    }
    let fodder = my_creatures(g, p, None);
    if fodder.is_empty() {
        return say("You control no creature to sacrifice.");
    }
    let cands = creatures(g, p, false);
    let Some(t) = pick_target(g, p, "Chittering Witch: -2/-2 on which creature?", &cands)? else { return Ok(None) };
    let Some(f) = pick_cost(g, p, "Chittering Witch: sacrifice which creature?", &fodder)? else { return Ok(None) };
    if let Some(w) = pay(g, p, 1, "B", W) {
        return say(w);
    }
    sacrifice(g, p, f)?;
    aimed(g, p, m, false, "-2/-2", t, cands, |g, t| shrink(g, p, t, 2))
}

fn canker(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Cankerbloom";
    if let Some(w) = cost(g, p, 1, "", W) {
        return say(w);
    }
    if !live(g, m) {
        return say("Cankerbloom is not on the battlefield.");
    }
    let arts = all_targets(g, p, |g, x| crate::impls::partials::of_type(g, x, crate::cards::Types::ARTIFACT));
    let ench = all_targets(g, p, |g, x| crate::impls::partials::of_type(g, x, crate::cards::Types::ENCHANTMENT));
    let labels: Vec<String> =
        vec!["destroy target artifact".into(), "destroy target enchantment".into(), "proliferate".into()];
    let Some(k) = choices::choose(g, p, "choose", "Cankerbloom: choose one", &labels, Some("cancel"), None)? else {
        return Ok(None);
    };
    let (cands, what) = match k {
        0 => (arts, "destroy"),
        1 => (ench, "destroy"),
        _ => (vec![], "proliferate"),
    };
    let t = if k < 2 {
        if cands.is_empty() {
            return say("There is no such target.");
        }
        let Some(t) = pick_target(g, p, "Cankerbloom: destroy which permanent?", &cands)? else { return Ok(None) };
        Some(t)
    } else {
        None
    };
    if let Some(w) = pay(g, p, 1, "", W) {
        return say(w);
    }
    let cd = g.perm(m).cd;
    sacrifice(g, p, m)?;
    match t {
        Some(t) => aimed(g, p, m, true, what, t, cands, move |g, t| apply_removal(g, Some(p), t, "destroy", cd)),
        None => plain(g, p, m, true, "proliferate", |g| crate::impls::common::proliferate(g, p, 1)),
    }
}

fn vampire(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    let vamps = mine(g, p, |g, x| g.is_creature(x) && !g.perm(x).tapped && has_type(g, x, "vampire"));
    if vamps.len() < 5 {
        return say("You control fewer than five untapped Vampires to tap.");
    }
    let cands: Vec<PermId> = creatures(g, p, true);
    let Some(t) = pick_target(g, p, "Captivating Vampire: gain control of which creature?", &cands)? else {
        return Ok(None);
    };
    let Some(five) = pick_costs(g, p, "Captivating Vampire: tap which Vampire?", &vamps, 5)? else { return Ok(None) };
    for x in five {
        g.perm_mut(x).tapped = true;
        crate::impls::t0_captain::became_tapped(g, x);
    }
    aimed(g, p, m, false, "gain control", t, cands, |g, t| {
        if g.perm(t).owner != p || !g.player(p).perms.contains(&t) {
            crate::impls::t45_edgar::take_as_vampire(g, p, t, "Captivating Vampire")?;
        }
        Ok(())
    })
}

fn cratermaker(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Goblin Cratermaker";
    if let Some(w) = cost(g, p, 1, "", W) {
        return say(w);
    }
    if !live(g, m) {
        return say("Goblin Cratermaker is not on the battlefield.");
    }
    let labels: Vec<String> =
        vec!["2 damage to target creature".into(), "destroy target colorless nonland permanent".into()];
    let Some(k) = choices::choose(g, p, "choose", "Goblin Cratermaker: choose one", &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    let cands = if k == 0 {
        all_targets(g, p, |g, x| g.is_creature(x) && x != m)
    } else {
        all_targets(g, p, |g, x| {
            !crate::impls::partials::of_type(g, x, crate::cards::Types::LAND)
                && crate::engine::values::colors_of(g, x).is_empty()
        })
    };
    let cands: Vec<PermId> = cands.into_iter().filter(|&x| x != m || k == 0).collect();
    if cands.is_empty() {
        return say("There is no such target.");
    }
    let Some(t) = pick_target(g, p, "Goblin Cratermaker: which target?", &cands)? else { return Ok(None) };
    if let Some(w) = pay(g, p, 1, "", W) {
        return say(w);
    }
    sacrifice(g, p, m)?;
    aimed(g, p, m, true, if k == 0 { "2 damage" } else { "destroy" }, t, cands, move |g, t| {
        apply_removal(g, Some(p), t, if k == 0 { "dmg2" } else { "destroy" }, None)
    })
}

fn trashmaster(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    let gobs = mine(g, p, |g, x| g.is_creature(x) && has_type(g, x, "goblin"));
    if gobs.is_empty() {
        return say("You control no Goblin to sacrifice.");
    }
    let cands = all_targets(g, p, |g, x| crate::impls::partials::of_type(g, x, crate::cards::Types::ARTIFACT));
    if cands.is_empty() {
        return say("There is no artifact to target.");
    }
    let Some(t) = pick_target(g, p, "Goblin Trashmaster: destroy which artifact?", &cands)? else { return Ok(None) };
    let Some(f) = pick_cost(g, p, "Goblin Trashmaster: sacrifice which Goblin?", &gobs)? else { return Ok(None) };
    sacrifice(g, p, f)?;
    aimed(g, p, m, false, "destroy", t, cands, |g, t| apply_removal(g, Some(p), t, "destroy", None))
}

fn hireling(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Grim Hireling";
    if let Some(w) = sorcery(g, p) {
        return say(w);
    }
    if let Some(w) = cost(g, p, 0, "B", W) {
        return say(w);
    }
    let n = g.player(p).treasures;
    if n == 0 {
        return say("You control no Treasure to sacrifice.");
    }
    let xs = choices::amount_choices(n);
    let labels: Vec<String> =
        xs.iter().map(|x| format!("X = {x} (sacrifice {x} Treasure{})", if *x == 1 { "" } else { "s" })).collect();
    let Some(k) =
        choices::choose(g, p, "choose", "Grim Hireling: sacrifice how many Treasures?", &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    let x = xs[k];
    let cands = creatures(g, p, false);
    let Some(t) = pick_target(g, p, "Grim Hireling: -X/-X on which creature?", &cands)? else { return Ok(None) };
    if let Some(w) = pay(g, p, 0, "B", W) {
        return say(w);
    }
    for _ in 0..x {
        crate::impls::model_gaps_a4::sac_treasure(g, p)?;
    }
    aimed(g, p, m, false, &format!("-{x}/-{x}"), t, cands, move |g, t| shrink(g, p, t, x as i32))
}

/// 1 damage to any target: the person picks (a creature, planeswalker or player); `m` is the source of the damage
fn pick_any_target(g: &Game, p: PlayerId, m: PermId, prompt: &str) -> Res<Option<crate::impls::model_gaps_a4::Tgt>> {
    use crate::impls::model_gaps_a4::Tgt;
    let cands = choices::damage_targets(g, p, crate::engine::values::colors_of(g, m));
    if cands.is_empty() {
        return Ok(None);
    }
    let labels: Vec<String> = cands.iter().map(|&x| legal::describe_target(g, p, x)).collect();
    Ok(choices::choose(g, p, "target", prompt, &labels, Some("cancel"), None)?.map(|k| match cands[k] {
        Target::Player(q) => Tgt::Player(q),
        Target::Perm(x) => Tgt::Perm(x),
    }))
}

/// n damage from m to a chosen target as an ability on the stack (a copy picks a new one)
fn damage_ability(
    g: &mut Game,
    p: PlayerId,
    m: PermId,
    gone: bool,
    t: crate::impls::model_gaps_a4::Tgt,
    n: i32,
) -> Res<Why> {
    use crate::impls::model_gaps_a4::{Tgt, ping};
    let human_name = g.perm(m).name.to_string();
    let tp = if let Tgt::Perm(x) = t { Some(x) } else { None };
    let cd = g.perm(m).cd;
    let (src, card) = if gone { (None, cd) } else { (Some(m), None) };
    let what = match t {
        Tgt::Player(q) => g.player(q).name.to_string(),
        Tgt::Perm(x) => g.perm(x).name.to_string(),
    };
    stack::ability_run_w(g, p, src, card, &format!("{n} damage to {what}"), None, tp, 0.0, |g, is_copy| {
        let mut t = t;
        if is_copy
            && let Some(x) =
                pick_any_target(g, p, m, &format!("The copy of {human_name}'s ability: {n} damage to which target?"))?
        {
            t = x;
        }
        ping(g, p, m, t, n)
    })?;
    crate::engine::life::check_state(g)?;
    Ok(None)
}

fn hawkeye(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    let Some(t) = pick_any_target(g, p, m, "Hawkeye: 1 damage to which target?")? else {
        return say("There is no target for the damage.");
    };
    tap(g, m);
    damage_ability(g, p, m, false, t, 1)
}

fn fanatic(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if !live(g, m) {
        return say("Mogg Fanatic is not on the battlefield.");
    }
    let Some(t) = pick_any_target(g, p, m, "Mogg Fanatic: 1 damage to which target?")? else {
        return say("There is no target for the damage.");
    };
    sacrifice(g, p, m)?;
    damage_ability(g, p, m, true, t, 1)
}

fn hope(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if !live(g, m) {
        return say("Hope of Ghirapur is not on the battlefield.");
    }
    let hit = crate::impls::partials::hit_players(g, m);
    if hit.is_empty() {
        return say("No player was dealt combat damage by it this turn.");
    }
    let labels: Vec<String> = hit.iter().map(|&q| g.player(q).name.to_string()).collect();
    let Some(k) = choices::choose(
        g,
        p,
        "target",
        "Hope of Ghirapur: which player can't cast noncreature spells?",
        &labels,
        Some("cancel"),
        None,
    )?
    else {
        return Ok(None);
    };
    let d = hit[k];
    sacrifice(g, p, m)?;
    plain(g, p, m, true, "can't cast noncreature spells", move |g| {
        let t = g.player(p).turns;
        crate::engine::cast::add_cast_lock(
            g,
            d,
            crate::state::LockKind::Noncreature,
            crate::state::LockEnd::TurnOf(p, t),
        );
        Ok(())
    })
}

fn archers(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    let cands = all_targets(g, p, |g, x| g.is_creature(x) && crate::dsl::has_kw(g, x, "flying"));
    if cands.is_empty() {
        return say("There is no creature with flying to target.");
    }
    let Some(t) = pick_target(g, p, "Jagged-Scar Archers: damage to which creature with flying?", &cands)? else {
        return Ok(None);
    };
    tap(g, m);
    aimed(g, p, m, false, "damage", t, cands, move |g, t| {
        let n = epow(g, m).max(0);
        crate::human::choices::apply_damage(g, p, Target::Perm(t), n, "Jagged-Scar Archers", "burn")
    })
}

fn loran(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    let Some(q) = pick_opp(g, p, "Loran: which opponent draws a card?")? else {
        return say("There is no opponent to target.");
    };
    tap(g, m);
    plain(g, p, m, false, "you and an opponent draw", move |g| {
        zones::draw(g, p, 1, false)?;
        zones::draw(g, q, 1, false)
    })
}

fn mons(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Pashalik Mons";
    if let Some(w) = cost(g, p, 3, "R", W) {
        return say(w);
    }
    let gobs = mine(g, p, |g, x| g.is_creature(x) && has_type(g, x, "goblin"));
    if gobs.is_empty() {
        return say("You control no Goblin to sacrifice.");
    }
    let Some(f) = pick_cost(g, p, "Pashalik Mons: sacrifice which Goblin?", &gobs)? else { return Ok(None) };
    if let Some(w) = pay(g, p, 3, "R", W) {
        return say(w);
    }
    sacrifice(g, p, f)?;
    plain(g, p, m, false, "two Goblins", move |g| {
        let spec = zones::Tokens {
            color: Some(crate::cards::Colors::from_letters("R")),
            types: vec!["goblin"],
            ..zones::Tokens::new(2, 1)
        };
        zones::make_tokens(g, p, spec).map(|_| ())
    })
}

fn pia_pump(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Pia Nalaar";
    if let Some(w) = cost(g, p, 1, "R", W) {
        return say(w);
    }
    let cands = all_targets(g, p, |g, x| {
        g.is_creature(x) && crate::impls::partials::of_type(g, x, crate::cards::Types::ARTIFACT)
    });
    if cands.is_empty() {
        return say("There is no artifact creature to target.");
    }
    let Some(t) = pick_target(g, p, "Pia Nalaar: +1/+0 on which artifact creature?", &cands)? else { return Ok(None) };
    if let Some(w) = pay(g, p, 1, "R", W) {
        return say(w);
    }
    aimed(g, p, m, false, "+1/+0", t, cands, |g, t| {
        crate::impls::common::eot(g, t, 1, 0);
        Ok(())
    })
}

fn pia_block(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Pia Nalaar";
    if let Some(w) = cost(g, p, 1, "", W) {
        return say(w);
    }
    let arts = mine(g, p, |g, x| {
        crate::impls::partials::of_type(g, x, crate::cards::Types::ARTIFACT)
            && !crate::impls::partials::of_type(g, x, crate::cards::Types::LAND)
    });
    let tr = g.player(p).treasures;
    if arts.is_empty() && tr == 0 {
        return say("You control no artifact to sacrifice.");
    }
    let cands = creatures(g, p, false);
    let Some(t) = pick_target(g, p, "Pia Nalaar: which creature can't block this turn?", &cands)? else {
        return Ok(None);
    };
    // the artifact: a permanent, or a Treasure token the engine keeps as a count
    let mut labels: Vec<String> = arts.iter().map(|&a| g.perm(a).name.to_string()).collect();
    if tr > 0 {
        labels.push("a Treasure".into());
    }
    let Some(k) =
        choices::choose(g, p, "choose", "Pia Nalaar: sacrifice which artifact?", &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    if let Some(w) = pay(g, p, 1, "", W) {
        return say(w);
    }
    if k < arts.len() {
        sacrifice(g, p, arts[k])?;
    } else {
        crate::impls::model_gaps_a4::sac_treasure(g, p)?;
    }
    aimed(g, p, m, false, "can't block", t, cands, |g, t| {
        crate::impls::partials::eot_kw(g, t, "cant_block");
        Ok(())
    })
}

fn sai(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Sai, Master Thopterist";
    if let Some(w) = cost(g, p, 1, "U", W) {
        return say(w);
    }
    let arts = mine(g, p, |g, x| {
        crate::impls::partials::of_type(g, x, crate::cards::Types::ARTIFACT)
            && !crate::impls::partials::of_type(g, x, crate::cards::Types::LAND)
    });
    let tr = g.player(p).treasures as usize;
    if arts.len() + tr < 2 {
        return say("You control fewer than two artifacts to sacrifice.");
    }
    // Treasures are counted, not permanents: they are offered as one entry each
    let mut cands = arts.clone();
    let mut labels: Vec<String> = cands.iter().map(|&a| g.perm(a).name.to_string()).collect();
    let mut chosen_perms = vec![];
    let mut treasures = 0usize;
    for i in 0..2 {
        let mut ls = labels.clone();
        let tleft = tr - treasures;
        if tleft > 0 {
            ls.push("a Treasure".into());
        }
        let Some(k) = choices::choose(
            g,
            p,
            "choose",
            &format!("Sai: sacrifice which artifact? ({} of 2)", i + 1),
            &ls,
            if i == 0 { Some("cancel") } else { None },
            None,
        )?
        else {
            return Ok(None);
        };
        if k < cands.len() {
            chosen_perms.push(cands.remove(k));
            labels.remove(k);
        } else {
            treasures += 1;
        }
    }
    if let Some(w) = pay(g, p, 1, "U", W) {
        return say(w);
    }
    for a in chosen_perms {
        sacrifice(g, p, a)?;
    }
    for _ in 0..treasures {
        crate::impls::model_gaps_a4::sac_treasure(g, p)?;
    }
    plain(g, p, m, false, "draw a card", move |g| zones::draw(g, p, 1, false))
}

fn sabertooth(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Temur Sabertooth";
    if let Some(w) = cost(g, p, 1, "G", W) {
        return say(w);
    }
    let cands = my_creatures(g, p, Some(m));
    let mut labels: Vec<String> = vec!["don't return a creature".into()];
    labels.extend(cands.iter().map(|&x| legal::describe_target(g, p, Target::Perm(x))));
    let Some(k) = choices::choose(
        g,
        p,
        "choose",
        "Temur Sabertooth: return which creature to its owner's hand?",
        &labels,
        Some("cancel"),
        None,
    )?
    else {
        return Ok(None);
    };
    if let Some(w) = pay(g, p, 1, "G", W) {
        return say(w);
    }
    let t = if k == 0 { None } else { Some(cands[k - 1]) };
    plain(g, p, m, false, "return a creature", move |g| {
        if let Some(t) = t
            && live(g, t)
            && g.player(p).perms.contains(&t)
        {
            zones::bounce(g, t)?;
            crate::impls::partials::eot_kw(g, m, "indestructible");
        }
        Ok(())
    })
}

fn timberwatch(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    let cands = creatures(g, p, false);
    let Some(t) = pick_target(g, p, "Timberwatch Elf: +X/+X on which creature?", &cands)? else { return Ok(None) };
    tap(g, m);
    aimed(g, p, m, false, "+X/+X", t, cands, |g, t| {
        let x = crate::impls::t1::count_type(g, p, "elf", true);
        crate::impls::common::eot(g, t, x, x);
        Ok(())
    })
}

fn vilis(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Vilis, Broker of Blood";
    if let Some(w) = cost(g, p, 0, "B", W) {
        return say(w);
    }
    if g.player(p).life <= 2 {
        return say("You don't have the life to pay.");
    }
    let cands = creatures(g, p, false);
    let Some(t) = pick_target(g, p, "Vilis: -1/-1 on which creature?", &cands)? else { return Ok(None) };
    if let Some(w) = pay(g, p, 0, "B", W) {
        return say(w);
    }
    crate::engine::life::lose_life(g, p, 2, Some(p), "other", None)?;
    aimed(g, p, m, false, "-1/-1", t, cands, |g, t| shrink(g, p, t, 1))
}

fn wayfarer(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Weathered Wayfarer";
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    if !g.opps(p).any(|q| g.player(q).lands.len() > g.player(p).lands.len()) {
        return say("No opponent controls more lands than you.");
    }
    if let Some(w) = pay(g, p, 0, "W", W) {
        return say(w);
    }
    tap(g, m);
    plain(g, p, m, false, "search for a land", move |g| {
        let got =
            choices::search(g, p, &|g, c| g.db.get(c).land, 1, "Weathered Wayfarer: search for a land card", true)?;
        for c in got {
            crate::glog!(g, "    {} puts {} into their hand", g.player(p).name, g.db.get(c).name);
            g.player_mut(p).hand.push(c);
        }
        Ok(())
    })
}

fn spinner(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Destiny Spinner";
    if let Some(w) = cost(g, p, 3, "G", W) {
        return say(w);
    }
    let cands = mine(g, p, |g, x| crate::impls::partials::of_type(g, x, crate::cards::Types::LAND));
    if cands.is_empty() {
        return say("You control no land.");
    }
    let Some(t) = pick_target(g, p, "Destiny Spinner: which land becomes a creature?", &cands)? else {
        return Ok(None);
    };
    if let Some(w) = pay(g, p, 3, "G", W) {
        return say(w);
    }
    aimed(g, p, m, false, "becomes an Elemental", t, cands, |g, t| {
        let x = crate::impls::rules2::spinner_x(g, p);
        crate::impls::lands::animate(g, p, t, x, x, &["trample", "haste"], false);
        Ok(())
    })
}

fn displacer(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Eldrazi Displacer";
    if let Some(w) = cost(g, p, 3, "", W) {
        return say(w);
    }
    let cands = all_targets(g, p, |g, x| g.is_creature(x) && x != m);
    if cands.is_empty() {
        return say("There is no other creature to target.");
    }
    let Some(t) = pick_target(g, p, "Eldrazi Displacer: exile which creature?", &cands)? else { return Ok(None) };
    if let Some(w) = pay(g, p, 3, "", W) {
        return say(w);
    }
    aimed(g, p, m, false, "blink", t, cands, |g, t| {
        if g.perm(t).token {
            zones::leave(g, t)?;
            crate::glog!(g, "    {} (a token) is exiled for good", g.perm(t).name);
        } else if let Some(n) = crate::impls::t2::blink(g, p, t)? {
            g.perm_mut(n).tapped = true;
        }
        Ok(())
    })
}

fn speaker(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Formidable Speaker";
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    if let Some(w) = cost(g, p, 1, "", W) {
        return say(w);
    }
    let cands = all_targets(g, p, |_, x| x != m);
    if cands.is_empty() {
        return say("There is no other permanent to target.");
    }
    let Some(t) = pick_target(g, p, "Formidable Speaker: untap which permanent?", &cands)? else { return Ok(None) };
    if let Some(w) = pay(g, p, 1, "", W) {
        return say(w);
    }
    tap(g, m);
    aimed(g, p, m, false, "untap", t, cands, |g, t| {
        crate::impls::t6_green1::untap_perm(g, t);
        Ok(())
    })
}

fn patriot(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Patriot, Shield Wielder";
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    if let Some(w) = cost(g, p, 2, "", W) {
        return say(w);
    }
    let cands: Vec<PermId> = my_creatures(g, p, Some(m));
    if cands.is_empty() {
        return say("You control no other creature to target.");
    }
    let Some(t) = pick_target(g, p, "Patriot: +2/+0 and hexproof on which creature?", &cands)? else { return Ok(None) };
    if let Some(w) = pay(g, p, 2, "", W) {
        return say(w);
    }
    tap(g, m);
    aimed(g, p, m, false, "+2/+0 and hexproof", t, cands, |g, t| {
        crate::impls::common::eot(g, t, 2, 0);
        crate::impls::partials::eot_kw(g, t, "hexproof");
        Ok(())
    })
}

/// the cards of the given kind in every graveyard, as (owner, card)
fn gy_cards(g: &Game, f: impl Fn(&Game, CardId) -> bool) -> Vec<(PlayerId, CardId)> {
    g.players.iter().flat_map(|q| q.gy.iter().filter(|&&c| f(g, c)).map(move |&c| (q.id, c))).collect()
}

fn deathrite(g: &mut Game, p: PlayerId, m: PermId, pip: &str, spells: bool) -> Res<Why> {
    const W: &str = "Deathrite Shaman";
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    if let Some(w) = cost(g, p, 0, pip, W) {
        return say(w);
    }
    let cands =
        gy_cards(g, |g, c| if spells { g.db.get(c).instant || g.db.get(c).sorcery } else { g.db.get(c).creature });
    if cands.is_empty() {
        return say(if spells {
            "No graveyard holds an instant or sorcery card."
        } else {
            "No graveyard holds a creature card."
        });
    }
    let labels: Vec<String> = cands
        .iter()
        .map(|&(q, c)| format!("{} ({}'s graveyard)", choices::card_label(g, c), g.player(q).name))
        .collect();
    let Some(k) =
        choices::choose(g, p, "target", "Deathrite Shaman: exile which card?", &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    let (q, c) = cands[k];
    if let Some(w) = pay(g, p, 0, pip, W) {
        return say(w);
    }
    tap(g, m);
    plain(g, p, m, false, if spells { "each opponent loses 2 life" } else { "you gain 2 life" }, move |g| {
        if let Some(i) = g.player(q).gy.iter().position(|&x| x == c) {
            g.player_mut(q).gy.remove(i);
            g.player_mut(q).exile.push(c);
            crate::glog!(g, "    Deathrite Shaman exiles {}", g.db.get(c).name);
        }
        if spells {
            for o in opponents(g, p) {
                crate::engine::life::lose_life(g, o, 2, Some(p), "triggers", None)?;
            }
            Ok(())
        } else {
            crate::engine::life::gain(g, p, 2)
        }
    })
}

fn deathrite_drain(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    deathrite(g, p, m, "B", true)
}

fn deathrite_gain(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    deathrite(g, p, m, "G", false)
}

// ================================================================== batch 2
fn more(nm: &str) -> Option<Vec<PermAbility>> {
    Some(match nm {
        "Ioreth of the Healing House" => vec![
            ability("{T}: untap another target permanent", ioreth_one),
            ability("{T}: untap two other target legendary creatures", ioreth_two),
        ],
        "Goblin Engineer" => vec![ability(
            "{R}, {T}, sacrifice an artifact: return target artifact card with mana value 3 or less from your graveyard to the battlefield",
            engineer,
        )],
        "Goblin Welder" => {
            vec![ability("{T}: swap target artifact with target artifact card in that player's graveyard", welder)]
        }
        "High Perfect Morcant" => {
            vec![ability("tap three untapped Elves you control: proliferate (sorcery speed)", morcant)]
        }
        "Kinnan, Bonder Prodigy" => vec![ability(
            "{5}{G}{U}: look at the top five cards; you may put a non-Human creature card onto the battlefield",
            kinnan,
        )],
        "Loot, Exuberant Explorer" => vec![ability(
            "{4}{G}{G}, {T}: look at the top six cards; you may put a creature card onto the battlefield",
            loot,
        )],
        "Magda, Brazen Outlaw" => vec![ability(
            "sacrifice five Treasures: search for an artifact or Dragon card and put it onto the battlefield",
            magda,
        )],
        "Magus of the Candelabra" => vec![ability("{X}, {T}: untap X target lands", magus)],
        "Mina and Denn, Wildborn" => vec![ability(
            "{R}{G}, return a land you control to its owner's hand: target creature gains trample until end of turn",
            mina,
        )],
        "Nullmage Shepherd" => {
            vec![ability("tap four untapped creatures you control: destroy target artifact or enchantment", nullmage)]
        }
        "Nyx Weaver" => {
            vec![ability("{1}{B}{G}, exile it: return target card from your graveyard to your hand", nyx_weaver)]
        }
        "Oboro Breezecaller" => {
            vec![ability("{2}, return a land you control to its owner's hand: untap target land", oboro)]
        }
        "Preston, the Vanisher" => {
            vec![ability("{1}{W}, sacrifice five Illusions: exile target nonland permanent", preston)]
        }
        "Priest of Forgotten Gods" => vec![ability(
            "{T}, sacrifice two other creatures: any number of target players each lose 2 life and sacrifice a creature; you add {B}{B} and draw a card",
            priest,
        )],
        "Recruitment Officer" => vec![ability(
            "{3}{W}: look at the top four cards; you may put a creature card with mana value 3 or less into your hand",
            officer,
        )],
        "Ruthless Technomancer" => vec![ability(
            "{2}{B}, sacrifice X artifacts: return target creature card with power X or less from your graveyard to the battlefield",
            technomancer,
        )],
        "Sisay, Weatherlight Captain" => vec![ability(
            "{W}{U}{B}{R}{G}: search for a legendary permanent card with mana value less than its power and put it onto the battlefield",
            sisay,
        )],
        "Skrelv, Defector Mite" => vec![ability(
            "{W/P}, {T}: another target creature you control gains toxic 1 and hexproof from a color you choose and can't be blocked by creatures of that color",
            skrelv,
        )],
        "Tavern Scoundrel" => {
            vec![ability("{1}, {T}, sacrifice another permanent: flip a coin; if you win, two Treasures", scoundrel)]
        }
        "Teysa, Orzhov Scion" => vec![ability("sacrifice three white creatures: exile target creature", teysa)],
        "Urza, Powerstone Prodigy" => vec![ability("{1}, {T}: draw a card, then discard a card", urza)],
        "Eladamri, Korvecdal" => vec![ability(
            "{G}, {T}, tap two untapped creatures you control: reveal a card from your hand or the top of your library; a creature card enters (your turn only)",
            eladamri,
        )],
        "Drivnod, Carnage Dominus" => vec![ability(
            "{B/P}{B/P}, exile three creature cards from your graveyard: an indestructible counter on it",
            drivnod,
        )],
        "Elvish Reclaimer" => vec![ability(
            "{2}, {T}, sacrifice a land: search for a land card and put it onto the battlefield tapped",
            reclaimer,
        )],
        "Brutal Hordechief" => vec![ability(
            "{3}{R/W}{R/W}: creatures your opponents control block this turn if able, and you choose how they block",
            hordechief,
        )],
        "Scourge of Valkas" => vec![ability("{R}: it gets +1/+0 until end of turn", scourge)],
        "Lathril, Blade of the Elves" => vec![ability(
            "{T}, tap ten untapped Elves you control: each opponent loses 10 life and you gain 10 life",
            lathril,
        )],
        "Tekuthal, Inquiry Dominus" => vec![ability(
            "{1}{U/P}{U/P}, remove three counters from among other artifacts, creatures and planeswalkers you control: an indestructible counter on it",
            |g, p, m| tekuthal(g, p, m, m),
        )],
        _ => return None,
    })
}

/// cost to sacrifice artifacts: permanents or Treasures (a count in the engine)
enum Art {
    Perm(PermId),
    Treasure,
}

/// the person picks `n` artifacts to sacrifice; None when they cancel the first pick
fn pick_artifacts(g: &Game, p: PlayerId, n: usize, prompt: &str, except: Option<PermId>) -> Res<Option<Vec<Art>>> {
    use crate::cards::Types;
    let mut perms: Vec<PermId> = mine(g, p, |g, x| {
        crate::impls::partials::of_type(g, x, Types::ARTIFACT)
            && !crate::impls::partials::of_type(g, x, Types::LAND)
            && Some(x) != except
    });
    let mut treasures = g.player(p).treasures as usize;
    if perms.len() + treasures < n {
        return Ok(None);
    }
    let mut got = vec![];
    for i in 0..n {
        let mut labels: Vec<String> = perms.iter().map(|&a| g.perm(a).name.to_string()).collect();
        if treasures > 0 {
            labels.push("a Treasure".into());
        }
        let pr = if n > 1 { format!("{prompt} ({} of {n})", i + 1) } else { prompt.to_string() };
        let Some(k) = choices::choose(g, p, "choose", &pr, &labels, if i == 0 { Some("cancel") } else { None }, None)?
        else {
            return Ok(None);
        };
        if k < perms.len() {
            got.push(Art::Perm(perms.remove(k)));
        } else {
            treasures -= 1;
            got.push(Art::Treasure);
        }
    }
    Ok(Some(got))
}

fn sac_artifacts(g: &mut Game, p: PlayerId, arts: Vec<Art>) -> Res {
    for a in arts {
        match a {
            Art::Perm(x) => sacrifice(g, p, x)?,
            Art::Treasure => {
                crate::impls::model_gaps_a4::sac_treasure(g, p)?;
            }
        }
    }
    Ok(())
}

fn ioreth_one(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    let cands = all_targets(g, p, |_, x| x != m);
    if cands.is_empty() {
        return say("There is no other permanent to target.");
    }
    let Some(t) = pick_target(g, p, "Ioreth: untap which permanent?", &cands)? else { return Ok(None) };
    tap(g, m);
    aimed(g, p, m, false, "untap", t, cands, |g, t| {
        crate::impls::t6_green1::untap_perm(g, t);
        Ok(())
    })
}

fn ioreth_two(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    let cands = all_targets(g, p, |g, x| {
        g.is_creature(x) && x != m && g.perm(x).cd.is_some_and(|c| g.db.get(c).tag(crate::tag::Tag::Leg))
    });
    if cands.len() < 2 {
        return say("There are not two other legendary creatures to target.");
    }
    let Some(two) = pick_costs(g, p, "Ioreth: untap which legendary creature?", &cands, 2)? else { return Ok(None) };
    tap(g, m);
    plain(g, p, m, false, "untap two legendary creatures", move |g| {
        for &t in &two {
            if live(g, t) {
                crate::impls::t6_green1::untap_perm(g, t);
            }
        }
        Ok(())
    })
}

fn engineer(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Goblin Engineer";
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    if let Some(w) = cost(g, p, 0, "R", W) {
        return say(w);
    }
    let cards: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| g.db.get(c).types.has(crate::cards::Types::ARTIFACT) && g.db.get(c).cmc <= 3)
        .collect();
    if cards.is_empty() {
        return say("Your graveyard holds no artifact card with mana value 3 or less.");
    }
    let labels: Vec<String> = cards.iter().map(|&c| choices::card_label(g, c)).collect();
    let Some(k) =
        choices::choose(g, p, "target", "Goblin Engineer: return which artifact card?", &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    let c = cards[k];
    let Some(art) = pick_artifacts(g, p, 1, "Goblin Engineer: sacrifice which artifact?", None)? else {
        return say("You control no artifact to sacrifice.");
    };
    if let Some(w) = pay(g, p, 0, "R", W) {
        return say(w);
    }
    tap(g, m);
    sac_artifacts(g, p, art)?;
    plain(g, p, m, false, "return an artifact card", move |g| {
        if let Some(i) = g.player(p).gy.iter().position(|&x| x == c) {
            g.player_mut(p).gy.remove(i);
            zones::enter(g, p, c, zones::Enter::default())?;
        }
        Ok(())
    })
}

fn welder(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    let arts = all_targets(g, p, |g, x| crate::impls::partials::of_type(g, x, crate::cards::Types::ARTIFACT));
    let arts: Vec<PermId> = arts
        .into_iter()
        .filter(|&a| {
            let o = g.perm(a).owner;
            g.player(o).gy.iter().any(|&c| g.db.get(c).types.has(crate::cards::Types::ARTIFACT))
        })
        .collect();
    if arts.is_empty() {
        return say("No artifact has an artifact card in its controller's graveyard to swap with.");
    }
    let Some(a) = pick_target(g, p, "Goblin Welder: which artifact is sacrificed?", &arts)? else { return Ok(None) };
    let q = g.perm(a).owner;
    let cards: Vec<CardId> =
        g.player(q).gy.iter().copied().filter(|&c| g.db.get(c).types.has(crate::cards::Types::ARTIFACT)).collect();
    let labels: Vec<String> = cards.iter().map(|&c| choices::card_label(g, c)).collect();
    let Some(k) =
        choices::choose(g, p, "target", "Goblin Welder: which artifact card returns?", &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    let c = cards[k];
    tap(g, m);
    plain(g, p, m, false, "swap an artifact with an artifact card", move |g| {
        if live(g, a) && g.player(q).gy.contains(&c) {
            zones::die(g, a, "sac")?;
            if let Some(i) = g.player(q).gy.iter().position(|&x| x == c) {
                g.player_mut(q).gy.remove(i);
                zones::enter(g, q, c, zones::Enter::default())?;
            }
        }
        Ok(())
    })
}

fn morcant(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = sorcery(g, p) {
        return say(w);
    }
    let elves = mine(g, p, |g, x| g.is_creature(x) && !g.perm(x).tapped && has_type(g, x, "elf"));
    if elves.len() < 3 {
        return say("You control fewer than three untapped Elves to tap.");
    }
    let Some(three) = pick_costs(g, p, "High Perfect Morcant: tap which Elf?", &elves, 3)? else { return Ok(None) };
    for x in three {
        g.perm_mut(x).tapped = true;
        crate::impls::t0_captain::became_tapped(g, x);
    }
    plain(g, p, m, false, "proliferate", move |g| crate::impls::common::proliferate(g, p, 1))
}

/// look at the top `n` cards; the person may take one matching `pred` (to the battlefield or the hand); the rest go to
/// the bottom in a random order
fn dig(g: &mut Game, p: PlayerId, n: usize, pred: &dyn Fn(&Game, CardId) -> bool, prompt: &str, to_bf: bool) -> Res {
    let k = g.player(p).library.len().min(n);
    let mut top: Vec<CardId> = (0..k).filter_map(|_| g.player_mut(p).library.pop()).collect();
    let fits: Vec<CardId> = top.iter().copied().filter(|&c| pred(g, c)).collect();
    let mut pick = None;
    if !fits.is_empty() {
        let mut labels: Vec<String> = top.iter().map(|&c| choices::card_label(g, c)).collect();
        labels.retain(|_| true);
        let labels: Vec<String> = fits.iter().map(|&c| choices::card_label(g, c)).collect();
        let all: Vec<String> = top.iter().map(|&c| choices::card_label(g, c)).collect();
        crate::glog!(g, "    the top cards: {}", all.join(", "));
        pick = choices::choose(g, p, "choose", prompt, &labels, Some("none"), None)?.map(|i| fits[i]);
    }
    if let Some(c) = pick {
        top.retain(|&x| x != c);
    }
    g.rng.shuffle(&mut top);
    for c in top {
        g.player_mut(p).library.insert(0, c);
    }
    if let Some(c) = pick {
        crate::glog!(g, "    {} puts {}", g.player(p).name, g.db.get(c).name);
        if to_bf {
            zones::enter(g, p, c, zones::Enter::default())?;
        } else {
            g.player_mut(p).hand.push(c);
        }
    }
    Ok(())
}

fn kinnan(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = pay(g, p, 5, "GU", "Kinnan, Bonder Prodigy") {
        return say(w);
    }
    plain(g, p, m, false, "look at the top five cards", move |g| {
        dig(
            g,
            p,
            5,
            &|g, c| g.db.get(c).creature && !g.db.get(c).has_subtype("human"),
            "Kinnan: which non-Human creature card enters?",
            true,
        )
    })
}

fn loot(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    if let Some(w) = pay(g, p, 4, "GG", "Loot, Exuberant Explorer") {
        return say(w);
    }
    tap(g, m);
    plain(g, p, m, false, "look at the top six cards", move |g| {
        let lands = g.player(p).lands.len() as u32;
        dig(
            g,
            p,
            6,
            &move |g, c| g.db.get(c).creature && g.db.get(c).cmc <= lands,
            "Loot: which creature card enters?",
            true,
        )
    })
}

fn officer(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = pay(g, p, 3, "W", "Recruitment Officer") {
        return say(w);
    }
    plain(g, p, m, false, "look at the top four cards", move |g| {
        dig(
            g,
            p,
            4,
            &|g, c| g.db.get(c).creature && g.db.get(c).cmc <= 3,
            "Recruitment Officer: which creature card goes to your hand?",
            false,
        )
    })
}

fn magda(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if g.player(p).treasures < 5 {
        return say("You control fewer than five Treasures to sacrifice.");
    }
    for _ in 0..5 {
        crate::impls::model_gaps_a4::sac_treasure(g, p)?;
    }
    plain(g, p, m, false, "search for an artifact or Dragon card", move |g| {
        let got = choices::search(
            g,
            p,
            &|g, c| g.db.get(c).types.has(crate::cards::Types::ARTIFACT) || g.db.get(c).has_subtype("dragon"),
            1,
            "Magda: search for an artifact or Dragon card",
            true,
        )?;
        for c in got {
            zones::enter(g, p, c, zones::Enter::default())?;
        }
        Ok(())
    })
}

fn magus(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    let tapped =
        mine(g, p, |g, x| crate::impls::partials::of_type(g, x, crate::cards::Types::LAND) && g.perm(x).tapped);
    let lands = mine(g, p, |g, x| crate::impls::partials::of_type(g, x, crate::cards::Types::LAND));
    let xs: Vec<u32> = (1..=lands.len() as u32).filter(|&x| mana::cost_problem(g, p, x, "").is_none()).collect();
    if xs.is_empty() || tapped.is_empty() && lands.is_empty() {
        return say("You can't pay {X} for any X.");
    }
    let labels: Vec<String> = xs.iter().map(|x| format!("X = {x}")).collect();
    let Some(k) = choices::choose(g, p, "choose", "Magus of the Candelabra: X?", &labels, Some("cancel"), None)? else {
        return Ok(None);
    };
    let x = xs[k] as usize;
    let Some(ts) = pick_costs(g, p, "Magus of the Candelabra: untap which land?", &lands, x)? else { return Ok(None) };
    if let Some(w) = pay(g, p, x as u32, "", "Magus of the Candelabra") {
        return say(w);
    }
    tap(g, m);
    plain(g, p, m, false, "untap lands", move |g| {
        for &t in &ts {
            if live(g, t) {
                crate::impls::t6_green1::untap_perm(g, t);
            }
        }
        Ok(())
    })
}

/// return a land you control to its owner's hand (a cost); the person picks which
fn bounce_land_cost(g: &mut Game, p: PlayerId, prompt: &str) -> Res<Option<Option<CardId>>> {
    let lands = mine(g, p, |g, x| crate::impls::partials::of_type(g, x, crate::cards::Types::LAND));
    if lands.is_empty() {
        return Ok(Some(None));
    }
    let Some(l) = pick_cost(g, p, prompt, &lands)? else { return Ok(None) };
    let c = g.perm(l).card();
    crate::engine::turn::remove_land(g, p, l, Some(zones::Zone::Hand));
    crate::glog!(g, "  {} returns {} to their hand", g.player(p).name, g.db.get(c).name);
    Ok(Some(Some(c)))
}

fn mina(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Mina and Denn";
    if let Some(w) = cost(g, p, 0, "RG", W) {
        return say(w);
    }
    let cands = creatures(g, p, false);
    if cands.is_empty() {
        return say("There is no creature to target.");
    }
    let Some(t) = pick_target(g, p, "Mina and Denn: trample on which creature?", &cands)? else { return Ok(None) };
    if mine(g, p, |g, x| crate::impls::partials::of_type(g, x, crate::cards::Types::LAND)).is_empty() {
        return say("You control no land to return.");
    }
    let Some(_) = bounce_land_cost(g, p, "Mina and Denn: return which land to your hand?")? else { return Ok(None) };
    if let Some(w) = pay(g, p, 0, "RG", W) {
        return say(w);
    }
    aimed(g, p, m, false, "trample", t, cands, |g, t| {
        crate::impls::partials::eot_kw(g, t, "trample");
        Ok(())
    })
}

fn nullmage(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    use crate::cards::Types;
    let crew = mine(g, p, |g, x| g.is_creature(x) && !g.perm(x).tapped);
    if crew.len() < 4 {
        return say("You control fewer than four untapped creatures to tap.");
    }
    let cands = all_targets(g, p, |g, x| {
        crate::impls::partials::of_type(g, x, Types::ARTIFACT)
            || crate::impls::partials::of_type(g, x, Types::ENCHANTMENT)
    });
    if cands.is_empty() {
        return say("There is no artifact or enchantment to target.");
    }
    let Some(t) = pick_target(g, p, "Nullmage Shepherd: destroy which permanent?", &cands)? else { return Ok(None) };
    let Some(four) = pick_costs(g, p, "Nullmage Shepherd: tap which creature?", &crew, 4)? else { return Ok(None) };
    for x in four {
        g.perm_mut(x).tapped = true;
        crate::impls::t0_captain::became_tapped(g, x);
    }
    aimed(g, p, m, false, "destroy", t, cands, |g, t| apply_removal(g, Some(p), t, "destroy", None))
}

fn nyx_weaver(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Nyx Weaver";
    if let Some(w) = cost(g, p, 1, "BG", W) {
        return say(w);
    }
    if !live(g, m) {
        return say("Nyx Weaver is not on the battlefield.");
    }
    let cards = g.player(p).gy.clone();
    if cards.is_empty() {
        return say("Your graveyard is empty.");
    }
    let labels: Vec<String> = cards.iter().map(|&c| choices::card_label(g, c)).collect();
    let Some(k) =
        choices::choose(g, p, "target", "Nyx Weaver: return which card to your hand?", &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    let c = cards[k];
    if let Some(w) = pay(g, p, 1, "BG", W) {
        return say(w);
    }
    zones::leave(g, m)?; // exiled as the cost
    plain(g, p, m, true, "return a card", move |g| {
        if let Some(i) = g.player(p).gy.iter().position(|&x| x == c) {
            g.player_mut(p).gy.remove(i);
            g.player_mut(p).hand.push(c);
        }
        Ok(())
    })
}

fn oboro(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Oboro Breezecaller";
    if let Some(w) = cost(g, p, 2, "", W) {
        return say(w);
    }
    let lands = mine(g, p, |g, x| crate::impls::partials::of_type(g, x, crate::cards::Types::LAND));
    if lands.is_empty() {
        return say("You control no land to return.");
    }
    let cands = all_targets(g, p, |g, x| crate::impls::partials::of_type(g, x, crate::cards::Types::LAND));
    let Some(t) = pick_target(g, p, "Oboro Breezecaller: untap which land?", &cands)? else { return Ok(None) };
    let Some(_) = bounce_land_cost(g, p, "Oboro Breezecaller: return which land to your hand?")? else {
        return Ok(None);
    };
    if let Some(w) = pay(g, p, 2, "", W) {
        return say(w);
    }
    aimed(g, p, m, false, "untap", t, cands, |g, t| {
        crate::impls::t6_green1::untap_perm(g, t);
        Ok(())
    })
}

fn preston(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Preston, the Vanisher";
    if let Some(w) = cost(g, p, 1, "W", W) {
        return say(w);
    }
    let ill = mine(g, p, |g, x| g.is_creature(x) && has_type(g, x, "illusion"));
    if ill.len() < 5 {
        return say("You control fewer than five Illusions to sacrifice.");
    }
    let cands = all_targets(g, p, |g, x| !crate::impls::partials::of_type(g, x, crate::cards::Types::LAND));
    if cands.is_empty() {
        return say("There is no nonland permanent to target.");
    }
    let Some(t) = pick_target(g, p, "Preston: exile which nonland permanent?", &cands)? else { return Ok(None) };
    let Some(five) = pick_costs(g, p, "Preston: sacrifice which Illusion?", &ill, 5)? else { return Ok(None) };
    if let Some(w) = pay(g, p, 1, "W", W) {
        return say(w);
    }
    for x in five {
        sacrifice(g, p, x)?;
    }
    aimed(g, p, m, false, "exile", t, cands, |g, t| apply_removal(g, Some(p), t, "exile", None))
}

fn priest(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    let fodder = my_creatures(g, p, Some(m));
    if fodder.len() < 2 {
        return say("You control fewer than two other creatures to sacrifice.");
    }
    // any number of target players
    let mut left: Vec<PlayerId> =
        g.players.iter().filter(|q| q.alive && legal::player_targetable(g, q.id)).map(|q| q.id).collect();
    let mut chosen: Vec<PlayerId> = vec![];
    while !left.is_empty() {
        let labels: Vec<String> = left.iter().map(|&q| g.player(q).name.to_string()).collect();
        let pr = format!("Priest of Forgotten Gods: target which player? ({} chosen)", chosen.len());
        let Some(k) = choices::choose(
            g,
            p,
            "target",
            &pr,
            &labels,
            Some(if chosen.is_empty() { "no more players (none)" } else { "done" }),
            None,
        )?
        else {
            break;
        };
        chosen.push(left.remove(k));
    }
    let Some(two) = pick_costs(g, p, "Priest of Forgotten Gods: sacrifice which creature?", &fodder, 2)? else {
        return Ok(None);
    };
    tap(g, m);
    for x in two {
        sacrifice(g, p, x)?;
    }
    plain(g, p, m, false, "players lose 2 life and sacrifice a creature", move |g| {
        for &q in &chosen {
            if g.player(q).alive {
                crate::engine::life::lose_life(g, q, 2, Some(p), "drain", None)?;
                zones::edict(g, q, false)?;
            }
        }
        g.player_mut(p).pool.add('B', 2);
        zones::draw(g, p, 1, false)
    })
}

fn technomancer(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Ruthless Technomancer";
    if let Some(w) = cost(g, p, 2, "B", W) {
        return say(w);
    }
    let arts = g.player(p).treasures as usize
        + mine(g, p, |g, x| {
            crate::impls::partials::of_type(g, x, crate::cards::Types::ARTIFACT)
                && !crate::impls::partials::of_type(g, x, crate::cards::Types::LAND)
        })
        .len();
    if arts == 0 {
        return say("You control no artifact to sacrifice.");
    }
    let cards: Vec<CardId> =
        g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature && g.db.get(c).pow as usize <= arts).collect();
    if cards.is_empty() {
        return say("Your graveyard holds no creature card you could return.");
    }
    let labels: Vec<String> = cards.iter().map(|&c| choices::card_label(g, c)).collect();
    let Some(k) = choices::choose(
        g,
        p,
        "target",
        "Ruthless Technomancer: return which creature card?",
        &labels,
        Some("cancel"),
        None,
    )?
    else {
        return Ok(None);
    };
    let c = cards[k];
    let lo = (g.db.get(c).pow.max(1)) as usize;
    let xs: Vec<usize> = (lo..=arts).collect();
    let xl: Vec<String> = xs.iter().map(|x| format!("X = {x}")).collect();
    let xi = choices::choose_one(g, p, "choose", "Ruthless Technomancer: sacrifice how many artifacts?", &xl)?;
    let x = xs[xi];
    let Some(sac) = pick_artifacts(g, p, x, "Ruthless Technomancer: sacrifice which artifact?", None)? else {
        return Ok(None);
    };
    if let Some(w) = pay(g, p, 2, "B", W) {
        return say(w);
    }
    sac_artifacts(g, p, sac)?;
    plain(g, p, m, false, "return a creature card", move |g| {
        if let Some(i) = g.player(p).gy.iter().position(|&y| y == c) {
            g.player_mut(p).gy.remove(i);
            zones::enter(g, p, c, zones::Enter::default())?;
        }
        Ok(())
    })
}

fn sisay(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = pay(g, p, 0, "WUBRG", "Sisay") {
        return say(w);
    }
    plain(g, p, m, false, "search for a legendary permanent card", move |g| {
        let pw = epow(g, m);
        let got = choices::search(
            g,
            p,
            &move |g, c| {
                let d = g.db.get(c);
                d.tag(crate::tag::Tag::Leg) && d.perm && (d.cmc as i32) < pw
            },
            1,
            "Sisay: search for a legendary permanent card",
            true,
        )?;
        for c in got {
            zones::enter(g, p, c, zones::Enter::default())?;
        }
        Ok(())
    })
}

fn skrelv(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    let cands = my_creatures(g, p, Some(m));
    if cands.is_empty() {
        return say("You control no other creature to target.");
    }
    let Some(t) = pick_target(g, p, "Skrelv: which creature gains toxic and hexproof?", &cands)? else {
        return Ok(None);
    };
    let labels: Vec<String> = ["white", "blue", "black", "red", "green"].iter().map(|s| s.to_string()).collect();
    let Some(k) = choices::choose(g, p, "choose", "Skrelv: choose a color", &labels, Some("cancel"), None)? else {
        return Ok(None);
    };
    let ch = ['W', 'U', 'B', 'R', 'G'][k];
    if !crate::impls::tymna_cards::skrelv_activate(g, m, p, t, ch, 2)? {
        return say("Can't activate Skrelv: pay {W} or 2 life.");
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

fn scoundrel(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Tavern Scoundrel";
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    if let Some(w) = cost(g, p, 1, "", W) {
        return say(w);
    }
    let mut others = mine(g, p, |_, x| x != m);
    others.retain(|&x| !g.perm(x).is_cmd);
    let pl = g.player(p);
    let tokens: Vec<(&str, u32)> = vec![("a Treasure", pl.treasures), ("a Food", pl.foods), ("a Clue", pl.clues)];
    let mut labels: Vec<String> = others.iter().map(|&x| g.perm(x).name.to_string()).collect();
    let base = labels.len();
    let avail: Vec<usize> = (0..3).filter(|&i| tokens[i].1 > 0).collect();
    labels.extend(avail.iter().map(|&i| tokens[i].0.to_string()));
    if labels.is_empty() {
        return say("You control no other permanent to sacrifice.");
    }
    let Some(k) =
        choices::choose(g, p, "choose", "Tavern Scoundrel: sacrifice which permanent?", &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    if let Some(w) = pay(g, p, 1, "", W) {
        return say(w);
    }
    tap(g, m);
    if k < base {
        sacrifice(g, p, others[k])?;
    } else {
        match avail[k - base] {
            0 => {
                crate::impls::model_gaps_a4::sac_treasure(g, p)?;
            }
            1 => g.player_mut(p).foods -= 1,
            _ => g.player_mut(p).clues -= 1,
        }
    }
    plain(g, p, m, false, "flip a coin", move |g| {
        if g.rng.random() < 0.5 {
            zones::make_artifact_tokens(g, p, "Treasure", 2)?;
        }
        Ok(())
    })
}

fn teysa(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    let whites = mine(g, p, |g, x| g.is_creature(x) && crate::engine::values::colors_of(g, x).has('W'));
    if whites.len() < 3 {
        return say("You control fewer than three white creatures to sacrifice.");
    }
    let cands = creatures(g, p, false);
    let Some(t) = pick_target(g, p, "Teysa: exile which creature?", &cands)? else { return Ok(None) };
    let Some(three) = pick_costs(g, p, "Teysa: sacrifice which white creature?", &whites, 3)? else { return Ok(None) };
    for x in three {
        sacrifice(g, p, x)?;
    }
    aimed(g, p, m, false, "exile", t, cands, |g, t| apply_removal(g, Some(p), t, "exile", None))
}

fn urza(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Urza, Powerstone Prodigy";
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    if let Some(w) = pay(g, p, 1, "", W) {
        return say(w);
    }
    tap(g, m);
    plain(g, p, m, false, "draw a card, then discard a card", move |g| {
        zones::draw(g, p, 1, false)?;
        choices::discard(g, p, 1, "Urza: discard a card")
    })
}

fn eladamri(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Eladamri, Korvecdal";
    if g.active != Some(p) {
        return say("You can only activate that during your turn.");
    }
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    if let Some(w) = cost(g, p, 0, "G", W) {
        return say(w);
    }
    let crew = mine(g, p, |g, x| g.is_creature(x) && !g.perm(x).tapped && x != m);
    if crew.len() < 2 {
        return say("You control fewer than two other untapped creatures to tap.");
    }
    let Some(two) = pick_costs(g, p, "Eladamri: tap which creature?", &crew, 2)? else { return Ok(None) };
    // the card to reveal: a card from the hand, or the top of the library
    let hand = g.player(p).hand.clone();
    let mut labels: Vec<String> = vec![];
    let top = g.player(p).library.last().copied();
    if let Some(t) = top {
        labels.push(format!("the top card of your library ({})", choices::card_label(g, t)));
    }
    labels.extend(hand.iter().map(|&c| format!("from your hand: {}", choices::card_label(g, c))));
    if labels.is_empty() {
        return say("You have no card to reveal.");
    }
    let Some(k) = choices::choose(g, p, "choose", "Eladamri: reveal which card?", &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    if let Some(w) = pay(g, p, 0, "G", W) {
        return say(w);
    }
    tap(g, m);
    for x in two {
        g.perm_mut(x).tapped = true;
        crate::impls::t0_captain::became_tapped(g, x);
    }
    let from_top = top.is_some() && k == 0;
    let c = if from_top { top.unwrap() } else { hand[k - top.is_some() as usize] };
    plain(g, p, m, false, "reveal a card", move |g| {
        let there = if from_top { g.player(p).library.last() == Some(&c) } else { g.player(p).hand.contains(&c) };
        if !there {
            return Ok(());
        }
        crate::glog!(g, "    {} reveals {}", g.player(p).name, g.db.get(c).name);
        if g.db.get(c).creature {
            if from_top {
                g.player_mut(p).library.pop();
            } else if let Some(i) = g.player(p).hand.iter().position(|&x| x == c) {
                g.player_mut(p).hand.remove(i);
            }
            zones::enter(g, p, c, zones::Enter::default())?;
        }
        Ok(())
    })
}

fn drivnod(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if !live(g, m) {
        return say("Drivnod is not on the battlefield.");
    }
    let cards: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    if cards.len() < 3 {
        return say("Your graveyard holds fewer than three creature cards.");
    }
    // {B/P}{B/P}: each pip is {B} or 2 life
    let mut pips = String::new();
    let mut life = 0;
    for i in 0..2 {
        let b = mana::cost_problem(g, p, 0, &format!("{pips}B")).is_none();
        if b && g.player(p).life > life + 2 {
            let ls = ["pay {B}".to_string(), "pay 2 life".to_string()];
            let k = choices::choose_one(g, p, "choose", &format!("Drivnod: pay {{B/P}} ({} of 2)", i + 1), &ls)?;
            if k == 0 {
                pips.push('B');
            } else {
                life += 2;
            }
        } else if b {
            pips.push('B');
        } else if g.player(p).life > life + 2 {
            life += 2;
        } else {
            return say("Can't activate Drivnod. You can't pay {B/P}{B/P}.");
        }
    }
    let Some(three) = ({
        let mut left = cards.clone();
        let mut got = vec![];
        for i in 0..3 {
            let labels: Vec<String> = left.iter().map(|&c| choices::card_label(g, c)).collect();
            let Some(k) = choices::choose(
                g,
                p,
                "choose",
                &format!("Drivnod: exile which creature card? ({} of 3)", i + 1),
                &labels,
                if i == 0 { Some("cancel") } else { None },
                None,
            )?
            else {
                got.clear();
                break;
            };
            got.push(left.remove(k));
        }
        (!got.is_empty()).then_some(got)
    }) else {
        return Ok(None);
    };
    if !pips.is_empty()
        && let Some(w) = pay(g, p, 0, &pips, "Drivnod")
    {
        return say(w);
    }
    if life > 0 {
        crate::engine::life::lose_life(g, p, life, Some(p), "other", None)?;
    }
    for c in three {
        if let Some(i) = g.player(p).gy.iter().position(|&x| x == c) {
            g.player_mut(p).gy.remove(i);
            g.player_mut(p).exile.push(c);
        }
    }
    plain(g, p, m, false, "an indestructible counter", move |g| {
        if live(g, m) {
            g.perm_mut(m).data.set(DataKey::Indestr, Val::Bool(true));
            crate::glog!(g, "    {} puts an indestructible counter on {}", g.player(p).name, g.perm(m).name);
        }
        Ok(())
    })
}

fn reclaimer(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Elvish Reclaimer";
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    if let Some(w) = cost(g, p, 2, "", W) {
        return say(w);
    }
    let lands = mine(g, p, |g, x| crate::impls::partials::of_type(g, x, crate::cards::Types::LAND));
    if lands.is_empty() {
        return say("You control no land to sacrifice.");
    }
    let Some(l) = pick_cost(g, p, "Elvish Reclaimer: sacrifice which land?", &lands)? else { return Ok(None) };
    if let Some(w) = pay(g, p, 2, "", W) {
        return say(w);
    }
    tap(g, m);
    crate::glog!(g, "  {} sacrifices {}", g.player(p).name, g.perm(l).name);
    crate::impls::lands::sac_land(g, p, l)?;
    plain(g, p, m, false, "search for a land card", move |g| {
        let got = choices::search(g, p, &|g, c| g.db.get(c).land, 1, "Elvish Reclaimer: search for a land card", true)?;
        for c in got {
            crate::impls::t12_windgrace::land_onto_bf(g, p, c, true)?;
        }
        Ok(())
    })
}

/// Brutal Hordechief: the person pays; the blocks are chosen when the opponents' creatures block (`hordechief_blocks`
/// asks a person which attacker each creature blocks)
fn hordechief(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const W: &str = "Brutal Hordechief";
    // {R/W}{R/W}: the person pays each hybrid pip with {R} or {W}
    let mut pips = String::new();
    for _ in 0..2 {
        let (r, w) = (
            mana::cost_problem(g, p, 3, &format!("{pips}R")).is_none(),
            mana::cost_problem(g, p, 3, &format!("{pips}W")).is_none(),
        );
        match (r, w) {
            (true, true) => {
                let ls = ["pay {R}".to_string(), "pay {W}".to_string()];
                let k = choices::choose_one(g, p, "choose", "Brutal Hordechief: pay {R/W} with", &ls)?;
                pips.push(if k == 0 { 'R' } else { 'W' });
            }
            (true, false) => pips.push('R'),
            (false, true) => pips.push('W'),
            _ => return say("Can't activate Brutal Hordechief. You can't pay {3}{R/W}{R/W}."),
        }
    }
    if let Some(w) = pay(g, p, 3, &pips, W) {
        return say(w);
    }
    plain(g, p, m, false, "opponents block as you choose", move |g| {
        let now = crate::impls::partials::turn_now(g);
        g.perm_mut(m).data.set(DataKey::Lure, now);
        crate::glog!(
            g,
            "  {} activates Brutal Hordechief: opponents block as {} chooses",
            g.player(p).name,
            g.player(p).name
        );
        Ok(())
    })
}

// ================================================================== batch 3: Tekuthal, Kefka (Marvin names the donor), Scourge, Lathril
/// the person pays `n` hybrid-Phyrexian pips of colour `c` ({c} or 2 life each): the mana pips and the life, or why not
fn phyrexian(g: &Game, p: PlayerId, gn: u32, n: usize, c: char, what: &str) -> Res<Result<(String, i32), String>> {
    let mut pips = String::new();
    let mut life = 0;
    for i in 0..n {
        let have = mana::cost_problem(g, p, gn, &format!("{pips}{c}")).is_none();
        let can_life = g.player(p).life > life + 2;
        match (have, can_life) {
            (true, true) => {
                let ls = [format!("pay {{{c}}}"), "pay 2 life".to_string()];
                let k = choices::choose_one(g, p, "choose", &format!("{what}: pay {{{c}/P}} ({} of {n})", i + 1), &ls)?;
                if k == 0 {
                    pips.push(c);
                } else {
                    life += 2;
                }
            }
            (true, false) => pips.push(c),
            (false, true) => life += 2,
            _ => return Ok(Err(format!("Can't activate {what}. You can't pay the {{{c}/P}} symbols."))),
        }
    }
    if mana::cost_problem(g, p, gn, &pips).is_some() {
        return Ok(Err(format!("Can't activate {what}. {}", mana::cost_problem(g, p, gn, &pips).unwrap_or_default())));
    }
    Ok(Ok((pips, life)))
}

/// Tekuthal: {1}{U/P}{U/P}, remove three counters from among other artifacts, creatures and planeswalkers you control:
/// an indestructible counter on Tekuthal (`tek`, which the text names; `m` is the source: Tekuthal, or Marvin using it)
pub fn tekuthal(g: &mut Game, p: PlayerId, m: PermId, tek: PermId) -> Res<Why> {
    const W: &str = "Tekuthal";
    if !live(g, tek) {
        return say("Tekuthal is not on the battlefield.");
    }
    if g.perm(tek).data.truthy(DataKey::Indestr) {
        return say("Tekuthal already has an indestructible counter.");
    }
    let has = |g: &Game, x: PermId| g.perm(x).plus.max(0) + g.perm(x).loyalty.unwrap_or(0).max(0);
    let mut pool: Vec<PermId> = mine(g, p, |g, x| x != m && (g.perm(x).plus > 0 || g.perm(x).loyalty.unwrap_or(0) > 0));
    let total: i32 = pool.iter().map(|&x| has(g, x)).sum();
    if total < 3 {
        return say("You control fewer than three counters on other permanents to remove.");
    }
    let (pips, life) = match phyrexian(g, p, 1, 2, 'U', W)? {
        Ok(x) => x,
        Err(w) => return say(w),
    };
    let mut taken: Vec<PermId> = vec![];
    for i in 0..3 {
        pool.retain(|&x| has(g, x) > taken.iter().filter(|&&y| y == x).count() as i32);
        let Some(x) = pick_perm(
            g,
            p,
            "choose",
            &format!("Tekuthal: remove a counter from which permanent? ({} of 3)", i + 1),
            &pool,
            if i == 0 { Some("cancel") } else { None },
        )?
        else {
            return Ok(None);
        };
        taken.push(x);
    }
    if let Some(w) = pay(g, p, 1, &pips, W) {
        return say(w);
    }
    if life > 0 {
        crate::engine::life::lose_life(g, p, life, Some(p), "other", None)?;
    }
    for x in taken {
        let y = g.perm_mut(x);
        if y.plus > 0 {
            y.plus -= 1;
        } else if let Some(l) = y.loyalty {
            y.loyalty = Some(l - 1);
        }
    }
    plain(g, p, m, false, "an indestructible counter", move |g| {
        if live(g, tek) {
            g.perm_mut(tek).data.set(DataKey::Indestr, Val::Bool(true));
            crate::glog!(g, "  {} puts an indestructible counter on Tekuthal", g.player(p).name);
        }
        Ok(())
    })
}

/// Kefka: {8}: each opponent sacrifices a permanent of their choice; transform Kefka (`kef`, which the text names)
pub fn kefka(g: &mut Game, p: PlayerId, m: PermId, kef: PermId) -> Res<Why> {
    if !live(g, kef) || g.perm(kef).data.truthy(DataKey::Ruin) {
        return say("Kefka has already transformed.");
    }
    if let Some(w) = sorcery(g, p) {
        return say(format!("Can't activate Kefka. {w}"));
    }
    if let Some(w) = mana::cost_problem(g, p, 8, "") {
        return say(format!("Can't activate Kefka. {w}"));
    }
    if !crate::impls::mine::kefka_ruin_do(g, m, p, kef)? {
        return say("Can't activate Kefka now.");
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

fn scourge(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = pay(g, p, 0, "R", "Scourge of Valkas") {
        return say(w);
    }
    plain(g, p, m, false, "+1/+0", move |g| {
        if live(g, m) {
            crate::impls::common::eot(g, m, 1, 0);
        }
        Ok(())
    })
}

fn lathril(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(w) = tap_problem(g, p, m) {
        return say(w);
    }
    let elves = mine(g, p, |g, x| g.is_creature(x) && !g.perm(x).tapped && has_type(g, x, "elf") && x != m);
    if elves.len() < 10 {
        return say("You control fewer than ten other untapped Elves to tap.");
    }
    let Some(ten) = pick_costs(g, p, "Lathril: tap which Elf?", &elves, 10)? else { return Ok(None) };
    tap(g, m);
    for x in ten {
        g.perm_mut(x).tapped = true;
        crate::impls::t0_captain::became_tapped(g, x);
    }
    plain(g, p, m, false, "each opponent loses 10 life and you gain 10", move |g| {
        for q in opponents(g, p) {
            crate::engine::life::lose_life(g, q, 10, Some(p), "drain", None)?;
        }
        crate::engine::life::gain(g, p, 10)
    })
}
