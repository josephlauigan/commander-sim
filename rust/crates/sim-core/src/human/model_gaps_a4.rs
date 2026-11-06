//! Practice mode: the abilities of Siege-Gang Commander, Syr Konrad, Professional Face-Breaker, Tinybones and Ezuri
//! (`impls/model_gaps_a4.rs` has the effects and the AI's use). For a person each is listed whenever the rules allow
//! it, paid from the pool, with the choices the Oracle text gives (Siege-Gang: the Goblin sacrificed and the target of
//! the 2 damage), and runs through the same stack code as the AI's. The functions take the ability's source `m`: the
//! creature itself, or Marvin, Murderous Mimic using it (`marvin::borrowed`).

use super::abilities::{PermAbility, Why, ability};
use super::{choices, legal, mana};
use crate::engine::values::colors_of;
use crate::flow::Res;
use crate::ids::{PermId, PlayerId};
use crate::impls::model_gaps_a4 as mg;
use crate::state::Game;

fn pay(g: &mut Game, p: PlayerId, gn: u32, pips: &str, what: &str) -> Why {
    mana::pay_from_pool(g, p, gn, pips).map(|why| format!("Can't activate {what}. {why}"))
}

fn problem(g: &Game, p: PlayerId, gn: u32, pips: &str, what: &str) -> Why {
    mana::cost_problem(g, p, gn, pips).map(|why| format!("Can't activate {what}. {why}"))
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

/// the abilities of p's permanent m (named `nm`) that this file provides, or None
pub fn abilities(_g: &Game, _p: PlayerId, _m: PermId, nm: &str) -> Option<Vec<PermAbility>> {
    Some(match nm {
        mg::SIEGE => vec![ability("{1}{R}, sacrifice a Goblin: 2 damage to any target", siege)],
        mg::KONRAD => vec![ability("{1}{B}: each player mills a card", konrad)],
        mg::FACE_BREAKER => vec![ability("sacrifice a Treasure: exile the top card of your library, you may play it this turn", face_breaker)],
        mg::TINYBONES => vec![ability("{4}{B}{B}: each opponent with no cards in hand loses 10 life", tinybones)],
        mg::EZURI => vec![ability("{2}{G}{G}{G}: Elf creatures you control get +3/+3 and trample until end of turn", ezuri)],
        _ => return None,
    })
}

fn siege(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    const WHAT: &str = "Siege-Gang Commander";
    if !live(g, m) {
        return Ok(Some(format!("{WHAT} is not on the battlefield.")));
    }
    let gobs = mg::goblins(g, p);
    if gobs.is_empty() {
        return Ok(Some("You control no Goblin to sacrifice.".into()));
    }
    if let Some(why) = problem(g, p, 1, "R", WHAT) {
        return Ok(Some(why));
    }
    let cands = choices::damage_targets(g, p, colors_of(g, m));
    if cands.is_empty() {
        return Ok(Some("There is no target for the damage.".into()));
    }
    let labels: Vec<String> = cands.iter().map(|&x| legal::describe_target(g, p, x)).collect();
    let Some(k) = choices::choose(g, p, "target", &format!("{WHAT}: 2 damage to which target?"), &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    let t = match cands[k] {
        super::Target::Player(q) => mg::Tgt::Player(q),
        super::Target::Perm(x) => mg::Tgt::Perm(x),
    };
    let f = if gobs.len() == 1 {
        gobs[0]
    } else {
        let Some(f) = super::cards::pick_perm(g, p, "choose", &format!("{WHAT}: sacrifice which Goblin?"), &gobs, Some("cancel"))?
        else {
            return Ok(None);
        };
        f
    };
    if let Some(why) = pay(g, p, 1, "R", WHAT) {
        return Ok(Some(why));
    }
    crate::glog!(g, "  {} activates {}: sacrifices {}", g.player(p).name, g.perm(m).name, g.perm(f).name);
    crate::engine::zones::die(g, f, "sac")?;
    mg::siege_resolve(g, p, m, t, false)?;
    crate::engine::life::check_state(g)?;
    Ok(None)
}

fn konrad(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = problem(g, p, 1, "B", "Syr Konrad") {
        return Ok(Some(why));
    }
    if let Some(why) = pay(g, p, 1, "B", "Syr Konrad") {
        return Ok(Some(why));
    }
    crate::glog!(g, "  {} activates {}: each player mills a card", g.player(p).name, g.perm(m).name);
    mg::konrad_resolve(g, p, m)?;
    Ok(None)
}

fn face_breaker(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if g.player(p).treasures == 0 {
        return Ok(Some("You control no Treasure to sacrifice.".into()));
    }
    crate::glog!(g, "  {} activates {}: sacrifices a Treasure", g.player(p).name, g.perm(m).name);
    mg::sac_treasure(g, p)?;
    mg::face_breaker_resolve(g, p, m)?;
    Ok(None)
}

fn tinybones(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = pay(g, p, 4, "BB", "Tinybones, Trinket Thief") {
        return Ok(Some(why));
    }
    crate::glog!(g, "  {} activates {}", g.player(p).name, g.perm(m).name);
    mg::tinybones_resolve(g, p, m)?;
    Ok(None)
}

fn ezuri(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = pay(g, p, 2, "GGG", "Ezuri, Renegade Leader") {
        return Ok(Some(why));
    }
    crate::glog!(g, "  {} activates {}", g.player(p).name, g.perm(m).name);
    mg::ezuri_resolve(g, p, m)?;
    Ok(None)
}
