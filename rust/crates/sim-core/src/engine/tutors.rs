//! Tutors and the Game Changer mechanics (engine.py: tutor, atraxa_reveal ... ad_nauseam).

use crate::ai;
use crate::engine::life::lose_life;
use crate::engine::mana::total_mana;
use crate::engine::values::{commander_out, threat};
use crate::engine::zones::*;
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

/// p searches for a card of this kind and puts it in hand ('any', 'cre2', 'art' ...): the deck's AI picks
pub fn tutor(g: &mut Game, p: PlayerId, kind: &str) -> Res {
    // HUMAN(phase 9): hc.search
    let Some(name) = ai::tutor_pick(g, p, kind) else { return Ok(()) };
    let Some(c) = searchable(g, p).into_iter().find(|&c| c == name) else { return Ok(()) };
    let lib = &mut g.player_mut(p).library;
    let i = lib.iter().position(|&x| x == c).unwrap();
    lib.remove(i);
    if let Some(a) = agent_for(g, p) {
        agent_take(g, a, p, c);
        let mut lib = std::mem::take(&mut g.player_mut(p).library);
        g.rng.shuffle(&mut lib);
        g.player_mut(p).library = lib;
        return Ok(());
    }
    let pl = g.player_mut(p);
    pl.hand.push(c);
    pl.stat("tutored", 1);
    pl.seen.insert(c);
    crate::glog!(g, "    {} tutors {}", g.player(p).name, g.db.get(c).name); // PORT(phase 9): log_secret
    shuffle_library(g, p);
    Ok(())
}

pub fn shuffle_library(g: &mut Game, p: PlayerId) {
    let mut lib = std::mem::take(&mut g.player_mut(p).library);
    g.rng.shuffle(&mut lib);
    g.player_mut(p).library = lib;
}

/// Atraxa, Grand Unifier: reveal the top ten; for each card type put one card of that type into your hand (a
/// multi-type card fills one type); the rest go to the bottom in a random order
pub fn atraxa_reveal(g: &mut Game, p: PlayerId) -> Res {
    let n = g.player(p).library.len().min(10);
    let top: Vec<CardId> = (0..n).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
    let mut free: Vec<char> = "ABCEILPS".chars().collect(); // artifact, battle, creature ... sorcery
    let mut taken = vec![];
    // HUMAN(phase 9): hc.atraxa_pick
    let mut order = top.clone();
    order.sort_by(|&a, &b| card_worth(g, p, b, false).total_cmp(&card_worth(g, p, a, false)));
    for c in order {
        let codes = type_codes(g, c);
        let ts: Vec<char> = codes.into_iter().filter(|t| free.contains(t)).collect();
        if ts.is_empty() {
            continue;
        }
        // the scarcer type slot
        let scarce =
            *ts.iter().min_by_key(|&&t| top.iter().filter(|&&x| type_codes(g, x).contains(&t)).count()).unwrap();
        free.retain(|&t| t != scarce);
        taken.push(c);
    }
    let mut rest: Vec<CardId> = top.iter().copied().filter(|c| !taken.contains(c)).collect();
    g.rng.shuffle(&mut rest);
    let pl = g.player_mut(p);
    pl.library.splice(0..0, rest);
    pl.hand.extend(&taken);
    crate::glog!(
        g,
        "    Atraxa reveals ten: {} to hand",
        taken.iter().map(|&c| g.db.get(c).name.to_string()).collect::<Vec<_>>().join(", ")
    );
    Ok(())
}

/// a card's one-letter type codes, in the order Python's `types` string has them
fn type_codes(g: &Game, c: CardId) -> Vec<char> {
    let d = g.db.get(c);
    let mut v = vec![];
    for (t, ch) in [
        (crate::cards::Types::LAND, 'L'),
        (crate::cards::Types::CREATURE, 'C'),
        (crate::cards::Types::INSTANT, 'I'),
        (crate::cards::Types::SORCERY, 'S'),
        (crate::cards::Types::ARTIFACT, 'A'),
        (crate::cards::Types::ENCHANTMENT, 'E'),
        (crate::cards::Types::PLANESWALKER, 'P'),
    ] {
        if d.types.has(t) {
            v.push(ch);
        }
    }
    v
}

/// how much p's AI wants card c in hand (or, in_gy, in the graveyard: flashback cards keep most of their value)
pub fn card_worth(g: &Game, p: PlayerId, c: CardId, in_gy: bool) -> f64 {
    let mut v = ai::deck_prio(g, p, c);
    let d = g.db.get(c);
    if v <= 0.0 && d.dsl.is_some() {
        v = crate::dsl::card_value(g, p, c) * 10.0;
    }
    if v <= 0.0 {
        // held interaction has no cast priority but is worth keeping
        let t = &d.tags;
        v = if t.str(Tag::Wipe) == Some("rift") {
            60.0
        } else if t.has(Tag::Ctr) {
            55.0
        } else if t.has(Tag::Mastery) || t.has(Tag::Crackle) {
            50.0
        } else if t.has(Tag::Rem) || t.has(Tag::Wipe) {
            45.0
        } else if t.has(Tag::Prot) || t.has(Tag::X) {
            40.0
        } else {
            0.0
        };
    }
    if d.land {
        let pl = g.player(p);
        let lands = pl.lands.len() + pl.hand.iter().filter(|&&x| g.db.get(x).land).count();
        v = if lands < 7 { 25.0 } else { 5.0 };
    }
    if in_gy {
        return if d.tag(Tag::Fb) || d.tag(Tag::Fbnib) { v * 0.8 } else { 0.0 };
    }
    v
}

/// Search for n cards with different names; an opponent chooses which `keep` of them go to your hand, the rest go to
/// your graveyard (Gifts Ungiven, Intuition). You pick the pile that is best against their choice.
pub fn pile_tutor(g: &mut Game, p: PlayerId, n: usize, keep: usize) -> Res {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let opp = max_by(&opps, |q| threat(g, p, q));
    let mut cs: Vec<CardId> = vec![];
    for &c in &g.player(p).library {
        if !cs.contains(&c) {
            cs.push(c);
        }
    }
    if cs.is_empty() {
        return Ok(());
    }
    let mut hv: Vec<(CardId, f64)> = cs.iter().map(|&c| (c, ai::tutor_value(g, p, c))).collect();
    let gv: Vec<(CardId, f64)> =
        cs.iter().map(|&c| (c, card_worth(g, p, c, true).max(15.0 * ai::gy_worth(g, p, c)))).collect();
    if let Some(want) = ai::tutor_pick(g, p, "any")
        && let Some(x) = hv.iter_mut().find(|x| x.0 == want)
    {
        x.1 += 50.0; // the deck's tutor target goes in the pile
    }
    let val = |v: &[(CardId, f64)], c: CardId| v.iter().find(|x| x.0 == c).unwrap().1;
    let mut by_h = cs.clone();
    by_h.sort_by(|&a, &b| val(&hv, b).total_cmp(&val(&hv, a)));
    let mut by_g = cs.clone();
    by_g.sort_by(|&a, &b| val(&gv, b).total_cmp(&val(&gv, a)));
    let mut pool: Vec<CardId> = vec![];
    for c in by_h.into_iter().take(8).chain(by_g.into_iter().take(3)) {
        if !pool.contains(&c) {
            pool.push(c);
        }
    }
    let k = n.min(pool.len());
    // the opponent's choice: the split that leaves you the least value
    let result = |pile: &[CardId]| -> (f64, Vec<CardId>) {
        let mut best: Option<(f64, Vec<CardId>)> = None;
        for hand in combinations(pile, keep.min(pile.len())) {
            let v: f64 = hand.iter().map(|&c| val(&hv, c)).sum::<f64>()
                + pile.iter().filter(|c| !hand.contains(c)).map(|&c| val(&gv, c)).sum::<f64>();
            if best.as_ref().is_none_or(|b| v < b.0) {
                best = Some((v, hand));
            }
        }
        best.unwrap()
    };
    // HUMAN(phase 9): the person picks the pile (hc.search); the opponent still splits it
    let mut best: Option<(f64, Vec<CardId>, Vec<CardId>)> = None;
    for pl in combinations(&pool, k) {
        let r = result(&pl);
        if best.as_ref().is_none_or(|b| r.0 > b.0) {
            best = Some((r.0, pl, r.1));
        }
    }
    let (_, pile, hand) = best.unwrap();
    for &c in &pile {
        let lib = &mut g.player_mut(p).library;
        let i = lib.iter().position(|&x| x == c).unwrap();
        lib.remove(i);
    }
    if let Some(a) = agent_for(g, p) {
        for &c in &pile {
            agent_take(g, a, p, c);
        }
        shuffle_library(g, p);
        return Ok(());
    }
    for &c in &pile {
        let pl = g.player_mut(p);
        if hand.contains(&c) {
            pl.hand.push(c);
            pl.seen.insert(c);
        } else {
            pl.gy.push(c);
        }
    }
    g.player_mut(p).stat("tutored", 1);
    shuffle_library(g, p);
    let names = |v: &[CardId]| v.iter().map(|&c| g.db.get(c).name.to_string()).collect::<Vec<_>>().join(", ");
    let binned: Vec<CardId> = pile.iter().copied().filter(|c| !hand.contains(c)).collect();
    crate::glog!(
        g,
        "    {} gets {}{}",
        g.player(p).name,
        names(&hand),
        opp.map_or(String::new(), |o| format!("; {} bins {}", g.player(o).name, names(&binned)))
    );
    Ok(())
}

/// every k-element subset of xs, in Python's itertools.combinations order
pub fn combinations<T: Copy>(xs: &[T], k: usize) -> Vec<Vec<T>> {
    let n = xs.len();
    if k > n {
        return vec![];
    }
    let mut out = vec![];
    let mut idx: Vec<usize> = (0..k).collect();
    loop {
        out.push(idx.iter().map(|&i| xs[i]).collect());
        let Some(i) = (0..k).rev().find(|&i| idx[i] != i + n - k) else { break };
        idx[i] += 1;
        for j in i + 1..k {
            idx[j] = idx[j - 1] + 1;
        }
    }
    out
}

/// Jeska's Will: add {R} for each card in target opponent's hand, and/or exile the top three cards of your library,
/// playable this turn (both if you control your commander)
pub fn jeskas_will(g: &mut Game, p: PlayerId) -> Res {
    // HUMAN(phase 9): play/cards.jeskas_will
    let most = g.opps(p).map(|q| g.player(q).hand.len()).max().unwrap_or(0) as u32;
    let both = commander_out(g, p);
    let need: i64 =
        g.player(p).hand.iter().filter(|&&x| !g.db.get(x).land).map(|&x| g.db.get(x).cmc as i64).sum::<i64>()
            - total_mana(g, p, false) as i64;
    let mana = both || g.jeska_mana || (most >= 4 && need >= 3); // (the Breach line: always mana)
    if mana {
        g.player_mut(p).floating.r += most;
        crate::glog!(g, "    Jeska's Will adds {} red mana", most);
    }
    if both || !mana {
        let n = g.player(p).library.len().min(3);
        let top: Vec<CardId> = (0..n).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
        let pl = g.player_mut(p);
        for &c in &top {
            pl.hand.push(c);
            pl.seen.insert(c);
        }
        pl.impulse.extend(&top);
        crate::glog!(
            g,
            "    Jeska's Will exiles {} (playable this turn)",
            top.iter().map(|&c| g.db.get(c).name.to_string()).collect::<Vec<_>>().join(", ")
        );
        if let Some(&l) = top.iter().find(|&&c| g.db.get(c).land)
            && g.player(p).land_turn != g.player(p).turns as i32
        {
            let pl = g.player_mut(p);
            pl.hand.retain(|&c| c != l);
            pl.impulse.retain(|&c| c != l);
            pl.land_turn = pl.turns as i32;
            let tapped = ai::land_enters_tapped(g, p, l);
            g.add_land(p, l, tapped);
            landfall(g, p)?;
        }
    }
    Ok(())
}

/// Chrome Mox: you may exile a nonartifact, nonland card from your hand; it taps for that card's colours
pub fn chrome_imprint(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let ident = g.player(p).ident;
    let cs: Vec<CardId> = g
        .player(p)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            let d = g.db.get(c);
            !d.land && !d.types.has(crate::cards::Types::ARTIFACT) && d.colors.intersects(ident)
        })
        .collect();
    let Some(c) = min_by(&cs, |c| card_worth(g, p, c, false)) else { return Ok(()) };
    let pl = g.player_mut(p);
    let i = pl.hand.iter().position(|&x| x == c).unwrap();
    pl.hand.remove(i);
    pl.exile.push(c);
    let cols = g.db.get(c).colors;
    let cols = crate::cards::Colors::from_letters(
        &"WUBRG".chars().filter(|&ch| cols.has(ch) && ident.has(ch)).collect::<String>(),
    );
    g.perm_mut(m).colors = cols;
    crate::glog!(g, "    Chrome Mox imprints {}", g.db.get(c).name);
    Ok(())
}

/// Search for a card of this kind and put it on top of your library (Vampiric Tutor and Imperial Seal: any card,
/// 2 life; Mystical Tutor: an instant or sorcery, no life)
pub fn tutor_to_top(g: &mut Game, p: PlayerId, kind: &str, life: i32) -> Res {
    // HUMAN(phase 9): hc.search
    g.player_mut(p).to_top = true;
    let name = ai::tutor_pick(g, p, kind);
    g.player_mut(p).to_top = false;
    if life > 0 {
        lose_life(g, p, life, Some(p), "other", None)?;
    }
    let Some(name) = name else { return Ok(()) };
    let Some(c) = searchable(g, p).into_iter().find(|&c| c == name) else { return Ok(()) };
    let lib = &mut g.player_mut(p).library;
    let i = lib.iter().position(|&x| x == c).unwrap();
    lib.remove(i);
    if let Some(a) = agent_for(g, p) {
        agent_take(g, a, p, c);
        shuffle_library(g, p);
        return Ok(());
    }
    shuffle_library(g, p);
    let pl = g.player_mut(p);
    pl.library.push(c);
    pl.stat("tutored", 1);
    crate::glog!(g, "    {} puts {} on top of their library", g.player(p).name, g.db.get(c).name);
    Ok(())
}

/// Ad Nauseam: reveal the top card, put it in hand, lose life equal to its mana value; repeat while it's safe: never
/// below what the table could hit you for, with room for a 7-drop, and stop once a combo is ready
pub fn ad_nauseam(g: &mut Game, p: PlayerId, floor: Option<i32>) -> Res {
    let floor = floor.unwrap_or_else(|| ai::necro_floor(g, p));
    let mut n = 0;
    while !g.player(p).library.is_empty() && g.player(p).alive && g.player(p).life - 7 > floor && n < 15 {
        let pl = g.player_mut(p);
        let c = pl.library.pop().unwrap();
        pl.hand.push(c);
        pl.seen.insert(c);
        n += 1;
        let mv = g.db.get(c).cmc as i32;
        lose_life(g, p, mv, Some(p), "other", None)?;
        if crate::cardcode::combo_ready(g, p) {
            break; // enough: the combo is in hand
        }
    }
    g.player_mut(p).stat("adnaus_cards", n);
    crate::glog!(g, "    Ad Nauseam: {} cards, life now {}", n, g.player(p).life);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::combinations;

    #[test]
    fn combinations_in_python_order() {
        assert_eq!(
            combinations(&[1, 2, 3, 4], 2),
            vec![vec![1, 2], vec![1, 3], vec![1, 4], vec![2, 3], vec![2, 4], vec![3, 4]]
        );
        assert_eq!(combinations(&[1, 2], 3), Vec::<Vec<i32>>::new());
        assert_eq!(combinations(&[7], 0), vec![Vec::<i32>::new()]);
    }
}
