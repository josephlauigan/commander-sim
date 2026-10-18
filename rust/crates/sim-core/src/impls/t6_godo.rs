//! Tier 6 "godo" batch: the cards of JD's Godo, Bandit Warlord deck (Godo + Helm of the Host extra combats) that the
//! compiled Oracle text and the engine's tags don't cover (the audit verdicts are in data/card_overrides.toml; each
//! card's doc comment says how it is modeled and what is left out; what needs the shared engine is in
//! rust/agents/engine-needs-godo.md). Helm of the Host itself is already modeled (`t3::helm`, `combat_start`).

use crate::cards::CardDb;
use crate::engine::stack::trigger_window;
use crate::engine::tutors::shuffle_library;
use crate::engine::values::once_per_turn;
use crate::engine::values::epow;
use crate::engine::zones::{Enter, enter, max_by};
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::human::human_of;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;
use crate::sym::{Sym, intern};

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn name_of(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

fn is_equipment_card(g: &Game, c: CardId) -> bool {
    g.db.get(c).has_subtype("equipment")
}

fn on_bf_named(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| live(g, m) && name_of(g, m) == name)
}

fn first_attack_key(src: PermId) -> Sym {
    intern(&format!("godo_first_attack_{}", src.0))
}

// ================================================================== Godo, Bandit Warlord
/// the copy cards that make Godo's second fetch worth Hammer first (the spec's line 3): on the battlefield or in hand
const COPIERS: [&str; 6] = [
    "Peter Parker's Camera",
    "Twinflame",
    "Molten Duplication",
    "Heat Shimmer",
    "Electroduplicate",
    "Cursed Mirror",
];

/// which Equipment Godo fetches: Hammer of Nazahn first when a copier is available (the copy then fetches Helm and
/// Hammer attaches it free), else Helm of the Host, else Hammer, else the AI's usual pick (None)
fn godo_wants(g: &Game, p: PlayerId) -> Option<&'static str> {
    let lib = |n: &str| g.player(p).library.iter().any(|&c| &*g.db.get(c).name == n);
    let copier = COPIERS.iter().any(|n| {
        on_bf_named(g, p, n) || g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == *n)
    });
    let helm = on_bf_named(g, p, "Helm of the Host");
    let hammer = on_bf_named(g, p, "Hammer of Nazahn");
    if copier && !hammer && !helm && lib("Hammer of Nazahn") {
        Some("Hammer of Nazahn")
    } else if !helm && lib("Helm of the Host") {
        Some("Helm of the Host")
    } else if !hammer && lib("Hammer of Nazahn") {
        Some("Hammer of Nazahn")
    } else {
        None
    }
}

/// **Godo, Bandit Warlord** ({5}{R}, 3/3, legendary Human Barbarian): when Godo enters (the card or Helm's token
/// copy), search the library for an Equipment, put it onto the battlefield, shuffle (the enter triggers of that
/// Equipment, Hammer's attach, run as it enters). Whenever Godo attacks for the first time each turn, untap it and
/// every Samurai, and there is an additional combat phase (`extra_combats`, which the engine caps at four combats a
/// turn: Godo's loop needs more, see engine-needs-godo.md).
fn godo_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src || !live(g, src) {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "search for an Equipment, put it onto the battlefield", Some(6.0))? {
        return Ok(());
    }
    let want = godo_wants(g, o);
    let pred = move |g: &Game, c: CardId| match want {
        Some(n) => &*g.db.get(c).name == n,
        None => is_equipment_card(g, c),
    };
    let got = super::t1::tutor_named(g, o, &pred, 1, "battlefield")?;
    if human_of(g, o).is_some() {
        // the human's pick may still be in the library list
        for &c in &got {
            let lib = &mut g.player_mut(o).library;
            if let Some(i) = lib.iter().position(|&x| x == c) {
                lib.remove(i);
            }
        }
        shuffle_library(g, o);
    }
    for c in got {
        crate::glog!(g, "    Godo puts {} onto the battlefield", g.db.get(c).name);
        enter(g, o, c, Enter { sick: false, ..Enter::default() })?;
    }
    Ok(())
}

fn godo_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p
        || !atk.contains(&src)
        || g.player(p).flag_turn.get(first_attack_key(src)) == Some(&g.turn_stamp())
    {
        return Ok(vec![]);
    }
    let ok = trigger_window(g, p, Some(src), "untap Godo and the Samurai, additional combat", Some(6.0))?;
    once_per_turn(g, p, first_attack_key(src));
    if !ok {
        return Ok(vec![]);
    }
    for m in g.player(p).perms.clone() {
        if live(g, m) && g.is_creature(m) && (m == src || super::tymna_cards::is_samurai(g, m)) {
            g.perm_mut(m).tapped = false;
        }
    }
    g.player_mut(p).extra_combats += 1;
    crate::glog!(g, "    Godo untaps; an additional combat phase follows");
    Ok(vec![])
}


// ================================================================== Hammer of Nazahn
/// the creature an Equipment is attached to: Godo first (Helm's token copies the equipped creature, so Godo is the
/// one to wear it), else the strongest creature that can attack (not summoning sick), else the strongest
fn attach_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m) && owner(g, m) == p).collect();
    max_by(&cs, |m| {
        let godo = if name_of(g, m) == "Godo, Bandit Warlord" { 100.0 } else { 0.0 };
        godo + if g.perm(m).sick { 0.0 } else { 10.0 } + epow(g, m) as f64
    })
}

/// **Hammer of Nazahn** ({4} legendary Equipment): whenever Hammer or another Equipment you control enters, you may
/// attach that Equipment to target creature you control (the AI always does, to `attach_target`); equipped creature
/// gets +2/+0 and indestructible, equip {4} (the compiled abilities).
fn hammer_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if !live(g, src) || !live(g, m) || owner(g, m) != o || !g.perm(m).cd.is_some_and(|c| is_equipment_card(g, c)) {
        return Ok(());
    }
    let Some(t) = attach_target(g, o) else { return Ok(()) };
    let name = format!("attach {} to {}", g.perm(m).name, g.perm(t).name);
    if !trigger_window(g, o, Some(src), &name, Some(5.0))? {
        return Ok(());
    }
    if live(g, m) && live(g, t) && g.is_creature(t) {
        super::cloud::attach(g, m, t);
    }
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Godo, Bandit Warlord")?;
    c.etb = Some(godo_etb);
    c.attack = Some(godo_attack);
    r.card(db, "Hammer of Nazahn")?.etb = Some(hammer_etb);
    Ok(())
}
