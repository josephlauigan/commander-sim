//! Ward (rule 702.21): "Whenever this permanent becomes the target of a spell or ability an opponent controls, counter
//! that spell or ability unless that player pays [the ward cost]."
//!
//! `pay_ward` is the one place every ward is paid: ward {N} (a card's own, `CardDef::ward`, and Sheltered by Ghosts'),
//! Sauron's "sacrifice a legendary artifact or creature", Hexing Squelcher's "pay 2 life", Prismari's "pay 5 life",
//! Valgavoth's "sacrifice three nonland permanents". It is called where a targeted effect lands on its target: every
//! removal kind (`removal::apply_removal_rest`: destroy, exile, bounce, damage, -X/-X, tuck ...), the steal effects
//! (`control::steal`), and the targeted tap effects. The AI pays when it can afford it (a spell it chose to cast at a
//! warded target was priced with the ward in `removal_options`); a person is asked.

use crate::cardcode;
use crate::engine::mana::{can_pay, pay};
use crate::engine::values::*;
use crate::engine::zones::*;
use crate::flow::Res;
use crate::ids::{PermId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

/// `actor`'s spell or ability targets permanent `m`: false if a ward on it counters the spell or ability (the
/// effect does not happen), true if there is no ward or it was paid. Targeting your own permanent never wards.
pub fn pay_ward(g: &mut Game, actor: PlayerId, m: PermId) -> Res<bool> {
    let owner = g.perm(m).owner;
    if actor == owner || !g.perm(m).on_bf {
        return Ok(true);
    }
    let name = g.perm(m).name;
    let countered = |g: &mut Game| {
        crate::glog!(g, "    ward counters the effect on {}", name);
        Ok(false)
    };
    // ward {N}
    let ward = g.perm(m).cd.map_or(0, |c| g.db.get(c).ward) + cardcode::aura_ward(g, m);
    if ward > 0 {
        if crate::human::is_human(g, actor) {
            let prompt = format!("{}: ward {{{ward}}}: pay it or the spell or ability is countered?", shown(g, m));
            if !crate::human::mana::pay_tax_ask(g, actor, ward, "ward", Some(prompt))? {
                return countered(g);
            }
        } else {
            if !can_pay(g, actor, ward, "", false) {
                return countered(g);
            }
            pay(g, actor, ward, "", false)?;
        }
    }
    if card_tag(g, m, Tag::Sauron) {
        // Sauron's ward: sacrifice a legendary artifact or creature
        let legs = ward_legends(g, actor);
        if crate::human::is_human(g, actor) && !legs.is_empty() {
            // the person may let the spell be countered instead
            let labels: Vec<String> = legs
                .iter()
                .map(|&x| crate::human::legal::describe_target(g, actor, crate::human::Target::Perm(x)))
                .collect();
            let prompt =
                format!("{}: ward - sacrifice a legendary artifact or creature, or it is countered", shown(g, m));
            let Some(k) =
                crate::human::choices::choose(g, actor, "choose", &prompt, &labels, Some("let it be countered"), None)?
            else {
                return countered(g);
            };
            die(g, legs[k], "sac")?;
        } else {
            let Some(x) = min_by(&legs, |x| pval(g, x)) else { return countered(g) };
            die(g, x, "sac")?;
        }
    }
    // --- Tiers 4/5 (krenko): Hexing Squelcher's "Ward—Pay 2 life" (on itself and its controller's other creatures)
    if !crate::impls::t45_krenko::life_ward(g, actor, m)? {
        return countered(g);
    }
    // --- scion: Prismari, the Inspiration's "Ward—Pay 5 life" (Scion as a copy of it too)
    if !crate::impls::t6_scion::prismari_ward(g, actor, m)? {
        return countered(g);
    }
    // --- Tiers 1/2 (kaalia-sythis): Valgavoth, Terror Eater's "Ward—Sacrifice three nonland permanents"
    if !crate::impls::t12_kaalia_sythis::sac_ward(g, actor, m)? {
        return countered(g);
    }
    Ok(true)
}

/// the name of what m shows (a land permanent has no name of its own)
fn shown(g: &Game, m: PermId) -> String {
    g.perm(m).cd.map_or_else(|| g.perm(m).name.to_string(), |c| g.db.get(c).name.to_string())
}

/// Can `actor` pay for a spell or ability that targets `m` (the AI's look-ahead): no ward, or one it can afford now
/// (mana wards only: `extra` more mana is spent first).
pub fn ward_affordable(g: &Game, actor: PlayerId, m: PermId, extra_generic: u32, pips: &str) -> bool {
    let ward = g.perm(m).cd.map_or(0, |c| g.db.get(c).ward);
    ward == 0 || g.perm(m).owner == actor || can_pay(g, actor, extra_generic + ward, pips, false)
}
