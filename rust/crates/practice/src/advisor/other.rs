//! Hints for every decision other than the main phase, attacks and counters: the AI's own pick where it has a
//! function for it (the mulligan and bottom, discards, sacrifices, scry and surveil), otherwise a full sentence on
//! what the decision is and what to weigh. Never empty, never null.

use serde_json::{Value, json};
use sim_core::ai::{brain, topdeck};
use sim_core::control::Request;
use sim_core::human::Target;
use sim_core::human::choices::card_label;
use sim_core::human::legal;
use sim_core::ids::{CardId, PlayerId};
use sim_core::state::Game;

fn out(text: impl Into<String>, detail: Vec<String>, choice: Value) -> Value {
    json!({"text": text.into(), "detail": detail, "choice": choice})
}

fn tip(text: &str, detail: &[&str]) -> Value {
    out(text, detail.iter().map(|s| s.to_string()).collect(), Value::Null)
}

fn lands_in_hand(g: &Game, p: PlayerId) -> usize {
    g.player(p).hand.iter().filter(|&&c| g.db.get(c).land).count()
}

/// discard: the AI bins a land when it holds three or more, otherwise its dearest spare spell
fn discard(g: &Game, p: PlayerId, choices: &[String]) -> Option<Value> {
    let hand = g.player(p).hand.clone();
    let lands: Vec<CardId> = hand.iter().copied().filter(|&c| g.db.get(c).land).collect();
    let nonl: Vec<CardId> = hand.iter().copied().filter(|&c| !g.db.get(c).land).collect();
    let keep = sim_core::ai::discard_keep(g, p);
    let spare: Vec<CardId> = nonl.iter().copied().filter(|c| !keep.contains(c)).collect();
    let dearest = |v: &[CardId]| v.iter().copied().max_by_key(|&c| g.db.get(c).cmc);
    let pick = if !keep.is_empty() && spare.len() + lands.len() > 0 {
        if lands.len() > 2 || spare.is_empty() {
            lands.first().copied().or_else(|| nonl.iter().copied().min_by_key(|&c| g.db.get(c).cmc))
        } else {
            dearest(&spare)
        }
    } else if lands.len() > 2 || nonl.is_empty() {
        lands.first().copied()
    } else {
        dearest(&nonl)
    }?;
    let lab = card_label(g, pick);
    let idx = choices.iter().position(|c| *c == lab)?;
    let why = if g.db.get(pick).land {
        format!("you hold {} lands, more than the 2 it keeps", lands.len())
    } else {
        format!("it is the most expensive card it can spare (cost {})", g.db.get(pick).cmc)
    };
    Some(out(
        format!("Discard {lab}."),
        vec![format!("The AI discards a land when it holds three or more, otherwise its dearest spare spell: {why}.")],
        json!(idx),
    ))
}

fn scry(g: &Game, p: PlayerId, prompt: &str, choices: &[String]) -> Option<Value> {
    // "Scry: Sheoldred, the Apocalypse {2}{B}{B}": the card's name is before the cost
    let label = prompt.split_once(": ")?.1;
    let name = label.split(" {").next()?.trim();
    let c = g.db.by_name(name)?.id;
    let want = topdeck::desire(g, p, c);
    let keep = want >= topdeck::KEEP;
    let away = if prompt.starts_with("Surveil") { "your graveyard" } else { "the bottom" };
    Some(out(
        if keep { format!("Keep {name} on top.") } else { format!("Put {name} on {away}.") },
        vec![format!(
            "The AI rates {name} {want:.0} as a draw right now and keeps cards rated {:.0} or more on top (lands rate high while you hold few).",
            topdeck::KEEP
        )],
        json!(if keep || choices.len() < 2 { 0 } else { 1 }),
    ))
}

fn sacrifice(g: &Game, p: PlayerId, choices: &[String]) -> Option<Value> {
    let m = brain::spare_creature(g, p)?;
    let lab = legal::describe_target(g, p, Target::Perm(m));
    let idx = choices.iter().position(|c| *c == lab)?;
    Some(out(format!("Sacrifice {lab}."), vec!["The AI sacrifices the creature it can best spare.".into()], json!(idx)))
}

pub fn hint(g: &mut Game, p: PlayerId, req: &Request) -> Value {
    let prompt = req.prompt.clone();
    let low = prompt.to_lowercase();
    let choices = &req.choices;
    let kind = &*req.kind;
    if kind == "mulligan" {
        return super::mulligan::hint(g, p, &prompt);
    }
    if low.starts_with("put a card on the bottom") {
        return super::mulligan::bottom(g, p, choices);
    }
    if low.contains("discard")
        && !choices.is_empty()
        && let Some(h) = discard(g, p, choices)
    {
        return h;
    }
    if (low.starts_with("scry") || low.starts_with("surveil"))
        && let Some(h) = scry(g, p, &prompt, choices)
    {
        return h;
    }
    if low.contains("sacrifice")
        && let Some(h) = sacrifice(g, p, choices)
    {
        return h;
    }
    let life = g.player(p).life;
    match kind {
        "continue" => tip("Nothing to decide: continue to the next turn.", &["(this is the step-by-step pause)"]),
        "block" => tip(
            "Block when your blocker survives or trades for something worth more; chump block only to stay out of lethal range.",
            &[
                &format!(
                    "You are at {life} life. Compare the attacker's damage with your life: a block that saves a big hit is worth losing a small creature."
                ),
                "'no block' is right when your creature would die for nothing and you can afford the damage.",
            ],
        ),
        "target" => tip(
            "Aim at the biggest threat, or at the opponent who is closest to winning or to dying.",
            &[
                "Kill or take the permanent that does the most for its owner; send damage at players only when nothing on the table is worth it.",
                "Mind hexproof, protection and what the target could do in response.",
            ],
        ),
        "search" => {
            let lands = g.player(p).lands.len() + lands_in_hand(g, p);
            if lands < 5 {
                tip(
                    "Take what you are short of: a land (or ramp) first.",
                    &[&format!("You have {lands} lands in play and hand; get to about 5-6 before fetching threats.")],
                )
            } else {
                tip(
                    "You have enough lands: take the card your hand and board lack most (a threat, removal or card advantage).",
                    &[
                        &format!("You have {lands} lands in play and hand."),
                        "Searching shuffles your library, so take the best card, not just any.",
                    ],
                )
            }
        }
        "choose" => {
            if choices.len() == 2 && choices[0] == "yes" && choices[1] == "no" {
                tip(
                    "Say yes if the effect helps you and its cost is small; read the prompt for what you give up.",
                    &[&prompt],
                )
            } else if low.contains("mode") || low.contains("choose one") || low.contains("which?") {
                tip(
                    "Pick the mode that answers the biggest problem on the table right now, or builds your plan if there is none.",
                    &[&prompt, "Removal beats card draw when an opponent's threat is about to win."],
                )
            } else if low.contains("x =") {
                tip(
                    "Spend the mana you can spare on X, but keep what you need for your next spell or an answer.",
                    &[&prompt],
                )
            } else if low.contains("tap which") || low.contains("pay") {
                tip(
                    "Pay with the permanents you will not need this turn, and keep your best blockers and mana untapped.",
                    &[&prompt],
                )
            } else if low.contains("creature type") {
                tip(
                    "Name the creature type that is most common among your creatures (or the opponents', whichever the effect rewards).",
                    &[&prompt],
                )
            } else if low.contains("cast") {
                tip("Cast the spell that is worth the most now; passing it up may lose it.", &[&prompt])
            } else {
                tip(
                    "Pick the option that gives you the most for the least; read the prompt for what each one does.",
                    &[&prompt],
                )
            }
        }
        _ => tip(
            &format!("This is a '{kind}' decision: weigh what each option costs you against what it gains."),
            &[&prompt],
        ),
    }
}
