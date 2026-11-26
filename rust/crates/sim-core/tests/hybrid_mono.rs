//! Monocoloured hybrid symbols ({2/W}: Karn, Gilded Guardian, The Reaper, King No More, Spectral Procession) and a card that
//! "is all colors" (Transguild Courier). A {2/X} is paid with its colour or with {2}; the {2} is chosen before any reduction
//! or tax (CR 601.2b, 601.2f), so a generic reducer applies to it and the commander tax is summed with it before the floor. Oracle text is the
//! expectation throughout.

mod play_common;
use play_common::*;
use sim_core::engine::chars;
use sim_core::engine::mana::{can_pay_spell, cost_of};
use sim_core::testkit::{set_commander, table};

const KARN: &str = "Karn, Gilded Guardian";

fn tbl() -> Game {
    let mut g = table(&["sheoldred", "seph"]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    g
}

fn sorted(s: &str) -> String {
    let mut v: Vec<char> = s.chars().collect();
    v.sort();
    v.into_iter().collect()
}

/// `n` untapped lands of each basic type given (letters W U B R G)
fn sources(g: &mut Game, spec: &[(char, usize)]) {
    for &(col, n) in spec {
        let name = match col {
            'W' => "Plains",
            'U' => "Island",
            'B' => "Swamp",
            'R' => "Mountain",
            _ => "Forest",
        };
        lands(g, P0, name, n, false);
    }
}

fn karn_in_hand(g: &mut Game) -> sim_core::ids::CardId {
    hand(g, P0, &[KARN])[0]
}

fn price(g: &Game, c: sim_core::ids::CardId) -> (u32, String) {
    let (gn, pips) = cost_of(g, P0, c);
    (gn, sorted(&pips))
}

// ======================================================== the card data
#[test]
fn karn_record() {
    let g = tbl();
    let d = g.db.get(card(&g, KARN));
    assert_eq!(d.cmc, 10, "five {{2/X}} symbols: mana value 2 each");
    for ch in ['W', 'U', 'B', 'R', 'G'] {
        assert!(d.pips.contains(ch), "{ch} is a pip");
    }
    assert_eq!(&*d.hybrid, "W2U2B2R2G2");
    assert_eq!(d.colors.count(), 5, "it is every colour");
    assert_eq!(d.devotion_to("W"), 1, "a {{2/W}} counts once toward devotion to white (CR 700.5)");
    assert_eq!(d.devotion_to("WU"), 2);

    let reaper = g.db.get(card(&g, "The Reaper, King No More"));
    assert_eq!(reaper.cmc, 6);
    assert_eq!((reaper.colors.has('B'), reaper.colors.has('R'), reaper.colors.has('G')), (true, true, true));
    assert_eq!(reaper.colors.count(), 3);

    let sp = g.db.get(card(&g, "Spectral Procession"));
    assert_eq!(sp.cmc, 6);
    assert!(sp.colors.has('W') && sp.colors.count() == 1);
}

#[test]
fn no_other_card_gained_a_two_hybrid() {
    let g = tbl();
    let n = (0..g.db.len() as u16).filter(|&i| g.db.get(sim_core::ids::CardId(i)).hybrid.contains('2')).count();
    assert_eq!(n, 3, "Karn, The Reaper, Spectral Procession (Wildgrowth Archaic's override states its own cost)");
}

// ======================================================== the cost
#[test]
fn karn_costs_five_with_all_colours() {
    let mut g = tbl();
    sources(&mut g, &[('W', 1), ('U', 1), ('B', 1), ('R', 1), ('G', 1)]);
    let c = karn_in_hand(&mut g);
    let (gn, pips) = cost_of(&g, P0, c);
    assert_eq!((gn, sorted(&pips)), (0, sorted("WUBRG")));
    assert!(can_pay_spell(&g, P0, c, gn, &pips, false), "castable with five lands");
    assert_eq!(sim_core::engine::hybrid::priced(&g, c).as_deref(), Some("BGRUW"), "the colours it was priced at");
}

#[test]
fn karn_costs_seven_with_wub_only() {
    let mut g = tbl();
    sources(&mut g, &[('W', 3), ('U', 2), ('B', 2)]);
    let c = karn_in_hand(&mut g);
    let (gn, pips) = price(&g, c);
    assert_eq!((gn, pips.as_str()), (4, sorted("WUB").as_str()), "{{2}} for red and for green");
    assert!(can_pay_spell(&g, P0, c, gn, &pips, false));
    assert_eq!(sim_core::engine::hybrid::priced(&g, c).as_deref(), Some("BUW"));
}

#[test]
fn karn_uncastable_with_six_wub_sources() {
    let mut g = tbl();
    sources(&mut g, &[('W', 2), ('U', 2), ('B', 2)]);
    let c = karn_in_hand(&mut g);
    let (gn, pips) = cost_of(&g, P0, c);
    assert_eq!(
        (gn, sorted(&pips)),
        (4, sorted("WUB")),
        "the price names the {{2}}s for the colours it has no source for"
    );
    assert!(!can_pay_spell(&g, P0, c, gn, &pips, false), "seven mana of cost, six lands");
}

#[test]
fn karn_with_every_colour_but_too_few_lands_keeps_its_printed_colours() {
    let mut g = tbl();
    sources(&mut g, &[('W', 1), ('U', 1), ('B', 1), ('R', 1), ('G', 1)]);
    let c = karn_in_hand(&mut g);
    // tap one: the colour has no untapped source, so its {2/X} is priced as {2}
    let m = g.player(P0).lands[4];
    g.perm_mut(m).tapped = true;
    let (gn, pips) = price(&g, c);
    assert_eq!((gn, pips.as_str()), (2, sorted("WUBR").as_str()));
    assert!(!can_pay_spell(&g, P0, c, gn, &pips, false));
}

#[test]
fn wind_crystal_reduces_a_two() {
    let mut g = tbl();
    perm(&mut g, P0, "The Wind Crystal"); // white spells you cast cost {1} less; Karn is white too
    sources(&mut g, &[('W', 2), ('U', 2), ('B', 2)]);
    let c = karn_in_hand(&mut g);
    let (gn, pips) = price(&g, c);
    assert_eq!((gn, pips.as_str()), (3, sorted("WUB").as_str()), "the {{2}}s chosen for R and G are reduced by one");
    assert!(can_pay_spell(&g, P0, c, gn, &pips, false), "six lands pay it");

    let mut g = tbl();
    perm(&mut g, P0, "The Wind Crystal");
    sources(&mut g, &[('W', 1), ('U', 1), ('B', 1), ('R', 1), ('G', 1)]);
    let c = karn_in_hand(&mut g);
    let (gn, pips) = price(&g, c);
    assert_eq!((gn, pips.as_str()), (0, sorted("WUBRG").as_str()), "all five colours: nothing to reduce, unchanged");
}

#[test]
fn tax_adds_two_generic() {
    let mut g = tbl();
    set_commander(&mut g, P0, KARN);
    let c = card(&g, KARN);
    sources(&mut g, &[('W', 1), ('U', 1), ('B', 1), ('R', 1), ('G', 1)]);
    assert_eq!(price(&g, c), (0, sorted("WUBRG")), "no earlier cast");
    g.player_mut(P0).add_tax(c); // one earlier cast from the command zone
    assert_eq!(g.player(P0).tax_of(c), 2);
    assert_eq!(price(&g, c), (2, sorted("WUBRG")), "+{{2}}, after the {{2/X}} choice");
}

#[test]
fn spectral_procession() {
    let mut g = tbl();
    sources(&mut g, &[('W', 3)]);
    let c = hand(&mut g, P0, &["Spectral Procession"])[0];
    assert_eq!(price(&g, c), (0, "WWW".to_string()));
    assert!(can_pay_spell(&g, P0, c, 0, "WWW", false));

    let mut g = tbl();
    let c = hand(&mut g, P0, &["Spectral Procession"])[0];
    assert_eq!(price(&g, c), (6, String::new()), "no white source: {{2}} for each");

    let mut g = tbl();
    sources(&mut g, &[('W', 1), ('B', 5)]);
    let c = hand(&mut g, P0, &["Spectral Procession"])[0];
    assert_eq!(price(&g, c), (4, "W".to_string()), "one white, and {{2}}{{2}} paid with the swamps");
    assert!(can_pay_spell(&g, P0, c, 4, "W", false));
}

#[test]
fn a_two_colour_hybrid_is_not_touched() {
    // Kitchen Finks ({1}{G/W}{G/W}) still pays the other colour as before
    let mut g = tbl();
    sources(&mut g, &[('W', 3)]);
    let c = hand(&mut g, P0, &["Kitchen Finks"])[0];
    assert_eq!(price(&g, c), (1, "WW".to_string()));
}

// ======================================================== "is all colors" (Transguild Courier)
#[test]
fn courier_is_all_colours() {
    let mut g = tbl();
    let id = card(&g, "Transguild Courier");
    assert_eq!(g.db.get(id).colors.count(), 5, "in the card database (a characteristic-defining ability: every zone)");
    for ch in ['W', 'U', 'B', 'R', 'G'] {
        assert!(g.db.get(id).colors.has(ch));
    }
    assert_eq!(&*g.db.get(id).pips, "", "its cost is still {{4}}");
    // in the library
    let c = take(&mut g, P0, "Transguild Courier");
    g.player_mut(P0).library.push(c);
    assert_eq!(g.db.get(c).colors.count(), 5);
    // on the battlefield, through the characteristics pass
    let m = perm(&mut g, P0, "Transguild Courier");
    let cols = chars::colors(&g, m);
    assert_eq!(cols.count(), 5);
    assert!(cols.has('W') && cols.has('U') && cols.has('B') && cols.has('R') && cols.has('G'));
    // and it is every colour for the spell-colour readers too
    assert!(sim_core::engine::stack::blue_card(&g, c));
}

#[test]
fn another_colourless_golem_stays_colourless() {
    let g = tbl();
    assert_eq!(g.db.get(card(&g, "Sol Ring")).colors.count(), 0);
}

#[test]
fn cursed_mirror_copy_of_courier_is_five_colours() {
    use sim_core::state::Ctx;
    let mut g = tbl();
    perm(&mut g, P0, "Transguild Courier");
    let mirror = hand(&mut g, P0, &["Cursed Mirror"])[0];
    sim_core::engine::cast::cast_card(&mut g, P0, mirror, "hand", Ctx::default()).unwrap();
    let copy = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).on_bf && g.perm(m).phys == Some(mirror) && &*g.perm(m).name == "Transguild Courier")
        .expect("Cursed Mirror entered as a copy of Transguild Courier");
    assert_eq!(chars::colors(&g, copy).count(), 5, "a copy has the copied card's colours");
}
