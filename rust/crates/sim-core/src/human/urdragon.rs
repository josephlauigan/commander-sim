//! Practice mode for the Ur-Dragon deck's cards (`decklists/JD/ur-dragon-dragon-copies.md`) and the table shortcuts the
//! Sauron deck shares with it: what a person at the seat can do by hand that the bots do through their own code.
//!
//! The Ur-Dragon deck itself (rebuilt 11/03):
//! - **Its three combo loops** (Astral Dragon + Cursed Mirror; Miirym + Astral Dragon + Doubling Season; Garth One-Eye +
//!   Twinflame + Spark Double + Doubling Season): "start the loop" on each piece on the battlefield while the
//!   catalogue's `ready` holds (`impls/t45_urdragon_combos.rs`). It plays the catalogue's own steps (the cast, Miirym's
//!   trigger, Twinflame, Regrowth, with the opponents' windows and the pieces-must-survive check), then makes the
//!   tokens the loop makes: hasty Astral Dragon copies of the Mirror or hasty Garths, as many rounds as the person
//!   picks or enough to kill the table (the token cap bounds both), or 3/3 flying Dragon tokens for Miirym's line.
//!   Cancelling leaves everything to play by hand.
//! - **Twinflame and Dualcaster Mage**: Twinflame's copy (new targets, hasty token copies), Dualcaster Mage's enter
//!   trigger for the person (a flashed-in one or a token) and the loop shortcut (infinite hasty Dualcaster Mages).
//! - **Choosing what to attach** (Hammer of Nazahn, Sigarda's Aid, Brass Squire), **Peter Parker's Camera** and
//!   **Jaxis, the Troublemaker** activated by the person.
//!
//! For the Sauron deck's Godo, Bandit Warlord, Combat Celebrant and Helm of the Host (they left the Ur-Dragon list in
//! the 11/03 rebuild; the code and its tests live here):
//! - **The Helm loop** (Godo or Combat Celebrant wearing Helm of the Host): by hand it is 3 or more prompts per
//!   combat. Once the loop is live, the person may declare "repeat": every additional combat this turn attacks the
//!   chosen opponent with everything that can attack, with no more attack prompts. Blocks, damage, the triggers and
//!   the opponents' answers stay real. In repeat mode Godo's enter trigger skips its search and Combat Celebrant
//!   exerts without asking.
//!
//! None of this is reached by a bot: every entry point is for a seat with a person.

use super::abilities::{PermAbility, Why, ability};
use super::{choices, legal, mana};
use crate::cards::Types;
use crate::engine::stack;
use crate::engine::zones;
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::combos::{self, Combo};
use crate::state::{CopyExc, Ctx, DataKey, Game, StackKind, Step, Val};
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
                live(g, a)
                    && g.is_creature(a)
                    && g.perm(a).cd.is_some_and(|c| matches!(cards_name(g, c), GODO | CELEBRANT))
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
    let stamp = g.turn_stamp();
    let declined = intern("ur_repeat_declined");
    if repeat_on(g, p)
        || g.player(p).flag_turn.get(declined) == Some(&stamp)
        || groups.len() != 1
        || !g.human_walker_atk.is_empty()
        || !loop_live(g, p)
    {
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
    if choices::choose_one(g, p, "choose", &prompt, &labels)? != 0 {
        g.player_mut(p).flag_turn.insert(declined, stamp); // asked once a turn
    } else {
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
            let on = g
                .perm(e)
                .attached
                .filter(|&t| live(g, t))
                .map_or(String::new(), |t| format!(" (on {})", g.perm(t).name));
            format!("{}{on}", g.perm(e).name)
        })
        .collect();
    let Some(i) =
        choices::choose(g, p, "target", "Brass Squire: attach which Equipment?", &labels, Some("cancel"), None)?
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
    let Some(k) =
        choices::choose(g, p, "target", "Peter Parker's Camera: copy which ability?", &labels, Some("cancel"), None)?
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
        "Brass Squire" => {
            vec![ability("{T}: attach target Equipment you control to target creature you control", squire_go)]
        }
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
    g.player(p)
        .perms
        .iter()
        .rev()
        .copied()
        .find(|&m| live(g, m) && g.perm(m).cd.is_some_and(|c| cards_name(g, c) == name))
}

/// Dualcaster Mage entered (flashed in, or a token): "When this creature enters, copy target instant or sorcery
/// spell. You may choose new targets for the copy." `spell` is the spell it responds to, on the stack with its
/// targets; with no spell there is nothing to copy. The person may copy it (once, by hand) or, with Twinflame under
/// their own control before combat, run the loop.
pub fn dualcaster_etb(g: &mut Game, p: PlayerId, dm: PermId, spell: Option<(CardId, &Ctx)>) -> Res {
    let Some((sc, sctx)) = spell else { return Ok(()) };
    let d = g.db.get(sc);
    if !(d.instant || d.sorcery)
        || !live(g, dm)
        || !g.stack.iter().any(|it| it.kind == StackKind::Spell && it.card == Some(sc))
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
        let blockers =
            g.player(q).perms.iter().filter(|&&m| live(g, m) && g.is_creature(m) && !g.perm(m).tapped).count();
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

// ------------------------------------------------------------------ the Ur-Dragon deck's three loops
// The catalogue (`impls/t45_urdragon_combos.rs`) plays these for a bot: the steps with their answer windows, then a
// finish. The person gets the same steps (the engine's own casts, Miirym's trigger, the opponents' windows and the
// pieces-must-survive check) and a finish that makes the tokens the loop makes, as many as the person asks for. It is an
// ability on every piece on the battlefield, listed only while the catalogue's `ready` holds (the pieces out, the
// timing, and the whole line's mana in the pool); the person can always cancel and play it by hand.

const ASTRAL: &str = "Astral Dragon";
const MIRROR_CARD: &str = "Cursed Mirror";
const GARTH: &str = "Garth One-Eye";
const LOOP_MIRROR: &str = "Astral Dragon + Cursed Mirror";
const LOOP_MIIRYM: &str = "Miirym + Astral Dragon + Doubling Season";
const LOOP_GARTH: &str = "Garth One-Eye + Twinflame + Spark Double + Doubling Season";

/// the loops of the catalogue that are live for p and have the permanent m as a piece
fn live_loops(g: &Game, p: PlayerId, m: PermId) -> Vec<&'static str> {
    let Some(cd) = g.perm(m).cd else { return vec![] };
    if !live(g, m) {
        return vec![];
    }
    let nm = cards_name(g, cd);
    [LOOP_MIRROR, LOOP_MIIRYM, LOOP_GARTH]
        .into_iter()
        .filter(|&part| {
            let Some(i) = combos::combo_index(part) else { return false };
            combos::all()[i].groups.iter().any(|grp| grp.contains(&nm)) && combos::ready_to_go(g, p, i)
        })
        .collect()
}

/// "start the loop" on each piece of a live Ur-Dragon loop (Astral Dragon + Cursed Mirror; Miirym + Astral Dragon +
/// Doubling Season; Garth One-Eye + Twinflame + Spark Double + Doubling Season)
pub fn loop_abilities(g: &Game, p: PlayerId, m: PermId) -> Vec<PermAbility> {
    live_loops(g, p, m)
        .into_iter()
        .map(|part| {
            let what = match part {
                LOOP_MIRROR | LOOP_MIIRYM => "cast Astral Dragon",
                _ => "cast Twinflame",
            };
            ability(format!("start the loop: {part} ({what} and play the loop out)"), move |g: &mut Game, p, _m| {
                go_loop(g, p, part)
            })
        })
        .collect()
}

fn round_labels(kill: &str) -> Vec<String> {
    let mut v = vec![kill.to_string()];
    v.extend(["once around", "twice around", "three times around", "five times around"].map(String::from));
    v
}

/// hasty creatures it takes to kill every opponent through their untapped blockers, at `pow` damage each
fn attackers_needed(g: &Game, p: PlayerId, pow: i32) -> usize {
    let pow = pow.max(1) as usize;
    let mut n = 3;
    for q in g.opps(p).filter(|&q| g.player(q).alive) {
        let blockers = g.player(q).perms.iter().filter(|&&m| live(g, m) && g.is_creature(m) && !g.perm(m).tapped).count();
        n += (g.player(q).life.max(0) as usize).div_ceil(pow) + blockers;
    }
    n.clamp(5, crate::engine::zones::TOKEN_CAP)
}

/// p's creatures that are named `name`, can attack now (untapped, not summoning sick)
fn ready_attackers(g: &Game, p: PlayerId, name: &str) -> usize {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| {
            let x = g.perm(m);
            live(g, m) && g.is_creature(m) && !x.tapped && !x.sick && x.cd.is_some_and(|c| cards_name(g, c) == name)
        })
        .count()
}

fn go_loop(g: &mut Game, p: PlayerId, part: &'static str) -> Res<Why> {
    let Some(i) = combos::combo_index(part) else { return Ok(Some("That loop is not in the catalogue.".into())) };
    if !combos::ready_to_go(g, p, i) {
        return Ok(Some(format!(
            "The loop ({part}) is not live: it needs the pieces out, the card in hand, the whole line's mana in your pool \
             and (Astral Dragon + Cursed Mirror, Garth) a combat still to come."
        )));
    }
    let (prompt, labels, finish): (String, Vec<String>, [fn(&mut Game, PlayerId) -> Res; 5]) = match part {
        LOOP_MIRROR => (
            "Astral Dragon + Cursed Mirror: cast Astral Dragon (the opponents may answer it, its trigger and the pieces). \
             Each hasty Dragon copy of the Mirror makes two more. How far?"
                .into(),
            round_labels("until there are enough hasty Dragons to kill the table (up to the token cap)"),
            [
                mirror_finish::<0>,
                mirror_finish::<1>,
                mirror_finish::<2>,
                mirror_finish::<3>,
                mirror_finish::<5>,
            ],
        ),
        LOOP_GARTH => (
            "Garth + Twinflame + Spark Double + Doubling Season: cast Twinflame on Spark Double's Garth, Regrowth brings \
             it back (the opponents may answer each). Then Twinflame again on every Garth. How far?"
                .into(),
            round_labels("until there are enough hasty Garths to kill the table (up to the token cap)"),
            [garth_finish::<0>, garth_finish::<1>, garth_finish::<2>, garth_finish::<3>, garth_finish::<5>],
        ),
        _ => (
            "Miirym + Astral Dragon + Doubling Season: cast Astral Dragon (the opponents may answer it, its trigger and \
             the pieces). Miirym's copy and every Project Image add Doubling Seasons: the board grows without bound. How \
             many 3/3 flying Dragon tokens (summoning sick; the next combat is the kill)?"
                .into(),
            vec![
                "fill the battlefield (up to the token cap)".to_string(),
                "about a hundred".to_string(),
                "about fifty".to_string(),
                "about ten".to_string(),
                "about five".to_string(),
            ],
            [flood::<0>, flood::<100>, flood::<50>, flood::<10>, flood::<5>],
        ),
    };
    let Some(k) = choices::choose(g, p, "choose", &prompt, &labels, Some("cancel, I'll play it by hand"), None)? else {
        return Ok(None);
    };
    let cmb = combos::all()[i];
    let person = Combo {
        name: cmb.name,
        groups: cmb.groups,
        ready: cmb.ready,
        mana: cmb.mana,
        finish: Some(finish[k]),
        xcost: cmb.xcost,
        steps: cmb.steps,
    };
    crate::glog!(g, "  {} goes for the {part} loop", pname(g, p));
    combos::run_steps(g, p, &person)?;
    crate::engine::life::check_state(g)?;
    Ok(None)
}

/// Astral Dragon + Cursed Mirror: each Astral Dragon's Project Image makes two Cursed Mirror tokens, each entering as a
/// copy of Astral Dragon with haste (an artifact too) whose own Project Image makes two more. `R` rounds, or (0) until
/// the hasty Dragons could kill the table. The Mirror copies last until end of turn, then the tokens are Mirrors again.
fn mirror_finish<const R: usize>(g: &mut Game, p: PlayerId) -> Res {
    let (Some(astral), Some(mirror)) = (g.db.id(ASTRAL), g.db.id(MIRROR_CARD)) else { return Ok(()) };
    let exc = CopyExc { add_types: Types::ARTIFACT, haste: true, revert_to: Some(mirror), ..CopyExc::default() };
    let need = |g: &Game| if R == 0 { attackers_needed(g, p, 3) } else { usize::MAX };
    // the Project Image of the Dragon just cast stands in for the first round (its step only checked the pieces)
    let (mut triggers, mut rounds) = (1, 0);
    g.combo_spell = true; // the Dragons' own Project Image is the loop itself: no per-token prompt
    let r = (|| -> Res {
        while !g.over && triggers > 0 && (R == 0 || rounds < R) && ready_attackers(g, p, ASTRAL) < need(g) {
            let per = crate::engine::repl::tokens(g, p, 2) as usize;
            let mut made = 0;
            'round: for _ in 0..triggers {
                for _ in 0..per {
                    let Some(t) = zones::enter_token_copy_with(g, p, astral, Some(exc))? else { break 'round };
                    if g.perm(t).on_bf {
                        g.perm_mut(t).sick = false; // haste
                        made += 1;
                    }
                }
            }
            rounds += 1;
            triggers = made;
            crate::glog!(g, "    round {rounds}: {made} more hasty Astral Dragons (Cursed Mirror copies)");
        }
        Ok(())
    })();
    g.combo_spell = false;
    r?;
    crate::glog!(
        g,
        "    {} hasty Astral Dragons can attack (the Mirror tokens go back to being Mirrors next turn)",
        ready_attackers(g, p, ASTRAL)
    );
    crate::engine::life::check_state(g)
}

/// Garth + Twinflame + Spark Double + Doubling Season, after the first Twinflame and Regrowth: Twinflame again on every
/// Garth (strive paid for by the Black Lotuses the Garths make), a hasty token Garth for each, twice as many with
/// Doubling Season; the tokens are exiled at the end step. `R` rounds, or (0) until the hasty Garths could kill the table.
fn garth_finish<const R: usize>(g: &mut Game, p: PlayerId) -> Res {
    let garths = |g: &Game| -> Vec<PermId> {
        g.player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| live(g, m) && g.is_creature(m) && g.perm(m).cd.is_some_and(|c| cards_name(g, c) == GARTH))
            .collect()
    };
    let Some(&first) = garths(g).first() else { return Ok(()) };
    let pow = crate::engine::values::epow(g, first);
    let exc = CopyExc { haste: true, not_legendary: true, ..CopyExc::default() };
    let mut rounds = 0;
    let need = if R == 0 { attackers_needed(g, p, pow) } else { usize::MAX };
    while !g.over && (R == 0 || rounds < R) && ready_attackers(g, p, GARTH) < need {
        let targets = garths(g);
        let per = crate::engine::repl::tokens(g, p, 1) as usize;
        let mut made = 0;
        'round: for &t in &targets {
            let Some(cd) = g.perm(t).cd else { continue };
            for _ in 0..per {
                let Some(tok) = zones::enter_token_copy_with(g, p, cd, Some(exc))? else { break 'round };
                if g.perm(tok).on_bf {
                    let x = g.perm_mut(tok);
                    x.sick = false; // haste
                    x.temp = true; // exiled at the beginning of the next end step
                    made += 1;
                }
            }
        }
        rounds += 1;
        crate::glog!(g, "    round {rounds}: Twinflame on {} Garth(s): {made} more hasty token Garths", targets.len());
        if made == 0 {
            break;
        }
    }
    crate::glog!(g, "    {} hasty Garths can attack (exiled at the end step)", ready_attackers(g, p, GARTH));
    crate::engine::life::check_state(g)
}

/// Miirym + Astral Dragon + Doubling Season: about `N` 3/3 flying Dragon tokens (0: up to the token cap), the catalogue's
/// own finish with a number
fn flood<const N: u32>(g: &mut Game, p: PlayerId) -> Res {
    let mult = crate::engine::repl::tokens(g, p, 1).max(1);
    let n = if N == 0 { crate::engine::zones::TOKEN_CAP as u32 } else { N.div_ceil(mult) };
    crate::impls::t45_urdragon_combos::dragon_flood_n(g, p, n)?;
    crate::engine::life::check_state(g)
}
