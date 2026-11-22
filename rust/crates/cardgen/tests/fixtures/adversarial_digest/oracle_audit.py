"""Independent, offline raw-JSON oracle for digest/map integration tests. Run with python3 -I."""
import collections
import json
import pathlib
import sys
import unicodedata


def fold(s):
    s = s.lower().translate(str.maketrans({"’": "'", "‘": "'", "ʼ": "'", "`": "'", "´": "'", "꞉": ":"}))
    s = s.replace("æ", "ae").replace("œ", "oe").replace("ß", "ss")
    return "".join(c for c in unicodedata.normalize("NFKD", s) if not "\u0300" <= c <= "\u036f").lower().strip()


root = pathlib.Path(sys.argv[1])
cards = json.loads((root / "data/knowledge/oracle.json").read_text())["cards"]
roles = json.loads((root / "data/knowledge/roles.json").read_text())["cards"]
decks = json.loads((root / "data/decks.json").read_text())
covered = [c for c in cards if c["legalities"].get("commander") == "legal"
           and not c.get("all_parts_meld", False)
           and not {"Stickers", "Attraction"}.intersection(c["type_line"].split())]
names = collections.defaultdict(list)
faces = collections.defaultdict(list)
for c in cards:
    names[fold(c["name"])].append(c)
    for f in c.get("card_faces", []):
        faces[fold(f["name"])].append(c)


def candidates(key):
    d = next(d for group in ("mine", "pool") for d in decks[group] if d["key"] == key)
    own = set()
    for n in d["cards"] + [d["commander"]] + ([d["commander2"]] if d.get("commander2") else []):
        own.update(c["oracle_id"] for c in names.get(fold(n), faces.get(fold(n), [])))
    hits = [c for c in covered if set(c["color_identity"]) <= set(d["ident"]) and c["oracle_id"] not in own]
    hits.sort(key=lambda c: (c.get("edhrec_rank", float("inf")), fold(c["name"]), c["name"]))
    # a role page lists every card that has the role in any position, as one list in EDHREC order
    return {"all": [c["name"] for c in hits], "roles": {
        r: [c["name"] for c in hits if r in roles[c["oracle_id"]]["roles"]]
        for r in {a["primary"] for a in roles.values()} if r},
        "roleless": [c["name"] for c in hits if not roles[c["oracle_id"]]["roles"]]}


grid = {}
for c in covered:
    ci = c["color_identity"]
    col = "C" if not ci else ci[0] if len(ci) == 1 else "M"
    a = roles[c["oracle_id"]]
    for r in a["roles"]:
        row = grid.setdefault(r, [0] * 9)
        row["WUBRGCM".index(col)] += 1
        row[7] += 1
        row[8] += r == a["primary"]

keywords = collections.Counter(k for c in covered for k in set(c["keywords"]))
subtypes = collections.Counter()
creature_face_subtypes = set()
for c in covered:
    seen = set()
    for face in c["type_line"].split(" // "):
        if "—" not in face:
            continue
        head, tail = face.split("—", 1)
        words = {w.replace("Time_Lord", "Time Lord") for w in tail.replace("Time Lord", "Time_Lord").split()}
        seen.update(words)
        if "Creature" in head.split():
            creature_face_subtypes.update(words)
    subtypes.update(seen)
spell = []
for path in sorted((root / "data/spellbook").glob("*.json")):
    v = json.loads(path.read_text())
    spell.append([v.get("deck", path.stem), v.get("identity", "-"), v.get("fetched", "-")]
                 + [str(len(v.get(k, []))) for k in ("included", "included_by_changing_commanders", "almost_included")])
print(json.dumps({
    "legalities": dict(collections.Counter(c["legalities"].get("commander", "unknown") for c in cards)),
    "identities": dict(collections.Counter("".join(k for k in "WUBRG" if k in c["color_identity"]) or "C" for c in covered)),
    "game_changers": sorted({c["name"] for c in cards if c.get("game_changer", False)}),
    "banned": sorted({c["name"] for c in cards if c["legalities"].get("commander") == "banned"}),
    "grid": grid, "covered": len(covered),
    "with_role": sum(bool(roles[c["oracle_id"]]["roles"]) for c in covered),
    "routed": sum(roles[c["oracle_id"]]["needs_llm"] for c in covered),
    "keywords": sorted(keywords.items(), key=lambda x: (-x[1], x[0]))[:60],
    "subtypes": sorted(subtypes.items(), key=lambda x: (-x[1], x[0]))[:40],
    "subtype_vocabulary": len(subtypes),
    "creature_face_subtype_vocabulary": len(creature_face_subtypes),
    "spellbook": spell,
    "candidates": {k: candidates(k) for k in ("cloud", "thanos", "thrasios", "lathril-golgari-elves")},
}))
