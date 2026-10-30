//! The table as one seat is allowed to see it (Python's `play/view.py`): plain JSON, safe to hand to another thread.
//! Hidden information stays hidden: other players' hands and every library are counts only.
//!
//! Four pieces of the "you" part depend on the human-rules modules that practice-core / practice-cards are still
//! porting (the mana pool and its sources, graveyard plays, hand cards with a second use, land abilities). They are
//! behind the `seam` functions at the bottom, which give Python's empty answers until those modules land.

use serde_json::{Map, Value, json};
use sim_core::engine::values::{epow, etgh};
use sim_core::human::legal::token_name;
use sim_core::ids::PlayerId;
use sim_core::state::{Game, Step};

pub fn step_name(s: Step) -> &'static str {
    match s {
        Step::Start => "start",
        Step::Main1 => "main1",
        Step::Combat => "combat",
        Step::Main2 => "main2",
        Step::End => "end",
    }
}

fn perm(g: &Game, m: sim_core::ids::PermId) -> Map<String, Value> {
    let m_ = g.perm(m);
    let name = if m_.token { token_name(g, m_) } else { m_.name.to_string() };
    let mut d = Map::new();
    d.insert("name".into(), name.into());
    d.insert("tapped".into(), m_.tapped.into());
    d.insert("token".into(), m_.token.into());
    if g.is_creature(m) {
        d.insert("pt".into(), format!("{}/{}", epow(g, m), etgh(g, m)).into());
        d.insert("sick".into(), m_.sick.into());
    }
    if m_.plus != 0 {
        d.insert("counters".into(), m_.plus.into());
    }
    if let Some(l) = m_.loyalty {
        d.insert("loyalty".into(), l.into());
    }
    if let Some(a) = m_.attached
        && !g.perm(a).name.is_empty()
    {
        d.insert("attached_to".into(), g.perm(a).name.to_string().into());
    }
    if m_.is_cmd {
        d.insert("commander".into(), true.into());
    }
    if m_.orig != m_.owner {
        d.insert("owner".into(), (&*g.player(m_.orig).key).into()); // a stolen card shows its owner's printing
    }
    if m_.phased {
        d.insert("phased_out".into(), true.into());
    }
    if m_.data.truthy(sim_core::state::DataKey::Flipped) {
        d.insert("face".into(), 1.into()); // transformed (Sephiroth, One-Winged Angel): show the back face
        d.insert("transformed".into(), true.into());
    }
    d
}

fn player(g: &Game, p: PlayerId, me: &str) -> Value {
    let pl = g.player(p);
    let you = &*pl.key == me;
    // commander damage is counted per commander: the most one of a seat's commanders has dealt (rule 903.10a)
    let cmd_dmg: Map<String, Value> = g
        .players
        .iter()
        .filter(|q| pl.cmd_dmg_from(q.id.index()) != 0)
        .map(|q| (q.name.to_string(), pl.cmd_dmg_from(q.id.index()).into()))
        .collect();
    let names = |v: &[sim_core::ids::CardId]| -> Vec<String> { v.iter().map(|&c| g.db.get(c).name.to_string()).collect() };
    let battlefield: Vec<Value> = pl
        .perms
        .iter()
        .enumerate()
        .map(|(i, &m)| {
            let mut d = perm(g, m);
            d.insert("i".into(), i.into());
            Value::Object(d)
        })
        .collect();
    let lands: Vec<Value> = pl
        .lands
        .iter()
        .enumerate()
        .map(|(i, &l)| {
            let land = g.land(l);
            let mut d = json!({"name": &*g.db.get(land.card()).name, "tapped": land.tapped, "i": i});
            if you && seam::land_has_ability(g, p, l) {
                d["ability"] = true.into();
            }
            d
        })
        .collect();
    let mut d = json!({
        "key": &*pl.key, "name": &*pl.name, "commander": &*g.db.get(pl.cmd).name, "you": you, "alive": pl.alive,
        "life": pl.life, "poison": pl.poison, "commander_damage": cmd_dmg,
        "hand_count": pl.hand.len(), "library": pl.library.len(),
        "graveyard": names(&pl.gy), "exile": names(&pl.exile),
        "commander_in_zone": pl.cmd_in_zone, "tax": pl.tax, "treasures": pl.treasures,
        "battlefield": battlefield, "lands": lands,
    });
    if let Some(c2) = pl.cmd2 {
        // Partner: the second commander, with its own zone, tax and damage (who dealt how much with each)
        d["commander2"] = (&*g.db.get(c2).name).into();
        d["commander2_in_zone"] = pl.cmd2_in_zone.into();
        d["tax2"] = pl.tax2.into();
        let by: Map<String, Value> = g
            .players
            .iter()
            .filter(|q| pl.cmd_dmg[q.id.index()] != 0 || pl.cmd2_dmg[q.id.index()] != 0)
            .map(|q| (q.name.to_string(), json!([pl.cmd_dmg[q.id.index()], pl.cmd2_dmg[q.id.index()]])))
            .collect();
        d["commander_damage_by"] = by.into();
    }
    if you {
        d["hand"] = names(&pl.hand).into();
        d["hand_land"] = pl.hand.iter().map(|&c| g.db.get(c).land).collect::<Vec<_>>().into();
        d["hand_special"] = pl.hand.iter().map(|&c| seam::hand_special(g, c)).collect::<Vec<_>>().into();
        d["mana_pool"] = seam::mana_pool_text(g, p).into();
        d["mana_sources"] = seam::mana_sources(g, p);
        d["graveyard_playable"] = seam::graveyard_playable(g, p);
    }
    d
}

/// the table from the seat of deck `me`; call on the game thread (or on a snapshot)
pub fn build_view(g: &Game, me: &str) -> Value {
    json!({
        "round": g.round,
        "active": g.active.map(|a| g.player(a).key.to_string()),
        "step": step_name(g.step),
        "over": g.over,
        "winner": g.winner.map(|w| g.player(w).key.to_string()),
        "players": g.players.iter().map(|q| player(g, q.id, me)).collect::<Vec<_>>(),
    })
}

/// The pieces that come from the human-rules modules. `hand_special` and `land_has_ability` wait on practice-cards
/// (`human/cards.rs`, `human/abilities.rs`) and give Python's "nothing" until then.
pub mod seam {
    use super::*;
    use sim_core::human::legal::{GyKind, check_land_gy, gy_mode};
    use sim_core::human::mana::sources;
    use sim_core::ids::{CardId, LandId};

    /// the human's mana pool as text ("WWU"); `mana.pool_of(p).text()`
    pub fn mana_pool_text(g: &Game, p: PlayerId) -> String {
        g.player(p).pool.text()
    }
    /// `mana.sources(g, p)`: what the person can still tap
    pub fn mana_sources(g: &Game, p: PlayerId) -> Value {
        sources(g, p)
            .into_iter()
            .map(|s| {
                let mut d = json!({"id": s.id, "name": s.name, "colours": s.colours, "amount": s.amount});
                if let Some(r) = s.restricted {
                    d["restricted"] = r.into();
                }
                if s.treasure {
                    d["treasure"] = true.into();
                }
                if let Some(l) = s.land {
                    d["land"] = l.into();
                }
                if let Some(m) = s.perm {
                    d["perm"] = m.into();
                }
                d
            })
            .collect::<Vec<_>>()
            .into()
    }
    /// graveyard cards the person may play: `[{i, name, how}]`
    pub fn graveyard_playable(g: &Game, p: PlayerId) -> Value {
        let mut out = vec![];
        for (i, &c) in g.player(p).gy.iter().enumerate() {
            let how = match gy_mode(g, p, c) {
                Some((k, _, _)) => match k {
                    GyKind::Yawg => "yawg",
                    GyKind::Flashback => "flashback",
                    GyKind::Sac3 => "sac3",
                    GyKind::Escape => "escape",
                    GyKind::Muldrotha => "muldrotha",
                },
                None if g.db.get(c).land && check_land_gy(g, p, c).is_none() => "land",
                None if sim_core::human::alt::gy_hook(g, c) => "alt", // an escape, unearth ... the card code keeps for the AI
                None => continue,
            };
            out.push(json!({"i": i, "name": &*g.db.get(c).name, "how": how}));
        }
        out.into()
    }
    /// a land with a spell side or channel (`cards.HAND`)
    pub fn hand_special(_g: &Game, _c: CardId) -> bool {
        false
    }
    /// `abilities.land_abilities(g, p, L)` is not empty
    pub fn land_has_ability(g: &Game, p: PlayerId, l: LandId) -> bool {
        !sim_core::human::abilities::land_abilities(g, p, l).is_empty() || sim_core::human::alt::land_hook(g, l)
    }
    /// the mana the person has in the pool, counted as any colour for a copy of the game (`mana.pool_of(p).total()`)
    pub fn pool_total(g: &Game, p: PlayerId) -> u32 {
        g.player(p).pool.total()
    }
}
