#!/usr/bin/env python3
"""Compare two recordings made by scripts/trace-record.sh. Reads files only; runs no game.

  trace-compare.py BASE_DIR BRANCH_DIR [--rewrite 'old -> new' ...] [--expect-missing deck/tier ...] [--allow deck ...]
                   [--changed-cards 'A;B'] [--explained 'A;B']

l2 recordings (full records and logs), one class per (deck|within, tier, seed):
  SAME    sha_full equal
  DIAG    only sha_play equal (the evaluation lines changed, play did not)
  PLAY    same result, different actions: the first differing line number and both lines
  RESULT  winner, wintype or last round differ
  ERROR   a game exited non-zero in either recording
  MISSING in the base only (a failure unless declared with --expect-missing deck/tier, or just `deck`)
  NEW     in the branch only
--rewrite 'old -> new' (a Python regex and its replacement; split at the first ' -> ', or at the first '->' if the
text has no spaced arrow) is applied to every line of the base logs before hashing, for a deliberate wording change.
Exit non-zero if any record whose deck is not in --allow is not SAME (declared MISSING excepted). Every allowed non-SAME
record is listed: each needs a one-line reason in the PR.

l3 recordings (summary mode): panics, the timeout count and the no-winner count per deck-tier on the same seeds against the
base, each seed that newly times out, and the cast count per card. No win rates are read or printed. Fails on any panic in
the branch, a timeout or no-winner count above the base's, an unexpected MISSING, and a --changed-cards card with no cast
in the branch (or none left where the base had some) unless named in --explained.
"""
import argparse
import collections
import json
import os
import re
import sys

DECIDES = re.compile(r"^R\d+\s+\[.* decides: ")
CLASSES = ["SAME", "DIAG", "PLAY", "RESULT", "ERROR", "MISSING", "NEW"]


def load(d):
    recs = {}
    path = os.path.join(d, "records.jsonl")
    if not os.path.exists(path):
        sys.exit(f"trace-compare: no records.jsonl in {d}")
    with open(path) as f:
        for ln in f:
            if ln.strip():
                r = json.loads(ln)
                recs[(r["deck"], r["tier"], r["k"])] = r
    meta = {}
    mp = os.path.join(d, "meta.json")
    if os.path.exists(mp):
        meta = json.load(open(mp))
    return recs, meta


def is_l3(recs, meta):
    return meta.get("mode") == "l3" or any("casts" in r for r in recs.values())


def tag(key):
    return f"{key[0]}/{key[1]}"


def parse_rewrite(spec):
    if " -> " in spec:
        old, new = spec.split(" -> ", 1)
    elif "->" in spec:
        old, new = spec.split("->", 1)
    else:
        sys.exit(f"trace-compare: bad --rewrite (need 'old -> new'): {spec}")
    return re.compile(old), new


def read_lines(d, rel, rewrites=()):
    with open(os.path.join(d, rel), "rb") as f:
        text = f.read().decode("utf-8", "surrogateescape")
    lines = text.splitlines(keepends=True)
    for rx, new in rewrites:
        lines = [rx.sub(new, ln) for ln in lines]
    return lines


def hashes(lines):
    import hashlib

    full = hashlib.sha256("".join(lines).encode("utf-8", "surrogateescape")).hexdigest()
    play = hashlib.sha256("".join(ln for ln in lines if not DECIDES.match(ln)).encode("utf-8", "surrogateescape")).hexdigest()
    return full, play


def first_difference(a, b):
    """(line number in a, line number in b, line a, line b) of the first difference between the play lines"""
    pa = [(i + 1, ln) for i, ln in enumerate(a) if not DECIDES.match(ln)]
    pb = [(i + 1, ln) for i, ln in enumerate(b) if not DECIDES.match(ln)]
    for n in range(max(len(pa), len(pb))):
        x = pa[n] if n < len(pa) else (len(a) + 1, "<end of log>\n")
        y = pb[n] if n < len(pb) else (len(b) + 1, "<end of log>\n")
        if x[1] != y[1]:
            return x[0], y[0], x[1].rstrip("\n"), y[1].rstrip("\n")
    return None


def classify(key, b, r, base_dir, branch_dir, rewrites, expect):
    """(class, detail lines)"""
    if r is None:
        return "MISSING", []
    if b is None:
        return "NEW", []
    if b["exit"] != 0 or r["exit"] != 0:
        return "ERROR", [f"base exit {b['exit']}: {b.get('stderr1', '')}", f"branch exit {r['exit']}: {r.get('stderr1', '')}"]
    blines = None
    b_full, b_play = b["sha_full"], b["sha_play"]
    if rewrites:
        blines = read_lines(base_dir, b["log"], rewrites)
        b_full, b_play = hashes(blines)
    if b_full == r["sha_full"]:
        return "SAME", []
    if b_play == r["sha_play"]:
        return "DIAG", []
    res = lambda x: (x["winner"], x["wintype"], x["last_round"])
    if res(b) != res(r):
        return "RESULT", [f"base   {res(b)}", f"branch {res(r)}"]
    blines = blines or read_lines(base_dir, b["log"])
    rlines = read_lines(branch_dir, r["log"])
    d = first_difference(blines, rlines)
    if d is None:
        return "PLAY", ["play lines equal but hashes differ"]
    return "PLAY", [f"first difference at base line {d[0]}, branch line {d[1]}", f"base:   {d[2]}", f"branch: {d[3]}"]


def expected_missing(key, expect):
    return tag(key) in expect or key[0] in expect


def compare_l2(base, branch, a):
    rewrites = [parse_rewrite(s) for s in a.rewrite]
    expect, allow = set(a.expect_missing), set(a.allow)
    keys = sorted(set(base[0]) | set(branch[0]))
    counts = collections.defaultdict(collections.Counter)
    failing, allowed, declared = [], [], []
    for key in keys:
        b, r = base[0].get(key), branch[0].get(key)
        cls, detail = classify(key, b, r, a.base, a.branch, rewrites, expect)
        counts[tag(key)][cls] += 1
        if cls == "SAME":
            continue
        if cls == "MISSING" and expected_missing(key, expect):
            declared.append(key)
        elif key[0] in allow and cls != "ERROR":
            allowed.append((key, cls, detail))
        else:
            failing.append((key, cls, detail))

    print(f"base   {a.base}: commit {base[1].get('commit', '?')[:12]}, binary {base[1].get('binary_sha256', '?')[:16]}")
    print(f"branch {a.branch}: commit {branch[1].get('commit', '?')[:12]}, binary {branch[1].get('binary_sha256', '?')[:16]}")
    if rewrites:
        print("rewrites applied to the base logs: " + "; ".join(a.rewrite))
    print(f"\n{'deck/tier':<28}" + "".join(f"{c:>8}" for c in CLASSES))
    total = collections.Counter()
    for t in sorted(counts):
        print(f"{t:<28}" + "".join(f"{counts[t][c]:>8}" for c in CLASSES))
        total.update(counts[t])
    print(f"{'total':<28}" + "".join(f"{total[c]:>8}" for c in CLASSES))
    if declared:
        print(f"\nexpected MISSING (declared): {len(declared)} records in {sorted({tag(k) for k in declared})}")

    def show(title, rows):
        print(f"\n{title}")
        for key, cls, detail in rows:
            print(f"  {cls:<7} {tag(key)} seed {key[2]}")
            for d in detail:
                print(f"      {d}")

    if allowed:
        show("allowed non-SAME records (each needs a one-line reason):", allowed)
    if failing:
        show("FAILING records:", failing)
    ok = not failing
    if not keys:
        print("\nno records in either recording")
        ok = False
    print(f"\nRESULT: {'PASS' if ok else 'FAIL'} ({len(failing)} failing, {len(allowed)} allowed non-SAME, "
          f"{total['SAME']} of {sum(total.values())} SAME)")
    return 0 if ok else 1


def compare_l3(base, branch, a):
    expect = set(a.expect_missing)
    changed = [c.strip() for c in ";".join(a.changed_cards).split(";") if c.strip()]
    explained = {c.strip() for c in ";".join(a.explained).split(";") if c.strip()}
    keys_b, keys_r = set(base[0]), set(branch[0])
    problems = []
    unexpected = sorted(k for k in keys_b - keys_r if not expected_missing(k, expect))
    declared = [k for k in keys_b - keys_r if expected_missing(k, expect)]
    for k in unexpected:
        problems.append(f"MISSING {tag(k)} seed {k[2]}")
    if not (keys_b & keys_r):
        problems.append("no seed is in both recordings")
    has_own = lambda recs: any("own_casts" in r for r in recs[0].values())
    if has_own(base) != has_own(branch):
        problems.append("only one recording has own-seat cast counts (made before they were recorded): re-record the older one")
    print(f"base   {a.base}: commit {base[1].get('commit', '?')[:12]}")
    print(f"branch {a.branch}: commit {branch[1].get('commit', '?')[:12]}")
    print(f"same seeds: {len(keys_b & keys_r)}; only in the base {len(keys_b - keys_r)} "
          f"({len(declared)} declared, {len(unexpected)} not); only in the branch {len(keys_r - keys_b)}")

    by = collections.defaultdict(list)
    for k in sorted(keys_b & keys_r):
        by[tag(k)].append((base[0][k], branch[0][k]))
    isbad = lambda r: r["exit"] != 0 or bool(r.get("panic"))
    print(f"\n{'deck/tier':<26}{'seeds':>6}{'panic b/r':>11}{'timeout b/r':>13}{'no-winner b/r':>15}")
    cast_rows = collections.defaultdict(lambda: collections.defaultdict(lambda: [0, 0]))
    own_rows = collections.defaultdict(lambda: collections.defaultdict(lambda: [0, 0]))
    newly = []
    for t in sorted(by):
        pairs = by[t]
        pb, pr = sum(isbad(b) for b, _ in pairs), sum(isbad(r) for _, r in pairs)
        tb, tr = sum(b["timeout"] for b, _ in pairs), sum(r["timeout"] for _, r in pairs)
        nb, nr = sum(not b["winner_present"] for b, _ in pairs), sum(not r["winner_present"] for _, r in pairs)
        print(f"{t:<26}{len(pairs):>6}{f'{pb}/{pr}':>11}{f'{tb}/{tr}':>13}{f'{nb}/{nr}':>15}")
        if pr:
            for _, r in pairs:
                if isbad(r):
                    problems.append(f"PANIC {t} seed {r['k']}: exit {r['exit']}: {r.get('panic', '')}")
        if tr > tb:
            problems.append(f"timeouts up in {t}: base {tb}, branch {tr} of {len(pairs)}")
        if nr > nb:
            problems.append(f"no-winner games up in {t}: base {nb}, branch {nr} of {len(pairs)}")
        for b, r in pairs:
            if r["timeout"] and not b["timeout"]:
                newly.append(f"{t} seed {r['k']}")
            for c, v in b.get("casts", {}).items():
                cast_rows[c][t][0] += v
            for c, v in r.get("casts", {}).items():
                cast_rows[c][t][1] += v
            for c, v in b.get("own_casts", {}).items():
                own_rows[c][t][0] += v
            for c, v in r.get("own_casts", {}).items():
                own_rows[c][t][1] += v
    if newly:
        print("\nseeds that newly time out: " + ", ".join(newly))
    if own_rows:
        print("\ngames where the deck's OWN seat casts the card (base / branch; the exact card name):")
        for c in sorted(own_rows):
            tb = sum(v[0] for v in own_rows[c].values())
            tr = sum(v[1] for v in own_rows[c].values())
            per = ", ".join(f"{t} {v[0]}/{v[1]}" for t, v in sorted(own_rows[c].items()) if v[0] or v[1])
            print(f"  {c}: total {tb}/{tr}" + (f"  [{per}]" if per else ""))
    if cast_rows:
        print("\ngames with a `casts <card>` line, any seat, exact card name (base / branch):")
        for c in sorted(cast_rows):
            tb = sum(v[0] for v in cast_rows[c].values())
            tr = sum(v[1] for v in cast_rows[c].values())
            per = ", ".join(f"{t} {v[0]}/{v[1]}" for t, v in sorted(cast_rows[c].items()) if v[0] or v[1])
            print(f"  {c}: total {tb}/{tr}" + (f"  [{per}]" if per else ""))
    # the deck's own seat is what shows the card was played; recordings without own-seat counts fall back to any seat
    check_rows = own_rows if own_rows else cast_rows
    for c in changed:
        if c in explained:
            continue
        if c not in check_rows:
            problems.append(f"changed card {c!r} is not in the recording (was --cards given?)")
            continue
        tr = sum(v[1] for v in check_rows[c].values())
        if tr == 0:
            problems.append(f"changed card {c!r} was never cast by a deck's own seat in the branch" if own_rows else f"changed card {c!r} was never cast in the branch")
        for t, v in sorted(check_rows[c].items()):
            if v[0] and not v[1]:
                problems.append(f"changed card {c!r}: cast in {v[0]} base games of {t}, none in the branch")
    if problems:
        print("\nPROBLEMS:")
        for p in problems:
            print("  " + p)
    print(f"\nRESULT: {'PASS' if not problems else 'FAIL'} ({len(problems)} problems)")
    return 0 if not problems else 1


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("base")
    ap.add_argument("branch")
    ap.add_argument("--rewrite", action="append", default=[], metavar="'old -> new'")
    ap.add_argument("--expect-missing", nargs="+", action="extend", default=[], metavar="deck/tier")
    ap.add_argument("--allow", nargs="+", action="extend", default=[], metavar="deck")
    ap.add_argument("--changed-cards", action="append", default=[], metavar="'A;B'")
    ap.add_argument("--explained", action="append", default=[], metavar="'A;B'")
    a = ap.parse_args()
    base, branch = load(a.base), load(a.branch)
    l3 = (is_l3(*base), is_l3(*branch))
    if l3[0] != l3[1]:
        sys.exit("trace-compare: one recording is l3 (summary) and the other is not")
    return compare_l3(base, branch, a) if l3[0] else compare_l2(base, branch, a)


if __name__ == "__main__":
    sys.exit(main())
