//! Practice mode for the Ur-Dragon deck's combos (`decklists/JD/ur-dragon-equipment.md`): what a person at the seat can
//! do by hand that the bots do through their own code.
//!
//! - **The Helm loop** (Godo, Bandit Warlord or Combat Celebrant wearing Helm of the Host): by hand it is 3 or more
//!   prompts per combat. Once the loop is live, the person may declare "repeat": every additional combat this turn
//!   attacks the chosen opponent with everything that can attack, with no more attack prompts. Blocks, damage, the
//!   triggers and the opponents' answers stay real. In repeat mode Godo's enter trigger skips its search and Combat
//!   Celebrant exerts without asking.
//! - **Choosing what to attach** (Hammer of Nazahn, Sigarda's Aid, Brass Squire).
//! - **Dualcaster Mage + Twinflame**: Twinflame's copy (new targets, hasty token copies), Dualcaster Mage's enter
//!   trigger for the person (a flashed-in one or a token) and the loop shortcut (infinite hasty Dualcaster Mages).
//! - **Peter Parker's Camera** and **Jaxis, the Troublemaker** activated by the person.
//!
//! None of this is reached by a bot: every entry point is for a seat with a person.

use super::abilities::{PermAbility, Why, ability};
use super::{choices, legal, mana};
use crate::engine::stack;
use crate::engine::zones;
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, StackKind, Step, Val};
use crate::sym::{Sym, intern};

pub const GODO: &str = "Godo, Bandit Warlord";
pub const CELEBRANT: &str = "Combat Celebrant";
pub const HELM: &str = "Helm of the Host";
pub const DUALCASTER: &str = "Dualcaster Mage";
pub const TWINFLAME: &str = "Twinflame";

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn pname(g: &Game, p: PlayerId) -> String {
    g.player(p).name.to_string()
}

fn cards_name(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

// ------------------------------------------------------------------ the Helm loop: repeat the attack
fn repeat_key() -> Sym {
    intern("ur_repeat")
}

fn dest_key(q: PlayerId) -> Sym {
    intern(&format!("ur_repeat_at_{}", q.0))
}

/// Has p declared "repeat the attack in every additional combat" this turn?
pub fn repeat_on(g: &Game, p: PlayerId) -> bool {
    g.player(p).flag_turn.get(repeat_key()) == Some(&g.turn_stamp())
}

fn set_dest(g: &mut Game, p: PlayerId, q: PlayerId) {
    let stamp = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(dest_key(q), stamp);
}

/// the opponent the repeated attack goes to: the one chosen, or, once they are out, the next one in turn order
fn repeat_dest(g: &mut Game, p: PlayerId) -> Option<PlayerId> {
    let stamp = g.turn_stamp();
    let opps: Vec<PlayerId> = g.after(p).filter(|&q| g.player(q).alive).collect();
    if let Some(&q) = opps.iter().find(|&&q| g.player(p).flag_turn.get(dest_key(q)) == Some(&stamp)) {
        return Some(q);
    }
    let q = *opps.first()?;
    set_dest(g, p, q);
    crate::glog!(g, "  {} repeats the attack at {} now", pname(g, p), pname(g, q));
    Some(q)
}

/// Is the Helm loop live: p controls Helm of the Host worn by Godo or Combat Celebrant (the Helm makes a token of it
/// every combat, and its first attack adds a combat)?
pub fn loop_live(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&h| {
        live(g, h)
            && g.perm(h).cd.is_some_and(|c| cards_name(g, c) == HELM)
            && g.perm(h).attached.is_some_and(|a| {
                live(g, a) && g.is_creature(a) && g.perm(a).cd.is_some_and(|c| matches!(cards_name(g, c), GODO | CELEBRANT))
            })
    })
}

/// In repeat mode the attack is declared for the person: everything that can attack, at the chosen opponent.
pub fn repeat_attack(g: &mut Game, p: PlayerId, cands: &[PermId]) -> Option<Vec<(PlayerId, Vec<PermId>)>> {
    if !repeat_on(g, p) {
        return None;
    }
    let q = repeat_dest(g, p)?;
    crate::glog!(g, "  {} attacks {} with {} creature(s) (repeat)", pname(g, p), pname(g, q), cands.len());
    Some(vec![(q, cands.to_vec())])
}

/// After the person declared an attack with the loop live, offer to repeat it in every additional combat this turn
pub fn offer_repeat(g: &mut Game, p: PlayerId, groups: &[(PlayerId, Vec<PermId>)]) -> Res {
    if repeat_on(g, p) || groups.len() != 1 || !g.human_walker_atk.is_empty() || !loop_live(g, p) {
        return Ok(());
    }
    let q = groups[0].0;
    let prompt = format!(
        "Helm loop: repeat this in every additional combat this turn, with everything that can attack at {}? \
         (No more attack prompts. Blocks, damage and answers stay real; Godo's searches are skipped and \
         Combat Celebrant exerts.)",
        pname(g, q)
    );
    let labels = ["yes, repeat the attack".to_string(), "no, I'll declare each combat".to_string()];
    if choices::choose_one(g, p, "choose", &prompt, &labels)? == 0 {
        let stamp = g.turn_stamp();
        g.player_mut(p).flag_turn.insert(repeat_key(), stamp);
        set_dest(g, p, q);
        crate::glog!(g, "  {} will repeat the attack at {} in every additional combat", pname(g, p), pname(g, q));
    }
    Ok(())
}

// ------------------------------------------------------------------ attaching: the person chooses
/// "you may attach it to target creature you control" (Hammer of Nazahn, Sigarda's Aid): the creature the person
/// picks, or None for no attach
pub fn pick_attach_target(g: &Game, p: PlayerId, source: &str, e: PermId) -> Res<Option<PermId>> {
    let cre: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m) && m != e).collect();
    if cre.is_empty() {
        return Ok(None);
    }
    let labels: Vec<String> = cre.iter().map(|&m| legal::describe_target(g, p, super::Target::Perm(m))).collect();
    let prompt = format!("{source}: attach {} to which creature?", g.perm(e).name);
    Ok(choices::choose(g, p, "target", &prompt, &labels, Some("don't attach"), None)?.map(|k| cre[k]))
}

// ------------------------------------------------------------------ Brass Squire
fn squire_go(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    // {T}: attach target Equipment you control to target creature you control (instant speed)
    if g.perm(m).tapped {
        return Ok(Some("Brass Squire is tapped.".into()));
    }
    if g.perm(m).sick {
        return Ok(Some("Brass Squire has summoning sickness (it came under your control this turn).".into()));
    }
    let eqs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&e| live(g, e) && g.perm(e).cd.is_some_and(|c| g.db.get(c).has_subtype("equipment")))
        .collect();
    if eqs.is_empty() {
        return Ok(Some("You control no Equipment to attach.".into()));
    }
    let labels: Vec<String> = eqs
        .iter()
        .map(|&e| {
            let on = g.perm(e).attached.filter(|&t| live(g, t)).map_or(String::new(), |t| format!(" (on {})", g.perm(t).name));
            format!("{}{on}", g.perm(e).name)
        })
        .collect();
    let Some(i) = choices::choose(g, p, "target", "Brass Squire: attach which Equipment?", &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    let e = eqs[i];
    let cre: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&c| live(g, c) && g.is_creature(c) && g.perm(e).attached != Some(c))
        .collect();
    if cre.is_empty() {
        return Ok(Some(format!("{} has no other creature of yours to go on.", g.perm(e).name)));
    }
    let labels: Vec<String> = cre.iter().map(|&c| legal::describe_target(g, p, super::Target::Perm(c))).collect();
    let prompt = format!("Brass Squire: attach {} to which creature?", g.perm(e).name);
    let Some(j) = choices::choose(g, p, "target", &prompt, &labels, Some("cancel"), None)? else { return Ok(None) };
    let to = cre[j];
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "  {} activates Brass Squire: {} to {}", pname(g, p), g.perm(e).name, g.perm(to).name);
    if stack::ability_window(g, p, Some(m), "attach Equipment", Some(5.0), Some(to))? && live(g, e) && live(g, to) {
        crate::impls::cloud::attach(g, e, to);
    }
    Ok(None)
}

// ------------------------------------------------------------------ Peter Parker's Camera
fn camera_go(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    let x = g.perm(m);
    if x.tapped {
        return Ok(Some("Peter Parker's Camera is tapped.".into()));
    }
    if x.data.int(DataKey::Counters) <= 0 {
        return Ok(Some("Peter Parker's Camera has no film counters left.".into()));
    }
    let items: Vec<(u32, String)> = g
        .stack
        .iter()
        .rev()
        .filter(|it| it.controller == p && !it.countered && stack::copyable(it))
        .map(|it| (it.id, it.name.clone()))
        .collect();
    if items.is_empty() {
        return Ok(Some(
            "None of your activated or triggered abilities that can be copied is on the stack. Use the Camera while one is waiting to resolve.".into(),
        ));
    }
    if let Some(why) = mana::cost_problem(g, p, 2, "") {
        return Ok(Some(format!("Can't activate Peter Parker's Camera. {why}")));
    }
    let labels: Vec<String> = items.iter().map(|x| x.1.clone()).collect();
    let Some(k) = choices::choose(
        g,
        p,
        "target",
        "Peter Parker's Camera: copy which ability?",
        &labels,
        Some("cancel"),
        None,
    )?
    else {
        return Ok(None);
    };
    mana::pay_from_pool(g, p, 2, "");
    g.perm_mut(m).tapped = true;
    let n = g.perm(m).data.int(DataKey::Counters);
    g.perm_mut(m).data.set(DataKey::Counters, Val::Int(n - 1));
    crate::glog!(g, "  {} activates Peter Parker's Camera: copies {}", pname(g, p), items[k].1);
    // the Camera's own ability is on the stack above it: if that is countered, nothing is copied
    if stack::ability_window(g, p, Some(m), "copy target activated or triggered ability", Some(3.0), None)? {
        stack::copy_trigger(g, p, items[k].0)?;
    }
    Ok(None)
}

/// Does the person get priority on their own ability (item) so that the Camera can copy it?
pub fn camera_window(g: &Game, q: PlayerId, it: &crate::state::StackItem) -> bool {
    it.controller == q
        && it.kind != StackKind::Spell
        && it.imp >= 5.0
        && stack::copyable(it)
        && stack::camera_ready(g, q)
        && !repeat_on(g, q)
}

// ------------------------------------------------------------------ Jaxis, the Troublemaker
fn jaxis_go(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    // {R}, {T}, discard a card: a token that's a copy of another target creature you control; haste; draw a card when
    // it dies; sacrificed at the beginning of the next end step. Sorcery speed
    if let Some(why) = legal::sorcery_timing(g, p) {
        return Ok(Some(why.replace("do that", "activate Jaxis")));
    }
    if g.perm(m).tapped {
        return Ok(Some("Jaxis, the Troublemaker is tapped.".into()));
    }
    if g.perm(m).sick {
        return Ok(Some("Jaxis has summoning sickness (it came under your control this turn).".into()));
    }
    if g.player(p).hand.is_empty() {
        return Ok(Some("You have no card to discard.".into()));
    }
    if let Some(why) = mana::cost_problem(g, p, 0, "R") {
        return Ok(Some(format!("Can't activate Jaxis. {why}")));
    }
    let cre: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&c| live(g, c) && g.is_creature(c) && c != m).collect();
    if cre.is_empty() {
        return Ok(Some("You control no other creature to copy.".into()));
    }
    let labels: Vec<String> = cre.iter().map(|&c| legal::describe_target(g, p, super::Target::Perm(c))).collect();
    let Some(k) = choices::choose(g, p, "target", "Jaxis: copy which creature?", &labels, Some("cancel"), None)? else {
        return Ok(None); // targets first, then the costs
    };
    let t = cre[k];
    let hand = g.player(p).hand.clone();
    let d = choices::pick_cards(g, p, &hand, 1, "Jaxis: discard a card", "choose")?[0];
    mana::pay_from_pool(g, p, 0, "R");
    g.perm_mut(m).tapped = true;
    zones::discard_cards(g, p, &[d])?;
    crate::glog!(g, "  {} activates Jaxis: copy {} (discards {})", pname(g, p), g.perm(t).name, cards_name(g, d));
    let label = format!("copy {}", g.perm(t).name);
    if !stack::ability_window(g, p, Some(m), &label, None, Some(t))? || !live(g, t) {
        return Ok(None);
    }
    let cd = g.perm(t).cd.unwrap();
    if let Some(tok) = zones::enter_token_copy(g, p, cd)? {
        let y = g.perm_mut(tok);
        y.sick = false; // haste
        y.data.set(DataKey::JaxisCopy, Val::Bool(true)); // drawn for and sacrificed at the end step (Jaxis's own hook)
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

/// the person's abilities of Brass Squire, Peter Parker's Camera and Jaxis (None: not one of these cards)
pub fn abilities(nm: &str) -> Option<Vec<PermAbility>> {
    Some(match nm {
        "Brass Squire" => vec![ability("{T}: attach target Equipment you control to target creature you control", squire_go)],
        "Peter Parker's Camera" => vec![ability(
            "{2}, {T}, remove a film counter: copy target activated or triggered ability you control",
            camera_go,
        )],
        "Jaxis, the Troublemaker" => vec![ability(
            "{R}, {T}, discard a card: a hasty token copy of another creature you control (sorcery speed)",
            jaxis_go,
        )],
        _ => return None,
    })
}

// ------------------------------------------------------------------ Twinflame and Dualcaster Mage
fn my_creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect()
}

/// Twinflame's effect: a token copy of each creature, with haste, exiled at the beginning of the next end step. When
/// the tokens are made by a copy of the spell, `spell` is the original still on the stack: a Dualcaster Mage token's
/// enter trigger may copy it.
pub fn twinflame_tokens(g: &mut Game, p: PlayerId, targets: &[PermId], spell: Option<(CardId, &Ctx)>) -> Res {
    for &m in targets {
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
        let Some(cp) = cp else { continue };
        let x = g.perm_mut(cp);
        x.sick = false; // haste; exiled at the end step
        x.temp = true;
        if g.perm(cp).on_bf && g.perm(cp).cd.is_some_and(|c| cards_name(g, c) == DUALCASTER) {
            dualcaster_etb(g, p, cp, spell)?;
        }
    }
    Ok(())
}

/// A copy of Twinflame resolves for the person: it has as many targets as the spell had, and they may be changed
pub fn twinflame_copy(g: &mut Game, p: PlayerId, spell: CardId, ctx: &Ctx) -> Res {
    let slots = ctx.perms.len().max(1);
    let mut chosen: Vec<PermId> = vec![];
    for i in 0..slots {
        let cands: Vec<PermId> = my_creatures(g, p).into_iter().filter(|m| !chosen.contains(m)).collect();
        if cands.is_empty() {
            break;
        }
        let keep = ctx.perms.get(i).copied().filter(|m| cands.contains(m));
        if cands.len() == 1 {
            chosen.push(cands[0]);
            continue;
        }
        let labels: Vec<String> = cands.iter().map(|&m| legal::describe_target(g, p, super::Target::Perm(m))).collect();
        let cancel = keep.map(|k| format!("keep the same target ({})", g.perm(k).name));
        let prompt = format!("The copy of Twinflame: target creature #{} ({} in all)", i + 1, slots);
        match choices::choose(g, p, "target", &prompt, &labels, cancel.as_deref(), None)? {
            Some(k) => chosen.push(cands[k]),
            None => chosen.extend(keep),
        }
    }
    let names: Vec<String> = chosen.iter().map(|&m| g.perm(m).name.to_string()).collect();
    crate::glog!(g, "    the copy of Twinflame makes hasty token copies of {}", names.join(", "));
    twinflame_tokens(g, p, &chosen, Some((spell, ctx)))
}

fn bf_last_named(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().rev().copied().find(|&m| live(g, m) && g.perm(m).cd.is_some_and(|c| cards_name(g, c) == name))
}

/// Dualcaster Mage entered (flashed in, or a token): "When this creature enters, copy target instant or sorcery
/// spell. You may choose new targets for the copy." `spell` is the spell it responds to, on the stack with its
/// targets; with no spell there is nothing to copy. The person may copy it (once, by hand) or, with Twinflame under
/// their own control before combat, run the loop.
pub fn dualcaster_etb(g: &mut Game, p: PlayerId, dm: PermId, spell: Option<(CardId, &Ctx)>) -> Res {
    let Some((sc, sctx)) = spell else { return Ok(()) };
    let d = g.db.get(sc);
    if !(d.instant || d.sorcery) || !live(g, dm) || !g.stack.iter().any(|it| it.kind == StackKind::Spell && it.card == Some(sc))
    {
        return Ok(());
    }
    let sname = d.name.to_string();
    if !stack::trigger_window(g, p, Some(dm), "copy target instant or sorcery spell", Some(6.0))? || !live(g, dm) {
        return Ok(());
    }
    let can_loop = sname == TWINFLAME
        && g.active == Some(p)
        && g.step == Step::Main1
        && g.stack.iter().any(|it| it.kind == StackKind::Spell && it.card == Some(sc) && it.controller == p);
    let mut labels = vec![format!("copy {sname} (you may choose new targets)")];
    if can_loop {
        labels.push(
            "the loop: copy it again and again, aimed at a Dualcaster Mage each time (infinitely many hasty Dualcaster Mages to attack with)"
                .to_string(),
        );
    }
    let prompt = format!("Dualcaster Mage: copy {sname}?");
    let Some(k) = choices::choose(g, p, "choose", &prompt, &labels, Some("don't copy"), None)? else { return Ok(()) };
    if k == 0 {
        crate::glog!(g, "    Dualcaster Mage copies {sname}");
        return crate::engine::cast::copy_spell(g, p, sc, Some(sctx), false);
    }
    dualcaster_loop(g, p, dm)
}

/// the number of hasty Dualcaster Mages that kill the table through any blockers (2 damage each)
fn loop_tokens(g: &Game, p: PlayerId) -> usize {
    let mut n = 5;
    for q in g.opps(p) {
        let blockers = g
            .player(q)
            .perms
            .iter()
            .filter(|&&m| live(g, m) && g.is_creature(m) && !g.perm(m).tapped)
            .count();
        n += ((g.player(q).life.max(0) as usize) + 1) / 2 + blockers;
    }
    n.clamp(10, 200)
}

/// Dualcaster Mage + Twinflame, as a table shortcut: the opponents get a window to answer a key piece, then the
/// infinite copies are played out as enough hasty Dualcaster Mage tokens to kill the table (exiled at the end step),
/// to attack with: blocks and damage are real.
fn dualcaster_loop(g: &mut Game, p: PlayerId, dm: PermId) -> Res {
    crate::glog!(g, "  {} goes for the Dualcaster Mage + Twinflame loop", pname(g, p));
    if crate::ai::combo_interrupted(g, p, "urdragon", &[dm])? {
        crate::glog!(g, "    ...the loop is stopped");
        return Ok(());
    }
    let n = loop_tokens(g, p);
    let cd = g.perm(dm).cd.unwrap();
    let mut made = 0;
    for _ in 0..n {
        let Some(t) = zones::enter_token_copy(g, p, cd)? else { break };
        let x = g.perm_mut(t);
        x.sick = false;
        x.temp = true;
        made += 1;
    }
    crate::glog!(
        g,
        "    {made} hasty Dualcaster Mages (as good as infinite; exiled at the end step): attack with them"
    );
    crate::engine::life::check_state(g)
}

/// The person casts Dualcaster Mage in response to `spell` (on the stack, controlled by `caster`): the creature spell
/// resolves, then its enter trigger. None, or why not
pub fn dualcaster_flash(g: &mut Game, p: PlayerId, c: CardId, spell: CardId, ctx: Option<Ctx>) -> Res<Why> {
    let sd = g.db.get(spell);
    if !(sd.instant || sd.sorcery) {
        return Ok(Some(format!("Dualcaster Mage copies an instant or sorcery spell; {} is not one.", sd.name)));
    }
    if let Some(why) = mana::cost_problem(g, p, 1, "RR") {
        return Ok(Some(format!("Can't cast Dualcaster Mage. {why}")));
    }
    mana::pay_from_pool(g, p, 1, "RR");
    crate::glog!(g, "  {} flashes in Dualcaster Mage to copy {}", pname(g, p), cards_name(g, spell));
    if crate::engine::cast::cast_card(g, p, c, "hand", Default::default())?
        && !g.over
        && let Some(dm) = bf_last_named(g, p, DUALCASTER)
    {
        let ctx = ctx.unwrap_or_default();
        dualcaster_etb(g, p, dm, Some((spell, &ctx)))?;
    }
    Ok(None)
}
