//! What the AI would do at your decision (practice mode's Hint; Python's `play/advisor.py`).
//!
//! The AI is asked on a copy of the game (as the look-ahead does), never the real one, so asking changes nothing.
//! In the copy your seat has no mana pool; mana you have floating counts as mana of any colour there.
//!
//! `hint(g, p, req)` returns `{"text": one line, "detail": [lines], "choice": the AI's pick or null}`.

pub mod mulligan;
mod other;

use crate::view::seam::pool_total;
use serde_json::{Value, json};
use sim_core::ai::{brain, search};
use sim_core::control::Request;
use sim_core::engine::stack::pick_counter;
use sim_core::ids::PlayerId;
use sim_core::state::{Game, Step};
use std::collections::HashSet;

/// a copy of g to play on, with the person's floating mana counted as mana of any colour
pub fn copy(g: &mut Game, p: PlayerId) -> Game {
    let pool = pool_total(g, p);
    let mut g2 = search::clone(g);
    g2.player_mut(p).floating.any += pool;
    g2.player_mut(p).pool = Default::default(); // the copy pays like the AI, from its lands
    g2
}

fn label(lbl: &str, hand: &HashSet<String>) -> String {
    if lbl.starts_with("stop") {
        return format!(
            "Pass: play nothing more this phase{}",
            if lbl.contains("hold") { " (hold your mana up)" } else { "" }
        );
    }
    if hand.contains(lbl) {
        return format!("Cast {lbl}");
    }
    if let Some((spell, target)) = lbl.split_once(" -> ")
        && hand.contains(spell)
    {
        return format!("Cast {spell} on {target}");
    }
    lbl.to_string()
}

fn main_phase(g: &mut Game, p: PlayerId) -> Value {
    let mut g2 = copy(g, p);
    let post = g.step == Step::Main2;
    let opts = match brain::main_options(&mut g2, p, post) {
        Ok(o) => o,
        Err(_) => return json!({"text": "Nothing to do: pass.", "detail": [], "choice": null}),
    };
    let mut names: HashSet<String> = g2.player(p).hand.iter().map(|&c| g2.db.get(c).name.to_string()).collect();
    for c in g2.player(p).cmds_in_zone() {
        names.insert(g2.db.get(c).name.to_string());
    }
    if opts.len() < 2 {
        return match opts.first() {
            Some(o) => json!({"text": label(&o.label, &names), "detail": [], "choice": o.label}),
            None => json!({"text": "Nothing to do: pass.", "detail": [], "choice": null}),
        };
    }
    let (pick, detail): (usize, Vec<String>);
    if search::enabled(&mut g2, p) {
        search::LAST_SCORES.with(|l| l.borrow_mut().clear());
        let o = search::choose(&mut g2, p, post, &opts);
        let mut ranked = search::LAST_SCORES.with(|l| l.borrow().clone());
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut d = vec![
            "Look-ahead (each play tried on six copies of the game, played on to the end of your next turn):"
                .to_string(),
        ];
        d.extend(ranked.iter().map(|(l, v)| format!("{}: {v:+.1}", label(l, &names))));
        pick = o.unwrap_or_else(|| best_utility(&opts));
        detail = d;
    } else {
        pick = best_utility(&opts);
        let mut idx: Vec<usize> = (0..opts.len()).collect();
        idx.sort_by(|&a, &b| opts[b].utility.partial_cmp(&opts[a].utility).unwrap_or(std::cmp::Ordering::Equal));
        let mut d = vec!["The AI's own ratings (no look-ahead in this game):".to_string()];
        d.extend(idx.iter().take(4).map(|&i| format!("{}: {:.1}", label(&opts[i].label, &names), opts[i].utility)));
        detail = d;
    }
    json!({"text": label(&opts[pick].label, &names), "detail": detail, "choice": opts[pick].label})
}

fn best_utility(opts: &[sim_core::hooks::Opt]) -> usize {
    let mut b = 0;
    for i in 1..opts.len() {
        if opts[i].utility > opts[b].utility {
            b = i;
        }
    }
    b
}

fn attack(g: &mut Game, p: PlayerId) -> Value {
    let mut g2 = copy(g, p);
    if !search::enabled(&mut g2, p) {
        return json!({"text": "Attack with the creatures that can attack safely.",
                      "detail": ["(no look-ahead in this game)"], "choice": null});
    }
    let Some((d, how)) = search::choose_attack(&mut g2, p) else {
        return json!({"text": "No attack.", "detail": [], "choice": null});
    };
    let who = g.player(d).name.to_string();
    let text = match how {
        "none" => "Don't attack.".to_string(),
        "all" => format!("Attack {who} with everything."),
        _ => format!("Attack {who} with the creatures that can attack safely."),
    };
    json!({"text": text, "detail": ["Look-ahead over attacking each opponent (safe attackers, or everything) and not attacking."],
           "choice": [d.index(), how]})
}

fn counter(g: &mut Game, p: PlayerId, spell: sim_core::ids::CardId) -> Option<Value> {
    let cc = g.cur_cast.as_ref()?;
    if cc.0 != spell {
        return None;
    }
    let sname = g.db.get(spell).name.to_string();
    let Some(ctr) = pick_counter(g, p, spell) else {
        return Some(
            json!({"text": format!("Let {sname} resolve: you can't counter it now."), "detail": [], "choice": false}),
        );
    };
    let cname = g.db.get(ctr).name.to_string();
    let active = g.active?;
    let pool = pool_total(g, p);
    let cast = g.cur_cast.clone();
    let mut g2 = search::clone(g); // asked on a copy: the real game's counters stay put
    g2.cur_cast = cast;
    g2.player_mut(p).floating.any += pool;
    g2.player_mut(p).pool = Default::default(); // the copy pays like the AI, from its lands
    if !search::enabled(&mut g2, p) {
        return Some(
            json!({"text": format!("You could counter {sname} with {cname}; the hint needs the look-ahead AI to weigh it."),
                           "detail": [], "choice": null}),
        );
    }
    let yes = search::choose_counter(&mut g2, p, active, spell);
    Some(json!({"text": if yes { format!("Counter {sname} with {cname}.") } else { format!("Let {sname} resolve.") },
                "detail": ["Look-ahead: the game played on with and without countering it, to the end of your next turn."],
                "choice": yes}))
}

/// the AI's advice for request req (the game is waiting on it, so it holds still). Always `text` (a non-empty
/// string), `detail` (an array) and `choice`; a panic while working it out on the copy becomes a plain sentence.
pub fn hint(g: &mut Game, p: PlayerId, req: &Request) -> Value {
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| hint_inner(g, p, req)));
    let mut v = r.unwrap_or_else(|_| json!({"text": format!("The AI could not weigh this {} decision; read the prompt and pick what helps you most.", req.kind), "detail": [], "choice": null}));
    let ok = v.get("text").and_then(Value::as_str).is_some_and(|t| !t.trim().is_empty() && t != "null");
    if !ok {
        v["text"] = json!(format!("Weigh the options for this {} decision against your plan.", req.kind));
    }
    if !v.get("detail").is_some_and(Value::is_array) {
        v["detail"] = json!([]);
    }
    if v.get("choice").is_none() {
        v["choice"] = Value::Null;
    }
    v
}

fn hint_inner(g: &mut Game, p: PlayerId, req: &Request) -> Value {
    match &*req.kind {
        "priority" => {
            let stack_empty = req.data.get("stack").and_then(|s| s.as_array()).is_none_or(|a| a.is_empty());
            if !stack_empty {
                let spell = g.cur_cast.as_ref().map(|c| c.0);
                if let Some(s) = spell
                    && let Some(res) = counter(g, p, s)
                {
                    return res;
                }
            }
            if g.active == Some(p) && matches!(g.step, Step::Main1 | Step::Main2) && stack_empty {
                return main_phase(g, p);
            }
            json!({"text": "Pass, unless you have an instant you want to use now.",
                   "detail": ["(the AI only plays instants at this point in answer to something)"], "choice": null})
        }
        "attack" => attack(g, p),
        _ => other::hint(g, p, req),
    }
}
