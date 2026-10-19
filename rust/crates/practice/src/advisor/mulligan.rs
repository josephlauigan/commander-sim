//! The mulligan hint: the AI's own keep rule (`engine::turn::keeps_hand`) applied to your opening hand, with the
//! reasons in plain words; and, once you keep after mulligans, the card the AI would put on the bottom.

use serde_json::{Value, json};
use sim_core::cards::Colors;
use sim_core::engine::turn::{bottom_pick, keeps_hand};
use sim_core::human::choices::card_label;
use sim_core::ids::{CardId, PlayerId};
use sim_core::state::Game;
use sim_core::tag::Tag;

const COLOURS: [(char, &str); 5] = [('W', "white"), ('U', "blue"), ('B', "black"), ('R', "red"), ('G', "green")];

fn names(g: &Game, v: &[CardId]) -> String {
    let mut out: Vec<String> = v.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    out.sort();
    out.join(", ")
}

fn colour_words(c: Colors) -> String {
    let w: Vec<&str> = COLOURS.iter().filter(|x| c.has(x.0)).map(|x| x.1).collect();
    if w.is_empty() { "no colour".into() } else { w.join(" and ") }
}

/// the colours a land can make (its colour identity and basic land types; a colourless land makes none)
fn land_colours(g: &Game, c: CardId) -> Colors {
    let d = g.db.get(c);
    let mut s = String::new();
    for (ch, _) in COLOURS {
        if d.identity.is_some_and(|i| i.has(ch)) || d.land_types.has(ch) {
            s.push(ch);
        }
    }
    Colors::from_letters(&s)
}

/// how many cards the AI would keep: the hand size in the prompt "Your opening hand (7 to keep)."
fn cards_to_keep(prompt: &str) -> usize {
    prompt.split('(').nth(1).and_then(|s| s.split(' ').next()).and_then(|s| s.parse().ok()).unwrap_or(7)
}

pub fn hint(g: &mut Game, p: PlayerId, prompt: &str) -> Value {
    let hand = g.player(p).hand.clone();
    let keep_n = cards_to_keep(prompt);
    let bottom = 7usize.saturating_sub(keep_n);
    let lands: Vec<CardId> = hand.iter().copied().filter(|&c| g.db.get(c).land).collect();
    let spells: Vec<CardId> = hand.iter().copied().filter(|&c| !g.db.get(c).land).collect();
    let rocks: Vec<CardId> =
        spells.iter().copied().filter(|&c| g.db.get(c).tag(Tag::Rock) || g.db.get(c).tag(Tag::Dork)).collect();
    let keeps = keeps_hand(g, p);
    let forced = bottom >= 2; // the AI keeps its fourth hand whatever it holds
    let key = g.player(p).key.to_string();
    let n = lands.len();

    let mut d: Vec<String> = vec![];
    if key == "cloud" {
        d.push(format!(
            "The AI's rule for this deck: keep 3 to 5 lands, or 2 lands plus a mana rock, a mana creature or a creature that costs 2 or less. This hand has {n} land(s){}.",
            if n == 2 {
                if keeps { " and one of those cheap plays" } else { " and none of those cheap plays" }
            } else {
                ""
            }
        ));
    } else {
        d.push(format!(
            "The AI's rule: keep a hand of 7 with 2 to 5 lands, mulligan 0-1 landers and 6-7 landers. This hand has {n} land(s): {}.",
            if keeps { "inside that range" } else if n < 2 { "too few" } else { "too many" }
        ));
    }
    if n > 0 {
        d.push(format!("Lands: {}.", names(g, &lands)));
    }
    if rocks.is_empty() {
        d.push("Mana help: no mana rock or mana creature in this hand (the AI's rule doesn't count them, except for Cloud).".into());
    } else {
        d.push(format!(
            "Mana help: {} (the AI's rule doesn't count these; they make a 2-3 lander safer).",
            names(g, &rocks)
        ));
    }
    // colours
    let mut have = Colors::NONE;
    let mut s = String::new();
    for &c in &lands {
        for (ch, _) in COLOURS {
            if land_colours(g, c).has(ch) {
                s.push(ch);
            }
        }
    }
    if !s.is_empty() {
        have = Colors::from_letters(&s);
    }
    let ident = g.player(p).ident;
    let missing: String =
        COLOURS.iter().filter(|x| ident.has(x.0) && !have.has(x.0)).map(|x| x.1).collect::<Vec<_>>().join(" and ");
    d.push(if n == 0 {
        "Colours: no lands, so no colours at all.".to_string()
    } else if missing.is_empty() {
        format!("Colours: the lands make {}; your commander's colours are covered.", colour_words(have))
    } else {
        format!(
            "Colours: the lands make {}; you have no {missing} source yet (the AI's rule doesn't check colours).",
            colour_words(have)
        )
    });
    // what it casts on turns 1-3
    let mut early: Vec<String> = vec![];
    for &c in &spells {
        let cd = g.db.get(c);
        if cd.cmc as usize <= n.min(3) + rocks.len().min(1) && cd.cmc > 0 {
            let ok = cd.pips.chars().all(|ch| have.has(ch) || !"WUBRG".contains(ch));
            if ok {
                early.push(format!("{} (cost {})", cd.name, cd.cmc));
            }
        }
    }
    d.push(if early.is_empty() {
        "Turns 1-3: nothing in this hand is cheap enough and in your colours to cast early.".to_string()
    } else {
        format!("Turns 1-3 you could cast: {}.", early.join(", "))
    });
    // commander
    let pl = g.player(p);
    let cmd = pl.cmd;
    let cd = g.db.get(cmd);
    let sources = n + rocks.len();
    let when = if cd.cmc as usize <= sources {
        format!("on turn {} if you hit every land drop", (cd.cmc as usize).max(1))
    } else {
        format!("only after drawing {} more mana source(s)", cd.cmc as usize - sources)
    };
    d.push(format!("Commander {} costs {}: it comes down {when}.", cd.name, cd.cmc));
    d.push("The AI's rule does not look at key cards (tutors, ramp, win conditions); neither does this hint.".into());

    let text;
    if keeps {
        text = format!("Keep ({n} lands, {} spells).", spells.len());
    } else if forced {
        text = "Keep: this is a small hand, and the AI keeps its fourth hand whatever it holds.".to_string();
    } else {
        text = format!("Mulligan ({n} land{} is outside the AI's keep range).", if n == 1 { "" } else { "s" });
    }
    if bottom > 0 {
        let pick = bottom_pick(g, p);
        let bottom_note = match pick {
            Some(c) => format!(
                "If you keep, you put {bottom} card(s) on the bottom; the AI would start with {}: it bottoms a land when it holds more than four, otherwise its most expensive spell.",
                card_label(g, c)
            ),
            None => format!("If you keep, you put {bottom} card(s) on the bottom."),
        };
        d.push(bottom_note);
    }
    let keep_it = keeps || forced;
    json!({"text": text, "detail": d, "choice": if keep_it { "keep" } else { "mulligan" }})
}

/// the bottom-of-library choice after a mulligan (labels are `card_label`s of the hand)
pub fn bottom(g: &mut Game, p: PlayerId, choices: &[String]) -> Value {
    let Some(c) = bottom_pick(g, p) else {
        return json!({"text": "Bottom any card: your hand is empty.", "detail": [], "choice": null});
    };
    let lab = card_label(g, c);
    let idx = choices.iter().position(|x| *x == lab);
    let d = g.db.get(c);
    let lands = g.player(p).hand.iter().filter(|&&x| g.db.get(x).land).count();
    let why = if d.land {
        format!("you hold {lands} lands, more than the 4 the AI wants to keep")
    } else {
        format!("it is the most expensive spell in the hand (cost {}), the hardest to cast on a short hand", d.cmc)
    };
    json!({"text": format!("Put {lab} on the bottom."),
           "detail": [format!("The AI bottoms a land when it holds more than four, otherwise its most expensive spell: {why}.")],
           "choice": idx})
}

/// whether the AI would keep the hand on the table at this request, and a short reason (for the end-of-game review)
pub fn verdict(g: &Game, p: PlayerId, prompt: &str) -> (bool, String) {
    let n = g.player(p).hand.iter().filter(|&&c| g.db.get(c).land).count();
    let bottom = 7usize.saturating_sub(cards_to_keep(prompt));
    let keeps = keeps_hand(g, p);
    if keeps {
        (true, format!("Keep ({n} lands is inside the AI's range)"))
    } else if bottom >= 2 {
        (true, format!("Keep (the AI keeps its fourth hand whatever it holds; this one has {n} lands)"))
    } else if g.player(p).key == "cloud" {
        (
            false,
            format!(
                "Mulligan ({n} lands and no cheap play; Cloud's AI keeps 3-5 lands, or 2 plus a rock, mana creature or cheap creature)"
            ),
        )
    } else {
        (false, format!("Mulligan ({n} lands; the AI keeps 2 to 5)"))
    }
}
