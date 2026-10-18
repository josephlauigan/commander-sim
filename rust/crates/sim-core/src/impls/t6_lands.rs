//! Tier 6 lands batch (rust/agents/t6-batches.md, `lands`): utility lands of the Tier 6 decks. Cards only: what
//! needs a new engine feature is listed in rust/agents/engine-needs-lands.md.
//!
//! Lands are not permanents: their abilities are offered through the `land_options` / `hand_options` slots, their
//! mana through `land_cols`, `dyn_mana_land` and `on_tap_land` (see impls/lands.rs).

use super::lands::{can_pay_without, pay_without};
use crate::cards::{CardDb, Colors, Types};
use crate::engine::stack::ability_window_card;
use crate::engine::tutors::card_worth;
use crate::engine::zones::max_by;
use crate::hooks::{Action, Opt};
use crate::ids::CardId;
use crate::tag::Tag;
use crate::engine::life::lose_life;
use crate::flow::Res;
use crate::hooks::Registry;
use crate::ids::{LandId, PlayerId};
use crate::state::{DataKey, Game};

fn name_of(g: &Game, l: LandId) -> &str {
    &g.db.get(g.land(l).cd).name
}

// ================================================================== City of Brass
/// **City of Brass**: {T}: any colour (tag c=A); whenever it becomes tapped it deals 1 damage to you.
fn brass_tap(g: &mut Game, p: PlayerId, _l: LandId, _n: u32) -> Res {
    lose_life(g, p, 1, Some(p), "other", Some(true))
}

// ================================================================== City of Traitors
/// **City of Traitors**: {T}: {C}{C} (tag amt=2). When you play another land, sacrifice it. The engine calls
/// `land_etb` for the land just played, so every land without code of its own gets this check (`traitors_check`).
fn traitors_check(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    let mine: Vec<LandId> = g
        .player(p)
        .lands
        .iter()
        .copied()
        .filter(|&x| x != l && name_of(g, x) == "City of Traitors")
        .collect();
    for c in mine {
        if g.player(p).lands.contains(&c) {
            super::lands::sac_land(g, p, c)?;
            crate::glog!(g, "    {} sacrifices City of Traitors (another land was played)", g.player(p).name);
        }
    }
    Ok(())
}

// ================================================================== Gemstone Caverns
/// **Gemstone Caverns**: {T}: Add {C}; with a luck counter on it, one mana of any colour instead. The opening-hand
/// clause (begin the game with it on the battlefield with a luck counter, exiling a card) needs the engine: see
/// engine-needs-lands.md. The luck counter is `DataKey::Counters` (an Int) on the land.
fn caverns_cols(g: &Game, p: PlayerId, l: LandId) -> Colors {
    if g.land(l).data.int(DataKey::Counters) > 0 { g.player(p).ident } else { Colors::NONE }
}

// ================================================================== the "{1},{T}: any colour" lands
/// **The Mycosynth Gardens, Talon Gates of Madara, Abstergo Entertainment**: {T}: Add {C}. {1},{T}: Add one mana of any
/// colour. A filter for one mana: the land makes any colour only when another untapped land (not one of these) pays
/// the {1}; the count of mana stays right (the land is one mana, the payer another, as if the payer made colourless
/// mana and the land its own colourless mana). With nothing else to pay, colourless only
const ANY_FILTER: [&str; 3] = ["The Mycosynth Gardens", "Talon Gates of Madara", "Abstergo Entertainment"];

fn any_filter_cols(g: &Game, p: PlayerId, l: LandId) -> Colors {
    let payer = g.player(p).lands.iter().any(|&x| x != l && !g.land(x).tapped && !ANY_FILTER.contains(&name_of(g, x)));
    if payer { g.player(p).ident } else { Colors::NONE }
}

// ================================================================== Abstergo Entertainment
/// **Abstergo Entertainment**: {3},{T}, exile it: return up to one target historic card (artifact, legendary, Saga)
/// from your graveyard to your hand, then exile all graveyards. The AI does it with 3 spare mana when a historic card
/// in its graveyard is worth at least 30 as a card in hand
fn historic(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.types.has(Types::ARTIFACT) || d.tag(Tag::Leg) || d.has_subtype("saga")
}

fn abstergo_target(g: &Game, p: PlayerId) -> Option<CardId> {
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| historic(g, c)).collect();
    max_by(&cs, |c| card_worth(g, p, c, false)).filter(|&c| card_worth(g, p, c, false) >= 30.0)
}

fn abstergo(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post == Some(false) || g.land(l).tapped || !can_pay_without(g, p, l, 3, "") {
        return Ok(vec![]);
    }
    let Some(t) = abstergo_target(g, p) else { return Ok(vec![]) };
    let u = card_worth(g, p, t, false) / 30.0;
    Ok(vec![Opt {
        utility: u,
        label: "Abstergo Entertainment".into(),
        act: Some(Action::Plan { f: abstergo_go, arg: l.0 as i64 }),
    }])
}

fn abstergo_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    if !g.player(p).lands.contains(&l) || g.land(l).tapped || !pay_without(g, p, l, 3, "")? {
        return Ok(false);
    }
    let cd = g.land(l).cd;
    crate::engine::turn::remove_land(g, p, l); // {T} and exile it as costs
    g.player_mut(p).exile.push(cd);
    crate::glog!(g, "  {} exiles Abstergo Entertainment", g.player(p).name);
    let t = abstergo_target(g, p);
    if ability_window_card(g, p, cd, "return a historic card, exile all graveyards", None, None)? {
        if let Some(c) = t.filter(|c| g.player(p).gy.contains(c)) {
            let i = g.player(p).gy.iter().position(|&x| x == c).unwrap();
            g.player_mut(p).gy.remove(i);
            g.player_mut(p).hand.push(c);
            crate::glog!(g, "    {} returns {} to hand", g.player(p).name, g.db.get(c).name);
        }
        for i in 0..g.players.len() {
            super::common::exile_gy(g, PlayerId(i as u8), Some("Abstergo Entertainment"));
        }
    }
    Ok(true)
}

// CODE-HERE

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "City of Brass")?.on_tap_land = Some(brass_tap);
    // City of Traitors: land_etb on every land that has none yet (the City itself included: playing a second City
    // sacrifices the first, and the new one stays)
    let lands: Vec<String> = db.iter().filter(|d| d.land).map(|d| d.name.to_string()).collect();
    for n in lands {
        let c = r.card(db, &n)?;
        if c.land_etb.is_none() {
            c.land_etb = Some(traitors_check);
        }
    }
    r.card(db, "Gemstone Caverns")?.land_cols = Some(caverns_cols);
    for n in ANY_FILTER {
        r.card(db, n)?.land_cols = Some(any_filter_cols);
    }
    r.card(db, "Abstergo Entertainment")?.land_options = Some(abstergo);
    // REGISTER-HERE
    Ok(())
}
