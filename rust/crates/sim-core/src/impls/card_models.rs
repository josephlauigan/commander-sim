//! Card code for the Partial cards of JD's decks that the ability language can't express (card-models PRs, 10-30).
//! Each function says which Oracle sentence it plays; `tests/cards_jd_models.rs` checks them.

use crate::cards::CardDb;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::trigger_window;
use crate::engine::tutors::card_worth;
use crate::engine::values::threat;
use crate::engine::zones::{draw, min_by};
use crate::flow::Res;
use crate::hooks::{AltCost, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};

// ------------------------------------------------------------------ Baleful Mastery (Sauron)
const BALEFUL: &str = "Baleful Mastery";

/// "You may pay {1}{B} rather than pay this spell's mana cost."
fn baleful_ok(g: &Game, p: PlayerId, _c: CardId, _spell: Option<CardId>) -> bool {
    can_pay(g, p, 1, "B", false)
}

/// pays {1}{B} and remembers it for the resolution ("if the {1}{B} cost was paid, an opponent draws a card")
fn baleful_pay(g: &mut Game, p: PlayerId, _c: CardId, _spell: Option<CardId>) -> Res<bool> {
    if !pay(g, p, 1, "B", false)? {
        return Ok(false);
    }
    let st = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(intern("baleful_alt"), st);
    Ok(true)
}

/// an opponent's card: about half a card of ours. The AI takes the cheap cost only when {3}{B} can't be paid
/// (altcost::use_alt)
const BALEFUL_ALT: AltCost = AltCost { ok: baleful_ok, pay: baleful_pay, lost: |_, _, _, _| 0.5 };

/// "If the {1}{B} cost was paid, an opponent draws a card. Exile target creature or planeswalker." The opponent who
/// draws is the one the caster fears least; then the compiled exile resolves
fn baleful_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let st = g.turn_stamp();
    if g.player_mut(p).flag_turn.shift_remove("baleful_alt") == Some(st) {
        let opps: Vec<PlayerId> = g.opps(p).collect();
        if let Some(q) = min_by(&opps, |q| threat(g, p, q)) {
            crate::glog!(
                g,
                "  {} paid {{1}}{{B}} for Baleful Mastery: {} draws a card",
                g.player(p).name,
                g.player(q).name
            );
            draw(g, q, 1, false)?;
        }
    }
    crate::dsl::resolve_spell(g, p, c, ctx)?;
    Ok("gy")
}

// ------------------------------------------------------------------ Dragonlord Ojutai (The Ur-Dragon)
const OJUTAI: &str = "Dragonlord Ojutai";

/// "Dragonlord Ojutai has hexproof as long as it's untapped."
fn ojutai_hexproof(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "hexproof" && m == src && !g.perm(src).tapped
}

/// "Whenever Dragonlord Ojutai deals combat damage to a player, look at the top three cards of your library. Put one
/// of them into your hand and the rest on the bottom of your library in any order." The AI keeps the card worth most
/// (tutors::card_worth); a person picks
fn ojutai_hit(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if a != src || p != g.perm(src).owner {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "look at the top three, one to hand", None)? {
        return Ok(());
    }
    let n = g.player(p).library.len().min(3);
    if n == 0 {
        return Ok(());
    }
    let top: Vec<CardId> = (0..n).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
    let k = if crate::human::is_human(g, p) {
        let labels: Vec<String> = top.iter().map(|&c| crate::human::choices::card_label(g, c)).collect();
        let pr = "Dragonlord Ojutai: put which card into your hand? (the rest go to the bottom)";
        crate::human::choices::choose(g, p, "choose", pr, &labels, None, None)?.unwrap_or(0)
    } else {
        let w: Vec<f64> = top.iter().map(|&c| card_worth(g, p, c, false)).collect();
        (0..n).max_by(|&i, &j| w[i].total_cmp(&w[j]).then(j.cmp(&i))).unwrap_or(0)
    };
    let pl = g.player_mut(p);
    pl.hand.push(top[k]);
    for (i, &c) in top.iter().enumerate() {
        if i != k {
            pl.library.insert(0, c);
        }
    }
    pl.stat("jr_ojutai_digs", 1);
    crate::glog!(g, "  Dragonlord Ojutai: {} takes one of the top three", g.player(p).name);
    Ok(())
}

// ------------------------------------------------------------------ Gemstone Caverns (Thanos)
const CAVERNS: &str = "Gemstone Caverns";

/// "If this card is in your opening hand and you're not the starting player, you may begin the game with Gemstone
/// Caverns on the battlefield with a luck counter on it. If you do, exile a card from your hand." Called for each AI
/// seat once its mulligans are done (`turn::setup_game`; seat 0 starts). The AI always does it (a free land that
/// makes any colour, for its worst card: the card it would put on the bottom of a mulligan)
pub fn caverns_opening(g: &mut Game, p: PlayerId) {
    if p.index() == 0 {
        return;
    }
    let Some(c) = g.db.id(CAVERNS).filter(|c| g.player(p).hand.contains(c)) else { return };
    let pl = g.player_mut(p);
    pl.hand.retain(|&x| x != c);
    let l = g.add_land(p, c, false);
    g.perm_mut(l).data.set(DataKey::Counters, Val::Int(1));
    if let Some(x) = crate::engine::turn::bottom_pick(g, p) {
        let pl = g.player_mut(p);
        pl.hand.retain(|&y| y != x);
        pl.exile.push(x);
    }
    g.player_mut(p).stat("jr_caverns_opening", 1);
    crate::glog!(g, "  {} begins the game with Gemstone Caverns (a luck counter), exiling a card", g.player(p).name);
}

// ------------------------------------------------------------------ Buster Sword (Cloud)
const BUSTER: &str = "Buster Sword";

/// the spell in hand Buster Sword would cast free for `dmg` damage: a permanent spell, or an instant or sorcery
/// worth casting free (t0_stella::free_worth); the highest mana value first, then the card worth most
fn buster_pick(g: &Game, p: PlayerId, dmg: i32) -> Option<CardId> {
    g.player(p)
        .hand
        .iter()
        .copied()
        .filter(|&x| {
            let d = g.db.get(x);
            !d.land && (d.cmc as i32) <= dmg && (d.perm || crate::impls::t0_stella::free_worth(g, p, x).is_some())
        })
        .filter(|&x| !g.cast_checks() || crate::engine::cast::castable(g, p, x, "hand"))
        .max_by(|&a, &b| {
            let k = |x: CardId| g.db.get(x).cmc as f64 * 3.0 + card_worth(g, p, x, false) * 0.01;
            k(a).total_cmp(&k(b))
        })
}

/// "Whenever equipped creature deals combat damage to a player, draw a card, then you may cast a spell from your
/// hand with mana value less than or equal to that damage without paying its mana cost." (The +3/+2 and equip {2}
/// are compiled.) A person picks the spell, or none
fn buster_hit(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, dmg: i32) -> Res {
    if g.perm(src).attached != Some(a) || g.perm(src).owner != p {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "draw a card, then a free spell", None)? {
        return Ok(());
    }
    draw(g, p, 1, false)?;
    let pick = if crate::human::is_human(g, p) {
        let cs: Vec<CardId> = g
            .player(p)
            .hand
            .iter()
            .copied()
            .filter(|&x| !g.db.get(x).land && (g.db.get(x).cmc as i32) <= dmg)
            .collect();
        let labels: Vec<String> = cs.iter().map(|&c| crate::human::choices::card_label(g, c)).collect();
        let pr = format!("Buster Sword: cast a spell with mana value {dmg} or less free?");
        crate::human::choices::choose(g, p, "choose", &pr, &labels, Some("none"), None)?.map(|k| cs[k])
    } else {
        buster_pick(g, p, dmg)
    };
    let Some(x) = pick else { return Ok(()) };
    g.player_mut(p).hand.retain(|&y| y != x);
    g.player_mut(p).stat("jr_buster_free", 1);
    if g.db.get(x).perm {
        g.last_x = 0;
        crate::glog!(g, "    {} casts {} free (Buster Sword)", g.player(p).name, g.db.get(x).name);
        crate::engine::cast::cast_card(g, p, x, "lib", Ctx::default())?;
    } else {
        crate::impls::t0_stella::free_cast(g, p, x, false)?;
    }
    Ok(())
}
// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, BALEFUL)?;
    c.alt_cost = Some(BALEFUL_ALT);
    c.resolve = Some(baleful_resolve);
    let c = r.card(db, OJUTAI)?;
    c.grant_kw = Some(ojutai_hexproof);
    c.combat_damage = Some(ojutai_hit);
    r.card(db, BUSTER)?.combat_damage = Some(buster_hit);
    Ok(())
}
