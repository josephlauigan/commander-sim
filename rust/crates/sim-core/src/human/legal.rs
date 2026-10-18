//! What the person may do by the rules (Python's `play/legal.py`): legality checks with Python's wording, the
//! legal targets of a spell, and how a target is described to the person.

use super::{Target, mana};
use crate::cards::{Colors, Types};
use crate::engine::values::{colors_of, epow, etgh, protected_from, untargetable};
use crate::engine::{self, cast, hooks};
use crate::hooks::Event;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Game, Perm, Step};
use crate::tag::Tag;
use std::sync::OnceLock;

/// Teferi's Protection and The One Ring give protection from everything.
pub fn player_targetable(g: &Game, q: PlayerId) -> bool {
    let q = g.player(q);
    q.alive && !q.life_locked && !q.ring_prot
}

type Guesser = fn(&Game, &Perm) -> Option<String>;
static GUESSER: OnceLock<Guesser> = OnceLock::new();

/// The practice crate registers Python's `view.guess_token` (it needs the token catalog of `images.py`).
pub fn set_token_guesser(f: Guesser) {
    let _ = GUESSER.set(f);
}

/// A token's name as players say it: 'Orc Army', 'Goblin token', 'Treasure' (Python's `view.token_name`).
pub fn token_name(g: &Game, m: &Perm) -> String {
    if m.army {
        return "Orc Army".into();
    }
    if !m.name.is_empty() && m.name != "Token" {
        return m.name.to_string();
    }
    let mut kinds: Vec<&str> = m.ttypes.iter().copied().filter(|t| !t.is_empty()).collect();
    kinds.sort_unstable();
    if !kinds.is_empty() {
        let words: Vec<String> = kinds.iter().map(|k| title(k)).collect();
        return format!("{} token", words.join(" "));
    }
    match GUESSER.get().and_then(|f| f(g, m)) {
        Some(guess) => format!("{guess} token"),
        None => "Token".into(),
    }
}

/// Python's `str.title()` for a lower-case word: first letter up, the rest down, after every non-letter.
pub fn title(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut up = true;
    for ch in s.chars() {
        if ch.is_alphabetic() {
            if up {
                out.extend(ch.to_uppercase());
            } else {
                out.extend(ch.to_lowercase());
            }
            up = false;
        } else {
            out.push(ch);
            up = true;
        }
    }
    out
}

/// 'Sheoldred, the Apocalypse (Prosper, tapped)' / 'Krenko (player, 31 life)'.
pub fn describe_target(g: &Game, p: PlayerId, x: Target) -> String {
    match x {
        Target::Player(q) => {
            let pl = g.player(q);
            format!("{} (player, {} life){}", pl.name, pl.life, if q == p { " (you)" } else { "" })
        }
        Target::Perm(id) => {
            let m = g.perm(id);
            let name = if m.token { token_name(g, m) } else { m.name.to_string() };
            let mut bits = vec![if m.owner == p { "you".to_string() } else { g.player(m.owner).name.to_string() }];
            if g.is_creature(id) {
                bits.push(format!("{}/{}", epow(g, id), etgh(g, id)));
            }
            if m.tapped {
                bits.push("tapped".into());
            }
            format!("{name} ({})", bits.join(", "))
        }
    }
}

// ------------------------------------------------------------------ timing
/// land drops p has this turn (Exploration, Azusa and similar add more)
pub fn land_drops(g: &Game, p: PlayerId) -> i32 {
    1 + if g.hooks.is_empty() { 0 } else { hooks::total_count(g, Event::ExtraLands, p) }
        + g.player(p).extra_land_now as i32
}

/// None if p may do sorcery-speed things now (its own main phase), else why not
pub fn sorcery_timing(g: &Game, p: PlayerId) -> Option<String> {
    if g.active != Some(p) {
        return Some("It isn't your turn.".into());
    }
    if !matches!(g.step, Step::Main1 | Step::Main2) {
        return Some("You can only do that in one of your main phases.".into());
    }
    if g.responding > 0 {
        return Some("You can only do that when nothing is waiting to resolve (not in response).".into());
    }
    None
}

fn nm(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

pub fn check_land(g: &Game, p: PlayerId, c: CardId) -> Option<String> {
    if !g.player(p).hand.contains(&c) {
        return Some(format!("{} isn't in your hand.", nm(g, c)));
    }
    if !g.db.get(c).land {
        return Some(format!("{} isn't a land.", nm(g, c)));
    }
    if let Some(why) = sorcery_timing(g, p) {
        return Some(why.replace("do that", "play a land"));
    }
    if g.player(p).lands_played as i32 >= land_drops(g, p) {
        return Some(if land_drops(g, p) == 1 {
            "You've already played a land this turn.".into()
        } else {
            "You've used all your land drops this turn.".into()
        });
    }
    None
}

pub fn instant_speed(g: &Game, p: PlayerId, c: CardId) -> bool {
    let d = g.db.get(c);
    d.instant || d.tag(Tag::Flash) || d.has_kw("flash") || cast::grants_flash(g, p, c)
}

/// the colours of c's Phyrexian mana symbols ({B/P}: 'B'), from its printed mana cost; '' if none
pub fn phyrexian(g: &Game, c: CardId) -> String {
    g.db.get(c).phyrexian.to_string()
}

/// What c costs to cast (before target-dependent extras), with every Phyrexian symbol counted as its mana (the
/// person chooses mana or life as they cast it: `phyrexian_ways`). The engine stores a few cards at the cost the AI
/// always pays; the person pays the printed cost.
pub fn base_cost(g: &Game, p: PlayerId, c: CardId) -> (u32, String) {
    if nm(g, c) == "Bloodchief's Thirst" {
        return (0, "B".into()); // {B}; kicker {2}{B} for a target over mana value 2
    }
    let (gn, mut pips) = engine::mana::cost_of(g, p, c);
    let d = g.db.get(c);
    let mut seen = vec![];
    for col in phyrexian(g, c).chars() {
        if seen.contains(&col) {
            continue;
        }
        seen.push(col);
        while pips.matches(col).count() < d.pips.matches(col).count() {
            pips.push(col); // the engine may have dropped one for the AI
        }
    }
    (gn, pips)
}

/// The ways to pay c's Phyrexian symbols you can afford now: [(life paid, pips left to pay with mana)], each symbol
/// paid with its colour or with 2 life (and you need that much life to pay it).
pub fn phyrexian_ways(g: &Game, p: PlayerId, c: CardId, gn: u32, pips: &str) -> Vec<(i32, String)> {
    let phy: Vec<char> = phyrexian(g, c).chars().collect();
    let mut out = vec![];
    for k in 0..=phy.len() {
        let mut left = pips.to_string();
        for &col in &phy[..k] {
            if let Some(i) = left.find(col) {
                left.remove(i);
            }
        }
        if 2 * k as i32 > g.player(p).life.max(0) && k > 0 {
            continue;
        }
        if mana::cost_problem(g, p, gn, &left).is_none() {
            out.push((2 * k as i32, left));
        }
    }
    out
}

/// Can p cast c from `zone` ('hand' or 'cmd') now? Checks where it is, timing, locks and the mana in the pool.
pub fn check_cast(g: &mut Game, p: PlayerId, c: CardId, zone: &str) -> Option<String> {
    let d = g.db.get(c).clone();
    let pl = g.player(p);
    let name = d.name.to_string();
    if zone == "hand" && !pl.hand.contains(&c) {
        return Some(format!("{name} isn't in your hand."));
    }
    if zone == "cmd" && !pl.cmd_zone_of(c) {
        return Some("Your commander is not in the command zone.".into());
    }
    if d.land {
        return Some(format!("{name} is a land: play it as your land drop instead."));
    }
    if d.tag(Tag::Ctr) {
        return Some(format!("{name} counters a spell: cast it when a spell you can counter is cast."));
    }
    if engine::values::silenced(g, p) {
        return Some("You can't cast spells during this player's turn (Conqueror's Flail).".into());
    }
    if name == "Return the Favor" {
        return Some("Return the Favor targets a spell: cast it in response to one.".into());
    }
    if !instant_speed(g, p, c) && name != "Necromancy" {
        // Necromancy: "as though it had flash"
        if let Some(why) = sorcery_timing(g, p) {
            return Some(why.replace("do that", &format!("cast {name}")));
        }
    }
    if !cast::castable(g, p, c, zone) {
        return Some(format!("Something on the battlefield stops you casting {name} right now."));
    }
    if d.tag(Tag::Needsac) && !g.player(p).perms.iter().any(|&m| g.is_creature(m) && !g.perm(m).phased) {
        return Some(format!("{name} needs a creature to sacrifice as you cast it, and you control none."));
    }
    if d.has_dsl() && !cast::additional_cost(g, p, c, true).unwrap_or(false) {
        return Some(format!("You can't pay {name}'s additional cost."));
    }
    let tg = spell_targets(g, p, c);
    if let Some(t) = &tg
        && t.is_empty()
        && !d.tag(Tag::Wipe)
    {
        return Some(format!("{name} has no legal target right now."));
    }
    if let Some(kind) = d.tags.str(Tag::Rean)
        && super::choices::rean_candidates(g, p, kind).is_empty()
    {
        return Some(format!("{name} has no creature card to return."));
    }
    if name == "Deadly Dispute"
        && g.player(p).treasures == 0
        && !g.player(p).perms.iter().any(|&m| {
            let x = g.perm(m);
            (g.is_creature(m) || x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT))) && !x.phased
        })
    {
        return Some("Deadly Dispute needs an artifact or creature to sacrifice as you cast it.".into());
    }
    if let Some(why) = super::cards::needs(g, p, c) {
        return Some(why);
    }
    let (gn, pips) = base_cost(g, p, c);
    let phy = phyrexian(g, c);
    if !phy.is_empty() && !phyrexian_ways(g, p, c, gn, &pips).is_empty() {
        return None; // mana, life, or a mix
    }
    if let Some(why) = mana::cost_problem(g, p, gn, &pips) {
        let extra = if phy.is_empty() {
            String::new()
        } else {
            let syms: Vec<String> = phy.chars().map(|x| format!("{x}/P")).collect();
            format!(" Its {{{}}} can be paid with 2 life each instead.", syms.join("}{"))
        };
        return Some(format!("Can't cast {name}. {why}{extra}"));
    }
    None
}

// ------------------------------------------------------------------ targets
fn ty_has(g: &Game, m: PermId, t: Types) -> bool {
    match g.perm(m).cd {
        Some(c) => g.db.get(c).types.has(t),
        None => t == Types::CREATURE, // a token's 'C'
    }
}

fn tgt_ok(g: &Game, kind: &str, m: PermId) -> bool {
    let cr = g.is_creature(m);
    let (a, pw, e) =
        (ty_has(g, m, Types::ARTIFACT), ty_has(g, m, Types::PLANESWALKER), ty_has(g, m, Types::ENCHANTMENT));
    match kind {
        "cp" => cr || pw,
        "cap" => cr || a || pw,
        "ce" => cr || e,
        "nl" | "p" => true,
        "a" => a,
        "cna" => cr && !a,
        "ae" => a || e,
        "blue" => g.perm(m).cd.is_some_and(|c| g.db.get(c).pips.contains('U')),
        "ac" => cr || a,
        _ => cr, // 'c' and anything unknown
    }
}

/// What spell c may target, by the rules (not by what the AI would pick): a list of permanents and players, or None
/// if c doesn't target. Your own permanents count too; hexproof, shroud and protection are respected.
pub fn spell_targets(g: &Game, p: PlayerId, c: CardId) -> Option<Vec<Target>> {
    let d = g.db.get(c);
    let t = &d.tags;
    if !t.has(Tag::Rem) {
        return None;
    }
    let tgt = t.str(Tag::Tgt).unwrap_or("c");
    let pips = Colors::from_letters(&d.pips);
    let mut out = vec![];
    for q in g.players.iter().filter(|q| q.alive) {
        for &m in &q.perms {
            let x = g.perm(m);
            if x.phased || !tgt_ok(g, tgt, m) {
                continue;
            }
            if t.has(Tag::Alsoart) && !g.is_creature(m) && !ty_has(g, m, Types::ARTIFACT) {
                continue;
            }
            if untargetable(g, m) || protected_from(g, m, pips) {
                continue;
            }
            if t.has(Tag::Nonblack) && colors_of(g, m).has('B') {
                continue;
            }
            if t.has(Tag::Evil) && !(ty_has(g, m, Types::ENCHANTMENT) || (g.is_creature(m) && etgh(g, m) >= 4)) {
                continue; // Destroy Evil
            }
            if t.has(Tag::Verdict) && !g.in_combat.contains(&m) {
                continue; // Divine Verdict
            }
            let mcd = x.cd.map(|c| g.db.get(c));
            if t.has(Tag::Mv4) && mcd.is_some_and(|m| m.cmc > 4) {
                continue;
            }
            if t.has(Tag::Newonly) && !engine::removal::entered_since_last_turn(g, p, m) {
                continue;
            }
            if &*d.name == "Fatal Push"
                && let Some(mc) = mcd
                && mc.cmc > 2
                && (mc.cmc > 4 || !crate::cardcode::revolt(g, p))
            {
                continue;
            }
            out.push(Target::Perm(m));
        }
    }
    if t.has(Tag::Face) {
        out.extend(g.players.iter().filter(|q| player_targetable(g, q.id)).map(|q| Target::Player(q.id)));
    }
    Some(out)
}

/// More mana this target needs: ward (on an opponent's permanent), Bloodchief's Thirst's kicker.
pub fn target_extra_cost(g: &Game, p: PlayerId, c: CardId, x: Target) -> (u32, String) {
    let (mut gn, mut pips) = (0, String::new());
    if let Target::Perm(m) = x {
        let xm = g.perm(m);
        if let Some(cd) = xm.cd {
            let md = g.db.get(cd);
            if xm.owner != p && md.ward > 0 {
                gn += md.ward;
            }
            if nm(g, c) == "Bloodchief's Thirst" && md.cmc > 2 {
                gn += 2;
                pips.push('B'); // kicked: {2}{B} more, any mana value
            }
        }
    }
    (gn, pips)
}

// ------------------------------------------------------------------ equipment
/// the equip cost of equipment m, or None if m isn't equipment
pub fn equip_cost(g: &Game, m: PermId) -> Option<u32> {
    let cd = g.db.get(g.perm(m).cd?);
    for a in cd.abilities.iter() {
        if let crate::dsl::model::Ability::Static { kind, mana, .. } = a
            && kind == "equip_cost"
        {
            return Some(mana.unwrap_or(0));
        }
    }
    let t = &cd.tags;
    if t.str(Tag::Prot) == Some("boots") {
        return Some(if &*cd.name == "Lightning Greaves" { 0 } else { 1 });
    }
    for (tag, v) in
        [(Tag::Sword, 2), (Tag::Flail, 2), (Tag::Cloak, 2), (Tag::Animist, 2), (Tag::Helm, 1), (Tag::Nim, 4)]
    {
        if t.has(tag) {
            return Some(v);
        }
    }
    None
}

pub fn check_equip(g: &Game, p: PlayerId, e: PermId, target: PermId) -> Option<String> {
    let Some(cost) = equip_cost(g, e).filter(|_| g.player(p).perms.contains(&e)) else {
        return Some("That isn't equipment you control.".into());
    };
    if let Some(why) = sorcery_timing(g, p) {
        return Some(why.replace("do that", "equip"));
    }
    if !g.player(p).perms.contains(&target) || !g.is_creature(target) || g.perm(target).phased {
        return Some("Equip only onto a creature you control.".into());
    }
    if g.perm(e).attached == Some(target) {
        return Some(format!("{} is already on {}.", g.perm(e).name, g.perm(target).name));
    }
    mana::cost_problem(g, p, cost, "").map(|why| format!("Can't equip {}. {why}", g.perm(e).name))
}

// ------------------------------------------------------------------ counterspells
/// a way to cast counterspell ctr without its mana cost right now, in words, or None
pub fn alternative_counter_cost(g: &Game, p: PlayerId, ctr: CardId) -> Option<&'static str> {
    let d = g.db.get(ctr);
    let pl = g.player(p);
    let blues = pl.hand.iter().any(|&x| x != ctr && g.db.get(x).pips.contains('U'));
    if d.tag(Tag::Fierce) && engine::values::commander_out(g, p) {
        return Some("free while your commander is out");
    }
    if d.tag(Tag::Pact) {
        return Some("free now; pay {3}{U}{U} at your next upkeep or lose");
    }
    if d.tag(Tag::Misstep) && pl.life > 10 {
        return Some("2 life");
    }
    if d.tag(Tag::Fon) && g.active != Some(p) && blues {
        return Some("exile a blue card");
    }
    if d.tag(Tag::Free) && blues {
        return Some("exile a blue card and pay 1 life");
    }
    None
}

/// can p counter `spell` (on the stack) with ctr from hand now?
pub fn check_counter(g: &Game, p: PlayerId, ctr: CardId, spell: CardId) -> Option<String> {
    if !g.player(p).hand.contains(&ctr) {
        return Some(format!("{} isn't in your hand.", nm(g, ctr)));
    }
    if !g.db.get(ctr).tag(Tag::Ctr) {
        return Some(format!("{} isn't a counterspell.", nm(g, ctr)));
    }
    if !engine::stack::counter_ok(g, ctr, spell) {
        return Some(format!("{} can't counter {}.", nm(g, ctr), nm(g, spell)));
    }
    if engine::values::silenced(g, p) {
        return Some("You can't cast spells during this player's turn (Conqueror's Flail).".into());
    }
    if g.cast_checks() && !cast::castable(g, p, ctr, "hand") {
        return Some(format!("Something on the battlefield stops you casting {} right now.", nm(g, ctr)));
    }
    let (gn, pips) = engine::stack::counter_cost(g, ctr, Some(spell), Some(p));
    let problem = mana::cost_problem(g, p, gn, &pips);
    match problem {
        None => None,
        Some(_) if alternative_counter_cost(g, p, ctr).is_some() => None,
        Some(why) => Some(format!("Can't cast {}. {why}", nm(g, ctr))),
    }
}

// ------------------------------------------------------------------ casting from the graveyard
/// How p may cast a card from their graveyard.
#[derive(Debug, Clone, PartialEq)]
pub enum GyKind {
    Yawg,
    Flashback,
    Sac3,
    Escape,
    Muldrotha,
}

/// How p may cast card c from their graveyard now: (kind, generic, pips), or None. Yawgmoth's Will: the normal
/// cost; flashback: its own cost; Dread Return: sacrifice three creatures; Underworld Breach: the normal cost plus
/// three other cards from your graveyard exiled.
pub fn gy_mode(g: &Game, p: PlayerId, c: CardId) -> Option<(GyKind, u32, String)> {
    let pl = g.player(p);
    let d = g.db.get(c);
    if !pl.gy.contains(&c) || d.land {
        return None;
    }
    if pl.yawg && pl.yawg_gy.contains(&c) {
        let (gn, pips) = engine::mana::cost_of(g, p, c);
        return Some((GyKind::Yawg, gn, pips));
    }
    // flashback costs the engine's tags leave out
    let mut fb: Option<String> = d.tags.str(Tag::Fb).filter(|s| !s.is_empty()).map(String::from);
    let mut fb_cost: Option<(u32, String)> = None;
    match &*d.name {
        "Unburial Rites" if fb.is_none() => fb_cost = Some((3, "W".into())),
        "Dread Return" if fb.is_none() => fb = Some("sac3".into()),
        _ => {}
    }
    if fb.is_none() && fb_cost.is_none() && (d.instant || d.sorcery) && super::cards::granted_flashback(g, p, c) {
        fb_cost = Some(engine::mana::cost_of(g, p, c)); // Flashback (the card): its mana cost
    }
    if fb.as_deref() == Some("sac3") {
        return Some((GyKind::Sac3, 0, String::new()));
    }
    if let Some((gn, pips)) = fb_cost {
        return Some((GyKind::Flashback, gn, pips));
    }
    if let Some(s) = fb {
        let (gn, pips) = cast::parse_cost(&s);
        return Some((GyKind::Flashback, gn, pips));
    }
    if engine::values::has(g, p, Tag::Breach) && pl.gy.len() >= 4 {
        let (gn, pips) = engine::mana::cost_of(g, p, c);
        return Some((GyKind::Escape, gn, pips));
    }
    if crate::cardcode::muld_types(g, p, c) {
        // Muldrotha: a permanent of an unused type
        let (gn, pips) = engine::mana::cost_of(g, p, c);
        return Some((GyKind::Muldrotha, gn, pips));
    }
    None
}

pub fn check_cast_gy(g: &Game, p: PlayerId, c: CardId) -> Option<String> {
    let name = nm(g, c).to_string();
    let Some((kind, gn, pips)) = gy_mode(g, p, c) else {
        return Some(format!("You can't cast {name} from your graveyard right now."));
    };
    if !instant_speed(g, p, c)
        && let Some(why) = sorcery_timing(g, p)
    {
        return Some(why.replace("do that", &format!("cast {name}")));
    }
    if engine::values::silenced(g, p) {
        return Some("You can't cast spells during this player's turn (Conqueror's Flail).".into());
    }
    if !cast::castable(g, p, c, "gy") {
        return Some(format!("Something on the battlefield stops you casting {name} right now."));
    }
    let d = g.db.get(c);
    if kind == GyKind::Sac3 && g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count() < 3
    {
        return Some(format!("{name}'s flashback needs three creatures to sacrifice."));
    }
    if let Some(l) = d.tags.int(Tag::Fblife)
        && kind == GyKind::Flashback
        && g.player(p).life <= l
    {
        return Some(format!("{name}'s flashback costs {l} life, which you don't have to spare."));
    }
    if let Some(k) = d.tags.str(Tag::Rean)
        && super::choices::rean_candidates(g, p, k).is_empty()
    {
        return Some(format!("{name} has no creature card to return."));
    }
    mana::cost_problem(g, p, gn, &pips).map(|why| format!("Can't cast {name} from your graveyard. {why}"))
}

pub fn check_land_gy(g: &Game, p: PlayerId, c: CardId) -> Option<String> {
    let pl = g.player(p);
    if !pl.gy.contains(&c) || !g.db.get(c).land {
        return Some(format!("{} isn't a land in your graveyard.", nm(g, c)));
    }
    if !(pl.yawg && pl.yawg_gy.contains(&c)) && !crate::cardcode::muld_types(g, p, c) {
        return Some("You can't play lands from your graveyard right now.".into());
    }
    if let Some(why) = sorcery_timing(g, p) {
        return Some(why.replace("do that", "play a land"));
    }
    if pl.lands_played as i32 >= land_drops(g, p) {
        return Some("You've already played a land this turn.".into());
    }
    None
}

/// `legal.WHAT`: how a spell's target kind reads in a prompt
pub fn what(kind: &str) -> &'static str {
    match kind {
        "c" => "creature",
        "cp" => "creature or planeswalker",
        "cap" => "creature, artifact or planeswalker",
        "ce" => "creature or enchantment",
        "nl" => "nonland permanent",
        "p" => "permanent",
        "a" => "artifact",
        "cna" => "nonartifact creature",
        "ae" => "artifact or enchantment",
        "blue" => "blue permanent",
        "ac" => "artifact or creature",
        _ => "target",
    }
}
