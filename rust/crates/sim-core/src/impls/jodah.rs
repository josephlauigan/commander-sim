//! Python's `cards/impl/jodah.py`: Jodah, the Unifier (JD's WUBRG Legends deck, key `jodah`) and its 99.
//!
//! - Jodah: legendary creatures you control get +X/+X (X = the legendary creatures you control); whenever you cast a
//!   legendary spell from your hand, exile from the top until a legendary nonland card with lesser mana value, cast it
//!   free, the rest to the bottom in a random order.
//! - The AI: the outside decks' generic priorities, with legendary spells first while Jodah is out (the more
//!   expensive, the more the cascade can find); Jodah's protection (audit/jodah, 10-09); the tutor wish list.
//! - The rework candidates and swap candidates (Command Beacon, Maelstrom Nexus and Wanderer, Sisay, Shalai, Venat,
//!   Tymna, Garland) and the 99's cards that need code.
//!
//! Plaza of Heroes' and Unclaimed Territory's colours (Python's mine.plaza_colors and territory_colors, which
//! engine._land_cols calls by name) are here too: only Jodah's list runs them.

use std::sync::OnceLock;

use super::common::{WalkerAb, best_opp_creature, best_opp_nonland, oring_exile, oring_return, walker};
use super::lands::{can_pay_without, pay_without, sac_land};
use super::partials::{at_once, eot_kw, first_max, first_min, on, pack, remove_card, unpack};
use crate::ai::brain::removal_risk;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, castable, copy_spell, on_cast};
use crate::engine::life::{check_state, gain, lose_life};
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{ability_window, ability_window_card, counter_window, equip_to, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library, tutor};
use crate::engine::values::{
    board_power, colors_of, commander_out, epow, etgh, indestructible, once_per_turn, player_hexproof, protected_from,
    pval, threat, untargetable,
};
use crate::engine::zones::{
    Enter, Tokens, agent_for, agent_take, amass, die, discard_cards, draw, enter, exile_perm, landfall, leave,
    make_artifact_tokens, make_tokens, max_by, min_by, searchable, teferis_protection,
};
use crate::flow::Res;
use crate::hooks::{Action, Assign, Event, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, PermData, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

pub const JODAH: &str = "Jodah, the Unifier";
const TEFERIS_PROTECTION: &str = "Teferi's Protection";

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

/// the cards of a list (names not in the card database are left out)
fn ids(g: &Game, names: &[&str]) -> Vec<CardId> {
    names.iter().filter_map(|n| g.db.id(n)).collect()
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

// ------------------------------------------------------------------ the A/B switches (audit/jodah)
/// jodah._AI: the AI fixes switched on (environment JODAH_AI, default 'first,tutor')
fn ai_on(name: &str) -> bool {
    static AI: OnceLock<Vec<String>> = OnceLock::new();
    AI.get_or_init(|| {
        std::env::var("JODAH_AI").unwrap_or_else(|_| "first,tutor".into()).split(',').map(String::from).collect()
    })
    .iter()
    .any(|x| x == name)
}

/// jodah.HOLD (environment JODAH_HOLD, default '3,6')
fn hold_weights() -> (f64, f64) {
    static HOLD: OnceLock<(f64, f64)> = OnceLock::new();
    *HOLD.get_or_init(|| {
        let s = std::env::var("JODAH_HOLD").unwrap_or_else(|_| "3,6".into());
        let v: Vec<f64> = s.split(',').map(|x| x.trim().parse().unwrap_or(0.0)).collect();
        (v.first().copied().unwrap_or(3.0), v.get(1).copied().unwrap_or(6.0))
    })
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
/// order. HUMAN(phase 9): a person says whether to cast it.
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
        let key = g.player(o).key;
        let want = !g.db.get(h).tag(Tag::Ctr) && !crate::ai::plans::response_only(g, key, h);
        if want && castable(g, o, h, "lib") {
            g.last_x = 0; // cast free: X is 0
            if jodah {
                let t = g.player(o).turns;
                g.player_mut(o).milestone.entry("jodah").or_insert(t);
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

/// jodah.jodah_prio: the generic priorities; with Jodah out, legendary spells first, the dearer the better (more to
/// cascade into). With Jodah castable now, Jodah comes first: a legend cast before it gives up its cascade, and any
/// other spell that leaves too little mana for Jodah puts it off a turn.
pub fn jodah_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let d = g.db.get(c);
    let leg = legend_card(g, c);
    let out = jodahs(g, p) > 0;
    if prot(&d.name).is_some() && !(leg && out) {
        return 0; // kept for removal and wipes
    }
    let mut v = crate::ai::pool::generic_prio(g, p, c);
    let cmd = g.player(p).cmd;
    if c == cmd {
        return v.max(86);
    }
    if v == 0 && d.has_dsl() {
        v = (crate::dsl::card_value(g, p, c) * 10.0) as i32;
    }
    if leg && out {
        v = v.max(40) + 4 + (2 * d.cmc as i32).min(16);
    } else if leg && d.creature {
        v = v.max(35);
    }
    if ai_on("first") && v > 15 && jodah_payable(g, p) {
        let (gn, pips) = cost_of(g, p, cmd);
        let (cg, cp) = cost_of(g, p, c);
        if leg || !can_pay(g, p, gn + cg, &format!("{pips}{cp}"), false) {
            v = 15;
        }
    }
    v.min(90)
}

const KALDRA: [&str; 3] = ["Sword of Kaldra", "Shield of Kaldra", "Helm of Kaldra"];

/// jodah.jodah_tutor: the card a tutor finds: before Jodah, the mana that casts it (Coalition Relic fixes, Sisay's
/// Ring doesn't); with Jodah out or on the way, the dearest legend not already in hand (its cascade can find any
/// cheaper legend); the third Kaldra piece; Toxic Deluge or Force of Will under pressure (instant and sorcery tutors).
/// `okn`: the cards this tutor may find.
pub fn jodah_tutor(g: &Game, p: PlayerId, kind: &str, okn: &[CardId]) -> Option<CardId> {
    if !ai_on("tutor") {
        return None;
    }
    let pl = g.player(p);
    let have: Vec<CardId> = pl.perms.iter().filter_map(|&m| g.perm(m).cd).collect();
    let hand = &pl.hand;
    let mana = pl.lands.len()
        + pl.perms
            .iter()
            .filter(|&&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Rock) || g.db.get(c).tag(Tag::Dork)))
            .count();
    let out = jodahs(g, p) > 0;
    let mut order: Vec<CardId> = vec![];
    if (ai_on("ptutor") || ai_on("ptutorall"))
        && out
        && !hand.iter().any(|&c| PROTECTION_ORDER.contains(&cname(g, c)))
        && the_jodah(g, p).is_some_and(|j| !untargetable(g, j))
        && (ai_on("ptutorall") || removal_risk(g, p) >= 0.3)
    {
        // Jodah out and exposed: protect it
        order.extend(ids(g, &PROTECTION_ORDER).into_iter().filter(|c| !have.contains(c)));
    }
    if !out && pl.cmd_in_zone && mana < 4 {
        order.extend(ids(g, &["Coalition Relic", "Star Compass", "Moss Diamond", "Fyndhorn Elder"]));
    }
    let kaldra: Vec<CardId> = ids(g, &KALDRA).into_iter().filter(|c| !have.contains(c) && !hand.contains(c)).collect();
    if kaldra.len() == 1 {
        order.extend(kaldra);
    }
    let mut big: Vec<CardId> = searchable(g, p)
        .into_iter()
        .filter(|&c| legend_card(g, c) && g.db.get(c).creature && !hand.iter().any(|&h| cname(g, h) == cname(g, c)))
        .collect();
    big.sort_by(|&a, &b| g.db.get(b).cmc.cmp(&g.db.get(a).cmc).then_with(|| cname(g, a).cmp(cname(g, b))));
    if !hand.iter().any(|&c| legend_card(g, c) && g.db.get(c).cmc >= 6) {
        order.extend(big.iter().copied().filter(|&c| g.db.get(c).cmc >= 6 && cname(g, c) != "Szadek, Lord of Secrets"));
    }
    if kind == "is" {
        let power: i32 = g
            .opps(p)
            .flat_map(|q| g.player(q).perms.iter().copied())
            .filter(|&m| g.is_creature(m) && !g.perm(m).phased)
            .map(|m| epow(g, m))
            .sum();
        if power >= 15 {
            order.extend(ids(g, &["Toxic Deluge"]));
        }
        order.extend(ids(g, &["Demonic Tutor", "Force of Will"]));
    }
    if matches!(kind, "art" | "ench" | "ae") {
        order.extend(ids(g, &["Blackblade Reforged", "Privileged Position", "Coalition Relic", "Court of Ardenvale"]));
    }
    order.into_iter().find(|c| okn.contains(c))
}

// ================================================================== protecting Jodah (audit/jodah, 10-09)
/// A protection card the AI keeps for removal aimed at Jodah or a wipe that would hit it (jodah.PROTECT): its cost,
/// whether it is free while Jodah is out, what it stops ('tgt' targeted removal, 'destroy' destroy and damage
/// (targeted or a wipe), 'both', 'all' anything), and what it does: grant hexproof ('hex') and/or indestructible
/// ('ind') to Jodah or (_all) to every creature (Flawless Maneuver, Unbreakable Formation) or permanent (Heroic
/// Intervention, Lazotep Plating) until end of turn; 'swat' sends the removal at an opponent's permanent (the 'swat'
/// cards also guard a key legend, see jodah_protect); 'coat' puts Mithril Coat on Jodah; 'phase' is Teferi's
/// Protection. Listed cheapest first: free ones, then by mana (Bolt Bend: {R} with a creature of power 4 or more, as
/// with Jodah out); Teferi's Protection last.
#[derive(Debug, Clone, Copy)]
pub struct Prot {
    pub name: &'static str,
    pub cost: (u32, &'static str),
    pub free: bool,
    pub stops: &'static str,
    pub does: &'static str,
}

const fn pr(
    name: &'static str,
    cost: (u32, &'static str),
    free: bool,
    stops: &'static str,
    does: &'static str,
) -> Prot {
    Prot { name, cost, free, stops, does }
}

pub const PROTECT: [Prot; 13] = [
    pr("Flawless Maneuver", (2, "W"), true, "destroy", "ind_all"),
    pr("Deflecting Swat", (2, "R"), true, "tgt", "swat"),
    pr("Bolt Bend", (3, "R"), false, "tgt", "swat"),
    pr("Tamiyo's Safekeeping", (0, "G"), false, "both", "hexind"),
    pr("Loran's Escape", (0, "W"), false, "both", "hexind"),
    pr("Snakeskin Veil", (0, "G"), false, "tgt", "hex"),
    pr("Royal Treatment", (0, "G"), false, "tgt", "hex"),
    pr("Dark Endurance", (1, "B"), false, "destroy", "ind"),
    pr("Heroic Intervention", (1, "G"), false, "both", "hexind_all"),
    pr("Lazotep Plating", (1, "U"), false, "tgt", "hex_all"),
    pr("Unbreakable Formation", (2, "W"), false, "destroy", "ind_all"),
    pr("Mithril Coat", (3, ""), false, "destroy", "coat"),
    pr("Teferi's Protection", (2, "W"), false, "all", "phase"),
];

/// a card's PROTECT entry
pub fn prot(name: &str) -> Option<&'static Prot> {
    PROTECT.iter().find(|x| x.name == name)
}

/// wipes that indestructible survives
const WIPE_DESTROY: [&str; 5] = ["destroy", "dmg13", "austere", "austere2", "nib"];

/// what the tutors find (switch ptutor), best first
const PROTECTION_ORDER: [&str; 17] = [
    "Lightning Greaves",
    "Swiftfoot Boots",
    "Mithril Coat",
    "Giver of Runes",
    "Mother of Runes",
    "Flawless Maneuver",
    "Deflecting Swat",
    "Bolt Bend",
    "Heroic Intervention",
    "Tamiyo's Safekeeping",
    "Loran's Escape",
    "Teferi's Protection",
    "Snakeskin Veil",
    "Royal Treatment",
    "Lazotep Plating",
    "Unbreakable Formation",
    "Dark Endurance",
];

/// CI.RESPONSE_ONLY['jodah']: never cast for their interpreter value (brain.card_utility)
pub fn response_only(name: &str) -> bool {
    prot(name).is_some()
}

/// jodah._grant: keywords until end of turn
fn grant(g: &mut Game, ms: &[PermId], kws: &[&'static str]) {
    for &m in ms {
        for &k in kws {
            eot_kw(g, m, k);
        }
    }
}

/// jodah._saved: the reports' count of saves
fn saved(g: &mut Game, p: PlayerId, name: &str, wipe: bool) {
    stat(g, p, "jprot_saves", 1);
    stat(g, p, &format!("jprot_{}{name}", if wipe { "wipe_" } else { "" }), 1);
}

/// jodah.prot_cost: a protection card's mana cost now: Bolt Bend costs {3} less with a creature of power 4 or more
pub fn prot_cost(g: &Game, p: PlayerId, name: &str) -> (u32, &'static str) {
    let (mut gn, pips) = prot(name).unwrap().cost;
    if name == "Bolt Bend"
        && g.player(p).perms.iter().any(|&m| g.is_creature(m) && !g.perm(m).phased && epow(g, m) >= 4)
    {
        gn = 0;
    }
    (gn, pips)
}

/// jodah._cast_protection: cast protection card c from hand in response (free if it can be): true if it resolves
fn cast_protection(g: &mut Game, p: PlayerId, c: CardId) -> Res<bool> {
    let name = g.db.get(c).name.clone();
    let (gn, pips) = prot_cost(g, p, &name);
    let free = prot(&name).unwrap().free && commander_out(g, p);
    if !g.player(p).hand.contains(&c) || !castable(g, p, c, "hand") || !(free || can_pay(g, p, gn, pips, false)) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    if !free {
        pay(g, p, gn, pips, false)?;
    }
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::glog!(g, "  {} casts {}{}", pname(g, p), name, if free { " (free)" } else { "" });
    on_cast(g, p, c)?;
    let ok = counter_window(g, p, c, 6.0, vec![])?;
    if &*name == "Mithril Coat" && ok {
        let e = enter(g, p, c, Enter::default())?;
        let to = the_jodah(g, p).or_else(|| best_legend(g, p));
        g.perm_mut(e).attached = to;
    } else if &*name == TEFERIS_PROTECTION {
        g.player_mut(p).exile.push(c);
    } else {
        g.player_mut(p).gy.push(c);
    }
    Ok(ok)
}

/// jodah._swat_targets: where Deflecting Swat or Bolt Bend can send the removal: any opponent's permanent it can target
fn swat_targets(g: &Game, p: PlayerId, kind: &str) -> Vec<PermId> {
    g.opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&x| !g.perm(x).phased && !untargetable(g, x) && (g.is_creature(x) || !kind.starts_with("dmg")))
        .collect()
}

/// jodah._apply: the protection's effect on Jodah j (and the rest of p's board); false if it found nothing to do
/// (Deflecting Swat with no new target left)
#[allow(clippy::too_many_arguments)]
fn apply(
    g: &mut Game,
    p: PlayerId,
    j: PermId,
    name: &str,
    does: &str,
    spell: Option<CardId>,
    actor: Option<PlayerId>,
    kind: &str,
) -> Res<bool> {
    if does == "phase" {
        teferis_protection(g, p);
        return Ok(true);
    }
    if does == "swat" {
        let alt = swat_targets(g, p, kind);
        let Some(t) = first_max(&alt, |x| (pval(g, x), Some(owner(g, x)) == actor)) else { return Ok(false) };
        crate::glog!(
            g,
            "    {name}: {} now targets {} ({})",
            spell.map_or("", |s| cname(g, s)),
            g.perm(t).name,
            pname(g, owner(g, t))
        );
        apply_removal(g, actor, t, kind, spell)?;
        return Ok(true);
    }
    let kws: &[&'static str] = match does.split('_').next().unwrap_or("") {
        "hex" => &["hexproof"],
        "ind" => &["indestructible"],
        "hexind" => &["hexproof", "indestructible"],
        _ => &[],
    };
    let scope: Vec<PermId> = if does.ends_with("_all") {
        g.player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| !g.perm(m).phased && (g.is_creature(m) || does != "ind_all"))
            .collect()
    } else {
        vec![j]
    };
    grant(g, &scope, kws);
    match name {
        "Snakeskin Veil" | "Royal Treatment" => g.perm_mut(j).plus += 1, // the counter / the Royal Role's +1/+1
        "Tamiyo's Safekeeping" => gain(g, p, 2)?,
        "Lazotep Plating" => amass(g, p, 1)?,
        "Loran's Escape" => crate::cardcode::scry(g, p, 1, false)?,
        _ => {}
    }
    Ok(true)
}

/// jodah._stops: does protection that stops `stops` stop removal (or a wipe) of this kind?
fn stops_it(stops: &str, kind: &str, targeted: bool) -> bool {
    let destroyish = kind == "destroy" || kind.starts_with("dmg") || (!targeted && WIPE_DESTROY.contains(&kind));
    stops == "all"
        || (matches!(stops, "tgt" | "both") && targeted)
        || (matches!(stops, "destroy" | "both") && destroyish)
}

/// jodah._rune_save: Giver of Runes (another creature: anything) or Mother of Runes (a coloured spell): tap for
/// protection
fn rune_save(g: &mut Game, p: PlayerId, j: PermId, spell: Option<CardId>) -> Option<&'static str> {
    for name in ["Giver of Runes", "Mother of Runes"] {
        for m in mine_named(g, p, name) {
            let x = g.perm(m);
            if m == j || x.tapped || x.sick {
                continue;
            }
            if name == "Mother of Runes"
                && !spell.is_some_and(|s| g.db.get(s).pips.chars().any(|ch| "WUBRG".contains(ch)))
            {
                continue;
            }
            g.perm_mut(m).tapped = true;
            crate::glog!(g, "    {} taps {name}: Jodah gains protection", pname(g, p));
            return Some(name);
        }
    }
    None
}

/// jodah._plaza_save: Plaza of Heroes: {3}, {T}, exile it: a legendary creature gains hexproof and indestructible until
/// end of turn
fn plaza_save(g: &mut Game, p: PlayerId, j: PermId) -> Res<bool> {
    if ai_on("noplaza") {
        return Ok(false); // audit/jodah A/B: as before 10-09
    }
    let plazas: Vec<LandId> = g
        .player(p)
        .lands
        .iter()
        .copied()
        .filter(|&l| cname(g, g.land(l).cd) == "Plaza of Heroes" && !g.land(l).tapped)
        .collect();
    for l in plazas {
        if !can_pay_without(g, p, l, 3, "") || !pay_without(g, p, l, 3, "")? {
            continue;
        }
        let cd = g.land(l).cd;
        crate::engine::turn::remove_land(g, p, l);
        g.player_mut(p).exile.push(cd);
        crate::glog!(g, "    {} exiles Plaza of Heroes: Jodah gains hexproof and indestructible", pname(g, p));
        if ability_window_card(g, p, cd, "hexproof and indestructible for Jodah", None, None)? {
            grant(g, &[j], &["hexproof", "indestructible"]);
            return Ok(true);
        }
        return Ok(false);
    }
    Ok(false)
}

/// jodah.key_legend: a legendary creature other than Jodah worth a redirect (Deflecting Swat, Bolt Bend) while Jodah
/// is not out to need it: value 5 or more (a bomb, or power 8 or more)
fn key_legend(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.is_creature(m)
        && legendary(g, m)
        && g.perm(m).cd.is_some()
        && name_of(g, m) != JODAH
        && jodahs(g, p) == 0
        && pval(g, m) >= 5.0
}

/// jodah.jodah_protect: removal aimed at Jodah: the cheapest answer that stops it (free spells, Giver or Mother of
/// Runes, then by mana, Plaza of Heroes, Teferi's Protection last). Aimed at a key legend: a redirect only. True if it
/// is safe.
pub fn jodah_protect(
    g: &mut Game,
    p: PlayerId,
    m: PermId,
    kind: &str,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if g.perm(m).cd.is_none() || matches!(kind, "edict" | "wipe") || actor.is_none() || actor == Some(p) {
        return Ok(false);
    }
    let other = name_of(g, m) != JODAH;
    if other && !key_legend(g, p, m) {
        return Ok(false);
    }
    let shrink = kind.starts_with("shrink") || kind == "zero"; // -X/-X: indestructible doesn't help
    if !other && let Some(n) = rune_save(g, p, m, spell) {
        saved(g, p, n, false);
        return Ok(true);
    }
    for pt in PROTECT.iter() {
        let Some(c) = g.player(p).hand.iter().copied().find(|&c| cname(g, c) == pt.name) else { continue };
        if !stops_it(pt.stops, kind, true) || (shrink && pt.stops == "destroy") || (other && pt.does != "swat") {
            continue;
        }
        if pt.does == "swat" && (spell.is_none() || swat_targets(g, p, kind).is_empty()) {
            continue;
        }
        if pt.name == TEFERIS_PROTECTION && g.player(p).life_locked {
            continue;
        }
        if !cast_protection(g, p, c)? || !apply(g, p, m, pt.name, pt.does, spell, actor, kind)? {
            return Ok(false);
        }
        let label = format!("{}{}", pt.name, if other { " (legend)" } else { "" });
        saved(g, p, &label, false);
        return Ok(true);
    }
    if !other && plaza_save(g, p, m)? {
        saved(g, p, "Plaza of Heroes", false);
        return Ok(true);
    }
    Ok(false)
}

/// jodah.jodah_wipe_response: a wipe that would take Jodah: indestructible (destroy and damage wipes) or Teferi's
/// Protection. 'all', 'indes' (the whole board is indestructible) or None (Jodah alone may be safe)
pub fn jodah_wipe_response(g: &mut Game, p: PlayerId, kind: Sym, caster: PlayerId) -> Res<Option<Sym>> {
    let Some(j) = the_jodah(g, p) else { return Ok(None) };
    if caster == p || !crate::ai::decks::WipeRule::new(g, caster, kind).hits(g, j, p) {
        return Ok(None);
    }
    for pt in PROTECT.iter() {
        if pt.does == "swat" || !stops_it(pt.stops, kind, false) {
            continue;
        }
        let Some(c) = g.player(p).hand.iter().copied().find(|&c| cname(g, c) == pt.name) else { continue };
        if pt.name == TEFERIS_PROTECTION && g.player(p).life_locked {
            continue;
        }
        if !cast_protection(g, p, c)? {
            return Ok(None);
        }
        apply(g, p, j, pt.name, pt.does, None, None, "")?;
        saved(g, p, pt.name, true);
        return Ok(match pt.does {
            "phase" => Some("all"),
            "ind_all" | "hexind_all" => Some("indes"),
            _ => None,
        });
    }
    if WIPE_DESTROY.contains(&kind) && plaza_save(g, p, j)? {
        saved(g, p, "Plaza of Heroes", true);
    }
    Ok(None)
}

/// jodah.jodah_hold: (card, value) of keeping mana up for a protection spell while Jodah is out (switch hold): worth
/// more the likelier an opponent holds instant removal; free spells (Flawless Maneuver, Deflecting Swat) need nothing
/// kept
pub fn jodah_hold(g: &Game, p: PlayerId) -> (Option<CardId>, f64) {
    if !ai_on("hold") || jodahs(g, p) == 0 {
        return (None, 0.0);
    }
    let cs: Vec<CardId> = g
        .player(p)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            let Some(pt) = prot(cname(g, c)) else { return false };
            let (gn, pips) = prot_cost(g, p, pt.name);
            !(pt.free && commander_out(g, p)) && can_pay(g, p, gn, pips, false)
        })
        .collect();
    let Some(c) = first_min(&cs, |c| {
        let (gn, pips) = prot_cost(g, p, cname(g, c));
        gn + pips.chars().count() as u32
    }) else {
        return (None, 0.0);
    };
    let (a, b) = hold_weights();
    (Some(c), a + b * removal_risk(g, p))
}

/// jodah.jodah_options: main phase: Swiftfoot Boots, Lightning Greaves or Mithril Coat onto Jodah when they are
/// elsewhere (the generic equip only moves unattached equipment)
pub fn jodah_options(g: &mut Game, p: PlayerId, _post: bool) -> Vec<Opt> {
    let Some(j) = the_jodah(g, p) else { return vec![] };
    let mut o = vec![];
    for e in g.player(p).perms.clone() {
        let x = g.perm(e);
        if x.cd.is_none() || x.phased || x.attached == Some(j) {
            continue;
        }
        let name = name_of(g, e);
        let Some(n) = equip_cost(name) else { continue };
        if !can_pay(g, p, n, "", false) || (name != "Mithril Coat" && untargetable(g, j)) {
            continue;
        }
        if name == "Mithril Coat" && indestructible(g, j) {
            continue;
        }
        let label = format!("equip {name} to Jodah");
        o.push(plan(4.0 + 0.2 * pval(g, j) - 0.5 * n as f64, label, jodah_equip, pack(e.0, j.0)));
    }
    o
}

fn equip_cost(name: &str) -> Option<u32> {
    match name {
        "Swiftfoot Boots" => Some(1),
        "Lightning Greaves" => Some(0),
        "Mithril Coat" => Some(3),
        _ => None,
    }
}

fn jodah_equip(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (e, j) = unpack(arg);
    let (e, j) = (PermId(e), PermId(j));
    let n = equip_cost(name_of(g, e)).unwrap_or(0);
    if !on(g, p, j) || !can_pay(g, p, n, "", false) {
        return Ok(false);
    }
    equip_to(g, p, e, j, n)
}

/// jodah._runes_home: Jodah's AI keeps Mother and Giver of Runes home (untapped, to protect Jodah) instead of
/// attacking with them; other decks attack with them as usual
fn runes_home(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    if g.player(p).key == "jodah" {
        g.perm_mut(m).noatk = true;
    }
    Ok(())
}

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

/// jodah.sisay_pick: a legendary permanent card with mana value below Sisay's power: the tutor wish list's pick (the
/// third Kaldra piece, the dearest legendary creature), else the dearest legend
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
    if let Some(n) = jodah_tutor(g, p, "leg", &ok)
        && let Some(&c) = ok.iter().find(|&&c| c == n)
    {
        return Some(c);
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
            g.perm_mut(m).plus += 1;
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
/// 6, at least 10) and the library has more than X cards. HUMAN(phase 9): a person decides.
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
    if x == 0 {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), &format!("pay {x} life, draw {x}"), None)? || !once_per_turn(g, p, "tymna") {
        return Ok(());
    }
    if g.player(p).life < x || g.player(p).life_locked {
        return Ok(());
    }
    if !(g.player(p).life - x >= crate::ai::necro_floor(g, p) && g.player(p).library.len() > x as usize) {
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
fn garland_pick(g: &Game, _p: PlayerId, q: PlayerId) -> Option<PermId> {
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
/// (then the one with the smaller board, the less likely to lose the crown). HUMAN(phase 9): a person chooses.
fn garland_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let opps: Vec<PlayerId> = g.opps(o).filter(|&q| g.player(q).alive && !player_hexproof(g, q)).collect();
    let Some(q) = first_max(&opps, |q| {
        let t = garland_pick(g, o, q);
        (t.map_or(-1.0, |t| pval(g, t)), -board_power(g, q))
    }) else {
        return Ok(());
    };
    let label = format!("{} becomes the monarch", pname(g, q));
    if !trigger_window(g, o, Some(src), &label, Some(4.0))? || !g.player(q).alive {
        return Ok(());
    }
    stat(g, o, "jr_garland_etb", 1);
    crate::cardcode::become_monarch(g, q)
}

/// Whenever an opponent becomes the monarch, gain control of target creature that player controls for as long as
/// they're the monarch (garland_check ends it). HUMAN(phase 9): a person picks the creature.
fn garland_steal(g: &mut Game, src: Src, q: PlayerId) -> Res {
    let o = owner(g, src);
    if q == o || !g.player(q).alive {
        return Ok(());
    }
    let Some(t) = garland_pick(g, o, q) else { return Ok(()) };
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
        g.perm_mut(t).plus += 1;
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
        g.perm_mut(src).plus += x;
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
    g.perm_mut(src).plus += 1;
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
    if pval(g, t) < 3.0 {
        return Ok(vec![]);
    }
    let label = format!("Kaervek's Purge on {}", g.perm(t).name);
    Ok(vec![plan(pval(g, t) * 0.5 - 0.8, label, purge_go, pack(c.0 as u32, t.0))])
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
    g.player_mut(p).spells_this_turn += 1;
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
/// resolution: draw, removal and tutors). HUMAN(phase 9): a person decides.
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
    if p != owner(g, src)
        || post != Some(false)
        || g.active != Some(p)
        || g.player(p).hand.len() > 2
        || total_mana(g, p, false) < 3
    {
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
    let cards = pl.library.iter().chain(&pl.hand).chain(&pl.gy).chain(&pl.exile).copied().chain(perms).chain([pl.cmd]);
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
    for name in ["Mother of Runes", "Giver of Runes"] {
        r.card(db, name)?.as_enters = Some(runes_home);
    }
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
