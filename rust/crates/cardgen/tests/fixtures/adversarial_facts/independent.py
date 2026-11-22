"""Independent phase-4 reference checks, invoked only with python3 -I. Standard library, offline, read-only.

Production names and deck keys come exclusively from the current exported data. The fixtures and assertions in
the Rust driver supply synthetic cases. This oracle deliberately does not import or invoke cardgen helpers.
"""

import bisect
import collections
import json
from pathlib import Path
import re
import sys
import unicodedata


mode, root, artifacts = sys.argv[1], Path(sys.argv[2]), Path(sys.argv[3])


def read(path):
    return json.loads(path.read_text())


def folded(text, lower=True):
    replacements = {"’": "'", "‘": "'", "ʼ": "'", "´": "'", "꞉": ":",
                    "Æ": "Ae", "æ": "ae", "Œ": "Oe", "œ": "oe", "ß": "ss"}
    if lower:
        text = text.lower().replace("`", "'")
    text = "".join(replacements.get(c, c) for c in text)
    result = "".join(c for c in unicodedata.normalize("NFKD", text) if not unicodedata.combining(c))
    return result.lower().strip() if lower else result


catalog = read(root / "data/knowledge/oracle.json")["cards"]
decks = read(root / "data/decks.json")["mine"]
full_names, face_names = collections.defaultdict(list), collections.defaultdict(list)
by_id = {c["oracle_id"]: c for c in catalog}
for c in catalog:
    full_names[folded(c["name"])].append(c["oracle_id"])
    for face in c.get("card_faces", []):
        if folded(face["name"]) != folded(c["name"]):
            face_names[folded(face["name"])].append(c["oracle_id"])


def resolve(name):
    hits = full_names.get(folded(name)) or face_names.get(folded(name), [])
    legal = [i for i in hits if by_id[i]["legalities"].get("commander") == "legal"]
    return set(legal or hits)


members = {}
for d in decks:
    names = d["cards"] + [d["commander"], d.get("commander2", "")]
    members[d["key"]] = set().union(*(resolve(n) for n in names if n.strip()))

rows = read(artifacts / "rows.json")
by_row = {r["oracle_id"]: r for r in rows}
assert len(rows) == len(catalog) == len(by_row), (len(rows), len(catalog), len(by_row))
recorded = set().union(*(resolve(c["name"]) for c in read(root / "data/cards.json")["cards"]))


def findings_reference():
    """Scan one literal name at a time, independent from the Rust first-token trie and its alias construction.

    Real files contain ordinary one-backtick spans; Markdown fence/complex-span semantics are attacked separately
    in synthetic tests. A name is not matched across physical lines. Duplicate aliases/occurrences collapse by line.
    """
    aliases = collections.defaultdict(set)
    for c in catalog:
        for name in [c["name"]] + [f["name"] for f in c.get("card_faces", [])]:
            aliases[folded(name, False)].update(resolve(name))
    found = collections.defaultdict(dict)
    files = []
    for d in decks:
        rel = f"audit/{d['key']}/FINDINGS.md"
        file = root / rel
        if not file.exists():
            continue
        lines = file.read_text().splitlines()
        norm = [folded(line, False) for line in lines]
        joined = "\n".join(norm)
        starts = [0]
        for line in norm:
            starts.append(starts[-1] + len(line) + 1)
        headings, heading = [], ""
        for line in lines:
            headings.append(heading)
            if line.startswith("#"):
                heading = line[:200]
        files.append(rel)
        cands = collections.defaultdict(list)
        for name, ids in aliases.items():
            if not name:
                continue
            # Alphanumeric boundary (underscore is punctuation per the specified contract).
            pattern = r"(?<![^\W_])" + re.escape(name)
            if name[-1].isalnum():
                pattern += r"(?![^\W_])"
            for match in re.finditer(pattern, joined):
                n = bisect.bisect_right(starts, match.start()) - 1
                if len(name) <= 4 and "=>" not in norm[n]:
                    pos, end = match.start() - starts[n], match.end() - starts[n]
                    spans = list(re.finditer(r"\*\*(.*?)\*\*|`([^`]*)`", norm[n]))
                    if not any(any(m.start(g) <= pos and end <= m.end(g) for g in (1, 2)
                                   if m.start(g) >= 0) for m in spans):
                        continue
                cands[n].append((match.start() - starts[n], match.end() - starts[n], ids))
        # round 3 (documented rule): at each position the longest name wins and names inside it are skipped
        for n, matches in cands.items():
            entry = {"file": rel, "line": n + 1, "heading": headings[n], "text": lines[n][:200]}
            skip = 0
            for start in sorted({m[0] for m in matches}):
                if start < skip:
                    continue
                best = max(m[1] for m in matches if m[0] == start)
                skip = best
                for m in matches:
                    if m[0] == start and m[1] == best:
                        for i in m[2]:
                            found[i][(files.index(rel), n)] = entry
    return {i: [entry for _, entry in sorted(entries.items())] for i, entries in found.items()}


if mode == "rows":
    expected_findings = findings_reference()
    for i, c in by_id.items():
        r = by_row[i]
        assert isinstance(r["decks"], list) and isinstance(r["findings"], list), r
        assert r["decks"] == [d["key"] for d in decks if i in members[d["key"]]], r
        assert ("status" in r) == (i in recorded), r
        assert ("status_note" in r) == (i in recorded), r
        assert r["findings"] == expected_findings.get(i, []), (c["name"], r["findings"], expected_findings.get(i, []))
        for k in ("name", "oracle_id", "layout", "mana_cost", "cmc", "type_line", "identity", "legal", "gc",
                  "edhrec_rank", "usd", "oracle_text", "keywords", "set", "released_at", "rarity", "reserved"):
            assert k in r, (c["name"], k)
    safe_count = 0
    for d in decks:
        own = read(artifacts / f"own-{d['key']}.json")
        assert {r["oracle_id"] for r in own} == members[d["key"]], d["key"]
        safe = read(artifacts / f"safe-{d['key']}.json")
        for r in safe:
            i = r["oracle_id"]
            assert r["legal"] == "legal" and set(r["identity"]) <= set(d["ident"]), (d["key"], r)
            assert i not in members[d["key"]] and r["status"] in ("Full", "Full-auto"), (d["key"], r)
        safe_count += len(safe)
    print(json.dumps({"rows": len(rows), "decks": len(decks), "engine_cards": len(recorded),
                      "findings": sum(len(v) for v in expected_findings.values()), "safe_candidates": safe_count}))

elif mode == "views":
    files = sorted(artifacts.glob("digest-*.txt"))
    assert len(files) == len(decks) + 32
    all_lines = 0
    sizes = {}
    pattern = re.compile(r"(?:\* )?(?:(?:BAN|NL) )?(?:GC )?(?:#[0-9]+ )?(?:\$[0-9]+ )?"
                         r"(?:m:[FfAPUN] )?(?:d:[A-Z]+ )?(?:k:[1-9][0-9]* )?(?:\? )?")
    for path in files:
        raw = path.read_bytes()
        text = raw.decode()
        assert len(raw) < 200_000 and (len(raw) + 3) // 4 <= 42_000, (path.name, len(raw))
        model = [l for l in text.splitlines() if l.startswith("# m: ")]
        letter_header = [l for l in text.splitlines() if l.startswith("# d: ")]
        assert len(model) == 1 and len(letter_header) == bool(decks), path.name
        letters = dict(re.findall(r"([A-Z])=(\S+)", letter_header[0])) if decks else {}
        assert len(letters) == len(decks) and set(letters.values()) == set(members)
        current = path.stem.removeprefix("digest-")
        section = None
        for line in text.splitlines():
            if line.startswith("## "):
                section = line[3]
            if not line or line.startswith("#"):
                continue
            all_lines += 1
            fields = line.split("|")
            assert len(fields) == 6 and all(fields) and len(line) <= 160, line
            flags = fields[-1]
            assert flags == "-" or pattern.fullmatch(flags + " "), line
            if "…" in fields[0]:
                continue  # A shortened display name cannot be unambiguously resolved without its Oracle ID.
            ids = resolve(fields[0])
            assert len(ids) == 1, (path.name, fields[0], ids)
            i = next(iter(ids))
            tokens = flags.split()
            m = [t[2:] for t in tokens if t.startswith("m:")]
            expected_m = {"Full": "F", "Full-auto": "f", "Approximate": "A", "Partial": "P", "Unmodeled": "U"}
            # round 3: m:N = no engine record
            assert m == ([expected_m[by_row[i]["status"]]] if i in recorded else ["N"]), line
            k = [int(t[2:]) for t in tokens if t.startswith("k:")]
            expected_k = len(by_row[i]["findings"])
            assert k == ([expected_k] if expected_k else []), line
            dflags = [t[2:] for t in tokens if t.startswith("d:")]
            expected_d = "".join(letter for letter, key in letters.items()
                                 if i in members[key] and key != current)
            assert dflags == ([expected_d] if expected_d else []), (path.name, section, line, expected_d)
        sizes[path.stem] = len(raw)
    text = (artifacts / "map.md").read_text()
    assert len(text.encode()) < 32768
    cover = [c["oracle_id"] for c in catalog if c["legalities"].get("commander") == "legal"
             and not c.get("all_parts_meld") and not set(c["type_line"].split()) & {"Stickers", "Attraction"}]
    expected = collections.Counter(by_row[i].get("status", "no record") for i in cover)
    section = text.split("## Engine model coverage", 1)[1].split("\n## ", 1)[0]
    actual = {s: int(n.replace(",", "")) for s, n in re.findall(
        r"^\| (Full|Full-auto|Approximate|Partial|Unmodeled|no record) \| ([0-9,]+) \|$", section, re.M)}
    assert actual == dict(expected), (actual, expected)
    for d in decks:
        row = next(l for l in text.splitlines() if l.startswith(f"| {d['key']} |"))
        cells = [c.strip() for c in row.split("|")[1:-1]]
        below, breakdown, norecord, mentions = cells[-4], cells[-3], int(cells[-2]), int(cells[-1])  # round 3 columns
        own = [i for i in members[d["key"]] if not set(by_id[i]["type_line"].split()) & {"Stickers", "Attraction"}]
        counts = [sum(by_row[i].get("status") == s for i in own)
                  for s in ("Full-auto", "Approximate", "Partial", "Unmodeled")]
        assert int(below) == sum(counts) and breakdown == "f%d A%d P%d U%d" % tuple(counts), d["key"]
        assert norecord == sum("status" not in by_row[i] for i in own), d["key"]
        assert mentions == sum(len(by_row[i]["findings"]) for i in own), d["key"]
    print(json.dumps({"digests": len(files), "lines": all_lines, "largest_digest": max(sizes.values()),
                      "map_bytes": len(text.encode()), "engine_coverage": actual}))
else:
    raise AssertionError(mode)
