//! Card code for the other cards of the karn_gilded deck that had no record (E5): Infinite Guideline Station, Breya,
//! Etherium Shaper, Gold-Forged Thopteryx, The Reaper, King No More, Dread Tiller, Herd Heirloom and Twitching Doll.
//! Transguild Courier (out of the deck since 2026-11-25, in no deck now) keeps only its data (its colours are
//! `NamedFx::SelfAllColors`), and Shorikai, Genesis Engine's ability is compiled (the `dsl` of its override in
//! data/card_overrides.toml), so neither has code here. The audit verdicts are in
//! data/card_overrides.toml; each function's doc comment says how the card is modeled and what is left out.
//!
//! One path per card: every card here has `dsl = []` in its override.
//!
//! - **Triggers** (Station's and Breya's enter triggers, Station's attack trigger, The Reaper's two, Dread Tiller's two)
//!   go on the stack through `trigger_window`, so an opponent's Elesh Norn blanks them and the doublers repeat them.
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
use crate::engine::stack::{ability_run, ability_run_w, trigger_window};
use crate::engine::values::{
    colors_of, epow, etgh, protected_from, ptypes, pval, untargetable, untargetable_by_its_controller,
};
use crate::engine::zones::{Enter, TOKEN_CAP, Tokens, die, draw, enter, make_tokens, max_by, sac_worth, token_copy_of};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::karn_gilded::{PER_CARD, PER_DISCARD, copy_room};
use crate::state::{DataKey, Game, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

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
    // a creature with haste (the Crystal's, Greaves) can attack the turn it came, so it is not idle
    let idle = |g: &Game, m: PermId| (g.perm(m).sick && !crate::engine::combat::has_haste(g, m)) || g.perm(m).noatk;
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

/// can `src`'s ability or trigger target `m`: not hexproof or shroud against an opponent, and no protection from one of
/// the source's colours (Breya is white, blue, black and red; The Reaper black, red and green)
fn targetable_by(g: &Game, src: PermId, m: PermId) -> bool {
    if owner(g, m) == owner(g, src) {
        // our own creature: hexproof lets us target it, shroud (the keyword, Lightning Greaves, Whispersilk Cloak) and
        // protection from a colour of the source do not
        return !untargetable_by_its_controller(g, m, colors_of(g, src));
    }
    !(untargetable(g, m) || protected_from(g, m, colors_of(g, src)))
}

/// the opposing creature the -4/-4 mode kills that is worth most (4 toughness or less, worth 3 or more), one whose ward
/// the AI can pay on top of Breya's own {2}
fn breya_kill(g: &Game, p: PlayerId, src: PermId) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| {
            live(g, m)
                && g.is_creature(m)
                && targetable_by(g, src, m)
                && etgh(g, m) <= 4
                && pval(g, m) >= 3.0
                && crate::engine::ward::ward_affordable(g, p, m, 2, "")
        })
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
    if let Some(t) = breya_kill(g, p, src) {
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
            let cs: Vec<PermId> =
                crate::human::cards::creatures(g, p, false).into_iter().filter(|&m| targetable_by(g, src, m)).collect();
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
                    if targetable_by(g, src, t) && (!foreign || crate::engine::ward::pay_ward(g, p, t)?) {
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
        .filter(|&m| m != src && live(g, m) && g.is_creature(m) && targetable_by(g, src, m) && repl::minus_ok(g, m))
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
        let mut left: Vec<PermId> =
            crate::human::cards::creatures(g, o, false).into_iter().filter(|&m| targetable_by(g, src, m)).collect();
        if g.trig_probe {
            // only whether the trigger goes on the stack: the targets are chosen (and a person asked) once, as it does
            if left.is_empty() {
                return Ok(());
            }
            return trigger_window(g, o, Some(src), "a -1/-1 counter on up to two creatures", Some(5.0)).map(|_| ());
        }
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
        if !live(g, t) || !targetable_by(g, src, t) || (owner(g, t) != o && !crate::engine::ward::pay_ward(g, o, t)?) {
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

// ================================================================== Dread Tiller
const TILLER: &str = "Dread Tiller";
/// the value the AI gives a -1/-1 counter that kills an opposing creature: the death feeds the Tiller's own second
/// ability (a land) and any Reaper, so every such target ranks above every counter that does not kill
const TILLER_KILL: f64 = 100.0;
/// the harm of a -1/-1 counter on the Tiller itself (2/4 becomes 1/3): less than on any other creature of ours
const TILLER_SELF_HARM: f64 = 0.1;
/// the AI never puts a counter that kills one of its own creatures, unless nothing else can be targeted
const TILLER_OWN_KILL: f64 = -1000.0;
/// the life total at or under which the AI no longer puts a land that costs life to tap (Mana Confluence, City of Brass)
const TILLER_LIFE_FLOOR: i32 = 10;
/// the AI's value for each colour of the deck's identity that none of its lands makes yet, which the land would add
const TILLER_NEW_COLOUR: f64 = 10.0;
/// the utility lands the AI prefers, best first: a land that makes no new colour is ranked by this list (the points are
/// the position from the end, so Cabal Coffers is worth the most); any other land is worth 0, and Mana Confluence and
/// City of Brass half a point less than that
const TILLER_UTILITY: [&str; 6] = [
    "Cabal Coffers",
    "Urborg, Tomb of Yawgmoth",
    "Reliquary Tower",
    "Mistrise Village",
    "Path of Ancestry",
    "Command Tower",
];
/// the hand size the cleanup step keeps (`turn::cleanup` discards while a hand holds more)
const TILLER_HAND_LIMIT: usize = 7;
/// the tie-break between two copies of the same land: the graveyard one goes first, so the hand keeps its choice, except
/// that a hand at the cleanup limit goes first
const TILLER_ZONE: f64 = 0.5;

/// the creatures the enters trigger may target: any creature on the battlefield (ours, an opponent's, the Tiller
/// itself) that `src` may target
fn tiller_targets(g: &Game, src: PermId) -> Vec<PermId> {
    g.players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| live(g, m) && g.is_creature(m) && targetable_by(g, src, m))
        .collect()
}

/// would t come back as it dies, with a +1/+1 counter, so that the counter that kills it only makes it bigger: undying
/// (`zones::die_rest`: a card of its owner's with the keyword and no +1/+1 counter now) or its owner's Mikaeus, the Unhallowed
/// (a nonhuman creature card, not a commander, not Mikaeus; the AI's one that already came back does not again). The
/// cap on returns in one turn (`zones::returns_left`) is not looked at: it changes state
fn tiller_returns(g: &Game, t: PermId) -> bool {
    let x = g.perm(t);
    let Some(cd) = x.cd else { return false };
    let p = x.owner;
    if x.token || x.orig != p || x.plus > 0 || g.player(p).is_commander(cd) || repl::no_graveyard(g, p) {
        return false;
    }
    let d = g.db.get(cd);
    let mikaeus = crate::engine::values::has(g, p, Tag::Mikaeus)
        && d.creature
        && !d.tag(Tag::Human)
        && !d.tag(Tag::Mikaeus)
        && (human(g, p) || !x.undying);
    mikaeus || d.has_kw("undying")
}

/// what the AI thinks a counter on t is worth to o (see `tiller_etb`): an opposing creature it kills is worth most, then
/// the opposing creature worth most (`pval`); a creature that can't have the counter is worth nothing either way; one of
/// ours is a loss, the least for the Tiller itself, the most for a creature the counter would kill
fn tiller_worth(g: &Game, o: PlayerId, src: PermId, t: PermId) -> f64 {
    let opposing = owner(g, t) != o;
    if !repl::minus_ok(g, t) {
        return 0.0;
    }
    let dies = etgh(g, t) <= 1;
    if opposing {
        if !crate::engine::ward::ward_affordable(g, o, t, 0, "") {
            return 0.0; // the ward counters the trigger: nothing happens, which costs nothing
        }
        if dies && tiller_returns(g, t) {
            return -pval(g, t); // it dies and comes back with a +1/+1 counter: a gift, not a kill
        }
        return if dies { TILLER_KILL + pval(g, t) } else { pval(g, t) };
    }
    if dies {
        return TILLER_OWN_KILL;
    }
    if t == src { -TILLER_SELF_HARM } else { -(TILLER_SELF_HARM + pval(g, t)) }
}

/// "When this creature enters, put a -1/-1 counter on target creature." Any creature on the battlefield, ours or an
/// opponent's, the Tiller itself too; the trigger must target, so it goes on the least harmful creature of ours when
/// nothing of an opponent's can be used. The AI: an opposing creature the counter kills, the one worth most first (the
/// death feeds the second ability); else the opposing creature worth most; else a creature of ours that can't have the
/// counter; else the Tiller itself; else our least valuable creature that survives; a creature of ours that it would kill
/// last of all. An opposing creature whose ward the AI can't pay is worth nothing to it (the ward counters the trigger and
/// no counter is put), like a creature that can't take the counter: above shrinking a creature of ours. A person is asked
/// (no way to decline) and pays the ward as it resolves, and the creature is then skipped if they don't. A creature that
/// can't have -1/-1 counters (Melira, Sylvok Outcast) is a legal target but gets none.
fn tiller_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = crate::engine::hooks::trigger_controller(g, src);
    let cands = tiller_targets(g, src);
    if cands.is_empty() {
        return Ok(());
    }
    const NAME: &str = "a -1/-1 counter on target creature";
    if g.trig_probe {
        // only whether the trigger goes on the stack: its target is chosen (and a person asked) once, as it does
        return trigger_window(g, o, Some(src), NAME, Some(5.0)).map(|_| ());
    }
    let target = if human(g, o) {
        let prompt = "Dread Tiller: put a -1/-1 counter on which creature?";
        crate::human::cards::pick_perm(g, o, "target", prompt, &cands, None)?.unwrap_or(cands[0])
    } else {
        max_by(&cands, |t| tiller_worth(g, o, src, t)).unwrap_or(cands[0])
    };
    if !trigger_window(g, o, Some(src), NAME, Some(5.0))? {
        return Ok(());
    }
    if !live(g, target) || !targetable_by(g, src, target) {
        return Ok(());
    }
    if owner(g, target) != o && !crate::engine::ward::pay_ward(g, o, target)? {
        return Ok(());
    }
    if repl::minus_ok(g, target) {
        repl::put_counters(g, o, target, -1);
        crate::glog!(g, "    {}: a -1/-1 counter on {}", TILLER, g.perm(target).name);
    }
    g.bf_ver += 1;
    check_state(g)
}

/// the land cards p may put onto the battlefield: (card, true if it is in the graveyard), one entry for each card of each
/// zone (copies of a basic land are one card)
fn tiller_lands(g: &Game, p: PlayerId) -> Vec<(CardId, bool)> {
    let mut out: Vec<(CardId, bool)> = vec![];
    for (zone, gy) in [(&g.player(p).hand, false), (&g.player(p).gy, true)] {
        for &c in zone {
            if g.db.get(c).land && !out.contains(&(c, gy)) {
                out.push((c, gy));
            }
        }
    }
    out
}

/// the colours a land card taps for (its tag: 'A' is the deck's whole identity, 'C' none)
fn tiller_land_colours(g: &Game, p: PlayerId, c: CardId) -> Colors {
    match g.db.get(c).tags.str(Tag::C).unwrap_or("C") {
        "A" => g.player(p).ident,
        "C" => Colors::NONE,
        s => Colors::from_letters(s),
    }
}

/// the AI's worth of putting land card c (in the graveyard if `gy`) onto the battlefield, None if it never would: a land
/// that costs life to tap when the life is at or under `TILLER_LIFE_FLOOR`. More for each colour of the deck's identity
/// that none of its lands makes now, then the utility lands, then the graveyard copy (the hand copy when the hand is at
/// the limit and p has no way around it)
fn tiller_land_worth(g: &Game, p: PlayerId, c: CardId, gy: bool, made: Colors, hand_full: bool) -> Option<f64> {
    let d = g.db.get(c);
    let pays = matches!(&*d.name, "Mana Confluence" | "City of Brass");
    if pays && g.player(p).life <= TILLER_LIFE_FLOOR {
        return None;
    }
    let ident = g.player(p).ident;
    let own = tiller_land_colours(g, p, c);
    let new = "WUBRG".chars().filter(|&ch| ident.has(ch) && own.has(ch) && !made.has(ch)).count();
    let utility = TILLER_UTILITY.iter().position(|&n| n == &*d.name).map_or(0.0, |i| (TILLER_UTILITY.len() - i) as f64);
    let zone = if gy != hand_full { TILLER_ZONE } else { 0.0 };
    Some(TILLER_NEW_COLOUR * new as f64 + utility + zone - if pays { 0.5 } else { 0.0 })
}

/// "Whenever a creature with a -1/-1 counter on it dies, you may put a land card from your hand or graveyard onto the
/// battlefield tapped." Any creature on any side that dies with a -1/-1 counter on it (read from the creature as it died:
/// the engine nets +1/+1 and -1/-1 counters in `plus`, so a creature whose counters cancel has none), a token, the Tiller
/// itself, and a creature that died to its own enters trigger; each Tiller triggers on its own. The Tiller sees its own
/// death (`self_dies`) and that of every creature that dies while it is on the battlefield (`dies`). The AI always does it
/// when it has a land card in either zone, and puts the land `tiller_land_worth` ranks first; a person is asked which land
/// (each card of each zone is listed) and may decline. The land is not played (no land drop) and enters tapped, with its
/// own enters ability and landfall.
fn tiller_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if g.perm(src).neutered || g.perm(m).plus >= 0 || !g.is_creature(m) {
        return Ok(());
    }
    let p = crate::engine::hooks::trigger_controller(g, src);
    if !g.player(p).alive || tiller_lands(g, p).is_empty() {
        return Ok(());
    }
    if !trigger_window(
        g,
        p,
        Some(src),
        "put a land card from your hand or graveyard onto the battlefield tapped",
        Some(3.0),
    )? {
        return Ok(());
    }
    let lands = tiller_lands(g, p); // read again as the trigger resolves
    if lands.is_empty() {
        return Ok(());
    }
    let pick = if human(g, p) {
        let labels: Vec<String> = lands
            .iter()
            .map(|&(c, gy)| format!("{} ({})", g.db.get(c).name, if gy { "graveyard" } else { "hand" }))
            .collect();
        let prompt = "Dread Tiller: put a land from your hand or graveyard onto the battlefield tapped?";
        match crate::human::choices::choose(g, p, "choose", prompt, &labels, Some("decline"), None)? {
            Some(i) => lands[i],
            None => return Ok(()),
        }
    } else {
        let made =
            g.player(p).lands.iter().fold(Colors::NONE, |a, &l| a.union(crate::engine::mana::land_colors_now(g, p, l)));
        let hand_full = !crate::engine::turn::no_max_hand(g, p)
            && crate::engine::values::hand_proper(g, p).len() > TILLER_HAND_LIMIT;
        let best = lands
            .iter()
            .filter_map(|&(c, gy)| tiller_land_worth(g, p, c, gy, made, hand_full).map(|w| ((c, gy), w)))
            .fold(
                None,
                |best: Option<((CardId, bool), f64)>, x| if best.is_none_or(|b| x.1 > b.1) { Some(x) } else { best },
            );
        let Some((pick, _)) = best else { return Ok(()) };
        pick
    };
    let (cd, gy) = pick;
    let zone = if gy { &mut g.player_mut(p).gy } else { &mut g.player_mut(p).hand };
    if !crate::impls::partials::remove_card(zone, cd) {
        return Ok(());
    }
    let l = g.add_land(p, cd, true);
    crate::cardcode::land_etb(g, p, l)?;
    crate::engine::turn::land_arrives(g, p, l, cd)?;
    crate::glog!(g, "    {}: {} puts {} onto the battlefield tapped", TILLER, pname(g, p), g.db.get(cd).name);
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
                && (!x.sick || crate::engine::combat::has_haste(g, m))
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
    if x.tapped || crate::engine::combat::tap_sick(g, src) || crate::human::legal::sorcery_timing(g, p).is_some() {
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
    if !live(g, src)
        || owner(g, src) != p
        || x.tapped
        || crate::engine::combat::tap_sick(g, src)
        || crate::human::legal::sorcery_timing(g, p).is_some()
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

// ================================================================== The Fire Crystal
const CRYSTAL: &str = "The Fire Crystal";

// "Red spells you cast cost {1} less to cast." and "Creatures you control have haste." are compiled (the `static:cost`
// and `static:keyword` of its override); the copy ability is card code: a person's ability (`human::cards::abilities`,
// `fire_crystal_go`) and the AI's option (`crystal_options`, below). Both end in `crystal_activate` and
// `crystal_resolve`, so the stack window and the copy are the same for a person and for the AI.
//
// **The AI** (PROVISIONAL: no constant here has been tuned by a sim; the switches are `AI_CRYSTAL` and
// `AI_CRYSTAL_HOLD`, `ai::table_fixes`).
// The Crystal untaps only in its controller's untap step, so there is one activation per turn cycle, and the
// end-of-turn window just before the controller's own turn (`ai::end_of_turn_window`) is the best moment: the {4}{R}{R}
// is whatever p left unspent in its own turn and did not use in the opponents' (the window makes no mana: the lands
// untap only in p's untap step, which comes right after), so what is spent there is free, and without the hold rule the
// AI spends its mana on anything with a utility above "stop", so the window sees six spare mana only late in the game;
// the copy is there for the whole of the controller's turn (it can attack and use {T} abilities, haste being the Crystal's own) and a Karn
// copy's cards arrive before the draw step. So in that window the AI copies the best target by `copy_value` when it is
// worth at least `FC_EOT_MIN`; in a main phase it copies only to win (a copy of
// Thassa's Oracle that empties the library into its trigger). The hold rule (`crystal_hold`, read by
// `ai::brain::main_options`) keeps the mana up in the main phase when that copy is worth three cards or more.

/// may p's Crystal target its own creature m: not shrouded and not protected from red, the Crystal's colour (hexproof
/// stops only an opponent)
fn crystal_may_target(g: &Game, p: PlayerId, src: PermId, m: PermId) -> bool {
    crate::impls::vondam_elspeth::may_target(g, p, m) && !protected_from(g, m, colors_of(g, src))
}

/// the creatures the Crystal's ability can target: p's own, tokens with no card record included (`zones::token_copy_of`
/// copies those too), that it may target
fn crystal_targets(g: &Game, p: PlayerId, src: PermId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.is_creature(m) && crystal_may_target(g, p, src, m))
        .collect()
}

/// the ability as it resolves (or as a copy of it resolves): if the target is still a creature of ours that the Crystal
/// may target, p creates a token that's a copy of it (`zones::token_copy_of`: the state-based actions run before the
/// copy's triggers go on the stack), marked with p, the player told to sacrifice it at the next end step
fn crystal_resolve(g: &mut Game, p: PlayerId, src: PermId, t: PermId) -> Res {
    if !live(g, t) || !g.is_creature(t) || owner(g, t) != p || !crystal_may_target(g, p, src, t) {
        return Ok(());
    }
    token_copy_of(g, p, t, &[(DataKey::CopySac, Val::Player(p))])?;
    Ok(())
}

/// "{4}{R}{R}, {T}: Create a token that's a copy of target creature you control. Sacrifice it at the beginning of the
/// next end step." Any time its controller has priority. The target is chosen, then the cost is paid from the pool, then
/// the ability goes on the stack (the table may respond: a removed target makes it do nothing). The token is marked
/// `DataKey::CopySac`: `turn::end_step` sacrifices it at the next end step to begin, whoever's turn it is, and a token
/// made during an end step waits for the following one. The copy has haste from the Crystal's own static ability, and
/// its enters trigger is the copied card's (a copy of Karn: the legend rule removes one Karn first, then the trigger draws).
pub fn fire_crystal_go(g: &mut Game, p: PlayerId, src: PermId) -> Res<Option<String>> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).neutered {
        return Ok(Some("The Fire Crystal can't be activated now.".into()));
    }
    if g.perm(src).tapped {
        return Ok(Some("The Fire Crystal is tapped.".into()));
    }
    let cs = crystal_targets(g, p, src);
    if cs.is_empty() {
        return Ok(Some("You control no creature to copy.".into()));
    }
    if let Some(why) = crate::human::mana::cost_problem(g, p, 4, "RR") {
        return Ok(Some(format!("Can't activate The Fire Crystal. {why}")));
    }
    let prompt = "The Fire Crystal: copy which creature?";
    let Some(t) = crate::human::cards::pick_perm(g, p, "target", prompt, &cs, Some("cancel"))? else {
        return Ok(None); // targets first, then the costs
    };
    if let Some(why) = crate::human::mana::pay_from_pool(g, p, 4, "RR") {
        return Ok(Some(format!("Can't activate The Fire Crystal. {why}")));
    }
    g.perm_mut(src).tapped = true; // the cost
    crystal_activate(g, p, src, t)?;
    Ok(None)
}

/// the ability, cost paid and `src` tapped: on the stack with target t (the table may respond: a removed target makes it
/// do nothing, Rings of Brighthearth copies it), then its effect (`crystal_resolve`). A copy of the ability lets its
/// controller pick a new target when that is a person; an AI keeps the target. True if the original resolved.
fn crystal_activate(g: &mut Game, p: PlayerId, src: PermId, t: PermId) -> Res<bool> {
    crate::glog!(g, "  {} activates The Fire Crystal: a token copy of {}", pname(g, p), g.perm(t).name);
    let what = format!("a token copy of {}", g.perm(t).name);
    ability_run_w(g, p, Some(src), None, &what, Some(4.0), Some(t), 0.0, |g, is_copy| {
        let mut t = t;
        if is_copy && human(g, p) {
            // "you may choose a new target for the copy"
            let now = crystal_targets(g, p, src);
            let prompt = "The copy of The Fire Crystal's ability: copy which creature?";
            if let Some(n) = crate::human::cards::pick_perm(g, p, "target", prompt, &now, Some("keep the target"))? {
                t = n;
            }
        }
        crystal_resolve(g, p, src, t)
    })
}

// ------------------------------------------------------------------ the AI's copy
/// PROVISIONAL (never tuned by a sim), like every `FC_` constant. The scale is cards: `karn_gilded::PER_CARD` (1.5) per
/// card that stays in hand, as the blink choosers use.
/// a copy that wins the game (Thassa's Oracle with the library inside its reach)
const FC_WIN: f64 = 50.0;
/// the utility of that copy, in any window
const FC_WIN_U: f64 = 9.5;
/// the end-of-turn window: the mana and the Crystal are free there, so a copy worth anything is made
const FC_EOT_MIN: f64 = 0.5;
/// the utility there is `FC_EOT_BASE` plus the value (at most `FC_EOT_VALUE_CAP`)
const FC_EOT_BASE: f64 = 1.0;
const FC_EOT_VALUE_CAP: f64 = 8.0;
/// per point of power of a copy that is there for p's next combat (times 1.5 flying), at most `FC_BODY_CAP`
const FC_BODY: f64 = 0.15;
const FC_BODY_CAP: f64 = 2.0;
/// a lasting creature token an enters trigger makes
const FC_TOKEN: f64 = 0.8;
/// the share of the removed permanent's `pval` that an enters trigger that removes it is worth
const FC_REMOVAL: f64 = 0.8;
/// what a copy of Thassa's Oracle is worth when it does not win (a look at the top cards)
const FC_ORACLE_LOOK: f64 = 0.3;
/// The Reaper's counter on a creature it does not kill (it feeds the second ability): `reaper_etb` shrinks the two worth most
const FC_REAPER_COUNTER: f64 = 0.4;
/// the hold rule (`crystal_hold`): a best copy worth at least this (three cards) is held mana for
pub const FC_HOLD_MIN: f64 = 4.5;
/// the share of the end-of-turn utility that "stop" is worth when it holds the mana
pub const FC_HOLD_SHARE: f64 = 0.6;
/// the share of the hold's worth that a cast which would break the reserve loses
pub const FC_HOLD_PEN: f64 = 0.7;
/// an opponent this close to winning (`Situation::danger`): the mana is spent on the table, not held
const FC_HOLD_DANGER: f64 = 0.8;
/// a combo piece (`cardcode::combo_imp`) at least this important is cast whatever it leaves
const FC_HOLD_COMBO: f64 = 7.0;
/// a hand this large (cards) is cast from, not held against: the cleanup would discard the rest
const FC_HOLD_HAND: usize = 7;
/// a tutor of at most this mana value is exempt from the reserve
const FC_HOLD_TUTOR_CMC: u32 = 2;

const ORACLE: &str = "Thassa's Oracle";

/// does p control the Crystal `src` ready to use: on the battlefield, p's, untapped, with its abilities on
fn crystal_ready(g: &Game, p: PlayerId, src: PermId) -> bool {
    live(g, src) && owner(g, src) == p && !g.perm(src).tapped && !g.perm(src).neutered
}

/// is p's Crystal `src` locked now: an activated-ability lock (`cardcode::ability_locked`: Pithing Needle or Phyrexian Revoker
/// naming it, Collector Ouphe, Karn, the Great Creator, Abeyance, split second), or, `window`, at the end-of-turn window
/// before p's turn (a Grand Abolisher or Myrel, Shield of Argive of the seat before p stops p's artifacts then: `g.active`
/// is still that seat, so `ability_locked` sees it only in the window itself, and the main phase before it asks ahead)
fn crystal_locked(g: &Game, p: PlayerId, src: PermId, window: bool) -> bool {
    if crate::cardcode::ability_locked(g, src, p) {
        return true;
    }
    window
        && g.after(p).filter(|&q| g.player(q).alive).last().is_some_and(|q| {
            g.player(q).perms.iter().any(|&m| {
                let x = g.perm(m);
                x.on_bf
                    && !x.phased
                    && !x.neutered
                    && matches!(card_name_of(g, m), Some("Grand Abolisher" | "Myrel, Shield of Argive"))
            })
        })
}

/// may p's AI activate its Crystal `src` now, or, `window`, in the end-of-turn window: p is not a person, `AI_CRYSTAL` is on,
/// the Crystal is ready and not locked, and {4}{R}{R} can be paid. The one predicate of `crystal_options` (the activation)
/// and `crystal_hold` (keeping the mana up for it), so the hold is never offered for an activation that would be refused.
fn crystal_usable(g: &Game, p: PlayerId, src: PermId, window: bool) -> bool {
    !human(g, p)
        && crate::ai::table_fixes::crystal_on()
        && crystal_ready(g, p, src)
        && !crystal_locked(g, p, src, window)
        && can_pay(g, p, 4, "RR", false)
}

/// the name of the card m is a copy or the card of (a token with no card record: None)
fn card_name_of(g: &Game, m: PermId) -> Option<&str> {
    g.perm(m).cd.map(|c| &*g.db.get(c).name)
}

/// would a token copy of p's Thassa's Oracle `m` win now: the copy's trigger is not blanked and the library is inside the
/// Oracle's reach (devotion to blue, the copy's {U}{U} counting at resolution; the test of `combos::oracle_prio`)
fn oracle_wins(g: &Game, p: PlayerId, m: PermId) -> bool {
    card_name_of(g, m) == Some(ORACLE)
        && !crate::engine::hooks::enter_silenced(g, p, Some(m))
        && g.player(p).library.len() <= crate::impls::combos::oracle_devotion(g, p) + 2
}

/// what blinking p's creature y is worth to p's AI for a copy of Restoration Angel: `t2::flicker_worth`, except that the real
/// Karn's draw is valued as a copy's is (`karn_gilded::reenter_with_room` with `copy_room`: the cards arrive before p's
/// turn and its cleanup discard still applies), rounded as `t2::blink_value` rounds it. Used only by `copy_value`; the
/// blink choosers keep `reenter_now`.
fn blink_worth_for_copy(g: &Game, p: PlayerId, y: PermId) -> f64 {
    match crate::impls::karn_gilded::reenter_with_room(g, p, y, || copy_room(g, p)) {
        Some(v) => v.round(),
        None => crate::impls::t2::flicker_worth(g, p, y),
    }
}

/// what a token copy of p's creature `m`, made by p's Crystal `src` now, is worth to p's AI (cards x `PER_CARD`).
/// `at_eot`: in the end-of-turn window before p's turn (the copy lives through that turn); false: a main phase.
/// `f64::NEG_INFINITY`: never copy it. A pure read: no RNG, no change.
///
/// 0. A face-down creature: its copy is a vanilla 2/2, so only the body of one (the step below), never the card's name.
/// 1. Never: a card with the `Wipe` tag; p at `TOKEN_CAP` tokens; under a Doomsday pile (after the Oracle check).
/// 2. A copy of Thassa's Oracle that wins: `FC_WIN`.
/// 3. The enters trigger, times its instances (`enters_trigger_times`: 0 under an opposing Elesh Norn or Hushbringer,
///    more with Gandalf or p's own Norn), by card:
///    - Karn, Gilded Guardian: `karn_gilded::copy_draw_value` (the cards, `DECKING` when it would deck p);
///    - Restoration Angel: the best blink worth (`blink_worth_for_copy`: `t2::flicker_worth`, with the real Karn's draw
///      valued with `copy_room` like a copy of Karn) among p's non-Angel creatures, when at least 3.0 (the Angel's own
///      threshold);
///    - Flickerwisp: `t12_brago_meren::wisp_value` with the same `copy_room` (the original is never the target);
///    - Solitude: `FC_REMOVAL` times the `pval` of the best opposing creature p can afford the ward of, when that is 2.0
///      or more (`common::solitude`);
///    - The Reaper, King No More: the `pval` of each of the two creatures it shrinks if the counter kills it, else
///      `FC_REAPER_COUNTER`;
///    - Breya, Etherium Shaper: two Thopters; Infinite Guideline Station: a Robot for each multicoloured permanent p has
///      now (the copy is not counted: it is legendary and the legend rule removes it before the trigger counts); each
///      `FC_TOKEN`;
///    - Thassa's Oracle that does not win: `FC_ORACLE_LOOK`;
///    - a card with a triggered `rem` (Skyclave Apparition): `FC_REMOVAL` times the best `pval` it can take
///      (`removal::legal_targets`, as `etb_removal_pick`), at least 2.0, less the Illusion its owner gets when the copy
///      is sacrificed (`0.5 + 0.25 * mana value`);
///    - any other legendary creature: 0 (the copy dies to the legend rule, and the generic value would score its
///      watcher hooks as enters effects: Yoshimaru, Haliya);
///    - any other card: `ai::card_etb_value` (Bowmasters 2.5, Baleful Strix's draw 1.5, a card-code or compiled enters
///      trigger at least 1.5);
///    - a token with no card record: 0.
/// 4. The body, only at the end-of-turn window and not for a legendary copy (the real card stays): `FC_BODY` per point of
///    power, times 1.5 flying, at most `FC_BODY_CAP`.
pub fn copy_value(g: &Game, p: PlayerId, _src: PermId, m: PermId, at_eot: bool) -> f64 {
    let x = g.perm(m);
    let Some(cd) = x.cd else { return 0.0 };
    let d = g.db.get(cd);
    let facedown = x.data.truthy(DataKey::Facedown);
    if (!facedown && d.tags.has(Tag::Wipe))
        || g.player(p).perms.iter().filter(|&&q| g.perm(q).token).count() >= TOKEN_CAP
    {
        return f64::NEG_INFINITY;
    }
    if facedown {
        // its copy is a vanilla colourless 2/2 with no name or abilities (CR 708.2): a body, whatever the card is
        return if at_eot { (FC_BODY * 2.0).min(FC_BODY_CAP) } else { 0.0 };
    }
    if oracle_wins(g, p, m) {
        return FC_WIN;
    }
    if crate::impls::doomsday::active(g, p) {
        return f64::NEG_INFINITY; // the pile is planned: no copy may draw into it
    }
    let times = crate::engine::hooks::enters_trigger_times(g, p, None, Some(m)) as f64;
    let legendary = crate::cardcode::is_legendary(g, m);
    let etb = match &*d.name {
        crate::impls::karn_gilded::KARN_GILDED => crate::impls::karn_gilded::copy_draw_value(g, p, m),
        "Restoration Angel" => {
            let best = g
                .player(p)
                .perms
                .iter()
                .copied()
                .filter(|&y| {
                    g.is_creature(y)
                        && !g.perm(y).phased
                        && y != m
                        && !crate::engine::values::has_type(g, y, "angel")
                        && !protected_from(g, y, colors_of(g, m))
                })
                .map(|y| blink_worth_for_copy(g, p, y))
                .fold(0.0_f64, f64::max);
            if best >= 3.0 { best * times } else { 0.0 }
        }
        "Flickerwisp" => crate::impls::t12_brago_meren::wisp_value(g, p, m, copy_room(g, p)) * times,
        "Solitude" => {
            let target =
                crate::impls::common::best_opp_creature(g, p, |y| crate::engine::ward::ward_affordable(g, p, y, 0, ""));
            match target.map(|t| pval(g, t)) {
                Some(v) if v >= 2.0 => FC_REMOVAL * v * times,
                _ => 0.0,
            }
        }
        REAPER => {
            let v: f64 = reaper_targets(g, p, m)
                .into_iter()
                .map(|t| if etgh(g, t) <= 1 { pval(g, t) } else { FC_REAPER_COUNTER })
                .sum();
            v * times
        }
        BREYA => 2.0 * FC_TOKEN * times,
        STATION => multicoloured(g, p) as f64 * FC_TOKEN * times,
        ORACLE => FC_ORACLE_LOOK * times,
        _ if d.tags.str(Tag::Rem).is_some_and(|r| !crate::engine::zones::rem_is_static(r)) && d.tags.has(Tag::Etb) => {
            let rem = d.tags.str(Tag::Rem).unwrap_or("destroy");
            let tg = crate::engine::removal::legal_targets(
                g,
                p,
                rem,
                d.tags.str(Tag::Tgt).unwrap_or("c"),
                d.tags.has(Tag::Mv4),
                Some(cd),
            );
            let best = max_by(&tg, |t| pval(g, t));
            match best.filter(|&t| pval(g, t) >= 2.0) {
                Some(t) => {
                    // Skyclave Apparition: its owner gets an X/X Illusion when the copy is sacrificed
                    let illusion = if &*d.name == "Skyclave Apparition" {
                        let cmc = g.perm(t).cd.map_or(0, |c| g.db.get(c).cmc);
                        0.5 + 0.25 * cmc as f64
                    } else {
                        0.0
                    };
                    (FC_REMOVAL * pval(g, t) - illusion) * times
                }
                None => 0.0,
            }
        }
        _ if legendary => 0.0,
        _ => {
            // a card that draws is worth `PER_DISCARD` for the cards beyond the room p's hand has (as a copy of Karn)
            let draws = d.tags.int_or_0(Tag::Draw).max(0) as usize;
            let surplus = copy_room(g, p).map_or(0, |r| draws.saturating_sub(r)) as f64;
            let kept = crate::ai::card_etb_value(g, p, cd) - (PER_CARD - PER_DISCARD) * surplus;
            kept.max(0.0) * times
        }
    };
    let body = if legendary || !at_eot {
        0.0
    } else {
        let fly = if x.fly || crate::dsl::has_kw(g, m, "flying") { 1.5 } else { 1.0 };
        (FC_BODY * epow(g, m).max(0) as f64 * fly).min(FC_BODY_CAP)
    };
    etb + body
}

/// the creatures the Crystal can copy and what each copy is worth now, the best first (ties: the older, `born`). In a main
/// phase only a copy of Thassa's Oracle that wins is looked at: nothing else is worth the mana there (see the module
/// section's doc), and the scoring of the rest is skipped.
fn best_copy(g: &Game, p: PlayerId, src: PermId, at_eot: bool) -> Option<(PermId, f64)> {
    if crystal_locked(g, p, src, at_eot) {
        return None;
    }
    let mut best: Option<(PermId, f64)> = None;
    for m in crystal_targets(g, p, src) {
        if !at_eot && !oracle_wins(g, p, m) {
            continue;
        }
        let v = copy_value(g, p, src, m, at_eot);
        if v == f64::NEG_INFINITY {
            continue;
        }
        let better = match best {
            None => true,
            Some((b, bv)) => v > bv || (v == bv && g.perm(m).born < g.perm(b).born),
        };
        if better {
            best = Some((m, v));
        }
    }
    best
}

/// the AI's use of the Crystal: `{4}{R}{R}, {T}` for the best copy, in the end-of-turn window before p's own turn when it
/// is worth `FC_EOT_MIN`, and in any window when it wins (Thassa's Oracle). Nothing for a person (their ability is
/// `fire_crystal_go`), with `AI_CRYSTAL` off, with the cost not payable, or with nothing worth copying. The activation
/// is `fire_crystal_ai`; its argument is the target and the window (`(target << 1) | at_eot`).
fn crystal_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let at_eot = post.is_none();
    if !human(g, p) && !g.player(p).stats.contains_key("fc_seen") {
        g.player_mut(p).stat("fc_seen", 1); // the Crystal was in play in this game (the report's denominator)
    }
    if !crystal_usable(g, p, src, at_eot) {
        return Ok(vec![]);
    }
    if at_eot {
        // the end-of-turn window asks every option again after each play: count the window once
        let key = intern("fc_eot_ready");
        if g.player(p).flag_turn.get(key) != Some(&g.turn_stamp()) {
            let stamp = g.turn_stamp();
            let pl = g.player_mut(p);
            pl.flag_turn.insert(key, stamp);
            pl.stat("fc_eot_ready", 1);
        }
    }
    let Some((t, v)) = best_copy(g, p, src, at_eot) else { return Ok(vec![]) };
    let Some(u) = copy_utility(v, at_eot) else { return Ok(vec![]) };
    let arg = ((t.0 as i64) << 1) | at_eot as i64;
    Ok(vec![ability(u, &format!("The Fire Crystal: copy {}", g.perm(t).name), src, fire_crystal_ai, arg)])
}

/// the utility of a copy worth v in this window: a win in any window, anything worth `FC_EOT_MIN` in the end-of-turn one
fn copy_utility(v: f64, at_eot: bool) -> Option<f64> {
    if v >= FC_WIN {
        Some(FC_WIN_U)
    } else if at_eot && v >= FC_EOT_MIN {
        Some(FC_EOT_BASE + v.min(FC_EOT_VALUE_CAP))
    } else {
        None
    }
}

/// the AI's activation: as a person's, the target and then the cost and then the tap, with everything checked again and
/// nothing spent on a refusal. `arg` is `(target << 1) | at_eot` (see `crystal_options`).
fn fire_crystal_ai(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (t, at_eot) = (PermId((arg >> 1) as u32), arg & 1 == 1);
    if human(g, p)
        || !crystal_ready(g, p, src)
        || crystal_locked(g, p, src, at_eot)
        || !crystal_targets(g, p, src).contains(&t)
    {
        return Ok(false);
    }
    let v = copy_value(g, p, src, t, at_eot);
    if copy_utility(v, at_eot).is_none() || !pay(g, p, 4, "RR", false)? {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true; // the cost
    let (win, name, stamp) = (v >= FC_WIN, g.perm(t).name, g.turn_stamp());
    let pl = g.player_mut(p);
    pl.flag_turn.insert(intern("fc_acted"), stamp); // the hold's starvation bound reads it (`crystal_hold`)
    pl.stat("fc_acts", 1);
    pl.stat(if win { "fc_acts_win" } else { "fc_acts_eot" }, 1);
    pl.stat(intern(&format!("fc_copy:{name}")), 1);
    if !win {
        pl.stat("fc_value10", (v * 10.0).round() as i64); // a win's value is a sentinel, not a worth in cards
    }
    crystal_activate(g, p, src, t)?;
    Ok(true)
}

// ------------------------------------------------------------------ the hold rule (AI_CRYSTAL_HOLD)
/// what keeping {4}{R}{R} up for the end-of-turn copy is worth now, as a utility for "stop" (`FC_HOLD_SHARE` of the copy's
/// utility), None when there is nothing to hold for: the switch off, no ready Crystal of p's (a person has none), the
/// cost not payable now, an opponent near a win (`danger` at least `FC_HOLD_DANGER`: the mana goes to the table), or the
/// best copy at the end-of-turn window under `FC_HOLD_MIN`, or a win (taken now, in the main phase, by `crystal_options`);
/// also None when the activation would be refused (`crystal_usable`: a lock now or at the window), when p's hand is
/// `FC_HOLD_HAND` cards or more, and for one turn after a hold whose window did not activate (`fc_hold` and `fc_acted`
/// rounds; a game without a hold has neither flag). Reads the game only.
pub fn crystal_hold(g: &Game, p: PlayerId, danger: f64) -> Option<f64> {
    if !crate::ai::table_fixes::crystal_hold_on() || !crate::ai::table_fixes::crystal_on() || danger >= FC_HOLD_DANGER {
        return None;
    }
    if human(g, p) {
        return None;
    }
    let src =
        g.player(p).perms.iter().copied().find(|&m| card_name_of(g, m) == Some(CRYSTAL) && crystal_ready(g, p, m))?;
    if crate::engine::values::hand_proper(g, p).len() >= FC_HOLD_HAND {
        return None; // a full hand is cast from: the cleanup would throw the rest away
    }
    // starvation: p held last turn and the Crystal then did not activate in the window: no second hold in a row
    let round = |k: &str| g.player(p).flag_turn.get(intern(k)).map(|s| s.round);
    if round("fc_hold").is_some_and(|h| h + 1 == g.round) && round("fc_acted") != Some(g.round) {
        return None;
    }
    if !crystal_usable(g, p, src, true) {
        return None;
    }
    let (_, v) = best_copy(g, p, src, true)?;
    if v >= FC_WIN || v < FC_HOLD_MIN {
        return None;
    }
    Some(FC_HOLD_SHARE * (FC_EOT_BASE + v.min(FC_EOT_VALUE_CAP)))
}

/// the hold rule made "stop" worth `crystal_hold` this turn: counted once a turn (`main_options` is asked after each play)
pub fn note_hold(g: &mut Game, p: PlayerId) {
    let key = intern("fc_hold");
    let stamp = g.turn_stamp();
    let pl = g.player_mut(p);
    if pl.flag_turn.get(key) != Some(&stamp) {
        pl.flag_turn.insert(key, stamp);
        pl.stat("fc_hold", 1);
    }
}

/// does casting c now leave p unable to pay {4}{R}{R}? Never for a cast that is exempt: a mana source (a Rock, a Dork,
/// Fastmana, a ritual that pays for itself), a cheap tutor (a `tut` or `seal` card of mana value 2 or less: Vampiric,
/// Demonic, Mystical, Enlightened, Imperial Seal), p's commander, a combo piece (`combo_imp` at least `FC_HOLD_COMBO`).
pub fn crystal_reserve_broken(g: &Game, p: PlayerId, c: CardId) -> bool {
    let d = g.db.get(c);
    let t = &d.tags;
    if t.has(Tag::Rock)
        || t.has(Tag::Dork)
        || t.has(Tag::Fastmana)
        || ((t.has(Tag::Tut) || t.has(Tag::Seal)) && d.cmc <= FC_HOLD_TUTOR_CMC)
        || crate::engine::mana::ritual_net(g, p, c) > 0
        || g.player(p).is_commander(c)
        || crate::cardcode::combo_imp(g, p, c) >= FC_HOLD_COMBO
    {
        return false;
    }
    let (cg, cp) = crate::engine::mana::cost_of(g, p, c);
    !can_pay(g, p, cg + 4, &format!("{cp}RR"), false)
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
    let c = r.card(db, TILLER)?;
    c.etb = Some(tiller_etb);
    c.dies = Some(tiller_dies);
    c.self_dies = Some(tiller_dies);
    r.card(db, HEIRLOOM)?.options = Some(heirloom_options);
    let c = r.card(db, DOLL)?;
    c.on_tap_perm = Some(doll_tap);
    c.options = Some(doll_options);
    r.card(db, CRYSTAL)?.options = Some(crystal_options);
    Ok(())
}
