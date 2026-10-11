//! Python's play/cards.py. Ported by the practice-cards task.
//! The per-card choices of the people's decks (targets, modes, tutors, pay-or-not, ordering), and the helpers the
//! person's branches in `impls/*` share.

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
        n => return more_abilities(g, p, m, n),
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

// ================================================================== casting from hand: modes the card code keeps for the AI
/// cards._spell_cast: the steps of casting c from hand once it is paid for: on the stack, cast triggers, a window for
/// opponents. True if it resolves (the caller carries out its effect and puts the card away)
fn spell_cast(g: &mut Game, p: PlayerId, c: CardId, imp: f64, what: Option<&str>) -> Res<bool> {
    if let Some(i) = g.player(p).hand.iter().position(|&x| x == c) {
        g.player_mut(p).hand.remove(i);
    }
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat(crate::sym::intern("spells_cast"), 1);
    pl.cast_names.insert(c);
    let shown = what.map_or_else(|| cname(g, c), str::to_string);
    crate::glog!(g, "  {} casts {}", pname(g, p), shown);
    crate::engine::cast::on_cast(g, p, c)?;
    if g.over || !g.player(p).alive {
        return Ok(false);
    }
    crate::engine::stack::counter_window(g, p, c, imp, vec![])
}

/// cards._cast_normally
fn cast_normally(g: &mut Game, p: PlayerId, c: CardId) -> Res<Why> {
    if let Some(why) = legal::check_cast(g, p, c, "hand") {
        return Ok(Some(why));
    }
    super::human::cast(g, p, c, "hand")
}

/// cards._cycle(2)
fn cycle2(g: &mut Game, p: PlayerId, c: CardId) -> Res<Why> {
    if let Some(why) = super::mana::cost_problem(g, p, 2, "") {
        return Ok(Some(format!("Can't cycle {}. {why}", cname(g, c))));
    }
    super::mana::pay_from_pool(g, p, 2, "");
    let pl = g.player_mut(p);
    if let Some(i) = pl.hand.iter().position(|&x| x == c) {
        pl.hand.remove(i);
    }
    pl.gy.push(c);
    crate::glog!(g, "  {} cycles {}", pname(g, p), cname(g, c));
    crate::engine::zones::draw(g, p, 1, false)?;
    Ok(None)
}

fn gy_push(g: &mut Game, p: PlayerId, c: CardId) {
    g.player_mut(p).gy.push(c);
}

fn my_creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect()
}

fn names_of(g: &Game, ms: &[PermId]) -> String {
    ms.iter().map(|&m| g.perm(m).name.to_string()).collect::<Vec<_>>().join(", ")
}

/// cards._twinflame
fn twinflame(g: &mut Game, p: PlayerId, c: CardId) -> Res<Why> {
    use crate::engine::zones;
    if let Some(why) = legal::check_cast(g, p, c, "hand") {
        return Ok(Some(why));
    }
    let cre = my_creatures(g, p);
    if cre.is_empty() {
        return Ok(Some("Twinflame needs a creature you control to target.".into()));
    }
    let mut picked: Vec<PermId> = vec![];
    loop {
        let left: Vec<PermId> = cre.iter().copied().filter(|m| !picked.contains(m)).collect();
        if left.is_empty() {
            break;
        }
        let n = picked.len();
        let prompt = format!(
            "Twinflame: target creature #{} ({} in all)",
            n + 1,
            super::mana::cost_text(1 + 2 * n as u32, &"R".repeat(n + 1))
        );
        let cancel = if picked.is_empty() { "cancel" } else { "done" };
        let Some(m) = pick_perm(g, p, "target", &prompt, &left, Some(cancel))? else { break };
        picked.push(m);
    }
    if picked.is_empty() {
        return Ok(None);
    }
    let extra = picked.len() - 1;
    if let Some(why) = super::mana::pay_from_pool(g, p, 1 + 2 * extra as u32, &"R".repeat(extra + 1)) {
        return Ok(Some(format!("Can't cast Twinflame on {} creatures. {why}", picked.len())));
    }
    let what = format!("Twinflame copying {}", names_of(g, &picked));
    if !spell_cast(g, p, c, 3.0, Some(&what))? {
        gy_push(g, p, c);
        return Ok(None);
    }
    for m in picked {
        if !g.player(p).perms.contains(&m) {
            continue;
        }
        let cp = match g.perm(m).cd {
            Some(cd) => zones::enter_token_copy(g, p, cd)?,
            None => {
                // a token with no card: the same creature token
                let x = g.perm(m);
                let mut spec = zones::Tokens::new(1, x.pow);
                spec.tgh = Some(x.tgh);
                spec.fly = x.fly;
                spec.sick = false;
                zones::make_tokens(g, p, spec)?.first().copied()
            }
        };
        if let Some(cp) = cp {
            let x = g.perm_mut(cp);
            x.sick = false; // haste; exiled at the end step
            x.temp = true;
        }
    }
    gy_push(g, p, c);
    crate::engine::life::check_state(g)?;
    Ok(None)
}

/// cards._ephemerate
fn ephemerate(g: &mut Game, p: PlayerId, c: CardId) -> Res<Why> {
    if let Some(why) = legal::check_cast(g, p, c, "hand") {
        return Ok(Some(why));
    }
    let cre = my_creatures(g, p);
    if cre.is_empty() {
        return Ok(Some("Ephemerate needs a creature you control to target.".into()));
    }
    let Some(m) = pick_perm(g, p, "target", "Ephemerate: exile and return which creature?", &cre, Some("cancel"))?
    else {
        return Ok(None);
    };
    super::mana::pay_from_pool(g, p, 0, "W");
    let what = format!("Ephemerate on {}", g.perm(m).name);
    if !spell_cast(g, p, c, 3.0, Some(&what))? {
        gy_push(g, p, c);
        return Ok(None);
    }
    let pl = g.player_mut(p);
    pl.exile.push(c);
    pl.rebound.push(c);
    if g.player(p).perms.contains(&m) {
        blink(g, p, m)?;
    }
    Ok(None)
}

/// cards._sokenzan
fn sokenzan(g: &mut Game, p: PlayerId, c: CardId) -> Res<Why> {
    use crate::tag::Tag;
    let leg = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| {
            g.is_creature(m) && g.perm(m).cd.is_some_and(|cd| g.db.get(cd).tag(Tag::Leg)) && !g.perm(m).phased
        })
        .count() as i32;
    let n = (3 - leg).max(0) as u32;
    if let Some(why) = super::mana::cost_problem(g, p, n, "R") {
        return Ok(Some(format!("Can't channel Sokenzan. {why}")));
    }
    super::mana::pay_from_pool(g, p, n, "R");
    let pl = g.player_mut(p);
    if let Some(i) = pl.hand.iter().position(|&x| x == c) {
        pl.hand.remove(i);
    }
    pl.gy.push(c);
    crate::glog!(g, "  {} channels Sokenzan: two 1/1 Spirits with haste", pname(g, p));
    let mut spec = crate::engine::zones::Tokens::new(2, 1);
    spec.sick = false;
    spec.types = vec!["spirit"];
    crate::engine::zones::make_tokens(g, p, spec)?;
    Ok(None)
}

/// cards._insight
fn insight(g: &mut Game, p: PlayerId, c: CardId) -> Res<Why> {
    if let Some(why) = legal::sorcery_timing(g, p) {
        return Ok(Some(why.replace("do that", "cast Bloodsoaked Insight (a sorcery)")));
    }
    let gn = crate::impls::mine::insight_cost(g, p);
    let Some(ok) = ["BB", "BR", "RR"].into_iter().find(|pp| super::mana::cost_problem(g, p, gn, pp).is_none()) else {
        return Ok(Some(format!(
            "Can't cast Bloodsoaked Insight. It costs {{{gn}}}{{B/R}}{{B/R}} ({{1}} less for each life your opponents lost this turn). {}",
            super::mana::cost_problem(g, p, gn, "BB").unwrap_or_default()
        )));
    };
    let opps: Vec<PlayerId> = g.opps(p).collect();
    if opps.is_empty() {
        return Ok(Some("There is no opponent to target.".into()));
    }
    let labels: Vec<String> = opps.iter().map(|&q| pname(g, q)).collect();
    let Some(k) = pick(g, p, "target", "Bloodsoaked Insight: whose library?", &labels)? else { return Ok(None) };
    let q = opps[k];
    super::mana::pay_from_pool(g, p, gn, ok);
    let what = format!("Bloodsoaked Insight on {}", pname(g, q));
    if !spell_cast(g, p, c, 2.0, Some(&what))? {
        gy_push(g, p, c);
        return Ok(None);
    }
    let n = 3.min(g.player(q).library.len());
    let top: Vec<CardId> = (0..n).map(|_| g.player_mut(q).library.pop().unwrap()).collect();
    let until = g.player(p).turns + 1;
    let pl = g.player_mut(p);
    pl.hand.extend(top.iter().copied());
    pl.impulse_long.extend(top.iter().map(|&x| (x, until)));
    let names: Vec<String> = top.iter().map(|&x| cname(g, x)).collect();
    crate::glog!(g, "    {} from {}: playable until the end of your next turn", names.join(", "), pname(g, q));
    gy_push(g, p, c);
    Ok(None)
}

type HandFn = fn(&mut Game, PlayerId, CardId) -> Res<Why>;

/// cards.HAND: the ways to cast a hand card whose casting card code keeps for the AI: (label, fn)
fn hand_ways(name: &str) -> Option<Vec<(&'static str, HandFn)>> {
    Some(match name {
        "Twinflame" => {
            vec![("strive: token copies of creatures you control ({1}{R}, plus {2}{R} per extra target)", twinflame)]
        }
        "Ephemerate" => vec![("exile a creature you control, then return it (rebound)", ephemerate)],
        "Sokenzan, Crucible of Defiance" => {
            vec![("channel ({3}{R}, {1} less per legendary creature): two 1/1 Spirits with haste", sokenzan)]
        }
        "Bloodsoaked Insight // Sanguine Morass" => {
            vec![("Bloodsoaked Insight: an opponent's top three, playable until the end of your next turn", insight)]
        }
        "Reconnaissance Mission" | "Unearth" => {
            vec![("cast it", cast_normally), ("cycling {2}: discard it, draw a card", cycle2)]
        }
        "Clever Concealment" => vec![("phase out any number of your nonland permanents (convoke)", concealment)],
        "Rootborn Defenses" => vec![("populate; your creatures gain indestructible until end of turn", rootborn)],
        "Triplicate Spirits" => vec![("three 1/1 white flying Spirits (convoke)", triplicate)],
        "Whipcorder" => vec![
            ("cast it ({W}{W})", cast_normally),
            ("cast it face down as a 2/2 creature for {3} (morph)", morph_cast),
        ],
        _ => return None,
    })
}

/// `c.name in cards.HAND`: a hand card whose casting card code handles
pub fn hand_override(g: &Game, c: CardId) -> bool {
    hand_ways(&g.db.get(c).name).is_some()
}

/// cards.cast_from_hand: a card of yours whose casting the card code keeps for the AI: pick how (when there's a
/// choice), then cast it
pub fn cast_from_hand(g: &mut Game, p: PlayerId, c: CardId) -> Res<Why> {
    let ways = hand_ways(&cname(g, c)).unwrap_or_default();
    let mut k = 0;
    if ways.len() > 1 {
        let labels: Vec<String> = ways.iter().map(|w| w.0.to_string()).collect();
        let Some(i) = pick(g, p, "choose", &format!("{}: which?", cname(g, c)), &labels)? else { return Ok(None) };
        k = i;
    }
    (ways[k].1)(g, p, c)
}

/// `c.name in cards.GY`: Python's table is empty
pub fn gy_override(_g: &Game, _c: CardId) -> bool {
    false
}

/// `cards.GY[c.name](g, p, c)`: never reached (no card is listed)
pub fn cast_from_gy(_g: &mut Game, _p: PlayerId, _c: CardId) -> Res<Why> {
    Ok(None)
}

/// cards.needs: what c needs before it can be cast at all (the rules check asks this), or None
pub fn needs(g: &Game, p: PlayerId, c: CardId) -> Option<String> {
    let nm = cname(g, c);
    let any_is = || {
        g.player(p).gy.iter().any(|&x| {
            let d = g.db.get(x);
            d.instant || d.sorcery
        })
    };
    match nm.as_str() {
        "Twinflame" | "Ephemerate" => {
            if my_creatures(g, p).is_empty() {
                Some(format!("{nm} needs a creature you control to target."))
            } else {
                None
            }
        }
        "Mizzix's Mastery" if !any_is() => {
            Some("Mizzix's Mastery needs an instant or sorcery card in your graveyard to target.".into())
        }
        "Flashback" if !any_is() => {
            Some("Flashback needs an instant or sorcery card in your graveyard to target.".into())
        }
        "Take Up the Shield" if creatures(g, p, false).is_empty() => {
            Some("Take Up the Shield has no creature to target.".into())
        }
        _ => None,
    }
}

// ------------------------------------------------------------------ cards first written for the Zur deck
/// cards.convoke_pay: convoke: tap your creatures to help pay (each pays {1} or one coloured pip of its colour), the
/// rest from your pool. None once paid, or why not (nothing is tapped or spent then)
pub fn convoke_pay(g: &mut Game, p: PlayerId, c: CardId, gn: u32, pips: &str) -> Res<Why> {
    let mut gn = gn;
    let mut pips = pips.to_string();
    let mut tapped_ms: Vec<PermId> = vec![];
    while gn > 0 || !pips.is_empty() {
        let left: Vec<PermId> = g
            .player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| g.is_creature(m) && !g.perm(m).tapped && !g.perm(m).phased && !tapped_ms.contains(&m))
            .collect();
        if left.is_empty() {
            break;
        }
        let prompt = format!(
            "{} (convoke): tap a creature to help pay? Still to pay: {}",
            cname(g, c),
            super::mana::cost_text(gn, &pips)
        );
        let Some(m) = pick_perm(g, p, "choose", &prompt, &left, Some("pay the rest with mana"))? else { break };
        let cols = crate::engine::values::colors_of(g, m);
        if let Some(col) = pips.chars().find(|&x| cols.has(x)) {
            let i = pips.find(col).unwrap();
            pips.remove(i);
        } else if gn > 0 {
            gn -= 1;
        } else {
            continue;
        }
        tapped_ms.push(m);
    }
    if let Some(why) = super::mana::cost_problem(g, p, gn, &pips) {
        return Ok(Some(format!("Can't cast {}. {why}", cname(g, c))));
    }
    for &m in &tapped_ms {
        g.perm_mut(m).tapped = true;
    }
    super::mana::pay_from_pool(g, p, gn, &pips);
    if !tapped_ms.is_empty() {
        crate::glog!(g, "  {} taps {} to convoke {}", pname(g, p), names_of(g, &tapped_ms), cname(g, c));
    }
    Ok(None)
}

/// cards._timing
fn timing(g: &Game, p: PlayerId, c: CardId) -> Why {
    if crate::engine::values::silenced(g, p) {
        return Some("You can't cast spells during this player's turn (Conqueror's Flail).".into());
    }
    if !legal::instant_speed(g, c) {
        if let Some(why) = legal::sorcery_timing(g, p) {
            return Some(why.replace("do that", &format!("cast {}", cname(g, c))));
        }
    }
    None
}

/// cards._concealment
fn concealment(g: &mut Game, p: PlayerId, c: CardId) -> Res<Why> {
    if let Some(why) = timing(g, p, c) {
        return Ok(Some(why));
    }
    let mut picked: Vec<PermId> = vec![];
    loop {
        let left: Vec<PermId> = g
            .player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| !g.perm(m).phased && !picked.contains(&m) && g.perm(m).attached.is_none())
            .collect();
        if left.is_empty() {
            break;
        }
        let prompt = format!("Clever Concealment: phase out which of your permanents? ({} so far)", picked.len());
        let cancel = if picked.is_empty() { "cancel" } else { "done" };
        let Some(m) = pick_perm(g, p, "target", &prompt, &left, Some(cancel))? else { break };
        picked.push(m);
    }
    if picked.is_empty() {
        return Ok(None);
    }
    if let Some(why) = convoke_pay(g, p, c, 2, "WW")? {
        return Ok(Some(why));
    }
    let what = format!("Clever Concealment on {}", names_of(g, &picked));
    if !spell_cast(g, p, c, 4.0, Some(&what))? {
        gy_push(g, p, c);
        return Ok(None);
    }
    for &m in &picked {
        if g.player(p).perms.contains(&m) {
            g.perm_mut(m).phased = true;
            let all: Vec<PermId> = g
                .player(p)
                .perms
                .clone()
                .into_iter()
                .chain(g.players.iter().flat_map(|q| q.perms.clone()).collect::<Vec<_>>())
                .collect();
            for x in all {
                if g.perm(x).attached == Some(m) {
                    g.perm_mut(x).phased = true; // what's attached phases out with it
                }
            }
        }
    }
    gy_push(g, p, c);
    crate::glog!(g, "    {} phase out", names_of(g, &picked));
    Ok(None)
}

/// cards._rootborn
fn rootborn(g: &mut Game, p: PlayerId, c: CardId) -> Res<Why> {
    if let Some(why) = timing(g, p, c).or_else(|| super::mana::cost_problem(g, p, 2, "W")) {
        let keep =
            (why.starts_with("You") || why.starts_with("Cast") || why.starts_with("It")) && !why.contains("costs");
        return Ok(Some(if keep { why } else { format!("Can't cast {}. {why}", cname(g, c)) }));
    }
    super::mana::pay_from_pool(g, p, 2, "W");
    if !spell_cast(g, p, c, 3.0, None)? {
        gy_push(g, p, c);
        return Ok(None);
    }
    crate::impls::zur::rootborn(g, p)?;
    gy_push(g, p, c);
    Ok(None)
}

/// cards._triplicate
fn triplicate(g: &mut Game, p: PlayerId, c: CardId) -> Res<Why> {
    if let Some(why) = timing(g, p, c) {
        return Ok(Some(why));
    }
    if let Some(why) = convoke_pay(g, p, c, 4, "WW")? {
        return Ok(Some(why));
    }
    if !spell_cast(g, p, c, 3.0, None)? {
        gy_push(g, p, c);
        return Ok(None);
    }
    let mut spec = crate::engine::zones::Tokens::new(3, 1);
    spec.fly = true;
    spec.color = Some(crate::cards::Colors::from_letters("W"));
    spec.types = vec!["spirit"];
    crate::engine::zones::make_tokens(g, p, spec)?;
    gy_push(g, p, c);
    Ok(None)
}

/// cards._morph_cast
fn morph_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res<Why> {
    if let Some(why) = timing(g, p, c).or_else(|| cost(g, p, 3, "", "cast it face down")) {
        return Ok(Some(why.replace("Can't activate", "Can't cast")));
    }
    super::mana::pay_from_pool(g, p, 3, "");
    if !spell_cast(g, p, c, 2.0, Some("a face-down creature (morph)"))? {
        gy_push(g, p, c);
        return Ok(None);
    }
    crate::impls::galadriel::enter_face_down(g, p, c)?;
    Ok(None)
}

// ------------------------------------------------------------------ The Eternal Wanderer, Azorius Guildmage, Niv-Mizzet
fn wanderer_plus(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    use crate::cards::Types;
    let cands: Vec<PermId> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&x| {
            !g.perm(x).phased
                && (g.is_creature(x) || g.perm(x).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)))
                && !(g.perm(x).owner != p && crate::engine::values::untargetable(g, x))
        })
        .collect();
    let t = if cands.is_empty() {
        None
    } else {
        pick_perm(
            g,
            p,
            "target",
            "The Eternal Wanderer +1: exile which artifact or creature until its owner's next end step?",
            &cands,
            Some("no target"),
        )?
    };
    let Some(t) = t else { return go(|_| Ok(())) };
    go(move |g| {
        if g.perm(t).on_bf && g.player(g.perm(t).owner).perms.contains(&t) {
            crate::impls::zur::wanderer_exile(g, p, t)?;
        }
        Ok(())
    })
}

fn wanderer_zero(_g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    go(move |g| {
        let mut spec = crate::engine::zones::Tokens::new(1, 2);
        spec.color = Some(crate::cards::Colors::from_letters("W"));
        spec.types = vec!["samurai"];
        for x in crate::engine::zones::make_tokens(g, p, spec)? {
            g.perm_mut(x).data.set(
                crate::state::DataKey::Kws,
                crate::state::Val::List(vec![crate::state::Val::Str("double strike")]),
            );
        }
        Ok(())
    })
}

fn wanderer_minus4(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    let mine = my_creatures(g, p);
    let mut keep = None;
    if !mine.is_empty() {
        let labels = target_labels(g, p, &mine.iter().map(|&x| Target::Perm(x)).collect::<Vec<_>>());
        let k = choices::choose_one(
            g,
            p,
            "choose",
            "The Eternal Wanderer -4: which of your creatures do you keep? (the rest are sacrificed)",
            &labels,
        )?;
        keep = Some(mine[k]);
    }
    go(move |g| {
        for q in g.players.iter().filter(|q| q.alive).map(|q| q.id).collect::<Vec<_>>() {
            let k2 = if q == p { keep } else { crate::impls::zur::keep_one(g, q, p) };
            let doomed: Vec<PermId> = g
                .player(q)
                .perms
                .iter()
                .copied()
                .filter(|&x| g.is_creature(x) && !g.perm(x).phased && Some(x) != k2)
                .collect();
            for x in doomed {
                crate::engine::zones::die(g, x, "sac")?;
            }
        }
        Ok(())
    })
}

fn guildmage_tap(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = cost(g, p, 2, "W", "Azorius Guildmage") {
        return Ok(Some(why));
    }
    let cre = creatures(g, p, false);
    if cre.is_empty() {
        return Ok(Some("There is no creature to tap.".into()));
    }
    let Some(t) = pick_perm(g, p, "target", "Azorius Guildmage: tap which creature?", &cre, Some("cancel"))? else {
        return Ok(None);
    };
    super::mana::pay_from_pool(g, p, 2, "W");
    crate::glog!(g, "  {} activates Azorius Guildmage: tap {}", pname(g, p), g.perm(t).name);
    let what = format!("tap {}", g.perm(t).name);
    if crate::engine::stack::ability_window(g, p, Some(m), &what, None, Some(t))? {
        g.perm_mut(t).tapped = true;
    }
    Ok(None)
}

fn guildmage_counter(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    use crate::state::StackKind;
    let abil: Vec<(u32, String, PlayerId)> = g
        .stack
        .iter()
        .rev()
        .filter(|it| it.kind == StackKind::Ability)
        .map(|it| (it.id, it.name.clone(), it.controller))
        .collect();
    if abil.is_empty() {
        return Ok(Some("There is no activated ability on the stack to counter.".into()));
    }
    if let Some(why) = cost(g, p, 2, "U", "Azorius Guildmage") {
        return Ok(Some(why));
    }
    let mut k = 0;
    if abil.len() > 1 {
        let labels: Vec<String> = abil.iter().map(|(_, n, c)| format!("{n} ({})", pname(g, *c))).collect();
        let Some(i) = pick(g, p, "target", "Azorius Guildmage: counter which ability?", &labels)? else {
            return Ok(None);
        };
        k = i;
    }
    let (id, name, _) = abil[k].clone();
    super::mana::pay_from_pool(g, p, 2, "U");
    crate::glog!(g, "  {} activates Azorius Guildmage: counter {}", pname(g, p), name);
    if crate::engine::stack::ability_window(g, p, Some(m), &format!("counter {name}"), None, None)? {
        if let Some(it) = g.stack.iter_mut().find(|it| it.id == id) {
            it.countered = true;
        }
    }
    Ok(None)
}

fn guildmage(g: &Game) -> Vec<PermAbility> {
    use crate::state::StackKind;
    let mut out = vec![ability("{2}{W}: tap target creature", guildmage_tap)];
    if g.stack.iter().any(|it| it.kind == StackKind::Ability) {
        out.push(ability("{2}{U}: counter target activated ability", guildmage_counter));
    }
    out
}

fn niv_draw(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tapped(g, m, None) {
        return Ok(Some(why));
    }
    if g.perm(m).sick {
        return Ok(Some("Niv-Mizzet has summoning sickness (it came under your control this turn).".into()));
    }
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "  {} activates Niv-Mizzet, the Firemind: draw a card", pname(g, p));
    if crate::engine::stack::ability_window(g, p, Some(m), "draw a card", None, None)? {
        crate::engine::zones::draw(g, p, 1, false)?;
    }
    Ok(None)
}

// ------------------------------------------------------------------ copying a spell: Return the Favor, Dualcaster Mage
/// cards.copy_card
pub fn copy_card(g: &Game, c: CardId) -> bool {
    matches!(&*g.db.get(c).name, "Return the Favor" | "Dualcaster Mage")
}

/// cards.copy_in_response: p casts Return the Favor or Dualcaster Mage at `spell` (cast by `caster`, on the stack
/// now). None, or why not
pub fn copy_in_response(g: &mut Game, p: PlayerId, c: CardId, spell: Option<CardId>, caster: PlayerId) -> Res<Why> {
    let Some(spell) = spell else { return Ok(None) };
    let sd = g.db.get(spell);
    let (sname, s_is) = (sd.name.to_string(), sd.instant || sd.sorcery);
    let ctx: Option<crate::state::Ctx> = match &g.cur_cast {
        Some((cc, ctx, _)) if *cc == spell => Some(ctx.clone()),
        _ => None,
    };
    if &*g.db.get(c).name == "Dualcaster Mage" {
        if !s_is {
            return Ok(Some(format!("Dualcaster Mage copies an instant or sorcery spell; {sname} is not one.")));
        }
        if let Some(why) = super::mana::cost_problem(g, p, 1, "RR") {
            return Ok(Some(format!("Can't cast Dualcaster Mage. {why}")));
        }
        super::mana::pay_from_pool(g, p, 1, "RR");
        crate::glog!(g, "  {} flashes in Dualcaster Mage to copy {}", pname(g, p), sname);
        if crate::engine::cast::cast_card(g, p, c, "hand", Default::default())? && !g.over {
            crate::engine::cast::copy_spell(g, p, spell, ctx.as_ref(), false)?;
        }
        return Ok(None);
    }
    let mut modes: Vec<(&str, String)> = vec![];
    if s_is {
        modes.push(("copy", format!("copy {sname} (you may choose new targets)")));
    }
    if ctx.as_ref().is_some_and(|x| x.target.is_some() || x.face.is_some()) {
        modes.push(("redirect", format!("change the target of {sname}")));
    }
    if modes.is_empty() {
        return Ok(Some(format!(
            "Return the Favor has nothing to do to {sname} (not an instant or sorcery, and no single target)."
        )));
    }
    let mut labels: Vec<String> = modes.iter().map(|(_, t)| format!("{t} (+{{1}})")).collect();
    if modes.len() == 2 {
        labels.push("both (+{2})".into());
    }
    let Some(k) = pick(g, p, "choose", "Return the Favor (spree): which modes?", &labels)? else { return Ok(None) };
    let chosen: Vec<&str> = if k < modes.len() { vec![modes[k].0] } else { modes.iter().map(|m| m.0).collect() };
    if let Some(why) = super::mana::cost_problem(g, p, chosen.len() as u32, "RR") {
        return Ok(Some(format!("Can't cast Return the Favor. {why}")));
    }
    let mut new: Option<Target> = None;
    if chosen.contains(&"redirect") {
        let cx = ctx.clone().unwrap_or_default();
        let tg: Vec<Target> = legal::spell_targets(g, caster, spell)
            .unwrap_or_default()
            .into_iter()
            .filter(|x| match x {
                Target::Perm(m) => cx.target != Some(*m),
                Target::Player(q) => cx.face != Some(*q),
            })
            .collect();
        if tg.is_empty() {
            return Ok(Some(format!("{sname} has no other legal target.")));
        }
        let labels = target_labels(g, p, &tg);
        let Some(j) = pick(g, p, "target", &format!("Return the Favor: the new target for {sname}"), &labels)? else {
            return Ok(None);
        };
        new = Some(tg[j]);
    }
    super::mana::pay_from_pool(g, p, chosen.len() as u32, "RR");
    let what = format!("Return the Favor ({}) at {}", chosen.join(" and "), sname);
    if !spell_cast(g, p, c, 5.0, Some(&what))? {
        gy_push(g, p, c);
        return Ok(None);
    }
    let mut ctx = ctx;
    if let (Some(n), Some(_)) = (new, ctx.as_ref()) {
        // the live context of the spell on the stack changes with it
        if let Some((_, live, _)) = g.cur_cast.as_mut() {
            live.target = None;
            live.face = None;
            match n {
                Target::Player(q) => live.face = Some(q),
                Target::Perm(m) => live.target = Some(m),
            }
            ctx = Some(live.clone());
        }
        crate::glog!(g, "    {} now targets {}", sname, legal::describe_target(g, p, n));
    }
    if chosen.contains(&"copy") {
        crate::engine::cast::copy_spell(g, p, spell, ctx.as_ref(), false)?;
    }
    gy_push(g, p, c);
    Ok(None)
}

// ------------------------------------------------------------------ choices as your spells resolve (Veyran's deck)
/// cards.jeskas_will: choose one (both if you control your commander): add {R} for each card in target opponent's
/// hand; exile the top three cards of your library, you may play them this turn. True: handled
pub fn jeskas_will(g: &mut Game, p: PlayerId) -> Res<bool> {
    let mut opts: Vec<String> = vec![
        "add {R} for each card in target opponent's hand".into(),
        "exile your top three cards: you may play them this turn".into(),
    ];
    if crate::engine::values::commander_out(g, p) {
        opts.push("both".into());
    }
    let prompt = format!(
        "Jeska's Will: choose one{}",
        if opts.len() == 3 { " (or both: you control your commander)" } else { "" }
    );
    let k = choices::choose_one(g, p, "choose", &prompt, &opts)?;
    if k == 0 || k == 2 {
        let opps: Vec<PlayerId> = g.opps(p).collect();
        if !opps.is_empty() {
            let labels: Vec<String> =
                opps.iter().map(|&q| format!("{} ({} cards)", pname(g, q), g.player(q).hand.len())).collect();
            let j = choices::choose_one(g, p, "target", "Jeska's Will: which opponent's hand?", &labels)?;
            let n = g.player(opps[j]).hand.len() as u32;
            g.player_mut(p).pool.add('R', n);
            crate::glog!(g, "    Jeska's Will adds {} red mana", n);
        }
    }
    if k == 1 || k == 2 {
        let top = look(g, p, 3);
        for &x in &top {
            let pl = g.player_mut(p);
            pl.hand.push(x);
            pl.seen.insert(x);
        }
        g.player_mut(p).impulse.extend(top.iter().copied());
        let names: Vec<String> = top.iter().map(|&x| cname(g, x)).collect();
        crate::glog!(g, "    Jeska's Will exiles {} (playable this turn)", names.join(", "));
    }
    Ok(true)
}

/// cards.crackle: 5X damage to each of up to X targets. True: handled
pub fn crackle(g: &mut Game, p: PlayerId, _c: CardId, ctx: &crate::state::Ctx) -> Res<bool> {
    let x = ctx.x;
    if x <= 0 {
        return Ok(true);
    }
    let mut tg = choices::damage_targets(g, p, crate::cards::Colors::from_letters("R"));
    let mut picked: Vec<Target> = vec![];
    while (picked.len() as i32) < x && !tg.is_empty() {
        let labels = target_labels(g, p, &tg);
        let prompt =
            format!("Crackle with Power: {} damage to which target? ({} of up to {})", 5 * x, picked.len() + 1, x);
        let cancel = if picked.is_empty() { None } else { Some("no more targets") };
        let Some(k) = choices::choose(g, p, "target", &prompt, &labels, cancel, None)? else { break };
        picked.push(tg.remove(k));
    }
    for t in picked {
        damage(g, p, t, 5 * x, "Crackle with Power", "burn")?;
    }
    Ok(true)
}

/// cards.mastery_target: Mizzix's Mastery's target as it's cast: an instant or sorcery card in your graveyard (None:
/// cancelled)
pub fn mastery_target(g: &mut Game, p: PlayerId, c: CardId) -> Res<Option<CardId>> {
    let cs: Vec<CardId> =
        g.player(p).gy.iter().copied().filter(|&x| (g.db.get(x).instant || g.db.get(x).sorcery) && x != c).collect();
    let labels: Vec<String> = cs.iter().map(|&x| choices::card_label(g, x)).collect();
    let k = pick(g, p, "target", "Mizzix's Mastery: exile which instant or sorcery (you cast a copy free)?", &labels)?;
    Ok(k.map(|k| cs[k]))
}

/// cards.flashback_target: Flashback (the card): target instant or sorcery card in your graveyard gains flashback
/// this turn. True: handled
pub fn flashback_target(g: &mut Game, p: PlayerId) -> Res<bool> {
    let cs: Vec<CardId> =
        g.player(p).gy.iter().copied().filter(|&x| g.db.get(x).instant || g.db.get(x).sorcery).collect();
    // (Python excludes the Flashback card itself, c; it's on the stack, not in the graveyard, so no difference)
    if cs.is_empty() {
        return Ok(true);
    }
    let labels: Vec<String> = cs.iter().map(|&x| choices::card_label(g, x)).collect();
    let k = choices::choose_one(
        g,
        p,
        "target",
        "Flashback: which instant or sorcery gains flashback this turn (its mana cost)?",
        &labels,
    )?;
    let st = g.turn_stamp();
    let pl = g.player_mut(p);
    if pl.fb_grant.as_ref().map(|x| x.0) != Some(st) {
        pl.fb_grant = Some((st, vec![])); // last turn's grants have ended
    }
    pl.fb_grant.as_mut().unwrap().1.push(cs[k]);
    crate::glog!(g, "    {} gains flashback this turn", cname(g, cs[k]));
    Ok(true)
}

/// cards.granted_flashback
pub fn granted_flashback(g: &Game, p: PlayerId, c: CardId) -> bool {
    matches!(&g.player(p).fb_grant, Some((st, cs)) if *st == g.turn_stamp() && cs.contains(&c))
}

/// cards.dredge: your draw step: dredge a card in your graveyard instead of drawing? True if you did
pub fn dredge(g: &mut Game, p: PlayerId) -> Res<bool> {
    use crate::tag::Tag;
    let imps: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).tag(Tag::Dredge)).collect();
    if imps.is_empty() || g.player(p).library.len() < 5 {
        return Ok(false);
    }
    let nm = cname(g, imps[0]);
    if !choices::yes_no(
        g,
        p,
        &format!("Draw step: dredge {nm} (mill five, return it to your hand) instead of drawing?"),
    )? {
        return Ok(false);
    }
    let pl = g.player_mut(p);
    if let Some(i) = pl.gy.iter().position(|&x| x == imps[0]) {
        pl.gy.remove(i);
    }
    pl.hand.push(imps[0]);
    crate::engine::zones::mill(g, p, 5)?;
    g.player_mut(p).stat(crate::sym::intern("dredged"), 1);
    crate::glog!(g, "  {} dredges {} (mills five)", pname(g, p), nm);
    Ok(true)
}

/// cards.CAST_TARGET: tagless spells with a creature target chosen as they're cast
pub enum CastTarget {
    /// the card has no such target
    NotNeeded,
    /// the person cancelled
    Cancelled,
    Picked(PermId),
}

/// cards.CAST_TARGET lookup + pick_creature(optional=True, keep=...): only Take Up the Shield is listed
pub fn cast_target(g: &Game, p: PlayerId, c: CardId) -> Res<CastTarget> {
    if &*g.db.get(c).name != "Take Up the Shield" {
        return Ok(CastTarget::NotNeeded);
    }
    let keep =
        |g: &Game, p: PlayerId, m: PermId| !(g.perm(m).owner == p && crate::impls::zur::untargetable_by_you(g, m));
    let prompt = "Take Up the Shield: which creature gets a +1/+1 counter, lifelink and indestructible?";
    Ok(match pick_creature(g, p, prompt, false, true, Some(&keep))? {
        Some(t) => CastTarget::Picked(t),
        None => CastTarget::Cancelled,
    })
}

// ------------------------------------------------------------------ Galadriel's deck (Bant Rebels)
fn creature_tap_ok(g: &Game, m: PermId) -> Why {
    tapped(g, m, None).or_else(|| {
        if g.perm(m).sick {
            Some(format!("{} has summoning sickness (it came under your control this turn).", pm_name(g, m)))
        } else {
            None
        }
    })
}

/// cards._gal_pay: pay a creature's ability from your pool (Secluded Courtyard's mana may pay if it's of the named
/// type). None once paid, or why not
fn gal_pay(g: &mut Game, p: PlayerId, m: PermId, gn: u32, pips: &str, what: &str) -> Why {
    use super::mana::Spending;
    let prev = std::mem::replace(&mut g.player_mut(p).pool.spending, Some(Spending::Perm(m)));
    let why = cost(g, p, gn, pips, what).or_else(|| {
        super::mana::pay_from_pool(g, p, gn, pips);
        None
    });
    g.player_mut(p).pool.spending = prev;
    why
}

/// cards._pick_x: X for an ability, up to the mana in your pool
fn pick_x(g: &Game, p: PlayerId, what: &str, lo: u32) -> Res<Option<u32>> {
    let f = g.player(p).floating;
    let n = g.player(p).pool.total() + f.u + f.b + f.r + f.g + f.c + f.any;
    if n < lo {
        return Ok(None);
    }
    let xs: Vec<u32> = (lo..=n).collect();
    let labels: Vec<String> = xs.iter().map(|x| format!("X = {x}")).collect();
    Ok(pick(g, p, "choose", &format!("{what}: X = ? (your pool has {n} mana)"), &labels)?.map(|k| xs[k]))
}

fn searcher(gn: u32, cap: u32) -> Vec<PermAbility> {
    vec![ability(
        format!(
            "{{{gn}}}, {{T}}: search for a Rebel permanent card with mana value {cap} or less, put it onto the battlefield"
        ),
        move |g: &mut Game, p: PlayerId, m: PermId| {
            if let Some(why) = creature_tap_ok(g, m).or_else(|| {
                let nm = pm_name(g, m);
                gal_pay(g, p, m, gn, "", &nm)
            }) {
                return Ok(Some(why));
            }
            g.perm_mut(m).tapped = true;
            let nm = pm_name(g, m);
            crate::glog!(g, "  {} activates {}", pname(g, p), nm);
            if crate::engine::stack::ability_window(
                g,
                p,
                Some(m),
                &format!("search for a Rebel (mana value {cap} or less)"),
                None,
                None,
            )? {
                crate::impls::galadriel::put_rebel(g, p, &nm, cap, false)?;
            }
            Ok(None)
        },
    )]
}

fn lin_search(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = creature_tap_ok(g, m) {
        return Ok(Some(why));
    }
    let Some(x) = pick_x(g, p, "Lin Sivvi", 0)? else { return Ok(None) };
    if let Some(why) = gal_pay(g, p, m, x, "", "Lin Sivvi") {
        return Ok(Some(why));
    }
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "  {} activates Lin Sivvi, X = {}", pname(g, p), x);
    if crate::engine::stack::ability_window(
        g,
        p,
        Some(m),
        &format!("search for a Rebel (mana value {x} or less)"),
        None,
        None,
    )? {
        crate::impls::galadriel::put_rebel(g, p, "Lin Sivvi", x, false)?;
    }
    Ok(None)
}

fn lin_bottom(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    let gy = g.player(p).gy.clone();
    let mut cs = crate::impls::galadriel::rebel_cards(g, p, &gy, 99);
    cs.extend(gy.iter().copied().filter(|&c| !g.db.get(c).perm && g.db.get(c).has_subtype("rebel")));
    if cs.is_empty() {
        return Ok(Some("There is no Rebel card in your graveyard.".into()));
    }
    let labels: Vec<String> = cs.iter().map(|&c| cname(g, c)).collect();
    let Some(k) = pick(
        g,
        p,
        "target",
        "Lin Sivvi: put which Rebel card from your graveyard on the bottom of your library?",
        &labels,
    )?
    else {
        return Ok(None);
    };
    if let Some(why) = gal_pay(g, p, m, 3, "", "Lin Sivvi") {
        return Ok(Some(why));
    }
    let c = cs[k];
    crate::glog!(g, "  {} activates Lin Sivvi: {} to the bottom of the library", pname(g, p), cname(g, c));
    if crate::engine::stack::ability_window(
        g,
        p,
        Some(m),
        &format!("put {} on the bottom of the library", cname(g, c)),
        None,
        None,
    )? {
        let pl = g.player_mut(p);
        if let Some(i) = pl.gy.iter().position(|&x| x == c) {
            pl.gy.remove(i);
            pl.library.insert(0, c);
        }
    }
    Ok(None)
}

fn revivalist(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = creature_tap_ok(g, m) {
        return Ok(Some(why));
    }
    let gy = g.player(p).gy.clone();
    if crate::impls::galadriel::rebel_cards(g, p, &gy, 5).is_empty() {
        return Ok(Some("There is no Rebel permanent card with mana value 5 or less in your graveyard.".into()));
    }
    if let Some(why) = gal_pay(g, p, m, 6, "", "Ramosian Revivalist") {
        return Ok(Some(why));
    }
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "  {} activates Ramosian Revivalist", pname(g, p));
    if crate::engine::stack::ability_window(g, p, Some(m), "return a Rebel from your graveyard", None, None)? {
        crate::impls::galadriel::put_rebel(g, p, "Ramosian Revivalist", 5, true)?;
    }
    Ok(None)
}

fn bringer(colour: char, word: &'static str) -> Vec<PermAbility> {
    let label = format!("{{T}}, sacrifice it: exile target {word} creature");
    vec![ability(label, move |g: &mut Game, p: PlayerId, m: PermId| {
        if let Some(why) = creature_tap_ok(g, m) {
            return Ok(Some(why));
        }
        let w = crate::cards::Colors::from_letters("W");
        let cre: Vec<PermId> = creatures(g, p, false)
            .into_iter()
            .filter(|&x| {
                crate::engine::values::colors_of(g, x).has(colour) && !crate::engine::values::protected_from(g, x, w)
            })
            .collect();
        if cre.is_empty() {
            return Ok(Some(format!("There is no {word} creature to target.")));
        }
        let prompt = format!("{}: exile which {word} creature?", pm_name(g, m));
        let Some(t) = pick_perm(g, p, "target", &prompt, &cre, Some("cancel"))? else { return Ok(None) };
        crate::impls::galadriel::bringer_use(g, p, m, t)?;
        Ok(None)
    })]
}

fn ballista(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = creature_tap_ok(g, m) {
        return Ok(Some(why));
    }
    let w = crate::cards::Colors::from_letters("W");
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
                && !(g.perm(x).owner != p && crate::engine::values::untargetable(g, x))
                && !crate::engine::values::protected_from(g, x, w)
        })
        .collect();
    if cre.is_empty() {
        return Ok(Some("Ballista Squad needs an attacking or blocking creature to target.".into()));
    }
    let Some(t) = pick_perm(
        g,
        p,
        "target",
        "Ballista Squad: X damage to which attacking or blocking creature?",
        &cre,
        Some("cancel"),
    )?
    else {
        return Ok(None);
    };
    let Some(x) = pick_x(g, p, "Ballista Squad (plus {W})", 0)? else { return Ok(None) };
    if let Some(why) = gal_pay(g, p, m, x, "W", "Ballista Squad") {
        return Ok(Some(why));
    }
    g.perm_mut(m).tapped = true;
    let tn = g.perm(t).name;
    crate::glog!(g, "  {} activates Ballista Squad: {} damage to {}", pname(g, p), x, tn);
    if crate::engine::stack::ability_window(g, p, Some(m), &format!("{x} damage to {tn}"), None, Some(t))?
        && g.player(g.perm(t).owner).perms.contains(&t)
    {
        if crate::engine::values::no_damage(g, t) {
            crate::glog!(g, "    the damage to {} is prevented", tn);
        } else if crate::engine::values::etgh(g, t) <= x as i32 {
            crate::engine::removal::apply_removal(g, Some(p), t, &format!("dmg{x}"), None)?;
        } else {
            crate::glog!(g, "    {} survives", tn);
        }
    }
    Ok(None)
}

fn tapper(gn: u32, pips: &'static str, tough: i32, label: &'static str) -> Vec<PermAbility> {
    vec![ability(label, move |g: &mut Game, p: PlayerId, m: PermId| {
        if let Some(why) = creature_tap_ok(g, m) {
            return Ok(Some(why));
        }
        let cre: Vec<PermId> =
            creatures(g, p, false).into_iter().filter(|&x| crate::engine::values::etgh(g, x) <= tough).collect();
        if cre.is_empty() {
            return Ok(Some("There is no creature it can tap.".into()));
        }
        let nm = pm_name(g, m);
        let Some(t) = pick_perm(g, p, "target", &format!("{nm}: tap which creature?"), &cre, Some("cancel"))? else {
            return Ok(None);
        };
        if let Some(why) = gal_pay(g, p, m, gn, pips, &nm) {
            return Ok(Some(why));
        }
        g.perm_mut(m).tapped = true;
        let tn = g.perm(t).name;
        crate::glog!(g, "  {} activates {}: tap {}", pname(g, p), nm, tn);
        if crate::engine::stack::ability_window(g, p, Some(m), &format!("tap {tn}"), None, Some(t))?
            && g.player(g.perm(t).owner).perms.contains(&t)
        {
            g.perm_mut(t).tapped = true;
        }
        Ok(None)
    })]
}

fn maskwood(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tapped(g, m, None).or_else(|| cost(g, p, 3, "", "Maskwood Nexus")) {
        return Ok(Some(why));
    }
    super::mana::pay_from_pool(g, p, 3, "");
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "  {} activates Maskwood Nexus", pname(g, p));
    if crate::engine::stack::ability_window(g, p, Some(m), "a 2/2 Shapeshifter with changeling", None, None)? {
        crate::impls::galadriel::shapeshifter(g, p)?;
    }
    Ok(None)
}

fn mirror(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    let Some(x) = pick_x(g, p, "Mirror Entity", 0)? else { return Ok(None) };
    if let Some(why) = gal_pay(g, p, m, x, "", "Mirror Entity") {
        return Ok(Some(why));
    }
    crate::glog!(g, "  {} activates Mirror Entity, X = {}", pname(g, p), x);
    if crate::engine::stack::ability_window(g, p, Some(m), &format!("your creatures become {x}/{x}"), None, None)? {
        crate::impls::galadriel::mirror(g, p, x as i32)?;
    }
    Ok(None)
}

fn elspeth_plus(_g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    go(move |g| {
        let mut spec = crate::engine::zones::Tokens::new(3, 1);
        spec.color = Some(crate::cards::Colors::from_letters("W"));
        spec.types = vec!["soldier"];
        crate::engine::zones::make_tokens(g, p, spec)?;
        Ok(())
    })
}

fn elspeth_minus3(_g: &mut Game, _p: PlayerId, _m: PermId) -> Res<Ready> {
    go(|g| {
        for q in g.players.iter().map(|q| q.id).collect::<Vec<_>>() {
            for x in g.player(q).perms.clone() {
                if g.is_creature(x) && !g.perm(x).phased && crate::engine::values::epow(g, x) >= 4 {
                    crate::engine::zones::die(g, x, "destroy")?;
                }
            }
        }
        Ok(())
    })
}

fn elspeth_minus7(_g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    go(move |g| {
        crate::impls::galadriel::elspeth_emblem(g, p);
        Ok(())
    })
}

fn turn_up(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = cost(g, p, 0, "W", "turn Whipcorder face up") {
        return Ok(Some(why));
    }
    super::mana::pay_from_pool(g, p, 0, "W");
    crate::impls::galadriel::turn_face_up(g, p, m);
    Ok(None)
}

// ------------------------------------------------------------------ Necropotence (Zur's deck)
fn necro(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if crate::engine::combat::blocked(g, p, "Necropotence") {
        return Ok(Some("Disruptor Flute names Necropotence: its ability can't be activated.".into()));
    }
    let most = (g.player(p).life.min(g.player(p).library.len() as i32)).min(30);
    if most < 1 {
        return Ok(Some("You have no life or no library to pay with.".into()));
    }
    let labels: Vec<String> = (1..=most).map(|n| format!("pay {n} life")).collect();
    let prompt = format!(
        "Necropotence: pay how much life? (you have {}; the cards come to your hand at your next end step)",
        g.player(p).life
    );
    let Some(k) = pick(g, p, "choose", &prompt, &labels)? else { return Ok(None) };
    let n = k as i32 + 1;
    crate::engine::life::lose_life(g, p, n, Some(p), "other", None)?;
    crate::glog!(g, "  {} pays {} life to Necropotence", pname(g, p), n);
    if crate::engine::stack::ability_window(g, p, Some(m), &format!("exile the top {n} card(s) face down"), None, None)?
        && !g.over
    {
        necro_exile(g, p, n as usize);
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

/// ais.necro_exile: Necropotence, activated n times by the person: the top n cards exiled face down until p's next end
/// step (this turn's if it's p's turn before the end step, else the next)
fn necro_exile(g: &mut Game, p: PlayerId, n: usize) {
    let ready = if g.active == Some(p) && g.step != crate::state::Step::End {
        g.player(p).turns
    } else {
        g.player(p).turns + 1
    };
    for _ in 0..n.min(g.player(p).library.len()) {
        let c = g.player_mut(p).library.pop().unwrap();
        g.player_mut(p).necro_wait.push((c, ready));
    }
}

// ------------------------------------------------------------------ Y'shtola's deck (Esper Drain)
/// cards._own_hand: the cards in your hand (not the ones taken from opponents, which are in exile)
fn own_hand(g: &Game, p: PlayerId) -> Vec<CardId> {
    g.player(p).hand.iter().copied().filter(|c| !g.player(p).stolen.contains_key(c)).collect()
}

fn syphon_mage(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    let why = creature_tap_ok(g, m)
        .or_else(|| if own_hand(g, p).is_empty() { Some("You have no card in hand to discard.".into()) } else { None })
        .or_else(|| cost(g, p, 2, "B", "Urborg Syphon-Mage"));
    if why.is_some() {
        return Ok(why);
    }
    let cs = own_hand(g, p);
    let labels: Vec<String> = cs.iter().map(|&c| choices::card_label(g, c)).collect();
    let Some(k) = pick(g, p, "choose", "Urborg Syphon-Mage: discard which card?", &labels)? else { return Ok(None) };
    super::mana::pay_from_pool(g, p, 2, "B");
    g.perm_mut(m).tapped = true;
    crate::engine::zones::discard_cards(g, p, &[cs[k]])?;
    crate::impls::yshtola::syphon(g, p, m)?;
    Ok(None)
}

fn inheritance(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = cost(g, p, 5, "B", "Ill-Gotten Inheritance") {
        return Ok(Some(why));
    }
    let opps: Vec<PlayerId> = g.opps(p).collect();
    if opps.is_empty() {
        return Ok(Some("There is no opponent to target.".into()));
    }
    let labels: Vec<String> = opps.iter().map(|&q| format!("{} ({} life)", pname(g, q), g.player(q).life)).collect();
    let Some(k) = pick(g, p, "target", "Ill-Gotten Inheritance: 4 damage to which opponent?", &labels)? else {
        return Ok(None);
    };
    super::mana::pay_from_pool(g, p, 5, "B");
    crate::impls::yshtola::inheritance_sac(g, p, m, opps[k])?;
    Ok(None)
}

fn jesters_cap(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tapped(g, m, None).or_else(|| cost(g, p, 2, "", "Jester's Cap")) {
        return Ok(Some(why));
    }
    let ps: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    let labels: Vec<String> =
        ps.iter().map(|&q| format!("{}{}", pname(g, q), if q == p { " (you)" } else { "" })).collect();
    let Some(k) = pick(g, p, "target", "Jester's Cap: search which player's library?", &labels)? else {
        return Ok(None);
    };
    super::mana::pay_from_pool(g, p, 2, "");
    crate::impls::yshtola::cap_use(g, p, m, ps[k])?;
    Ok(None)
}

fn narset_minus2(_g: &mut Game, _p: PlayerId, m: PermId) -> Res<Ready> {
    let p = _p;
    go(move |g| crate::impls::common::narset_dig(g, p, m))
}

/// the rest of cards.ABILITIES (the decks added later in cards.py)
fn more_abilities(g: &Game, p: PlayerId, m: PermId, nm: &str) -> Option<Vec<PermAbility>> {
    let _ = p;
    Some(match nm {
        "Azorius Guildmage" => guildmage(g),
        "The Eternal Wanderer" => walker(&[
            (1, "exile up to one target artifact or creature until its owner's next end step", wanderer_plus),
            (0, "create a 2/2 white Samurai with double strike", wanderer_zero),
            (-4, "each player keeps one creature and sacrifices the rest", wanderer_minus4),
        ]),
        "Niv-Mizzet, the Firemind" => vec![ability("{T}: draw a card", niv_draw)],
        "Ramosian Sergeant" => searcher(3, 2),
        "Ramosian Lieutenant" => searcher(4, 3),
        "Ramosian Captain" | "Defiant Vanguard" => searcher(5, 4),
        "Ramosian Commander" => searcher(6, 5),
        "Lin Sivvi, Defiant Hero" => vec![
            ability(
                "{X}, {T}: search for a Rebel permanent card with mana value X or less, put it onto the battlefield",
                lin_search,
            ),
            ability("{3}: put target Rebel card from your graveyard on the bottom of your library", lin_bottom),
        ],
        "Ramosian Revivalist" => vec![ability(
            "{6}, {T}: return a Rebel permanent card with mana value 5 or less from your graveyard to the battlefield",
            revivalist,
        )],
        "Lawbringer" => bringer('R', "red"),
        "Lightbringer" => bringer('B', "black"),
        "Ballista Squad" => vec![ability("{X}{W}, {T}: X damage to target attacking or blocking creature", ballista)],
        "Errant Doomsayers" => tapper(0, "", 2, "{T}: tap target creature with toughness 2 or less"),
        "Whipcorder" => {
            if g.perm(m).data.truthy(crate::state::DataKey::Facedown) {
                vec![ability("turn it face up ({W})", turn_up)]
            } else {
                tapper(0, "W", 999, "{W}, {T}: tap target creature")
            }
        }
        "Maskwood Nexus" => vec![ability("{3}, {T}: a 2/2 blue Shapeshifter with changeling", maskwood)],
        "Mirror Entity" => vec![ability(
            "{X}: until end of turn, your creatures have base power and toughness X/X and gain all creature types",
            mirror,
        )],
        "Elspeth, Sun's Champion" => walker(&[
            (1, "create three 1/1 white Soldiers", elspeth_plus),
            (-3, "destroy all creatures with power 4 or greater", elspeth_minus3),
            (-7, "emblem: your creatures get +2/+2 and have flying", elspeth_minus7),
        ]),
        "Necropotence" => vec![ability(
            "pay 1 life (any number of times): exile the top card of your library face down; put it into your hand at the beginning of your next end step",
            necro,
        )],
        "Urborg Syphon-Mage" => vec![ability(
            "{2}{B}, {T}, discard a card: each other player loses 2 life; you gain the life lost this way",
            syphon_mage,
        )],
        "Ill-Gotten Inheritance" => {
            vec![ability("{5}{B}, sacrifice it: 4 damage to target opponent; you gain 4 life", inheritance)]
        }
        "Jester's Cap" => vec![ability(
            "{2}, {T}, sacrifice it: search target player's library for three cards and exile them",
            jesters_cap,
        )],
        "Narset, Parter of Veils" => {
            walker(&[(-2, "look at the top four; take a noncreature, nonland card", narset_minus2)])
        }
        _ => return None,
    })
}
