//! Python's play/abilities.py. Ported by the practice-cards task.
//! Activated abilities of your decks' cards that the engine keeps in deck AI code rather than in card hooks (the AI
//! uses them through its own plans). For the human seat they are abilities like any other: listed on the permanent or
//! land, checked for timing, tapping and mana, then carried out. Each effect returns None or the reason it can't.
//!
//! Python lists `(label, fn)`; here an `Ability<T>` (T is the `PermId` or `PermId` the ability belongs to). `act`
//! returns `Ok(None)` when it was carried out (or the person cancelled) and `Ok(Some(why))` when it can't be done.

use super::{Target, choices, legal, mana};
use crate::engine::{self, stack, zones};
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Step, Val};
use crate::tag::Tag;

/// the reason an activation can't be done, or None (done, or cancelled)
pub type Why = Option<String>;
pub type Act<T> = Box<dyn Fn(&mut Game, PlayerId, T) -> Res<Why>>;

/// one activated ability the person can use: its label in the list, and what activating it does
pub struct Ability<T> {
    pub label: String,
    pub act: Act<T>,
}

pub type PermAbility = Ability<PermId>;
pub type LandAbility = Ability<PermId>;

pub fn ability<T>(label: impl Into<String>, f: impl Fn(&mut Game, PlayerId, T) -> Res<Why> + 'static) -> Ability<T> {
    Ability { label: label.into(), act: Box::new(f) }
}

fn name(g: &Game, c: CardId) -> String {
    g.db.get(c).name.to_string()
}

fn tap_ok(g: &Game, m: PermId, creature: bool) -> Why {
    let x = g.perm(m);
    if x.tapped {
        return Some(format!("{} is tapped.", x.name));
    }
    if creature && x.sick {
        return Some(format!("{} has summoning sickness (it came under your control this turn).", x.name));
    }
    None
}

fn land_tap_ok(g: &Game, l: PermId) -> Why {
    if g.perm(l).tapped {
        Some(format!("{} is tapped.", name(g, g.perm(l).card())))
    } else if g.perm(l).sick && g.is_creature(l) {
        Some(format!("{} is a summoning-sick creature: it can't use a {{T}} ability.", name(g, g.perm(l).card())))
    } else {
        None
    }
}

fn pay(g: &mut Game, p: PlayerId, gn: u32, pips: &str, what: &str) -> Why {
    mana::pay_from_pool(g, p, gn, pips).map(|why| format!("Can't activate {what}. {why}"))
}

// ------------------------------------------------------------------ permanents
/// abilities.archivist: Jace's Archivist: {U}, {T}: each player discards their hand, then draws as many as the most
/// discarded
pub fn archivist(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, true).or_else(|| pay(g, p, 0, "U", "Jace's Archivist")) {
        return Ok(Some(why));
    }
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "  {} activates Jace's Archivist", g.player(p).name);
    stack::ability_run_w(g, p, Some(m), None, "each player discards their hand and draws", None, None, 0.0, |g, _| {
        let n = g.players.iter().filter(|q| q.alive).map(|q| engine::values::hand_count(g, q.id)).max().unwrap_or(0);
        crate::glog!(g, "    everyone discards and draws {}", n);
        let ids: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
        for &q in &ids {
            if g.player(q).alive {
                let hand = g.player(q).hand.clone();
                zones::discard_cards(g, q, &hand)?;
            }
        }
        for &q in &ids {
            if g.player(q).alive {
                zones::draw(g, q, n as u32, false)?;
            }
        }
        engine::life::check_state(g)
    })?;
    Ok(None)
}

/// abilities.assault: Aggravated Assault: {3}{R}{R}: untap your creatures; after this main phase, an additional
/// combat (followed by a main phase). Sorcery speed
pub fn assault(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = legal::sorcery_timing(g, p) {
        return Ok(Some(why.replace("do that", "activate Aggravated Assault")));
    }
    if let Some(why) = pay(g, p, 3, "RR", "Aggravated Assault") {
        return Ok(Some(why));
    }
    stack::ability_run(g, p, Some(m), None, "untap, an additional combat", Some(7.0), None, |g| {
        for x in g.player(p).perms.clone() {
            if g.is_creature(x) {
                g.perm_mut(x).tapped = false;
            }
        }
        crate::glog!(g, "  {} activates Aggravated Assault: creatures untap, an extra combat follows", g.player(p).name);
        if g.step == Step::Main1 {
            g.player_mut(p).extra_combats += 1; // it comes right after this turn's combat
        } else {
            engine::combat::combat(g, p)?; // main 2: the extra combat now, then this main phase continues
        }
        Ok(())
    })?;
    Ok(None)
}

/// abilities.tortured: Tortured Existence: {B}, discard a creature card: return target creature card from your
/// graveyard to your hand
pub fn tortured(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    let tg: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    let fodder: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    if tg.is_empty() {
        return Ok(Some("There is no creature card in your graveyard to return.".into()));
    }
    if fodder.is_empty() {
        return Ok(Some("You have no creature card in hand to discard.".into()));
    }
    if let Some(why) = mana::cost_problem(g, p, 0, "B") {
        return Ok(Some(format!("Can't activate Tortured Existence. {why}")));
    }
    let labels: Vec<String> = tg.iter().map(|&c| choices::card_label(g, c)).collect();
    let Some(k) = choices::choose(
        g,
        p,
        "target",
        "Tortured Existence: return which creature card?",
        &labels,
        Some("cancel"),
        None,
    )?
    else {
        return Ok(None); // targets first, then the costs
    };
    mana::pay_from_pool(g, p, 0, "B");
    let d = choices::pick_cards(g, p, &fodder, 1, "Tortured Existence: discard a creature card", "choose")?[0];
    zones::discard_cards(g, p, &[d])?;
    let c = tg[k];
    stack::ability_run_w(g, p, Some(m), None, &format!("return {} to hand", name(g, c)), None, None, 0.0, |g, is_copy| {
        let mut c = c;
        if is_copy {
            // "you may choose a new target for the copy"
            let now: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&x| g.db.get(x).creature).collect();
            let labels: Vec<String> = now.iter().map(|&x| choices::card_label(g, x)).collect();
            let prompt = "Tortured Existence (copy): return which creature card?";
            if let Some(j) = choices::choose(g, p, "target", prompt, &labels, Some("keep the target"), None)? {
                c = now[j];
            }
        }
        if let Some(i) = g.player(p).gy.iter().position(|&x| x == c) {
            g.player_mut(p).gy.remove(i);
            g.player_mut(p).hand.push(c);
        }
        crate::glog!(g, "  {} uses Tortured Existence: {} for {}", g.player(p).name, name(g, d), name(g, c));
        Ok(())
    })?;
    Ok(None)
}

/// abilities.loop_abilities: Sephiroth's infinite loops as a table shortcut: on each piece of a loop that is
/// assembled, 'start the loop' (the engine plays it out as its end result, after a window for opponents to answer a
/// key piece)
fn loop_abilities(g: &Game, p: PlayerId, m: PermId) -> Vec<PermAbility> {
    use crate::impls::mine;
    let mut out = vec![];
    let Some(cd) = g.perm(m).cd else { return out };
    let nm = name(g, cd);
    out.extend(super::combos::yawg_loop_abilities(g, p, m)); // Mikaeus + Yawgmoth + two undying creatures
    out.extend(super::combos::niv_abilities(g, m));
    out.extend(super::urdragon::loop_abilities(g, p, m)); // the Ur-Dragon deck's three loops
    if !mine::loop_card(&nm) {
        return out;
    }
    for (i, keys, kills) in mine::loops_now(g, p) {
        let l = &mine::LOOPS[i];
        if !l.groups.iter().any(|grp| grp.contains(&nm.as_str())) && !mine::OUTLETS.contains(&nm.as_str()) {
            continue;
        }
        let label =
            format!("start the loop: {}{}", l.name, if kills { "" } else { " (no payoff on the battlefield yet)" });
        out.push(ability(label, move |g: &mut Game, p: PlayerId, _m: PermId| {
            if i == 0 {
                return super::combos::trisk_loop(g, p, &keys); // Mikaeus + Triskelion: the person picks what it hits
            }
            mine::run_loop(g, p, i, &keys, kills)?;
            Ok(None)
        }));
    }
    out
}

/// the Basalt Monolith and the Rings of Brighthearth p controls (on the battlefield, not phased out)
fn basalt_rings(g: &Game, p: PlayerId) -> Option<(PermId, PermId)> {
    let find = |n: &str| {
        g.player(p).perms.iter().copied().find(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && !x.neutered && &*x.name == n
        })
    };
    Some((find("Basalt Monolith")?, find("Rings of Brighthearth")?))
}

/// Basalt Monolith + Rings of Brighthearth can start now: {5} in all (the same test as `combos::power_artifact`): the
/// Basalt untapped plus {2} more, or {5} from elsewhere when it is tapped
fn basalt_loop_ready(g: &Game, p: PlayerId) -> Option<(PermId, PermId)> {
    let (b, r) = basalt_rings(g, p)?;
    (mana::available(g, p) >= 5 && !crate::cardcode::ability_locked(g, b, p)).then_some((b, r))
}

/// how much {C} the person takes out of the loop, the largest first
const LOOP_AMOUNTS: [u32; 7] = [999, 500, 200, 100, 50, 20, 10];

/// Basalt Monolith and Rings of Brighthearth: unbounded colourless mana as a table shortcut. Basalt taps for {C}{C}{C};
/// its {3} untap, copied by Rings for {2}, untaps it twice and it taps twice more: {6} for {5}, one more each time
/// round. So: pay the {5} to start, give the table its window (an ability on the stack, the Stifle-type answers, and
/// removal on a piece), then take the {C} the person asks for, Basalt left tapped as the real loop leaves it.
fn basalt_loop(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Why> {
    let Some((b, r)) = basalt_loop_ready(g, p) else {
        return Ok(Some(
            "The loop needs Basalt Monolith and Rings of Brighthearth, and {5} in all to start it.".into(),
        ));
    };
    if !mana::pay_tax_ask(g, p, 5, "the Basalt Monolith loop", None)? {
        return Ok(Some("Can't pay the {5} to start the loop.".into()));
    }
    crate::glog!(g, "  {} starts the loop: Basalt Monolith and Rings of Brighthearth", g.player(p).name);
    let live = |g: &Game, m: PermId| g.perm(m).on_bf && !g.perm(m).phased;
    if !stack::ability_window(g, p, Some(b), "start the loop: infinite colourless mana", Some(10.0), None)?
        || !live(g, b)
        || !live(g, r)
        || crate::ai::combo_interrupted(g, p, "basalt", &[b, r])?
    {
        crate::glog!(g, "    ...the loop is stopped");
        return Ok(None);
    }
    let amounts: Vec<u32> = LOOP_AMOUNTS.to_vec();
    let labels: Vec<String> = amounts.iter().map(|n| format!("{n} {{C}}")).collect();
    let k = choices::choose_one(g, p, "choose", "The loop runs: how much {C} do you want?", &labels)?;
    let n = amounts[k];
    g.player_mut(p).pool.add('C', n);
    g.perm_mut(b).tapped = true;
    crate::glog!(g, "    the loop makes {n} {{C}}; Basalt Monolith ends tapped");
    Ok(None)
}

/// Staff of Domination: {1}: untap it, {5}, {T}: draw a card, again and again with mana to spare
fn staff_draw(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    let abil = crate::dsl::human_abilities(g, p, m);
    let find = |pre: &str, post: &str| abil.iter().find(|a| a.0.starts_with(pre) && a.0.ends_with(post)).map(|a| a.1);
    let (Some(untap), Some(draw)) = (find("{1}:", "untap"), find("{5}, {T}:", "draw")) else {
        return Ok(Some("Staff of Domination's abilities aren't available.".into()));
    };
    let room = g.player(p).library.len().saturating_sub(1) as u32; // never the last card
    if room == 0 {
        return Ok(Some("Your library is nearly empty.".into()));
    }
    let xs: Vec<u32> = choices::amount_choices(room).into_iter().filter(|&x| x >= 1).collect();
    let labels: Vec<String> = xs.iter().map(|x| format!("draw {x} card{}", if *x == 1 { "" } else { "s" })).collect();
    let Some(i) = choices::choose(
        g,
        p,
        "choose",
        "Staff of Domination: draw how many cards ({1} untap, then {5}, {T} each time)?",
        &labels,
        Some("cancel"),
        None,
    )?
    else {
        return Ok(None);
    };
    let n = xs[i];
    let auto = if n > 1 && !stack::rings_of(g, p).is_empty() {
        Some(choices::yes_no(
            g,
            p,
            "Rings of Brighthearth: pay {2} to copy each draw ability (two cards for each {5}, {T})?",
        )?)
    } else {
        None
    };
    let mut drawn = 0u32;
    while drawn < n && !g.over && g.player(p).library.len() > 1 {
        if g.perm(m).tapped && !stack::with_rings_auto(Some(false), || crate::dsl::activate(g, p, m, untap))? {
            break;
        }
        let before = g.player(p).library.len();
        // (a single draw asks Rings' question as it is; the last draw of a longer run is never copied: it would draw
        // one card too many)
        let ans = if n > 1 && n - drawn < 2 { Some(false) } else { auto };
        if !stack::with_rings_auto(ans, || crate::dsl::activate(g, p, m, draw))? {
            break;
        }
        drawn += before.saturating_sub(g.player(p).library.len()) as u32;
    }
    if drawn == 0 {
        return Ok(Some("Can't activate Staff of Domination: it costs {1} to untap and {5} to draw.".into()));
    }
    crate::glog!(
        g,
        "    {} drew {drawn} card{} with Staff of Domination",
        g.player(p).name,
        if drawn == 1 { "" } else { "s" }
    );
    Ok(None)
}

/// abilities.permanent_abilities: the abilities of permanent m
pub fn permanent_abilities(g: &Game, p: PlayerId, m: PermId) -> Vec<PermAbility> {
    let Some(cd) = g.perm(m).cd else { return vec![] };
    let d = g.db.get(cd);
    let mut out = vec![];
    if d.tag(Tag::Archivist) {
        out.push(ability("{U}, {T}: each player discards their hand and draws that many", archivist));
    }
    if d.tag(Tag::Assault) {
        out.push(ability("{3}{R}{R}: untap your creatures; an additional combat (sorcery speed)", assault));
    }
    if d.tags.str(Tag::Fill) == Some("tortured") {
        out.push(ability(
            "{B}, discard a creature card: return a creature card from your graveyard to your hand",
            tortured,
        ));
    }
    out.extend(loop_abilities(g, p, m));
    // Basalt Monolith + Rings of Brighthearth: the infinite-mana shortcut on each piece
    if matches!(&*d.name, "Basalt Monolith" | "Rings of Brighthearth") && basalt_loop_ready(g, p).is_some() {
        out.push(ability("start the loop: infinite colourless mana", basalt_loop));
    }
    if &*d.name == "Staff of Domination" {
        out.push(ability("draw N cards: untap for {1}, draw for {5} and tapping it, again and again", staff_draw));
    }
    if &*d.name == "Urza, Lord High Artificer" {
        out.push(ability("{5}: shuffle your library, then play the top card free this turn", urza_five));
    }
    out
}

// ------------------------------------------------------------------ lands
/// abilities.passage: Rogue's Passage: {4}, {T}: target creature can't be blocked this turn
fn passage(g: &mut Game, p: PlayerId, l: PermId) -> Res<Why> {
    if let Some(why) = land_tap_ok(g, l) {
        return Ok(Some(why));
    }
    if let Some(why) = mana::cost_problem(g, p, 4, "") {
        return Ok(Some(format!("Can't activate Rogue's Passage. {why}")));
    }
    let cre: Vec<PermId> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| {
            g.is_creature(m) && !g.perm(m).phased && !(g.perm(m).owner != p && engine::values::untargetable(g, m))
        })
        .collect();
    if cre.is_empty() {
        return Ok(Some("There is no creature to target.".into()));
    }
    let labels: Vec<String> = cre.iter().map(|&x| legal::describe_target(g, p, Target::Perm(x))).collect();
    let Some(k) = choices::choose(
        g,
        p,
        "target",
        "Rogue's Passage: which creature can't be blocked this turn?",
        &labels,
        Some("cancel"),
        None,
    )?
    else {
        return Ok(None);
    };
    mana::pay_from_pool(g, p, 4, "");
    g.perm_mut(l).tapped = true;
    let m = cre[k];
    crate::glog!(g, "  {} activates Rogue's Passage: {} can't be blocked this turn", g.player(p).name, g.perm(m).name);
    let cd = g.perm(l).card();
    let nm = format!("{} can't be blocked", g.perm(m).name);
    stack::ability_run_w(g, p, None, Some(cd), &nm, None, Some(m), 0.0, |g, is_copy| {
        let mut t = m;
        if is_copy {
            // "you may choose a new target for the copy"
            let labels: Vec<String> = cre.iter().map(|&x| legal::describe_target(g, p, Target::Perm(x))).collect();
            let prompt = "Rogue's Passage (copy): which creature can't be blocked this turn?";
            if let Some(j) = choices::choose(g, p, "target", prompt, &labels, Some("keep the target"), None)? {
                t = cre[j];
            }
        }
        if g.perm(t).on_bf {
            let st = g.turn_stamp();
            g.perm_mut(t).data.set(DataKey::Unbl, Val::Stamp(st));
        }
        Ok(())
    })?;
    Ok(None)
}

/// abilities.grounds: Scavenger Grounds: {2}, {T}, sacrifice a Desert: exile all graveyards
fn grounds(g: &mut Game, p: PlayerId, l: PermId) -> Res<Why> {
    if let Some(why) = land_tap_ok(g, l) {
        return Ok(Some(why));
    }
    let deserts: Vec<PermId> =
        g.player(p).lands.iter().copied().filter(|&x| g.db.get(g.perm(x).card()).tag(Tag::Desert)).collect();
    if let Some(why) = pay(g, p, 2, "", "Scavenger Grounds") {
        return Ok(Some(why));
    }
    g.perm_mut(l).tapped = true;
    let d = if deserts.contains(&l) { l } else { deserts[0] }; // it is a Desert itself: sacrifice it (the usual choice)
    engine::turn::remove_land(g, p, d, Some(crate::engine::zones::Zone::Gy));
    crate::glog!(g, "  {} activates Scavenger Grounds", g.player(p).name);
    let cd = g.perm(l).card();
    stack::ability_run_w(g, p, None, Some(cd), "exile all graveyards", Some(9.0), None, 0.0, |g, _| {
        for q in g.players.iter().filter(|q| q.alive).map(|q| q.id).collect::<Vec<_>>() {
            let gy = std::mem::take(&mut g.player_mut(q).gy);
            g.player_mut(q).exile.extend(gy);
        }
        crate::glog!(g, "    all graveyards are exiled");
        Ok(())
    })?;
    Ok(None)
}

/// abilities.land_abilities: the abilities of land l (cards.py's first, then the Passage / Scavenger Grounds ones)
pub fn land_abilities(g: &Game, p: PlayerId, l: PermId) -> Vec<LandAbility> {
    let mut out = super::cards::land_abilities(g, p, l);
    out.extend(super::karn::land_abilities(g, p, l));
    out.extend(super::fetch::land_abilities(g, l));
    out.extend(super::cloud::land_abilities(g, l));
    out.extend(super::model_gaps::land_abilities(g, l));
    let d = g.db.get(g.perm(l).card());
    if d.tag(Tag::Passage) {
        out.push(ability("{4}, {T}: target creature can't be blocked this turn", passage));
    }
    if d.tag(Tag::Desert) && &*d.name == "Scavenger Grounds" {
        out.push(ability("{2}, {T}, sacrifice a Desert: exile all graveyards", grounds));
    }
    out
}

/// Urza, Lord High Artificer: {5}: shuffle your library, then exile the top card; you may play it free this turn
/// (the AI's resolution, `impls::t5::urza_top`; added 2026-11-04 when Urza joined the Ur-Dragon list)
fn urza_five(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if g.player(p).library.is_empty() {
        return Ok(Some("Your library is empty.".into()));
    }
    if let Some(why) = pay(g, p, 5, "", "Urza") {
        return Ok(Some(why));
    }
    crate::glog!(g, "  {} activates Urza, Lord High Artificer", g.player(p).name);
    crate::impls::t5::urza_top(g, m, p)?;
    Ok(None)
}
