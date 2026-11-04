//! Fetch lands for a person (practice mode). The AI cracks a fetch land at once as it enters (`engine::turn::crack_fetch`);
//! a person keeps it on the battlefield and activates it when they choose, paying what the card says ({T}, 1 life,
//! {2} for Terminal Moraine, the sacrifice), and picks the land from the legal ones when the ability resolves (they may
//! find nothing). The ability is listed by `abilities::land_abilities`.

use super::abilities::{Ability, Why, ability};
use super::{choices, mana};
use crate::engine::{self, stack, turn, zones};
use crate::flow::Res;
use crate::ids::{PermId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

/// the fetch lands a person activates themselves: (name, generic mana, life)
const FETCHES: &[(&str, u32, i32)] = &[
    ("Arid Mesa", 0, 1),
    ("Bloodstained Mire", 0, 1),
    ("Flooded Strand", 0, 1),
    ("Marsh Flats", 0, 1),
    ("Misty Rainforest", 0, 1),
    ("Polluted Delta", 0, 1),
    ("Prismatic Vista", 0, 1),
    ("Scalding Tarn", 0, 1),
    ("Verdant Catacombs", 0, 1),
    ("Windswept Heath", 0, 1),
    ("Wooded Foothills", 0, 1),
    ("Evolving Wilds", 0, 0),
    ("Terramorphic Expanse", 0, 0),
    ("Fabled Passage", 0, 0),
    ("Terminal Moraine", 2, 0),
];

fn cost_of(name: &str) -> Option<(u32, i32)> {
    FETCHES.iter().find(|f| f.0 == name).map(|f| (f.1, f.2))
}

/// is this a fetch land a person cracks themselves (the other `Tag::F` lands are still cracked at once)?
pub fn person_fetch(name: &str) -> bool {
    cost_of(name).is_some()
}

/// the ability list of land l: the crack ability, when l is a fetch land (and Blood Moon hasn't taken it away)
pub fn land_abilities(g: &Game, l: PermId) -> Vec<Ability<PermId>> {
    let d = g.db.get(g.perm(l).card());
    if !d.tag(Tag::F) || engine::mana::moon_land(g, l) {
        return vec![];
    }
    let Some((gn, life)) = cost_of(&d.name) else { return vec![] };
    let mut label = String::from("{T}, ");
    if gn > 0 {
        label = format!("{{{gn}}}, {label}");
    }
    if life > 0 {
        label.push_str(&format!("pay {life} life, "));
    }
    label.push_str("sacrifice it: search your library for ");
    label.push_str(match turn::true_fetch(&d.name) {
        Some("basic") => "a basic land card".to_string(),
        Some(k) => {
            let n = |c: char| match c {
                'W' => "Plains",
                'U' => "Island",
                'B' => "Swamp",
                'R' => "Mountain",
                _ => "Forest",
            };
            let mut cs = k.chars();
            format!("a {} or {} card", n(cs.next().unwrap()), n(cs.next().unwrap()))
        }
        None => "a basic land card".to_string(),
    }.as_str());
    vec![ability(label, crack)]
}

/// the fetch land's ability: costs first ({2}, {T}, life, sacrifice), then the ability goes on the stack and, when it
/// resolves, the person picks a land card from the legal ones (or none)
pub fn crack(g: &mut Game, p: PlayerId, l: PermId) -> Res<Why> {
    let cd = g.perm(l).card();
    let name = g.db.get(cd).name.to_string();
    let Some((gn, life)) = cost_of(&name) else { return Ok(Some(format!("{name} has no ability to activate."))) };
    if !g.perm(l).on_bf || !g.player(p).lands.contains(&l) {
        return Ok(Some(format!("{name} is not on the battlefield.")));
    }
    if engine::mana::moon_land(g, l) {
        return Ok(Some(format!("{name} has lost its abilities.")));
    }
    if g.perm(l).tapped {
        return Ok(Some(format!("{name} is tapped.")));
    }
    if life > 0 && g.player(p).life < life {
        return Ok(Some(format!("{name}: you can't pay {life} life.")));
    }
    if gn > 0 {
        if let Some(why) = mana::cost_problem(g, p, gn, "") {
            return Ok(Some(format!("Can't activate {name}. {why}")));
        }
        mana::pay_from_pool(g, p, gn, "");
    }
    g.perm_mut(l).tapped = true;
    if life > 0 {
        engine::life::lose_life(g, p, life, Some(p), "other", None)?;
    }
    crate::glog!(g, "  {} activates {}", g.player(p).name, name);
    crate::impls::lands::sac_land(g, p, l)?; // the sacrifice is a cost: it leaves before the ability can be answered
    engine::life::check_state(g)?;
    if g.over || !g.player(p).alive {
        return Ok(None);
    }
    if !stack::ability_window_card(g, p, cd, "search for a land card", None, None)? {
        return Ok(None);
    }
    let kinds = turn::true_fetch(&name);
    let found = choices::search(
        g,
        p,
        &move |g: &Game, c| turn::fetch_legal(g, c, kinds),
        1,
        &format!("{name}: search for a land card"),
        true,
    )?;
    let Some(&c) = found.first() else {
        crate::glog!(g, "    {} finds nothing", g.player(p).name);
        return Ok(None);
    };
    if let Some(a) = zones::agent_for(g, p) {
        zones::agent_take(g, a, p, c); // Opposition Agent
        return Ok(None);
    }
    // the land comes in tapped unless the card says otherwise; Fabled Passage untaps it with four or more lands
    // (the Passage is gone: three others, and the new one)
    let forced = kinds.is_none() && !(name == "Fabled Passage" && g.player(p).lands.len() >= 3);
    let tapped = if forced {
        true
    } else if let Some(n) = turn::shock_cost(g, p, c) {
        // a shock land: the person chooses whether to pay for it to enter untapped
        let pay = g.player(p).life >= n
            && choices::yes_no(g, p, &format!("{}: pay {n} life so it enters untapped?", g.db.get(c).name))?;
        if pay {
            engine::life::lose_life(g, p, n, Some(p), "other", None)?;
        }
        !pay
    } else {
        turn::land_enters_tapped(g, p, c)
    };
    let l2 = g.add_land(p, c, tapped);
    crate::glog!(g, "    cracks {} for {}", name, g.db.get(c).name);
    crate::cardcode::land_etb(g, p, l2)?; // Bojuka Bog ...
    zones::landfall(g, p)?;
    Ok(None)
}
