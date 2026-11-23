#!/usr/bin/env python3
"""L2 check for the Cali format-data PR (C2): data/decks.json differs from the base version only by the new `formats`
table and kyodai's `format` and `tiers`; `formats` equals data/formats.toml.

    python3 -I scripts/tests/decks_json_l2.py [--base REF] [--tree DIR]

REF is the commit holding the old decks.json (default origin/main, before this PR); DIR is the checkout holding the
new one (default: the repository this script is in). Exit 0 = all hold. A one-time check for the PR that introduced
`formats`: once it is merged the base already has them, so it will (rightly) fail.
"""
import argparse
import json
import subprocess
import sys
import tomllib
from pathlib import Path

ap = argparse.ArgumentParser()
ap.add_argument("--base", default="origin/main")
ap.add_argument("--tree", default=str(Path(__file__).resolve().parents[2]))
a = ap.parse_args()
tree = Path(a.tree)

old = json.loads(subprocess.run(["git", "-C", str(tree), "show", f"{a.base}:data/decks.json"], check=True,
                                capture_output=True, text=True).stdout)
new = json.loads((tree / "data/decks.json").read_text())
fmts = tomllib.loads((tree / "data/formats.toml").read_text())

bad = []


def check(ok, what):
    if not ok:
        bad.append(what)
    print(("ok   " if ok else "FAIL ") + what)


check(set(new) == set(old) | {"formats"} and "formats" not in old, "top-level keys = the old keys plus formats")
check(new["version"] == old["version"] == 1, "version stays 1")
check(len(new["pool"]) == len(old["pool"]) and all(x == y for x, y in zip(new["pool"], old["pool"])),
      f"pool equal element by element ({len(old['pool'])} decks)")
om = {d["key"]: d for d in old["mine"]}
nm = {d["key"]: d for d in new["mine"]}
check(list(nm) == list(om), f"mine has the same decks in the same order ({', '.join(nm)})")
for k, d in nm.items():
    if k == "kyodai":
        continue
    check(d == om[k], f"mine {k} unchanged")
want = dict(om["kyodai"], format="cali", tiers=["t5", "t6"])
check(nm["kyodai"] == want, "kyodai differs only by format = cali and tiers = [t5, t6]")
check({k for k in nm["kyodai"] if nm["kyodai"][k] != om["kyodai"].get(k)} == {"format", "tiers"},
      "the fields that differ on kyodai are exactly format and tiers")
check(all("format" not in d for k, d in nm.items() if k != "kyodai"), "only kyodai carries a format")
check(all("format" not in d for d in new["pool"]), "no pool deck carries a format")
want_f = {k: {f: v for f, v in t.items()} for k, t in fmts.items()}
check(new["formats"] == want_f, "formats equals data/formats.toml (commander has no banned list; cali has 18 names)")
check(len(new["formats"]["cali"]["banned"]) == 18, "cali.banned has 18 names")
sys.exit(1 if bad else 0)
