//! Practice mode: every activated ability of the Karn deck's cards (JD, 10/21). Most of these were AI card code that
//! only offered an ability when the AI wanted it (a wipe worth it, a fate counter on its best permanent ...); the
//! person decides for themselves, so each is listed whenever the rules allow it, its cost is paid from the pool, and
//! the choices (targets, which artifacts to sacrifice, X, which card to take) are asked of the person. Where the
//! engine already has the effect (`impls/karn_cards.rs`, `karn_engines.rs`, `karn.rs`) the ability calls it.
//!
//! Cards the engine never modelled (Bonders' Enclave, Buried Ruin, Treasure Vault, Urza's Factory) are written here.
//! Pithing Needle does not stop a person (as for the other practice abilities).

use super::abilities::{LandAbility, PermAbility, Why, ability};
use super::{Target, cards, choices, legal, mana};
use crate::cards::{Colors, Types};
use crate::engine::stack::{ability_window, ability_window_card};
use crate::engine::values::{epow, untargetable};
use crate::engine::zones::{self, Tokens, die};
use crate::flow::Res;
use crate::hooks::Action;
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};

const PERMS: [&str; 17] = [
    "Grim Monolith",
    "Liquimetal Torque",
    "Mazemind Tome",
    "Mishra's Bauble",
    "Urza's Bauble",
    "Nevinyrral's Disk",
    "Oblivion Stone",
    "Ratchet Bomb",
    "Soul-Guide Lantern",
    "Steel Hellkite",
    "Steel Overseer",
    "Unstable Obelisk",
    "Kuldotha Forgemaster",
    "Walking Ballista",
    "Batterskull",
    "Endless Atlas",
    "Mishra's Factory",
];
const LANDS: [&str; 8] = [
    "Bonders' Enclave",
    "Buried Ruin",
    "Inventors' Fair",
    "Mishra's Factory",
    "Mishra's Foundry",
    "Treasure Vault",
    "Urza's Factory",
    "War Room",
];

/// A permanent whose abilities are all listed here (the AI's hook options for it are left out, so nothing shows twice).
pub fn owns_perm(g: &Game, m: PermId) -> bool {
    let n = &*g.perm(m).name;
    PERMS.contains(&n) || n == "Mishra's Foundry"
}

/// A land whose abilities are all listed here.
pub fn owns_land(g: &Game, l: LandId) -> bool {
    LANDS.contains(&&*g.db.get(g.land(l).cd).name)
}

// ------------------------------------------------------------------ helpers
fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn need(g: &Game, p: PlayerId, gn: u32, pips: &str, what: &str) -> Why {
    mana::cost_problem(g, p, gn, pips).map(|why| format!("Can't activate {what}. {why}"))
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

fn land_tap_ok(g: &Game, l: LandId) -> Why {
    if g.land(l).tapped { Some(format!("{} is tapped.", lname(g, l))) } else { None }
}

fn lname(g: &Game, l: LandId) -> String {
    g.db.get(g.land(l).cd).name.to_string()
}

/// the engine's own code for the ability, run for the person; Some(why) if it refused
fn run(g: &mut Game, p: PlayerId, a: Action, what: &str) -> Res<Why> {
    if !crate::ai::act::perform(g, p, &a)? {
        return Ok(Some(format!("{what}: that can't be done right now.")));
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

fn via_ability(g: &mut Game, p: PlayerId, src: PermId, f: crate::hooks::AbilityFn, arg: i64, what: &str) -> Res<Why> {
    run(g, p, Action::Ability { src, f, arg }, what)
}

/// every nonland permanent p may target (hexproof and shroud respected for the others')
fn targets(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| live(g, m) && !(g.perm(m).owner != p && untargetable(g, m)))
        .collect()
}

/// the mana in the pool (and floating), the most an X can be paid with
fn mana_in_pool(g: &Game, p: PlayerId) -> u32 {
    let f = g.player(p).floating;
    g.player(p).pool.total() + f.u + f.b + f.r + f.g + f.c + f.any
}

fn pick_n(g: &Game, p: PlayerId, what: &str, lo: u32, hi: u32) -> Res<Option<u32>> {
    if hi < lo {
        return Ok(None);
    }
    let xs: Vec<u32> = (lo..=hi).collect();
    let labels: Vec<String> = xs.iter().map(|x| format!("X = {x}")).collect();
    let prompt = format!("{what}: X = ? (your pool has {} mana)", mana_in_pool(g, p));
    Ok(choices::choose(g, p, "choose", &prompt, &labels, Some("cancel"), None)?.map(|k| xs[k]))
}

fn art_perm(g: &Game, m: PermId) -> bool {
    crate::impls::karn_cards::is_art(g, m)
}

fn assembly_worker(g: &Game, m: PermId) -> bool {
    live(g, m) && g.is_creature(m) && g.perm(m).ttypes.contains(&"assembly-worker")
}

// ------------------------------------------------------------------ permanents
/// the abilities of p's permanent m that this module lists (the animated Mishra's Factory / Foundry is a token)
pub fn perm_abilities(g: &Game, _p: PlayerId, m: PermId) -> Vec<PermAbility> {
    let x = g.perm(m);
    if !x.on_bf || x.phased {
        return vec![];
    }
    if x.cd.is_none() {
        if matches!(x.data.get(DataKey::Land), Some(Val::Land(_))) {
            return match &*x.name {
                "Mishra's Factory" => {
                    vec![ability("{T}: target Assembly-Worker gets +1/+1 until end of turn", factory_pump)]
                }
                "Mishra's Foundry" => vec![ability(
                    "{1}, {T}: target attacking Assembly-Worker gets +2/+2 until end of turn",
                    foundry_pump,
                )],
                _ => vec![],
            };
        }
        return vec![];
    }
    match &*x.name {
        "Grim Monolith" => vec![ability("{4}: untap Grim Monolith", monolith)],
        "Liquimetal Torque" => {
            vec![ability("{T}: target nonland permanent becomes an artifact until end of turn", torque)]
        }
        "Mazemind Tome" => vec![
            ability("{T}, put a page counter: scry 1", |g, p, m| tome(g, p, m, false)),
            ability("{2}, {T}, put a page counter: draw a card", |g, p, m| tome(g, p, m, true)),
        ],
        "Mishra's Bauble" => vec![ability(
            "{T}, sacrifice: look at the top card of target player's library; draw a card at the next upkeep",
            |g, p, m| bauble(g, p, m, false),
        )],
        "Urza's Bauble" => vec![ability(
            "{T}, sacrifice: look at a random card in target player's hand; draw a card at the next upkeep",
            |g, p, m| bauble(g, p, m, true),
        )],
        "Nevinyrral's Disk" => {
            vec![ability("{1}, {T}: destroy all artifacts, creatures and enchantments", disk)]
        }
        "Oblivion Stone" => vec![
            ability("{4}, {T}: put a fate counter on target permanent", stone_mark),
            ability(
                "{5}, {T}, sacrifice: destroy each nonland permanent without a fate counter, then remove all fate counters",
                stone_wipe,
            ),
        ],
        "Ratchet Bomb" => vec![
            ability("{T}: put a charge counter on Ratchet Bomb", bomb_tick),
            ability(
                format!(
                    "{{T}}, sacrifice: destroy each nonland permanent with mana value {} (its charge counters)",
                    crate::impls::karn_cards::charges(g, m)
                ),
                bomb_blow,
            ),
        ],
        "Soul-Guide Lantern" => vec![
            ability("{T}, sacrifice: exile each opponent's graveyard", lantern_exile),
            ability("{1}, {T}, sacrifice: draw a card", lantern_draw),
        ],
        "Steel Hellkite" => vec![
            ability("{2}: Steel Hellkite gets +1/+0 until end of turn", hellkite_pump),
            ability(
                "{X}: destroy each nonland permanent with mana value X whose controller was dealt combat damage by it this turn (once each turn)",
                hellkite_x,
            ),
        ],
        // {T}, pay 1 life: exile the top card (the AI uses it only on a land; you may on anything)
        "Mystic Forge" => vec![ability("{T}, pay 1 life: exile the top card of your library", forge)],
        "Steel Overseer" => {
            vec![ability("{T}: put a +1/+1 counter on each artifact creature you control", overseer)]
        }
        "Unstable Obelisk" => vec![ability("{7}, {T}, sacrifice: destroy target permanent", obelisk)],
        "Kuldotha Forgemaster" => vec![ability(
            "{T}, sacrifice three artifacts: search your library for an artifact card, put it onto the battlefield",
            forgemaster,
        )],
        "Walking Ballista" => vec![
            ability("{4}: put a +1/+1 counter on Walking Ballista", ballista_grow),
            ability("remove a +1/+1 counter: 1 damage to any target", ballista_ping),
        ],
        "Batterskull" => vec![ability("{3}: return Batterskull to its owner's hand", skull)],
        "Endless Atlas" => vec![ability("{2}, {T}: draw a card (three or more lands with the same name)", atlas)],
        _ => vec![],
    }
}

fn monolith(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if !g.perm(m).tapped {
        return Ok(Some("Grim Monolith is already untapped.".into()));
    }
    if let Some(why) = need(g, p, 4, "", "Grim Monolith") {
        return Ok(Some(why));
    }
    run(g, p, Action::Plan { f: crate::ai::decks::untap_monolith, arg: m.0 as i64 }, "Grim Monolith")
}

fn torque(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false) {
        return Ok(Some(why));
    }
    let tg: Vec<PermId> = targets(g, p).into_iter().filter(|&x| !g.perm(x).ttypes.contains(&"land")).collect();
    let Some(t) = cards::pick_perm(
        g,
        p,
        "target",
        "Liquimetal Torque: which permanent becomes an artifact?",
        &tg,
        Some("cancel"),
    )?
    else {
        return Ok(None);
    };
    via_ability(g, p, m, crate::impls::karn_engines::torque_go, t.0 as i64, "Liquimetal Torque")
}

fn tome(g: &mut Game, p: PlayerId, m: PermId, draw: bool) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false) {
        return Ok(Some(why));
    }
    if draw && let Some(why) = need(g, p, 2, "", "Mazemind Tome") {
        return Ok(Some(why));
    }
    via_ability(g, p, m, crate::impls::karn_engines::mazemind_go, draw as i64, "Mazemind Tome")
}

/// Mishra's Bauble / Urza's Bauble: {T}, sacrifice: look at the top card of a player's library (or a random card of
/// their hand); draw a card at the beginning of the next upkeep
fn bauble(g: &mut Game, p: PlayerId, m: PermId, hand: bool) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false) {
        return Ok(Some(why));
    }
    let who: Vec<PlayerId> =
        g.players.iter().filter(|q| q.alive && (!hand || !q.hand.is_empty())).map(|q| q.id).collect();
    if who.is_empty() {
        return Ok(Some("No player has a card in hand to look at.".into()));
    }
    let labels: Vec<String> = who.iter().map(|&q| legal::describe_target(g, p, Target::Player(q))).collect();
    let nm = g.perm(m).name.to_string();
    let prompt = format!(
        "{nm}: look at {} of which player?",
        if hand { "a random card in the hand" } else { "the top card of the library" }
    );
    let Some(k) = choices::choose(g, p, "target", &prompt, &labels, Some("cancel"), None)? else { return Ok(None) };
    let q = who[k];
    g.perm_mut(m).tapped = true;
    die(g, m, "sac")?;
    crate::glog!(g, "  {} sacrifices {} (draws at the next upkeep)", g.player(p).name, nm);
    let cd = g.perm(m).cd.unwrap();
    if !crate::engine::stack::ability_window_card(g, p, cd, "look at a card; draw at the next upkeep", None, None)? {
        return Ok(None);
    }
    let seen = if hand {
        let h = g.player(q).hand.clone();
        (!h.is_empty()).then(|| h[g.rng.below(h.len() as u64) as usize])
    } else {
        g.player(q).library.last().copied()
    };
    let qn = g.player(q).name.to_string();
    let text = match seen {
        Some(c) => format!(
            "{nm}: {}'s {} is {}. You draw a card at the beginning of the next upkeep.",
            qn,
            if hand { "random card" } else { "top card" },
            g.db.get(c).name
        ),
        None => format!("{nm}: {qn} has no card there. You draw a card at the beginning of the next upkeep."),
    };
    choices::choose_one(g, p, "choose", &text, &["OK".to_string()])?;
    g.player_mut(p).delayed_draws += 1;
    Ok(None)
}

fn disk(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false).or_else(|| need(g, p, 1, "", "Nevinyrral's Disk")) {
        return Ok(Some(why));
    }
    via_ability(g, p, m, crate::impls::karn_cards::disk_go, 0, "Nevinyrral's Disk")
}

fn stone_mark(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false).or_else(|| need(g, p, 4, "", "Oblivion Stone")) {
        return Ok(Some(why));
    }
    let tg = targets(g, p);
    let Some(t) = cards::pick_perm(
        g,
        p,
        "target",
        "Oblivion Stone: put a fate counter on which permanent?",
        &tg,
        Some("cancel"),
    )?
    else {
        return Ok(None);
    };
    via_ability(g, p, m, crate::impls::karn_cards::stone_mark, t.0 as i64, "Oblivion Stone")
}

fn stone_wipe(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false).or_else(|| need(g, p, 5, "", "Oblivion Stone")) {
        return Ok(Some(why));
    }
    via_ability(g, p, m, crate::impls::karn_cards::stone_wipe, 0, "Oblivion Stone")
}

fn bomb_tick(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false) {
        return Ok(Some(why));
    }
    via_ability(g, p, m, crate::impls::karn_cards::bomb_tick, 0, "Ratchet Bomb")
}

fn bomb_blow(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false) {
        return Ok(Some(why));
    }
    via_ability(g, p, m, crate::impls::karn_cards::bomb_blow, 0, "Ratchet Bomb")
}

fn lantern_exile(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false) {
        return Ok(Some(why));
    }
    g.perm_mut(m).tapped = true;
    die(g, m, "sac")?;
    let cd = g.perm(m).cd.unwrap();
    crate::glog!(g, "  {} sacrifices Soul-Guide Lantern", g.player(p).name);
    if ability_window_card(g, p, cd, "exile each opponent's graveyard", None, None)? {
        for q in g.opps(p).collect::<Vec<_>>() {
            let gy = std::mem::take(&mut g.player_mut(q).gy);
            g.player_mut(q).exile.extend(gy);
        }
        crate::glog!(g, "    each opponent's graveyard is exiled");
    }
    Ok(None)
}

fn lantern_draw(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false).or_else(|| need(g, p, 1, "", "Soul-Guide Lantern")) {
        return Ok(Some(why));
    }
    mana::pay_from_pool(g, p, 1, "");
    g.perm_mut(m).tapped = true;
    die(g, m, "sac")?;
    let cd = g.perm(m).cd.unwrap();
    crate::glog!(g, "  {} sacrifices Soul-Guide Lantern", g.player(p).name);
    if ability_window_card(g, p, cd, "draw a card", None, None)? {
        zones::draw(g, p, 1, false)?;
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

fn hellkite_pump(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = need(g, p, 2, "", "Steel Hellkite") {
        return Ok(Some(why));
    }
    mana::pay_from_pool(g, p, 2, "");
    crate::glog!(g, "  {} pumps Steel Hellkite (+1/+0)", g.player(p).name);
    if ability_window(g, p, Some(m), "+1/+0", None, None)? && live(g, m) {
        g.perm_mut(m).eot_pt.0 += 1;
    }
    Ok(None)
}

fn hellkite_x(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if g.perm(m).data.get(DataKey::Used) == Some(&Val::Stamp(g.turn_stamp())) {
        return Ok(Some("Steel Hellkite's {X} ability can be activated only once each turn.".into()));
    }
    if crate::impls::karn_cards::hit_players(g, m).is_empty() {
        return Ok(Some("Steel Hellkite hasn't dealt combat damage to a player this turn.".into()));
    }
    let hi = mana_in_pool(g, p);
    let Some(x) = pick_n(g, p, "Steel Hellkite", 0, hi)? else { return Ok(None) };
    if let Some(why) = need(g, p, x, "", "Steel Hellkite") {
        return Ok(Some(why));
    }
    via_ability(g, p, m, crate::impls::karn_cards::hellkite_go, x as i64, "Steel Hellkite")
}

fn forge(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false) {
        return Ok(Some(why));
    }
    if g.player(p).life <= 1 {
        return Ok(Some("You can't pay 1 life.".into()));
    }
    via_ability(g, p, m, crate::impls::karn_engines::forge_exile, 0, "Mystic Forge")
}

fn overseer(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, true) {
        return Ok(Some(why));
    }
    via_ability(g, p, m, crate::impls::karn_cards::overseer_go, 0, "Steel Overseer")
}

fn obelisk(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false).or_else(|| need(g, p, 7, "", "Unstable Obelisk")) {
        return Ok(Some(why));
    }
    let tg = targets(g, p);
    let Some(t) = cards::pick_perm(g, p, "target", "Unstable Obelisk: destroy which permanent?", &tg, Some("cancel"))?
    else {
        return Ok(None);
    };
    mana::pay_from_pool(g, p, 7, "");
    g.perm_mut(m).tapped = true;
    die(g, m, "sac")?;
    crate::glog!(g, "  {} activates Unstable Obelisk", g.player(p).name);
    if ability_window(g, p, Some(m), "destroy target permanent", Some(6.0), Some(t))?
        && live(g, t)
        && !(g.perm(t).owner != p && untargetable(g, t))
    {
        crate::engine::removal::apply_removal(g, Some(p), t, "destroy", None)?;
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

fn forgemaster(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, true) {
        return Ok(Some(why));
    }
    let mut arts: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&x| live(g, x) && art_perm(g, x)).collect();
    if arts.len() < 3 {
        return Ok(Some("You need three artifacts to sacrifice.".into()));
    }
    let mut fodder = vec![];
    for i in 0..3 {
        let prompt = format!("Kuldotha Forgemaster: sacrifice which artifact? ({} of 3)", i + 1);
        let Some(x) = cards::pick_perm(g, p, "choose", &prompt, &arts, Some("cancel"))? else { return Ok(None) };
        arts.retain(|&y| y != x);
        fodder.push(x);
    }
    g.perm_mut(m).tapped = true;
    for x in fodder {
        crate::glog!(g, "  {} sacrifices {} to Kuldotha Forgemaster", g.player(p).name, g.perm(x).name);
        die(g, x, "sac")?;
    }
    if ability_window(g, p, Some(m), "search for an artifact", Some(6.0), None)? {
        let art = |g: &Game, c: CardId| {
            let d = g.db.get(c);
            d.types.has(Types::ARTIFACT) && !d.land
        };
        let got = choices::search(g, p, &art, 1, "Kuldotha Forgemaster: search for an artifact card", true)?;
        for c in got {
            crate::glog!(g, "    Kuldotha Forgemaster fetches {}", g.db.get(c).name);
            zones::enter(g, p, c, zones::Enter::default())?;
        }
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

fn ballista_grow(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = need(g, p, 4, "", "Walking Ballista") {
        return Ok(Some(why));
    }
    mana::pay_from_pool(g, p, 4, "");
    crate::glog!(g, "  {} activates Walking Ballista (+1/+1 counter)", g.player(p).name);
    if ability_window(g, p, Some(m), "+1/+1 counter", None, None)? && live(g, m) {
        crate::cardcode::put_plus(g, m, 1);
    }
    Ok(None)
}

fn ballista_ping(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if g.perm(m).plus < 1 {
        return Ok(Some("Walking Ballista has no +1/+1 counter to remove.".into()));
    }
    let tg = choices::damage_targets(g, p, Colors::NONE);
    if tg.is_empty() {
        return Ok(Some("There is no target.".into()));
    }
    let labels: Vec<String> = tg.iter().map(|&x| legal::describe_target(g, p, x)).collect();
    let Some(k) =
        choices::choose(g, p, "target", "Walking Ballista: 1 damage to which target?", &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    let t = tg[k];
    g.perm_mut(m).plus -= 1; // the cost: it shrinks (and dies at 0/0) whatever happens to the ping
    g.bf_ver += 1;
    crate::glog!(g, "  {} removes a counter from Walking Ballista", g.player(p).name);
    let tp = if let Target::Perm(x) = t { Some(x) } else { None };
    let still = live(g, m);
    let ok = if still { ability_window(g, p, Some(m), "1 damage", None, tp)? } else { true };
    if ok {
        let alive = match t {
            Target::Player(q) => g.player(q).alive,
            Target::Perm(x) => live(g, x),
        };
        if alive {
            choices::apply_damage(g, p, t, 1, "Walking Ballista", "burn")?;
        }
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

fn skull(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = need(g, p, 3, "", "Batterskull") {
        return Ok(Some(why));
    }
    via_ability(g, p, m, crate::impls::karn_engines::skull_go, 0, "Batterskull")
}

fn atlas(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, false) {
        return Ok(Some(why));
    }
    if !crate::impls::karn_engines::atlas_ready(g, p) {
        return Ok(Some("Endless Atlas needs three or more lands with the same name.".into()));
    }
    if let Some(why) = need(g, p, 2, "", "Endless Atlas") {
        return Ok(Some(why));
    }
    via_ability(g, p, m, crate::impls::karn_engines::atlas_go, 0, "Endless Atlas")
}

/// {T} of Mishra's Factory (animated): target Assembly-Worker gets +1/+1 until end of turn
fn factory_pump(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, true) {
        return Ok(Some(why));
    }
    let tg: Vec<PermId> = targets(g, p).into_iter().filter(|&x| assembly_worker(g, x)).collect();
    let Some(t) =
        cards::pick_perm(g, p, "target", "Mishra's Factory: which Assembly-Worker gets +1/+1?", &tg, Some("cancel"))?
    else {
        return Ok(None);
    };
    g.perm_mut(m).tapped = true;
    pump(g, p, Some(m), None, t, (1, 1))
}

/// {1}, {T} of Mishra's Foundry (animated): target attacking Assembly-Worker gets +2/+2 until end of turn
fn foundry_pump(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = tap_ok(g, m, true).or_else(|| need(g, p, 1, "", "Mishra's Foundry")) {
        return Ok(Some(why));
    }
    let tg: Vec<PermId> =
        targets(g, p).into_iter().filter(|&x| assembly_worker(g, x) && g.in_combat.contains(&x)).collect();
    if tg.is_empty() {
        return Ok(Some("There is no attacking Assembly-Worker to target.".into()));
    }
    let prompt = "Mishra's Foundry: which attacking Assembly-Worker gets +2/+2?";
    let Some(t) = cards::pick_perm(g, p, "target", prompt, &tg, Some("cancel"))? else { return Ok(None) };
    mana::pay_from_pool(g, p, 1, "");
    g.perm_mut(m).tapped = true;
    pump(g, p, Some(m), None, t, (2, 2))
}

fn pump(g: &mut Game, p: PlayerId, src: Option<PermId>, card: Option<CardId>, t: PermId, by: (i32, i32)) -> Res<Why> {
    let what = format!("{} gets +{}/+{}", g.perm(t).name, by.0, by.1);
    let ok = match (src, card) {
        (Some(m), _) => ability_window(g, p, Some(m), &what, None, Some(t))?,
        (None, Some(c)) => ability_window_card(g, p, c, &what, None, Some(t))?,
        _ => true,
    };
    if ok && live(g, t) {
        g.perm_mut(t).eot_pt.0 += by.0;
        g.perm_mut(t).eot_pt.1 += by.1;
        g.bf_ver += 1;
        crate::glog!(g, "    {what} until end of turn");
    }
    Ok(None)
}

// ------------------------------------------------------------------ lands
/// the abilities of p's land l that this module lists
pub fn land_abilities(g: &Game, _p: PlayerId, l: LandId) -> Vec<LandAbility> {
    match &*lname(g, l) {
        "Bonders' Enclave" => {
            vec![ability("{3}, {T}: draw a card (you control a creature with power 4 or greater)", enclave)]
        }
        "Buried Ruin" => {
            vec![ability("{2}, {T}, sacrifice: return target artifact card from your graveyard to your hand", ruin)]
        }
        "Inventors' Fair" => vec![ability(
            "{4}, {T}, sacrifice: search your library for an artifact card, put it in your hand (three or more artifacts)",
            fair,
        )],
        "Mishra's Factory" => vec![
            ability("{1}: becomes a 2/2 Assembly-Worker artifact creature until end of turn", |g, p, l| {
                animate(g, p, l)
            }),
            ability("{T}: target Assembly-Worker gets +1/+1 until end of turn", |g, p, l| {
                land_pump(g, p, l, 0, (1, 1))
            }),
        ],
        "Mishra's Foundry" => vec![
            ability("{2}: becomes a 2/2 Assembly-Worker artifact creature until end of turn", |g, p, l| {
                animate(g, p, l)
            }),
            ability("{1}, {T}: target attacking Assembly-Worker gets +2/+2 until end of turn", |g, p, l| {
                land_pump(g, p, l, 1, (2, 2))
            }),
        ],
        "Treasure Vault" => vec![ability("{X}{X}, {T}, sacrifice: create X Treasure tokens", vault)],
        "Urza's Factory" => {
            vec![ability("{7}, {T}: create a 2/2 colorless Assembly-Worker artifact creature token", urza_factory)]
        }
        "War Room" => {
            vec![ability("{3}, {T}, pay life equal to the colors in your commander's identity: draw a card", war_room)]
        }
        _ => vec![],
    }
}

fn enclave(g: &mut Game, p: PlayerId, l: LandId) -> Res<Why> {
    if let Some(why) = land_tap_ok(g, l) {
        return Ok(Some(why));
    }
    let big = g.player(p).perms.iter().any(|&m| live(g, m) && g.is_creature(m) && epow(g, m) >= 4);
    if !big {
        return Ok(Some("Bonders' Enclave: you control no creature with power 4 or greater.".into()));
    }
    if let Some(why) = need(g, p, 3, "", "Bonders' Enclave") {
        return Ok(Some(why));
    }
    mana::pay_from_pool(g, p, 3, "");
    g.land_mut(l).tapped = true;
    crate::glog!(g, "  {} activates Bonders' Enclave", g.player(p).name);
    if ability_window_card(g, p, g.land(l).cd, "draw a card", None, None)? {
        zones::draw(g, p, 1, false)?;
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

fn ruin(g: &mut Game, p: PlayerId, l: LandId) -> Res<Why> {
    if let Some(why) = land_tap_ok(g, l) {
        return Ok(Some(why));
    }
    let arts: Vec<CardId> =
        g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).types.has(Types::ARTIFACT)).collect();
    if arts.is_empty() {
        return Ok(Some("There is no artifact card in your graveyard.".into()));
    }
    if let Some(why) = need(g, p, 2, "", "Buried Ruin") {
        return Ok(Some(why));
    }
    let labels: Vec<String> = arts.iter().map(|&c| choices::card_label(g, c)).collect();
    let Some(k) =
        choices::choose(g, p, "target", "Buried Ruin: return which artifact card?", &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    let c = arts[k];
    mana::pay_from_pool(g, p, 2, "");
    let cd = g.land(l).cd;
    crate::impls::lands::sac_land(g, p, l)?;
    crate::glog!(g, "  {} sacrifices Buried Ruin", g.player(p).name);
    if ability_window_card(g, p, cd, &format!("return {} to hand", g.db.get(c).name), None, None)? {
        let pl = g.player_mut(p);
        if let Some(i) = pl.gy.iter().position(|&x| x == c) {
            pl.gy.remove(i);
            pl.hand.push(c);
        }
    }
    Ok(None)
}

fn fair(g: &mut Game, p: PlayerId, l: LandId) -> Res<Why> {
    if let Some(why) = land_tap_ok(g, l) {
        return Ok(Some(why));
    }
    if crate::impls::karn_engines::artifact_total(g, p) < 3 {
        return Ok(Some("Inventors' Fair: you control fewer than three artifacts.".into()));
    }
    if let Some(why) = need(g, p, 4, "", "Inventors' Fair") {
        return Ok(Some(why));
    }
    run(g, p, Action::Plan { f: crate::impls::karn_engines::fair_go, arg: l.0 as i64 }, "Inventors' Fair")
}

fn war_room(g: &mut Game, p: PlayerId, l: LandId) -> Res<Why> {
    if let Some(why) = land_tap_ok(g, l) {
        return Ok(Some(why));
    }
    let n = g.player(p).ident.count() as i32;
    if g.player(p).life < n {
        return Ok(Some(format!("War Room: you can't pay {n} life.")));
    }
    if let Some(why) = need(g, p, 3, "", "War Room") {
        return Ok(Some(why));
    }
    run(g, p, Action::Plan { f: crate::impls::lands::war_room_go, arg: l.0 as i64 }, "War Room")
}

fn vault(g: &mut Game, p: PlayerId, l: LandId) -> Res<Why> {
    if let Some(why) = land_tap_ok(g, l) {
        return Ok(Some(why));
    }
    let Some(x) = pick_n(g, p, "Treasure Vault ({X}{X})", 0, mana_in_pool(g, p) / 2)? else {
        return Ok(None);
    };
    if let Some(why) = need(g, p, 2 * x, "", "Treasure Vault") {
        return Ok(Some(why));
    }
    mana::pay_from_pool(g, p, 2 * x, "");
    let cd = g.land(l).cd;
    crate::impls::lands::sac_land(g, p, l)?;
    crate::glog!(g, "  {} sacrifices Treasure Vault for {} Treasure(s)", g.player(p).name, x);
    if ability_window_card(g, p, cd, &format!("create {x} Treasure tokens"), None, None)? {
        zones::add_treasure(g, p, x as i32)?;
    }
    Ok(None)
}

fn urza_factory(g: &mut Game, p: PlayerId, l: LandId) -> Res<Why> {
    if let Some(why) = land_tap_ok(g, l).or_else(|| need(g, p, 7, "", "Urza's Factory")) {
        return Ok(Some(why));
    }
    mana::pay_from_pool(g, p, 7, "");
    g.land_mut(l).tapped = true;
    crate::glog!(g, "  {} activates Urza's Factory", g.player(p).name);
    if ability_window_card(g, p, g.land(l).cd, "create a 2/2 Assembly-Worker", None, None)? {
        let mut spec = Tokens::new(1, 2);
        spec.tgh = Some(2);
        spec.color = Some(Colors::NONE);
        spec.types = vec!["artifact", "assembly-worker"];
        zones::make_tokens(g, p, spec)?;
    }
    Ok(None)
}

/// {1} / {2}: the land becomes a 2/2 Assembly-Worker artifact creature until end of turn (the engine's token standing
/// in for the land). A land that came under your control this turn is summoning sick.
fn animate(g: &mut Game, p: PlayerId, l: LandId) -> Res<Why> {
    let cost = crate::impls::karn::cost_of_land(g, l);
    let nm = lname(g, l);
    if let Some(why) = land_tap_ok(g, l).or_else(|| need(g, p, cost, "", &nm)) {
        return Ok(Some(why));
    }
    let new_land = g.player(p).land_turn == g.player(p).turns as i32 && g.player(p).lands.last() == Some(&l);
    mana::pay_from_pool(g, p, cost, "");
    if !ability_window_card(g, p, g.land(l).cd, "becomes a creature", None, None)? {
        return Ok(None);
    }
    let m = crate::impls::karn::animate(g, p, l);
    if new_land {
        g.perm_mut(m).sick = true;
    }
    Ok(None)
}

/// the land's own {T} ability (while it is not a creature): pump an Assembly-Worker; `pay` is the generic cost
fn land_pump(g: &mut Game, p: PlayerId, l: LandId, pay: u32, by: (i32, i32)) -> Res<Why> {
    let nm = lname(g, l);
    if let Some(why) = land_tap_ok(g, l).or_else(|| need(g, p, pay, "", &nm)) {
        return Ok(Some(why));
    }
    let attacking = by.0 == 2;
    let tg: Vec<PermId> = targets(g, p)
        .into_iter()
        .filter(|&x| assembly_worker(g, x) && (!attacking || g.in_combat.contains(&x)))
        .collect();
    if tg.is_empty() {
        return Ok(Some(format!(
            "{nm}: there is no {}Assembly-Worker to target.",
            if attacking { "attacking " } else { "" }
        )));
    }
    let prompt = format!("{nm}: which Assembly-Worker gets +{}/+{}?", by.0, by.1);
    let Some(t) = cards::pick_perm(g, p, "target", &prompt, &tg, Some("cancel"))? else { return Ok(None) };
    if pay > 0 {
        mana::pay_from_pool(g, p, pay, "");
    }
    g.land_mut(l).tapped = true;
    let cd = g.land(l).cd;
    pump(g, p, None, Some(cd), t, by)
}
