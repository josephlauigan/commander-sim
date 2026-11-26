#!/usr/bin/env python3
"""Replay the kb_guard prompt classifier over JD's own human prompts (read-only).

  replay_history.py                      tier counts over every human prompt in the local transcripts
  replay_history.py --sample 100         also print a seeded sample (hash, tier, text) to hand-label
  replay_history.py --labels FILE        score the sample against labels (lines `HASH<TAB>S|W|N` (HASH = sha1[:8] of the prompt, as printed by --sample): S = the KB must be
                                         consulted, W = card-adjacent, N = not card related)

Transcripts are an internal Claude Code format: absent fields are tolerated. Nothing is written; do not commit
prompt text or labels (they hold personal data).
"""
import argparse
import glob
import hashlib
import json
import os
import random
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kb_guard  # noqa: E402

DEFAULT_DIR = os.path.expanduser("~/.claude/projects/-home-joseph-projects-commander-sim")
SKIP = re.compile(r"^\s*(<command-name>|<local-command|<task-notification|<system-reminder|\[Request interrupted)")


def human_prompts(directory):
    seen = set()
    for path in sorted(glob.glob(os.path.join(directory, "*.jsonl"))):
        try:
            f = open(path, encoding="utf-8")
        except OSError:
            continue
        with f:
            for ln in f:
                try:
                    d = json.loads(ln)
                except Exception:
                    continue
                if d.get("type") != "user" or d.get("isMeta") or (d.get("origin") or {}).get("kind") != "human":
                    continue
                c = (d.get("message") or {}).get("content")
                if isinstance(c, list):
                    c = "\n".join(b.get("text", "") for b in c if isinstance(b, dict) and b.get("type") == "text")
                if not isinstance(c, str) or not c.strip() or SKIP.match(c):
                    continue
                key = d.get("uuid") or (path, c)
                if key in seen:
                    continue
                seen.add(key)
                yield c


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dir", default=DEFAULT_DIR)
    ap.add_argument("--sample", type=int, default=0)
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--labels")
    ap.add_argument("--errors", action="store_true", help="list the false positives and negatives")
    ap.add_argument("--root", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
    a = ap.parse_args()

    decks = kb_guard.deck_words(a.root)
    root = os.path.realpath(a.root)
    roots = [root] + [m for m in [kb_guard.main_checkout(root)] if m and m != root]
    names = kb_guard.load_names(roots)
    print("card names loaded" if names else "no catalog: card names not used")
    rows = [(p,) + kb_guard.classify(p, decks, names) for p in human_prompts(a.dir)]
    counts = {"strong": 0, "weak": 0, "none": 0, "brief": 0}
    for _, tier, brief in rows:
        counts["brief" if brief else tier] += 1
    print(f"human prompts: {len(rows)}; strong {counts['strong']}, weak {counts['weak']}, none {counts['none']}, "
          f"brief-exempt {counts['brief']}")
    # briefs are exempt by design, so the sample is over non-brief prompts
    pool = [r for r in rows if not r[2]]
    by_hash = {hashlib.sha1(p.encode()).hexdigest()[:8]: (p, tier) for p, tier, _ in pool}
    if a.sample:
        for p, tier, _ in random.Random(a.seed).sample(pool, min(a.sample, len(pool))):
            print(f"{hashlib.sha1(p.encode()).hexdigest()[:8]}\t[{tier}]\t{' '.join(p.split())[:220]}")
    if a.labels:
        lab, group = {}, {}
        for ln in open(a.labels, encoding="utf-8"):
            m = re.match(r"^\s*(\w{8})\s+([SWNswn])\b(?:.*# seed (\d+))?", ln)
            if m and m.group(1) in by_hash:
                lab[m.group(1)] = m.group(2).upper()
                group[m.group(1)] = m.group(3) or "-"
        def rate(x, y):
            return f"{x / y:.2f}" if y else "n/a"

        def score(hashes, title):
            tp = fp = fn = stp = sfp = sfn = 0
            for h in hashes:
                tier, truth = by_hash[h][1], lab[h]
                flagged, wanted = tier in ("strong", "weak"), truth in ("S", "W")
                tp += flagged and wanted
                fp += flagged and not wanted
                fn += (not flagged) and wanted
                sflag, swant = tier == "strong", truth == "S"
                stp += sflag and swant
                sfp += sflag and not swant
                sfn += (not sflag) and swant
                if a.errors and flagged != wanted:
                    print(f"  {'FP' if flagged else 'FN'} {h} [{tier}] label {truth}: {' '.join(by_hash[h][0].split())[:100]}")
            print(f"{title}: {len(hashes)} labelled prompts")
            print(f"  strong|weak vs S|W: precision {rate(tp, tp + fp)} (tp {tp}, fp {fp}), recall {rate(tp, tp + fn)} (fn {fn})")
            print(f"  strong vs S:        precision {rate(stp, stp + sfp)} (tp {stp}, fp {sfp}), recall {rate(stp, stp + sfn)} (fn {sfn})")

        for g in sorted(set(group.values())):
            score([h for h in lab if group[h] == g], f"seed {g}")
        score(list(lab), "all")

if __name__ == "__main__":
    main()
