//! The Tier 4 chulane-bant-value-combo and Tier 5 kinnan-simic-mana-combo cards (impls/t45_chulane_kinnan.rs and the
//! compiled pattern under `// --- Tiers 4/5 (chulane-kinnan)`) and their new combos, on hand-built positions. Each
//! expectation follows the card's Oracle text.

use sim_core::engine::combat::can_block;
use sim_core::engine::hooks::{fire_trigger, run_hook};
use sim_core::engine::mana::{land_colors_now, mana_units, total_mana};
use sim_core::engine::removal::apply_removal;
use sim_core::engine::zones;
use sim_core::hooks::{Call, Event};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::combos;
use sim_core::state::{Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const CHULANE: &str = "chulane-bant-value-combo";
const KINNAN: &str = "kinnan-simic-mana-combo";

fn chulane() -> Game {
    table(&[CHULANE, "seph", "sauron"])
}

fn kinnan() -> Game {
    table(&[KINNAN, "seph", "sauron"])
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn untapped_lands(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.iter().filter(|&&l| !g.perm(l).tapped).count()
}

fn serpent(g: &mut Game, p: PlayerId) -> PermId {
    let spec = zones::Tokens { sick: false, types: vec!["serpent"], ..zones::Tokens::new(1, 3) };
    zones::make_tokens(g, p, spec).unwrap()[0]
}

fn cast_event(g: &mut Game, caster: PlayerId, c: CardId) {
    fire_trigger(g, Event::Cast, Call::Cast { caster, c }).unwrap();
}

fn combo_ready(g: &Game, p: PlayerId, part: &str) -> bool {
    combos::ready(g, p, combos::combo_index(part).unwrap_or_else(|| panic!("no combo {part:?}")))
}

// ------------------------------------------------------------------ Peregrine Drake (compiled)
#[test]
fn peregrine_drake_untaps_up_to_five_lands() {
    let mut g = chulane();
    lands(&mut g, P0, "Island", 7, true);
    perm(&mut g, P0, "Peregrine Drake");
    assert_eq!(untapped_lands(&g, P0), 5, "five of the seven tapped lands untap");
}

#[test]
fn frantic_search_untaps_three_lands_with_the_same_pattern() {
    let mut g = chulane();
    lands(&mut g, P0, "Island", 5, true);
    let c = take(&mut g, P0, "Frantic Search");
    sim_core::engine::cast::resolve(&mut g, P0, c, &Default::default(), "hand").unwrap();
    assert_eq!(untapped_lands(&g, P0), 3);
}

// ------------------------------------------------------------------ Intruder Alarm
#[test]
fn intruder_alarm_untaps_all_creatures_and_holds_them_in_the_untap_step() {
    let mut g = chulane();
    perm(&mut g, P0, "Intruder Alarm");
    let (a, b) = (token(&mut g, P0, 2), token(&mut g, P1, 3));
    g.perm_mut(a).tapped = true;
    g.perm_mut(b).tapped = true;
    perm(&mut g, P2, "Llanowar Elves");
    assert!(!g.perm(a).tapped && !g.perm(b).tapped, "any creature entering untaps every creature");
    g.perm_mut(a).tapped = true;
    let l = lands(&mut g, P0, "Forest", 1, true)[0];
    sim_core::engine::turn::step_start(&mut g, P0).unwrap();
    assert!(g.perm(a).tapped, "creatures don't untap during their controller's untap step");
    assert!(!g.perm(l).tapped, "lands still do");
}

// ------------------------------------------------------------------ Mana Breach
#[test]
fn mana_breach_returns_a_land_of_the_caster() {
    let mut g = chulane();
    perm(&mut g, P0, "Mana Breach");
    lands(&mut g, P1, "Island", 2, false);
    let t = lands(&mut g, P1, "Island", 1, true)[0];
    let c = card(&g, "Counterspell");
    cast_event(&mut g, P1, c);
    assert_eq!(g.player(P1).lands.len(), 2);
    assert!(!g.player(P1).lands.contains(&t), "the tapped land goes back first");
    assert!(g.player(P1).hand.contains(&g.perm(t).card()));
    let n0 = g.player(P0).lands.len();
    let c = card(&g, "Counterspell");
    cast_event(&mut g, P2, c);
    assert_eq!(g.player(P0).lands.len(), n0, "only the caster returns a land");
}

// ------------------------------------------------------------------ Cloudstone Curio
#[test]
fn cloudstone_curio_rebuys_a_creature_under_aluren() {
    let mut g = chulane();
    perm(&mut g, P0, "Aluren");
    perm(&mut g, P0, "Chulane, Teller of Tales");
    perm(&mut g, P0, "Cloudstone Curio");
    let oracle = perm(&mut g, P0, "Coiling Oracle");
    perm(&mut g, P0, "Llanowar Elves");
    assert!(!g.perm(oracle).on_bf, "a creature entered: Coiling Oracle returns to be cast again for free");
    assert!(g.player(P0).hand.contains(&card(&g, "Coiling Oracle")));
}

#[test]
fn cloudstone_curio_keeps_a_creature_it_cannot_recast() {
    let mut g = chulane();
    perm(&mut g, P0, "Cloudstone Curio");
    let oracle = perm(&mut g, P0, "Coiling Oracle");
    perm(&mut g, P0, "Llanowar Elves");
    assert!(g.perm(oracle).on_bf, "no Aluren and no mana: it stays");
}

// ------------------------------------------------------------------ Azorius Chancery (bounce land)
#[test]
fn azorius_chancery_returns_a_land_and_taps_for_two() {
    let mut g = chulane();
    let plains = lands(&mut g, P0, "Plains", 1, false)[0];
    let c = take(&mut g, P0, "Azorius Chancery");
    sim_core::engine::turn::play_land_card(&mut g, P0, c, "plays").unwrap();
    assert!(!g.player(P0).lands.contains(&plains) && g.player(P0).hand.contains(&g.perm(plains).card()));
    let l = *g.player(P0).lands.last().unwrap();
    assert!(g.perm(l).tapped, "enters tapped");
    g.perm_mut(l).tapped = false;
    let u = mana_units(&g, P0, false);
    assert!(u.iter().any(|w| w.amt == 2 && w.cols.has('W') && w.cols.has('U')), "{{T}}: add {{W}}{{U}}");
}

// ------------------------------------------------------------------ Enduring Vitality
#[test]
fn enduring_vitality_returns_as_a_noncreature_enchantment() {
    let mut g = kinnan();
    let v = perm(&mut g, P0, "Enduring Vitality");
    assert!(g.is_creature(v));
    zones::die(&mut g, v, "destroy").unwrap();
    let back = on_bf(&g, P0, "Enduring Vitality").expect("it returns");
    assert!(!g.is_creature(back), "It's an enchantment. (It's not a creature.)");
    let bear = token(&mut g, P0, 2);
    let u = mana_units(&g, P0, false);
    assert!(
        u.iter().any(|w| w.src == sim_core::engine::mana::Source::Perm(bear) && w.cols.has('U')),
        "still gives the ability"
    );
    assert!(!u.iter().any(|w| w.src == sim_core::engine::mana::Source::Perm(back)), "it taps for nothing itself");
}

// ------------------------------------------------------------------ Laboratory Maniac
#[test]
fn laboratory_maniac_wins_on_a_draw_from_an_empty_library() {
    let mut g = chulane();
    perm(&mut g, P0, "Laboratory Maniac");
    g.player_mut(P0).library.clear();
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(g.over && g.winner == Some(P0));
}

// ------------------------------------------------------------------ Barkchannel Pathway
#[test]
fn barkchannel_pathway_taps_for_the_face_chosen() {
    let mut g = kinnan();
    lands(&mut g, P0, "Forest", 3, false);
    let c = take(&mut g, P0, "Barkchannel Pathway // Tidechannel Pathway");
    sim_core::engine::turn::play_land_card(&mut g, P0, c, "plays").unwrap();
    let l = *g.player(P0).lands.last().unwrap();
    let cols = land_colors_now(&g, P0, l);
    assert!(cols.has('U') && !cols.has('G'), "with three Forests it is played as Tidechannel Pathway");
    // a fourth and fifth Island don't change a face already chosen
    lands(&mut g, P0, "Island", 5, false);
    assert!(land_colors_now(&g, P0, l).has('U'));
}

// ------------------------------------------------------------------ Koma, Cosmos Serpent
#[test]
fn koma_sacrifices_a_serpent_to_survive_destruction() {
    let mut g = kinnan();
    let koma = perm(&mut g, P0, "Koma, Cosmos Serpent");
    let coil = serpent(&mut g, P0);
    apply_removal(&mut g, Some(P1), koma, "destroy", None).unwrap();
    assert!(g.perm(koma).on_bf, "Koma gains indestructible");
    assert!(!g.perm(coil).on_bf, "the Serpent paid for it");
    apply_removal(&mut g, Some(P1), koma, "destroy", None).unwrap();
    assert!(g.perm(koma).on_bf, "indestructible until end of turn");
    let mut g = kinnan();
    let koma = perm(&mut g, P0, "Koma, Cosmos Serpent");
    apply_removal(&mut g, Some(P1), koma, "destroy", None).unwrap();
    assert!(!g.perm(koma).on_bf, "no other Serpent: it dies");
}

#[test]
fn koma_taps_the_active_opponents_biggest_attacker_keeping_a_serpent() {
    let mut g = kinnan();
    let koma = perm(&mut g, P0, "Koma, Cosmos Serpent");
    let (s1, s2) = (serpent(&mut g, P0), serpent(&mut g, P0));
    let (big, small) = (token(&mut g, P1, 6), token(&mut g, P1, 2));
    g.active = Some(P1);
    run_hook(&mut g, koma, Event::EachCombat, &Call::Player { p: P1 }).unwrap();
    assert!(g.perm(big).tapped && !g.perm(small).tapped);
    assert_eq!([s1, s2].iter().filter(|&&s| g.perm(s).on_bf).count(), 1, "one Serpent sacrificed, one kept");
}

// ------------------------------------------------------------------ Nezahal, Primal Tide
#[test]
fn nezahal_discards_three_to_dodge_removal_and_returns_at_the_end_step() {
    let mut g = kinnan();
    let n = perm(&mut g, P0, "Nezahal, Primal Tide");
    hand(&mut g, P0, &["Island", "Forest", "Ponder", "Counterspell"]);
    apply_removal(&mut g, Some(P1), n, "destroy", None).unwrap();
    assert!(!g.perm(n).on_bf);
    assert_eq!(g.player(P0).hand.len(), 1, "three cards discarded");
    assert!(g.player(P0).exile.contains(&card(&g, "Nezahal, Primal Tide")) && !g.eot_returns.is_empty());
    sim_core::impls::galadriel::eot_returns(&mut g).unwrap();
    assert!(on_bf(&g, P0, "Nezahal, Primal Tide").is_some(), "back at the beginning of the next end step");
}

#[test]
fn nezahal_without_three_cards_is_destroyed() {
    let mut g = kinnan();
    let n = perm(&mut g, P0, "Nezahal, Primal Tide");
    hand(&mut g, P0, &["Island", "Forest"]);
    apply_removal(&mut g, Some(P1), n, "destroy", None).unwrap();
    assert!(!g.perm(n).on_bf && g.player(P0).gy.contains(&card(&g, "Nezahal, Primal Tide")));
}

// ------------------------------------------------------------------ Nyxbloom Ancient
#[test]
fn nyxbloom_ancient_triples_mana_from_tapped_permanents() {
    let mut g = kinnan();
    lands(&mut g, P0, "Forest", 2, false);
    perm(&mut g, P0, "Llanowar Elves");
    assert_eq!(total_mana(&g, P0, false), 3);
    perm(&mut g, P0, "Nyxbloom Ancient");
    assert_eq!(total_mana(&g, P0, false), 9, "each land and the Elf make three");
    assert!(sim_core::engine::mana::can_pay(&g, P0, 7, "GG", false));
}

// ------------------------------------------------------------------ Spellskite
#[test]
fn spellskite_takes_removal_aimed_at_a_better_creature() {
    let mut g = kinnan();
    lands(&mut g, P0, "Island", 1, false);
    let skite = perm(&mut g, P0, "Spellskite");
    let sphinx = perm(&mut g, P0, "Consecrated Sphinx");
    let c = card(&g, "Swords to Plowshares");
    apply_removal(&mut g, Some(P1), sphinx, "exile", Some(c)).unwrap();
    assert!(g.perm(sphinx).on_bf, "the Sphinx is safe");
    assert!(!g.perm(skite).on_bf, "Spellskite was exiled instead");
    assert_eq!(untapped_lands(&g, P0), 0, "{{U/P}} paid with {{U}}");
}

// ------------------------------------------------------------------ Void Winnower
#[test]
fn void_winnower_stops_even_spells_and_even_blockers() {
    let mut g = kinnan();
    let w = perm(&mut g, P0, "Void Winnower");
    let f = g.registry.get(card(&g, "Void Winnower")).unwrap().can_cast.unwrap();
    assert!(!f(&g, w, P1, card(&g, "Counterspell"), "hand"), "mana value 2");
    assert!(f(&g, w, P1, card(&g, "Swan Song"), "hand"), "mana value 1");
    assert!(f(&g, w, P0, card(&g, "Counterspell"), "hand"), "not my own spells");
    let attacker = token(&mut g, P0, 3);
    let tok = token(&mut g, P1, 2);
    let elf = perm(&mut g, P1, "Llanowar Elves");
    assert!(!can_block(&g, tok, attacker), "a token's mana value is 0");
    assert!(can_block(&g, elf, attacker), "mana value 1 blocks");
}

// ------------------------------------------------------------------ Wandering Archaic // Explore the Vastlands
#[test]
fn wandering_archaic_copies_an_unpaid_instant_or_sorcery() {
    let mut g = kinnan();
    perm(&mut g, P0, "Wandering Archaic // Explore the Vastlands");
    let h0 = g.player(P0).hand.len();
    let c = card(&g, "Night's Whisper");
    cast_event(&mut g, P1, c);
    assert_eq!(g.player(P0).hand.len(), h0 + 2, "P1 can't pay {{2}}: the copy draws me two");
    lands(&mut g, P1, "Island", 2, false);
    let h1 = g.player(P0).hand.len();
    let c = card(&g, "Night's Whisper");
    cast_event(&mut g, P1, c);
    assert_eq!(g.player(P0).hand.len(), h1, "paid");
    assert_eq!(untapped_lands(&g, P1), 0);
}

#[test]
fn explore_the_vastlands_gives_each_player_a_land_and_a_spell() {
    let mut g = kinnan();
    for p in [P0, P1, P2] {
        let lib = &mut g.player_mut(p).library;
        lib.clear();
    }
    for p in [P0, P1, P2] {
        for n in ["Llanowar Elves", "Island", "Ponder", "Birds of Paradise", "Fyndhorn Elves", "Forest"] {
            let c = card(&g, n);
            g.player_mut(p).library.push(c);
        }
    }
    let life0 = g.player(P1).life;
    sim_core::impls::t45_chulane_kinnan::explore_vastlands(&mut g, P0).unwrap();
    for p in [P0, P1, P2] {
        let pl = g.player(p);
        assert_eq!(pl.hand.len(), 2, "a land and a sorcery from the top five");
        assert!(pl.hand.contains(&card(&g, "Ponder")) && pl.hand.contains(&card(&g, "Forest")));
        assert_eq!(pl.library.len(), 4);
    }
    assert_eq!(g.player(P1).life, life0 + 3);
}

// ------------------------------------------------------------------ combos
#[test]
fn intruder_alarm_shrieking_drake_and_a_blue_dork_with_chulane_is_a_combo() {
    let mut g = chulane();
    perm(&mut g, P0, "Chulane, Teller of Tales");
    perm(&mut g, P0, "Intruder Alarm");
    hand(&mut g, P0, &["Shrieking Drake"]);
    let part = "Intruder Alarm + Shrieking Drake";
    assert!(!combo_ready(&g, P0, part), "no creature to pay {{U}}");
    perm(&mut g, P0, "Llanowar Elves");
    assert!(!combo_ready(&g, P0, part), "an Elf taps for {{G}} only");
    perm(&mut g, P0, "Birds of Paradise");
    assert!(combo_ready(&g, P0, part));
    let i = combos::combo_index(part).unwrap() as i64;
    g.step = Step::Main1;
    assert!(combos::attempt(&mut g, P0, i).unwrap());
    assert!(g.over && g.winner == Some(P0));
}

#[test]
fn aluren_cloudstone_curio_and_chulane_is_a_combo() {
    let mut g = chulane();
    for n in ["Aluren", "Cloudstone Curio", "Chulane, Teller of Tales", "Coiling Oracle"] {
        perm(&mut g, P0, n);
    }
    let part = "Aluren + Cloudstone Curio";
    assert!(!combo_ready(&g, P0, part), "a second cheap creature is needed in hand");
    hand(&mut g, P0, &["Llanowar Elves"]);
    lands(&mut g, P0, "Forest", 1, false);
    assert!(combo_ready(&g, P0, part));
}

#[test]
fn freed_from_the_real_combos_with_tender_nyxbloom_and_cub() {
    let mut g = kinnan();
    perm(&mut g, P0, "Freed from the Real");
    perm(&mut g, P0, "Bloom Tender");
    assert!(!combo_ready(&g, P0, "Aura + Bloom Tender"), "no outlet for the mana");
    hand(&mut g, P0, &["Walking Ballista"]);
    assert!(combo_ready(&g, P0, "Aura + Bloom Tender"));

    let mut g = kinnan();
    hand(&mut g, P0, &["Walking Ballista", "Freed from the Real"]);
    lands(&mut g, P0, "Island", 1, false);
    perm(&mut g, P0, "Nyxbloom Ancient");
    perm(&mut g, P0, "Llanowar Elves");
    assert!(!combo_ready(&g, P0, "Nyxbloom Ancient"), "the Elf makes no {{U}}");
    perm(&mut g, P0, "Birds of Paradise");
    assert!(combo_ready(&g, P0, "Nyxbloom Ancient"), "the Aura cast from hand");

    let mut g = kinnan();
    hand(&mut g, P0, &["Walking Ballista"]);
    perm(&mut g, P0, "Pemmin's Aura");
    perm(&mut g, P0, "Badgermole Cub");
    perm(&mut g, P0, "Birds of Paradise");
    assert!(combo_ready(&g, P0, "Badgermole Cub + a blue"));
}
