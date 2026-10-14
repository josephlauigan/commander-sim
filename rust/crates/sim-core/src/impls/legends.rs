//! Card code for legendary and multicolour cards that decks of any kind may play: Jodah, the Unifier (the card) and
//! cards that were first modeled for a legends list (Garland, Tymna, the Kaldra pieces, Dakkon, Mordenkainen, Wrenn and
//! Six, Memory Jar, Profane Tutor and suspend, ...). Everything here is keyed by card name; no deck's own AI lives here.
//!
//! - Jodah: legendary creatures you control get +X/+X (X = the legendary creatures you control); whenever you cast a
//!   legendary spell from your hand, exile from the top until a legendary nonland card with lesser mana value, cast it
//!   free, the rest to the bottom in a random order.
//! - Command Beacon, Maelstrom Nexus and Wanderer, Sisay, Shalai, Venat, Tymna, Garland, and the rest of the cards that
//!   need code.
//!
//! Plaza of Heroes' and Unclaimed Territory's colours (Python's mine.plaza_colors and territory_colors, which
//! engine._land_cols calls by name) are here too.

use super::common::{WalkerAb, best_opp_creature, best_opp_nonland, oring_exile, oring_return, walker};
use super::lands::{can_pay_without, sac_land};
use super::partials::{at_once, eot_kw, first_max, first_min, on, pack, remove_card, unpack};
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, castable, copy_spell, on_cast};
use crate::engine::life::{check_state, gain, lose_life};
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{ability_window, ability_window_card, counter_window, equip_to, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library, tutor};
use crate::engine::values::{
    board_power, colors_of, epow, etgh, once_per_turn, player_hexproof, protected_from, pval, threat, untargetable,
};
use crate::engine::zones::{
    Enter, Tokens, agent_for, agent_take, die, discard_cards, draw, enter, exile_perm, landfall, leave,
    make_artifact_tokens, make_tokens, max_by, min_by, searchable,
};
use crate::flow::Res;
use crate::hooks::{Action, Assign, Event, Opt, Registry, Src};
use crate::human::{cards, choices, human_of};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, PermData, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

pub const JODAH: &str = "Jodah, the Unifier";

// ------------------------------------------------------------------ helpers
/// a permanent's card name ("" for a token)
fn name_of(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn stat(g: &mut Game, p: PlayerId, name: &str, n: i64) {
    g.player_mut(p).stat(intern(name), n);
}

/// jodah.legendary: a legendary permanent: printed, a legendary token (Kaldra, Voja), or the Ring-bearer
pub fn legendary(g: &Game, m: PermId) -> bool {
    if g.perm(m).cd.is_none() {
        return g.perm(m).data.truthy(DataKey::Legendary);
    }
    super::mine::is_legendary(g, m)
}

/// jodah.legend_card: a legendary nonland card
pub fn legend_card(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.tag(Tag::Leg) && !d.land
}

/// jodah.jodahs: the Jodahs p controls (not phased out)
pub fn jodahs(g: &Game, p: PlayerId) -> i32 {
    g.player(p).perms.iter().filter(|&&m| name_of(g, m) == JODAH && !g.perm(m).phased).count() as i32
}

/// jodah._the_jodah
fn the_jodah(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| name_of(g, m) == JODAH && !g.perm(m).phased)
}

/// jodah._attached: the Equipment named `name` on m (its controller's, not phased out)
fn attached_named(g: &Game, m: PermId, name: &str) -> Vec<PermId> {
    let o = owner(g, m);
    g.player(o)
        .perms
        .iter()
        .copied()
        .filter(|&e| name_of(g, e) == name && g.perm(e).attached == Some(m) && !g.perm(e).phased)
        .collect()
}

/// jodah._mine_named: p's permanents named `name` (not phased out)
fn mine_named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| name_of(g, m) == name && !g.perm(m).phased).collect()
}

/// jodah._best_legend: p's best legendary creature (an unsick one first)
fn best_legend(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased && legendary(g, m))
        .collect();
    first_max(&cs, |m| (!g.perm(m).sick, pval(g, m)))
}

/// put cards at the bottom of p's library (Python's `p.library[:0] = cards`)
fn to_bottom(g: &mut Game, p: PlayerId, cards: Vec<CardId>) {
    let lib = &mut g.player_mut(p).library;
    let rest = std::mem::replace(lib, cards);
    lib.extend(rest);
}

/// `p.library[-n:]`, taken off the library (top last)
fn take_top(g: &mut Game, p: PlayerId, n: usize) -> Vec<CardId> {
    let lib = &mut g.player_mut(p).library;
    let k = lib.len().saturating_sub(n);
    lib.split_off(k)
}

fn ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

fn plan(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

// ================================================================== Jodah
/// jodah._jodah_pt (common.CREATURE_PT): legendary creatures you control get +X/+X, X = your legendary creatures
fn jodah_pt(g: &Game, m: PermId) -> (i32, i32) {
    let p = owner(g, m);
    let k = jodahs(g, p);
    if k == 0 || !legendary(g, m) {
        return (0, 0);
    }
    let n =
        g.player(p).perms.iter().filter(|&&y| g.is_creature(y) && !g.perm(y).phased && legendary(g, y)).count() as i32;
    (k * n, k * n)
}

/// the engine reads legends' size from CREATURE_PT from here on
fn selfpt_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
    }
    Ok(())
}

fn selfpt_as(g: &mut Game, _p: PlayerId, _m: PermId) -> Res {
    g.selfpt = true;
    Ok(())
}

/// legends you control get +X/+X (X = your legendary creatures); casting a legendary spell from hand cascades into a
/// legendary nonland card with lesser mana value, cast free
fn jodah_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    if caster != o || !legend_card(g, c) {
        return Ok(());
    }
    match &g.cur_cast {
        Some((x, _, zone)) if *x == c && *zone == "hand" => {} // only spells cast from your hand
        _ => return Ok(()),
    }
    let mv = g.db.get(c).cmc;
    if !trigger_window(g, o, Some(src), &format!("cascade into a legend with mana value below {mv}"), Some(4.0))? {
        return Ok(());
    }
    jodah_cascade(g, o, mv)?;
    Ok(())
}

/// jodah.jodah_cascade: exile from the top until a legendary nonland card with mana value below mv; cast it free (you
/// may); the rest go to the bottom in a random order
pub fn jodah_cascade(g: &mut Game, o: PlayerId, mv: u32) -> Res<bool> {
    cascade(g, o, mv, "Jodah")
}

/// jodah.cascade: Jodah's legend cascade (who 'Jodah') or plain cascade (Maelstrom Nexus, Maelstrom Wanderer): exile
/// from the top until a nonland card (legendary, for Jodah) with mana value below mv; cast it free (you may: the AI
/// passes on counterspells and on the protection it keeps for responses); the rest go to the bottom in a random
/// order. A person says whether to cast it.
pub fn cascade(g: &mut Game, o: PlayerId, mv: u32, who: &str) -> Res<bool> {
    let jodah = who == "Jodah";
    let mut seen = vec![];
    let mut hit = None;
    while let Some(x) = g.player_mut(o).library.pop() {
        let d = g.db.get(x);
        if !d.land && d.cmc < mv && (!jodah || legend_card(g, x)) {
            hit = Some(x);
            break;
        }
        seen.push(x);
    }
    let mut cast = false;
    if let Some(h) = hit {
        let want = if human_of(g, o).is_some() {
            let q = format!("{who}: cast {} without paying its mana cost?", cname(g, h));
            choices::yes_no(g, o, &q)?
        } else {
            !g.db.get(h).tag(Tag::Ctr)
        };
        if want && castable(g, o, h, "lib") {
            g.last_x = 0; // cast free: X is 0
            if jodah {
                stat(g, o, "jodah_cascades", 1);
            } else {
                stat(g, o, "jr_cascade_hits", 1);
            }
            crate::glog!(g, "    {who} reveals {} ({} other cards): cast free", cname(g, h), seen.len());
            cast_card(g, o, h, "lib", Ctx::default())?;
            cast = true;
        } else {
            crate::glog!(g, "    {who} reveals {} ({} other cards): not cast", cname(g, h), seen.len());
            seen.push(h);
        }
    } else {
        // nothing cheaper: the whole library is exiled, then goes back
        crate::glog!(
            g,
            "    {who} exiles {} cards and finds no {}nonland card with mana value below {mv}: they all go to the bottom \
             in a random order (the library is shuffled)",
            seen.len(),
            if jodah { "legendary " } else { "" }
        );
    }
    g.rng.shuffle(&mut seen);
    to_bottom(g, o, seen); // to the bottom, random order
    Ok(cast)
}

/// jodah._jodah_from_hand (CI.SELF_CAST): counts Jodah cast from hand (Command Beacon's payoff)
fn jodah_from_hand(g: &mut Game, p: PlayerId, _c: CardId) -> Res {
    if g.cur_cast.as_ref().is_some_and(|x| x.2 == "hand") {
        stat(g, p, "jr_jodah_from_hand", 1);
    }
    Ok(())
}

// ================================================================== the AI
/// jodah.jodah_payable: Jodah is in the command zone (or in hand: Command Beacon) and can be cast now
pub fn jodah_payable(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    if !(pl.cmd_in_zone || pl.hand.contains(&pl.cmd)) {
        return false;
    }
    let (gn, pips) = cost_of(g, p, pl.cmd);
    can_pay(g, p, gn, &pips, false)
}

const KALDRA: [&str; 3] = ["Sword of Kaldra", "Shield of Kaldra", "Helm of Kaldra"];

/// Mithril Coat: attach it to a legendary creature you control as it enters: Jodah first
fn coat_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src && g.perm(src).attached.is_none() {
        let o = owner(g, src);
        let to = the_jodah(g, o).or_else(|| best_legend(g, o));
        g.perm_mut(src).attached = to;
    }
    Ok(())
}

// ================================================================== the rework candidates (audit/jodah, 10-09)
// ------------------------------------------------------------------ Command Beacon
/// {T}, sacrifice: the commander from the command zone to hand, where casting it pays no commander tax. Used on your
/// turn when Jodah is castable from hand after the sacrifice, and the tax is 4 or more, or it is 2 and Jodah can't be
/// cast from the command zone this turn
fn beacon(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let pl = g.player(p);
    if post.is_none() || g.land(l).tapped || g.active != Some(p) || !pl.cmd_in_zone || pl.tax < 2 {
        return Ok(vec![]);
    }
    let cmd = pl.cmd;
    let tax = pl.tax;
    let (gn, pips) = cost_of(g, p, cmd);
    g.pay_for = Some(cmd); // Plaza of Heroes' colours count
    let after = can_pay_without(g, p, l, gn.saturating_sub(tax), &pips);
    let now = can_pay(g, p, gn, &pips, false);
    g.pay_for = None;
    if !after || (now && tax < 4) {
        return Ok(vec![]);
    }
    let label = format!("Command Beacon: {} to hand", cname(g, cmd));
    Ok(vec![plan(9.5, label, beacon_go, l.0 as i64)])
}

fn beacon_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    if !(g.land(l).on_bf && g.player(p).lands.contains(&l)) || g.land(l).tapped || !g.player(p).cmd_in_zone {
        return Ok(false);
    }
    let cd = g.land(l).cd;
    sac_land(g, p, l)?;
    stat(g, p, "jr_beacon", 1);
    let cmd = g.player(p).cmd;
    crate::glog!(
        g,
        "  {} sacrifices Command Beacon: {} to hand (tax {} saved)",
        pname(g, p),
        cname(g, cmd),
        g.player(p).tax
    );
    if ability_window_card(g, p, cd, &format!("{} to hand", cname(g, cmd)), None, None)? && g.player(p).cmd_in_zone {
        let pl = g.player_mut(p);
        pl.cmd_in_zone = false;
        pl.hand.push(cmd);
    }
    Ok(true)
}

// ------------------------------------------------------------------ Maelstrom Nexus, Maelstrom Wanderer
/// the first spell you cast each turn has cascade (a legend from hand with Jodah out cascades twice)
fn nexus(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    let st = g.turn_stamp();
    let first = match &g.player(o).turn_casts {
        Some((t, cs)) => *t == st && cs.first() == Some(&c),
        None => false,
    };
    if caster != o || !first {
        return Ok(());
    }
    let mv = g.db.get(c).cmc;
    if !trigger_window(g, o, Some(src), &format!("cascade (mana value below {mv})"), Some(4.0))? {
        return Ok(());
    }
    stat(g, o, "jr_nexus", 1);
    cascade(g, o, mv, "Maelstrom Nexus")?;
    Ok(())
}

fn prio_50(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    50
}

/// Maelstrom Wanderer: cascade, cascade (cast triggers: they resolve before the Wanderer)
fn wanderer_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let mv = g.db.get(c).cmc;
    let label = format!("{}: cascade", cname(g, c));
    for _ in 0..2 {
        if !g.player(p).alive || g.over || !trigger_window(g, p, None, &label, Some(4.0))? {
            continue;
        }
        stat(g, p, "jr_wanderer", 1);
        cascade(g, p, mv, "Maelstrom Wanderer")?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Sisay, Weatherlight Captain
/// +1/+1 for each colour among your other legendary permanents
fn sisay_pt(g: &Game, p: PlayerId, m: PermId) -> (i32, i32) {
    let mut cols = Colors::NONE;
    for &x in &g.player(p).perms {
        if x != m && !g.perm(x).phased && legendary(g, x) {
            cols = cols.union(colors_of(g, x));
        }
    }
    let n = cols.count() as i32;
    (n, n)
}

/// jodah.sisay_pick: a legendary permanent card with mana value below Sisay's power: the dearest legendary creature, else
/// the dearest legend
pub fn sisay_pick(g: &Game, p: PlayerId, power: i32) -> Option<CardId> {
    let ok: Vec<CardId> = searchable(g, p)
        .into_iter()
        .filter(|&c| {
            let d = g.db.get(c);
            d.tag(Tag::Leg) && (d.perm || d.creature) && !d.land && (d.cmc as i32) < power
        })
        .collect();
    if ok.is_empty() {
        return None;
    }
    first_max(&ok, |c| {
        let d = g.db.get(c);
        (d.creature, d.cmc, card_worth(g, p, c, false))
    })
}

/// {W}{U}{B}{R}{G}: a legendary permanent card with mana value below Sisay's power onto the battlefield. On your turn,
/// never while Jodah could be cast with the same mana
fn sisay(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || post.is_none() || g.active != Some(p) || !can_pay(g, p, 0, "WUBRG", false) {
        return Ok(vec![]);
    }
    if jodah_payable(g, p) {
        return Ok(vec![]);
    }
    let Some(t) = sisay_pick(g, p, epow(g, src)) else { return Ok(vec![]) };
    let u = 7.0f64.min(1.5 + 0.6 * g.db.get(t).cmc as f64);
    Ok(vec![ability(u, format!("Sisay: {}", cname(g, t)), src, sisay_go, t.0 as i64)])
}

fn sisay_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = CardId(arg as u16);
    if !on(g, p, src) || !can_pay(g, p, 0, "WUBRG", false) || !g.player(p).library.contains(&t) {
        return Ok(false);
    }
    pay(g, p, 0, "WUBRG", false)?;
    stat(g, p, "jr_sisay", 1);
    crate::glog!(g, "  {} activates Sisay: {}", pname(g, p), cname(g, t));
    if !ability_window(g, p, Some(src), &format!("search for {}", cname(g, t)), None, None)?
        || !g.player(p).library.contains(&t)
    {
        return Ok(true);
    }
    remove_card(&mut g.player_mut(p).library, t);
    shuffle_library(g, p);
    match agent_for(g, p) {
        Some(a) => agent_take(g, a, p, t),
        None => {
            enter(g, p, t, Enter::default())?;
        }
    }
    Ok(true)
}

// ------------------------------------------------------------------ Shalai, Voice of Plenty (packages C and D)
const SHALAI: &str = "Shalai, Voice of Plenty";

/// your planeswalkers and other creatures have hexproof (Shalai herself doesn't)
fn shalai_hexproof(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "hexproof"
        && m != src
        && owner(g, m) == owner(g, src)
        && (g.is_creature(m) || g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER)))
}

fn shalai_you(g: &Game, src: Src, q: PlayerId) -> bool {
    q == owner(g, src)
}

/// {4}{G}{G}: a +1/+1 counter on each creature you control; with mana left late (second main phase, or the end of
/// the turn before yours) and two or more creatures
fn shalai_counters(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let n = g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count();
    if post == Some(false) || n < 2 || !can_pay(g, p, 4, "GG", false) {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.0 + 0.5 * n as f64, "Shalai: +1/+1 counters".into(), src, shalai_go, 0)])
}

fn shalai_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !on(g, p, src) || !can_pay(g, p, 4, "GG", false) {
        return Ok(false);
    }
    pay(g, p, 4, "GG", false)?;
    if !ability_window(g, p, Some(src), "+1/+1 counters", None, None)? {
        return Ok(true);
    }
    for m in g.player(p).perms.clone() {
        if g.is_creature(m) && !g.perm(m).phased {
            crate::cardcode::put_plus(g, m, 1);
        }
    }
    stat(g, p, "jr_shalai_counters", 1);
    crate::glog!(g, "  {} activates Shalai: a +1/+1 counter on each creature", pname(g, p));
    Ok(true)
}

// ------------------------------------------------------------------ Venat, Heart of Hydaelyn // Hydaelyn (10-09)
const VENAT: &str = "Venat, Heart of Hydaelyn";

fn hydaelyn(g: &Game, m: PermId) -> bool {
    g.perm(m).data.truthy(DataKey::Hydaelyn)
}

/// Venat: whenever you cast a legendary spell, draw a card; only once each turn
fn venat_draw(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    if caster != o
        || hydaelyn(g, src)
        || !legend_card(g, c)
        || g.player(o).flag_turn.get("venat") == Some(&g.turn_stamp())
    {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "draw a card", None)? || !once_per_turn(g, o, "venat") {
        return Ok(());
    }
    draw(g, o, 1, false)?;
    stat(g, o, "jr_venat_draws", 1);
    Ok(())
}

/// Hero's Sundering: {7}, {T}, as a sorcery: exile target nonland permanent, then Venat transforms. On the best
/// opposing nonland permanent, when it is worth 3 or more; never while Jodah could be cast instead
fn sundering(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if p != x.owner
        || post.is_none()
        || g.active != Some(p)
        || hydaelyn(g, src)
        || x.tapped
        || x.sick
        || !can_pay(g, p, 7, "", false)
        || (cname(g, g.player(p).cmd) == JODAH && jodah_payable(g, p))
    {
        return Ok(vec![]);
    }
    let Some(t) = best_opp_nonland(g, p, |_| true) else { return Ok(vec![]) };
    if pval(g, t) < 3.0 {
        return Ok(vec![]);
    }
    let leader = leader(g, p);
    let u = 1.0 + pval(g, t) * if Some(owner(g, t)) == leader { 1.25 } else { 1.0 };
    Ok(vec![ability(u, format!("Hero's Sundering -> {}", g.perm(t).name), src, sundering_go, t.0 as i64)])
}

/// brain.Situation's leader: the opponent with the most threat
fn leader(g: &Game, p: PlayerId) -> Option<PlayerId> {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    max_by(&opps, |q| threat(g, p, q))
}

fn sundering_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    if !on(g, p, src) || g.perm(src).tapped || !g.perm(t).on_bf || !can_pay(g, p, 7, "", false) {
        return Ok(false);
    }
    pay(g, p, 7, "", false)?;
    g.perm_mut(src).tapped = true;
    stat(g, p, "jr_venat_sunder", 1);
    crate::glog!(g, "  {} activates Hero's Sundering: exile {}, transform Venat", pname(g, p), g.perm(t).name);
    let label = format!("exile {}, transform", g.perm(t).name);
    if !ability_window(g, p, Some(src), &label, None, Some(t))? {
        return Ok(true);
    }
    if !g.perm(t).on_bf || untargetable(g, t) {
        return Ok(true); // no legal target: it does nothing
    }
    let cd = g.perm(src).cd;
    apply_removal(g, Some(p), t, "exile", cd)?;
    if on(g, p, src) {
        let x = g.perm_mut(src);
        x.data.set(DataKey::Hydaelyn, Val::Bool(true));
        x.pow = 4;
        x.tgh = 4;
        crate::glog!(g, "    Venat transforms into Hydaelyn, the Mothercrystal");
    }
    Ok(true)
}

fn hydaelyn_indestructible(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "indestructible" && m == src && hydaelyn(g, src)
}

/// Hydaelyn, beginning of combat on your turn: a +1/+1 counter on another creature you control, indestructible until
/// your next turn, a card if it is legendary. Jodah first, else the most valuable (a legend counts 2 more)
fn blessing(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !hydaelyn(g, src) || !once_per_turn(g, p, "blessing") {
        return Ok(());
    }
    stat(g, p, "jr_hyd_turns", 1);
    let cs: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && m != src && !g.perm(m).phased).collect();
    let Some(t) = first_max(&cs, |m| (name_of(g, m) == JODAH, pval(g, m) + if legendary(g, m) { 2.0 } else { 0.0 }))
    else {
        return Ok(());
    };
    let label = format!("+1/+1 counter, indestructible: {}", g.perm(t).name);
    if !trigger_window(g, p, Some(src), &label, None)? || !on(g, p, t) {
        return Ok(());
    }
    let turns = g.player(p).turns as i64;
    let x = g.perm_mut(t);
    x.plus += 1;
    x.data.set(DataKey::IndestrUntil, Val::List(vec![Val::Player(p), Val::Int(turns)])); // engine.indestructible
    crate::glog!(
        g,
        "    Hydaelyn blesses {}: a +1/+1 counter, indestructible until {}'s next turn",
        g.perm(t).name,
        pname(g, p)
    );
    if legendary(g, t) {
        draw(g, p, 1, false)?;
        stat(g, p, "jr_hyd_draws", 1);
    }
    Ok(())
}

// ------------------------------------------------------------------ Tymna the Weaver (10-09)
/// (bookkeeping: the opponents Tymna's controller dealt combat damage this turn)
fn tymna_hit(g: &mut Game, src: Src, p: PlayerId, _a: PermId, d: PlayerId, _dmg: i32) -> Res {
    if p == owner(g, src) && d != p {
        let st = g.turn_stamp();
        g.player_mut(p).flag_turn.insert(intern(&format!("hit{}", d.index())), st);
    }
    Ok(())
}

/// at the beginning of your postcombat main phase: you may pay X life to draw X (X = opponents dealt combat damage
/// this turn). The AI pays when the life left is at least Necropotence's floor (what the table could hit it for plus
/// 6, at least 10) and the library has more than X cards. A person decides.
fn tymna(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let st = g.turn_stamp();
    if p != owner(g, src) || g.player(p).flag_turn.get("tymna") == Some(&st) {
        return Ok(());
    }
    let x = (0..g.players.len())
        .filter(|&i| {
            let q = PlayerId(i as u8);
            q != p && g.player(q).alive && g.player(p).flag_turn.get(&*format!("hit{i}")) == Some(&st)
        })
        .count() as i32;
    stat(g, p, "ty_tymna_turns", 1);
    stat(g, p, "ty_opps_hit", x as i64);
    if x == 0 {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), &format!("pay {x} life, draw {x}"), None)? || !once_per_turn(g, p, "tymna") {
        return Ok(());
    }
    if g.player(p).life < x || g.player(p).life_locked {
        return Ok(());
    }
    let pay_it = if human_of(g, p).is_some() {
        choices::yes_no(g, p, &format!("Tymna the Weaver: pay {x} life to draw {x}?"))?
    } else {
        g.player(p).life - x >= crate::ai::necro_floor(g, p) && g.player(p).library.len() > x as usize
    };
    if !pay_it {
        return Ok(());
    }
    lose_life(g, p, x, Some(p), "other", None)?;
    draw(g, p, x as u32, false)?;
    stat(g, p, "jr_tymna_draws", x as i64);
    crate::glog!(g, "    {} pays {x} life to Tymna the Weaver and draws {x}", pname(g, p));
    Ok(())
}

// ------------------------------------------------------------------ Garland, Royal Kidnapper (10-09)
const GARLAND: &str = "Garland, Royal Kidnapper";

/// jodah.garland_pick: q's creature Garland's trigger takes: the most valuable one p can target
pub fn garland_pick(g: &Game, _p: PlayerId, q: PlayerId) -> Option<PermId> {
    let ub = Colors::from_letters("UB");
    let cs: Vec<PermId> = g
        .player(q)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && !untargetable(g, m) && !protected_from(g, m, ub))
        .collect();
    max_by(&cs, |m| pval(g, m))
}

/// When Garland enters, target opponent becomes the monarch: the one whose best creature is most worth stealing
/// (then the one with the smaller board, the less likely to lose the crown). A person chooses.
fn garland_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    crate::impls::garland::entered(g, o, src);
    let mut opps: Vec<PlayerId> = g.opps(o).filter(|&q| g.player(q).alive && !player_hexproof(g, q)).collect();
    if opps.len() > 1 {
        opps.retain(|&q| g.monarch != Some(q)); // crowning the monarch changes nothing: no new steal
    }
    if opps.is_empty() {
        return Ok(());
    }
    let q = if human_of(g, o).is_some() {
        cards::pick_player(g, o, "target", "Garland: which opponent becomes the monarch?", &opps)?
    } else {
        first_max(&opps, |q| {
            let t = garland_pick(g, o, q);
            (t.map_or(-1.0, |t| pval(g, t)), -board_power(g, q))
        })
        .unwrap()
    };
    let label = format!("{} becomes the monarch", pname(g, q));
    if !trigger_window(g, o, Some(src), &label, Some(4.0))? || !g.player(q).alive {
        return Ok(());
    }
    stat(g, o, "jr_garland_etb", 1);
    crate::cardcode::become_monarch(g, q)
}

/// Whenever an opponent becomes the monarch, gain control of target creature that player controls for as long as
/// they're the monarch (garland_check ends it). A person picks the creature.
fn garland_steal(g: &mut Game, src: Src, q: PlayerId) -> Res {
    let o = owner(g, src);
    if q == o || !g.player(q).alive {
        return Ok(());
    }
    let t = if human_of(g, o).is_some() {
        let prompt = format!("Garland: gain control of which of {}'s creatures?", g.player(q).name);
        cards::pick_creature(g, o, &prompt, false, false, Some(&|g: &Game, _, m| g.perm(m).owner == q))?
    } else {
        garland_pick(g, o, q)
    };
    let Some(t) = t else { return Ok(()) };
    let label = format!("gain control of {}", g.perm(t).name);
    if !trigger_window(g, o, Some(src), &label, Some(6.0))? {
        return Ok(());
    }
    if !on(g, q, t) || untargetable(g, t) || g.monarch != Some(q) {
        return Ok(()); // the duration is over
    }
    let cd = g.perm(src).cd;
    if crate::ai::protect_response(g, q, t, "steal", Some(o), cd)? || !on(g, q, t) {
        return Ok(());
    }
    super::marchesa::steal(g, o, t, false);
    let r = g.round;
    g.garland_list.push((o, t, q, r));
    g.garland = true;
    stat(g, o, "jr_garland_steals", 1);
    let took = format!("jr_garland_took:{}", g.perm(t).name);
    stat(g, o, &took, 1);
    Ok(())
}

/// (bookkeeping: combat damage by the creatures Garland took)
fn garland_hit(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, dmg: i32) -> Res {
    if p == owner(g, src) && g.garland_list.iter().any(|x| x.1 == a) {
        stat(g, p, "jr_garland_dmg", dmg as i64);
        if !g.perm(a).data.truthy(DataKey::GarlandHit) {
            g.perm_mut(a).data.set(DataKey::GarlandHit, Val::Bool(true));
            stat(g, p, "jr_garland_hit", 1);
        }
    }
    Ok(())
}

/// jodah.garland_check: Garland's control effects end once the player a creature came from is no longer the monarch
/// (or has left the game); a creature whose owner has left the game leaves with them
pub fn garland_check(g: &mut Game) {
    let mut keep = vec![];
    for (o, t, q, r) in std::mem::take(&mut g.garland_list) {
        if !on(g, o, t) {
            stat(g, o, "jr_garland_gone", 1); // it left (or changed control again)
            continue;
        }
        if g.player(q).alive && g.monarch == Some(q) {
            keep.push((o, t, q, r));
            continue;
        }
        g.player_mut(o).perms.retain(|&x| x != t);
        g.bf_ver += 1;
        stat(g, o, "jr_garland_back", 1);
        stat(g, o, "jr_garland_rounds", g.round as i64 - r as i64);
        if g.monarch == Some(o) {
            stat(g, o, "jr_garland_back_me", 1); // (you took the crown back)
        }
        let orig = g.perm(t).orig;
        if g.player(orig).alive {
            g.perm_mut(t).owner = orig;
            g.player_mut(orig).perms.push(t);
            crate::glog!(g, "    {} returns to {} (Garland)", g.perm(t).name, pname(g, orig));
        } else {
            g.perm_mut(t).on_bf = false; // its owner left the game: so does it
        }
    }
    g.garland = !keep.is_empty();
    g.garland_list = keep;
}

// ================================================================== Kaldra
/// jodah.kaldra_exile: Sword of Kaldra: the equipped creature dealt damage to victim: exile it. True if it was exiled
pub fn kaldra_exile(g: &mut Game, src: PermId, victim: PermId) -> Res<bool> {
    if attached_named(g, src, "Sword of Kaldra").is_empty() {
        return Ok(false);
    }
    if !g.perm(victim).on_bf || protected_from(g, victim, Colors::NONE) {
        return Ok(false);
    }
    crate::glog!(g, "    Sword of Kaldra exiles {}", g.perm(victim).name);
    let (vo, name) = (owner(g, victim), g.perm(victim).name);
    *g.player_mut(vo).lost_names.entry(name).or_insert(0) += 1;
    exile_perm(g, victim)?;
    Ok(true)
}

/// Shield of Kaldra: the Kaldra equipment are indestructible (CI.SELF_REGEN)
fn kaldra_indestructible(g: &mut Game, m: PermId) -> Res<bool> {
    Ok(KALDRA.contains(&name_of(g, m)) && !mine_named(g, owner(g, m), "Shield of Kaldra").is_empty())
}

/// Helm of Kaldra: {1}: with Helm, Sword and Shield of Kaldra all out, create Kaldra (a legendary 4/4 Avatar) wearing
/// all three
fn kaldra_assemble(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || post.is_none() || !can_pay(g, p, 1, "", false) {
        return Ok(vec![]);
    }
    if kaldra_pieces(g, p).is_none() || has_kaldra(g, p) {
        return Ok(vec![]);
    }
    Ok(vec![ability(9.0, "Helm of Kaldra: create Kaldra".into(), src, kaldra_go, 0)])
}

fn kaldra_pieces(g: &Game, p: PlayerId) -> Option<Vec<PermId>> {
    KALDRA.iter().map(|n| mine_named(g, p, n).first().copied()).collect()
}

fn has_kaldra(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).cd.is_none() && g.perm(m).data.truthy(DataKey::Kaldra))
}

fn kaldra_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let Some(pieces) = kaldra_pieces(g, p) else { return Ok(false) };
    if !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    if !ability_window(g, p, Some(src), "create Kaldra", None, None)? {
        return Ok(true);
    }
    let mut data = PermData::default();
    data.set(DataKey::Legendary, Val::Bool(true));
    data.set(DataKey::Kaldra, Val::Bool(true));
    let spec = Tokens { tgh: Some(4), color: Some(Colors::NONE), types: vec!["avatar"], data, ..Tokens::new(1, 4) };
    let k = make_tokens(g, p, spec)?;
    if let Some(&k) = k.first() {
        for e in pieces {
            g.perm_mut(e).attached = Some(k);
        }
        g.perm_mut(k).data.set(DataKey::Indestr, Val::Bool(true));
        crate::glog!(g, "  {} creates Kaldra wearing Sword, Shield and Helm", pname(g, p));
    }
    Ok(true)
}

// ------------------------------------------------------------------ Blackblade Reforged
/// equipped creature +1/+1 per land you control (common.CREATURE_PT)
fn blackblade_pt(g: &Game, m: PermId) -> (i32, i32) {
    let k = attached_named(g, m, "Blackblade Reforged").len() as i32;
    let n = k * g.player(owner(g, m)).lands.len() as i32;
    (n, n)
}

/// equip a legendary creature {3}, any other {7}
fn blackblade_equip(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || post.is_none() || g.active != Some(p) {
        return Ok(vec![]);
    }
    if g.perm(src).attached.is_some_and(|a| on(g, p, a)) {
        return Ok(vec![]);
    }
    let mut t = best_legend(g, p);
    let mut n = 3u32;
    if t.is_none() {
        let cs: Vec<PermId> = g
            .player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| g.is_creature(m) && !g.perm(m).phased && !g.perm(m).noatk)
            .collect();
        t = max_by(&cs, |m| pval(g, m));
        n = 7;
    }
    let Some(t) = t else { return Ok(vec![]) };
    if !can_pay(g, p, n, "", false) {
        return Ok(vec![]);
    }
    let u = 1.5 + 0.25 * g.player(p).lands.len() as f64 - 0.2 * n as f64;
    let label = format!("equip Blackblade Reforged to {}", g.perm(t).name);
    Ok(vec![ability(u, label, src, blackblade_go, pack(t.0, n))])
}

fn blackblade_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (t, n) = unpack(arg);
    let t = PermId(t);
    if !can_pay(g, p, n, "", false) || !on(g, p, t) {
        return Ok(false);
    }
    equip_to(g, p, src, t, n)
}

// ------------------------------------------------------------------ planeswalkers
fn always_1_2(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(1.2)
}

fn dakkon_surveil(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    crate::cardcode::scry(g, p, 2, true)
}

fn dakkon_exile_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let t = best_opp_creature(g, p, |_| true)?;
    (pval(g, t) >= 3.0).then(|| pval(g, t) - 1.0)
}

fn dakkon_exile(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(t) = best_opp_creature(g, p, |_| true) {
        apply_removal(g, Some(p), t, "exile", None)?;
    }
    Ok(())
}

fn dakkon_ult_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let pl = g.player(p);
    pl.hand
        .iter()
        .chain(pl.gy.iter())
        .any(|&c| g.db.get(c).types.has(Types::ARTIFACT) && g.db.get(c).cmc >= 5)
        .then_some(6.0)
}

fn dakkon_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let pl = g.player(p);
    let arts: Vec<CardId> = pl
        .hand
        .iter()
        .chain(pl.gy.iter())
        .copied()
        .filter(|&c| g.db.get(c).types.has(Types::ARTIFACT) && !g.db.get(c).land)
        .collect();
    let Some(c) = first_max(&arts, |c| (g.db.get(c).cmc, card_worth(g, p, c, false))) else { return Ok(()) };
    let pl = g.player_mut(p);
    if pl.hand.contains(&c) {
        remove_card(&mut pl.hand, c);
    } else {
        remove_card(&mut pl.gy, c);
    }
    enter(g, p, c, Enter::default())?;
    Ok(())
}

static DAKKON: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "surveil 2", val: always_1_2, eff: dakkon_surveil },
    WalkerAb { delta: -3, label: "exile a creature", val: dakkon_exile_val, eff: dakkon_exile },
    WalkerAb { delta: -6, label: "an artifact onto the battlefield", val: dakkon_ult_val, eff: dakkon_ult },
];

/// Dakkon, Shadow Slayer enters with loyalty equal to your lands
fn dakkon_enters(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let n = g.player(p).lands.len() as i32;
    g.perm_mut(m).loyalty = Some(n);
    Ok(())
}

fn always_2_5(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(2.5)
}

fn mord_draw(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    draw(g, p, 2, false)?;
    let hand = g.player(p).hand.clone();
    if let Some(x) = min_by(&hand, |c| card_worth(g, p, c, false)) {
        let pl = g.player_mut(p);
        remove_card(&mut pl.hand, x);
        pl.library.insert(0, x);
    }
    Ok(())
}

fn mord_dog_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let n = g.player(p).hand.len();
    (n >= 3).then(|| 0.6 * n as f64)
}

fn mord_dog(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let mut data = PermData::default();
    data.set(DataKey::MordDog, Val::Bool(true));
    let spec = Tokens {
        tgh: Some(0),
        color: Some(Colors::from_letters("U")),
        types: vec!["dog", "illusion"],
        data,
        ..Tokens::new(1, 0)
    };
    make_tokens(g, p, spec)?;
    Ok(())
}

/// common.TOKEN_PT: Mordenkainen's Dog Illusion has power and toughness twice its controller's hand
fn mord_dog_pt(g: &Game, m: PermId) -> (i32, i32) {
    if g.perm(m).data.truthy(DataKey::MordDog) {
        let n = 2 * g.player(owner(g, m)).hand.len() as i32;
        return (n, n);
    }
    (0, 0)
}

fn always_9(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(9.0)
}

/// -10: exchange hand and library (Python also sets `p.no_max_hand`, which nothing reads)
fn mord_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let pl = g.player_mut(p);
    std::mem::swap(&mut pl.hand, &mut pl.library);
    shuffle_library(g, p);
    Ok(())
}

static MORDENKAINEN: [WalkerAb; 3] = [
    WalkerAb { delta: 2, label: "draw two, one to the bottom", val: always_2_5, eff: mord_draw },
    WalkerAb { delta: -2, label: "a Dog Illusion (twice your hand)", val: mord_dog_val, eff: mord_dog },
    WalkerAb { delta: -10, label: "swap hand and library", val: always_9, eff: mord_ult },
];

fn w6_land_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    Some(if g.player(p).gy.iter().any(|&c| g.db.get(c).land) { 1.5 } else { 0.8 })
}

fn w6_land(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(x) = g.player(p).gy.iter().copied().find(|&c| g.db.get(c).land) {
        let pl = g.player_mut(p);
        remove_card(&mut pl.gy, x);
        pl.hand.push(x);
    }
    Ok(())
}

fn w6_ping_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let t = best_opp_creature(g, p, |m| etgh(g, m) <= 1)?;
    (pval(g, t) >= 1.5).then(|| pval(g, t))
}

fn w6_ping(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(t) = best_opp_creature(g, p, |m| etgh(g, m) <= 1) {
        apply_removal(g, Some(p), t, "dmg1", None)?;
    } else {
        let opps: Vec<PlayerId> = g.opps(p).collect();
        if let Some(q) = first_min(&opps, |q| g.player(q).life) {
            lose_life(g, q, 1, Some(p), "burn", Some(true))?;
        }
    }
    Ok(())
}

fn always_6(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(6.0)
}

/// -7: emblem: retrace (Python records `p.w6_retrace`, which nothing reads)
fn w6_ult(_g: &mut Game, _p: PlayerId, _src: PermId) -> Res {
    Ok(())
}

static WRENN: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "a land back to hand", val: w6_land_val, eff: w6_land },
    WalkerAb { delta: -1, label: "1 damage", val: w6_ping_val, eff: w6_ping },
    WalkerAb { delta: -7, label: "emblem: retrace", val: always_6, eff: w6_ult },
];

fn spark_attach_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    Some(if g.player(p).perms.iter().any(|&m| g.is_creature(m)) { 2.0 } else { 0.5 })
}

fn spark_attach(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    let t = best_legend(g, p).or_else(|| {
        let cs: Vec<PermId> =
            g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
        max_by(&cs, |m| epow(g, m) as f64)
    });
    if let Some(t) = t {
        g.perm_mut(src).attached = Some(t);
        crate::cardcode::put_plus(g, t, 1);
    }
    Ok(())
}

fn always_3_5(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(3.5)
}

fn spark_draw(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    draw(g, p, 2, false)
}

fn spark_mana_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    g.player(p).hand.iter().any(|&c| g.db.get(c).cmc >= 8).then_some(5.0)
}

fn spark_mana(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    g.player_mut(p).floating.any += 10;
    Ok(())
}

static AETHERSPARK: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "attach, +1/+1 counter", val: spark_attach_val, eff: spark_attach },
    WalkerAb { delta: -5, label: "draw two", val: always_3_5, eff: spark_draw },
    WalkerAb { delta: -10, label: "add ten mana", val: spark_mana_val, eff: spark_mana },
];

/// combat damage by the equipped creature on your turn adds that much loyalty
fn spark_loyalty(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, dmg: i32) -> Res {
    let x = g.perm(src);
    if p == x.owner && x.attached == Some(a) && x.loyalty.is_some() {
        let l = x.loyalty.unwrap();
        g.perm_mut(src).loyalty = Some(l + dmg);
    }
    Ok(())
}

/// Carth the Lion: your loyalty abilities cost an extra [+1]
fn carth_extra(g: &Game, src: Src, p: PlayerId) -> i32 {
    (p == owner(g, src)) as i32
}

/// jodah._carth_look: a planeswalker from the top seven to hand, the rest to the bottom in a random order
fn carth_look(g: &mut Game, o: PlayerId) -> Res {
    let top = take_top(g, o, 7);
    let pw: Vec<CardId> = top.iter().copied().filter(|&c| g.db.get(c).types.has(Types::PLANESWALKER)).collect();
    let mut rest = top;
    if let Some(c) = max_by(&pw, |c| card_worth(g, o, c, false)) {
        remove_card(&mut rest, c);
        g.player_mut(o).hand.push(c);
        crate::glog!(g, "    Carth finds {}", cname(g, c));
    }
    g.rng.shuffle(&mut rest);
    to_bottom(g, o, rest);
    Ok(())
}

fn carth_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, owner(g, src), Some(src), "look at seven for a planeswalker", None)? {
        carth_look(g, owner(g, src))?;
    }
    Ok(())
}

fn carth_pw_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, src);
    if owner(g, m) == o
        && g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER))
        && trigger_window(g, o, Some(src), "look at seven", None)?
    {
        carth_look(g, o)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ creatures
/// jodah.discover: exile from the top until a nonland card with mana value n or less: cast it free (or to hand); the
/// rest to the bottom in a random order
pub fn discover(g: &mut Game, o: PlayerId, n: u32, src_name: &str) -> Res {
    let mut seen = vec![];
    let mut hit = None;
    while let Some(x) = g.player_mut(o).library.pop() {
        let d = g.db.get(x);
        if !d.land && d.cmc <= n {
            hit = Some(x);
            break;
        }
        seen.push(x);
    }
    if let Some(h) = hit {
        if castable(g, o, h, "lib") && !(g.db.get(h).tag(Tag::Ctr) && g.stack.is_empty()) {
            g.last_x = 0;
            crate::glog!(g, "    {src_name}: discover {n} casts {}", cname(g, h));
            cast_card(g, o, h, "lib", Ctx::default())?;
        } else {
            g.player_mut(o).hand.push(h);
        }
    }
    g.rng.shuffle(&mut seen);
    to_bottom(g, o, seen);
    Ok(())
}

/// attacking: tap your two least valuable untapped artifacts or creatures to discover 3
fn caparocti(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p != owner(g, src) || !atk.contains(&src) {
        return Ok(vec![]);
    }
    let mut cand: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            !x.tapped
                && !x.phased
                && !atk.contains(&m)
                && m != src
                && (g.is_creature(m) || x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)))
        })
        .collect();
    cand.sort_by(|&a, &b| pval(g, a).partial_cmp(&pval(g, b)).unwrap_or(std::cmp::Ordering::Equal));
    if cand.len() < 2 || !trigger_window(g, p, Some(src), "tap two: discover 3", None)? {
        return Ok(vec![]);
    }
    for &m in &cand[..2] {
        g.perm_mut(m).tapped = true;
    }
    discover(g, p, 3, "Caparocti Sunborn")?;
    Ok(vec![])
}

/// jodah._regen: pay the regeneration cost (the AI always does when it can)
fn regen(g: &mut Game, m: PermId, gn: u32, pips: &str) -> Res<bool> {
    let p = owner(g, m);
    if !can_pay(g, p, gn, pips, false) {
        return Ok(false);
    }
    pay(g, p, gn, pips, false)?;
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "    {} regenerates", g.perm(m).name);
    Ok(true)
}

/// Cromat: {B}{G} regenerate
fn cromat_regen(g: &mut Game, m: PermId) -> Res<bool> {
    regen(g, m, 0, "BG")
}

/// Korlash, Heir to Blackblade: {1}{B} regenerate
fn korlash_regen(g: &mut Game, m: PermId) -> Res<bool> {
    regen(g, m, 1, "B")
}

/// Cromat, before attacking: {U}{R} for flying when the defenders have no fliers to block, {R}{W} pumps with spare
/// mana (the blocker-destroy and top-of-library abilities are not used)
fn cromat_combat(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || g.perm(src).tapped || g.perm(src).sick {
        return Ok(());
    }
    let opp_fly = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .any(|m| g.perm(m).fly && g.is_creature(m) && !g.perm(m).tapped);
    if !g.perm(src).fly && !opp_fly && can_pay(g, p, 0, "UR", false) && total_mana(g, p, false) >= 4 {
        pay(g, p, 0, "UR", false)?;
        eot_kw(g, src, "flying");
        g.dsl_on = true;
        crate::glog!(g, "  Cromat gains flying");
    }
    while total_mana(g, p, false) >= 4 && can_pay(g, p, 0, "RW", false) {
        pay(g, p, 0, "RW", false)?;
        let x = g.perm_mut(src);
        x.eot_pt = (x.eot_pt.0 + 1, x.eot_pt.1 + 1);
        g.dsl_on = true;
    }
    Ok(())
}

/// Korlash: power and toughness equal to your Swamps (grandeur needs a second copy: never in a singleton deck)
fn korlash_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    let n = g.player(p).lands.iter().filter(|&&l| g.db.get(g.land(l).cd).land_types.has('B')).count() as i32;
    (n, n)
}

/// Dragonlord Dromoka: opponents can't cast spells during your turn
fn dromoka(g: &Game, src: Src, caster: PlayerId, _c: CardId, _zone: Sym) -> bool {
    let o = owner(g, src);
    !(caster != o && g.active == Some(o))
}

/// Genesis Hydra: by the spare mana it would be cast with
fn hydra_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let x = total_mana(g, p, false) as i32 - 2;
    if x >= 3 { 40 + (5 * x).min(30) } else { 0 }
}

/// X (the spare mana it was cast with; 0 when cast free): the top X cards, a nonland permanent with mana value X or
/// less onto the battlefield, the rest shuffled in; the Hydra gets X +1/+1 counters. The reveal happens as it resolves
/// rather than as it is cast (only a counterspell tells the two apart)
fn hydra_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let x = std::mem::take(&mut g.last_x);
    if x > 0 {
        let mut top = take_top(g, o, x as usize);
        let ok: Vec<CardId> = top
            .iter()
            .copied()
            .filter(|&y| {
                let d = g.db.get(y);
                !d.land && d.cmc as i32 <= x && (d.perm || d.creature)
            })
            .collect();
        if let Some(y) = first_max(&ok, |y| (g.db.get(y).cmc, card_worth(g, o, y, false))) {
            remove_card(&mut top, y);
            crate::glog!(g, "    Genesis Hydra puts {} onto the battlefield", cname(g, y));
            enter(g, o, y, Enter::default())?;
        }
        g.player_mut(o).library.extend(top);
        shuffle_library(g, o);
    }
    if on(g, o, src) {
        crate::cardcode::add_counters(g, src, x); // Hardened Scales applies
        if etgh(g, src) <= 0 {
            die(g, src, "sba")?;
        }
    }
    Ok(())
}

/// Lagrella, the Magpie: exiles the best creature of each opponent until it leaves
fn lagrella_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if m != src || !on(g, o, src) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "exile a creature of each opponent", Some(5.0))? || !on(g, o, src) {
        return Ok(());
    }
    oring_exile(g, src, o, |g, x| g.is_creature(x), true, true)
}

fn lagrella_leaves(g: &mut Game, _src: Src, m: PermId) -> Res {
    oring_return(g, m)
}

/// Mirri: while tapped, no more than one creature can attack you
fn mirri_cap(g: &Game, src: Src, _attacker: PlayerId, d: PlayerId) -> Option<i32> {
    (d == owner(g, src) && g.perm(src).tapped).then_some(1)
}

/// Mirri attacks: each opponent blocks with at most one creature
fn mirri_blocks(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId, assign: &mut Assign) -> Res {
    if p != owner(g, src) || !atk.contains(&src) {
        return Ok(());
    }
    let mut attackers: Vec<PermId> = vec![];
    for &(a, _) in assign.iter() {
        if !attackers.contains(&a) {
            attackers.push(a);
        }
    }
    attackers.sort_by_key(|&a| -epow(g, a)); // stable: Python's sorted(list(assign), key=-power)
    let mut used: Option<PermId> = None;
    for a in attackers {
        let Some(b) = assign.iter().find(|x| x.0 == a).map(|x| x.1) else { continue };
        if owner(g, b) != d {
            continue;
        }
        match used {
            Some(u) if u != b => assign.retain(|x| x.0 != a),
            _ => used = Some(b),
        }
    }
    Ok(())
}

fn no_options(_g: &mut Game, _src: Src, _p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    Ok(vec![])
}

/// Sol'kanar the Swamp King: you gain 1 life whenever any player casts a black spell
fn solkanar(g: &mut Game, src: Src, _caster: PlayerId, c: CardId) -> Res {
    if g.db.get(c).pips.contains('B') {
        let o = owner(g, src);
        gain(g, o, 1)?;
    }
    Ok(())
}

/// Tolsimir Wolfblood: {T}: Voja, a legendary 2/2 Wolf (one at a time)
fn tolsimir(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if p != x.owner || x.tapped || x.sick || post.is_none() {
        return Ok(vec![]);
    }
    if g.player(p).perms.iter().any(|&m| g.perm(m).cd.is_none() && g.perm(m).data.truthy(DataKey::Voja)) {
        return Ok(vec![]);
    }
    Ok(vec![ability(2.5, "Tolsimir: create Voja".into(), src, tolsimir_go, 0)])
}

fn tolsimir_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.perm(src).tapped {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if !ability_window(g, p, Some(src), "create Voja", None, None)? {
        return Ok(true);
    }
    let mut data = PermData::default();
    data.set(DataKey::Legendary, Val::Bool(true));
    data.set(DataKey::Voja, Val::Bool(true));
    let spec = Tokens {
        tgh: Some(2),
        color: Some(Colors::from_letters("GW")),
        types: vec!["wolf"],
        data,
        ..Tokens::new(1, 2)
    };
    make_tokens(g, p, spec)?;
    Ok(true)
}

/// Urza, Powerstone Prodigy: {1}, {T}: draw then discard; discarding an artifact makes a Powerstone (once a turn)
fn urza_loot(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if p != x.owner
        || x.tapped
        || x.sick
        || post.is_none()
        || !can_pay(g, p, 1, "", false)
        || g.player(p).hand.is_empty()
    {
        return Ok(vec![]);
    }
    Ok(vec![ability(if post == Some(true) { 1.0 } else { 0.4 }, "Urza: loot".into(), src, urza_go, 0)])
}

fn urza_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.perm(src).tapped || !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    g.perm_mut(src).tapped = true;
    if !ability_window(g, p, Some(src), "draw, then discard", None, None)? {
        return Ok(true);
    }
    draw(g, p, 1, false)?;
    let hand = g.player(p).hand.clone();
    if !hand.is_empty() {
        let arts: Vec<CardId> = hand
            .iter()
            .copied()
            .filter(|&c| g.db.get(c).types.has(Types::ARTIFACT) && card_worth(g, p, c, false) < 30.0)
            .collect();
        let pool = if arts.is_empty() { &hand } else { &arts };
        let x = min_by(pool, |c| card_worth(g, p, c, false)).unwrap();
        discard_cards(g, p, &[x])?;
        if g.db.get(x).types.has(Types::ARTIFACT) && once_per_turn(g, p, "urza_stone") {
            make_artifact_tokens(g, p, "Powerstone", 1)?; // (common.make_artifact_tokens makes a Clue of it)
        }
    }
    Ok(true)
}

/// King Darien XLVIII: {3}{G}{W}: a +1/+1 counter and a 1/1 Soldier with spare mana (the sacrifice-for-protection
/// ability is not used)
fn darien(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || post.is_none() || g.active != Some(p) || !can_pay(g, p, 3, "GW", false) {
        return Ok(vec![]);
    }
    let u = if post == Some(true) { 1.2 } else { 0.3 };
    Ok(vec![ability(u, "King Darien: counter and a Soldier".into(), src, darien_go, 0)])
}

fn darien_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 3, "GW", false) {
        return Ok(false);
    }
    pay(g, p, 3, "GW", false)?;
    if !ability_window(g, p, Some(src), "a counter and a Soldier", None, None)? {
        return Ok(true);
    }
    crate::cardcode::put_plus(g, src, 1);
    let spec =
        Tokens { tgh: Some(1), color: Some(Colors::from_letters("W")), types: vec!["soldier"], ..Tokens::new(1, 1) };
    make_tokens(g, p, spec)?;
    Ok(true)
}

/// Sisters of Stone Death: {B}{G}: exile a creature blocking the Sisters (the lure and the {2}{B} reanimation of
/// exiled creatures are not used; Python also notes the exiled card in `data['sisters']`, which nothing reads)
fn sisters(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId, assign: &mut Assign) -> Res {
    if p != owner(g, src) || !atk.contains(&src) {
        return Ok(());
    }
    let Some(b) = assign.iter().find(|x| x.0 == src).map(|x| x.1) else { return Ok(()) };
    if on(g, d, b) && can_pay(g, p, 0, "BG", false) && !untargetable(g, b) {
        pay(g, p, 0, "BG", false)?;
        crate::glog!(g, "  Sisters of Stone Death exile {}", g.perm(b).name);
        let name = g.perm(b).name;
        *g.player_mut(d).lost_names.entry(name).or_insert(0) += 1;
        exile_perm(g, b)?;
        assign.retain(|x| x.0 != src);
    }
    Ok(())
}

// ------------------------------------------------------------------ spells
fn survey_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.player(p).lands.len() <= 6 { 46 } else { 20 }
}

/// Cartographer's Survey: up to two lands from the top seven onto the battlefield tapped; the rest to the bottom
fn survey(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut top = take_top(g, p, 7);
    let mut lands: Vec<CardId> = top.iter().copied().filter(|&x| g.db.get(x).land).collect();
    let ncol = |x: CardId| g.db.get(x).tags.str(Tag::C).map_or(0, |s| s.chars().count()) as i64;
    lands.sort_by_key(|&x| -ncol(x));
    lands.truncate(2);
    for x in lands {
        remove_card(&mut top, x);
        g.add_land_entering(p, x, true);
        landfall(g, p)?;
    }
    g.rng.shuffle(&mut top);
    to_bottom(g, p, top);
    Ok("gy")
}

fn prio_36(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    36
}

/// Experimental Augury: the best of the top three to hand, the rest to the bottom; proliferate
fn augury(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut top = take_top(g, p, 3);
    if let Some(x) = max_by(&top, |y| card_worth(g, p, y, false)) {
        remove_card(&mut top, x);
        g.player_mut(p).hand.push(x);
    }
    to_bottom(g, p, top);
    crate::cardcode::proliferate_all(g, p)?;
    Ok("gy")
}

fn despair_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.opps(p).next().is_some() { 50 } else { 0 }
}

fn despair_one(g: &mut Game, p: PlayerId, q: PlayerId, kind: Types) -> Res {
    let pool: Vec<PermId> = g
        .player(q)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            !g.perm(m).phased
                && if kind == Types::CREATURE {
                    g.is_creature(m)
                } else {
                    g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(kind))
                }
        })
        .collect();
    if let Some(m) = min_by(&pool, |m| pval(g, m)) {
        die(g, m, "sac")?;
    } else {
        lose_life(g, q, 2, Some(p), "other", None)?;
        draw(g, p, 1, false)?;
    }
    Ok(())
}

/// Invoke Despair: the most threatening opponent sacrifices a creature, an enchantment and a planeswalker (their
/// least valuable); for each they can't, they lose 2 and you draw
fn invoke(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let Some(q) = max_by(&opps, |o| threat(g, p, o)) else { return Ok("gy") };
    if !g.player(q).alive {
        return Ok("gy");
    }
    for k in [Types::CREATURE, Types::ENCHANTMENT, Types::PLANESWALKER] {
        despair_one(g, p, q, k)?;
    }
    check_state(g)?;
    Ok("gy")
}

/// Kaervek's Purge: {X}{B}{R}: destroy target creature with mana value X; it deals its power to its controller
fn purge(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !g.player(p).hand.contains(&c) {
        return Ok(vec![]);
    }
    let tg: Vec<PermId> = legal_targets(g, p, "destroy", "c", false, Some(c))
        .into_iter()
        .filter(|&m| g.perm(m).cd.is_some_and(|cd| can_pay(g, p, g.db.get(cd).cmc, "BR", false)))
        .collect();
    let Some(t) = max_by(&tg, |m| pval(g, m) + 0.2 * epow(g, m) as f64) else { return Ok(vec![]) };
    // Kefka, Ruler of Ruin: the life lost on your turn draws that many cards, so the power it deals is card draw
    let draws =
        if g.player(p).key == "kefka" && crate::impls::kefka::flipped(g, p) { epow(g, t).clamp(0, 6) } else { 0 };
    if pval(g, t) < 3.0 && draws < 2 {
        return Ok(vec![]);
    }
    let label = format!("Kaervek's Purge on {}", g.perm(t).name);
    let u = pval(g, t) * 0.5 - 0.8 + 0.7 * draws as f64;
    Ok(vec![plan(u, label, purge_go, pack(c.0 as u32, t.0))])
}

fn purge_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, t) = unpack(arg);
    let (c, t) = (CardId(c as u16), PermId(t));
    let x = g.perm(t).cd.map_or(0, |cd| g.db.get(cd).cmc);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, x, "BR", false) || !g.perm(t).on_bf {
        return Ok(false);
    }
    pay(g, p, x, "BR", false)?;
    remove_card(&mut g.player_mut(p).hand, c);
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c); // the card report counts a card as cast from this set
    crate::glog!(g, "  {} casts Kaervek's Purge -> {} (X={x})", pname(g, p), g.perm(t).name);
    on_cast(g, p, c)?;
    let ok = counter_window(g, p, c, 4.0, vec![])?;
    g.player_mut(p).gy.push(c);
    if ok && g.perm(t).on_bf && !untargetable(g, t) {
        let (q, power) = (owner(g, t), epow(g, t));
        apply_removal(g, Some(p), t, "destroy", Some(c))?;
        if !on(g, q, t) && power != 0 {
            lose_life(g, q, power, Some(p), "burn", Some(true))?;
        }
    }
    Ok(true)
}

/// Profane Tutor: suspend 2 for {1}{B}: two upkeeps later, a free Demonic Tutor
fn profane(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "B", false) {
        return Ok(vec![]);
    }
    Ok(vec![plan(2.2, "suspend Profane Tutor".into(), profane_go, c.0 as i64)])
}

fn profane_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "B", false) {
        return Ok(false);
    }
    pay(g, p, 1, "B", false)?;
    let pl = g.player_mut(p);
    remove_card(&mut pl.hand, c);
    pl.exile.push(c);
    pl.suspended.push((c, 2));
    crate::glog!(g, "  {} suspends Profane Tutor", pname(g, p));
    Ok(true)
}

/// jodah.suspend_upkeep: the beginning of p's upkeep: a time counter off each suspended card; at zero it's cast free
pub fn suspend_upkeep(g: &mut Game, p: PlayerId) -> Res {
    for e in g.player(p).suspended.clone() {
        let c = e.0;
        let Some(i) = g.player(p).suspended.iter().position(|x| *x == e) else { continue };
        if !g.player(p).exile.contains(&c) {
            g.player_mut(p).suspended.remove(i);
            continue;
        }
        let left = e.1.saturating_sub(1);
        g.player_mut(p).suspended[i].1 = left;
        if left > 0 {
            continue;
        }
        let pl = g.player_mut(p);
        pl.suspended.remove(i);
        remove_card(&mut pl.exile, c);
        crate::glog!(g, "  {} casts {} from suspend", pname(g, p), cname(g, c));
        cast_card(g, p, c, "lib", Ctx::default())?;
    }
    Ok(())
}

/// Profane Tutor: search for any card
fn profane_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    tutor(g, p, "any")?;
    Ok("gy")
}

fn prio_0(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

// ------------------------------------------------------------------ Mirari, Memory Jar, Court of Ardenvale
const MIRARI_WORTH: [&str; 5] =
    ["Fact or Fiction", "Invoke Despair", "Demonic Tutor", "Cartographer's Survey", "Experimental Augury"];

/// Mirari: whenever you cast an instant or sorcery, pay {3} to copy it (the AI copies spells worth a second
/// resolution: draw, removal and tutors). A person decides.
fn mirari(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    let d = g.db.get(c);
    if caster != o || !(d.instant || d.sorcery) || !can_pay(g, o, 3, "", false) {
        return Ok(());
    }
    let worth =
        [Tag::Draw, Tag::Rem, Tag::Tut, Tag::Drawcre].iter().any(|&k| d.tag(k)) || MIRARI_WORTH.contains(&&*d.name);
    if !worth || d.tag(Tag::Ctr) {
        return Ok(());
    }
    let label = format!("pay {{3}}: copy {}", d.name);
    if human_of(g, o).is_some() && !choices::yes_no(g, o, &format!("Mirari: pay {{3}} to copy {}?", d.name))? {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), &label, None)? {
        return Ok(());
    }
    pay(g, o, 3, "", false)?;
    copy_spell(g, o, c, None, false)
}

fn mirari_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let n = g.player(p).hand.iter().filter(|&&x| g.db.get(x).instant || g.db.get(x).sorcery).count();
    if n >= 2 { 44 } else { 26 }
}

/// Memory Jar: {T}, sacrifice: everyone sets their hand aside and draws seven; at the end step they discard and get
/// the old hands back. Used with a small hand and mana left to spend the seven
fn jar(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || post != Some(false) || g.active != Some(p) {
        return Ok(vec![]);
    }
    // the sheoldred deck with Sheoldred out: seven cards for everyone is 14 life gained and 14 lost by each opponent
    if g.player(p).key == "sheoldred" && super::sheoldred::sheoldred_out(g, p) {
        return Ok(vec![ability(8.0, "Memory Jar (Sheoldred)".into(), src, jar_go, 0)]);
    }
    if g.player(p).hand.len() > 2 || total_mana(g, p, false) < 3 {
        return Ok(vec![]);
    }
    let u = 3.0 + 0.4 * (7.0 - g.player(p).hand.len() as f64);
    Ok(vec![ability(u, "Memory Jar".into(), src, jar_go, 0)])
}

fn jar_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !on(g, p, src) {
        return Ok(false);
    }
    leave(g, src)?;
    let cd = g.perm(src).cd.unwrap();
    g.player_mut(p).gy.push(cd);
    if !ability_window_card(g, p, cd, "wheel for seven", None, None)? {
        return Ok(true);
    }
    let mut due = vec![];
    for i in 0..g.players.len() {
        let q = PlayerId(i as u8);
        if !g.player(q).alive {
            continue;
        }
        let pl = g.player_mut(q);
        let held = std::mem::take(&mut pl.hand);
        pl.exile.extend(held.iter().copied());
        due.push((q, held));
        draw(g, q, 7, false)?;
    }
    g.jar_due.extend(due);
    crate::glog!(g, "  {} cracks Memory Jar", pname(g, p));
    Ok(true)
}

/// jodah.jar_end: Memory Jar's end step: each player discards the seven and takes the old hand back
pub fn jar_end(g: &mut Game) -> Res {
    let due = std::mem::take(&mut g.jar_due);
    for (q, held) in due {
        if !g.player(q).alive {
            continue;
        }
        let hand = g.player(q).hand.clone();
        discard_cards(g, q, &hand)?;
        for c in held {
            if remove_card(&mut g.player_mut(q).exile, c) {
                g.player_mut(q).hand.push(c);
            }
        }
    }
    Ok(())
}

fn prio_38(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    38
}

/// Court of Ardenvale: you become the monarch
fn court_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if m == src && trigger_window(g, o, Some(src), "become the monarch", None)? {
        crate::cardcode::become_monarch(g, o)?;
    }
    Ok(())
}

/// Court of Ardenvale: each upkeep, a permanent card with mana value 3 or less from your graveyard to hand, or onto
/// the battlefield while you are the monarch
fn court(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = owner(g, src);
    if p != o {
        return Ok(());
    }
    let cs: Vec<CardId> = g
        .player(o)
        .gy
        .iter()
        .copied()
        .filter(|&c| {
            let d = g.db.get(c);
            (d.perm || d.creature) && !d.land && d.cmc <= 3
        })
        .collect();
    if cs.is_empty() || !trigger_window(g, o, Some(src), "return a permanent card", None)? {
        return Ok(());
    }
    let c = max_by(&cs, |c| card_worth(g, o, c, false)).unwrap();
    if !remove_card(&mut g.player_mut(o).gy, c) {
        return Ok(());
    }
    if g.monarch == Some(o) {
        enter(g, o, c, Enter::default())?;
        crate::glog!(g, "    Court of Ardenvale returns {} to the battlefield", cname(g, c));
    } else {
        g.player_mut(o).hand.push(c);
        crate::glog!(g, "    Court of Ardenvale returns {} to hand", cname(g, c));
    }
    Ok(())
}

// ------------------------------------------------------------------ mana
/// Fyndhorn Elder: {T}: {G}{G}
fn elder_mana(_g: &Game, _p: PlayerId, _m: PermId) -> u32 {
    2
}

// ------------------------------------------------------------------ lands (Python's mine.py)
/// mine.plaza_colors: Plaza of Heroes: any colour for a legendary spell; otherwise the colours among your legendary
/// permanents (its {C} is the empty set)
fn plaza_colors(g: &Game, p: PlayerId, _l: LandId) -> Colors {
    if g.pay_for.is_some_and(|c| g.db.get(c).tag(Tag::Leg)) {
        return g.player(p).ident;
    }
    let mut cols = Colors::NONE;
    for &m in &g.player(p).perms {
        if let Some(c) = g.perm(m).cd
            && super::mine::is_legendary(g, m)
        {
            cols = cols.union(Colors::from_letters(&g.db.get(c).pips));
        }
    }
    cols
}

/// mine.territory_type: the creature type Unclaimed Territory names as it enters: the one most of p's creature cards
/// share (Sauron's deck names Orc). Python caches it on the player and the land; it is worked out each time here (the
/// same answer unless p's cards change hands)
pub fn territory_type(g: &Game, p: PlayerId) -> Sym {
    if g.player(p).key == "sauron" {
        return "orc"; // mine.TERRITORY_TYPE
    }
    let pl = g.player(p);
    let perms = pl.perms.iter().filter_map(|&m| g.perm(m).cd);
    let cards = pl.library.iter().chain(&pl.hand).chain(&pl.gy).chain(&pl.exile).copied().chain(perms).chain(pl.commanders());
    let mut n: Vec<(&str, i32)> = vec![];
    for c in cards {
        let d = g.db.get(c);
        if !d.creature {
            continue;
        }
        for s in d.subtypes.iter() {
            match n.iter_mut().find(|x| x.0 == &**s) {
                Some(x) => x.1 += 1,
                None => n.push((s, 1)),
            }
        }
    }
    match first_min(&n, |x| (-x.1, x.0)) {
        Some((t, _)) => intern(t),
        None => "human",
    }
}

/// mine.territory_colors: Unclaimed Territory: {C}, or any colour for a creature spell of the named type (a
/// changeling is every type)
fn territory_colors(g: &Game, p: PlayerId, l: LandId) -> Colors {
    let Some(pf) = g.pay_for.map(|c| g.db.get(c)).filter(|d| d.creature) else { return Colors::NONE };
    let t = match g.land(l).data.get(DataKey::Ctype) {
        Some(Val::Str(s)) => *s,
        _ => territory_type(g, p),
    };
    if pf.has_subtype(t) || pf.has_kw("changeling") { g.player(p).ident } else { Colors::NONE }
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.creature_pt.push(jodah_pt);
    r.creature_pt.push(blackblade_pt);
    r.token_pt.push(mord_dog_pt);
    let c = r.card(db, JODAH)?;
    c.etb = Some(selfpt_etb);
    at_once(c, Event::Etb, true);
    c.cast = Some(jodah_cast);
    c.self_cast = Some(jodah_from_hand);
    let c = r.card(db, "Mithril Coat")?;
    c.etb = Some(coat_etb);
    at_once(c, Event::Etb, true);
    r.card(db, "Command Beacon")?.land_options = Some(beacon);
    let c = r.card(db, "Maelstrom Nexus")?;
    c.cast = Some(nexus);
    c.prio = Some(prio_50);
    r.card(db, "Maelstrom Wanderer")?.self_cast = Some(wanderer_cast);
    let c = r.card(db, "Sisay, Weatherlight Captain")?;
    c.self_pt = Some(sisay_pt);
    c.as_enters = Some(selfpt_as);
    c.options = Some(sisay);
    let c = r.card(db, SHALAI)?;
    c.grant_kw = Some(shalai_hexproof);
    c.player_hexproof = Some(shalai_you);
    c.options = Some(shalai_counters);
    c.pval = Some(6.0); // opponents' removal goes to her once she shields the rest
    if db.id(VENAT).is_some() {
        // Venat is not in data/cards.json (no list runs it; Python only knows it once its card data is fetched)
        let c = r.card(db, VENAT)?;
        c.cast = Some(venat_draw);
        c.options = Some(sundering);
        c.grant_kw = Some(hydaelyn_indestructible);
        c.crew = Some(blessing);
    }
    let c = r.card(db, "Tymna the Weaver")?;
    c.combat_damage = Some(tymna_hit);
    at_once(c, Event::CombatDamage, true);
    c.main2 = Some(tymna);
    let c = r.card(db, GARLAND)?;
    c.etb = Some(garland_etb);
    c.monarch = Some(garland_steal);
    c.combat_damage = Some(garland_hit);
    at_once(c, Event::CombatDamage, true);
    // the 99
    for name in KALDRA {
        r.card(db, name)?.self_regen = Some(kaldra_indestructible);
    }
    r.card(db, "Helm of Kaldra")?.options = Some(kaldra_assemble);
    let c = r.card(db, "Blackblade Reforged")?;
    c.etb = Some(selfpt_etb);
    at_once(c, Event::Etb, true);
    c.options = Some(blackblade_equip);
    walker(r, db, "Dakkon, Shadow Slayer", &DAKKON)?;
    r.card(db, "Dakkon, Shadow Slayer")?.as_enters = Some(dakkon_enters);
    walker(r, db, "Mordenkainen", &MORDENKAINEN)?;
    walker(r, db, "Wrenn and Six", &WRENN)?;
    walker(r, db, "The Aetherspark", &AETHERSPARK)?;
    let c = r.card(db, "The Aetherspark")?;
    c.combat_damage = Some(spark_loyalty);
    at_once(c, Event::CombatDamage, true);
    let c = r.card(db, "Carth the Lion")?;
    c.loyalty_extra = Some(carth_extra);
    c.etb = Some(carth_etb);
    c.dies = Some(carth_pw_dies);
    r.card(db, "Caparocti Sunborn")?.attack = Some(caparocti);
    let c = r.card(db, "Cromat")?;
    c.self_regen = Some(cromat_regen);
    c.crew = Some(cromat_combat);
    let c = r.card(db, "Korlash, Heir to Blackblade")?;
    c.self_regen = Some(korlash_regen);
    c.self_pt = Some(korlash_pt);
    c.as_enters = Some(selfpt_as); // before its 0/0 is checked
    r.card(db, "Dragonlord Dromoka")?.can_cast = Some(dromoka);
    let c = r.card(db, "Genesis Hydra")?;
    c.prio = Some(hydra_prio);
    c.etb = Some(hydra_etb);
    at_once(c, Event::Etb, true);
    let c = r.card(db, "Lagrella, the Magpie")?;
    c.etb = Some(lagrella_etb);
    c.leaves = Some(lagrella_leaves);
    at_once(c, Event::Leaves, true);
    let c = r.card(db, "Mirri, Weatherlight Duelist")?;
    c.attack_cap = Some(mirri_cap);
    c.blocks = Some(mirri_blocks);
    at_once(c, Event::Blocks, true);
    r.card(db, "Razia, Boros Archangel")?.options = Some(no_options);
    let c = r.card(db, "Sol'kanar the Swamp King")?;
    c.cast = Some(solkanar);
    at_once(c, Event::Cast, true);
    r.card(db, "Tolsimir Wolfblood")?.options = Some(tolsimir);
    r.card(db, "Urza, Powerstone Prodigy")?.options = Some(urza_loot);
    r.card(db, "King Darien XLVIII")?.options = Some(darien);
    let c = r.card(db, "Sisters of Stone Death")?;
    c.blocks = Some(sisters);
    at_once(c, Event::Blocks, true);
    // spells
    let c = r.card(db, "Cartographer's Survey")?;
    c.resolve = Some(survey);
    c.prio = Some(survey_prio);
    let c = r.card(db, "Experimental Augury")?;
    c.resolve = Some(augury);
    c.prio = Some(prio_36);
    let c = r.card(db, "Invoke Despair")?;
    c.resolve = Some(invoke);
    c.prio = Some(despair_prio);
    r.card(db, "Kaervek's Purge")?.hand_options = Some(purge);
    let c = r.card(db, "Profane Tutor")?;
    c.hand_options = Some(profane);
    c.resolve = Some(profane_resolve);
    c.prio = Some(prio_0); // never hard-cast (no mana cost): suspended from hand
    let c = r.card(db, "Mirari")?;
    c.cast = Some(mirari);
    c.prio = Some(mirari_prio);
    let c = r.card(db, "Memory Jar")?;
    c.options = Some(jar);
    c.prio = Some(prio_38);
    let c = r.card(db, "Court of Ardenvale")?;
    c.etb = Some(court_etb);
    c.upkeep = Some(court);
    c.prio = Some(prio_50);
    r.card(db, "Fyndhorn Elder")?.dyn_mana_perm = Some(elder_mana);
    // lands (mine.py)
    r.card(db, "Plaza of Heroes")?.land_cols = Some(plaza_colors);
    r.card(db, "Unclaimed Territory")?.land_cols = Some(territory_colors);
    Ok(())
}
