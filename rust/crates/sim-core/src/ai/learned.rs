//! --- phase4-eval: the learned evaluation (`audit/phase4/eval-design.md`).
//!
//! `search::evaluate` scores a playout's last position. For a generic-policy deck with the learned evaluation on
//! (`AI_EVAL`, on by default), a position short of a win or a loss is scored by a model fitted on recorded games (a
//! logistic regression plus boosted trees): the estimate that the evaluated seat goes on to win, mapped onto
//! `evaluate`'s scale (below 95, monotone).
//!
//! - `raw`: the numbers read from a position (per seat, in turn order from the evaluated seat). The recorder writes
//!   them, and the model reads its features from them, so training and play share one reading.
//! - `features`: the model's inputs, from `raw`.
//! - `Model`: the fitted weights (`data/eval/model.toml`, embedded and checked on first use).
//! - the recorder (`AI_EVAL_RECORD=<dir>`): one record at the end of each turn of a generic-policy seat in the real
//!   game, labelled at the game's end. It only reads the game: play is identical with it on or off.

use super::search;
use crate::ids::PlayerId;
use crate::state::Game;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;

// ------------------------------------------------------------------ the switch
enum Mode {
    All,
    Keys(Vec<String>),
}

fn mode() -> &'static Mode {
    static M: OnceLock<Mode> = OnceLock::new();
    M.get_or_init(|| parse_mode(std::env::var("AI_EVAL").ok().as_deref()))
}

/// `AI_EVAL`: unset or `1` learned for every generic-policy deck, `0` today's evaluation, `a,b` learned for those keys.
/// Stage C: the learned evaluation is the default for every generic-policy deck (the paired tests were level on t1-t5
/// in both halves, `audit/phase4/eval-handoff.md`); `0` keeps today's for a measurement arm. JD's decks and Tier 6
/// always keep today's (`values::generic_deck`).
fn parse_mode(v: Option<&str>) -> Mode {
    match v.map(str::trim) {
        None | Some("") | Some("1") => Mode::All,
        Some("0") => Mode::Keys(vec![]),
        Some(list) => Mode::Keys(list.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()),
    }
}

/// does the look-ahead score p's positions with the learned evaluation?
pub fn on(g: &Game, p: PlayerId) -> bool {
    let m = mode();
    if let Mode::Keys(k) = m
        && k.is_empty()
    {
        return false;
    }
    super::values::generic_deck(g, p)
        && match m {
            Mode::All => true,
            Mode::Keys(k) => k.iter().any(|x| x == g.player(p).key),
        }
}

// ------------------------------------------------------------------ what a position is read as
/// seats read (a pod has 4; a missing seat reads as all zeros)
pub const SEATS: usize = 4;
/// numbers read per seat
pub const PER: usize = 16;
pub const RAW: usize = SEATS * PER;

pub const ALIVE: usize = 0;
pub const LIFE: usize = 1;
pub const BOARD: usize = 2;
pub const POWER: usize = 3;
pub const HAND: usize = 4;
pub const LANDS: usize = 5;
pub const TREASURES: usize = 6;
pub const COMBO: usize = 7;
pub const STRENGTH: usize = 8;
pub const THREAT: usize = 9;
pub const CMD_TAKEN: usize = 10;
pub const POISON: usize = 11;
pub const DISTANCE: usize = 12;
pub const HAND_WORTH: usize = 13;
pub const CREATURES: usize = 14;
/// the commander damage this seat has taken from the evaluated seat's commanders
pub const CMD_FROM_ME: usize = 15;

/// a position as the evaluated seat p sees it, with the facts the features need besides the per-seat numbers
#[derive(Clone, Debug)]
pub struct Raw {
    pub v: [f32; RAW],
    /// p's seat index in turn order
    pub seat: u8,
    /// seats at the table
    pub nseats: u8,
    /// p's own turns so far
    pub turns: u16,
    /// today's raw term (`search::raw_term`: before the tanh)
    pub old: f32,
}

thread_local! {
    /// `values::threat` by card, for one card database (it reads no game state)
    static THREAT_CACHE: RefCell<(usize, HashMap<u16, f64>)> = RefCell::new((0, HashMap::new()));
}

fn threat_of(g: &Game, c: crate::ids::CardId) -> f64 {
    let db = std::sync::Arc::as_ptr(&g.db) as usize;
    THREAT_CACHE.with(|t| {
        let mut t = t.borrow_mut();
        if t.0 != db {
            *t = (db, HashMap::new());
        }
        *t.1.entry(c.0).or_insert_with(|| super::values::threat(g, c))
    })
}

/// read the position for p (nothing in the game changes)
pub fn raw(g: &Game, p: PlayerId) -> Raw {
    use crate::engine::values::epow;
    let n = g.players.len();
    let mut v = [0.0f32; RAW];
    for k in 0..SEATS.min(n) {
        let q = PlayerId(((p.index() + k) % n) as u8);
        let pl = g.player(q);
        if !pl.alive {
            continue;
        }
        let o = k * PER;
        let live: Vec<_> = pl.perms.iter().copied().filter(|&m| !g.perm(m).phased).collect();
        v[o + ALIVE] = 1.0;
        v[o + LIFE] = pl.life as f32;
        v[o + BOARD] = live.iter().map(|&m| search::board_value(g, m)).sum::<f64>() as f32;
        v[o + POWER] = live.iter().filter(|&&m| g.is_creature(m)).map(|&m| epow(g, m) as f64).sum::<f64>() as f32;
        v[o + HAND] = pl.hand.len() as f32;
        v[o + LANDS] = pl.lands.len() as f32;
        v[o + TREASURES] = pl.treasures as f32;
        v[o + COMBO] = crate::cardcode::combo_progress(g, q) as f32;
        v[o + STRENGTH] = search::strength(g, q) as f32;
        v[o + THREAT] = live.iter().filter_map(|&m| g.perm(m).cd).map(|c| threat_of(g, c)).sum::<f64>() as f32;
        v[o + CMD_TAKEN] = pl.cmd_dmg.iter().chain(pl.cmd2_dmg.iter()).copied().max().unwrap_or(0) as f32;
        v[o + POISON] = pl.poison as f32;
        v[o + DISTANCE] = super::planner::distance(g, q).min(6) as f32;
        v[o + HAND_WORTH] = pl.hand.iter().map(|&c| super::values::worth(g, q, c, true)).sum::<f64>() as f32;
        v[o + CREATURES] = live.iter().filter(|&&m| g.is_creature(m)).count() as f32;
        v[o + CMD_FROM_ME] = if k == 0 { 0 } else { pl.cmd_dmg[p.index()].max(pl.cmd2_dmg[p.index()]) } as f32;
    }
    Raw {
        v,
        seat: p.index() as u8,
        nseats: n as u8,
        turns: g.player(p).turns.min(u16::MAX as u32) as u16,
        old: search::raw_term(g, p) as f32,
    }
}

// ------------------------------------------------------------------ features
/// the per-seat terms the features read for me and for the strongest opponent
const SIDE: [(&str, usize); 13] = [
    ("life", LIFE),
    ("board", BOARD),
    ("power", POWER),
    ("hand", HAND),
    ("lands", LANDS),
    ("treasures", TREASURES),
    ("combo", COMBO),
    ("threat", THREAT),
    ("hand_worth", HAND_WORTH),
    ("poison", POISON),
    ("cmd_taken", CMD_TAKEN),
    ("distance", DISTANCE),
    ("creatures", CREATURES),
];
/// the terms averaged over the live opponents
const MEAN: [(&str, usize); 5] =
    [("life", LIFE), ("board", BOARD), ("power", POWER), ("hand", HAND), ("threat", THREAT)];

pub const NF: usize = 2 * SIDE.len() + MEAN.len() + 6 + 4 + 3 + 2;

/// the feature names, in order (the model file must list the same)
pub fn names() -> Vec<String> {
    let mut v: Vec<String> = SIDE.iter().map(|s| format!("me_{}", s.0)).collect();
    v.extend(SIDE.iter().map(|s| format!("top_{}", s.0)));
    v.extend(MEAN.iter().map(|s| format!("mean_{}", s.0)));
    for s in ["live_opps", "str_minus_top", "str_minus_mean", "life_minus_low", "cmd_dealt", "low_life"] {
        v.push(s.into());
    }
    for s in
        ["turns", "turns_x_board", "turns_x_hand", "turns_x_combo", "seat1", "seat2", "seat3", "old_raw", "old_eval"]
    {
        v.push(s.into());
    }
    v
}

/// a signed log: the heavy-tailed terms (a board of 40 tokens) read on a log scale
fn sq(x: f64) -> f64 {
    x.signum() * x.abs().ln_1p()
}

/// seat k's term i as a feature reads it: life clamped as `strength` does, the heavy-tailed terms on a log scale
fn term(r: &Raw, k: usize, i: usize) -> f64 {
    let x = r.v[k * PER + i] as f64;
    match i {
        LIFE => x.clamp(0.0, 60.0),
        BOARD | POWER | THREAT | HAND_WORTH | CREATURES | TREASURES => sq(x),
        _ => x,
    }
}

/// the model's inputs (None when no opponent is alive: `evaluate` scores that as a win)
pub fn features(r: &Raw) -> Option<[f64; NF]> {
    let opps: Vec<usize> = (1..SEATS.min(r.nseats as usize)).filter(|&k| r.v[k * PER + ALIVE] > 0.0).collect();
    let &first = opps.first()?;
    let top =
        opps.iter().copied().fold(first, |b, k| if r.v[k * PER + STRENGTH] > r.v[b * PER + STRENGTH] { k } else { b });
    let mut x = [0.0; NF];
    let mut i = 0;
    let mut put = |x: &mut [f64; NF], v: f64| {
        x[i] = v;
        i += 1;
    };
    for s in SIDE {
        put(&mut x, term(r, 0, s.1));
    }
    for s in SIDE {
        put(&mut x, term(r, top, s.1));
    }
    let n = opps.len() as f64;
    for s in MEAN {
        put(&mut x, opps.iter().map(|&k| term(r, k, s.1)).sum::<f64>() / n);
    }
    let me_str = term(r, 0, STRENGTH);
    let low = opps.iter().map(|&k| term(r, k, LIFE)).fold(f64::INFINITY, f64::min);
    put(&mut x, n);
    put(&mut x, sq(me_str - term(r, top, STRENGTH)));
    put(&mut x, sq(me_str - opps.iter().map(|&k| term(r, k, STRENGTH)).sum::<f64>() / n));
    put(&mut x, term(r, 0, LIFE) - low);
    put(&mut x, opps.iter().map(|&k| term(r, k, CMD_FROM_ME)).fold(0.0, f64::max));
    put(&mut x, low);
    let t = (r.turns as f64).min(20.0);
    put(&mut x, t);
    put(&mut x, t * term(r, 0, BOARD));
    put(&mut x, t * term(r, 0, HAND));
    put(&mut x, t * term(r, 0, COMBO));
    for s in 1..=3 {
        put(&mut x, f64::from(u8::from(r.seat as usize == s)));
    }
    put(&mut x, sq(r.old as f64));
    put(&mut x, 95.0 * (r.old as f64 / 100.0).tanh());
    Some(x)
}

// ------------------------------------------------------------------ the model
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    pub commit: String,
    pub data_hash: String,
    pub games: u64,
    pub positions: u64,
    pub lambda: f64,
    pub heldout_games: u64,
    pub heldout_logloss: f64,
    pub heldout_brier: f64,
    pub heldout_auc: f64,
    pub baseline_logloss: f64,
    pub baseline_brier: f64,
    pub baseline_auc: f64,
    pub note: String,
}

/// the baseline logistic on today's `evaluate`: p = σ(a + b·evaluate), the mapping's reference
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Map {
    pub a: f64,
    pub b: f64,
}

/// a depth-3 regression tree on the features (boosted on top of the logistic): node k splits on
/// `x[feat[k]] > cut[k]` (right) for k in 0..7, children 2k+1 and 2k+2; `leaf` holds the 8 values (nodes 7..15)
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Tree {
    pub feat: Vec<usize>,
    pub cut: Vec<f64>,
    pub leaf: Vec<f64>,
}

impl Tree {
    pub fn eval(&self, x: &[f64; NF]) -> f64 {
        let mut k = 0;
        for _ in 0..3 {
            k = 2 * k + 1 + usize::from(x[self.feat[k]] > self.cut[k]);
        }
        self.leaf[k - 7]
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub meta: Meta,
    pub map: Map,
    pub bias: f64,
    pub features: Vec<String>,
    pub mean: Vec<f64>,
    pub sd: Vec<f64>,
    pub weight: Vec<f64>,
    /// boosted trees added to the logistic's log-odds (none: the logistic alone)
    #[serde(default)]
    pub tree: Vec<Tree>,
}

impl Model {
    /// the log-odds that the evaluated seat wins
    pub fn logit(&self, x: &[f64; NF]) -> f64 {
        let mut z = self.bias;
        for i in 0..NF {
            z += self.weight[i] * (x[i] - self.mean[i]) / self.sd[i];
        }
        z + self.tree.iter().map(|t| t.eval(x)).sum::<f64>()
    }

    /// `evaluate`'s scale: the value today's `evaluate` gives to the same odds (by the baseline fit
    /// p = σ(a + b·evaluate)), kept below 95 by the same kind of squashing (`95 tanh(e / 95)`: about e near 0)
    pub fn scale(&self, z: f64) -> f64 {
        let e = (z - self.map.a) / self.map.b;
        95.0 * (e / 95.0).tanh()
    }
}

/// parse and check a model file
pub fn parse_model(text: &str) -> Result<Model, String> {
    let m: Model = toml::from_str(text).map_err(|e| format!("model file: {e}"))?;
    let want = names();
    if m.features != want {
        return Err(format!(
            "model file: the features are not the code's list ({} given, {} expected)",
            m.features.len(),
            want.len()
        ));
    }
    if m.mean.len() != NF || m.sd.len() != NF || m.weight.len() != NF {
        return Err(format!("model file: mean, sd and weight need {NF} numbers each"));
    }
    for t in &m.tree {
        if t.feat.len() != 7 || t.cut.len() != 7 || t.leaf.len() != 8 || t.feat.iter().any(|&f| f >= NF) {
            return Err("model file: a tree needs 7 splits (features in range) and 8 leaves".into());
        }
    }
    let trees = m.tree.iter().flat_map(|t| t.cut.iter().chain(&t.leaf));
    let all = m.mean.iter().chain(&m.sd).chain(&m.weight).chain([&m.bias, &m.map.a, &m.map.b]).chain(trees);
    if all.into_iter().any(|x| !x.is_finite()) {
        return Err("model file: a number is not finite".into());
    }
    if m.sd.iter().any(|&s| s <= 0.0) {
        return Err("model file: a scale (sd) is not positive".into());
    }
    if m.map.b <= 0.0 {
        return Err("model file: the mapping's slope b must be positive (monotone)".into());
    }
    Ok(m)
}

const MODEL_TOML: &str = include_str!("../../../../../data/eval/model.toml");

/// the embedded model (checked on first use)
pub fn model() -> &'static Model {
    static M: OnceLock<Model> = OnceLock::new();
    M.get_or_init(|| parse_model(MODEL_TOML).unwrap_or_else(|e| panic!("data/eval/model.toml: {e}")))
}

/// the learned value of a position short of a win or a loss
pub fn value(g: &Game, p: PlayerId) -> f64 {
    let r = raw(g, p);
    match features(&r) {
        Some(x) => {
            let m = model();
            m.scale(m.logit(&x))
        }
        None => search::win_score(g, p),
    }
}

// ------------------------------------------------------------------ the recorder
/// bytes in a record: seed u64, seat u8, label u8, nseats u8, pad u8, turns u16, round u16, old f32, raw f32 x RAW
pub const REC_BYTES: usize = 8 + 4 + 4 + 4 + 4 * RAW;

fn record_dir() -> Option<&'static std::path::Path> {
    static D: OnceLock<Option<std::path::PathBuf>> = OnceLock::new();
    D.get_or_init(|| std::env::var_os("AI_EVAL_RECORD").filter(|s| !s.is_empty()).map(Into::into)).as_deref()
}

thread_local! {
    static BUF: RefCell<Vec<(PlayerId, u16, Raw)>> = const { RefCell::new(Vec::new()) };
    static OUT: RefCell<Option<std::fs::File>> = const { RefCell::new(None) };
}

/// a game begins: drop anything left from a game that didn't finish
pub fn game_start() {
    if record_dir().is_some() {
        BUF.with(|b| b.borrow_mut().clear());
    }
}

/// the end of p's turn in the real game: record p's position (a generic-policy seat, alive, the game going on)
pub fn turn_end(g: &Game, p: PlayerId) {
    if record_dir().is_none() || g.in_search || g.over || !g.player(p).alive {
        return;
    }
    if !super::values::generic_deck(g, p) || g.opps(p).next().is_none() {
        return;
    }
    let r = raw(g, p);
    let round = g.round.min(u16::MAX as u32) as u16;
    BUF.with(|b| b.borrow_mut().push((p, round, r)));
}

/// encode one record
pub fn encode(seed: u64, label: u8, round: u16, r: &Raw, out: &mut Vec<u8>) {
    out.extend_from_slice(&seed.to_le_bytes());
    out.extend_from_slice(&[r.seat, label, r.nseats, 0]);
    out.extend_from_slice(&r.turns.to_le_bytes());
    out.extend_from_slice(&round.to_le_bytes());
    out.extend_from_slice(&r.old.to_le_bytes());
    for x in r.v {
        out.extend_from_slice(&x.to_le_bytes());
    }
}

/// decode one record: (seed, label, round, raw)
pub fn decode(b: &[u8]) -> (u64, u8, u16, Raw) {
    let u16_at = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
    let f32_at = |i: usize| f32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    let seed = u64::from_le_bytes(b[0..8].try_into().expect("8 bytes"));
    let mut v = [0.0f32; RAW];
    for (k, x) in v.iter_mut().enumerate() {
        *x = f32_at(20 + 4 * k);
    }
    let r = Raw { v, seat: b[8], nseats: b[10], turns: u16_at(12), old: f32_at(16) };
    (seed, b[9], u16_at(14), r)
}

/// the game is over: label its records (1: the seat won) and write them to this thread's file
pub fn game_end(g: &Game, seed: u64) {
    let Some(dir) = record_dir() else { return };
    let recs = BUF.with(|b| std::mem::take(&mut *b.borrow_mut()));
    if recs.is_empty() {
        return;
    }
    let mut out = Vec::with_capacity(recs.len() * REC_BYTES);
    for (p, round, r) in &recs {
        let label = u8::from(g.winner == Some(*p));
        encode(seed, label, *round, r, &mut out);
    }
    OUT.with(|f| {
        use std::io::Write;
        let mut f = f.borrow_mut();
        if f.is_none() {
            static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            std::fs::create_dir_all(dir).expect("AI_EVAL_RECORD: can't make the directory");
            let path = dir.join(format!("rec-{}-{n}.bin", std::process::id()));
            *f = Some(std::fs::File::create(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())));
        }
        f.as_mut().expect("opened").write_all(&out).expect("AI_EVAL_RECORD: write failed");
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_count() {
        assert_eq!(names().len(), NF);
    }

    #[test]
    fn modes() {
        assert!(matches!(parse_mode(None), Mode::All));
        assert!(matches!(parse_mode(Some("")), Mode::All));
        assert!(matches!(parse_mode(Some("0")), Mode::Keys(k) if k.is_empty()));
        assert!(matches!(parse_mode(Some("1")), Mode::All));
        assert!(matches!(parse_mode(Some("a, b")), Mode::Keys(k) if k == ["a", "b"]));
    }

    #[test]
    fn record_round_trip() {
        let mut v = [0.0f32; RAW];
        for (i, x) in v.iter_mut().enumerate() {
            *x = i as f32 * 0.5;
        }
        let r = Raw { v, seat: 2, nseats: 4, turns: 7, old: -3.25 };
        let mut b = Vec::new();
        encode(1_000_123, 1, 9, &r, &mut b);
        assert_eq!(b.len(), REC_BYTES);
        let (s, l, round, r2) = decode(&b);
        assert_eq!((s, l, round, r2.seat, r2.nseats, r2.turns, r2.old), (1_000_123, 1, 9, 2, 4, 7, -3.25));
        assert_eq!(r2.v, r.v);
    }
}
