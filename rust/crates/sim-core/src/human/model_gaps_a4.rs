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
        mg::FACE_BREAKER => vec![ability(
            "sacrifice a Treasure: exile the top card of your library, you may play it this turn",
            face_breaker,
        )],
        mg::TINYBONES => vec![ability("{4}{B}{B}: each opponent with no cards in hand loses 10 life", tinybones)],
        mg::EZURI => {
            vec![ability("{2}{G}{G}{G}: Elf creatures you control get +3/+3 and trample until end of turn", ezuri)]
        }
        KIKI => vec![ability(
            "{T}: create a token that's a copy of target nonlegendary creature you control, with haste; sacrifice it at the next end step",
            kiki,
        )],
        FANG => vec![ability("{B}, sacrifice X Squirrels: target creature gets +X/-X until end of turn", fang)],
        _ => return super::marvin_hooks::abilities(nm),
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
    let Some(k) =
        choices::choose(g, p, "target", &format!("{WHAT}: 2 damage to which target?"), &labels, Some("cancel"), None)?
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
        let Some(f) = super::cards::pick_perm(
            g,
            p,
            "choose",
            &format!("{WHAT}: sacrifice which Goblin?"),
            &gobs,
            Some("cancel"),
        )?
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

// ------------------------------------------------------------------ Kiki-Jiki, Mirror Breaker; Chatterfang
pub const KIKI: &str = "Kiki-Jiki, Mirror Breaker";
pub const FANG: &str = "Chatterfang, Squirrel General";

/// the person's Kiki-Jiki: {T}: a hasty token copy of a nonlegendary creature you choose (any, Marvin using the ability
/// too: it is itself nonlegendary)
fn kiki(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    let x = g.perm(m);
    if x.tapped {
        return Ok(Some(format!("{} is tapped.", x.name)));
    }
    if x.sick && !crate::dsl::has_kw(g, m, "haste") && !crate::impls::t45_krenko::elixir(g, p) {
        return Ok(Some(format!("{} has summoning sickness (it came under your control this turn).", x.name)));
    }
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&c| {
            live(g, c)
                && g.is_creature(c)
                && g.perm(c).cd.is_some_and(|cd| !g.db.get(cd).tag(crate::tag::Tag::Leg))
                && !(g.perm(c).owner != p && crate::engine::values::untargetable(g, c))
        })
        .collect();
    if cands.is_empty() {
        return Ok(Some("You control no nonlegendary creature to copy.".into()));
    }
    let Some(t) = super::cards::pick_perm(
        g,
        p,
        "target",
        "Kiki-Jiki: copy which nonlegendary creature?",
        &cands,
        Some("cancel"),
    )?
    else {
        return Ok(None);
    };
    if !crate::impls::t5::kiki_go(g, m, p, t.0 as i64)? {
        return Ok(Some("That can't be done right now.".into()));
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

/// the person's Chatterfang: {B}, sacrifice X Squirrels: target creature gets +X/-X until end of turn; the target and X
/// are the person's
fn fang(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if let Some(why) = problem(g, p, 0, "B", "Chatterfang") {
        return Ok(Some(why));
    }
    let n = crate::impls::rules2::squirrels(g, p).len() as u32;
    let cands = super::cards::creatures(g, p, false);
    if cands.is_empty() {
        return Ok(Some("There is no creature to target.".into()));
    }
    let Some(t) =
        super::cards::pick_perm(g, p, "target", "Chatterfang: +X/-X to which creature?", &cands, Some("cancel"))?
    else {
        return Ok(None);
    };
    let xs = choices::amount_choices(n);
    let labels: Vec<String> =
        xs.iter().map(|x| format!("X = {x} (sacrifice {x} Squirrel{})", if *x == 1 { "" } else { "s" })).collect();
    let Some(k) =
        choices::choose(g, p, "choose", "Chatterfang: sacrifice how many Squirrels?", &labels, Some("cancel"), None)?
    else {
        return Ok(None);
    };
    if !crate::impls::rules2::fang_go(g, m, p, crate::impls::partials::pack(t.0, xs[k]))? {
        return Ok(Some("That can't be done right now.".into()));
    }
    crate::engine::life::check_state(g)?;
    Ok(None)
}

// ------------------------------------------------------------------ Marvin, Murderous Mimic: the person's choices
/// the creatures whose hand-written abilities Marvin uses through the person's own path (the same questions, Marvin as
/// the source and the one that pays); the other hand-written abilities Marvin has are the AI's options (`abilities_of`)
pub const BORROWED: &[&str] = &[
    "Balan, Wandering Knight",
    "Brass Squire",
    "Cabal Patriarch",
    "Cid, Freeflier Pilot",
    "Dauthi Voidwalker",
    "Emry, Lurker of the Loch",
    "Garth One-Eye",
    "Gix, Yawgmoth Praetor",
    "Jaxis, the Troublemaker",
    "Kefka, Court Mage // Kefka, Ruler of Ruin",
    "Kuldotha Forgemaster",
    "Nicol Bolas, the Ravager",
    "Resplendent Angel",
    "Scion of the Ur-Dragon",
    "Steel Hellkite",
    "Steel Overseer",
    "Stoneforge Mystic",
    "Thanos, the Mad Titan",
    "Thrasios, Triton Hero",
    "Triskelion",
    "Urza, Lord High Artificer",
    "Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal",
    "Whipcorder",
    "Yawgmoth, Thran Physician",
    KIKI,
    FANG,
    mg::SIEGE,
    mg::KONRAD,
    mg::FACE_BREAKER,
    mg::TINYBONES,
    mg::EZURI,
    "Azorius Guildmage",
    "Blightspeaker",
    "Ramosian Sergeant",
    "Ramosian Lieutenant",
    "Ramosian Captain",
    "Defiant Vanguard",
    "Ramosian Commander",
    "Lin Sivvi, Defiant Hero",
    "Ramosian Revivalist",
    "Errant Doomsayers",
    "Lawbringer",
    "Lightbringer",
    "Urborg Syphon-Mage",
    "Sakura-Tribe Elder",
    "Jace's Archivist",
];

/// is this hook option (a label from Marvin's `options`) one a borrowed person path replaces?
pub fn replaced_label(label: &str) -> bool {
    label.starts_with("Marvin (Kiki-Jiki)")
        || BORROWED
            .iter()
            .chain(super::marvin_hooks::NAMES.iter())
            .any(|n| label.starts_with(&format!("Marvin copies {n}:")))
}

/// the abilities Marvin, Murderous Mimic (m) has from the creatures p controls that are in `BORROWED`, as the person's
/// own list for each creature, labelled with its name
pub fn marvin_abilities(g: &Game, p: PlayerId, m: PermId) -> Vec<PermAbility> {
    let me = g.perm(m).name;
    let mut out = vec![];
    if crate::cardcode::ability_locked(g, m, p) {
        return out;
    }
    let mut seen: Vec<crate::sym::Sym> = vec![];
    for &d in &g.player(p).perms {
        let x = g.perm(d);
        if d == m || x.name == me || x.cd.is_none() || !x.on_bf || x.phased || x.neutered || !g.is_creature(d) {
            continue;
        }
        let nm = g.db.get(x.cd.unwrap()).name.to_string();
        if seen.contains(&x.name)
            || !(BORROWED.contains(&nm.as_str()) || super::marvin_hooks::NAMES.contains(&nm.as_str()))
            || crate::engine::values::stopped(g, &nm)
        {
            continue;
        }
        if crate::impls::bulk_white::is_face_down(g, d) {
            continue;
        }
        seen.push(x.name);
        let mut list: Vec<PermAbility> = match nm.as_str() {
            // "transform Kefka" names the creature that has the ability, Marvin (rule 201.5), which can't transform (701.28c)
            "Kefka, Court Mage // Kefka, Ruler of Ruin" if g.perm(d).data.truthy(crate::state::DataKey::Ruin) => vec![], // Ruler of Ruin has no activated ability
            "Kefka, Court Mage // Kefka, Ruler of Ruin" => vec![ability(
                "{8}: each opponent sacrifices a permanent of their choice (sorcery speed; Marvin does not transform)",
                move |g, p, m| super::marvin_hooks::kefka(g, p, m, d),
            )],
            _ => vec![],
        };
        if list.is_empty() {
            list.extend(super::abilities::permanent_abilities(g, p, d));
            list.extend(super::karn::perm_abilities(g, p, d));
            list.extend(super::cloud::perm_abilities(g, p, d));
            list.extend(super::cards::abilities(g, p, d).unwrap_or_default());
        }
        let mut labels: Vec<String> = vec![];
        for mut a in list {
            if labels.contains(&a.label) {
                continue;
            }
            labels.push(a.label.clone());
            a.label = format!("Marvin copies {nm}: {}", a.label);
            out.push(a);
        }
    }
    out
}
