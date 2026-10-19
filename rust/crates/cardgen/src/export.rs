//! `cardgen export`: write `data/cards.json` and `data/decks.json` (the files the engine loads) from the data files,
//! the Scryfall cache and the deck lists. The files have the layout `sim-core::export` reads; the output is
//! deterministic: the same inputs give the same bytes.

use crate::cardinfo;
use crate::cd::{Cd, strs};
use crate::data::{self, Hooks, MyDeck, Project};
use crate::decks::{self, PoolDeck};
use crate::json::{self, J, Obj};
use crate::obj;
use crate::scryfall::{self, Cache};
use crate::sources::{self, Built, Wanted};
use std::collections::{BTreeMap, BTreeSet};

/// Everything read from disk.
pub struct Inputs {
    pub project: Project,
    pub carddb: Vec<data::CardDbLine>,
    pub overrides: Vec<(String, data::Entry)>,
    pub hooks: BTreeMap<String, Hooks>,
    pub my: Vec<MyDeck>,
    pub my_cards: Vec<(String, Vec<String>)>,
    pub pool: Vec<PoolDeck>,
    pub cache: Cache,
}

impl Inputs {
    pub fn load(project: &Project) -> Result<Inputs, String> {
        let my = data::load_my_decks(project)?;
        let mut my_cards = vec![];
        for d in &my {
            my_cards.push((d.key.clone(), decks::load(&decks::deck_path(project, d))?));
        }
        Ok(Inputs {
            project: project.clone(),
            carddb: data::load_carddb(project)?,
            overrides: data::load_overrides(project)?,
            hooks: data::load_hooks(project)?,
            my,
            my_cards,
            pool: decks::load_pool(project)?,
            cache: Cache::load(&project.data("scryfall_cache.json"))?,
        })
    }

    /// the cards the lists name (and the commanders)
    pub fn wanted(&self) -> Wanted {
        let main: BTreeSet<String> = self.my_cards.iter().flat_map(|(_, c)| c.iter().cloned()).collect();
        let mut played = main.clone();
        for d in &self.pool {
            played.extend(d.cards.iter().cloned());
        }
        let mut extra: BTreeSet<String> = self.my.iter().map(|d| d.commander.clone()).collect();
        extra.extend(self.my.iter().filter_map(|d| d.commander2.clone()));
        extra.extend(self.pool.iter().map(|d| d.commander.clone()));
        extra.extend(self.pool.iter().filter_map(|d| d.commander2.clone()));
        extra.extend(self.cache.names());
        // cards whose card code is kept after the deck that played them was retired (the titania deck, 2026-10-19):
        // double-faced, so the cache names them "A // B" and only a deck list would have asked for the front face
        extra.extend(["Studious First-Year", "Infestation Expert"].map(String::from));
        Wanted { main, played, extra }
    }

    /// every card name the lists need that is not in the cache yet (nor hand-tagged, unless a pool deck plays it:
    /// `cardgen validate` checks pool decks against Scryfall's records, hand-tagged or not)
    pub fn uncached(&self) -> Vec<String> {
        let hand: BTreeSet<&str> = self.carddb.iter().map(|l| l.name.as_str()).collect();
        let pool: BTreeSet<&str> = self.pool.iter().flat_map(|d| d.cards.iter().map(String::as_str)).collect();
        let w = self.wanted();
        let mut v: Vec<String> = w
            .played
            .iter()
            .chain(w.extra.iter())
            .filter(|n| (!hand.contains(n.as_str()) || pool.contains(n.as_str())) && !self.cache.contains(n))
            .cloned()
            .collect();
        v.sort();
        v.dedup();
        v
    }

    pub fn build(&self) -> Built {
        sources::build(&self.carddb, &self.overrides, &self.cache, &self.wanted())
    }
}

fn sorted_strs(v: &[String]) -> J {
    let mut s: Vec<&String> = v.iter().collect();
    s.sort();
    J::Arr(s.into_iter().map(|x| J::from(x.as_str())).collect())
}

/// one card's record in cards.json
pub fn card_record(cd: &Cd, cache: &Cache, hooks: Option<&Hooks>) -> J {
    let land = cd.land();
    let (lt, basic) = if land { cardinfo::land_types(cache, &cd.name) } else { (vec![], false) };
    let mut events: Vec<&String> = hooks.map(|h| h.events.iter().collect()).unwrap_or_default();
    events.sort();
    J::Obj(obj! {
        "name" => cd.name.as_str(),
        "types" => cd.types.as_str(),
        "generic" => cd.generic,
        "pips" => cd.pips.as_str(),
        "tags" => cd.tags.clone(),
        "pow" => cd.pow,
        "tgh" => cd.tgh,
        "bomb" => cd.bomb,
        "dsl" => match &cd.dsl { Some(d) => J::Arr(d.iter().cloned().map(J::Obj).collect()), None => J::Null },
        "start_loyalty" => cd.start_loyalty.clone(),
        "kws" => strs(&cd.kws),
        "subtypes" => strs(&cd.subtypes),
        "protfrom" => cd.protfrom.as_str(),
        "ward" => cd.ward,
        "identity" => cd.identity.clone(),
        "game_changer" => cd.game_changer,
        "source" => cd.source.as_str(),
        "unparsed" => J::Arr(cd.unparsed.iter().map(|s| J::from(s.as_str())).collect()),
        "derived" => obj! {"cmc" => cd.cmc, "land" => land, "creature" => cd.creature(), "instant" => cd.instant(),
            "sorcery" => cd.sorcery(), "perm" => cd.perm()},
        "phyrexian" => cardinfo::phyrexian(cache, cd),
        "land_etb_fx" => J::Arr(if land {
            cardinfo::land_etb_fx(cache, cd).into_iter().map(|(k, n)| J::Arr(vec![J::from(k), J::from(n)])).collect()
        } else { vec![] }),
        "enters_rule" => if land { cardinfo::enters_rule(cache, cd).unwrap_or(J::Null) } else { J::Null },
        "land_types" => J::Arr(lt.iter().map(|s| J::from(s.as_str())).collect()),
        "basic" => basic,
        "python_hooks" => J::Arr(events.into_iter().map(|s| J::from(s.as_str())).collect()),
        "spell_prio" => hooks.is_some_and(|h| h.prio),
    })
}

/// the colour identity in WUBRG order
fn ident_of(colors: impl Iterator<Item = char> + Clone) -> String {
    "WUBRG".chars().filter(|c| colors.clone().any(|x| x == *c)).collect()
}

/// the decks.json document
pub fn decks_doc(inp: &Inputs) -> Result<J, String> {
    let mut mine = vec![];
    let mut keys: Vec<&MyDeck> = inp.my.iter().collect();
    keys.sort_by(|a, b| a.key.cmp(&b.key));
    for d in keys {
        let cards = &inp.my_cards.iter().find(|(k, _)| *k == d.key).unwrap().1;
        let mut o = obj! {"key" => d.key.as_str(), "name" => d.name.as_str(), "commander" => d.commander.as_str()};
        if let Some(c2) = &d.commander2 {
            o.0.push(("commander2".to_string(), J::from(c2.as_str()))); // only decks with two commanders carry it
        }
        o.0.push(("ident".to_string(), J::from(ident_of(d.ident.chars()))));
        o.0.push(("tiers".to_string(), J::from(d.tiers.iter().map(|t| J::from(t.as_str())).collect::<Vec<J>>())));
        o.0.push(("cards".to_string(), J::from(sorted_strs(cards))));
        mine.push(J::Obj(o));
    }
    let mut pool = vec![];
    for d in &inp.pool {
        let rec = inp
            .cache
            .get(&d.commander)
            .ok_or(format!("pool deck {}: commander {} is not in the card cache", d.key, d.commander))?;
        let mut ids: Vec<char> = rec.strs("color_identity").iter().flat_map(|s| s.chars()).collect();
        let mut o = obj! {"tier" => d.tier.as_str(), "key" => d.key.as_str(), "name" => decks::display_name(&d.commander, d.commander2.as_deref()),
            "commander" => d.commander.as_str()};
        if let Some(c2) = &d.commander2 {
            let rec2 = inp
                .cache
                .get(c2)
                .ok_or(format!("pool deck {}: commander {} is not in the card cache", d.key, c2))?;
            ids.extend(rec2.strs("color_identity").iter().flat_map(|s| s.chars()));
            o.0.push(("commander2".to_string(), J::from(c2.as_str()))); // partner pairs only, as for your decks
        }
        o.0.push(("ident".to_string(), J::from(ident_of(ids.into_iter()))));
        o.0.push(("cards".to_string(), J::from(sorted_strs(&d.cards))));
        pool.push(J::Obj(o));
    }
    Ok(J::Obj(obj! {"version" => 1, "mine" => mine, "pool" => pool}))
}

pub struct Exported {
    pub cards_json: String,
    pub decks_json: String,
    pub n_cards: usize,
    pub n_decks: usize,
}

/// Build both files' contents. Fails if a listed card has no Scryfall record (run without `--offline` to fetch).
pub fn export(inp: &Inputs) -> Result<Exported, String> {
    let built = inp.build();
    if !built.missing.is_empty() {
        return Err(format!(
            "no Scryfall data for: {} (cardgen export fetches missing cards unless --offline)",
            built.missing.join(", ")
        ));
    }
    let cards: Vec<J> = built.cards.values().map(|cd| card_record(cd, &inp.cache, inp.hooks.get(&cd.name))).collect();
    let decks = decks_doc(inp)?;
    let n_decks = decks.get("mine").and_then(|m| m.as_arr()).map_or(0, |a| a.len())
        + decks.get("pool").and_then(|m| m.as_arr()).map_or(0, |a| a.len());
    let doc = |body: J| format!("{}\n", json::dumps(&body, Some(1), false));
    let n_cards = cards.len();
    Ok(Exported {
        cards_json: doc(J::Obj(obj! {"version" => 1, "cards" => cards})),
        decks_json: doc(decks),
        n_cards,
        n_decks,
    })
}

/// Fetch what the cache lacks (unless offline), then export; writes the files into `out`.
pub fn run(project: &Project, out: &std::path::Path, offline: bool) -> Result<(usize, usize), String> {
    let mut inp = Inputs::load(project)?;
    if !offline {
        let missing = inp.uncached();
        if !missing.is_empty() {
            scryfall::fetch(&mut inp.cache, &missing, true);
            inp.cache.save()?;
        }
    }
    let e = export(&inp)?;
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    for (f, body) in [("cards.json", &e.cards_json), ("decks.json", &e.decks_json)] {
        std::fs::write(out.join(f), body).map_err(|e| format!("{f}: {e}"))?;
    }
    Ok((e.n_cards, e.n_decks))
}

#[allow(dead_code)]
fn _unused(_: Obj) {}
