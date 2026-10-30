//! --- phase4-eval: fit the learned evaluation from recorded positions (`audit/phase4/eval-design.md`).
//!
//! `cargo run --release -p sim-core --example eval_train -- DATA_DIR OUT.toml COMMIT [--gbt]`
//!
//! DATA_DIR holds one directory per tier (`t1` .. `t5`), each with the recorder's `rec-*.bin` files
//! (`AI_EVAL_RECORD`). Games with seed % 5 == 0 are held out; of the rest, seed % 5 == 1 picks the penalty, then the
//! model is refitted on all of them. Every (game, seat) weighs 1 in total (its positions share it). Scores on the
//! held-out games: log loss, Brier and AUC, for the model and for today's `evaluate` through a fitted logistic.
//! `--gbt` also boosts depth-3 trees on the logistic's log-odds (rounds picked on the validation games); they are
//! kept in the model only if they beat the logistic alone by more than 0.005 held-out log loss.

use sim_core::ai::learned::{self, Map, Meta, Model, NF, REC_BYTES};
use std::collections::HashMap;

struct Data {
    x: Vec<[f64; NF]>,
    y: Vec<f64>,
    w: Vec<f64>,
    old: Vec<f64>,
    tier: Vec<u8>,
    seed: Vec<u64>,
}

fn load(dir: &str) -> (Data, String, u64) {
    let mut d = Data { x: vec![], y: vec![], w: vec![], old: vec![], tier: vec![], seed: vec![] };
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut groups: HashMap<(u8, u64, u8), u32> = HashMap::new();
    let mut key = vec![];
    for t in 1..=5u8 {
        let td = std::path::Path::new(dir).join(format!("t{t}"));
        let Ok(rd) = std::fs::read_dir(&td) else { continue };
        let mut files: Vec<_> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        files.retain(|p| p.extension().is_some_and(|e| e == "bin"));
        files.sort();
        for f in files {
            let b = std::fs::read(&f).expect("read a record file");
            for &c in &b {
                hash = (hash ^ c as u64).wrapping_mul(0x100000001b3);
            }
            assert_eq!(b.len() % REC_BYTES, 0, "{}: not whole records", f.display());
            for rec in b.chunks_exact(REC_BYTES) {
                let (seed, label, _round, r) = learned::decode(rec);
                let Some(x) = learned::features(&r) else { continue };
                d.x.push(x);
                d.y.push(f64::from(label));
                d.old.push(r.old as f64);
                d.tier.push(t);
                d.seed.push(seed);
                let k = (t, seed, r.seat);
                *groups.entry(k).or_default() += 1;
                key.push(k);
            }
        }
    }
    d.w = key.iter().map(|k| 1.0 / groups[k] as f64).collect();
    let games = groups.keys().map(|k| (k.0, k.1)).collect::<std::collections::HashSet<_>>().len() as u64;
    (d, format!("{hash:016x}"), games)
}

fn sigmoid(z: f64) -> f64 {
    1.0 / (1.0 + (-z).exp())
}

/// weighted log loss, Brier and AUC of probabilities p on (y, w)
fn scores(p: &[f64], y: &[f64], w: &[f64]) -> (f64, f64, f64) {
    let (mut ll, mut br, mut sw) = (0.0, 0.0, 0.0);
    for i in 0..p.len() {
        let q = p[i].clamp(1e-12, 1.0 - 1e-12);
        ll -= w[i] * (y[i] * q.ln() + (1.0 - y[i]) * (1.0 - q).ln());
        br += w[i] * (q - y[i]).powi(2);
        sw += w[i];
    }
    // AUC: the weighted chance a winner's position scores above a loser's (ties count half)
    let mut idx: Vec<usize> = (0..p.len()).collect();
    idx.sort_by(|&a, &b| p[a].total_cmp(&p[b]));
    let (mut neg, mut auc, mut pos_w, mut neg_w) = (0.0, 0.0, 0.0, 0.0);
    let mut i = 0;
    while i < idx.len() {
        let mut j = i;
        let (mut tp, mut tn) = (0.0, 0.0);
        while j < idx.len() && p[idx[j]] == p[idx[i]] {
            let k = idx[j];
            if y[k] > 0.5 {
                tp += w[k]
            } else {
                tn += w[k]
            }
            j += 1;
        }
        auc += tp * (neg + 0.5 * tn);
        neg += tn;
        pos_w += tp;
        neg_w += tn;
        i = j;
    }
    (ll / sw, br / sw, auc / (pos_w * neg_w))
}

/// a weighted L2 logistic regression on rows z (first column 1, not penalised), by Newton's method
fn fit_logistic(z: &[Vec<f64>], y: &[f64], w: &[f64], lambda: f64) -> Vec<f64> {
    let d = z[0].len();
    let sw: f64 = w.iter().sum();
    let mut beta = vec![0.0; d];
    let threads = 22;
    for _it in 0..50 {
        let chunk = z.len().div_ceil(threads);
        let parts: Vec<(Vec<f64>, Vec<f64>)> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..threads)
                .map(|t| {
                    let beta = &beta;
                    s.spawn(move || {
                        let mut g = vec![0.0; d];
                        let mut h = vec![0.0; d * d];
                        let lo = (t * chunk).min(z.len());
                        let hi = ((t + 1) * chunk).min(z.len());
                        for i in lo..hi {
                            let r = &z[i];
                            let e: f64 = r.iter().zip(beta).map(|(a, b)| a * b).sum();
                            let p = sigmoid(e);
                            let c = w[i] * (p - y[i]);
                            let v = w[i] * p * (1.0 - p);
                            for a in 0..d {
                                g[a] += c * r[a];
                                let va = v * r[a];
                                for b in a..d {
                                    h[a * d + b] += va * r[b];
                                }
                            }
                        }
                        (g, h)
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().expect("thread")).collect()
        });
        let mut g = vec![0.0; d];
        let mut h = vec![0.0; d * d];
        for (pg, ph) in parts {
            for a in 0..d {
                g[a] += pg[a] / sw;
            }
            for k in 0..d * d {
                h[k] += ph[k] / sw;
            }
        }
        for a in 0..d {
            for b in 0..a {
                h[a * d + b] = h[b * d + a];
            }
        }
        for a in 1..d {
            g[a] += lambda * beta[a];
            h[a * d + a] += lambda;
        }
        for a in 0..d {
            h[a * d + a] += 1e-10;
        }
        let step = solve(&mut h, &mut g, d);
        // step halving: keep the penalised loss going down
        let obj = |b: &[f64]| -> f64 {
            let mut l = 0.0;
            for i in 0..z.len() {
                let e: f64 = z[i].iter().zip(b).map(|(a, c)| a * c).sum();
                // log(1 + e^e) - y e, stable
                l += w[i] * (e.max(0.0) + (-e.abs()).exp().ln_1p() - y[i] * e);
            }
            l / sw + 0.5 * lambda * b[1..].iter().map(|x| x * x).sum::<f64>()
        };
        let now = obj(&beta);
        let mut t = 1.0;
        let mut next: Vec<f64>;
        loop {
            next = beta.iter().zip(&step).map(|(b, s)| b - t * s).collect();
            if obj(&next) <= now + 1e-12 || t < 1e-4 {
                break;
            }
            t *= 0.5;
        }
        let mx = beta.iter().zip(&next).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
        beta = next;
        if mx < 1e-8 {
            break;
        }
    }
    beta
}

/// solve H s = g (H symmetric positive definite) by Gaussian elimination with partial pivoting
fn solve(h: &mut [f64], g: &mut [f64], d: usize) -> Vec<f64> {
    for c in 0..d {
        let piv = (c..d).max_by(|&a, &b| h[a * d + c].abs().total_cmp(&h[b * d + c].abs())).expect("rows");
        if piv != c {
            for k in 0..d {
                h.swap(c * d + k, piv * d + k);
            }
            g.swap(c, piv);
        }
        let pv = h[c * d + c];
        for r in c + 1..d {
            let f = h[r * d + c] / pv;
            if f != 0.0 {
                for k in c..d {
                    h[r * d + k] -= f * h[c * d + k];
                }
                g[r] -= f * g[c];
            }
        }
    }
    let mut s = vec![0.0; d];
    for c in (0..d).rev() {
        let mut v = g[c];
        for k in c + 1..d {
            v -= h[c * d + k] * s[k];
        }
        s[c] = v / h[c * d + c];
    }
    s
}

fn dot(r: &[f64], b: &[f64]) -> f64 {
    r.iter().zip(b).map(|(a, c)| a * c).sum()
}

// ------------------------------------------------------------------ boosted trees on top of the logistic
/// a tree in bins: node k splits on bin > `split[k].1` of feature `split[k].0` (255: no split, all go left)
type BinTree = ([(usize, u8); 7], [f64; 8]);

fn leaf_of(t: &BinTree, r: &[u8]) -> usize {
    let mut k = 0;
    for _ in 0..3 {
        let (j, b) = t.0[k];
        k = 2 * k + 1 + usize::from(r[j] > b);
    }
    k - 7
}

fn wlogloss(f: &[f64], y: &[f64], w: &[f64]) -> f64 {
    let (mut l, mut sw) = (0.0, 0.0);
    for i in 0..f.len() {
        l += w[i] * (f[i].max(0.0) + (-f[i].abs()).exp().ln_1p() - y[i] * f[i]);
        sw += w[i];
    }
    l / sw
}

/// second-order gradient boosting of depth-3 trees, from the margins f (the logistic's log-odds); the margins of
/// the evaluation rows `ev` are updated alongside, and their log loss after each round is returned
#[allow(clippy::too_many_arguments)]
fn gbt(
    xb: &[Vec<u8>],
    f: &mut [f64],
    y: &[f64],
    w: &[f64],
    rounds: usize,
    eta: f64,
    ev: &[Vec<u8>],
    fe: &mut [f64],
    (ye, we): (&[f64], &[f64]),
) -> (Vec<BinTree>, Vec<f64>) {
    let n = xb.len();
    let d = xb[0].len();
    let lam = 1.0;
    // a leaf needs this much hessian weight (in (game, seat) units: a few hundred seat-games)
    let min_h = 0.01 * w.iter().sum::<f64>() / 100.0;
    let mut trees = vec![];
    let mut curve = vec![];
    let mut node = vec![0usize; n];
    for _ in 0..rounds {
        let gr: Vec<f64> = (0..n).map(|i| w[i] * (sigmoid(f[i]) - y[i])).collect();
        let he: Vec<f64> = (0..n).map(|i| w[i] * sigmoid(f[i]) * (1.0 - sigmoid(f[i]))).collect();
        let mut t: BinTree = ([(0, 255); 7], [0.0; 8]);
        node.iter_mut().for_each(|x| *x = 0);
        for depth in 0..3 {
            let first = (1 << depth) - 1;
            let count = 1 << depth;
            let mut hg = vec![0.0; count * d * 32];
            let mut hh = vec![0.0; count * d * 32];
            for i in 0..n {
                let k = node[i] - first;
                for j in 0..d {
                    let b = xb[i][j] as usize;
                    hg[(k * d + j) * 32 + b] += gr[i];
                    hh[(k * d + j) * 32 + b] += he[i];
                }
            }
            for k in 0..count {
                let (mut best, mut bf, mut bt) = (0.0, 0, 255u8);
                for j in 0..d {
                    let o = (k * d + j) * 32;
                    let (gt, ht): (f64, f64) = (hg[o..o + 32].iter().sum(), hh[o..o + 32].iter().sum());
                    let (mut gl, mut hl) = (0.0, 0.0);
                    for b in 0..31 {
                        gl += hg[o + b];
                        hl += hh[o + b];
                        let (g2, hr) = (gt - gl, ht - hl);
                        if hl < min_h || hr < min_h {
                            continue;
                        }
                        let gain = gl * gl / (hl + lam) + g2 * g2 / (hr + lam) - gt * gt / (ht + lam);
                        if gain > best {
                            (best, bf, bt) = (gain, j, b as u8);
                        }
                    }
                }
                t.0[first + k] = (bf, bt);
            }
            for i in 0..n {
                let (j, b) = t.0[node[i]];
                node[i] = 2 * node[i] + 1 + usize::from(xb[i][j] > b);
            }
        }
        let (mut lg, mut lh) = ([0.0; 8], [0.0; 8]);
        for i in 0..n {
            lg[node[i] - 7] += gr[i];
            lh[node[i] - 7] += he[i];
        }
        for l in 0..8 {
            t.1[l] = -eta * lg[l] / (lh[l] + lam);
        }
        for i in 0..n {
            f[i] += t.1[node[i] - 7];
        }
        for (i, r) in ev.iter().enumerate() {
            fe[i] += t.1[leaf_of(&t, r)];
        }
        curve.push(wlogloss(fe, ye, we));
        trees.push(t);
    }
    (trees, curve)
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() < 3 {
        eprintln!("usage: eval_train DATA_DIR OUT.toml COMMIT [--gbt]");
        std::process::exit(2);
    }
    let (d, hash, games) = load(&a[0]);
    let n = d.x.len();
    println!("positions {n}, games {games}, data hash {hash}");
    let held: Vec<bool> = d.seed.iter().map(|s| s % 5 == 0).collect();
    let val: Vec<bool> = d.seed.iter().map(|s| s % 5 == 1).collect();
    let tr: Vec<usize> = (0..n).filter(|&i| !held[i]).collect();
    let te: Vec<usize> = (0..n).filter(|&i| held[i]).collect();
    let heldout_games = d.seed.iter().zip(&d.tier).filter(|(s, _)| *s % 5 == 0).map(|(s, t)| (*s, *t));
    let heldout_games = heldout_games.collect::<std::collections::HashSet<_>>().len() as u64;

    // standardise on the fitted games
    let sw: f64 = tr.iter().map(|&i| d.w[i]).sum();
    let mut mean = vec![0.0; NF];
    let mut sd = vec![0.0; NF];
    for &i in &tr {
        for j in 0..NF {
            mean[j] += d.w[i] * d.x[i][j] / sw;
        }
    }
    for &i in &tr {
        for j in 0..NF {
            sd[j] += d.w[i] * (d.x[i][j] - mean[j]).powi(2) / sw;
        }
    }
    for s in sd.iter_mut() {
        *s = s.sqrt().max(1e-6);
    }
    let row = |i: usize| -> Vec<f64> {
        let mut r = Vec::with_capacity(NF + 1);
        r.push(1.0);
        for j in 0..NF {
            r.push((d.x[i][j] - mean[j]) / sd[j]);
        }
        r
    };
    let pick = |ix: &[usize], f: &dyn Fn(usize) -> f64| ix.iter().map(|&i| f(i)).collect::<Vec<f64>>();
    let ytr = pick(&tr, &|i| d.y[i]);
    let wtr = pick(&tr, &|i| d.w[i]);
    let yte = pick(&te, &|i| d.y[i]);
    let wte = pick(&te, &|i| d.w[i]);

    // the baselines: today's evaluate (and its raw term) through a fitted logistic
    let ev = |i: usize| 95.0 * (d.old[i] / 100.0).tanh();
    let one = |ix: &[usize], f: &dyn Fn(usize) -> f64| ix.iter().map(|&i| vec![1.0, f(i)]).collect::<Vec<_>>();
    let be = fit_logistic(&one(&tr, &ev), &ytr, &wtr, 0.0);
    let pb: Vec<f64> = te.iter().map(|&i| sigmoid(be[0] + be[1] * ev(i))).collect();
    let sb = scores(&pb, &yte, &wte);
    println!(
        "baseline evaluate: a {:.4} b {:.5}  held-out logloss {:.4} brier {:.4} auc {:.4}",
        be[0], be[1], sb.0, sb.1, sb.2
    );

    // the penalty, picked on the validation games (seed % 5 == 1) of a fit on the others
    let fit_ix: Vec<usize> = tr.iter().copied().filter(|&i| !val[i]).collect();
    let val_ix: Vec<usize> = tr.iter().copied().filter(|&i| val[i]).collect();
    let zf: Vec<Vec<f64>> = fit_ix.iter().map(|&i| row(i)).collect();
    let zv: Vec<Vec<f64>> = val_ix.iter().map(|&i| row(i)).collect();
    let (yf, wf) = (pick(&fit_ix, &|i| d.y[i]), pick(&fit_ix, &|i| d.w[i]));
    let (yv, wv) = (pick(&val_ix, &|i| d.y[i]), pick(&val_ix, &|i| d.w[i]));
    let mut best = (f64::INFINITY, 0.0);
    for lambda in [1e-5, 1e-4, 1e-3, 3e-3, 1e-2, 3e-2, 1e-1] {
        let b = fit_logistic(&zf, &yf, &wf, lambda);
        let p: Vec<f64> = zv.iter().map(|r| sigmoid(dot(r, &b))).collect();
        let s = scores(&p, &yv, &wv);
        println!("lambda {lambda:e}: validation logloss {:.5} brier {:.5} auc {:.4}", s.0, s.1, s.2);
        if s.0 < best.0 {
            best = (s.0, lambda);
        }
    }
    let lambda = best.1;
    let ztr: Vec<Vec<f64>> = tr.iter().map(|&i| row(i)).collect();
    let zte: Vec<Vec<f64>> = te.iter().map(|&i| row(i)).collect();
    let beta = fit_logistic(&ztr, &ytr, &wtr, lambda);
    let pl: Vec<f64> = zte.iter().map(|r| sigmoid(dot(r, &beta))).collect();
    let sl = scores(&pl, &yte, &wte);
    println!("learned (lambda {lambda:e}): held-out logloss {:.4} brier {:.4} auc {:.4}", sl.0, sl.1, sl.2);
    for t in 1..=5u8 {
        let k: Vec<usize> = (0..te.len()).filter(|&j| d.tier[te[j]] == t).collect();
        if k.is_empty() {
            continue;
        }
        let g = |v: &[f64]| k.iter().map(|&j| v[j]).collect::<Vec<f64>>();
        let (a, b) = (scores(&g(&pb), &g(&yte), &g(&wte)), scores(&g(&pl), &g(&yte), &g(&wte)));
        println!(
            "  t{t}: baseline logloss {:.4} brier {:.4} auc {:.4} | learned logloss {:.4} brier {:.4} auc {:.4} ({} positions)",
            a.0,
            a.1,
            a.2,
            b.0,
            b.1,
            b.2,
            k.len()
        );
    }
    let names = learned::names();
    let mut order: Vec<usize> = (0..NF).collect();
    order.sort_by(|&a, &b| beta[b + 1].abs().total_cmp(&beta[a + 1].abs()));
    println!("weights (standardised), largest first:");
    for j in order {
        println!("  {:<16} {:+.4}", names[j], beta[j + 1]);
    }

    let mut trees: Vec<learned::Tree> = vec![];
    let mut chosen = sl;
    let mut note = format!("logistic alone: held-out logloss {:.4} brier {:.4} auc {:.4}", sl.0, sl.1, sl.2);
    if a.iter().any(|s| s == "--gbt") {
        // bins: 32 quantiles of each feature on the fitted games
        let mut cuts = vec![vec![]; NF];
        for j in 0..NF {
            let mut v: Vec<f64> = tr.iter().map(|&i| d.x[i][j]).collect();
            v.sort_by(f64::total_cmp);
            cuts[j] = (1..32).map(|q| v[q * (v.len() - 1) / 32]).collect::<Vec<f64>>();
        }
        let bin = |i: usize| -> Vec<u8> {
            (0..NF).map(|j| cuts[j].iter().filter(|&&c| d.x[i][j] > c).count() as u8).collect()
        };
        // the rounds: boosted on the fit games from their logistic, scored on the validation games
        let bf = fit_logistic(&zf, &yf, &wf, lambda);
        let mut f_fit: Vec<f64> = zf.iter().map(|r| dot(r, &bf)).collect();
        let mut f_val: Vec<f64> = zv.iter().map(|r| dot(r, &bf)).collect();
        let xb_fit: Vec<Vec<u8>> = fit_ix.iter().map(|&i| bin(i)).collect();
        let xb_val: Vec<Vec<u8>> = val_ix.iter().map(|&i| bin(i)).collect();
        let start = wlogloss(&f_val, &yv, &wv);
        let (_, curve) = gbt(&xb_fit, &mut f_fit, &yf, &wf, 600, 0.05, &xb_val, &mut f_val, (&yv, &wv));
        let (r_best, l_best) =
            curve.iter().enumerate().fold((0, start), |b, (k, &l)| if l < b.1 { (k + 1, l) } else { b });
        println!("trees on the logistic: validation logloss {start:.5} -> {l_best:.5} at {r_best} rounds (of 600)");
        if r_best > 0 {
            let xbt: Vec<Vec<u8>> = tr.iter().map(|&i| bin(i)).collect();
            let xbe: Vec<Vec<u8>> = te.iter().map(|&i| bin(i)).collect();
            let mut f_tr: Vec<f64> = ztr.iter().map(|r| dot(r, &beta)).collect();
            let mut f_te: Vec<f64> = zte.iter().map(|r| dot(r, &beta)).collect();
            let (bt, _) = gbt(&xbt, &mut f_tr, &ytr, &wtr, r_best, 0.05, &xbe, &mut f_te, (&yte, &wte));
            let pg: Vec<f64> = f_te.iter().map(|&z| sigmoid(z)).collect();
            let sg = scores(&pg, &yte, &wte);
            println!("logistic + {r_best} trees: held-out logloss {:.4} brier {:.4} auc {:.4}", sg.0, sg.1, sg.2);
            note = format!("{note}; logistic + {r_best} trees: logloss {:.4} brier {:.4} auc {:.4}", sg.0, sg.1, sg.2);
            if sl.0 - sg.0 > 0.005 {
                println!("the trees beat the logistic by more than 0.005: kept");
                chosen = sg;
                trees = bt
                    .iter()
                    .map(|t| learned::Tree {
                        feat: t.0.iter().map(|s| s.0).collect(),
                        cut: t
                            .0
                            .iter()
                            .map(|s| if (s.1 as usize) < 31 { cuts[s.0][s.1 as usize] } else { f64::MAX })
                            .collect(),
                        leaf: t.1.to_vec(),
                    })
                    .collect();
            } else {
                println!("the trees don't beat the logistic by 0.005: the logistic alone");
            }
        }
    }

    let model = Model {
        meta: Meta {
            commit: a[2].clone(),
            data_hash: hash,
            games,
            positions: n as u64,
            lambda,
            heldout_games,
            heldout_logloss: chosen.0,
            heldout_brier: chosen.1,
            heldout_auc: chosen.2,
            baseline_logloss: sb.0,
            baseline_brier: sb.1,
            baseline_auc: sb.2,
            note: format!(
                "fitted by examples/eval_train.rs; held out: seed % 5 == 0; baseline: evaluate through a fitted logistic; {note}"
            ),
        },
        map: Map { a: be[0], b: be[1] },
        bias: beta[0],
        features: names,
        mean,
        sd,
        weight: beta[1..].to_vec(),
        tree: trees,
    };
    learned::parse_model(&toml::to_string(&model).expect("serialise")).expect("the fitted model checks");
    let text = format!(
        "# the learned evaluation's weights (audit/phase4/eval-design.md), written by examples/eval_train.rs\n{}",
        toml::to_string(&model).expect("serialise")
    );
    std::fs::write(&a[1], text).expect("write the model");
    // the mapped scale on the held-out positions, beside today's
    let mapped: Vec<f64> = te.iter().map(|&i| model.scale(model.logit(&d.x[i]))).collect();
    let olds: Vec<f64> = te.iter().map(|&i| ev(i)).collect();
    let m = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let (mo, mn) = (m(&olds), m(&mapped));
    let cov: f64 = olds.iter().zip(&mapped).map(|(a, b)| (a - mo) * (b - mn)).sum::<f64>();
    let vo: f64 = olds.iter().map(|a| (a - mo).powi(2)).sum::<f64>();
    let vn: f64 = mapped.iter().map(|b| (b - mn).powi(2)).sum::<f64>();
    println!(
        "held-out evaluate: today mean {mo:.2} sd {:.2}; learned mean {mn:.2} sd {:.2}; correlation {:.3}",
        (vo / olds.len() as f64).sqrt(),
        (vn / olds.len() as f64).sqrt(),
        cov / (vo * vn).sqrt()
    );
    println!("wrote {}", a[1]);
}
