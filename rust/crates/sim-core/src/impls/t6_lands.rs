//! Tier 6 lands batch (rust/agents/t6-batches.md, `lands`): utility lands of the Tier 6 decks. Cards only: what
//! needs a new engine feature is listed in rust/agents/engine-needs-lands.md.
//!
//! Lands are not permanents: their abilities are offered through the `land_options` / `hand_options` slots, their
//! mana through `land_cols`, `dyn_mana_land` and `on_tap_land` (see impls/lands.rs).

use crate::cards::{CardDb, Colors};
use crate::engine::life::lose_life;
use crate::flow::Res;
use crate::hooks::Registry;
use crate::ids::{LandId, PlayerId};
use crate::state::Game;

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
    let _ = Colors::NONE;
    Ok(())
}
