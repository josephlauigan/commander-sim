//! Practice mode: activated abilities of the Ur-Dragon, Kefka, Sauron and Thanos decks that the AI uses through its own
//! `options` hooks (which stay silent for a person, or only speak when the AI's value conditions say so). For a person
//! each is listed whenever the rules allow the activation, checked for timing, tapping and mana, then carried out,
//! asking the person what the card lets them choose (rust/agents/practice-gaps.md, Agent A).
//!
//! - **Garth One-Eye**: {T}; the name is chosen as the ability resolves (any of the six not yet chosen), then the copy's
//!   target (Terror, Disenchant) or X (Braingeyser) is chosen and the copy is cast from the pool. A copy the pool can't
//!   pay for ceases to exist, as the rules have it (the name is used up).
//! - **Kefka, Court Mage**: {8}, sorcery speed (`mine::kefka_ruin_go`; a person opponent picks what to sacrifice).
//! - **Nicol Bolas, the Ravager**: {4}{U}{B}{R}, sorcery speed (`card_models::bolas_flip`).
//! - **Thanos, the Mad Titan**: the power-up; odd or even is chosen as it resolves (`t6_thanos::thanos_go`).
//! - **Scion of the Ur-Dragon**: {2}; the person searches for the Dragon permanent card as the ability resolves.
//! - **Dragon's Hoard**, **Faerie Mastermind**, **Ranger-Captain of Eos**.
//!
//! Never reached for an AI seat: `cards::abilities` is only asked for a person's permanent.

use super::abilities::{PermAbility, Why, ability};
use super::{Target, choices, legal, mana};
use crate::engine::mana::cost_of;
use crate::engine::stack::{ability_window, ability_window_card};
use crate::engine::zones;
use crate::flow::Res;
use crate::ids::{PermId, PlayerId};
use crate::impls::{card_models, garth, mine, t6_scion, t6_thanos, t45_urdragon};
use crate::state::{DataKey, Game, Val};

const KEFKA: &str = "Kefka, Court Mage // Kefka, Ruler of Ruin";

fn pname(g: &Game, p: PlayerId) -> String {
    g.player(p).name.to_string()
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn tap_ok(g: &Game, m: PermId, creature: bool) -> Why {
    let x = g.perm(m);
    if x.tapped {
        return Some(format!("{} is tapped.", x.name));
    }
    if creature && x.sick && !crate::dsl::has_kw(g, m, "haste") {
        return Some(format!("{} has summoning sickness (it came under your control this turn).", x.name));
    }
    None
}

fn pay(g: &mut Game, p: PlayerId, gn: u32, pips: &str, what: &str) -> Why {
    mana::pay_from_pool(g, p, gn, pips).map(|why| format!("Can't activate {what}. {why}"))
}

fn sorcery(g: &Game, p: PlayerId, what: &str) -> Why {
    legal::sorcery_timing(g, p).map(|why| why.replace("do that", &format!("activate {what}")))
}

/// the abilities of p's permanent m named `nm` that this file provides, or None
pub fn abilities(g: &Game, _p: PlayerId, m: PermId, nm: &str) -> Option<Vec<PermAbility>> {
    Some(match nm {
        "Garth One-Eye" => vec![ability(
            "{T}: choose a card name (Disenchant, Braingeyser, Terror, Shivan Dragon, Regrowth, Black Lotus), create a copy and cast it",
            garth_person,
        )],
        KEFKA => {
            if g.perm(m).data.truthy(DataKey::Ruin) {
                vec![] // Kefka, Ruler of Ruin has no activated ability
            } else {
                vec![ability(
                    "{8}: each opponent sacrifices a permanent of their choice; transform Kefka (sorcery speed)",
                    kefka_ruin,
                )]
            }
        }
        "Nicol Bolas, the Ravager" => {
            let x = g.perm(m);
            if x.token || x.phys.is_some() {
                vec![] // a token or a copy has no front face to turn over
            } else {
                vec![ability("{4}{U}{B}{R}: exile him, return him transformed (sorcery speed)", bolas_flip)]
            }
        }
        card_models::ARISEN => arisen(),
        "Thanos, the Mad Titan" => {
            let (gn, pips) = t6_thanos::cost(g, m);
            vec![ability(
                format!(
                    "Power-up {}: two +1/+1 counters; choose odd or even; destroy each other creature of that mana value (once)",
                    mana::cost_text(gn, pips)
                ),
                thanos_power_up,
            )]
        }
        "Scion of the Ur-Dragon" => vec![ability(
            "{2}: search for a Dragon permanent card, put it into your graveyard; Scion becomes a copy of it until end of turn",
            scion_search,
        )],
        "Dragon's Hoard" => vec![ability("{T}, remove a gold counter: draw a card", hoard_draw)],
        "Faerie Mastermind" => vec![ability("{3}{U}: each player draws a card", mastermind_draw)],
        "Ranger-Captain of Eos" => {
            vec![ability("Sacrifice it: your opponents can't cast noncreature spells this turn", ranger_silence)]
        }
        _ => return None,
    })
}

// ------------------------------------------------------------------ Garth One-Eye
/// {T}: the ability goes on the stack; as it resolves the person chooses a name that hasn't been chosen, a copy of
/// that card is created, and they may cast it, paying its costs from the pool (a target or X is chosen as it is cast)
fn garth_person(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if g.perm(m).owner != p {
        return Ok(Some("Garth One-Eye isn't yours.".into()));
    }
    if let Some(why) = tap_ok(g, m, true) {
        return Ok(Some(why));
    }
    if garth::unchosen(g, m).is_empty() {
        return Ok(Some("Every name has been chosen for this Garth One-Eye.".into()));
    }
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "  {} activates {}", pname(g, p), g.perm(m).name);
    if !ability_window(g, p, Some(m), "choose a name, create a copy of it and cast it", None, None)? {
        return Ok(None);
    }
    // the name, as the ability resolves
    let left = garth::unchosen(g, m);
    if left.is_empty() {
        return Ok(None);
    }
    let labels: Vec<String> = left
        .iter()
        .map(|&i| {
            let Some(c) = g.db.id(garth::NAMES[i]) else { return garth::NAMES[i].to_string() };
            let (gn, pips) = cost_of(g, p, c);
            let ok = mana::cost_problem(g, p, gn, &pips).is_none();
            format!(
                "{} {}{}",
                garth::NAMES[i],
                mana::cost_text(gn, &pips),
                if ok { "" } else { " (you can't pay for it now)" }
            )
        })
        .collect();
    let k = choices::choose_one(
        g,
        p,
        "choose",
        "Garth One-Eye: choose a card name (a copy of it is created; you may cast it, paying its costs)",
        &labels,
    )?;
    let i = left[k];
    let mask = g.perm(m).data.int(DataKey::Garth) | (1 << i);
    g.perm_mut(m).data.set(DataKey::Garth, Val::Int(mask));
    let name = garth::NAMES[i];
    crate::glog!(g, "  {} chooses {}: a copy of it", g.perm(m).name, name);
    let Some(c) = g.db.id(name) else { return Ok(None) };
    // the target or X of the copy, chosen as it is cast
    let mut target: Option<PermId> = None;
    let mut x = 0u32;
    if let Some(tgts) = legal::spell_targets(g, p, c) {
        let tgts: Vec<PermId> =
            tgts.into_iter().filter_map(|t| if let Target::Perm(q) = t { Some(q) } else { None }).collect();
        if tgts.is_empty() {
            crate::glog!(g, "    {name} has no legal target: the copy ceases to exist");
            return Ok(None);
        }
        let labels: Vec<String> = tgts.iter().map(|&q| legal::describe_target(g, p, Target::Perm(q))).collect();
        let k =
            choices::choose_one(g, p, "target", &format!("{name} (Garth One-Eye's copy): choose a target"), &labels)?;
        target = Some(tgts[k]);
    }
    if name == "Braingeyser" {
        let (gn, pips) = cost_of(g, p, c);
        let have = g.player(p).pool.total();
        let top = have.saturating_sub(gn + pips.chars().count() as u32);
        if top > 0 {
            let xs = choices::amount_choices(top);
            let labels: Vec<String> = xs.iter().map(|x| format!("X = {x}")).collect();
            let k = choices::choose_one(
                g,
                p,
                "choose",
                "Braingeyser (Garth One-Eye's copy): choose X (paid from what is left in your pool)",
                &labels,
            )?;
            x = xs[k];
        }
    }
    if !garth::cast_copy(g, p, c, target, x)? {
        crate::glog!(
            g,
            "    {name} can't be cast (the mana isn't in your pool, or it has no target): the copy ceases to exist"
        );
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

// ------------------------------------------------------------------ Kefka, Court Mage
fn kefka_ruin(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if g.perm(m).data.truthy(DataKey::Ruin) {
        return Ok(Some("Kefka has already transformed.".into()));
    }
    if let Some(why) = sorcery(g, p, "Kefka") {
        return Ok(Some(why));
    }
    if let Some(why) = mana::cost_problem(g, p, 8, "") {
        return Ok(Some(format!("Can't activate Kefka. {why}")));
    }
    if !mine::kefka_ruin_go(g, m, p, 0)? {
        return Ok(Some("Can't activate Kefka now.".into()));
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

// ------------------------------------------------------------------ Nicol Bolas, the Ravager
fn bolas_flip(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = sorcery(g, p, "Nicol Bolas") {
        return Ok(Some(why));
    }
    if let Some(why) = mana::cost_problem(g, p, 4, "UBR") {
        return Ok(Some(format!("Can't activate Nicol Bolas. {why}")));
    }
    if !card_models::bolas_flip(g, m, p, 0)? {
        return Ok(Some("Can't activate Nicol Bolas now.".into()));
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

// ------------------------------------------------------------------ Thanos, the Mad Titan
fn thanos_power_up(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if t6_thanos::used(g, m) {
        return Ok(Some("Thanos has already powered up (each power-up ability may be activated only once).".into()));
    }
    let (gn, pips) = t6_thanos::cost(g, m);
    if let Some(why) = mana::cost_problem(g, p, gn, pips) {
        return Ok(Some(format!("Can't activate Thanos' power-up. {why}")));
    }
    if !t6_thanos::thanos_go(g, m, p, 0)? {
        return Ok(Some("Can't activate Thanos' power-up now.".into()));
    }
    Ok(None)
}

// ------------------------------------------------------------------ Scion of the Ur-Dragon
fn scion_search(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if t6_scion::is_copy(g, m) {
        return Ok(Some(
            "Scion of the Ur-Dragon is a copy of another card and has lost this ability for the turn.".into(),
        ));
    }
    if let Some(why) = pay(g, p, 2, "", "Scion of the Ur-Dragon") {
        return Ok(Some(why));
    }
    crate::glog!(g, "  {} activates Scion of the Ur-Dragon", pname(g, p));
    if !ability_window(g, p, Some(m), "search for a Dragon, become a copy of it", None, None)? || g.over {
        return Ok(None);
    }
    // the search, as the ability resolves
    let found = choices::search(
        g,
        p,
        &|g: &Game, c| {
            let d = g.db.get(c);
            d.perm && t45_urdragon::dragon_card(g, c)
        },
        1,
        "Scion of the Ur-Dragon: search your library for a Dragon permanent card (it goes to your graveyard)",
        true,
    )?;
    if let Some(&c) = found.first() {
        g.player_mut(p).gy.push(c);
        crate::glog!(g, "    {} puts {} into the graveyard", pname(g, p), g.db.get(c).name);
        if live(g, m) {
            t6_scion::become_copy(g, m, c);
        }
    }
    Ok(None)
}

// ------------------------------------------------------------------ Dragon's Hoard
fn hoard_draw(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false) {
        return Ok(Some(why));
    }
    if t45_urdragon::gold(g, m) <= 0 {
        return Ok(Some("Dragon's Hoard has no gold counter to remove.".into()));
    }
    if !t45_urdragon::hoard_go(g, m, p, 0)? {
        return Ok(Some("Can't activate Dragon's Hoard now.".into()));
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

// ------------------------------------------------------------------ Faerie Mastermind
fn mastermind_draw(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = pay(g, p, 3, "U", "Faerie Mastermind") {
        return Ok(Some(why));
    }
    crate::glog!(g, "  {} activates Faerie Mastermind: each player draws a card", pname(g, p));
    if !ability_window(g, p, Some(m), "each player draws a card", None, None)? {
        return Ok(None);
    }
    let first = g.active.unwrap_or(p);
    let order: Vec<PlayerId> = std::iter::once(first).chain(g.after(first)).collect();
    for q in order {
        if g.player(q).alive && !g.over {
            zones::draw(g, q, 1, false)?;
        }
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

// ------------------------------------------------------------------ Ranger-Captain of Eos
fn ranger_silence(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if !live(g, m) {
        return Ok(Some("Ranger-Captain of Eos isn't on the battlefield.".into()));
    }
    let cd = g.perm(m).card();
    zones::die(g, m, "sac")?;
    crate::glog!(
        g,
        "  {} sacrifices Ranger-Captain of Eos: opponents can't cast noncreature spells this turn",
        pname(g, p)
    );
    if ability_window_card(g, p, cd, "your opponents can't cast noncreature spells this turn", None, None)? {
        g.silence = Some((g.turn_stamp(), p));
    }
    Ok(None)
}

// ------------------------------------------------------------------ spells whose effect the engine has no code for
/// A person's instant or sorcery of these decks that the engine resolves only through the AI's protection code (the
/// AI casts it as a response in `ai::pool`, never through `cast_card`): Slip Out the Back. True if handled here.
pub fn resolve_spell(g: &mut Game, _p: PlayerId, c: crate::ids::CardId, ctx: &crate::state::Ctx) -> Res<bool> {
    match &*g.db.get(c).name {
        "Slip Out the Back" => {
            // put a +1/+1 counter on target creature; it phases out (and so does anything attached to it)
            if let Some(t) = ctx.target.filter(|&t| live(g, t)) {
                crate::engine::repl::put_plus(g, t, 1);
                let worn: Vec<PermId> = g
                    .players
                    .iter()
                    .flat_map(|q| q.perms.iter().copied())
                    .filter(|&x| g.perm(x).attached == Some(t))
                    .collect();
                for x in worn.into_iter().chain([t]) {
                    g.perm_mut(x).phased = true;
                }
                crate::glog!(g, "    {}: {} gets a +1/+1 counter and phases out", g.db.get(c).name, g.perm(t).name);
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

// ------------------------------------------------------------------ Nicol Bolas, the Arisen
/// The back face Bolas turns into: a 7-loyalty planeswalker whose abilities are compiled (`dsl::use_loyalty` carries
/// them out, asking the person for the targets). Without these the person could flip Bolas and never use the walker.
fn arisen() -> Vec<PermAbility> {
    let list: [(u16, i32, &str); 4] = [
        (0, 2, "draw two cards"),
        (1, -3, "Nicol Bolas deals 10 damage to target creature or planeswalker"),
        (2, -4, "put target creature or planeswalker card from a graveyard onto the battlefield under your control"),
        (3, -12, "exile all but the bottom card of target player's library"),
    ];
    list.into_iter()
        .map(|(idx, cost, text)| {
            ability(format!("{cost:+}: {text}"), move |g: &mut Game, p: PlayerId, m: PermId| {
                if let Some(why) = legal::sorcery_timing(g, p) {
                    return Ok(Some(why.replace("do that", "activate a loyalty ability")));
                }
                if !mine::pw_once(g, m) {
                    return Ok(Some(format!("{} has already used a loyalty ability this turn.", g.perm(m).name)));
                }
                let have = g.perm(m).loyalty.unwrap_or(0);
                if cost < 0 && have < -cost {
                    return Ok(Some(format!("{} has only {have} loyalty.", g.perm(m).name)));
                }
                if !crate::dsl::use_loyalty(g, p, m, idx)? {
                    return Ok(Some("Can't activate that now.".into()));
                }
                crate::engine::life::check_state(g)?;
                Ok(None)
            })
        })
        .collect()
}

// ------------------------------------------------------------------ Nature's Lore
/// A person's Nature's Lore: "search your library for a Forest card, put it onto the battlefield". The AI's ramp takes
/// a basic land of its own pick (`zones::land_ramp`); a person finds any Forest card (a Forest, or a dual land with the
/// Forest type). True if handled here.
pub fn forest_search(g: &mut Game, p: PlayerId, c: crate::ids::CardId) -> Res<bool> {
    if &*g.db.get(c).name != "Nature's Lore" {
        return Ok(false);
    }
    let found = choices::search(
        g,
        p,
        &|g: &Game, x| {
            let d = g.db.get(x);
            d.land && d.land_types.has('G')
        },
        1,
        "Nature's Lore: search your library for a Forest card (it enters untapped)",
        true,
    )?;
    if let Some(&l) = found.first() {
        g.add_land_entering(p, l, false);
        zones::landfall(g, p)?;
    }
    Ok(true)
}
