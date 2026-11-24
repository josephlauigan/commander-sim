//! Card code for the other cards of the karn_gilded deck that had no record (E5): Infinite Guideline Station, Breya,
//! Etherium Shaper, Gold-Forged Thopteryx, The Reaper, King No More, Herd Heirloom and Twitching Doll. Transguild
//! Courier is data only (its colours are `NamedFx::SelfAllColors`), and Shorikai, Genesis Engine's ability is compiled
//! (the `dsl` of its override in data/card_overrides.toml), so neither has code here. The audit verdicts are in
//! data/card_overrides.toml; each function's doc comment says how the card is modeled and what is left out.
//!
//! One path per card: every card here has `dsl = []` in its override.
//!
//! - **Triggers** (Station's and Breya's enter triggers, Station's attack trigger, The Reaper's two) go on the stack
//!   through `trigger_window`, so an opponent's Elesh Norn blanks them and the doublers repeat them.
//! - **Activated abilities** (Station, Breya's modal ability, Herd Heirloom's second ability, Twitching Doll's
//!   sacrifice) are `Opt`s of the card's `options` hook: the AI picks them by utility and a person gets them through the
//!   same hook, with their own choice of target, mode or creature where the AI would pick by value.
//! - **Mana**: Herd Heirloom ("any colour, only to cast a creature spell") is `rock=1:A` plus `Restr::CreatureSpell`
//!   (`engine::mana::restr_of`), Twitching Doll a `dork=A` creature with `on_tap_perm` adding the nest counter.
//! - **Ward**: Gold-Forged Thopteryx's ward {2} for legendary permanents is `thopteryx_ward`, read by
//!   `cardcode::aura_ward`; the grant that Herd Heirloom's second ability makes is read by
//!   `cardcode::combat_damage_cards` (`heirloom_draw`).

use crate::cards::{CardDb, Colors, Types};
use crate::engine::chars::colors;
use crate::engine::life::{check_state, gain, lose_life_dealt};
use crate::engine::mana::{can_pay, pay};
use crate::engine::repl;
use crate::engine::stack::{ability_run, trigger_window};
use crate::engine::values::{epow, etgh, ptypes, pval, untargetable};
use crate::engine::zones::{Enter, Tokens, die, draw, enter, make_tokens, max_by, sac_worth};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::sym::Sym;

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn pname(g: &Game, p: PlayerId) -> &str {
    &g.player(p).name
}

fn human(g: &Game, p: PlayerId) -> bool {
    crate::human::is_human(g, p)
}

fn ability(utility: f64, label: &str, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label: label.into(), act: Some(Action::Ability { src, f, arg }) }
}

/// an artifact permanent (a token made as an artifact creature has "artifact" among its types)
fn is_art(g: &Game, m: PermId) -> bool {
    ptypes(g, m).has(Types::ARTIFACT) || g.perm(m).ttypes.contains(&"artifact")
}

/// p's artifact permanents (the Treasure, Food and Clue tokens of this engine are counters on the player, never
/// permanents, so they are never counted or sacrificed here)
fn artifacts(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && is_art(g, m)).collect()
}

/// the index the person picks among `labels`, or None if they cancel
fn ask(g: &Game, p: PlayerId, prompt: &str, labels: &[String]) -> Res<Option<usize>> {
    crate::human::choices::choose(g, p, "choose", prompt, labels, Some("cancel"), None)
}

// ================================================================== Infinite Guideline Station
const STATION: &str = "Infinite Guideline Station";
/// its printed power and toughness, which count only while it is a creature (12 or more charge counters)
const STATION_PT: (i32, i32) = (7, 15);
/// the "12+" level: the `station=12` tag of its override is read by `engine::chars::station_met`
const STATION_GOAL: i64 = 12;

/// the multicoloured permanents p controls (two or more colours by their characteristics now: Karn, Gilded Guardian,
/// anything under a Leyline of the Guildpact, a Transguild Courier; colourless Robots and Treasures never)
pub fn multicoloured(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| live(g, m) && colors(g, m).count() >= 2).count()
}

fn charge(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.int(DataKey::Charge)
}

/// "When this enters, create a tapped 2/2 colorless Robot artifact creature token for each multicolored permanent you
/// control." Counted as the trigger resolves, the Station itself included (it is five colours).
fn station_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = crate::engine::hooks::trigger_controller(g, src);
    if !trigger_window(g, o, Some(src), "a tapped Robot for each multicolored permanent", Some(4.0))? {
        return Ok(());
    }
    let n = multicoloured(g, o);
    let spec = Tokens { color: Some(Colors::NONE), types: vec!["robot", "artifact"], ..Tokens::new(n as u32, 2) };
    let made = make_tokens(g, o, spec)?;
    for &t in &made {
        g.perm_mut(t).tapped = true; // "tapped"
    }
    crate::glog!(g, "    {}: {} tapped 2/2 Robot token(s) ({} multicolored permanents)", STATION, made.len(), n);
    g.bf_ver += 1;
    Ok(())
}

/// as it enters: its printed 7/15 (the card data has 0/0, an artifact's), the permanent's own base power and toughness,
/// so a later effect that sets them (Darksteel Mutation, Tezzeret's emblem) wins; it counts while it is a creature
fn station_as_enters(g: &mut Game, _p: PlayerId, m: PermId) -> Res {
    let x = g.perm_mut(m);
    if x.exc.pt.is_none() {
        (x.pow, x.tgh) = STATION_PT;
    }
    Ok(())
}

/// "12+ | Flying": it depends on the charge counters alone (`chars::station_met`)
fn station_grant(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    m == src && kw == "flying" && crate::engine::chars::station_met(g, g.perm(src))
}

/// "Whenever this attacks, draw a card for each multicolored permanent you control" (it attacks only as a creature)
fn station_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if !atk.contains(&src) || owner(g, src) != p {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "draw a card for each multicolored permanent", Some(4.0))? {
        return Ok(vec![]);
    }
    let n = multicoloured(g, p) as u32;
    crate::glog!(g, "    {}: {} draws {} card(s)", STATION, pname(g, p), n);
    draw(g, p, n, false)?;
    Ok(vec![])
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

fn station_met(g: &Game, src: PermId) -> bool {
    crate::engine::chars::station_met(g, g.perm(src))
}

/// Station (tap another untapped creature you control: charge counters equal to its power, only as a sorcery). The AI
/// taps the creatures that can't attack this turn anyway, or the ones whose power finishes it at once, and stops at 12;
/// a person chooses each creature.
fn station_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !live(g, src) || crate::human::legal::sorcery_timing(g, p).is_some() {
        return Ok(vec![]);
    }
    let cands = station_candidates(g, p, src);
    if cands.is_empty() {
        return Ok(vec![]);
    }
    if human(g, p) {
        return Ok(vec![ability(
            2.0,
            "Infinite Guideline Station: station (tap another creature)",
            src,
            station_go,
            -1,
        )]);
    }
    if station_met(g, src) {
        return Ok(vec![]); // the AI has nothing more to gain from it
    }
    let have = charge(g, src);
    let idle = |g: &Game, m: PermId| g.perm(m).sick || g.perm(m).noatk;
    let total: i64 = cands.iter().map(|&m| epow(g, m).max(0) as i64).sum();
    let finish = have + total >= STATION_GOAL;
    let mut best: Option<(PermId, f64)> = None;
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
        .map(|(m, u)| {
            let label = format!("Infinite Guideline Station: station (tap {})", g.perm(m).name);
            ability(u, &label, src, station_go, m.0 as i64)
        })
        .into_iter()
        .collect())
}

fn station_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    // only as a sorcery
    if !live(g, src) || owner(g, src) != p || crate::human::legal::sorcery_timing(g, p).is_some() {
        return Ok(false);
    }
    let cands = station_candidates(g, p, src);
    let t = if arg >= 0 {
        PermId(arg as u32)
    } else {
        let labels: Vec<String> =
            cands.iter().map(|&m| format!("{} (power {})", g.perm(m).name, epow(g, m).max(0))).collect();
        let prompt = "Infinite Guideline Station: tap which creature to put charge counters equal to its power on it?";
        match ask(g, p, prompt, &labels)? {
            Some(i) => cands[i],
            None => return Ok(false),
        }
    };
    if !cands.contains(&t) {
        return Ok(false);
    }
    g.perm_mut(t).tapped = true; // the cost
    crate::glog!(g, "  {} stations {}, tapping {}", pname(g, p), STATION, g.perm(t).name);
    let what = format!("station: charge counters equal to the power of {}", g.perm(t).name);
    ability_run(g, p, Some(src), None, &what, None, None, |g| {
        if !live(g, src) {
            return Ok(());
        }
        // the power as the ability resolves (a pump in response counts)
        let n = epow(g, t).max(0);
        let k = repl::counters(g, p, repl::On::Perm(src), repl::Ctr::Other, n, repl::Path::ALL).max(0);
        let total = charge(g, src) + k as i64;
        g.perm_mut(src).data.set(DataKey::Charge, Val::Int(total));
        if station_met(g, src) {
            crate::glog!(g, "    {} is a 7/15 flying artifact creature", STATION); // by its `station=12` tag (chars.rs)
        }
        Ok(())
    })?;
    Ok(true)
}

// ================================================================== Breya, Etherium Shaper
const BREYA: &str = "Breya, Etherium Shaper";

/// "When Breya enters, create two 1/1 blue Thopter artifact creature tokens with flying." Blue, with the artifact type
/// (they count for Karn's artifacts, Mox Opal's metalcraft and Breya's own cost).
fn breya_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = crate::engine::hooks::trigger_controller(g, src);
    if !trigger_window(g, o, Some(src), "two 1/1 blue Thopters", Some(4.0))? {
        return Ok(());
    }
    let spec = Tokens {
        fly: true,
        color: Some(Colors::from_letters("U")),
        types: vec!["thopter", "artifact"],
        ..Tokens::new(2, 1)
    };
    make_tokens(g, o, spec)?;
    Ok(())
}

/// the two artifacts Breya's cost sacrifices: the two p values least. Artifact permanents only, never Breya herself.
fn breya_fodder(g: &Game, p: PlayerId, src: PermId) -> Option<[PermId; 2]> {
    let mut v: Vec<PermId> = artifacts(g, p).into_iter().filter(|&m| m != src).collect();
    if v.len() < 2 {
        return None;
    }
    v.sort_by(|&a, &b| sac_worth(g, a).total_cmp(&sac_worth(g, b)));
    Some([v[0], v[1]])
}

/// the opposing creature the -4/-4 mode kills that is worth most (4 toughness or less, worth 3 or more)
fn breya_kill(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| live(g, m) && g.is_creature(m) && !untargetable(g, m) && etgh(g, m) <= 4 && pval(g, m) >= 3.0)
        .collect();
    max_by(&cs, |m| pval(g, m))
}

/// the opponent the 3 damage mode finishes (life 3 or less), the one with the least life
fn breya_burn(g: &Game, p: PlayerId) -> Option<PlayerId> {
    g.opps(p).filter(|&q| g.player(q).life <= 3).min_by_key(|&q| g.player(q).life)
}

/// what the AI does with the ability now: (mode, utility, target creature, target player). Modes: 0 = 3 damage to a
/// player, 1 = -4/-4, 2 = gain 5 life. The value is cut by what the two artifacts are worth.
type Plan = (i64, f64, Option<PermId>, Option<PlayerId>);

fn breya_plan(g: &Game, p: PlayerId, src: PermId, only: Option<i64>) -> Option<Plan> {
    let fodder = breya_fodder(g, p, src)?;
    let cost: f64 = fodder.iter().map(|&m| sac_worth(g, m)).sum();
    let mut best: Option<Plan> = None;
    let mut put = |mode: i64, v: f64, t: Option<PermId>, q: Option<PlayerId>| {
        if only.is_none_or(|o| o == mode) && v - 0.4 * cost > 0.5 && best.as_ref().is_none_or(|b| v > b.1) {
            best = Some((mode, v, t, q));
        }
    };
    if let Some(q) = breya_burn(g, p) {
        put(0, 8.0, None, Some(q));
    }
    if let Some(t) = breya_kill(g, p) {
        put(1, 1.0 + 0.5 * pval(g, t).min(8.0), Some(t), None);
    }
    if g.player(p).life <= 10 {
        put(2, 1.5 + 0.2 * (11 - g.player(p).life) as f64, None, None);
    }
    best
}

fn breya_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !live(g, src) || g.perm(src).neutered {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    if human(g, p) {
        if breya_fodder(g, p, src).is_none() {
            return Ok(vec![]);
        }
        return Ok(vec![ability(1.5, "Breya: {2}, sacrifice two artifacts: choose one", src, breya_go, -1)]);
    }
    Ok(breya_plan(g, p, src, None).map(|(mode, u, ..)| ability(u, "Breya", src, breya_go, mode)).into_iter().collect())
}

/// "{2}, Sacrifice two artifacts: Choose one: 3 damage to target player or planeswalker; target creature gets -4/-4 until
/// end of turn; you gain 5 life." `arg` is the AI's mode (a person, -1, picks the mode, the target and the two
/// artifacts). The damage mode takes a player: this engine has no damage to a planeswalker outside combat.
fn breya_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).neutered {
        return Ok(false);
    }
    let (mode, fodder, target, victim) = if arg >= 0 {
        let Some((mode, _, t, q)) = breya_plan(g, p, src, Some(arg)) else { return Ok(false) };
        let Some(f) = breya_fodder(g, p, src) else { return Ok(false) };
        (mode, f.to_vec(), t, q)
    } else {
        let modes = vec![
            "3 damage to target player".to_string(),
            "target creature gets -4/-4 until end of turn".to_string(),
            "you gain 5 life".to_string(),
        ];
        let Some(mode) = ask(g, p, "Breya: choose one", &modes)? else { return Ok(false) };
        let (mut t, mut q) = (None, None);
        if mode == 0 {
            let opps: Vec<PlayerId> = g.opps(p).collect();
            let labels: Vec<String> = opps.iter().map(|&o| pname(g, o).to_string()).collect();
            let Some(i) = ask(g, p, "Breya: 3 damage to which player?", &labels)? else { return Ok(false) };
            q = Some(opps[i]);
        } else if mode == 1 {
            let cs = crate::human::cards::creatures(g, p, false);
            if cs.is_empty() {
                return Ok(false);
            }
            let prompt = "Breya: -4/-4 on which creature?";
            let Some(m) = crate::human::cards::pick_perm(g, p, "target", prompt, &cs, Some("cancel"))? else {
                return Ok(false);
            };
            t = Some(m);
        }
        let arts: Vec<PermId> = artifacts(g, p);
        if arts.len() < 2 {
            return Ok(false);
        }
        let picked = crate::human::cards::pick_perms(g, p, "sacrifice", "Breya: sacrifice which artifacts?", &arts, 2)?;
        (mode as i64, picked, t, q)
    };
    if fodder.len() < 2 || fodder.iter().any(|&m| !live(g, m) || owner(g, m) != p) {
        return Ok(false);
    }
    if !pay(g, p, 2, "", false)? {
        return Ok(false);
    }
    for m in fodder {
        die(g, m, "sac")?; // the cost: two artifacts
    }
    let what = match mode {
        0 => "3 damage to target player",
        1 => "target creature gets -4/-4",
        _ => "you gain 5 life",
    };
    ability_run(g, p, Some(src), None, what, Some(5.0), target, |g| {
        match mode {
            0 => {
                if let Some(q) = victim {
                    lose_life_dealt(g, q, 3, Some(p), "triggers", None)?;
                }
            }
            1 => {
                if let Some(t) = target.filter(|&t| live(g, t)) {
                    let foreign = owner(g, t) != p;
                    if !(foreign && untargetable(g, t)) && (!foreign || crate::engine::ward::pay_ward(g, p, t)?) {
                        let x = g.perm_mut(t);
                        x.eot_pt = (x.eot_pt.0 - 4, x.eot_pt.1 - 4);
                        crate::glog!(g, "    {} gets -4/-4 until end of turn (Breya)", g.perm(t).name);
                        for m in g.players.iter().flat_map(|q| q.perms.clone()).collect::<Vec<_>>() {
                            if g.perm(m).on_bf && g.is_creature(m) && etgh(g, m) <= 0 {
                                die(g, m, "sba")?;
                            }
                        }
                    }
                }
            }
            _ => gain(g, p, 5)?,
        }
        check_state(g)
    })?;
    Ok(true)
}

// ================================================================== Gold-Forged Thopteryx
const THOPTERYX: &str = "Gold-Forged Thopteryx";

/// "Each legendary permanent you control has ward {2}": 2 for each Thopteryx its controller has (each gives its own
/// ward {2}, and wards add). Read by `cardcode::aura_ward`, as Rith's and Innkeeper's Talent's are.
pub fn thopteryx_ward(g: &Game, m: PermId) -> u32 {
    if !crate::impls::legends::legendary(g, m) {
        return 0;
    }
    let n = g
        .player(owner(g, m))
        .perms
        .iter()
        .filter(|&&t| {
            live(g, t) && !g.perm(t).neutered && g.perm(t).cd.is_some_and(|c| &*g.db.get(c).name == THOPTERYX)
        })
        .count() as u32;
    2 * n
}

// ================================================================== The Reaper, King No More
const REAPER: &str = "The Reaper, King No More";

/// the opposing creatures The Reaper's enter trigger shrinks: the two worth most that can be targeted
fn reaper_targets(g: &Game, p: PlayerId, src: PermId) -> Vec<PermId> {
    let mut cs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| m != src && live(g, m) && g.is_creature(m) && !untargetable(g, m) && repl::minus_ok(g, m))
        .collect();
    cs.sort_by(|&a, &b| pval(g, b).total_cmp(&pval(g, a)));
    cs.truncate(2);
    cs
}

/// "When The Reaper enters, put a -1/-1 counter on each of up to two target creatures." The AI shrinks the two opposing
/// creatures worth most (a creature with a -1/-1 counter is what its second ability feeds on); a person chooses up to
/// two creatures. An opposing creature's ward is paid or the creature is skipped.
fn reaper_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = crate::engine::hooks::trigger_controller(g, src);
    let targets: Vec<PermId> = if human(g, o) {
        let mut left = crate::human::cards::creatures(g, o, false);
        let mut picked = vec![];
        for i in 0..2 {
            if left.is_empty() {
                break;
            }
            let prompt = format!("The Reaper: put a -1/-1 counter on which creature? ({} of up to 2)", i + 1);
            match crate::human::cards::pick_perm(g, o, "target", &prompt, &left, Some("no more targets"))? {
                Some(t) => {
                    left.retain(|&x| x != t);
                    picked.push(t);
                }
                None => break,
            }
        }
        picked
    } else {
        reaper_targets(g, o, src)
    };
    if targets.is_empty() || !trigger_window(g, o, Some(src), "a -1/-1 counter on up to two creatures", Some(5.0))? {
        return Ok(());
    }
    for t in targets {
        if !live(g, t) || (owner(g, t) != o && (untargetable(g, t) || !crate::engine::ward::pay_ward(g, o, t)?)) {
            continue;
        }
        if repl::minus_ok(g, t) {
            repl::put_counters(g, o, t, -1);
            crate::glog!(g, "    The Reaper: a -1/-1 counter on {}", g.perm(t).name);
        }
    }
    g.bf_ver += 1;
    check_state(g)
}

/// "Whenever a creature an opponent controls with a -1/-1 counter on it dies, you may put that card onto the battlefield
/// under your control. Do this only once each turn." The counter is read from the creature as it died (the permanent
/// record keeps its counters), the card is taken from its owner's graveyard; a token has no card to take. The AI always
/// does it.
fn reaper_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let p = owner(g, src);
    if !live(g, src) || g.perm(src).neutered || m == src {
        return Ok(());
    }
    let (ctl, plus, token, card, orig) = {
        let x = g.perm(m);
        (x.owner, x.plus, x.token, x.phys.or(x.cd), x.orig)
    };
    if ctl == p || plus >= 0 || token || !g.is_creature(m) {
        return Ok(());
    }
    let Some(cd) = card else { return Ok(()) };
    let st = g.turn_stamp();
    let used = |g: &Game| g.perm(src).data.get(DataKey::Used) == Some(&Val::Stamp(st));
    if used(g) || crate::impls::t45_winota::jailed(g) || !g.player(orig).gy.contains(&cd) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "put the creature onto the battlefield under your control", Some(5.0))? {
        return Ok(());
    }
    // once each turn, checked again as the trigger resolves: a second one that turn does nothing
    if used(g) || !g.player(orig).gy.contains(&cd) {
        return Ok(());
    }
    g.perm_mut(src).data.set(DataKey::Used, Val::Stamp(st));
    crate::impls::partials::remove_card(&mut g.player_mut(orig).gy, cd);
    enter(g, p, cd, Enter { orig: Some(orig), ..Enter::default() })?;
    crate::glog!(g, "    {}: {} comes back under {}'s control", REAPER, g.db.get(cd).name, pname(g, p));
    Ok(())
}

// ================================================================== Herd Heirloom
const HEIRLOOM: &str = "Herd Heirloom";
/// the end-of-turn mark of a creature Herd Heirloom's second ability gave "whenever this creature deals combat damage to
/// a player, draw a card" (`eot_kw` is cleared as the turn ends)
const HEIRLOOM_DRAW: &str = "heirloom_draw";

/// the creature the ability is worth putting on: ours, power 4 or more, not marked yet, one that can attack (not sick,
/// not tapped, may attack), the one with the most power
fn heirloom_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            live(g, m)
                && g.is_creature(m)
                && epow(g, m) >= 4
                && !x.eot_kw.contains(&HEIRLOOM_DRAW)
                && !x.sick
                && !x.tapped
                && !x.noatk
        })
        .collect();
    max_by(&cs, |m| epow(g, m) as f64)
}

fn heirloom_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !live(g, src) || g.perm(src).tapped || g.perm(src).neutered {
        return Ok(vec![]);
    }
    if human(g, p) {
        let any = g.player(p).perms.iter().any(|&m| live(g, m) && g.is_creature(m) && epow(g, m) >= 4);
        let label = "Herd Heirloom: target creature with power 4 or more gains trample and draws a card";
        return Ok(if any { vec![ability(1.0, label, src, heirloom_go, -1)] } else { vec![] });
    }
    // before combat only: it is the attacker that gets the draw
    if post != Some(false) || g.active != Some(p) || heirloom_target(g, p).is_none() {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.4, "Herd Heirloom", src, heirloom_go, -2)])
}

/// "{T}: Until end of turn, target creature you control with power 4 or greater gains trample and 'Whenever this
/// creature deals combat damage to a player, draw a card.'" The grant stays on the creature (`eot_kw`), so it draws
/// even if the Heirloom is gone by then (`heirloom_draw`, called from `cardcode::combat_damage_cards`). `arg` -2 is the
/// AI's pick, -1 a person's.
fn heirloom_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped || g.perm(src).neutered {
        return Ok(false);
    }
    let t = if arg == -2 {
        let Some(t) = heirloom_target(g, p) else { return Ok(false) };
        t
    } else {
        let cs: Vec<PermId> =
            g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m) && epow(g, m) >= 4).collect();
        if cs.is_empty() {
            return Ok(false);
        }
        let prompt = "Herd Heirloom: which creature?";
        let Some(t) = crate::human::cards::pick_perm(g, p, "target", prompt, &cs, Some("cancel"))? else {
            return Ok(false);
        };
        t
    };
    g.perm_mut(src).tapped = true; // the cost
    ability_run(g, p, Some(src), None, "trample and a card for combat damage", Some(2.0), Some(t), |g| {
        if live(g, t) && epow(g, t) >= 4 {
            crate::impls::partials::eot_kw(g, t, "trample");
            crate::impls::partials::eot_kw(g, t, HEIRLOOM_DRAW);
            crate::glog!(
                g,
                "    {} gains trample and draws on combat damage to a player (Herd Heirloom)",
                g.perm(t).name
            );
        }
        Ok(())
    })?;
    Ok(true)
}

/// a creature of p's that has Herd Heirloom's grant dealt combat damage to a player: draw a card (called for each damage
/// event, so double strike draws twice)
pub fn heirloom_draw(g: &mut Game, p: PlayerId, a: PermId, dmg: i32) -> Res {
    if dmg > 0 && g.perm(a).eot_kw.contains(&HEIRLOOM_DRAW) {
        crate::glog!(g, "    Herd Heirloom: {} draws a card ({})", pname(g, p), g.perm(a).name);
        draw(g, p, 1, false)?;
    }
    Ok(())
}

// ================================================================== Twitching Doll
const DOLL: &str = "Twitching Doll";

fn nest(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.int(DataKey::Counters)
}

/// "{T}: Add one mana of any color. Put a nest counter on this creature." The mana is the `dork=A` tag; the counter is
/// put as the mana ability is paid, through the counter replacements (Vorinclex doubles it).
fn doll_tap(g: &mut Game, p: PlayerId, m: PermId, _n: u32) -> Res {
    let k = repl::counters(g, p, repl::On::Perm(m), repl::Ctr::Other, 1, repl::Path::ALL).max(0) as i64;
    let n = nest(g, m) + k;
    g.perm_mut(m).data.set(DataKey::Counters, Val::Int(n));
    Ok(())
}

/// the AI sacrifices it once it has three nest counters or more (three 2/2 Spiders for a mana creature), untapped, at
/// sorcery timing; a person has it listed with any counters
fn doll_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !live(g, src) || g.perm(src).neutered {
        return Ok(vec![]);
    }
    let x = g.perm(src);
    if x.tapped || x.sick || crate::human::legal::sorcery_timing(g, p).is_some() {
        return Ok(vec![]);
    }
    let n = nest(g, src);
    if n <= 0 || (!human(g, p) && n < 3) {
        return Ok(vec![]);
    }
    let label = format!("Twitching Doll: {{T}}, sacrifice ({n} nest counters)");
    Ok(vec![ability(0.8 + 0.5 * n as f64, &label, src, doll_go, 0)])
}

/// "{T}, Sacrifice this creature: Create a 2/2 green Spider creature token with reach for each counter on this
/// creature. Activate only as a sorcery." The counters are counted as the cost is paid.
fn doll_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let x = g.perm(src);
    if !live(g, src) || owner(g, src) != p || x.tapped || x.sick || crate::human::legal::sorcery_timing(g, p).is_some()
    {
        return Ok(false);
    }
    let n = nest(g, src).max(0);
    g.perm_mut(src).tapped = true;
    die(g, src, "sac")?;
    ability_run(g, p, Some(src), None, "a 2/2 Spider with reach for each nest counter", Some(4.0), None, |g| {
        let mut spec =
            Tokens { color: Some(Colors::from_letters("G")), types: vec!["spider"], ..Tokens::new(n as u32, 2) };
        spec.data.set(DataKey::Kws, Val::List(vec![Val::Str("reach")]));
        make_tokens(g, p, spec)?;
        crate::glog!(g, "    Twitching Doll: {} Spider token(s)", n);
        Ok(())
    })?;
    Ok(true)
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, STATION)?;
    c.etb = Some(station_etb);
    c.as_enters = Some(station_as_enters);
    c.grant_kw = Some(station_grant);
    c.attack = Some(station_attack);
    c.options = Some(station_options);
    let c = r.card(db, BREYA)?;
    c.etb = Some(breya_etb);
    c.options = Some(breya_options);
    let c = r.card(db, REAPER)?;
    c.etb = Some(reaper_etb);
    c.dies = Some(reaper_dies);
    r.card(db, HEIRLOOM)?.options = Some(heirloom_options);
    let c = r.card(db, DOLL)?;
    c.on_tap_perm = Some(doll_tap);
    c.options = Some(doll_options);
    Ok(())
}
