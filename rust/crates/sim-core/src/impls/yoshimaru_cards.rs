//! Cards of the Yoshimaru / Tymna blink deck, modelled before that deck is registered: Yoshimaru, Ever Faithful, Minas
//! Tirith, The Seriema (its enters search is card code) and Witch Enchanter // Witch-Blessed Meadow. (Kambal, Consul of Allocation is compiled from its Oracle text and needs no code.) Each card's text is
//! in the doc comment above its code.

use crate::cards::{CardDb, Colors, Types};
use crate::dsl::model::Filter;
use crate::engine::life::lose_life;
use crate::engine::repl;
use crate::engine::stack::{ability_window_card, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{card_name, epow, is_artifact_perm};
use crate::engine::zones::{agent_for, agent_take, draw, max_by, searchable};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

/// a legendary permanent: printed, a legendary token, or the Ring-bearer (`legends::legendary`)
fn legendary(g: &Game, m: PermId) -> bool {
    super::legends::legendary(g, m)
}

fn is_land_card(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).land)
}

// ================================================================== Yoshimaru, Ever Faithful
/// **Yoshimaru, Ever Faithful** ({W} 1/1, Partner): "Whenever another legendary permanent you control enters, put a
/// +1/+1 counter on Yoshimaru." A permanent that is not a land comes through the enters event; a land through the
/// engine's land-enters event (`Event::LandEnters`, fired once per land by the sweep `zones::lands_entered` makes over
/// the lands on the battlefield: played, fetched, put onto the battlefield, a token copy; a land that enters and leaves
/// again inside one resolution before the sweep is not seen, because `add_land` records no entry). Both are stack
/// triggers, so Panharmonicon, Ancient Greenwarden, Starfield Vocalist, Elesh Norn, Mother of Machines (an additional
/// trigger of yours; an opponent's stops it, `hooks::enter_blanked`), Delney and Hushbringer act on them.
/// A creature land is counted by the land event only (a token copy of a legendary land that is also a creature fires
/// both events).
fn yoshimaru_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src || !live(g, src) || g.perm(src).owner != p || is_land_card(g, m) || !legendary(g, m) {
        return Ok(());
    }
    counter(g, src, p)
}

/// the land event: a legendary land that entered after Yoshimaru did (a higher permanent number, so a land that was
/// already there when he came in, or while he was phased out, is not counted). The land need not still be there when
/// the trigger resolves: the counter does not depend on it (it left in response, or while the trigger waited)
fn yoshimaru_land(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m.0 <= src.0 || !live(g, src) || g.perm(src).owner != p || !legendary(g, m) {
        return Ok(());
    }
    counter(g, src, p)
}

/// the counter, after the trigger's window (the table may respond; a countered trigger does nothing)
fn counter(g: &mut Game, src: PermId, p: PlayerId) -> Res {
    if trigger_window(g, p, Some(src), "a +1/+1 counter on Yoshimaru", None)? && live(g, src) {
        repl::put_plus(g, src, 1);
    }
    Ok(())
}

// ================================================================== Minas Tirith
/// **Minas Tirith** (legendary land): enters tapped unless you control a legendary creature (the engine's enters rule,
/// from the card data). {T}: Add {W} (the `c=W` tag). {1}{W}, {T}: Draw a card. Activate only if you attacked with two
/// or more creatures this turn (two different creatures over the turn's combats, extra combats included: the player's
/// `attackers` list, not the size of the biggest single attack). The draw has no choice in it: the AI uses it with
/// spare mana after combat (and not with fewer than four cards left in its library); a person has it always listed,
/// with the reason when it can't be used yet, among the land's abilities (`human::abilities::land_abilities`), paying
/// from their pool.
pub(crate) fn attacked_with_two(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    pl.attacking == Some(g.turn_stamp()) && pl.attackers.len() >= 2
}

fn minas_options(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none()
        || crate::human::is_human(g, p)
        || g.perm(l).tapped
        || !attacked_with_two(g, p)
        || g.player(p).library.len() < 4
        || !super::lands::can_pay_without(g, p, l, 1, "W")
    {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.5,
        label: "Minas Tirith: draw a card".into(),
        act: Some(Action::Plan { f: minas_go, arg: l.0 as i64 }),
    }])
}

fn minas_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = PermId(arg as u32);
    if !g.player(p).lands.contains(&l)
        || g.perm(l).tapped
        || !attacked_with_two(g, p)
        || !super::lands::pay_without(g, p, l, 1, "W")?
    {
        return Ok(false);
    }
    g.perm_mut(l).tapped = true;
    crate::glog!(g, "  {} uses Minas Tirith", g.player(p).name);
    if ability_window_card(g, p, g.perm(l).card(), "draw a card", None, None)? {
        draw(g, p, 1, false)?;
    }
    Ok(true)
}

// ================================================================== The Seriema
/// **The Seriema** ({1}{W}{W}, legendary artifact, Spacecraft): when it enters, search for a legendary creature card
/// (the compiled `search`; a person picks the card). Station (tap another creature you control: put charge counters
/// equal to its power on it; only as a sorcery, with no maximum). At 7 or more charge counters it is a 5/5 artifact
/// creature with flying, and other tapped legendary creatures you control have indestructible. Being a creature is a
/// condition on the counters, not a one-time change: the card's `station=7` tag (data/card_overrides.toml) is read by
/// `chars::is_creature`, whatever put the counters there or took them away; the flying and indestructible depend on
/// the counters alone (`seven_plus`), not on being a creature by some other effect. The printed 5/5 is the
/// permanent's own power and toughness, set as it enters (`seriema_as_enters`), and when an existing permanent becomes
/// a copy of it without entering (Mirage Mirror, Shifting Woodland: `alela::set_face`), so a later base-setting effect
/// wins.
/// The AI stations with creatures that can't attack anyway, or when the tap finishes it, and no more once it is a
/// creature; a person chooses each creature to tap, one activation at a time, with or without 7 counters.
const SERIEMA: &str = "The Seriema";
const SERIEMA_PT: (i32, i32) = (5, 5);
const STATION_GOAL: i64 = 7;
const WITCH: &str = "Witch Enchanter // Witch-Blessed Meadow";

/// When The Seriema enters: search for a legendary creature card, reveal it, put it into your hand, then shuffle. The
/// searcher chooses the card, unless an Opposition Agent is out: then the Agent's controller chooses it (a person is
/// asked, the AI takes the card it values most) and takes it (`agent_take`). A person may find nothing.
fn seriema_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || !live(g, src) || g.perm(src).owner != p {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "search for a legendary creature", None)? {
        return Ok(());
    }
    let agent = agent_for(g, p);
    let chooser = agent.unwrap_or(p);
    let mut cands: Vec<CardId> = vec![];
    for c in searchable(g, p) {
        let d = g.db.get(c);
        if d.creature && d.tag(Tag::Leg) && !cands.iter().any(|&x| g.db.get(x).name == d.name) {
            cands.push(c);
        }
    }
    cands.sort_by(|&a, &b| g.db.get(a).name.cmp(&g.db.get(b).name));
    let found = if cands.is_empty() {
        None
    } else if crate::human::is_human(g, chooser) {
        let labels: Vec<String> = cands.iter().map(|&c| crate::human::choices::card_label(g, c)).collect();
        let prompt = if agent.is_some() {
            format!(
                "The Seriema: search {}'s library for a legendary creature card (Opposition Agent)",
                g.player(p).name
            )
        } else {
            "The Seriema: search for a legendary creature card".to_string()
        };
        crate::human::choices::choose(g, chooser, "search", &prompt, &labels, Some("find nothing"), None)?
            .map(|i| cands[i])
    } else if agent.is_some() {
        max_by(&cands, |c| card_worth(g, chooser, c, false) * 1000.0 + g.db.get(c).cmc as f64)
    } else {
        let f = Filter { kind: Some("creature".into()), ..Filter::default() };
        crate::ai::dsl_search_pick(g, p, &f, &cands)
            .or_else(|| max_by(&cands, |c| card_worth(g, p, c, false) * 1000.0 + g.db.get(c).cmc as f64))
    };
    if let Some(c) = found {
        let lib = &mut g.player_mut(p).library;
        if let Some(i) = lib.iter().position(|&x| x == c) {
            lib.remove(i);
        }
    }
    shuffle_library(g, p);
    let Some(c) = found else { return Ok(()) };
    if let Some(a) = agent {
        agent_take(g, a, p, c); // Opposition Agent
        return Ok(());
    }
    g.player_mut(p).hand.push(c);
    g.player_mut(p).seen.insert(c); // revealed
    g.player_mut(p).stat("tutored", 1);
    crate::ai::rules::mark_tutored(g, p, c);
    crate::engine::tutors::log_found(g, p, c, "hand", SERIEMA);
    Ok(())
}

/// as The Seriema enters: its printed 5/5 (the card data has 0/0, an artifact's). It is the permanent's own base
/// power and toughness, so a later effect that sets them (Darksteel Mutation, Kenrith's Transformation, Tezzeret's
/// emblem) overrides it; it only counts while it is a creature (at 7 or more charge counters)
fn seriema_as_enters(g: &mut Game, _p: PlayerId, m: PermId) -> Res {
    let x = g.perm_mut(m);
    if x.exc.pt.is_none() {
        (x.pow, x.tgh) = SERIEMA_PT;
    }
    Ok(())
}

/// The Seriema's printed power and toughness (its card data has 0/0, an artifact's): set as it enters
/// (`seriema_as_enters`) and when an existing permanent becomes a copy of it without entering (`alela::set_face`:
/// Mirage Mirror, Shifting Woodland; `set_face` keeps the P/T an animation already set). None for every other card.
pub(crate) fn printed_pt(g: &Game, cd: CardId) -> Option<(i32, i32)> {
    let d = g.db.get(cd);
    (d.tag(Tag::Station) && &*d.name == SERIEMA).then_some(SERIEMA_PT)
}

fn charge(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.int(DataKey::Charge)
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

/// the "7+" level of the station card: it depends only on the charge counters (CR 721.2b), not on whether some other
/// effect (Tezzeret's emblem) made it a creature
fn seven_plus(g: &Game, src: PermId) -> bool {
    crate::engine::chars::station_met(g, g.perm(src))
}

/// flying at 7+, and indestructible for the other tapped legendary creatures its controller has
fn seriema_grant(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    if !seven_plus(g, src) {
        return false;
    }
    if m == src {
        return kw == "flying";
    }
    kw == "indestructible"
        && g.perm(m).owner == g.perm(src).owner
        && g.perm(m).tapped
        && g.is_creature(m)
        && legendary(g, m)
}

fn seriema_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    // only as a sorcery: not offered in combat, on another player's turn or in response
    if post.is_none() || g.perm(src).owner != p || !live(g, src) || crate::human::legal::sorcery_timing(g, p).is_some()
    {
        return Ok(vec![]);
    }
    let cands = station_candidates(g, p, src);
    if cands.is_empty() {
        return Ok(vec![]);
    }
    if crate::human::is_human(g, p) {
        return Ok(vec![Opt {
            utility: 2.0,
            label: "The Seriema: station (tap another creature)".into(),
            act: Some(Action::Ability { src, f: station_go, arg: -1 }),
        }]);
    }
    if seven_plus(g, src) {
        return Ok(vec![]); // the AI has nothing more to gain from it
    }
    // the AI: creatures that can't attack this turn first (the biggest), the rest only when the taps reach 7 at once
    let have = charge(g, src);
    let idle = |g: &Game, m: PermId| g.perm(m).sick || g.perm(m).noatk;
    let mut best: Option<(PermId, f64)> = None;
    let total: i64 = cands.iter().map(|&m| epow(g, m).max(0) as i64).sum();
    let finish = have + total >= STATION_GOAL;
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
        .map(|(m, u)| Opt {
            utility: u,
            label: format!("The Seriema: station (tap {})", g.perm(m).name),
            act: Some(Action::Ability { src, f: station_go, arg: m.0 as i64 }),
        })
        .into_iter()
        .collect())
}

fn station_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    // only as a sorcery
    if !live(g, src) || g.perm(src).owner != p || crate::human::legal::sorcery_timing(g, p).is_some() {
        return Ok(false);
    }
    let cands = station_candidates(g, p, src);
    let t = if arg >= 0 {
        PermId(arg as u32)
    } else {
        let labels: Vec<String> =
            cands.iter().map(|&m| format!("{} (power {})", g.perm(m).name, epow(g, m).max(0))).collect();
        match crate::human::choices::choose(
            g,
            p,
            "choose",
            "The Seriema: tap which creature to put charge counters equal to its power on it?",
            &labels,
            Some("cancel"),
            None,
        )? {
            Some(i) => cands[i],
            None => return Ok(false),
        }
    };
    if !cands.contains(&t) {
        return Ok(false);
    }
    g.perm_mut(t).tapped = true; // the cost
    crate::glog!(g, "  {} stations The Seriema, tapping {}", g.player(p).name, g.perm(t).name);
    let what = format!("station: charge counters equal to the power of {}", g.perm(t).name);
    crate::engine::stack::ability_run(g, p, Some(src), None, &what, None, None, |g| {
        if !live(g, src) {
            return Ok(());
        }
        // the power as the ability resolves (a pump in response counts); if the creature has left, as it last
        // was: a departed permanent keeps its own pump and the anthems still out
        let n = epow(g, t).max(0);
        let k = repl::counters(g, p, repl::On::Perm(src), repl::Ctr::Other, n, repl::Path::ALL).max(0);
        let total = charge(g, src) + k as i64;
        g.perm_mut(src).data.set(DataKey::Charge, Val::Int(total));
        if seven_plus(g, src) {
            crate::glog!(g, "    The Seriema is a 5/5 artifact creature"); // by its `station=7` tag (chars.rs)
        }
        Ok(())
    })?;
    Ok(true)
}

// ================================================================== Witch Enchanter // Witch-Blessed Meadow
/// **Witch Enchanter** ({3}{W} 2/2 Human Warlock): when it enters, destroy target artifact or enchantment an opponent
/// controls (the compiled enters trigger; a person chooses the target). **Witch-Blessed Meadow** (land): "As this land
/// enters, you may pay 3 life. If you don't, it enters tapped." and {T}: Add {W}. The card data types the card as its
/// front face, so the engine's land play never takes it: it is offered as a land through `hand_options`, as
/// Shatterskull Smashing is (`t6_lands::mdfc_land_opt`), and taps for {W} through `land_cols`. As it enters as a land
/// its `land_etb` hook marks it `DataKey::LandFace`: it is then only a land (not a white Human Warlock creature) and
/// the creature face's compiled enters trigger does not fire. The AI plays it as a land only when short of lands (that
/// rule's heuristics) and pays the 3 life by the shared land-payment rule; a person may play it as their land drop any
/// time they could play a land, and answers the pay-3-life question (not asked without 3 life, or while their life
/// can't change). With Spelunking out the land enters untapped whatever is paid, so no life is paid and it is not
/// tapped (the controller's other legal order, tapped, is not offered: nothing rewards it); under Blood Moon it is a
/// Mountain without the ability, so nothing is paid or asked; it is still the land face (colourless, not a creature:
/// the face is not an ability, `moon_land_face`). The land face is playable from the hand only (a play permission from
/// exile or the graveyard does not offer it: an engine limit shared with every modal land).
fn meadow_cols(_g: &Game, _p: PlayerId, _l: PermId) -> Colors {
    Colors::from_letters("W")
}

/// as the land enters (before landfall and the land-enters event): it is the land face
fn meadow_land_etb(g: &mut Game, _p: PlayerId, l: PermId) -> Res {
    g.perm_mut(l).data.set(DataKey::LandFace, Val::Bool(true));
    Ok(())
}

/// the face a modal card entered as is a characteristic, not an ability: Blood Moon makes Witch-Blessed Meadow a
/// Mountain but does not turn it back into Witch Enchanter. cardcode::land_etb runs no land's enters hook under Blood
/// Moon and calls this instead (a no-op for every other land)
pub(crate) fn moon_land_face(g: &mut Game, l: PermId) {
    if &*g.db.get(g.perm(l).card()).name == WITCH {
        g.perm_mut(l).data.set(DataKey::LandFace, Val::Bool(true));
    }
}

fn meadow_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !crate::human::is_human(g, p) {
        // the AI's rule for when to play such a card as a land, but this card's own land play (`meadow_go`)
        let opt = super::t6_lands::mdfc_land_opt(g, c, p, post);
        return Ok(opt
            .map(|o| Opt { act: Some(Action::Plan { f: meadow_go, arg: c.0 as i64 }), ..o })
            .into_iter()
            .collect());
    }
    let pl = g.player(p);
    let allowed = 1 + crate::cardcode::extra_lands(g, p) + pl.extra_land_now;
    if post.is_none() || g.active != Some(p) || pl.lands_played >= allowed || !pl.hand.contains(&c) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.5,
        label: "play Witch Enchanter as a land (Witch-Blessed Meadow)".into(),
        act: Some(Action::Plan { f: meadow_go, arg: c.0 as i64 }),
    }])
}

fn meadow_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let human = crate::human::is_human(g, p);
    let allowed = 1 + crate::cardcode::extra_lands(g, p) + g.player(p).extra_land_now;
    let Some(i) = g.player(p).hand.iter().position(|&x| x == c) else { return Ok(false) };
    if g.player(p).lands_played >= allowed || (human && crate::human::legal::sorcery_timing(g, p).is_some()) {
        return Ok(false);
    }
    g.player_mut(p).hand.remove(i);
    // "As this land enters, you may pay 3 life": only with the life to pay (not while the player's life can't change),
    // and not with Spelunking out (the land enters untapped whatever is paid, and the controller may order the two
    // replacements that way) or under Blood Moon (a Mountain has no such ability of its own: nothing is paid or asked)
    let free = super::t12_windgrace::lands_enter_untapped(g, p) || crate::engine::mana::moon_card(g, c);
    let can_pay = !g.player(p).life_locked;
    let paid = !free
        && can_pay
        && if human {
            g.player(p).life >= 3
                && crate::human::choices::yes_no(g, p, "Witch-Blessed Meadow: pay 3 life so it enters untapped?")?
        } else {
            crate::engine::turn::pays_for_land(g, p, c, 3)
        };
    if paid {
        lose_life(g, p, 3, Some(p), "other", None)?;
    }
    crate::engine::turn::play_land_card(g, p, c, "plays")?;
    if let Some(&l) = g.player(p).lands.last().filter(|&&l| g.perm(l).card() == c) {
        if !paid && !free {
            g.perm_mut(l).tapped = true;
        }
        super::t6_lands::traitors_check(g, p, l)?;
    }
    crate::engine::life::check_state(g)?; // paying the life to 0 is lost at once
    Ok(true)
}

// ================================================================== Gandalf the White
const GANDALF: &str = "Gandalf the White";

/// **Gandalf the White** ({3}{W}{W} 4/5, Flash): "You may cast legendary spells and artifact spells as though they had
/// flash." A live Gandalf you control, not phased out and with abilities on, grants it for casting only (a land is
/// never cast: `cast::grants_flash` answers false for lands first). The second ability, one additional time for a
/// triggered ability caused by a legendary permanent or an artifact entering or leaving, is `gandalf_copies`.
pub fn gandalf_flash(g: &Game, p: PlayerId, c: CardId) -> bool {
    let d = g.db.get(c);
    (d.tag(Tag::Leg) || d.types.has(Types::ARTIFACT))
        && g.player(p).perms.iter().any(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && !x.neutered && card_name(g, m) == Some(GANDALF)
        })
}

/// **Gandalf the White**, second ability: "If a legendary permanent or an artifact entering or leaving the battlefield
/// causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time." x is
/// the permanent that entered or left as it last existed ("etb": entered; "dies": put into a graveyard from the
/// battlefield; "leaves": left any other way; "landfall": a land entered). It need not be yours: the ability is. None
/// is an artifact token that is a counter on the player (Treasure, Food, Clue): ("etb", None) entering and
/// ("dies", None) put into a graveyard from the battlefield (sacrificed). A cast, an attack and the like are not
/// entering or leaving. The caller checks that Gandalf is live (`hooked`).
fn gandalf_copies(g: &Game, src: Src, p: PlayerId, kind: Sym, x: Option<PermId>) -> i32 {
    if p != g.perm(src).owner || !matches!(kind, "etb" | "landfall" | "dies" | "leaves") {
        return 0;
    }
    match x {
        None => matches!(kind, "etb" | "dies") as i32,
        // as it last existed: what a permanent that has left kept of its being legendary or an artifact
        Some(m) => match g.perm(m).data.get(DataKey::LkiLegArt) {
            Some(Val::Bool(b)) if !g.perm(m).on_bf => *b as i32,
            _ => (legendary(g, m) || is_artifact_perm(g, m)) as i32,
        },
    }
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Yoshimaru, Ever Faithful")?;
    c.etb = Some(yoshimaru_etb);
    c.land_enters = Some(yoshimaru_land);
    r.card(db, GANDALF)?.trigger_copies = Some(gandalf_copies);
    r.card(db, "Minas Tirith")?.land_options = Some(minas_options);
    let w = r.card(db, WITCH)?;
    w.land_cols = Some(meadow_cols);
    w.land_etb = Some(meadow_land_etb);
    w.hand_options = Some(meadow_options);
    let c = r.card(db, SERIEMA)?;
    c.etb = Some(seriema_etb);
    c.as_enters = Some(seriema_as_enters);
    c.options = Some(seriema_options);
    c.grant_kw = Some(seriema_grant);
    Ok(())
}
