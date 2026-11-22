"""Independent report oracle: use JSON sets/maps, without importing cardgen code."""
import json
import pathlib
import sys

p = pathlib.Path(sys.argv[1])
current = json.loads((p / 'out/oracle.json').read_text())['cards']
previous = json.loads((p / 'out/oracle.prev.json').read_text())['cards']
decks = json.loads((p / 'clone/data/decks.json').read_text())['mine']
roles = json.loads((p / 'out/roles.json').read_text())['cards']
cache = json.loads((p / 'clone/data/scryfall_cache.json').read_text())
report = (p / 'report.md').read_text()
sections = {}
for block in report.split('\n## ')[1:]:
    title, body = block.split('\n', 1)
    sections[title] = body
assert list(sections) == ['Action needed', 'Run', 'New cards',
                          'Bans/unbans and Game Changer changes', 'Role changes and LLM stats']
old = {c['oracle_id']: c for c in previous}
new = {c['oracle_id']: c for c in current}
fresh = [c for k, c in new.items() if k not in old]
gone = [c for k, c in old.items() if k not in new]
legal = lambda c: c.get('legalities', {}).get('commander', 'unknown')
rank = lambda c: (c.get('edhrec_rank', 2**32-1), c['name'])
cap = lambda names, n: ', '.join(names[:n]) + (f' ({len(names)-n} more)' if len(names)>n else '')
assert (f"{len(fresh)} new Oracle cards ({sum(legal(c)=='legal' for c in fresh)} "
        f"Commander-legal), {len(gone)} gone from the catalog.") in sections['New cards']
for deck in decks:
    inside = sorted([c for c in fresh if legal(c)=='legal'
                     and set(c['color_identity']) <= set(deck['ident'])], key=rank)
    rows = []
    for c in inside:
        tags = roles.get(c['oracle_id'], {}).get('roles', [])
        rows.append(c['name'] + (' [' + ', '.join(tags) + ']' if tags else ''))
    line = f"- `{deck['key']}` ({deck['ident'] or 'colourless'}): {len(rows)} new"
    if rows:
        line += ': ' + cap(rows, 40)
    assert line in sections['New cards'], line
sets = {}
for c in fresh:
    sets.setdefault(c['set'], []).append(c)
for name, cards in sorted(sets.items(), key=lambda pair: (-len(pair[1]), pair[0]))[:30]:
    allowed = sorted([c for c in cards if legal(c)=='legal'], key=rank)
    line = f"- **{name or '(no set)'}**: {len(cards)} new, {len(allowed)} Commander-legal"
    if allowed:
        line += ': ' + cap([c['name'] for c in allowed], 25)
    assert line in sections['New cards'], line
changed = sorted([(old[k], c) for k,c in new.items() if k in old], key=lambda pair:(pair[1]['name'],pair[1]['oracle_id']))
bans = [(a,b) for a,b in changed if legal(a)!=legal(b)]
gc = [(a,b) for a,b in changed if a.get('game_changer',False)!=b.get('game_changer',False)]
catalog = sections['Bans/unbans and Game Changer changes']
for label, rows in [
    ('Banned', [f"{b['name']} ({legal(a)} -> {legal(b)})" for a,b in bans if legal(b)=='banned']),
    ('Unbanned', [f"{b['name']} ({legal(a)} -> {legal(b)})" for a,b in bans if legal(a)=='banned']),
    ('Other legality changes', [f"{b['name']} ({legal(a)} -> {legal(b)})" for a,b in bans
                                if legal(a)!='banned' and legal(b)!='banned']),
    ('Game Changer added', [b['name'] for a,b in gc if b.get('game_changer',False)]),
    ('Game Changer removed', [b['name'] for a,b in gc if not b.get('game_changer',False)]),
]:
    line = f"- {label}: {cap(rows,40) if rows else 'none'}"
    assert line in catalog, line
action = sections['Action needed']
for deck in decks:
    names = set(deck['cards'] + [deck['commander']] + ([deck['commander2']] if deck.get('commander2') else []))
    for a,b in bans:
        if b['name'] in names and legal(a)=='legal' and legal(b)!='legal':
            assert f"Deck `{deck['key']}`: **{b['name']}** became {legal(b).replace('_',' ')}" in action
    for a,b in gc:
        if b['name'] in names:
            assert f"{b['name']} {'joined' if b.get('game_changer',False) else 'left'} the Game Changer list" in action
    # Independent Oracle-text comparison for fixture and real-deck cache matches.
    differences = []
    for c in current:
        if c['name'] in names and c['name'] in cache:
            before = cache[c['name']].get('oracle_text', '')
            after = c.get('oracle_text', '')
            if before != after:
                differences.append(c['name'])
    for name in differences[:40]:
        assert name in action, name
assert '## Role changes and LLM stats' in report
print(f"independent oracle OK: {len(current)} current, {len(fresh)} new, {len(bans)} legality, {len(gc)} GC")
