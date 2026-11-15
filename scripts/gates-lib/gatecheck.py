#!/usr/bin/env python3
"""Parsing checks behind scripts/stage-gate.sh. Each subcommand prints a one-line verdict first, then detail lines, and
exits 0 (pass) or 1 (fail). Nothing here runs cargo or a game; it reads files and git work trees.

  suite LOG                              the `test result:` lines of a cargo test log: 0 failed, at least one run
  ignored BASE_LIST BRANCH_LIST BASE_N BRANCH_N
                                         the branch's ignored test names are a subset of the base's; count backstop
  names BASE_LIST BRANCH_LIST [DECLARED] deleted test names equal the declared deletion list exactly
  allowlists BASE_TREE BRANCH_TREE       frozen allow-lists: keys a subset of the base's, reasons unchanged
  newconsts BASE_TREE BRANCH_TREE        no new const named like FIX|ALARM|EXCEPT|GAP|SKIP|EXEMPT|ALLOW|QUIET|NO_
  verdict FILE                           the first line is `VERDICT: commit`
  audit-mine FILE                        every `== deck: N unique cards, M not Full` line has M = 0
  audit-paths FILE                       every `double:` row of the summary is 0 (paths_ok splits are declared, fine)
"""
import os
import re
import sys
from collections import Counter, defaultdict

# the allow-list consts to freeze: file under rust/crates/sim-core/tests/ -> {const: leading literals that form the key}
TESTS_DIR = "rust/crates/sim-core/tests"
FROZEN = {
    "practice_choice_probe.rs": {"TO_FIX": 1, "FALSE_ALARMS": 1},
    "practice_trigger_probe.rs": {"TO_FIX": 1, "FALSE_ALARMS": 1},
    "practice_ability_probe.rs": {"TO_FIX": 1, "FALSE_ALARMS": 1},
    "practice_opponent_probe.rs": {"TO_FIX": 1, "FALSE_ALARMS": 1},
    "practice_rings_probe.rs": {"EXCEPTIONS": 1},
    "spellbook.rs": {"KNOWN_GAPS": 2},  # (deck key, combo variant id): the deck key alone repeats
    "model_gaps_a4.rs": {"NO_CHOICE": 1, "QUIET": 1},
}
SUSPECT_NAME = re.compile(r"FIX|ALARM|EXCEPT|GAP|SKIP|EXEMPT|ALLOW|QUIET|NO_")
CONST_DECL = re.compile(r"^[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?(?:const|static)(?:[ \t]+mut)?[ \t]+([A-Za-z_][A-Za-z0-9_]*)\b", re.M)


def out(ok, head, lines=()):
    print(head)
    for ln in lines:
        print("  " + ln)
    return 0 if ok else 1


# ---------------------------------------------------------------- cargo test logs

TEST_RESULT = re.compile(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored")
RUNNING = re.compile(r"^\s*Running .*\((?:.*/)?([^/()]+?)(?:-[0-9a-f]{16})?\)\s*$")
DOCTESTS = re.compile(r"^\s*Doc-tests\s+(\S+)")
LISTED = re.compile(r"^(.+): (?:test|benchmark)$")


def read(path):
    with open(path, encoding="utf-8", errors="replace") as f:
        return f.read()


def suite(log):
    text = read(log)
    res = TEST_RESULT.findall(text)
    passed = sum(int(r[1]) for r in res)
    failed = sum(int(r[2]) for r in res)
    ignored = sum(int(r[3]) for r in res)
    problems = []
    if not res:
        problems.append("no `test result:` line in the log")
    if failed:
        problems.append(f"{failed} failed")
    bad = [ln for ln in text.splitlines() if ln.startswith("test ") and ln.endswith("FAILED")]
    problems += bad[:20]
    head = f"{failed} failed, {passed} passed, {ignored} ignored over {len(res)} binaries"
    return out(not problems, head, problems)


def listed_names(path):
    """the set of `binary::test` keys of a `cargo test -- --list` log (stderr and stdout merged)"""
    names = set()
    binary = "?"
    for ln in read(path).splitlines():
        m = RUNNING.match(ln)
        if m:
            binary = m.group(1)
            continue
        m = DOCTESTS.match(ln)
        if m:
            binary = "doctest-" + m.group(1)
            continue
        m = LISTED.match(ln)
        if m:
            names.add(f"{binary}::{m.group(1)}")
    return names


def ignored(base_list, branch_list, base_n, branch_n):
    base, branch = listed_names(base_list), listed_names(branch_list)
    extra = sorted(branch - base)
    problems = [f"ignored in the branch only: {n}" for n in extra]
    bn, rn = int(base_n), int(branch_n)
    if rn > bn:
        problems.append(f"`#[ignore` count {rn} is above the base's {bn}")
    head = f"{len(branch)} ignored names (base {len(base)}); #[ignore count {rn} (base {bn})"
    return out(not problems, head, problems)


def declared_list(path):
    if not path:
        return []
    items = []
    for ln in read(path).splitlines():
        ln = ln.strip()
        if ln and not ln.startswith("#"):
            items.append(ln)
    return items


def names(base_list, branch_list, declared=None):
    base, branch = listed_names(base_list), listed_names(branch_list)
    if not base:
        return out(False, "the base list holds no test names", ["was the base list built?"])
    deleted = sorted(base - branch)
    added = sorted(branch - base)
    decl = declared_list(declared)

    def matches(item, key):
        return item == key or item == key.split("::", 1)[1]

    problems = []
    for key in deleted:
        if not any(matches(d, key) for d in decl):
            problems.append(f"deleted and not declared: {key}")
    for d in decl:
        if not any(matches(d, key) for key in deleted):
            problems.append(f"declared but not deleted: {d}")
    head = f"{len(deleted)} deleted (declared {len(decl)}), {len(added)} added; base {len(base)}, branch {len(branch)}"
    return out(not problems, head, problems)


# ---------------------------------------------------------------- Rust allow-list parsing


def skip_trivia(src, i):
    """step over whitespace and comments at i; return the next significant index"""
    n = len(src)
    while i < n:
        if src[i].isspace():
            i += 1
        elif src.startswith("//", i):
            j = src.find("\n", i)
            i = n if j < 0 else j + 1
        elif src.startswith("/*", i):
            depth, i = 1, i + 2
            while i < n and depth:
                if src.startswith("/*", i):
                    depth, i = depth + 1, i + 2
                elif src.startswith("*/", i):
                    depth, i = depth - 1, i + 2
                else:
                    i += 1
        else:
            break
    return i


def read_string(src, i):
    """src[i] starts a string literal; return (raw text between the quotes, index after it)"""
    if src[i] == "r":  # raw string r"..." or r#"..."#
        j = i + 1
        hashes = 0
        while src[j] == "#":
            hashes += 1
            j += 1
        end = '"' + "#" * hashes
        k = src.index(end, j + 1)
        return src[j + 1 : k], k + len(end)
    j = i + 1
    while src[j] != '"':
        j += 2 if src[j] == "\\" else 1
    return src[i + 1 : j], j + 1


def const_entries(src, name, keylen):
    """the entries of `const NAME: &[...] = &[ ... ];` as a list of (key tuple, reason string), or None if absent"""
    m = re.search(r"^[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?const[ \t]+" + re.escape(name) + r"[ \t]*:", src, re.M)
    if not m:
        return None
    i = src.index("=", m.end())
    i = skip_trivia(src, i + 1)
    while src[i] == "&" or src[i].isspace():
        i += 1
    if src[i] != "[":
        raise ValueError(f"{name}: expected `[` after `=`")
    depth, i = 1, i + 1
    entries, tup = [], None
    while depth:
        i = skip_trivia(src, i)
        c = src[i]
        if c == '"' or (c == "r" and src[i + 1] in '"#'):
            lit, i = read_string(src, i)
            if depth == 1:
                entries.append(((lit,), ""))
            elif tup is not None:
                tup.append(lit)
            continue
        if c in "([":
            depth += 1
            if depth == 2 and tup is None:
                tup = []
        elif c in ")]":
            depth -= 1
            if depth == 1 and tup is not None:
                entries.append((tuple(tup[:keylen]), "\x1f".join(tup[keylen:])))
                tup = None
        i += 1
    return entries


def read_consts(tree):
    """{(file, const): [(key, reason)] or None} for every frozen const in a tree"""
    found = {}
    for fname, consts in FROZEN.items():
        path = os.path.join(tree, TESTS_DIR, fname)
        src = read(path) if os.path.exists(path) else ""
        for cname, keylen in consts.items():
            found[(fname, cname)] = const_entries(src, cname, keylen) if src else None
    return found


def allowlists(base_tree, branch_tree):
    base, branch = read_consts(base_tree), read_consts(branch_tree)
    problems, total = [], 0
    for ident in sorted(branch):
        fname, cname = ident
        b_entries, n_entries = base[ident], branch[ident]
        if n_entries is None:
            if b_entries is not None:  # a rename, or `const` turned into `static`, would switch the freeze off
                problems.append(f"{fname} {cname}: the const the base freezes is not found in the branch (renamed or removed?)")
            continue
        total += len(n_entries)
        keys_base = Counter(k for k, _ in (b_entries or []))
        reasons_base = defaultdict(list)
        for k, r in b_entries or []:
            reasons_base[k].append(r)
        seen = Counter()
        for k, r in n_entries:
            seen[k] += 1
            shown = " / ".join(k)
            if seen[k] > keys_base[k]:
                problems.append(f"{fname} {cname}: NEW entry: {shown}")
            elif r not in reasons_base[k]:
                problems.append(f"{fname} {cname}: CHANGED reason: {shown}")
    head = f"{total} frozen entries in {len(branch)} consts, none new or changed"
    return out(not problems, head if not problems else f"{len(problems)} frozen allow-list problems", problems)


def suspect_consts(tree):
    """Counter of (file relative to the tree, const name) for every const/static named like an allow-list"""
    found = Counter()
    root = os.path.join(tree, "rust/crates")
    for dirpath, dirs, files in os.walk(root):
        dirs[:] = [d for d in dirs if d not in ("target", ".git")]
        for f in files:
            if not f.endswith(".rs"):
                continue
            path = os.path.join(dirpath, f)
            for m in CONST_DECL.finditer(read(path)):
                if SUSPECT_NAME.search(m.group(1)):
                    found[(os.path.relpath(path, tree), m.group(1))] += 1
    return found


def newconsts(base_tree, branch_tree):
    base, branch = suspect_consts(base_tree), suspect_consts(branch_tree)
    problems = [f"new const {n} in {p}" for (p, n), c in sorted(branch.items()) if c > base[(p, n)]]
    return out(not problems, f"{sum(branch.values())} suspect-named consts (base {sum(base.values())}), none new", problems)


# ---------------------------------------------------------------- review verdict and audits


def verdict(path, head):
    """the first line is `VERDICT: commit` and a line `Reviewed head: <hash>` as the second line names the commit being gated (a prefix of
    at least 8 hex digits), so a verdict written for an earlier head does not pass a later one"""
    if not os.path.exists(path):
        return out(False, "no review file", [path])
    with open(path, encoding="utf-8", errors="replace") as f:
        text = f.read()
    first = text.split("\n", 1)[0].rstrip("\r").rstrip()
    problems = [] if first == "VERDICT: commit" else ["need exactly `VERDICT: commit`"]
    lines = text.split("\n")
    m = re.match(r"^Reviewed head: ([0-9a-f]{8,40})\s*$", lines[1]) if len(lines) > 1 else None
    if not m:
        problems.append("need a line `Reviewed head: <hash>` naming the reviewed commit")
    elif not head.startswith(m.group(1)):
        problems.append(f"the review is for {m.group(1)}, the head is {head[:12]}")
    return out(not problems, f"first line: {first!r}", problems)


def audit_mine(path):
    rows = re.findall(r"^== (\S+): (\d+) unique cards, (\d+) not Full\s*$", read(path), re.M)
    bad = [f"{d}: {n} not Full" for d, _, n in rows if int(n)]
    if not rows:
        bad.append("no `== deck: N unique cards, M not Full` line")
    return out(not bad, f"{len(rows)} decks, " + ("all Full" if not bad else "some below Full"), bad)


def audit_paths(path):
    rows = re.findall(r"^double: (hard|shadowed|competing|split)\s+(\d+)\s+(\d+)\s*$", read(path), re.M)
    bad = [f"double: {k} = {a} (all cards), {b} (played)" for k, a, b in rows if int(a) or int(b)]
    if len(rows) != 4:
        bad.append(f"expected 4 `double:` rows, found {len(rows)}")
    return out(not bad, "doubles: " + ("none" if not bad else "found"), bad)


COMMANDS = {
    "suite": (suite, 1),
    "ignored": (ignored, 4),
    "names": (names, (2, 3)),
    "allowlists": (allowlists, 2),
    "newconsts": (newconsts, 2),
    "verdict": (verdict, 2),
    "audit-mine": (audit_mine, 1),
    "audit-paths": (audit_paths, 1),
}


def main(argv):
    if len(argv) < 2 or argv[1] not in COMMANDS:
        print(__doc__, file=sys.stderr)
        return 2
    fn, want = COMMANDS[argv[1]]
    args = argv[2:]
    ok = len(args) in want if isinstance(want, tuple) else len(args) == want
    if not ok:
        print(f"gatecheck {argv[1]}: wrong number of arguments", file=sys.stderr)
        return 2
    try:
        return fn(*args)
    except (OSError, ValueError) as e:
        print(f"gatecheck {argv[1]}: error: {e}")
        return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
