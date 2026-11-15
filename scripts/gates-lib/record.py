#!/usr/bin/env python3
"""The worker behind scripts/trace-record.sh: plays the recorded games and writes the records. Run only by the
coordinator, inside the cargo lock and the memory cap (trace-record.sh does not take them itself).

One game per process: `env -i PATH HOME COMMANDER_SIM_ROOT=<tree> <copied binary> --deck D --pool T --seed 500000
--trace k` for a deck-tier, `--calibrate within --pool T --seed 500000 --trace k` (no --deck) for a within-tier pod. Output:
  <out>/records.jsonl   one JSON line per (deck|within, tier, k), sorted
  <out>/logs/*.log      the raw stdout of every l2 game (l3 keeps no logs)
  <out>/meta.json       mode, commit, binary sha256, seeds, timing
l3 lines hold only: exit code, panic text, timeout flag, winner-present flag, last round, and per card whether a
"casts <card>" line appeared. They never hold a winner's name, so no win share can be computed from them.
"""
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import time
import tomllib
from concurrent.futures import ThreadPoolExecutor

SEED0 = 500000
TIERS = ["t1", "t2", "t3", "t4", "t5", "t6"]
DECIDES = re.compile(rb"^R\d+\s+\[.* decides: ")
ROUND = re.compile(rb"^R(\d+)\s")
WINNER = re.compile(rb"^Winner: (.*) \((.*)\)\s*$")


def sha(data):
    return hashlib.sha256(data).hexdigest()


CAST_LINE = re.compile(rb"^R\d+\s+(.+?) casts (.*)$")


def load_names(tree):
    """card names (for the exact-name rule) and deck key -> seat display name, read from the tree's data"""
    names, seat = [], {}
    cj = json.load(open(os.path.join(tree, "data", "cards.json")))
    it = cj["cards"] if isinstance(cj, dict) else cj
    names = [c["name"] for c in it if isinstance(c, dict) and c.get("name")]
    dj = json.load(open(os.path.join(tree, "data", "decks.json")))
    for v in dj.get("mine", []):
        seat[v["key"]] = v.get("name", "")
    return names, seat


def cast_flags(out, cards, own, names):
    """per card: 1 if some seat cast exactly that card (not a longer card name that starts with it), and 1 if the deck's own
    seat did (`own` is the seat's display name, '' when unknown)"""
    anyc = {c: 0 for c in cards}
    ownc = {c: 0 for c in cards}
    longer = {c: [n.encode() for n in names if n != c and n.startswith(c) and not n[len(c) : len(c) + 1].isalnum()] for c in cards}
    for ln in out.splitlines():
        m = CAST_LINE.match(ln)
        if not m:
            continue
        actor, rest = m.group(1).strip(), m.group(2)
        if b": " in actor:  # "Scarlet Witch: Y'shtola casts X": the seat is after the card's prefix
            actor = actor.rsplit(b": ", 1)[1].strip()
        for c in cards:
            cb = c.encode()
            if not rest.startswith(cb):
                continue
            tail = rest[len(cb) : len(cb) + 1]
            if tail.isalnum() or tail == b"'":
                continue
            if any(rest.startswith(l) for l in longer[c]):
                continue
            anyc[c] = 1
            if own and actor == own.encode():
                ownc[c] = 1
    return anyc, ownc


def parse_game(out):
    """seat order, winner, wintype, last round, line count, sha_full and sha_play of one game's stdout (bytes)"""
    lines = out.splitlines(keepends=True)
    seat = winner = wintype = None
    last = 0
    play = []
    for ln in lines:
        if ln.startswith(b"Seat order: ") and seat is None:
            seat = ln[len(b"Seat order: ") :].decode("utf-8", "replace").strip()
        m = ROUND.match(ln)
        if m:
            last = max(last, int(m.group(1)))
        m = WINNER.match(ln)
        if m:
            winner = m.group(1).decode("utf-8", "replace")
            wintype = m.group(2).decode("utf-8", "replace")
        if not DECIDES.match(ln):
            play.append(ln)
    return {
        "seat_order": seat,
        "winner": winner,
        "wintype": wintype,
        "last_round": last,
        "lines": len(lines),
        "sha_full": sha(out),
        "sha_play": sha(b"".join(play)),
    }


def play(args, env, tree, timeout):
    try:
        p = subprocess.run(args, env=env, cwd=tree, capture_output=True, timeout=timeout, stdin=subprocess.DEVNULL)
        return p.returncode, p.stdout, p.stderr
    except subprocess.TimeoutExpired:
        return -9, b"", f"killed after {timeout}s".encode()


def first_line(b):
    for ln in b.decode("utf-8", "replace").splitlines():
        if ln.strip():
            return ln.strip()
    return ""


def deck_tiers(tree):
    with open(os.path.join(tree, "data/my_decks.toml"), "rb") as f:
        return {k: v["tiers"] for k, v in tomllib.load(f).items() if isinstance(v, dict) and "tiers" in v}


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--mode", required=True, choices=["l2", "l3", "goldfish"])
    ap.add_argument("--tree", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--bin", required=True, help="the copied commander-sim binary")
    ap.add_argument("--commit", required=True)
    ap.add_argument("--seeds", type=int)
    ap.add_argument("--decks", default="", help="comma list of deck keys; `within` selects the within-tier pods")
    ap.add_argument("--cards", default="", help="l3: card names, separated by ';'")
    ap.add_argument("-P", type=int, default=16, dest="par")
    ap.add_argument("--timeout", type=int, default=900, help="seconds per game")
    a = ap.parse_args()

    binary_sha = sha(open(a.bin, "rb").read())
    env = {"PATH": os.environ.get("PATH", ""), "HOME": os.environ.get("HOME", ""), "COMMANDER_SIM_ROOT": a.tree}
    os.makedirs(os.path.join(a.out, "logs"), exist_ok=True)
    wanted = [d for d in a.decks.split(",") if d]
    cards = [c.strip() for c in a.cards.split(";") if c.strip()]
    t0 = time.time()
    meta = {"mode": a.mode, "commit": a.commit, "binary_sha256": binary_sha, "tree": a.tree, "parallel": a.par,
            "decks": wanted, "cards": cards, "started": time.strftime("%F %T")}

    if a.mode == "goldfish":
        key = wanted[0] if wanted else "urdragon"
        games = a.seeds or 500
        rc, out, err = play([a.bin, "--goldfish", key, "--games", str(games), "--seed", str(SEED0)], env, a.tree, 3600)
        with open(os.path.join(a.out, "goldfish.txt"), "wb") as f:
            f.write(out)
        text = out.decode("utf-8", "replace")
        m = re.search(r"Kill turn \(own turn\): median (\S+), mean (\S+) \(over the (\d+) kills\)", text)
        routes = {}
        m2 = re.search(r"^How it won \(share of kills\): (.*)$", text, re.M)
        if m2 and m2.group(1).strip() != "-":
            routes = {k: float(v) for k, v in re.findall(r"(.+?) (\d+(?:\.\d+)?)%(?:, |$)", m2.group(1))}
        rec = {"commit": a.commit, "binary_sha256": binary_sha, "deck": key, "games": games, "seed": SEED0, "exit": rc,
               "stderr1": first_line(err), "median": m.group(1) if m else None, "mean": m.group(2) if m else None,
               "kills": int(m.group(3)) if m else None, "routes": routes}
        with open(os.path.join(a.out, "goldfish.json"), "w") as f:
            json.dump(rec, f, indent=1)
        meta["elapsed_s"] = round(time.time() - t0, 1)
        json.dump(meta, open(os.path.join(a.out, "meta.json"), "w"), indent=1)
        print(f"goldfish {key}: exit {rc}, median {rec['median']}, mean {rec['mean']}, routes {routes} ({meta['elapsed_s']}s)")
        return 0 if rc == 0 and m else 1

    seeds = a.seeds or (20 if a.mode == "l2" else 200)
    tiers_of = deck_tiers(a.tree)
    jobs = []  # (deck, tier)
    for deck, tiers in tiers_of.items():
        if not wanted or deck in wanted:
            jobs += [(deck, t) for t in tiers]
    if a.mode == "l2" and (not wanted or "within" in wanted):
        jobs += [("within", t) for t in TIERS]
    if not jobs:
        print("trace-record: no deck-tier selected", file=sys.stderr)
        return 2
    meta["seeds"] = seeds
    meta["deck_tiers"] = [f"{d}/{t}" for d, t in jobs]

    try:
        card_names, seat_names = load_names(a.tree)
    except (OSError, ValueError, KeyError) as e:
        print(f"record.py: cannot read data/cards.json or data/decks.json in {a.tree}: {e}", file=sys.stderr)
        return 2
    if a.mode == "l3" and a.cards and not card_names:
        print("record.py: --cards given but data/cards.json holds no card names", file=sys.stderr)
        return 2

    def one(job):
        deck, tier, k = job
        cmd = [a.bin] + (["--deck", deck] if deck != "within" else ["--calibrate", "within"]) + ["--pool", tier, "--seed", str(SEED0), "--trace", str(k)]
        rc, out, err = play(cmd, env, a.tree, a.timeout)
        rec = {"commit": a.commit, "binary_sha256": binary_sha, "deck": deck, "tier": tier, "k": k, "seed": SEED0 + k, "exit": rc}
        info = parse_game(out)
        if a.mode == "l2":
            rec["stderr1"] = first_line(err)
            rec.update(info)
            rel = f"logs/{deck}-{tier}-{k}.log"
            with open(os.path.join(a.out, rel), "wb") as f:
                f.write(out)
            rec["log"] = rel
        else:
            rec["panic"] = first_line(err) if rc != 0 or b"panic" in err else ""
            rec["timeout"] = (info["wintype"] or "").lower() == "timeout"
            rec["winner_present"] = bool(info["winner"]) and info["winner"].lower() != "none"
            rec["last_round"] = info["last_round"]
            anyc, ownc = cast_flags(out, cards, seat_names.get(deck, "") if deck != "within" else "", card_names)
            rec["casts"] = anyc
            rec["own_casts"] = ownc
        return rec

    todo = [(d, t, k) for d, t in jobs for k in range(seeds)]
    records, done = [], 0
    with ThreadPoolExecutor(max_workers=a.par) as ex:
        for rec in ex.map(one, todo):
            records.append(rec)
            done += 1
            if done % 100 == 0:
                print(f"  {done}/{len(todo)} games, {time.time() - t0:.0f}s", file=sys.stderr)
    records.sort(key=lambda r: (r["deck"], r["tier"], r["k"]))
    with open(os.path.join(a.out, "records.jsonl"), "w") as f:
        for r in records:
            f.write(json.dumps(r, sort_keys=True) + "\n")
    elapsed = time.time() - t0
    meta["elapsed_s"] = round(elapsed, 1)
    json.dump(meta, open(os.path.join(a.out, "meta.json"), "w"), indent=1)
    bad = sum(1 for r in records if r["exit"] != 0)
    print(f"{a.mode}: {len(records)} games in {elapsed:.0f}s ({len(records) / max(elapsed, 1e-9):.1f} games/s at -P {a.par}); "
          f"{bad} with a non-zero exit; binary sha256 {binary_sha[:16]}; records {a.out}/records.jsonl")
    return 0


if __name__ == "__main__":
    sys.exit(main())
