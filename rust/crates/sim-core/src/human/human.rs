//! The human seat's turn: priority in the main phases, and the actions a player takes (tap for mana, play a land,
//! cast a spell, pass) (Python's `play/human.py`). Each action is checked by `legal` first; an illegal one is refused
//! with the reason and the player keeps priority.
//!
//! Actions are JSON objects, the same from the text client and the browser:
//!   {"do": "tap", "source": id, "colour": "B"}   a mana source from `mana::sources`
//!   {"do": "land", "card": i}                    the i-th card in hand as your land drop
//!   {"do": "cast", "card": i}                    the i-th card in hand
//!   {"do": "cast", "zone": "cmd"}                your commander from the command zone
//!   {"do": "cast", "zone": "cmd", "cmd": 1}      with Partner: the second commander (0, the default, is the first
//!                                                in the zone; "cmd" may also be the commander's name)
//!   {"do": "use", "perm": i}                     an ability of your i-th permanent (Equip, loyalty, ...)
//!   {"do": "pass"}
//! Follow-up questions (which target, which ability) come back as 'target' / 'choose' requests.
//!
//! The `view` the page wants in `data` is added by the `Human` itself (see `human/mod.rs`).

use super::mana::Spending;
use super::{Target, abilities, cards, choices, human_of, legal, mana, specs};
use crate::control::{Answer, Human, Request};
use crate::engine::stack::{self, ABILITY_ANSWERS};
use crate::engine::values::epow;
use crate::engine::zones;
use crate::engine::{self, cast, turn};
use crate::flow::Res;
use crate::flow::Stop;
use crate::hooks::{Action, Event};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{Ctx, Game, StackItem, StackKind, Step};
use crate::sym::intern;
use crate::tag::Tag;
use serde_json::{Value, json};
use std::sync::Arc;

fn name_of(g: &Game, p: PlayerId) -> String {
    g.player(p).name.to_string()
}

fn card_name(g: &Game, c: CardId) -> String {
    g.db.get(c).name.to_string()
}

fn perm_name(g: &Game, m: PermId) -> String {
    g.perm(m).name.to_string()
}

fn tell_invalid(h: &Arc<dyn Human>, p: PlayerId, why: &str) {
    h.tell(p, intern("invalid"), why);
}

fn step_name(g: &Game) -> &'static str {
    match g.step {
        Step::Main1 => "Main phase 1",
        Step::Main2 => "Main phase 2",
        _ => "Your turn",
    }
}

/// the action object of an answer (anything else: an empty one, which is not a known action)
fn action_of(ans: Answer) -> Value {
    match ans {
        Answer::Action(v) if v.is_object() => v,
        _ => json!({}),
    }
}

fn do_of(act: &Value) -> &str {
    act.get("do").and_then(Value::as_str).unwrap_or("")
}

/// `act[key]` as an index (Python's `isinstance(i, int)`, so not a bool or a float)
fn index_of(act: &Value, key: &str) -> Option<usize> {
    act.get(key).and_then(Value::as_u64).map(|x| x as usize)
}

/// p's main phase, played by the person: act until they pass (Python's `human_main`)
pub fn human_main(g: &mut Game, p: PlayerId, _post: bool) -> Res {
    let Some(h) = human_of(g, p) else { return Ok(()) };
    while !g.over && g.player(p).alive {
        let prompt = format!("{}: you have priority", step_name(g));
        let req = Request { kind: intern("priority"), prompt, choices: vec![], data: json!({}) };
        // a closed or rewound session answers Cancel: unwind the game (Python's ask raises Cancelled); treating it as an
        // unknown action looped forever, telling "invalid" each time
        let ans = h.ask(g, p, req);
        if ans == Answer::Cancel {
            return Err(Stop::Cancelled);
        }
        let act = action_of(ans);
        if do_of(&act) == "pass" {
            return Ok(());
        }
        if let Some(why) = apply(g, p, &act)? {
            tell_invalid(&h, p, &why);
        }
    }
    Ok(())
}

/// the stack for the page, top first: each item's card, controller and what it targets (Python's `stack_view`)
pub fn stack_view(g: &Game) -> Vec<Value> {
    let mut out = vec![];
    for it in g.stack.iter().rev() {
        let name = match it.card {
            Some(c) => card_name(g, c),
            None => it.name.clone(),
        };
        let mut d = json!({
            "name": name,
            "controller": name_of(g, it.controller),
            "key": &*g.player(it.controller).key,
            "kind": match it.kind { StackKind::Spell => "spell", StackKind::Ability => "ability", StackKind::Trigger => "trigger" },
        });
        if it.kind != StackKind::Spell {
            d["text"] = json!(it.name);
        }
        if let Some(t) = it.ctx.counter {
            if let Some(tc) = g.stack.iter().find(|x| x.id == t).and_then(|x| x.card) {
                d["target"] = json!(card_name(g, tc));
            }
        } else if let Some(t) = it.ctx.target {
            d["target"] = json!(perm_name(g, t));
        }
        out.push(d);
    }
    out
}

/// q's auto-pass setting: 'respond' (asked only when you could do something, plus opponents' spells, attacks on you
/// and the end of each other turn), 'stack' (every spell, ability and trigger too) or 'all' (every step)
pub fn autopass(g: &Game, q: PlayerId) -> &'static str {
    match human_of(g, q).and_then(|h| h.autopass()) {
        Some(m @ ("respond" | "stack" | "all")) => m,
        _ => "respond",
    }
}

/// q (the person) gets priority with something on the stack. Your own spell: only when you hold a card that copies
/// it, or with auto-pass off; anyone else's: always. An ability or trigger: when you could do something about it, or
/// always with auto-pass at 'stack' or 'all'
pub fn stack_priority(g: &mut Game, q: PlayerId) -> Res {
    let mode = autopass(g, q);
    let Some(top) = g.stack.last().cloned() else { return Ok(()) };
    if top.kind != StackKind::Spell {
        if top.controller == q && mode != "all" {
            return Ok(());
        }
        if mode == "respond" && !can_respond(g, q) {
            return Ok(());
        }
        let who = if top.controller == q {
            if top.kind == StackKind::Ability { "You activate".to_string() } else { "Your trigger:".to_string() }
        } else {
            let n = name_of(g, top.controller);
            if top.kind == StackKind::Ability { format!("{n} activates") } else { format!("{n} has a trigger:") }
        };
        return respond(g, q, &format!("{who} {}", top.name), top.card, Some(top.controller));
    }
    if top.controller == q && mode != "all" && !copy_window(g, q, top.card) {
        return Ok(());
    }
    let who =
        if top.controller == q { "You cast".to_string() } else { format!("{} casts", name_of(g, top.controller)) };
    let tc = top.card.map(|c| card_name(g, c)).unwrap_or_default();
    let what = match top.ctx.counter.and_then(|t| g.stack.iter().find(|x| x.id == t)).and_then(|x| x.card) {
        Some(t) => format!("{tc} (countering {})", card_name(g, t)),
        None => tc,
    };
    respond(g, q, &format!("{who} {what}"), top.card, Some(top.controller))
}

/// practice mode: the caster gets priority on their own instant or sorcery when they hold a card that copies it
/// (Python's `engine._copy_window`)
pub fn copy_window(g: &Game, p: PlayerId, c: Option<CardId>) -> bool {
    let Some(c) = c else { return false };
    let d = g.db.get(c);
    (d.instant || d.sorcery)
        && super::is_human(g, p)
        && g.player(p).hand.iter().any(|&x| matches!(&*g.db.get(x).name, "Return the Favor" | "Dualcaster Mage"))
}

/// q (the person) has priority in a step of the turn, when their auto-pass setting stops there: with 'all', every
/// step; otherwise when attacked, at the end of each other player's turn, and in the declare blockers step of a
/// combat they're in when they could do something (Python's `step_priority`)
pub fn step_priority(g: &mut Game, q: PlayerId, step: &str, defender: Option<PlayerId>, attackers: &[PermId]) -> Res {
    let Some(a) = g.active else { return Ok(()) };
    let mode = autopass(g, q);
    if mode != "all" {
        let stop = match step {
            "end" => a != q,
            "attackers" => Some(q) == defender,
            "blockers" => (q == a || Some(q) == defender) && can_act(g, q)?,
            _ => false,
        };
        if !stop {
            return Ok(());
        }
    }
    let prompt = if step == "end" {
        if a == q { "End of your turn".to_string() } else { format!("End of {}'s turn", name_of(g, a)) }
    } else if step == "attackers" && Some(q) == defender {
        let power: i32 = attackers.iter().map(|&m| epow(g, m)).sum();
        format!("{} attacks you with {} creature(s) ({} power)", name_of(g, a), attackers.len(), power)
    } else {
        let sn = match step {
            "upkeep" => "upkeep",
            "draw" => "draw step",
            "combat" => "beginning of combat",
            "attackers" => "declare attackers step",
            "blockers" => "declare blockers step",
            "end" => "end step",
            other => other,
        };
        format!("{} {sn}", if a == q { "Your".to_string() } else { format!("{}'s", name_of(g, a)) })
    };
    respond(g, q, &prompt, None, None)
}

/// anything q could do at instant speed now: an instant or flash card they can afford, or an ability
pub fn can_act(g: &mut Game, q: PlayerId) -> Res<bool> {
    if can_respond(g, q) {
        return Ok(true);
    }
    g.responding += 1; // instant speed: sorcery-speed abilities aren't offered
    let r = can_act_abilities(g, q);
    g.responding -= 1;
    r
}

fn can_act_abilities(g: &mut Game, q: PlayerId) -> Res<bool> {
    // a tapped permanent's {T} abilities don't count
    let usable = |tapped: bool, labels: Vec<String>| labels.iter().any(|l| !(tapped && l.contains("{T}")));
    for m in g.player(q).perms.clone() {
        let x = g.perm(m);
        if x.phased || x.cd.is_none() {
            continue;
        }
        let tapped = x.tapped;
        let mut labels: Vec<String> = abilities::permanent_abilities(g, q, m).into_iter().map(|a| a.label).collect();
        labels.extend(cards::abilities(g, q, m).unwrap_or_default().into_iter().map(|a| a.label));
        if usable(tapped, labels) {
            return Ok(true);
        }
    }
    for l in g.player(q).lands.clone() {
        let labels: Vec<String> = abilities::land_abilities(g, q, l).into_iter().map(|a| a.label).collect();
        if usable(g.land(l).tapped, labels) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// does q hold anything to do at instant speed right now (an instant or flash card they can afford, an ability
/// counter)?
pub fn can_respond(g: &Game, q: PlayerId) -> bool {
    let have = engine::mana::total_mana(g, q, false) + g.player(q).pool.total();
    if g.player(q).hand.iter().any(|&c| {
        let d = g.db.get(c);
        legal::instant_speed(g, q, c) && d.cmc <= have && !d.land
    }) {
        return true;
    }
    g.player(q).perms.iter().any(|&m| g.perm(m).cd.is_some_and(|c| ABILITY_ANSWERS.contains(&&*g.db.get(c).name)))
}

/// q (the person) has priority in response to something (a spell on the stack, attackers, the end of a turn). They
/// may tap mana, cast instants and flash spells (a counterspell goes on the stack aimed at a spell there), use
/// instant-speed abilities, or pass. Returns once they pass, or once something they did went on the stack (it has
/// resolved by then, after its own round of priority)
pub fn respond(g: &mut Game, q: PlayerId, prompt: &str, spell: Option<CardId>, caster: Option<PlayerId>) -> Res {
    g.responding += 1; // something is waiting to resolve: no sorcery-speed play
    let r = respond_loop(g, q, prompt, spell, caster);
    g.responding = g.responding.saturating_sub(1);
    r
}

fn respond_loop(g: &mut Game, q: PlayerId, prompt: &str, spell: Option<CardId>, caster: Option<PlayerId>) -> Res {
    let Some(h) = human_of(g, q) else { return Ok(()) };
    // who gets priority on it, in turn order after the caster
    let order: Vec<Value> = match caster {
        Some(c) => g
            .after(c)
            .filter(|&x| g.player(x).alive)
            .map(|x| json!({ "key": &*g.player(x).key, "name": name_of(g, x) }))
            .collect(),
        None => vec![],
    };
    while !g.over && g.player(q).alive {
        let stack = if !g.stack.is_empty() {
            stack_view(g)
        } else if let Some(s) = spell {
            vec![json!({ "name": card_name(g, s) })]
        } else {
            vec![]
        };
        let req = Request {
            kind: intern("priority"),
            prompt: format!("{prompt}. You have priority"),
            choices: vec![],
            data: json!({
                "stack": stack,
                "order": order,
                "caster": caster.map(|c| name_of(g, c)),
            }),
        };
        // a closed or rewound session answers Cancel: unwind the game (Python's ask raises Cancelled); treating it as an
        // unknown action looped forever, telling "invalid" each time
        let ans = h.ask(g, q, req);
        if ans == Answer::Cancel {
            return Err(Stop::Cancelled);
        }
        let act = action_of(ans);
        if do_of(&act) == "pass" {
            return Ok(());
        }
        let n = g.stack_pushes;
        if do_of(&act) == "cast" && !g.stack.is_empty() && act.get("zone").and_then(Value::as_str) != Some("cmd") {
            if let Some(c) = hand_card(g, q, &act) {
                if g.db.get(c).tag(Tag::Ctr) {
                    if let Some(why) = cast_counterspell(g, q, c)? {
                        tell_invalid(&h, q, &why);
                        continue;
                    }
                    return Ok(());
                }
                if cards::copy_card(g, c) {
                    // Return the Favor, Dualcaster Mage: copy the spell
                    let (top_card, top_ctl) = {
                        let top = g.stack.last().unwrap();
                        (top.card, top.controller)
                    };
                    let mut why =
                        if card_name(g, c) != "Return the Favor" { legal::check_cast(g, q, c, "hand") } else { None };
                    if why.is_none() {
                        why = cards::copy_in_response(g, q, c, top_card, top_ctl)?;
                    }
                    if let Some(why) = why {
                        tell_invalid(&h, q, &why);
                    }
                    continue;
                }
            }
        }
        if let Some(why) = apply(g, q, &act)? {
            tell_invalid(&h, q, &why);
        }
        if g.stack_pushes != n && !g.stack.is_empty() {
            return Ok(()); // it went on the stack and resolved
        }
    }
    Ok(())
}

/// q casts counterspell c at a spell on the stack (the top one, or one they pick when there are several it can
/// counter). None, or why not
pub fn cast_counterspell(g: &mut Game, q: PlayerId, c: CardId) -> Res<Option<String>> {
    let many = g.stack.len() > 1;
    let cands: Vec<&StackItem> = g
        .stack
        .iter()
        .rev()
        .filter(|it| it.controller != q || many)
        .filter(|it| it.card.is_some_and(|sc| stack::counter_ok(g, c, sc)) && stack::counterable(g, it))
        .collect();
    if cands.is_empty() {
        return Ok(Some(format!("{} can't counter anything on the stack.", card_name(g, c))));
    }
    let mut pick = (cands[0].id, cands[0].card.unwrap());
    if cands.len() > 1 {
        let labels: Vec<String> =
            cands.iter().map(|x| format!("{} ({})", card_name(g, x.card.unwrap()), name_of(g, x.controller))).collect();
        let ids: Vec<(u32, CardId)> = cands.iter().map(|x| (x.id, x.card.unwrap())).collect();
        let prompt = format!("{}: counter which spell?", card_name(g, c));
        let Some(k) = choices::choose(g, q, "target", &prompt, &labels, Some("cancel"), None)? else {
            return Ok(Some("Cancelled.".into()));
        };
        pick = ids[k];
    }
    let (id, spell) = pick;
    if let Some(why) = legal::check_counter(g, q, c, spell) {
        return Ok(Some(why));
    }
    let alt = legal::alternative_counter_cost(g, q, c);
    let (gn, pips) = stack::counter_cost(g, c, Some(spell), Some(q));
    if mana::cost_problem(g, q, gn, &pips).is_none() {
        // pay from your pool
        mana::pay_from_pool(g, q, gn, &pips);
        g.free_counter = true; // already paid
    } else if alt.is_none() {
        let why = mana::cost_problem(g, q, gn, &pips).unwrap_or_default();
        return Ok(Some(format!("Can't cast {}. {why}", card_name(g, c))));
    }
    let r = stack::cast_counter_spell(g, q, c, id);
    g.free_counter = false;
    if !r? {
        return Ok(Some(format!("{} couldn't be cast.", card_name(g, c))));
    }
    Ok(None)
}

fn hand_card(g: &Game, p: PlayerId, act: &Value) -> Option<CardId> {
    g.player(p).hand.get(index_of(act, "card")?).copied()
}

/// carry out one action; returns None, or the reason it isn't legal (nothing changes then) (Python's `apply`)
pub fn apply(g: &mut Game, p: PlayerId, act: &Value) -> Res<Option<String>> {
    let d = do_of(act);
    let zone = act.get("zone").and_then(Value::as_str);
    if d == "tap" {
        let src = act.get("source").and_then(Value::as_i64).unwrap_or(-1);
        let colour = act.get("colour").and_then(Value::as_str).and_then(|s| s.chars().next());
        return mana::tap(g, p, src, colour);
    }
    if d == "land" && zone == Some("gy") {
        let Some(c) = index_of(act, "card").and_then(|i| g.player(p).gy.get(i).copied()) else {
            return Ok(Some("There is no such card in your graveyard.".into()));
        };
        if let Some(why) = legal::check_land_gy(g, p, c) {
            return Ok(Some(why));
        }
        let pl = g.player(p);
        if !(pl.yawg && pl.yawg_gy.contains(&c)) {
            crate::cardcode::muld_mark(g, p, "L")?; // Muldrotha
        }
        let pl = g.player_mut(p);
        let i = pl.gy.iter().position(|&x| x == c).unwrap();
        pl.gy.remove(i);
        turn::play_land_card(g, p, c, "plays from the graveyard")?;
        engine::life::check_state(g)?;
        return Ok(None);
    }
    if d == "cast" && zone == Some("gy") {
        let Some(c) = index_of(act, "card").and_then(|i| g.player(p).gy.get(i).copied()) else {
            return Ok(Some("There is no such card in your graveyard.".into()));
        };
        if super::alt::gy_hook(g, c) {
            // graveyard abilities the card code keeps for the AI (escape, unearth ...) beside the usual cast
            let alts = super::alt::gy_alts(g, p, c)?;
            if !alts.is_empty() {
                let normal = legal::check_cast_gy(g, p, c).is_none();
                let mut labels: Vec<String> = vec![];
                if normal {
                    labels.push(format!("{}: cast it from your graveyard", card_name(g, c)));
                }
                labels.extend(alts.iter().map(|w| w.0.clone()));
                let k = if labels.len() > 1 {
                    let prompt = format!("{}: which?", card_name(g, c));
                    let Some(k) = choices::choose(g, p, "choose", &prompt, &labels, Some("cancel"), None)? else {
                        return Ok(None);
                    };
                    k
                } else {
                    0
                };
                if normal && k == 0 {
                    return cast(g, p, c, "gy");
                }
                return super::alt::run(g, p, c, &alts[k - usize::from(normal)].1);
            }
        }
        if let Some(why) = legal::check_cast_gy(g, p, c) {
            return Ok(Some(why));
        }
        if cards::gy_override(g, c) {
            return cards::cast_from_gy(g, p, c); // graveyard casts the card code keeps for you
        }
        return cast(g, p, c, "gy");
    }
    if d == "land" {
        let Some(c) = hand_card(g, p, act) else {
            return Ok(Some("There is no such card in your hand.".into()));
        };
        if let Some(why) = legal::check_land(g, p, c) {
            return Ok(Some(why));
        }
        let pl = g.player_mut(p);
        let i = pl.hand.iter().position(|&x| x == c).unwrap();
        pl.hand.remove(i);
        turn::play_land_card(g, p, c, "plays")?;
        engine::life::check_state(g)?;
        return Ok(None);
    }
    if d == "use" {
        if act.get("land").is_some() {
            let Some(l) = index_of(act, "land").and_then(|i| g.player(p).lands.get(i).copied()) else {
                return Ok(Some("You don't control that land.".into()));
            };
            return use_land(g, p, l);
        }
        let Some(m) = index_of(act, "perm").and_then(|i| g.player(p).perms.get(i).copied()) else {
            return Ok(Some("You don't control that.".into()));
        };
        return use_perm(g, p, m);
    }
    if d == "cast" {
        let zone = if zone == Some("cmd") { "cmd" } else { "hand" };
        let c = if zone == "cmd" { commander_of(g, p, act) } else { hand_card(g, p, act) };
        let Some(c) = c else { return Ok(Some("There is no such card in your hand.".into())) };
        if zone == "hand" && cards::hand_override(g, c) {
            return cards::cast_from_hand(g, p, c);
        }
        if let Some(why) = legal::check_cast(g, p, c, zone) {
            return Ok(Some(why));
        }
        return cast(g, p, c, zone);
    }
    Ok(Some("Unknown action.".into()))
}

// ------------------------------------------------------------------ abilities
/// what an ability of a permanent does
pub enum AbilityKind {
    Equip,
    Extra(abilities::Act<PermId>),
    /// a card-code option the AI would also use
    Hook(Action),
}

/// what p can activate on permanent m now: Equip, and the abilities its card code offers (planeswalker loyalty
/// abilities, Triskelion, Deathrite ...) (Python's `abilities_of`)
pub fn abilities_of(g: &mut Game, p: PlayerId, m: PermId) -> Res<Vec<(String, AbilityKind)>> {
    let mut out = vec![];
    if crate::cardcode::ability_locked(g, m, p) {
        return Ok(out); // Arrest, Pithing Needle, Collector Ouphe ...: its activated abilities can't be activated
    }
    if let Some(n) = legal::equip_cost(g, m) {
        out.push((format!("Equip {{{n}}}"), AbilityKind::Equip));
    }
    let post = if g.active == Some(p) { Some(g.step == Step::Main2) } else { None }; // None: instant-speed only
    for a in abilities::permanent_abilities(g, p, m) {
        out.push((a.label, AbilityKind::Extra(a.act)));
    }
    out.extend(super::karn::perm_abilities(g, p, m).into_iter().map(|a| (a.label, AbilityKind::Extra(a.act))));
    if super::karn::owns_perm(g, m) {
        return Ok(out); // the Karn deck's cards: all listed above (the AI's hook options would show twice)
    }
    // the card's compiled abilities (its text, e.g. Wayfarer's Bauble), paid from your pool
    for (label, idx) in crate::dsl::human_abilities(g, p, m) {
        out.push((label, AbilityKind::Hook(Action::Dsl { src: m, idx })));
    }
    if let Some(own) = cards::abilities(g, p, m) {
        // your deck's card: its abilities by the rules
        out.extend(own.into_iter().map(|a| (a.label, AbilityKind::Extra(a.act))));
        return Ok(out);
    }
    if !g.hooks.is_empty() {
        for (src, imp) in engine::hooks::hooked(g, Event::Options) {
            if src == m {
                for o in (imp.options.unwrap())(g, src, p, post)? {
                    if let Some(a) = o.act {
                        out.push((o.label, AbilityKind::Hook(a)));
                    }
                }
            }
        }
    }
    Ok(out)
}

fn use_perm(g: &mut Game, p: PlayerId, m: PermId) -> Res<Option<String>> {
    let abil = abilities_of(g, p, m)?;
    let mname = perm_name(g, m);
    if abil.is_empty() {
        return Ok(Some(format!("{mname} has no ability you can activate right now.")));
    }
    let labels: Vec<String> = abil.iter().map(|a| a.0.clone()).collect();
    let Some(k) = choices::choose(g, p, "choose", &format!("{mname}: which ability?"), &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    let (label, kind) = abil.into_iter().nth(k).unwrap();
    match kind {
        AbilityKind::Equip => {
            let cre: Vec<PermId> = g
                .player(p)
                .perms
                .iter()
                .copied()
                .filter(|&x| g.is_creature(x) && !g.perm(x).phased && x != m)
                .collect();
            if cre.is_empty() {
                return Ok(Some("You have no creature to equip.".into()));
            }
            let labels: Vec<String> = cre.iter().map(|&x| legal::describe_target(g, p, Target::Perm(x))).collect();
            let prompt = format!("Equip {mname} onto which creature?");
            let Some(j) = choices::choose(g, p, "target", &prompt, &labels, Some("cancel"), None)? else {
                return Ok(None);
            };
            let t = cre[j];
            if let Some(why) = legal::check_equip(g, p, m, t) {
                return Ok(Some(why));
            }
            mana::pay_from_pool(g, p, legal::equip_cost(g, m).unwrap_or(0), "");
            crate::glog!(g, "  {} equips {} to {}", name_of(g, p), mname, perm_name(g, t));
            let tn = perm_name(g, t);
            if stack::ability_window(g, p, Some(m), &format!("equip to {tn}"), None, Some(t))?
                && g.player(p).perms.contains(&t)
                && g.player(p).perms.contains(&m)
            {
                g.perm_mut(m).attached = Some(t);
            }
            Ok(None)
        }
        AbilityKind::Extra(f) => f(g, p, m),
        AbilityKind::Hook(a) => {
            if !crate::ai::act::perform(g, p, &a)? {
                return Ok(Some(format!("{label}: that can't be done right now.")));
            }
            engine::life::check_state(g)?;
            Ok(None)
        }
    }
}

/// an activated ability of land L (beyond tapping for mana)
pub fn use_land(g: &mut Game, p: PlayerId, l: LandId) -> Res<Option<String>> {
    let abil = abilities::land_abilities(g, p, l);
    let lname = card_name(g, g.land(l).cd);
    let alts = super::alt::land_alts(g, p, l)?; // the land code's own abilities (the AI's hook)
    if abil.is_empty() && alts.is_empty() {
        return Ok(Some(format!("{lname} has no ability to activate beyond mana (tap it for mana with tap).")));
    }
    let mut labels: Vec<String> = abil.iter().map(|a| a.label.clone()).collect();
    labels.extend(alts.iter().map(|a| a.0.clone()));
    let Some(k) = choices::choose(g, p, "choose", &format!("{lname}: which ability?"), &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    if k >= abil.len() {
        let cd = g.land(l).cd;
        return super::alt::run(g, p, cd, &alts[k - abil.len()].1);
    }
    (abil[k].act)(g, p, l)
}

// ------------------------------------------------------------------ casting
/// cast card c from zone (its costs from your pool); restricted mana (Secluded Courtyard) may pay for it if it's a
/// creature spell of the named type
pub fn cast(g: &mut Game, p: PlayerId, c: CardId, zone: &str) -> Res<Option<String>> {
    let prev = std::mem::replace(&mut g.player_mut(p).pool.spending, Some(Spending::Card(c)));
    let r = cast_inner(g, p, c, zone);
    g.player_mut(p).pool.spending = prev;
    r
}

macro_rules! pick {
    ($e:expr) => {
        match $e {
            Some(k) => k,
            None => return Ok(None),
        }
    };
}

/// cast c (already checked legal): target and extra costs, pay from the pool, then the engine casts and resolves it.
/// Choices not yet made by the player (X for now) are made automatically and reported
fn cast_inner(g: &mut Game, p: PlayerId, c: CardId, zone: &str) -> Res<Option<String>> {
    let d = g.db.get(c).clone();
    let cname = d.name.to_string();
    let me = name_of(g, p);
    let mut zone = zone.to_string();
    let (mut gn, mut pips) = legal::base_cost(g, p, c);
    let cit = zone == "citadel"; // Bolas's Citadel: life equal to mana value, no mana cost
    if cit {
        zone = "lib".into();
        gn = 0;
        pips = String::new();
    }
    let mut ctx = Ctx::default();
    let mut gymode = None;
    if zone == "gy" {
        let Some((kind, mg, mp)) = legal::gy_mode(g, p, c) else {
            return Ok(Some(format!("You can't cast {cname} from your graveyard right now.")));
        };
        gn = mg;
        pips = mp;
        match kind {
            legal::GyKind::Escape => zone = "escape".into(), // Underworld Breach: back to the graveyard afterwards
            legal::GyKind::Muldrotha => {
                // which of its types this uses up (an artifact creature ...)
                let ts = crate::impls::mine::muld_types(g, p, c);
                let names = |t: &str| match t {
                    "L" => "land",
                    "C" => "creature",
                    "A" => "artifact",
                    "E" => "enchantment",
                    "P" => "planeswalker",
                    _ => "battle",
                };
                let k = if ts.len() > 1 {
                    let labels: Vec<String> = ts.iter().map(|t| names(t).to_string()).collect();
                    let prompt = format!("{cname}: cast it with Muldrotha as which type?");
                    pick!(choices::choose(g, p, "choose", &prompt, &labels, Some("cancel"), None)?)
                } else {
                    0
                };
                ctx.muld_type = Some(ts[k]);
                zone = "mgy".into();
            }
            _ => {}
        }
        gymode = Some(kind);
    }
    if d.tags.str(Tag::Wipe).is_some() && d.tags.has(Tag::Rem) && !cit {
        // overload (Cyclonic Rift, Vandalblast)
        let (og, op) = crate::ai::decks::wipe_cost(g, p, c);
        let over = format!("overloaded ({}): every one you don't control", mana::cost_text(og, &op));
        let k = if legal::spell_targets(g, p, c).is_some_and(|t| !t.is_empty()) {
            let one = format!("one target ({})", mana::cost_text(gn, &pips));
            pick!(choices::choose(
                g,
                p,
                "choose",
                &format!("{cname}: cast it how?"),
                &[one, over.clone()],
                Some("cancel"),
                None
            )?)
        } else {
            // nothing to target: only the overloaded mode is legal
            let prompt = format!("{cname}: nothing to target, so it can only be cast overloaded");
            pick!(choices::choose(g, p, "choose", &prompt, &[over.clone()], Some("cancel"), None)?);
            1
        };
        if k == 1 {
            if let Some(why) = mana::pay_from_pool(g, p, og, &op) {
                return Ok(Some(format!("Can't overload {cname}. {why}")));
            }
            crate::glog!(g, "  {me} casts {cname} overloaded");
            engine::cast::cast_card(g, p, c, intern(&zone), Ctx::default())?;
            return Ok(None);
        }
    }
    if d.tags.has(Tag::Mastery) && !cit {
        // Mizzix's Mastery: one target, or overloaded
        let one = format!("one target ({})", mana::cost_text(gn, &pips));
        let over = "overloaded ({5}{R}{R}{R}): every instant and sorcery card in your graveyard".to_string();
        let k = pick!(choices::choose(
            g,
            p,
            "choose",
            &format!("{cname}: cast it how?"),
            &[one, over],
            Some("cancel"),
            None
        )?);
        if k == 1 {
            gn = 5;
            pips = "RRR".into();
            ctx.overload = true;
        } else {
            let Some(pick) = cards::mastery_target(g, p, c)? else { return Ok(None) };
            ctx.mastery_pick = Some(pick);
        }
    }
    match cards::cast_target(g, p, c)? {
        cards::CastTarget::NotNeeded => {}
        cards::CastTarget::Cancelled => return Ok(None),
        cards::CastTarget::Picked(t) => ctx.target = Some(t), // a tagless spell with a creature target
    }
    if let Some(tgts) = legal::spell_targets(g, p, c) {
        // targets are chosen before the spell is paid for
        if tgts.is_empty() {
            return Ok(Some(format!("{cname} has no legal target right now.")));
        }
        let kind = d.tags.str(Tag::Tgt).unwrap_or("c");
        let what = format!("{}{}", legal::what(kind), if d.tags.has(Tag::Face) { " or player" } else { "" });
        let labels: Vec<String> = tgts.iter().map(|&x| legal::describe_target(g, p, x)).collect();
        let prompt = format!("{cname}: choose a target ({what})");
        let k = pick!(choices::choose(g, p, "target", &prompt, &labels, Some("cancel"), None)?);
        let x = tgts[k];
        let (eg, ep) = legal::target_extra_cost(g, p, c, x);
        gn += eg;
        pips.push_str(&ep);
        if let Some(why) = mana::cost_problem(g, p, gn, &pips) {
            return Ok(Some(format!("Can't cast {cname} on that target. {why}")));
        }
        match x {
            Target::Player(q) => ctx.face = Some(q),
            Target::Perm(m) => ctx.target = Some(m),
        }
    }
    // a card with its own resolve code and no Rem tag: what the person must choose (target, mode, cost ...)
    let spec_costs = match specs::choose_pre(g, p, c, &mut ctx)? {
        specs::Pre::Cancel => return Ok(None),
        specs::Pre::Problem(why) => return Ok(Some(why)),
        specs::Pre::Ok(costs) => costs,
    };
    let mut flash_sac = false;
    if d.tags.str(Tag::Rean).is_some() {
        // reanimation: its target is chosen as it's cast
        let (cd, src) = pick!(choices::choose_rean(g, p, c)?);
        ctx.rean_target = Some(cd);
        ctx.rean_src = Some(src);
        ctx.rean_value = f64::from(g.db.get(cd).cmc.max(4)); // how hard opponents try to stop it
        if cname == "Necromancy" && zone == "hand" && legal::sorcery_timing(g, p).is_some() {
            flash_sac = true; // cast as though it had flash: sacrificed at cleanup
        }
    }
    let mut sac_cost = cname == "Deadly Dispute";
    if d.tags.has(Tag::Deluge) {
        // Toxic Deluge: pay X life, all creatures get -X/-X
        let top = (g.player(p).life - 1).min(20).max(0);
        let labels: Vec<String> = (0..=top).map(|x| format!("X = {x}")).collect();
        let prompt = format!("{cname}: choose X (you pay X life; creatures with toughness X or less die)");
        let k = pick!(choices::choose(g, p, "choose", &prompt, &labels, Some("cancel"), None)?);
        ctx.deluge_x = Some(k as i32);
    }
    let mut discard_cost = false;
    if cname == "Bitter Triumph" {
        // additional cost: discard a card, or pay 3 life
        let others = g.player(p).hand.iter().any(|&x| x != c);
        let mut labels = vec!["pay 3 life".to_string()];
        if others {
            labels.push("discard a card".into());
        }
        let prompt = format!("{cname}: pay its extra cost how?");
        let k = pick!(choices::choose(g, p, "choose", &prompt, &labels, Some("cancel"), None)?);
        discard_cost = k == 1;
    }
    let mut fodder = None;
    if d.tags.has(Tag::Needsac) {
        // Diabolic Intent: sacrifice a creature as you cast it
        let cre: Vec<PermId> =
            g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
        if cre.is_empty() {
            return Ok(Some(format!("{cname} needs a creature to sacrifice as you cast it, and you control none.")));
        }
        let labels: Vec<String> = cre.iter().map(|&m| legal::describe_target(g, p, Target::Perm(m))).collect();
        let prompt = format!("{cname}: sacrifice which creature?");
        let k = pick!(choices::choose(g, p, "choose", &prompt, &labels, Some("cancel"), None)?);
        fodder = Some(cre[k]);
    }
    if d.has_dsl() && !cast::additional_cost(g, p, c, true)? {
        return Ok(Some(format!("You can't pay {cname}'s additional cost.")));
    }
    let mut life = 0;
    let phy = legal::phyrexian(g, p, c);
    if !phy.is_empty() && !cit {
        // {B/P}: its colour, or 2 life (your choice)
        let ways = legal::phyrexian_ways(g, p, c, gn, &pips);
        if ways.is_empty() {
            let why = mana::cost_problem(g, p, gn, &pips).unwrap_or_default();
            return Ok(Some(format!(
                "Can't cast {cname}. {why} Its Phyrexian mana can be paid with 2 life each instead."
            )));
        }
        let mut k = 0;
        if ways.len() > 1 {
            let sym: String = phy.chars().map(|x| format!("{{{x}/P}}")).collect();
            let labels: Vec<String> = ways
                .iter()
                .map(|(paid, _)| {
                    let mana_part: String =
                        phy.chars().skip((*paid / 2) as usize).map(|y| format!("{{{y}}}")).collect();
                    let life_part = if *paid != 0 { format!("{paid} life") } else { String::new() };
                    [mana_part, life_part].into_iter().filter(|x| !x.is_empty()).collect::<Vec<_>>().join(" and ")
                })
                .collect();
            let prompt = format!("{cname}: pay {sym} with mana or life?");
            k = pick!(choices::choose(g, p, "choose", &prompt, &labels, Some("cancel"), None)?);
        }
        life = ways[k].0;
        pips = ways[k].1.clone();
    }
    if let Some(why) = mana::pay_from_pool(g, p, gn, &pips) {
        return Ok(Some(format!("Can't cast {cname}. {why}{}", legal::cost_changers(g, p, c))));
    }
    if life > 0 {
        engine::life::lose_life(g, p, life, Some(p), "other", None)?;
        crate::glog!(g, "  {me} pays {life} life for {cname}'s Phyrexian mana");
    }
    if cit {
        if g.player(p).library.last() != Some(&c) {
            return Ok(Some("The top card of your library has changed.".into()));
        }
        g.player_mut(p).library.pop();
        engine::life::lose_life(g, p, d.cmc as i32, Some(p), "other", None)?;
        g.player_mut(p).stat(intern("citadel_casts"), 1);
        crate::glog!(g, "  {me} casts {cname} from the top for {} life (Citadel)", d.cmc);
    }
    if d.tags.has(Tag::Crackle) {
        // {X}{X}{X}{R}{R}: X from what's left in the pool
        let top = g.player(p).pool.total() / 3;
        let labels: Vec<String> = (0..=top).map(|x| format!("X = {x}")).collect();
        let prompt = format!("{cname}: choose X (it costs {{X}}{{X}}{{X}} more; 5X damage to each of up to X targets)");
        let k = choices::choose(g, p, "choose", &prompt, &labels, None, None)?.unwrap_or(0) as u32;
        mana::pay_from_pool(g, p, 3 * k, "");
        ctx.x = k as i32;
    }
    specs::choose_post(g, p, c, &mut ctx)?;
    if d.tags.has(Tag::Tokx) || d.tags.has(Tag::Xtutor) || d.tags.has(Tag::Xdrain) {
        // X: up to what's left in the pool
        let top = g.player(p).pool.total();
        let x = if top > 0 {
            let labels: Vec<String> = (0..=top).map(|x| format!("X = {x}")).collect();
            let prompt = format!("{cname}: choose X (paid from what is left in your pool)");
            choices::choose(g, p, "choose", &prompt, &labels, None, None)?.unwrap_or(0) as u32
        } else {
            0
        };
        mana::pay_from_pool(g, p, x, "");
        ctx.x = x as i32;
        if d.tags.has(Tag::Xtutor) {
            g.last_x = x as i32;
        }
    }
    if d.has_dsl() {
        cast::additional_cost(g, p, c, false)?;
    }
    specs::pay_costs(g, p, c, &spec_costs)?;
    if let Some(f) = fodder {
        crate::glog!(g, "  {me} sacrifices {} for {cname}", perm_name(g, f));
        zones::die(g, f, "sac")?;
    }
    if gymode == Some(legal::GyKind::Sac3) {
        // Dread Return's flashback: sacrifice three creatures
        for i in 0..3 {
            let cre: Vec<PermId> =
                g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
            choices::sacrifice_creature(
                g,
                p,
                &cre,
                &format!("{cname} flashback: sacrifice a creature ({} of 3)", i + 1),
            )?;
        }
    }
    if gymode == Some(legal::GyKind::Flashback)
        && let Some(n) = d.tags.int(Tag::Fblife).filter(|_| d.tags.has(Tag::Fblife))
    {
        engine::life::lose_life(g, p, n, Some(p), "other", None)?;
    }
    if gymode == Some(legal::GyKind::Escape) {
        // exile three other cards from your graveyard
        let others: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&y| y != c).collect();
        let prompt = format!("Escape {cname}: exile a card from your graveyard");
        for x in choices::pick_cards(g, p, &others, 3, &prompt, "choose")? {
            let pl = g.player_mut(p);
            if let Some(i) = pl.gy.iter().position(|&y| y == x) {
                pl.gy.remove(i);
            }
            pl.exile.push(x);
        }
    }
    if std::mem::take(&mut sac_cost) {
        choices::sac_artifact_or_creature(g, p, &cname)?;
    }
    if discard_cost {
        let others: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&y| y != c).collect();
        let x = choices::pick_cards(g, p, &others, 1, &format!("{cname}: discard a card"), "choose")?[0];
        zones::discard_cards(g, p, &[x])?;
        ctx.paid_otherwise = true;
    }
    if ctx.rean_target.is_some() {
        return cast_rean(g, p, c, &zone, ctx, flash_sac);
    }
    cast::cast_card(g, p, c, intern(&zone), ctx)?;
    Ok(None)
}

/// a reanimation spell (paid): the same steps the AI's reanimation takes, with the person's target. Auras like
/// Animate Dead are modelled as the effect only, as the AI's are
fn cast_rean(g: &mut Game, p: PlayerId, c: CardId, zone: &str, ctx: Ctx, flash_sac: bool) -> Res<Option<String>> {
    let from_hand = zone == "hand";
    let pl = g.player_mut(p);
    let src_zone = if from_hand { &mut pl.hand } else { &mut pl.gy };
    if let Some(i) = src_zone.iter().position(|&x| x == c) {
        src_zone.remove(i);
    }
    pl.spells_this_turn += 1;
    pl.stat(intern("spells_cast"), 1);
    pl.cast_names.insert(c);
    cast::on_cast(g, p, c)?;
    let (cd, src) = (ctx.rean_target.unwrap(), ctx.rean_src.unwrap());
    let extra = if src != p { format!(" in {}'s graveyard", name_of(g, src)) } else { String::new() };
    crate::glog!(g, "  {} casts {} targeting {}{extra}", name_of(g, p), card_name(g, c), card_name(g, cd));
    let in_hand_dest = from_hand;
    macro_rules! dest {
        () => {
            if in_hand_dest { &mut g.player_mut(p).gy } else { &mut g.player_mut(p).exile }
        };
    }
    if g.over || !g.player(p).alive {
        return Ok(None);
    }
    let value = ctx.rean_value;
    if !stack::counter_window(g, p, c, value, vec![])?
        || crate::ai::decks::sauron_grounds_response(g, p, value)?
        || (!g.hooks.is_empty() && crate::cardcode::gy_response(g, p, value, src)?)
    {
        dest!().push(c);
        return Ok(None);
    }
    let n0 = g.player(p).perms.len();
    crate::ai::seph_rean_resolve(g, p, c, &ctx)?;
    if flash_sac {
        // Necromancy at instant speed: its enter effects, then gone
        let new: Vec<PermId> =
            g.player(p).perms.iter().skip(n0).copied().filter(|&m| g.perm(m).cd == Some(cd)).collect();
        for m in new {
            crate::glog!(g, "    {} is sacrificed (Necromancy was cast at instant speed)", perm_name(g, m));
            zones::die(g, m, "sac")?;
        }
    }
    dest!().push(c);
    engine::life::check_state(g)?;
    Ok(None)
}

// ------------------------------------------------------------------ simultaneous triggers
fn trig_label(g: &Game, t: &crate::hooks::Trigger) -> String {
    let n = match t.src {
        Some(m) => g.perm(m).cd.map(|c| g.db.get(c).name.to_string()).unwrap_or_else(|| g.perm(m).name.to_string()),
        None => String::new(),
    };
    format!("{n}: {}", t.name.unwrap_or("None"))
}

/// the person orders their own simultaneous triggers (which resolves first); the AI keeps the code's order
/// (Python's `_order_triggers`)
pub fn order_triggers(g: &mut Game, real: Vec<crate::hooks::Trigger>) -> Res<Vec<crate::hooks::Trigger>> {
    if real.len() < 2 || !g.humans.any() {
        return Ok(real);
    }
    let mut out = vec![];
    let mut i = 0;
    while i < real.len() {
        let mut j = i;
        while j < real.len() && real[j].controller == real[i].controller {
            j += 1;
        }
        let mut group: Vec<crate::hooks::Trigger> = real[i..j].to_vec();
        let q = group[0].controller;
        if super::is_human(g, q) && group.len() > 1 {
            let mut left = group.clone();
            let mut first = vec![];
            let key = g.player(q).key;
            let last: Option<Vec<String>> = g.trig_orders.iter().find(|x| x.0 == key).map(|x| x.1.clone());
            let mut same = last.as_ref().is_some_and(|l| {
                let mut a = l.clone();
                let mut b: Vec<String> = left.iter().map(|t| trig_label(g, t)).collect();
                a.sort();
                b.sort();
                a == b
            });
            while left.len() > 1 {
                let labels: Vec<String> = left.iter().map(|t| trig_label(g, t)).collect();
                if labels.iter().all(|l| *l == labels[0]) {
                    break; // the rest are the same trigger (four Sephiroth drains): their order can't matter
                }
                let extra: Vec<String> = match (&last, same) {
                    (Some(l), true) => vec![format!("Same order as last time ({})", l.join(", then "))],
                    _ => vec![],
                };
                let mut shown = extra.clone();
                shown.extend(labels);
                let k = choices::choose_one(g, q, "choose", "Your triggers: which resolves first?", &shown)?;
                if same && k == 0 {
                    for name in last.as_ref().unwrap() {
                        let pos = left.iter().position(|t| &trig_label(g, t) == name).unwrap();
                        first.push(left.remove(pos));
                    }
                    break;
                }
                first.push(left.remove(k - extra.len()));
                same = false;
            }
            first.extend(left);
            let labels: Vec<String> = first.iter().map(|t| trig_label(g, t)).collect();
            match g.trig_orders.iter_mut().find(|x| x.0 == key) {
                Some(e) => e.1 = labels,
                None => g.trig_orders.push((key, labels)),
            }
            first.reverse(); // pushed in reverse: the first to resolve goes on top
            group = first;
        }
        out.extend(group);
        i = j;
    }
    Ok(out)
}

/// the commander a `{"do": "cast", "zone": "cmd"}` action means: the one named by `"cmd"` (its index among the
/// seat's commanders, or its name), else the first one in the command zone
fn commander_of(g: &Game, p: PlayerId, act: &Value) -> Option<CardId> {
    let pl = g.player(p);
    let named = match act.get("cmd") {
        Some(Value::Number(n)) => n.as_u64().and_then(|i| pl.commanders().nth(i as usize)),
        Some(Value::String(s)) => pl.commanders().find(|&c| g.db.get(c).name.eq_ignore_ascii_case(s)),
        _ => None,
    };
    named.or_else(|| pl.cmds_in_zone().next()).or(Some(pl.cmd))
}
