//! Tier 6 plan slot for `godo` (audit/t6/godo-bandit-warlord/SPEC.md). Contract: see `t6_plans.rs`.
//!
//! What the plan does: keep hands that reach six mana and have a named way to win (mulligan section); develop the
//! zero-drops before Vexing Bauble and Trinisphere shut them off; tutor Hammer first, then the copier or
//! protection the board asks for; go for a line early (the spec: a turn-2/3 win beats waiting) and hold the line
//! when the table shows answers and Godo holds none.

use crate::ids::{CardId, PlayerId};
use crate::state::Game;

use super::t6_combos_godo::{GODO, HELM, TOKEN_COPIERS, bf, cname, in_hand, in_lib};

const HAMMER: &str = "Hammer of Nazahn";

/// zero- and one-mana acceleration: counts as a mana source next to the lands
const ACCEL: &[&str] = &[
    "Sol Ring",
    "Mana Vault",
    "Chrome Mox",
    "Mox Diamond",
    "Lotus Petal",
    "Simian Spirit Guide",
    "Everflowing Chalice",
    "Arcane Signet",
];
const RITUALS: &[&str] = &["Rite of Flame", "Desperate Ritual", "Pyretic Ritual", "Seething Song", "Jeska's Will"];
/// a way to win, a tutor for one, or an engine that carries a long game (spec: "a path to victory")
const PATH: &[&str] = &[
    "Helm of the Host",
    HAMMER,
    "Gamble",
    "Imperial Recruiter",
    "Goblin Engineer",
    "Goblin Welder",
    "Treasonous Ogre",
    "Magnetic Theft",
    "Brass Squire",
    "Peter Parker's Camera",
    "Cursed Mirror",
    "Sculpting Steel",
    "Jaxis, the Troublemaker",
    "Tezzeret, Cruel Captain",
    "Urza's Saga",
    "Inventors' Fair",
    "Command Beacon",
    "The One Ring",
    "Party Thrasher",
    "Staff of Compleation",
    "Final Fortune",
    "Last Chance",
    "Wheel of Fortune",
];
/// the zero-drops that Vexing Bauble (and Trinisphere) would shut off
const FREE_DROPS: &[&str] =
    &["Chrome Mox", "Mox Diamond", "Mox Opal", "Lotus Petal", "Lion's Eye Diamond", "Sol Ring", "Mana Vault"];

fn count(g: &Game, p: PlayerId, names: &[&str]) -> usize {
    g.player(p).hand.iter().filter(|&&c| names.contains(&cname(g, c))).count()
}

fn lands_in_hand(g: &Game, p: PlayerId) -> usize {
    g.player(p).hand.iter().filter(|&&c| g.db.get(c).land).count()
}

fn godo_out(g: &Game, p: PlayerId) -> bool {
    bf(g, p, GODO).is_some()
}

/// cast priority of `c` (0-100 scale like `deck_prio`; 0 or less keeps it in hand); None = no opinion
pub fn prio(g: &Game, p: PlayerId, c: CardId) -> Option<f64> {
    let hand_free = |skip: CardId| g.player(p).hand.iter().any(|&x| x != skip && FREE_DROPS.contains(&cname(g, x)));
    match cname(g, c) {
        // spec: zero-drops first; Bauble counters our own free spells
        "Vexing Bauble" if hand_free(c) => Some(5.0),
        // Trinisphere makes our cheap cards cost three: only once the cheap pieces are down, and with a payoff near
        "Trinisphere" if hand_free(c) || (!godo_out(g, p) && g.player(p).turns <= 2) => Some(0.0),
        // Blood Moon halves our own Tomb and City: not while we lean on them and have not cast Godo
        "Blood Moon"
            if !godo_out(g, p) && (bf(g, p, "Ancient Tomb").is_some() || bf(g, p, "City of Traitors").is_some()) =>
        {
            Some(0.0)
        }
        // "one of the best starts available"
        "The One Ring" | "Party Thrasher" if g.player(p).turns <= 3 => Some(92.0),
        // an extra turn only helps once Godo is out and has something to untap into
        "Final Fortune" | "Last Chance" if !godo_out(g, p) => Some(0.0),
        _ => None,
    }
}

/// tutor wish list, best first; None = no opinion (the closest combo's missing pieces)
pub fn wish(g: &Game, p: PlayerId) -> Option<Vec<CardId>> {
    let hammer_out = bf(g, p, HAMMER).is_some();
    let helm_out = bf(g, p, HELM).is_some();
    let godo_on = godo_out(g, p);
    let copier_in_hand = count(g, p, TOKEN_COPIERS) > 0;
    let mut names: Vec<&str> = vec![];
    if !hammer_out && !(godo_on && helm_out) {
        names.push(HAMMER); // Godo's Helm then attaches itself, and the cast is 5 mana cheaper
    }
    if !godo_on && !helm_out && in_lib(g, p, HELM) && in_hand(g, p, "Brass Squire").is_none() && !hammer_out {
        names.push("Helm of the Host"); // only if Godo's trigger cannot be trusted; behind Hammer
    }
    if copier_in_hand && in_hand(g, p, "Dualcaster Mage").is_none() {
        names.push("Dualcaster Mage");
    }
    if godo_on {
        names.push("Deflecting Swat");
    }
    names.extend(["Hexing Squelcher", "Pyroblast", "Dualcaster Mage", "Imperial Recruiter", "Gamble"]);
    let mut out: Vec<CardId> = names.iter().filter_map(|n| g.db.id(n)).collect();
    for c in crate::cardcode::missing_pieces(g, p) {
        if !out.contains(&c) {
            out.push(c);
        }
    }
    Some(out)
}

/// mulligan keep rule for the current opening hand (spec "Mulligan"): reach six mana fast and see a way to win
pub fn keep(g: &Game, p: PlayerId) -> Option<bool> {
    Some(keep_reasons(g, p).0)
}

/// `keep`'s verdict with its reasons in plain words (practice mode's mulligan hint explains the hand with them)
pub fn keep_reasons(g: &Game, p: PlayerId) -> (bool, Vec<String>) {
    let lands = lands_in_hand(g, p);
    let accel = count(g, p, ACCEL);
    let rituals = count(g, p, RITUALS);
    let path = count(g, p, PATH) + (count(g, p, &["Dualcaster Mage"]) > 0 && count(g, p, TOKEN_COPIERS) > 0) as usize;
    let sources = lands + accel;
    let tutor = count(g, p, &["Gamble", "Imperial Recruiter", "Goblin Engineer"]);
    let in_hand = |names: &[&str]| -> String {
        let v: Vec<&str> =
            g.player(p).hand.iter().map(|&c| cname(g, c)).filter(|n| names.contains(n)).collect();
        if v.is_empty() { "none".to_string() } else { v.join(", ") }
    };
    let mut why = vec![
        "Godo's rule: reach six mana fast and see a way to win. It counts lands, fast mana and a path to victory \
         (Helm of the Host, Hammer of Nazahn, the tutors that find them, or Dualcaster Mage with a token copier)."
            .to_string(),
        format!("Lands: {lands}. Fast mana: {accel} ({}). Rituals: {rituals}. Tutors: {tutor}.", in_hand(ACCEL)),
        format!("Path to victory: {path} ({}).", in_hand(PATH)),
    ];
    let verdict = if lands == 0 && accel < 2 {
        why.push("No land and fewer than two fast-mana pieces: ship it.".into());
        false
    } else if lands == 1 && (accel < 2 && !(accel == 1 && rituals + tutor >= 2)) {
        // one land needs two accelerators (or one plus a ritual and a tutor) to reach Godo
        why.push(
            "One land needs two fast-mana pieces (or one plus two rituals or tutors) to reach Godo: ship it.".into(),
        );
        false
    } else if sources < 2 || lands >= 6 {
        why.push(if lands >= 6 {
            "Six or more lands: too little action, ship it.".into()
        } else {
            "Fewer than two mana sources: ship it.".into()
        });
        false
    } else if path >= 1 {
        why.push("Enough mana and a path to victory: keep.".into());
        true
    } else {
        // spec scenario 16: ramp without a path to victory is shipped, however much of it there is
        why.push("Enough mana but no path to victory: the rule ships ramp without a way to win.".into());
        false
    };
    (verdict, why)
}

/// extra value of passing with mana up this turn (added to the shared hold value); None = no opinion
pub fn hold_up(g: &Game, p: PlayerId) -> Option<f64> {
    // early turns: develop the mana; Godo needs every source he can get, and nothing here is worth holding for yet
    if !godo_out(g, p) && g.player(p).turns <= 3 && count(g, p, ACCEL) + count(g, p, RITUALS) > 0 {
        return Some(-2.0);
    }
    None
}

/// go for the combo named `combo` now (Some(true)), wait (Some(false)), or no opinion (None)
pub fn go_now(g: &Game, p: PlayerId, _combo: &str) -> Option<bool> {
    let prot = count(
        g,
        p,
        &["Pyroblast", "Red Elemental Blast", "Deflecting Swat", "Redirect Lightning", "Tibalt's Trickery"],
    )
    .min(2);
    let risk = crate::ai::t6_table::go_risk(g, p) * 0.8f64.powi(prot as i32);
    // spec: a turn-2/3 win beats waiting; later the bar rises with the round like the shared rule
    let thr = match g.round {
        0..=3 => 0.85,
        4..=5 => 0.7,
        _ => 1.0,
    };
    Some(risk <= thr)
}
