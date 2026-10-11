//! Python's play/cards.py. Ported by the practice-cards task.
//! The per-card choices of the people's decks (targets, modes, tutors, pay-or-not, ordering), and the helpers the
//! `HUMAN(phase 9)` sites in `impls/*` share.

use super::abilities::{LandAbility, PermAbility, Why, ability};
use super::{Target, choices, legal};
use crate::flow::Res;
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::Game;

/// `hc.choose(g, p, kind, prompt, [legal.describe_target(g, p, m) for m in cands], cancel=...)`: the permanent the
/// person picks from `cands`, None if they cancel (`cancel: None` is a forced choice: the first when no answer fits).
pub fn pick_perm(
    g: &Game,
    p: PlayerId,
    kind: &str,
    prompt: &str,
    cands: &[PermId],
    cancel: Option<&str>,
) -> Res<Option<PermId>> {
    let labels: Vec<String> = cands.iter().map(|&m| legal::describe_target(g, p, Target::Perm(m))).collect();
    Ok(choices::choose(g, p, kind, prompt, &labels, cancel, None)?.map(|k| cands[k]))
}

/// `hc.choose(g, p, kind, prompt, [NAME(q) for q in opps], cancel=None)`: the opponent the person picks.
pub fn pick_player(g: &Game, p: PlayerId, kind: &str, prompt: &str, opps: &[PlayerId]) -> Res<PlayerId> {
    let labels: Vec<String> = opps.iter().map(|&q| g.player(q).name.to_string()).collect();
    Ok(opps[choices::choose_one(g, p, kind, prompt, &labels)?])
}

/// cards._creatures: creatures p's spell or ability can target (opponents' hexproof and shroud respected)
pub fn creatures(g: &Game, p: PlayerId, opp_only: bool) -> Vec<PermId> {
    let mut out = vec![];
    for q in g.players.iter().filter(|q| q.alive && !(opp_only && q.id == p)) {
        for &m in &q.perms {
            if g.is_creature(m)
                && !g.perm(m).phased
                && !(g.perm(m).owner != p && crate::engine::values::untargetable(g, m))
            {
                out.push(m);
            }
        }
    }
    out
}

/// cards.pick_creature: the creature the person picks (None: no target, or they decline an `optional` one)
pub fn pick_creature(
    g: &Game,
    p: PlayerId,
    prompt: &str,
    opp_only: bool,
    optional: bool,
    keep: Option<&dyn Fn(&Game, PlayerId, PermId) -> bool>,
) -> Res<Option<PermId>> {
    let tg: Vec<PermId> = creatures(g, p, opp_only).into_iter().filter(|&m| keep.is_none_or(|f| f(g, p, m))).collect();
    if tg.is_empty() {
        return Ok(None);
    }
    pick_perm(g, p, "target", prompt, &tg, if optional { Some("no target") } else { None })
}

/// cards._blink: a token is exiled for good; else t2.blink
pub fn blink(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    if g.perm(m).token {
        crate::engine::zones::leave(g, m)?;
        crate::glog!(g, "    {} (a token) is exiled for good", g.perm(m).name);
        return Ok(());
    }
    crate::impls::t2::blink(g, p, m)?;
    Ok(())
}

/// cards.ephemerate_rebound: your upkeep: cast Ephemerate from exile for free, on a creature you choose; not cast, it
/// stays in exile
pub fn ephemerate_rebound(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let cre: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
    if cre.is_empty() {
        return Ok(());
    }
    let Some(m) = pick_perm(
        g,
        p,
        "target",
        "Rebound: cast Ephemerate again? Exile and return which creature",
        &cre,
        Some("don't cast it"),
    )?
    else {
        return Ok(());
    };
    let pl = g.player_mut(p);
    if let Some(i) = pl.exile.iter().position(|&x| x == c) {
        pl.exile.remove(i);
    }
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::engine::cast::on_cast(g, p, c)?;
    crate::glog!(g, "  {} casts Ephemerate from exile (rebound) on {}", g.player(p).name, g.perm(m).name);
    if !g.over && crate::engine::stack::counter_window(g, p, c, 3.0, vec![])? && g.player(p).perms.contains(&m) {
        blink(g, p, m)?;
    }
    g.player_mut(p).gy.push(c);
    Ok(())
}

/// cards.pick_exile: 'you may exile n cards from your graveyard': no, the suggested n, or n picked one at a time. The
/// cards, or []
pub fn pick_exile(
    g: &Game,
    p: PlayerId,
    pool: &[CardId],
    n: usize,
    prompt: &str,
    default: &[CardId],
) -> Res<Vec<CardId>> {
    let names: Vec<&str> = default.iter().map(|&c| &*g.db.get(c).name).collect();
    let labels = vec![format!("yes: {}", names.join(", ")), "yes, and I pick the cards".to_string()];
    let Some(k) = choices::choose(g, p, "choose", prompt, &labels, Some("no"), None)? else { return Ok(vec![]) };
    if k == 0 {
        return Ok(default.to_vec());
    }
    let mut picked: Vec<CardId> = vec![];
    while picked.len() < n {
        let left: Vec<CardId> = pool.iter().copied().filter(|c| !picked.contains(c)).collect();
        let labels: Vec<String> = left.iter().map(|&c| g.db.get(c).name.to_string()).collect();
        let prompt = format!("Exile which card? ({} of {n})", picked.len() + 1);
        let j = choices::choose_one(g, p, "choose", &prompt, &labels)?;
        picked.push(left[j]);
    }
    Ok(picked)
}

/// cards.pick_blue: an alternative cost that exiles a blue card from your hand (Force of Will, Force of Negation):
/// which one
pub fn pick_blue(g: &Game, p: PlayerId, blues: &[CardId], what: &str) -> Res<CardId> {
    if blues.len() == 1 {
        return Ok(blues[0]);
    }
    let labels: Vec<String> = blues.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    let k = choices::choose_one(g, p, "choose", &format!("{what}: exile which blue card from your hand?"), &labels)?;
    Ok(blues[k])
}

/// cards._look: the top n cards of the library, taken off it (the top first)
fn look(g: &mut Game, p: PlayerId, n: usize) -> Vec<CardId> {
    let k = n.min(g.player(p).library.len());
    (0..k).map(|_| g.player_mut(p).library.pop().unwrap()).collect()
}

/// cards._bottom: cards to the bottom of the library (index 0)
fn bottom(g: &mut Game, p: PlayerId, cards: Vec<CardId>) {
    g.player_mut(p).library.splice(0..0, cards);
}

/// cards.expressive_iteration: top three: one into your hand, one exiled (you may play it this turn), the last on the
/// bottom
pub fn expressive_iteration(g: &mut Game, p: PlayerId) -> Res {
    let mut top = look(g, p, 3);
    if top.is_empty() {
        return Ok(());
    }
    let keep = choices::pick_cards(g, p, &top, 1, "Expressive Iteration: put which card into your hand?", "choose")?[0];
    remove_one(&mut top, keep);
    g.player_mut(p).hand.push(keep);
    if !top.is_empty() {
        let prompt = "Expressive Iteration: exile which card (you may play it this turn)?";
        let ex = choices::pick_cards(g, p, &top, 1, prompt, "choose")?[0];
        remove_one(&mut top, ex);
        let pl = g.player_mut(p);
        pl.hand.push(ex);
        pl.impulse.push(ex);
    }
    let n = top.len();
    bottom(g, p, top);
    crate::glog!(g, "    {} keeps one card, exiles one to play this turn, puts {} on the bottom", g.player(p).name, n);
    Ok(())
}

/// cards.look_and_take: look at the top n, put k into your hand, the rest on the bottom (Flow State, Stock Up)
pub fn look_and_take(g: &mut Game, p: PlayerId, n: usize, k: usize, name: &str) -> Res {
    let mut top = look(g, p, n);
    let picked = choices::pick_cards(g, p, &top, k, &format!("{name}: put which card into your hand?"), "choose")?;
    for &x in &picked {
        remove_one(&mut top, x);
        g.player_mut(p).hand.push(x);
    }
    let left = top.len();
    bottom(g, p, top);
    crate::glog!(g, "    {} takes {}, puts {} on the bottom", g.player(p).name, picked.len(), left);
    Ok(())
}

/// Python's `list.remove`: the first element equal to x
fn remove_one(v: &mut Vec<CardId>, x: CardId) {
    if let Some(i) = v.iter().position(|&y| y == x) {
        v.remove(i);
    }
}

/// cards._damage: n damage from source to x, a player or permanent chosen already (the same rules as
/// choices.deal_damage)
pub fn damage(g: &mut Game, p: PlayerId, x: Target, n: i32, source: &str, kind: &str) -> Res {
    use crate::engine::{life, removal, zones};
    let m = match x {
        Target::Player(q) => {
            if g.player(q).alive {
                life::lose_life(g, q, n, Some(p), crate::sym::intern(kind), None)?;
            }
            return life::check_state(g);
        }
        Target::Perm(m) => m,
    };
    if !g.perm(m).on_bf {
        return Ok(());
    }
    let name = g.perm(m).name;
    if g.is_creature(m) && crate::engine::values::etgh(g, m) <= n {
        removal::apply_removal(g, Some(p), m, &format!("dmg{n}"), None)?;
    } else if g.is_creature(m) {
        crate::glog!(g, "    {} deals {} damage to {} (it survives)", source, n, name);
    }
    let x = g.perm(m);
    let pw = x.cd.is_some_and(|c| g.db.get(c).types.has(crate::cards::Types::PLANESWALKER));
    if x.on_bf && pw && x.loyalty.is_some() {
        let l = x.loyalty.unwrap() - n;
        g.perm_mut(m).loyalty = Some(l);
        if l <= 0 {
            zones::leave(g, m)?;
            zones::to_zone_card(g, m, zones::Zone::Gy);
        }
    }
    life::check_state(g)
}

/// cards.prismari_charm: choose one: surveil 2, then draw a card; 1 damage to each of one or two targets; return
/// target nonland permanent to its owner's hand
pub fn prismari_charm(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let modes: Vec<String> = [
        "surveil 2, then draw a card",
        "1 damage to each of one or two targets",
        "return target nonland permanent to its owner's hand",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let k = choices::choose_one(g, p, "choose", "Prismari Charm: choose one", &modes)?;
    if k == 0 {
        choices::scry(g, p, 2, true)?;
        crate::engine::zones::draw(g, p, 1, false)?;
        return Ok(());
    }
    if k == 1 {
        let mut tg = choices::damage_targets(g, p, crate::cards::Colors::from_letters("UR"));
        if tg.is_empty() {
            return Ok(());
        }
        let labels: Vec<String> = tg.iter().map(|&x| legal::describe_target(g, p, x)).collect();
        let a = choices::choose_one(g, p, "target", "Prismari Charm: 1 damage to which target?", &labels)?;
        let first = tg.remove(a);
        let b = if tg.is_empty() {
            None
        } else {
            let labels: Vec<String> = tg.iter().map(|&x| legal::describe_target(g, p, x)).collect();
            choices::choose(
                g,
                p,
                "target",
                "Prismari Charm: a second target?",
                &labels,
                Some("no second target"),
                None,
            )?
        };
        let mut hit = vec![first];
        if let Some(b) = b {
            hit.push(tg[b]);
        }
        for x in hit {
            damage(g, p, x, 1, "Prismari Charm", "burn")?;
        }
        return Ok(());
    }
    let tg: Vec<PermId> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| !g.perm(m).phased && !(g.perm(m).owner != p && crate::engine::values::untargetable(g, m)))
        .collect();
    if tg.is_empty() {
        return Ok(());
    }
    let Some(t) = pick_perm(g, p, "target", "Prismari Charm: return which permanent?", &tg, None)? else {
        return Ok(());
    };
    crate::engine::removal::apply_removal(g, Some(p), t, "bounce", Some(c))
}

/// What card c needs before the person can cast it that `legal` doesn't know (Python's `cards.needs`): the reason
/// it can't be cast, or None. PORT(practice-cards): stub, never blocks.
pub fn needs(_g: &Game, _p: PlayerId, _c: CardId) -> Option<String> {
    None
}

/// Does p's Flashback (the card) give graveyard card c flashback (Python's `cards.granted_flashback`).
/// PORT(practice-cards): stub.
pub fn granted_flashback(_g: &Game, _p: PlayerId, _c: CardId) -> bool {
    false
}

// ---- stand-ins for Python's per-card tables (play/cards.py): PORT(practice-cards) replaces each body ----
/// `cards.copy_card(c)`: Return the Favor, Dualcaster Mage (cast to copy a spell on the stack). PORT(practice-cards).
pub fn copy_card(_g: &Game, _c: CardId) -> bool {
    false
}

/// `cards.copy_in_response(g, p, c, spell, caster)`: None or why not. PORT(practice-cards): stub.
pub fn copy_in_response(
    _g: &mut Game,
    _p: PlayerId,
    _c: CardId,
    _spell: Option<CardId>,
    _caster: PlayerId,
) -> Res<Option<String>> {
    Ok(None)
}

/// `c.name in cards.HAND`: a hand card whose casting card code handles. PORT(practice-cards): stub.
pub fn hand_override(_g: &Game, _c: CardId) -> bool {
    false
}

/// `cards.cast_from_hand(g, p, c)`. PORT(practice-cards): stub.
pub fn cast_from_hand(_g: &mut Game, _p: PlayerId, _c: CardId) -> Res<Option<String>> {
    Ok(None)
}

/// `c.name in cards.GY`: a graveyard card whose casting card code handles. PORT(practice-cards): stub.
pub fn gy_override(_g: &Game, _c: CardId) -> bool {
    false
}

/// `cards.GY[c.name](g, p, c)`. PORT(practice-cards): stub.
pub fn cast_from_gy(_g: &mut Game, _p: PlayerId, _c: CardId) -> Res<Option<String>> {
    Ok(None)
}

/// The result of `cards.CAST_TARGET` (a tagless spell with a creature target, picked with `pick_creature`).
pub enum CastTarget {
    /// the card has no such target
    NotNeeded,
    /// the person cancelled
    Cancelled,
    Picked(PermId),
}

/// PORT(practice-cards): stub.
pub fn cast_target(_g: &Game, _p: PlayerId, _c: CardId) -> Res<CastTarget> {
    Ok(CastTarget::NotNeeded)
}

/// `cards.mastery_target(g, p, c)`: Mizzix's Mastery's one target; false = cancelled. PORT(practice-cards): stub
/// (a person cannot cast Mastery for one target until this is ported).
pub fn mastery_target(_g: &mut Game, _p: PlayerId, _c: CardId) -> Res<bool> {
    Ok(false)
}

// ================================================================== abilities by the rules (cards.py ABILITIES, LANDS)
/// a choice with the page's default 'cancel' (Python's `human.choose` default): the index, None if cancelled
fn pick(g: &Game, p: PlayerId, kind: &str, prompt: &str, labels: &[String]) -> Res<Option<usize>> {
    choices::choose(g, p, kind, prompt, labels, Some("cancel"), None)
}

fn target_labels(g: &Game, p: PlayerId, tg: &[Target]) -> Vec<String> {
    tg.iter().map(|&x| legal::describe_target(g, p, x)).collect()
}

fn pname(g: &Game, p: PlayerId) -> String {
    g.player(p).name.to_string()
}

fn cname(g: &Game, c: CardId) -> String {
    g.db.get(c).name.to_string()
}

fn pm_name(g: &Game, m: PermId) -> String {
    match g.perm(m).cd {
        Some(c) => cname(g, c),
        None => g.perm(m).name.to_string(),
    }
}

/// cards._cost: why an ability costing gn + pips can't be activated now
fn cost(g: &Game, p: PlayerId, gn: u32, pips: &str, what: &str) -> Why {
    super::mana::cost_problem(g, p, gn, pips).map(|why| format!("Can't activate {what}. {why}"))
}

/// cards._tapped
fn tapped(g: &Game, m: PermId, what: Option<&str>) -> Why {
    if g.perm(m).tapped {
        Some(format!("{} is tapped.", what.map_or_else(|| pm_name(g, m), str::to_string)))
    } else {
        None
    }
}

fn land_tapped(g: &Game, l: LandId, what: &str) -> Why {
    if g.land(l).tapped { Some(format!("{what} is tapped.")) } else { None }
}

// ------------------------------------------------------------------ planeswalkers
/// cards._loyalty: why a loyalty ability of m costing `cost` can't be activated now (sorcery speed, once per turn,
/// enough loyalty)
fn loyalty(g: &Game, p: PlayerId, m: PermId, cost: i32) -> Why {
    if let Some(why) = legal::sorcery_timing(g, p) {
        return Some(why.replace("do that", "activate a loyalty ability"));
    }
    if !crate::impls::mine::pw_once(g, m) {
        return Some(format!("{} has already used a loyalty ability this turn.", pm_name(g, m)));
    }
    let l = g.perm(m).loyalty.unwrap_or(0);
    if cost < 0 && l < -cost {
        return Some(format!("{} has only {} loyalty.", pm_name(g, m), l));
    }
    None
}

/// what a walker's effect says before the loyalty is paid (Python: a str, False, a callable, or None)
pub enum Ready {
    /// nothing more to do after paying
    Now,
    /// the reason it can't be done
    Why(String),
    /// the person cancelled a choice: costs nothing
    Cancel,
    /// carry this out after the loyalty is paid and the ability resolves
    Go(Box<dyn FnOnce(&mut Game) -> Res>),
}

fn go(f: impl FnOnce(&mut Game) -> Res + 'static) -> Res<Ready> {
    Ok(Ready::Go(Box::new(f)))
}

type WalkEff = fn(&mut Game, PlayerId, PermId) -> Res<Ready>;

/// cards._walker: abilities: [(cost, text, effect)]: the effect runs after the loyalty is paid; one that needs a choice
/// asks before paying (a cancelled choice costs nothing) by returning a `Ready::Go`
fn walker(list: &[(i32, &'static str, WalkEff)]) -> Vec<PermAbility> {
    list.iter()
        .map(|&(cost, text, eff)| {
            ability(format!("{cost:+}: {text}"), move |g: &mut Game, p: PlayerId, m: PermId| {
                if let Some(why) = loyalty(g, p, m, cost) {
                    return Ok(Some(why));
                }
                let ready = match eff(g, p, m)? {
                    Ready::Why(why) => return Ok(Some(why)),
                    Ready::Cancel => return Ok(None),
                    r => r,
                };
                crate::glog!(g, "  {} activates {} {:+}: {}", pname(g, p), pm_name(g, m), cost, text);
                crate::impls::mine::pw_use(g, m, cost)?; // a walker at 0 loyalty is gone; the effect still happens
                let name = format!("{cost:+}");
                let on_table = g.player(p).perms.contains(&m);
                let resolved = match (on_table, g.perm(m).cd) {
                    (false, Some(cd)) => crate::engine::stack::ability_window_card(g, p, cd, &name, None, None)?,
                    _ => crate::engine::stack::ability_window(g, p, Some(m), &name, None, None)?,
                };
                if !resolved {
                    return Ok(None);
                }
                if let Ready::Go(f) = ready {
                    f(g)?;
                }
                crate::engine::life::check_state(g)?;
                Ok(None)
            })
        })
        .map(|mut a| {
            if a.label.starts_with("+0") || a.label.starts_with("-0") {
                a.label = a.label.replacen(&a.label[..2], "0", 1);
            }
            a
        })
        .collect()
}

/// cards._any_player: any living player (you included), None if cancelled
fn any_player(g: &Game, p: PlayerId, prompt: &str) -> Res<Option<PlayerId>> {
    let ps: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    let labels: Vec<String> =
        ps.iter().map(|&q| format!("{}{}", pname(g, q), if q == p { " (you)" } else { "" })).collect();
    Ok(pick(g, p, "target", prompt, &labels)?.map(|k| ps[k]))
}

fn vraska_minus2(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    let cre = creatures(g, p, false);
    if cre.is_empty() {
        return Ok(Ready::Why("There is no creature to target.".into()));
    }
    let Some(t) = pick_perm(g, p, "target", "Vraska -2: which creature becomes a Treasure?", &cre, Some("cancel"))?
    else {
        return Ok(Ready::Cancel);
    };
    go(move |g| {
        if g.perm(t).on_bf && g.player(g.perm(t).owner).perms.contains(&t) {
            let q = g.perm(t).owner;
            let name = g.perm(t).name;
            crate::engine::zones::leave(g, t)?;
            g.player_mut(q).treasures += 1;
            crate::glog!(g, "    {} becomes a Treasure", name);
        }
        Ok(())
    })
}

fn vraska_minus9(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    let Some(q) = any_player(g, p, "Vraska -9: which player gets nine poison counters?")? else {
        return Ok(Ready::Cancel);
    };
    go(move |g| {
        if !crate::engine::values::melira(g, q) {
            let pl = g.player_mut(q);
            pl.poison = pl.poison.max(9);
        }
        Ok(())
    })
}

fn vraska_zero(_g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    go(move |g| {
        crate::engine::zones::draw(g, p, 1, false)?;
        crate::engine::life::lose_life(g, p, 1, Some(p), "other", None)?;
        crate::cardcode::proliferate_all(g, p)
    })
}

fn ralz_plus1(_g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    go(move |g| choices::scry(g, p, 2, true))
}

fn ralz_minus1(_g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    go(move |g| {
        for q in g.opps(p).collect::<Vec<_>>() {
            if !g.player(q).hand.is_empty() {
                crate::engine::cast::discard_worst(g, q, 1)?;
            }
        }
        Ok(())
    })
}

fn ralz_minus2(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    let cs: Vec<CardId> =
        g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature && g.db.get(c).cmc <= 3).collect();
    if cs.is_empty() {
        return Ok(Ready::Why("There is no creature card with mana value 3 or less in your graveyard.".into()));
    }
    let labels: Vec<String> = cs.iter().map(|&c| choices::card_label(g, c)).collect();
    let Some(k) = pick(g, p, "target", "Ral Zarek -2: return which creature card?", &labels)? else {
        return Ok(Ready::Cancel);
    };
    let c = cs[k];
    go(move |g| {
        if let Some(i) = g.player(p).gy.iter().position(|&x| x == c) {
            g.player_mut(p).gy.remove(i);
            crate::engine::zones::enter(g, p, c, Default::default())?;
        }
        Ok(())
    })
}

fn ralz_minus7(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    if opps.is_empty() {
        return Ok(Ready::Why("There is no opponent to target.".into()));
    }
    let labels: Vec<String> = opps.iter().map(|&q| pname(g, q)).collect();
    let Some(k) = pick(g, p, "target", "Ral Zarek -7: which opponent?", &labels)? else { return Ok(Ready::Cancel) };
    let q = opps[k];
    go(move |g| {
        let n = (0..5).filter(|_| g.rng.random() < 0.5).count() as u32;
        g.player_mut(q).skip_turns += n;
        crate::glog!(g, "    five coins: {} heads, {} skips {} turn(s)", n, pname(g, q), n);
        Ok(())
    })
}

fn ral_plus2(_g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    go(move |g| choices::scry(g, p, 1, false))
}

fn ral_minus2(_g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    go(move |g| {
        g.player_mut(p).ral_copy = Some(g.turn_stamp());
        Ok(())
    })
}

// ------------------------------------------------------------------ prepared back faces (cast a copy of the spell)
/// cards.PREPARED: card name -> (generic, pips, instant, spell, text). (Python's branch for a 'Reanimate' copy is
/// unreachable: no card here has it.)
const PREPARED: [(&str, u32, &str, bool, &str, &str); 4] = [
    ("Blazing Firesinger // Seething Song", 2, "R", true, "Seething Song", "add {R}{R}{R}{R}{R}"),
    ("Emeritus of Ideation // Ancestral Recall", 0, "U", true, "Ancestral Recall", "target player draws three"),
    ("Sanar, Unfinished Genius // Wild Idea", 3, "UR", false, "Wild Idea", "search for an instant or sorcery"),
    ("Emeritus of Conflict // Lightning Bolt", 0, "R", true, "Lightning Bolt", "3 damage to any target"),
];

/// cards._prepared
fn prepared(g: &Game, m: PermId) -> Vec<PermAbility> {
    let nm = pm_name(g, m);
    let Some(&(_, gn, pips, instant, spell, text)) = PREPARED.iter().find(|x| x.0 == nm) else { return vec![] };
    if !g.perm(m).data.truthy(crate::state::DataKey::Prepared) {
        return vec![];
    }
    let label = format!("prepared: cast a copy of {spell} ({}): {text}", super::mana::cost_text(gn, pips));
    vec![ability(label, move |g: &mut Game, p: PlayerId, m: PermId| {
        use crate::state::{DataKey, Val};
        if !g.perm(m).data.truthy(DataKey::Prepared) {
            let nm = pm_name(g, m);
            return Ok(Some(format!("{} is not prepared.", nm.split(" //").next().unwrap_or(&nm))));
        }
        if !instant && !crate::engine::values::has(g, p, crate::tag::Tag::Gandalf) {
            if let Some(why) = legal::sorcery_timing(g, p) {
                return Ok(Some(why.replace("do that", &format!("cast {spell} (a sorcery)"))));
            }
        }
        if let Some(why) = super::mana::cost_problem(g, p, gn, pips) {
            return Ok(Some(format!("Can't cast {spell}. {why}")));
        }
        // the copy's effect; the number is prepared_effect's, for Thousand-Year Storm's copies of the cast
        let mut num: Option<u8> = None;
        let mut eff: Box<dyn FnMut(&mut Game) -> Res> = match spell {
            "Ancestral Recall" => {
                let Some(q) = any_player(g, p, "Ancestral Recall: which player draws three?")? else {
                    return Ok(None);
                };
                if q == p {
                    num = Some(1);
                }
                Box::new(move |g| crate::engine::zones::draw(g, q, 3, false))
            }
            "Lightning Bolt" => {
                let tg = choices::damage_targets(g, p, crate::cards::Colors::from_letters("R"));
                if tg.is_empty() {
                    return Ok(Some("There is no target for Lightning Bolt.".into()));
                }
                let labels = target_labels(g, p, &tg);
                let Some(k) = pick(g, p, "target", "Lightning Bolt: 3 damage to which target?", &labels)? else {
                    return Ok(None);
                };
                let x = tg[k];
                Box::new(move |g| damage(g, p, x, 3, "Lightning Bolt", "burn"))
            }
            "Seething Song" => {
                num = Some(0);
                Box::new(move |g| {
                    g.player_mut(p).pool.add('R', 5);
                    Ok(())
                })
            }
            _ => {
                num = Some(2);
                Box::new(move |g| crate::engine::tutors::tutor(g, p, "is"))
            }
        };
        super::mana::pay_from_pool(g, p, gn, pips);
        g.perm_mut(m).data.set(DataKey::Prepared, Val::Bool(false));
        crate::impls::mine::cast_copy_with(g, p, num, Some(spell), instant, 4.0, &mut *eff)?;
        Ok(None)
    })]
}

// ------------------------------------------------------------------ other permanents
fn mind_stone(_g: &Game) -> Vec<PermAbility> {
    vec![ability("{1}, {T}, sacrifice: draw a card", |g: &mut Game, p: PlayerId, m: PermId| {
        if let Some(why) = tapped(g, m, None).or_else(|| cost(g, p, 1, "", "Mind Stone")) {
            return Ok(Some(why));
        }
        super::mana::pay_from_pool(g, p, 1, "");
        g.perm_mut(m).tapped = true;
        crate::glog!(g, "  {} sacrifices Mind Stone: draw a card", pname(g, p));
        let cd = g.perm(m).cd;
        crate::engine::zones::die(g, m, "sac")?;
        if let Some(cd) = cd
            && crate::engine::stack::ability_window_card(g, p, cd, "draw a card", None, None)?
        {
            crate::engine::zones::draw(g, p, 1, false)?;
        }
        Ok(None)
    })]
}

fn triskelion(_g: &Game) -> Vec<PermAbility> {
    vec![ability("remove a +1/+1 counter: 1 damage to any target", |g: &mut Game, p: PlayerId, m: PermId| {
        if g.perm(m).plus <= 0 {
            return Ok(Some("Triskelion has no +1/+1 counter to remove.".into()));
        }
        let tg = choices::damage_targets(g, p, Default::default());
        let labels = target_labels(g, p, &tg);
        let Some(k) = pick(g, p, "target", "Triskelion: 1 damage to which target?", &labels)? else {
            return Ok(None);
        };
        g.perm_mut(m).plus -= 1;
        crate::glog!(g, "  {} removes a counter from Triskelion", pname(g, p));
        let tp = if let Target::Perm(t) = tg[k] { Some(t) } else { None };
        if crate::engine::stack::ability_window(g, p, Some(m), "1 damage", None, tp)? {
            damage(g, p, tg[k], 1, "Triskelion", "triggers")?;
        }
        if g.player(p).perms.contains(&m) && crate::engine::values::etgh(g, m) <= 0 {
            crate::engine::zones::die(g, m, "sba")?;
        }
        Ok(None)
    })]
}

fn aetherflux(_g: &Game) -> Vec<PermAbility> {
    vec![ability("pay 50 life: 50 damage to any target", |g: &mut Game, p: PlayerId, m: PermId| {
        if g.player(p).life < 50 {
            return Ok(Some(format!("You need 50 life to pay (you have {}).", g.player(p).life)));
        }
        let tg = choices::damage_targets(g, p, Default::default());
        let labels = target_labels(g, p, &tg);
        let Some(k) = pick(g, p, "target", "Aetherflux Reservoir: 50 damage to which target?", &labels)? else {
            return Ok(None);
        };
        crate::engine::life::lose_life(g, p, 50, Some(p), "other", None)?;
        crate::glog!(g, "  {} pays 50 life: Aetherflux Reservoir", pname(g, p));
        let tp = if let Target::Perm(t) = tg[k] { Some(t) } else { None };
        if !crate::engine::stack::ability_window(g, p, Some(m), "50 damage", Some(9.0), tp)? {
            return Ok(None);
        }
        let pl = g.player_mut(p);
        pl.stat(crate::sym::intern("aether_shots"), 1);
        let turns = pl.turns;
        pl.milestone.entry(crate::sym::intern("aether")).or_insert(turns);
        damage(g, p, tg[k], 50, "Aetherflux Reservoir", "aether")?;
        Ok(None)
    })]
}

fn ste(_g: &Game) -> Vec<PermAbility> {
    vec![ability("sacrifice: a basic land onto the battlefield tapped", |g: &mut Game, p: PlayerId, m: PermId| {
        crate::glog!(g, "  {} sacrifices Sakura-Tribe Elder", pname(g, p));
        let cd = g.perm(m).cd;
        crate::engine::zones::die(g, m, "sac")?;
        if let Some(cd) = cd
            && crate::engine::stack::ability_window_card(g, p, cd, "search for a basic land", None, None)?
        {
            crate::engine::zones::land_ramp(g, p, 1, true)?;
        }
        Ok(None)
    })]
}

type OutletEffect = fn(&mut Game, PlayerId, PermId, i32) -> Res;

/// cards._outlet: a free sacrifice outlet: "Sacrifice a creature: <text>". effect(g, p, m, power) after the sacrifice
fn outlet(text: &'static str, effect: OutletEffect) -> Vec<PermAbility> {
    vec![ability(format!("sacrifice a creature: {text}"), move |g: &mut Game, p: PlayerId, m: PermId| {
        let cre: Vec<PermId> =
            g.player(p).perms.iter().copied().filter(|&x| g.is_creature(x) && !g.perm(x).phased).collect();
        if cre.is_empty() {
            return Ok(Some("You have no creature to sacrifice.".into()));
        }
        let outlet_name = pm_name(g, m);
        let Some(x) =
            pick_perm(g, p, "choose", &format!("{outlet_name}: sacrifice which creature?"), &cre, Some("cancel"))?
        else {
            return Ok(None);
        };
        let pw = crate::engine::values::epow(g, x);
        crate::glog!(g, "  {} sacrifices {} to {}", pname(g, p), g.perm(x).name, outlet_name);
        crate::engine::zones::die(g, x, "sac")?;
        if crate::engine::stack::ability_window(g, p, Some(m), text, None, None)? {
            effect(g, p, m, pw)?;
        }
        crate::engine::life::check_state(g)?;
        Ok(None)
    })]
}

fn feeder(g: &mut Game, p: PlayerId, m: PermId, _pw: i32) -> Res {
    if g.player(p).perms.contains(&m) {
        g.perm_mut(m).plus += 1;
    }
    Ok(())
}

fn dementia(g: &mut Game, p: PlayerId, _m: PermId, pw: i32) -> Res {
    let q = if pw > 0 { any_player(g, p, &format!("Altar of Dementia: which player mills {pw}?"))? } else { None };
    if let Some(q) = q {
        crate::engine::zones::mill(g, q, pw as usize)?;
        crate::glog!(g, "    Altar of Dementia: {} mills {}", pname(g, q), pw);
    }
    Ok(())
}

fn coalition(_g: &Game) -> Vec<PermAbility> {
    vec![ability(
        "{T}: put a charge counter on it (each one is a mana of any colour at your next precombat main phase)",
        |g: &mut Game, p: PlayerId, m: PermId| {
            use crate::state::{DataKey, Val};
            if let Some(why) = tapped(g, m, None) {
                return Ok(Some(why));
            }
            g.perm_mut(m).tapped = true;
            crate::glog!(g, "  {} puts a charge counter on Coalition Relic", pname(g, p));
            if crate::engine::stack::ability_window(g, p, Some(m), "a charge counter", None, None)?
                && g.player(p).perms.contains(&m)
            {
                let n = g.perm(m).data.int(DataKey::Charge);
                g.perm_mut(m).data.set(DataKey::Charge, Val::Int(n + 1));
            }
            Ok(None)
        },
    )]
}

/// cards._citadel: Bolas's Citadel: play the top card of your library (a spell for life equal to its mana value
/// instead of its mana cost); {T}, sacrifice ten nonland permanents: each opponent loses 10 life
fn citadel(g: &Game, p: PlayerId) -> Vec<PermAbility> {
    use crate::tag::Tag;
    let mut out = vec![];
    let top = g.player(p).library.last().copied();
    if let Some(top) = top {
        let d = g.db.get(top);
        let what = if d.land { format!("play {}", d.name) } else { format!("cast {} for {} life", d.name, d.cmc) };
        out.push(ability(
            format!("{what} from the top of your library"),
            move |g: &mut Game, p: PlayerId, _m: PermId| {
                if g.player(p).library.last() != Some(&top) {
                    return Ok(Some("The top card of your library has changed.".into()));
                }
                let d = g.db.get(top);
                let nm = d.name.to_string();
                if d.land {
                    if let Some(why) = legal::sorcery_timing(g, p) {
                        return Ok(Some(why.replace("do that", "play a land")));
                    }
                    if g.player(p).lands_played as i32 >= legal::land_drops(g, p) {
                        return Ok(Some("You've already played a land this turn.".into()));
                    }
                    g.player_mut(p).library.pop();
                    crate::engine::turn::play_land_card(g, p, top, "plays (from the top, Citadel)")?;
                    crate::engine::life::check_state(g)?;
                    return Ok(None);
                }
                if !legal::instant_speed(g, top) {
                    if let Some(why) = legal::sorcery_timing(g, p) {
                        return Ok(Some(why.replace("do that", &format!("cast {nm}"))));
                    }
                }
                if d.tag(Tag::Ctr) {
                    return Ok(Some(format!("{nm} counters a spell: it can only be cast in response to one.")));
                }
                if [Tag::X, Tag::Tokx, Tag::Xtutor, Tag::Xdrain, Tag::Crackle, Tag::Rean, Tag::Deluge]
                    .iter()
                    .any(|&t| d.tag(t))
                {
                    return Ok(Some(format!(
                        "{nm} has a choice of X or a graveyard target the Citadel path doesn't support; draw it instead."
                    )));
                }
                if g.player(p).life <= d.cmc as i32 {
                    return Ok(Some(format!("Casting {nm} costs {} life; you have {}.", d.cmc, g.player(p).life)));
                }
                if !crate::engine::cast::castable(g, p, top, "lib") {
                    return Ok(Some(format!("Something on the battlefield stops you casting {nm} right now.")));
                }
                super::human::cast(g, p, top, "citadel")
            },
        ));
    }
    out.push(ability(
        "{T}, sacrifice ten nonland permanents: each opponent loses 10 life",
        |g: &mut Game, p: PlayerId, m: PermId| {
            if let Some(why) = tapped(g, m, Some("Bolas's Citadel")) {
                return Ok(Some(why));
            }
            let fod: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&x| !g.perm(x).phased && x != m).collect();
            if fod.len() + 1 < 10 {
                return Ok(Some(format!("You need ten nonland permanents to sacrifice; you have {}.", fod.len() + 1)));
            }
            let mut picked = vec![m];
            while picked.len() < 10 {
                let left: Vec<PermId> = fod.iter().copied().filter(|x| !picked.contains(x)).collect();
                let labels = target_labels(g, p, &left.iter().map(|&x| Target::Perm(x)).collect::<Vec<_>>());
                let k = choices::choose_one(
                    g,
                    p,
                    "choose",
                    &format!("Bolas's Citadel: sacrifice which permanent? ({} of 10)", picked.len() + 1),
                    &labels,
                )?;
                picked.push(left[k]);
            }
            g.perm_mut(m).tapped = true;
            for x in picked {
                crate::engine::zones::die(g, x, "sac")?;
            }
            for q in g.opps(p).collect::<Vec<_>>() {
                crate::engine::life::lose_life(g, q, 10, Some(p), "other", None)?;
            }
            crate::glog!(g, "  {} sacrifices ten permanents to Bolas's Citadel: each opponent loses 10", pname(g, p));
            crate::engine::life::check_state(g)?;
            Ok(None)
        },
    ));
    out
}

/// cards.abilities: the abilities of your permanent m, or None when its card isn't listed here (its card-code
/// abilities apply)
pub fn abilities(g: &Game, p: PlayerId, m: PermId) -> Option<Vec<PermAbility>> {
    let cd = g.perm(m).cd?;
    let nm = cname(g, cd);
    Some(match nm.as_str() {
        "Bolas's Citadel" => citadel(g, p),
        "Carrion Feeder" => outlet("put a +1/+1 counter on Carrion Feeder", feeder),
        "Viscera Seer" => outlet("scry 1", |g, p, _m, _pw| choices::scry(g, p, 1, false)),
        "Ashnod's Altar" => outlet("add {C}{C}", |g, p, _m, _pw| {
            g.player_mut(p).pool.add('C', 2);
            Ok(())
        }),
        "Altar of Dementia" => outlet("target player mills cards equal to the sacrificed creature's power", dementia),
        "Coalition Relic" => coalition(g),
        "Vraska, Betrayal's Sting" => walker(&[
            (0, "draw a card, lose 1 life, proliferate", vraska_zero),
            (-2, "target creature becomes a Treasure", vraska_minus2),
            (-9, "a player's poison counters become nine", vraska_minus9),
        ]),
        "Ral Zarek, Guest Lecturer" => walker(&[
            (1, "surveil 2", ralz_plus1),
            (-1, "each opponent discards a card", ralz_minus1),
            (-2, "a creature card (mana value 3 or less) from your graveyard to the battlefield", ralz_minus2),
            (-7, "flip five coins: target opponent skips a turn per head", ralz_minus7),
        ]),
        "Ral, Storm Conduit" => {
            walker(&[(2, "scry 1", ral_plus2), (-2, "copy the next instant or sorcery you cast this turn", ral_minus2)])
        }
        "Mind Stone" => mind_stone(g),
        "Triskelion" => triskelion(g),
        "Aetherflux Reservoir" => aetherflux(g),
        "Sakura-Tribe Elder" => ste(g),
        "Nim Deathmantle" => vec![], // equip {4}: the Equip ability every equipment has
        n if PREPARED.iter().any(|x| x.0 == n) => prepared(g, m),
        _ => return None,
    })
}

// ------------------------------------------------------------------ lands
fn strip(_g: &Game) -> Vec<LandAbility> {
    vec![ability("{T}, sacrifice: destroy target land", |g: &mut Game, p: PlayerId, l: LandId| {
        if let Some(why) = land_tapped_if(g, l, "Strip Mine") {
            return Ok(Some(why));
        }
        let tg: Vec<(PlayerId, LandId)> =
            g.players.iter().filter(|q| q.alive).flat_map(|q| q.lands.iter().map(move |&x| (q.id, x))).collect();
        let labels: Vec<String> = tg
            .iter()
            .map(|&(q, x)| {
                format!("{} ({})", cname(g, g.land(x).cd), if q == p { "yours".to_string() } else { pname(g, q) })
            })
            .collect();
        let Some(k) = pick(g, p, "target", "Strip Mine: destroy which land?", &labels)? else { return Ok(None) };
        let (q, x) = tg[k];
        crate::engine::turn::remove_land(g, p, l);
        let cd = g.land(l).cd;
        g.player_mut(p).gy.push(cd);
        crate::glog!(
            g,
            "  {} sacrifices Strip Mine: destroy {} ({})",
            pname(g, p),
            cname(g, g.land(x).cd),
            pname(g, q)
        );
        let what = format!("destroy {}", cname(g, g.land(x).cd));
        if crate::engine::stack::ability_window_card(g, p, cd, &what, None, None)? && g.player(q).lands.contains(&x) {
            crate::impls::fixes::destroy_land(g, q, x)?;
        }
        Ok(None)
    })]
}

fn land_tapped_if(g: &Game, l: LandId, what: &str) -> Why {
    land_tapped(g, l, what)
}

fn lighthouse(_g: &Game) -> Vec<LandAbility> {
    vec![ability("{1}{U}{R}, {T}: draw a card, then discard a card", |g: &mut Game, p: PlayerId, l: LandId| {
        if let Some(why) =
            land_tapped(g, l, "Desolate Lighthouse").or_else(|| cost(g, p, 1, "UR", "Desolate Lighthouse"))
        {
            return Ok(Some(why));
        }
        super::mana::pay_from_pool(g, p, 1, "UR");
        g.land_mut(l).tapped = true;
        crate::glog!(g, "  {} loots with Desolate Lighthouse", pname(g, p));
        let cd = g.land(l).cd;
        if crate::engine::stack::ability_window_card(g, p, cd, "draw, then discard", None, None)? {
            crate::engine::zones::draw(g, p, 1, false)?;
            crate::engine::cast::discard_worst(g, p, 1)?;
        }
        Ok(None)
    })]
}

fn summit(_g: &Game) -> Vec<LandAbility> {
    vec![ability("{2}{U}{R}, {T}: surveil 1", |g: &mut Game, p: PlayerId, l: LandId| {
        if let Some(why) = land_tapped(g, l, "Spectacle Summit").or_else(|| cost(g, p, 2, "UR", "Spectacle Summit")) {
            return Ok(Some(why));
        }
        super::mana::pay_from_pool(g, p, 2, "UR");
        g.land_mut(l).tapped = true;
        let cd = g.land(l).cd;
        if crate::engine::stack::ability_window_card(g, p, cd, "surveil 1", None, None)? {
            choices::scry(g, p, 1, true)?;
        }
        Ok(None)
    })]
}

fn baraddur(_g: &Game) -> Vec<LandAbility> {
    vec![ability(
        "{X}{X}{B}, {T}: amass Orcs X (only if a creature died this turn)",
        |g: &mut Game, p: PlayerId, l: LandId| {
            if let Some(why) = land_tapped(g, l, "Barad-dûr") {
                return Ok(Some(why));
            }
            if g.died_turn != Some(g.turn_stamp()) {
                return Ok(Some("Barad-dûr: activate only if a creature died this turn.".into()));
            }
            let top = (g.player(p).pool.total() as i32 - 1).div_euclid(2).max(0);
            if let Some(why) = super::mana::cost_problem(g, p, 0, "B") {
                return Ok(Some(format!("Can't activate Barad-dûr. {why}")));
            }
            let mut labels: Vec<String> =
                (1..=top).map(|x| format!("X = {x} ({})", super::mana::cost_text(2 * x as u32, "B"))).collect();
            if labels.is_empty() {
                labels.push("X = 0 ({B})".into());
            }
            let Some(k) = pick(g, p, "choose", "Barad-dûr: choose X (it costs {X}{X}{B}: amass Orcs X)", &labels)?
            else {
                return Ok(None);
            };
            let x = if top > 0 { k as i32 + 1 } else { 0 };
            if let Some(why) = super::mana::pay_from_pool(g, p, 2 * x as u32, "B") {
                return Ok(Some(format!("Can't activate Barad-dûr. {why}")));
            }
            g.land_mut(l).tapped = true;
            crate::glog!(g, "  {} uses Barad-dûr: amass Orcs {}", pname(g, p), x);
            let cd = g.land(l).cd;
            if crate::engine::stack::ability_window_card(g, p, cd, &format!("amass Orcs {x}"), None, None)? {
                crate::engine::zones::amass(g, p, x)?;
            }
            Ok(None)
        },
    )]
}

fn mistrise(_g: &Game) -> Vec<LandAbility> {
    vec![ability(
        "{U}, {T}: the next spell you cast this turn can't be countered",
        |g: &mut Game, p: PlayerId, l: LandId| {
            if let Some(why) = land_tapped(g, l, "Mistrise Village").or_else(|| cost(g, p, 0, "U", "Mistrise Village"))
            {
                return Ok(Some(why));
            }
            super::mana::pay_from_pool(g, p, 0, "U");
            g.land_mut(l).tapped = true;
            g.player_mut(p).mistrise_next = Some(g.turn_stamp());
            crate::glog!(
                g,
                "  {} activates Mistrise Village: the next spell this turn can't be countered",
                pname(g, p)
            );
            Ok(None)
        },
    )]
}

/// cards.land_abilities
pub fn land_abilities(g: &Game, _p: PlayerId, l: LandId) -> Vec<LandAbility> {
    match &*cname(g, g.land(l).cd) {
        "Strip Mine" => strip(g),
        "Desolate Lighthouse" => lighthouse(g),
        "Spectacle Summit" => summit(g),
        "Barad-dûr" => baraddur(g),
        "Mistrise Village" => mistrise(g),
        _ => vec![],
    }
}
