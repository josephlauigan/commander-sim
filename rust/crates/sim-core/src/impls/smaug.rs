//! The smaug candidate deck (decklists/JD/candidates/smaug.md): its card code and AI plan. The cards it shares with
//! the lordofpain deck (Charforger, Kalain, Lagomos, Hero's Heirloom) are in `lordofpain.rs`.

use crate::cards::CardDb;
use crate::engine::mana::total_mana;
use crate::engine::zones::add_treasure;
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;

const SMAUG: &str = "Smaug the Impenetrable";

/// p's Smaug, if it is on the battlefield
pub fn smaug_of(g: &Game, p: PlayerId) -> Option<PermId> {
    super::lordofpain::controls_named(g, p, SMAUG)
}

/// Whenever Smaug is dealt noncombat damage, create that many Treasure tokens. Called by the engine wherever a
/// permanent is dealt noncombat damage (`engine::removal::apply_removal`, the ability language's damage to creatures);
/// Smaug is indestructible, so the damage itself changes nothing else. Combat damage never comes through here.
pub fn dealt_damage(g: &mut Game, m: PermId, n: i32) -> Res {
    if n <= 0 || !g.perm(m).on_bf || g.perm(m).cd.is_none_or(|c| &*g.db.get(c).name != SMAUG) {
        return Ok(());
    }
    let o = g.perm(m).owner;
    crate::glog!(g, "    Smaug is dealt {n} damage: {n} Treasure(s) for {}", g.player(o).name);
    add_treasure(g, o, n)
}

/// smaug's AI plan: Smaug first the moment seven mana is there (and every mana source before that, as the generic
/// plan already does), the Treasure-making creatures early; the rest is generic.
pub fn smaug_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let d = g.db.get(c);
    if c == g.player(p).cmd {
        return if total_mana(g, p, false) >= 7 { 97 } else { 60 };
    }
    let base = crate::ai::pool::generic_prio(g, p, c);
    let early = g.player(p).turns <= 5;
    match &*d.name {
        // the Treasure and mana makers that get the 7 faster
        "Kalain, Reclusive Painter" | "Generator Servant" | "Common Crook" | "Piggy Bank" | "Undercity Dire Rat"
            if early =>
        {
            base.max(70)
        }
        _ => base,
    }
}

/// {T}: Viashino Fangtail deals 1 damage to Smaug: a Treasure (the AI's use for the spare Fangtail once Smaug is out)
fn fangtail_ping(g: &mut Game, m: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let Some(s) = smaug_of(g, p) else { return Ok(false) };
    let x = g.perm(m);
    if !x.on_bf || x.tapped || x.sick {
        return Ok(false);
    }
    g.perm_mut(m).tapped = true;
    dealt_damage(g, s, 1)?;
    Ok(true)
}

/// the pingers' use on Smaug: each untapped Viashino Fangtail offers a Treasure
fn smaug_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.perm(src).owner != p {
        return Ok(vec![]);
    }
    let mut out = vec![];
    // Orcish Cannonade: 2 damage to Smaug is two Treasures and a card, for 3 life
    for &c in &g.player(p).hand {
        let (cg, cp) = crate::engine::mana::cost_of(g, p, c);
        if &*g.db.get(c).name == "Orcish Cannonade"
            && g.player(p).life > 15
            && crate::engine::mana::can_pay(g, p, cg, &cp, false)
        {
            out.push(Opt {
                utility: 2.0,
                label: "Orcish Cannonade at Smaug (two Treasures, a card)".into(),
                act: Some(Action::Removal {
                    card: c,
                    targets: vec![src],
                    kick: 0,
                    kind: Some(crate::sym::intern("dmg2")),
                }),
            });
        }
    }
    for &m in &g.player(p).perms {
        let x = g.perm(m);
        if x.on_bf
            && !x.tapped
            && !x.sick
            && !x.phased
            && x.cd.is_some_and(|c| &*g.db.get(c).name == "Viashino Fangtail")
        {
            out.push(Opt {
                utility: 2.5,
                label: "Viashino Fangtail pings Smaug (a Treasure)".into(),
                act: Some(Action::Ability { src: m, f: fangtail_ping, arg: 0 }),
            });
        }
    }
    Ok(out)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, SMAUG)?.options = Some(smaug_options);
    Ok(())
}

/// the smaug deck's pings (Goblin Medics, Goblin Arsonist, Bomber Corps, burn with no creature worth killing) go to
/// its own Smaug for Treasures instead of an opponent's face, unless the damage would be lethal there
pub fn ping_target(g: &Game, p: PlayerId, eff: i32) -> Option<PermId> {
    if g.player(p).key != "smaug" || g.opps(p).any(|q| g.player(q).life <= eff) {
        return None;
    }
    smaug_of(g, p)
}
