//! Sorin, Vengeful Bloodlord (Yoshimaru/Tymna deck): {2}{W}{B}, loyalty 4.
//!
//! - "During your turn, creatures and planeswalkers you control have lifelink": a `grant_kw` hook. The engine reads
//!   lifelink through `chars::has_keyword`, so combat damage and every damage site that reads
//!   `life::has_lifelink` see it; a phased-out or neutered Sorin grants nothing (`hooks::hooked`).
//! - "+2: Sorin deals 1 damage to target player or planeswalker": the AI aims it at a player a point from dying, an
//!   opposing planeswalker with 1 loyalty, else the opponent that threatens it most. Sorin is a planeswalker you control,
//!   so on your turn the damage gains you 1.
//! - "-X: Return target creature card with mana value X from your graveyard to the battlefield. That creature is a
//!   Vampire in addition to its other types": one AI ability per X (0 to 8, since the walker framework wants a fixed
//!   loyalty cost); the creature enters as a Vampire (a copy exception, applied before its enter triggers). A person
//!   chooses the card, and so X (`human::walkers`).

use crate::cards::{CardDb, Types};
use crate::engine::chars::card_types;
use crate::engine::values::{colors_of, player_hexproof_from, threat, untargetable};
use crate::engine::zones::{Enter, enter, max_by};
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::human::{Target, choices, legal};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::common::{WalkerAb, walker};
use crate::state::{CopyExc, Game};
use crate::sym::{Sym, intern};

pub const SORIN: &str = "Sorin, Vengeful Bloodlord";

/// during its controller's turn, the creatures and planeswalkers they control have lifelink
fn grant(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    if kw != "lifelink" {
        return false;
    }
    let o = g.perm(src).owner;
    g.active == Some(o) && g.perm(m).owner == o && (g.is_creature(m) || card_types(g, m).has(Types::PLANESWALKER))
}

// ------------------------------------------------------------------ +2
/// who the AI's +2 hits: a player at 1 life, an opposing planeswalker at 1 loyalty, else the most threatening opponent
fn plus_target(g: &Game, p: PlayerId, src: PermId) -> Option<Target> {
    let cols = colors_of(g, src);
    let players: Vec<PlayerId> =
        g.opps(p).filter(|&q| legal::player_targetable(g, q) && !player_hexproof_from(g, q, cols)).collect();
    if let Some(&q) = players.iter().find(|&&q| g.player(q).life <= 1) {
        return Some(Target::Player(q));
    }
    for q in g.opps(p) {
        for &w in &g.player(q).perms {
            let x = g.perm(w);
            if x.on_bf
                && !x.phased
                && x.loyalty == Some(1)
                && card_types(g, w).has(Types::PLANESWALKER)
                && !untargetable(g, w)
            {
                return Some(Target::Perm(w));
            }
        }
    }
    max_by(&players, |q| threat(g, p, q)).map(Target::Player)
}

fn plus_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    match plus_target(g, p, src)? {
        Target::Player(q) if g.player(q).life <= 1 => Some(8.0),
        _ => Some(2.0),
    }
}

fn plus_eff(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    let Some(t) = plus_target(g, p, src) else { return Ok(()) };
    let by = choices::by(g, Some(src));
    choices::apply_damage_by(g, p, t, 1, SORIN, "burn", by).map(|_| ())
}

// ------------------------------------------------------------------ -X
/// the creature card of p's graveyard with mana value `x` the AI would return: the one it values most
fn best_card(g: &Game, p: PlayerId, x: i32) -> Option<CardId> {
    let cs: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| {
            let d = g.db.get(c);
            d.creature && d.cmc as i32 == x
        })
        .collect();
    max_by(&cs, |c| crate::dsl::card_value(g, p, c))
}

/// the creature cards of p's graveyard with mana value `x` or less (any, when `x` is None)
pub fn candidates(g: &Game, p: PlayerId, x: Option<i32>, up_to: i32) -> Vec<CardId> {
    g.player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| {
            let d = g.db.get(c);
            d.creature && x.map_or(d.cmc as i32 <= up_to, |x| d.cmc as i32 == x)
        })
        .collect()
}

/// the card returns to the battlefield as a Vampire in addition to its other types (as it enters, so its own enter
/// triggers see it); false if it is no longer in the graveyard
pub fn return_as_vampire(g: &mut Game, p: PlayerId, c: CardId) -> Res<bool> {
    let Some(i) = g.player(p).gy.iter().position(|&x| x == c) else { return Ok(false) };
    g.player_mut(p).gy.remove(i);
    let exc = CopyExc { add_subtype: Some(intern("vampire")), ..CopyExc::default() };
    enter(g, p, c, Enter { exc: Some(exc), ..Enter::default() })?;
    crate::glog!(g, "    {} returns to the battlefield as a Vampire (Sorin)", g.db.get(c).name);
    Ok(true)
}

fn minus_val<const X: i32>(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    let c = best_card(g, p, X)?;
    let dies = g.perm(src).loyalty.unwrap_or(0) <= X; // Sorin goes with his last loyalty: worth less
    Some((1.5 + crate::dsl::card_value(g, p, c) * 0.5 + 0.3 * X as f64 - if dies { 2.0 } else { 0.0 }).max(0.1))
}

fn minus_eff<const X: i32>(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(c) = best_card(g, p, X) {
        return_as_vampire(g, p, c)?;
    }
    Ok(())
}

macro_rules! minus {
    ($x:literal) => {
        WalkerAb {
            delta: -$x,
            label: "return a creature card of that mana value from the graveyard as a Vampire",
            val: minus_val::<$x>,
            eff: minus_eff::<$x>,
        }
    };
}

static SORIN_AB: [WalkerAb; 10] = [
    WalkerAb { delta: 2, label: "1 damage to a player or planeswalker", val: plus_val, eff: plus_eff },
    minus!(0),
    minus!(1),
    minus!(2),
    minus!(3),
    minus!(4),
    minus!(5),
    minus!(6),
    minus!(7),
    minus!(8),
];

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, SORIN)?.grant_kw = Some(grant);
    walker(r, db, SORIN, &SORIN_AB)
}
