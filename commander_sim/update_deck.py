"""Replace a deck's list with a new one: paste the 100 cards, and the deck file's card sections are rewritten.

    python3 -m commander_sim.update_deck sauron new-list.txt            # from a file
    python3 -m commander_sim.update_deck sauron -                       # paste the list, then Ctrl-D
    python3 -m commander_sim.update_deck sauron new-list.txt --dry-run  # show what would change, write nothing
    python3 -m commander_sim.update_deck sauron new-list.txt --log "Why I made these swaps."

The deck is one of your deck keys (seph, veyran, sauron), a pool deck key, or a path to a deck .md file.

The list can be copied from most deck sites: "1 Card Name", "1x Card Name" or just "Card Name" per line. Set codes,
collector numbers and foil marks ("(MOM) 123 *F*") are ignored, as are blank lines and headers such as "Commander"
or "Deck". Anything after a "Sideboard" or "Maybeboard" header is ignored. The commander must be in the list.

What it does:
  1. checks the list: 100 cards, singleton, the commander present, every name found on Scryfall, colour identity,
     bans, and (pool decks) the tier's Game Changer rule; any problem stops it before a file is touched;
  2. rewrites '## Import list' and the card lines of '## Decklist by type' (keeping the file's own groups and any
     notes after them), and the Game Changers / Lands lines of a pool deck's header;
  3. with --log, adds an '**Updated <date>.**' line naming the cards out and in;
  4. for your decks, records the new list in tests/fixtures/my_decks_parsed.json (the deck guard test);
  5. reports what changed, how completely the simulator models the new cards, and where the rest of the file still
     mentions cards that left (strategy text isn't rewritten).
"""
import datetime, os, re, sys, unicodedata
from commander_sim import ROOT

BASICS = ('Plains', 'Island', 'Swamp', 'Mountain', 'Forest', 'Wastes', 'Snow-Covered Plains', 'Snow-Covered Island',
          'Snow-Covered Swamp', 'Snow-Covered Mountain', 'Snow-Covered Forest')
STOP = re.compile(r'^(sideboard|maybeboard|considering|tokens?)\b', re.I)
HEADER = re.compile(r'^(commander|commanders|deck|main ?deck|mainboard|companion|about|name)\b.*$', re.I)
# the groups of '## Decklist by type', in the order a new group is placed, and which cards go in each
GROUPS = ('Commander', 'Creatures', 'Planeswalkers', 'Battles', 'Enchantments', 'Artifacts', 'Instants and sorceries',
          'Sorceries', 'Instants', 'Lands')


# ------------------------------------------------------------------ reading the list
def parse_list(text):
    """[(count, name)] from a pasted list"""
    out = []
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith(('#', '//')): continue
        if STOP.match(line): break
        line = re.sub(r'\s+\*[A-Za-z]+\*', '', line)                        # foil / etched marks
        line = re.sub(r'\s+\([A-Za-z0-9]{2,6}\)(\s+[\w-]+)?\s*$', '', line)   # (SET) 123
        line = re.sub(r'\s+\[[^\]]*\]\s*$', '', line)                         # [SET]
        m = re.match(r'^(\d+)\s*x?\s+(.+)$', line, re.I)
        count, name = (int(m.group(1)), m.group(2).strip()) if m else (1, line)
        if not m and HEADER.match(line): continue
        out.append((count, name))
    return out


def _plain(s):
    """for comparing and sorting names: no accents, case or punctuation ("Mauhur Uruk-hai Captain" matches)"""
    s = unicodedata.normalize('NFKD', s)
    s = ''.join(c for c in s if not unicodedata.combining(c)).lower()
    return ' '.join(re.sub(r"[^\w/ ]", ' ', s).split())


def canonical(requested, rec, known=()):
    """the name to write for what was asked, or None when Scryfall found a different card. A two-faced card keeps
    the form the simulator already knows it by (its hand-written rules are registered under that name); a new card
    gets Scryfall's full name."""
    full = rec.get('name', '')
    front = full.split(' // ')[0]
    names = [full, front] + [f.get('name', '') for f in rec.get('card_faces') or []]
    if _plain(requested) not in {_plain(n) for n in names}: return None
    for n in (requested, full, front):
        if n in known: return n
    return full


# ------------------------------------------------------------------ the deck file
def _read(path):
    with open(path, encoding='utf-8') as fh: return fh.read()


MINE = {'seph': 'sephiroth-phyrexian-reanimator.md', 'veyran': 'veyran-izzet-spellslinger.md',
        'sauron': 'sauron-grixis-amass.md'}


def deck_path(which):
    if os.path.exists(which): return os.path.abspath(which)
    if which in MINE: return os.path.join(ROOT, 'decklists', 'mine', MINE[which])
    from commander_sim import pools
    d = pools.by_key().get(which)
    if d is None: sys.exit(f'No deck {which!r}: use seph, veyran, sauron, a pool deck key, or a path to a deck file')
    return d.path


def commander_of(path, text):
    for key, fname in MINE.items():                         # your decks: the commander the simulator plays
        if os.path.basename(path) == fname:
            from commander_sim import ais
            return ais.CMDS[key]
    m = re.search(r'^- \*\*Commander:\*\*\s*(.+?)\s*$', text, re.M)
    if m: return m.group(1)
    m = re.search(r'^\*\*Commander \(1\)\.\*\*\s*(.+?)\s*$', text, re.M)
    if m: return m.group(1)
    sys.exit(f'{path}: no commander line found')


def group_of(rec, labels):
    """the '## Decklist by type' group a card goes in, among the labels the file uses"""
    tl = (rec.get('type_line') or '').split(' // ')[0]
    if 'Land' in tl: return 'Lands'
    if 'Creature' in tl: return 'Creatures'
    if 'Planeswalker' in tl: return 'Planeswalkers'
    if 'Battle' in tl: return 'Battles'
    if 'Instant' in tl or 'Sorcery' in tl:
        if 'Instants and sorceries' in labels: return 'Instants and sorceries'
        return 'Instants' if 'Instant' in tl else 'Sorceries'
    if 'Enchantment' in tl: return 'Enchantments'
    if 'Artifact' in tl: return 'Artifacts'
    return 'Other'


def sort_key(old_singles):
    """the order the file's import list already uses (plain Python order, or ignoring case / punctuation)"""
    for key in (None, str.lower, _plain):
        k = key or (lambda x: x)
        if all(k(a) <= k(b) for a, b in zip(old_singles, old_singles[1:])): return k
    return _plain


def rewrite(text, commander, counts, recs, by_type_commas, today, log_note, key=_plain):
    """the new file text"""
    singles = sorted((n for n in counts if n not in BASICS), key=key)
    m = re.search(r'^## Import list \(\d+\)\s*\n\s*```\n(.*?)\n```', text, re.M | re.S)
    was = [l.split(' ', 1)[1] for l in (m.group(1).split('\n') if m else []) if ' ' in l]
    basics = sorted((n for n in counts if n in BASICS), key=lambda n: (was.index(n) if n in was else len(was), n))
    total = sum(counts.values())
    # import list
    block = '\n'.join([f'{counts[n]} {n}' for n in singles] + [f'{counts[n]} {n}' for n in basics])
    m = re.search(r'^## Import list \(\d+\)\s*\n\s*```\n(.*?)\n```', text, re.M | re.S)
    if not m: sys.exit('the file has no "## Import list" block')
    text = text[:m.start()] + f'## Import list ({total})\n\n```\n{block}\n```' + text[m.end():]
    # decklist by type: replace the bold group lines, keep everything else in the section
    m = re.search(r'^## Decklist by type \(\d+\)[ \t]*\n(.*?)(?=^## )', text, re.M | re.S)
    if m:
        body = m.group(1)
        lines = body.split('\n')
        bold = [i for i, l in enumerate(lines) if re.match(r'^\*\*[^*]+ \(\d+\)\.\*\*', l)]
        labels = [re.match(r'^\*\*(.+?) \(\d+\)\.\*\*', lines[i]).group(1) for i in bold]
        groups = {}
        for n in singles + basics:
            if n == commander: continue
            groups.setdefault(group_of(recs[n], labels), []).append(n)
        show = (lambda n: n) if by_type_commas else (lambda n: n.replace(',', ''))
        seen = re.findall(r'\b\d+ ([A-Z][\w-]*(?: [A-Z][\w-]*)*)', ' '.join(lines[i] for i in bold if 'Lands' in lines[i]))
        basic_rank = lambda n: (seen.index(n) if n in seen else len(seen), n)   # basics keep the file's order
        order = [l for l in labels if l != 'Commander'] + [g for g in GROUPS if g in groups and g not in labels] + \
                sorted(g for g in groups if g not in GROUPS)
        cmd_line = next((l for l in lines if l.startswith('**Commander (1).**')), None)
        new = [cmd_line or f'**Commander (1).** {commander}']            # an existing line is kept as written
        for g in order:
            cards = groups.get(g, [])
            if not cards: continue
            ones = [show(n) for n in cards if n not in BASICS]
            many = [f'{counts[n]} {n}' for n in sorted((n for n in cards if n in BASICS), key=basic_rank)]
            k = sum(counts[n] for n in cards)
            new.append(f'**{g} ({k}).** ' + ', '.join(ones + many))
        first, last = (bold[0], bold[-1]) if bold else (0, -1)
        rest = lines[last + 1:]
        body2 = '\n'.join(lines[:first] + '\n\n'.join(new).split('\n') + rest)
        text = text[:m.start()] + f'## Decklist by type ({total})\n' + body2 + text[m.end():]
    # a pool deck's header lines
    gcs = sorted((n for n in counts if recs[n].get('game_changer')), key=key)
    m = re.search(r'^- \*\*Game Changers \(\d+\):\*\*\s*(.*)$', text, re.M)
    same = m and int(re.search(r'\((\d+)\)', m.group(0)).group(1)) == len(gcs) and \
        all(n in m.group(1) or n.split(' // ')[0] in m.group(1) for n in gcs)
    if m and not same:                                                       # only when the set changed
        text = text[:m.start()] + f"- **Game Changers ({len(gcs)}):** {', '.join(gcs) or 'none'}" + text[m.end():]
    lands = [n for n in counts if 'Land' in (recs[n].get('type_line') or '').split(' // ')[0]]
    nb = sum(counts[n] for n in lands if n not in BASICS); b = sum(counts[n] for n in lands if n in BASICS)
    text = re.sub(r'^- \*\*Lands:\*\*.*$', f'- **Lands:** {nb + b} ({nb} nonbasic + {b} basic)', text, flags=re.M)
    if log_note is not None:
        text = add_log(text, today, log_note)
    return text


def add_log(text, today, line):
    """an '**Updated <date>.**' line after the last one near the top of the file (or after the title)"""
    entry = f'**Updated {today}.** {line}'.rstrip()
    lines = text.split('\n')
    top = next((i for i, l in enumerate(lines) if l.startswith('## ')), len(lines))
    ups = [i for i in range(top) if lines[i].startswith('**Updated ')]
    at = ups[-1] + 1 if ups else 1
    lines[at:at] = [entry] if ups else ['', entry]
    return '\n'.join(lines)


# ------------------------------------------------------------------ main
def main(argv):
    args = [a for a in argv if not a.startswith('--')]
    dry = '--dry-run' in argv
    log_note = argv[argv.index('--log') + 1] if '--log' in argv else None
    if log_note is not None: args.remove(log_note)
    if len(args) != 2: sys.exit(__doc__)
    which, src = args
    path = deck_path(which)
    text = _read(path)
    commander = commander_of(path, text)
    raw = sys.stdin.read() if src == '-' else _read(src)
    entries = parse_list(raw)
    if not entries: sys.exit('the list is empty')

    from commander_sim import pools
    from commander_sim.cards import scryfall, sources
    from commander_sim.decks import load
    recs = scryfall.fetch([n for _, n in entries] + [commander], verbose=False)
    old = load(path)
    pools.register()
    from commander_sim import engine
    known = set(old) | set(engine.DB)
    problems, counts = [], {}
    for k, n in entries:
        rec = recs.get(n)
        if rec is None: problems.append(f'not found on Scryfall: {n}'); continue
        name = canonical(n, rec, known)
        if name is None: problems.append(f'{n!r} matched a different card on Scryfall ({rec.get("name")}): check the spelling'); continue
        counts[name] = counts.get(name, 0) + k
        recs[name] = rec
    in_pool = os.sep + 'pool' + os.sep in path
    tier = os.path.basename(os.path.dirname(path)).split('-')[0] if in_pool else None
    deck = type('D', (), {'cards': [n for n, k in counts.items() for _ in range(k)], 'commander': commander,
                          'tier': tier, 'gc_claimed': None})()
    problems += pools.structural_problems(deck.cards, commander)       # every problem at once, not only the first
    problems += pools.card_problems(deck, recs)[0]
    if problems:
        print(f'Not updated: the new list for {os.path.basename(path)} has problems:')
        for p in problems: print('  - ' + p)
        sys.exit(1)

    out = sorted(set(old) - set(counts), key=_plain); inn = sorted(set(counts) - set(old), key=_plain)
    moved = sorted((n for n in set(old) & set(counts) if old.count(n) != counts[n]), key=_plain)
    by_type = re.search(r'^## Decklist by type.*?(?=^## )', text, re.M | re.S)
    typed = '\n'.join(l for l in by_type.group(0).split('\n') if not l.startswith('**Commander')) if by_type else ''
    commas = any(',' in n and n in typed for n in old if n != commander)
    today = datetime.date.today().isoformat()
    note = None
    if log_note is not None:
        note = (f"Out: {', '.join(out) or 'nothing'}. In: {', '.join(inn) or 'nothing'}." + (f' {log_note}' if log_note else ''))
    key = sort_key([n for n in old if n not in BASICS and old.count(n) == 1])
    new_text = rewrite(text, commander, counts, recs, commas, today, note, key)

    rel = os.path.relpath(path, ROOT) if path.startswith(ROOT + os.sep) else path
    print(f'{rel}: {len(out)} out, {len(inn)} in' + (f', {len(moved)} basic counts changed' if moved else ''))
    if out: print('  out: ' + ', '.join(out))
    if inn: print('  in:  ' + ', '.join(inn))
    for n in moved: print(f'  {n}: {old.count(n)} -> {counts[n]}')
    gcs = [n for n in counts if recs[n].get('game_changer')]
    print(f'  {sum(counts.values())} cards, {len(gcs)} Game Changers' + (f" ({', '.join(sorted(gcs))})" if gcs else ''))
    if new_text == text:
        print('Nothing to change.'); return
    if dry:
        print('\n(--dry-run: nothing written)')
    else:
        with open(path, 'w', encoding='utf-8') as fh: fh.write(new_text)
        print(f'Wrote {rel}.')
    report(path, text, out, inn, dry)


def report(path, text, out, inn, dry):
    """how well the new cards are modeled, and where the file still mentions cards that left"""
    from commander_sim.cards import sources
    from commander_sim import pools, pool_audit
    from commander_sim.decks import load
    pools.register()
    sources.ensure_cards(inn, verbose=False)
    mine = os.path.basename(path) in MINE.values()             # one of your decks (or a copy of one)
    real = os.path.dirname(path) == os.path.join(ROOT, 'decklists', 'mine')
    if inn:
        print('\nHow the simulator models the new cards:')
        for n in inn:
            s, note = pool_audit.card_status(n, mine=mine)
            print(f'  {s:11s} {n}' + (f': {note[:110]}' if s not in ('Full',) and note else ''))
        weak = [n for n in inn if pool_audit.RANK[pool_audit.card_status(n, mine=mine)[0]] < pool_audit.RANK['Full-auto']]
        if weak: print('  Cards below Full-auto play weaker (or stronger) than the real card; worth modeling before measuring.')
    lists = re.compile(r'^(## (Import list|Decklist by type)|\*\*[^*]+ \(\d+\)\.\*\*|\d+ )')
    hits = []
    kept = load(path) if os.path.exists(path) else []
    cmd_line = next((l for l in text.split('\n') if l.startswith('**Commander (1).**')), '')
    for i, line in enumerate(text.split('\n'), 1):
        if lists.match(line): continue
        for n in out:
            short = n.split(' // ')[0].split(',')[0]
            # a short name ("Kefka") only counts when no card still in the deck, and not the commander, shares it
            short_ok = len(short) >= 6 and short not in cmd_line and not any(short in k for k in kept)
            if n in line or n.replace(',', '') in line or (short_ok and short in line):
                hits.append((i, n, line.strip()))
    if hits:
        print('\nThe rest of the file still mentions cards that left (strategy text is not rewritten):')
        for i, n, line in hits[:25]:
            print(f'  line {i}: {n}: {line[:100]}')
        if len(hits) > 25: print(f'  ... and {len(hits) - 25} more')
    if real and not dry:
        _record_fixture()
        print('\nRecorded the new list for the deck guard (tests/fixtures/my_decks_parsed.json).')
    if not dry:
        print('Next: python3 -m unittest discover -s tests -t .   then measure it, e.g. python3 -m commander_sim --deck '
              '<key> --pool all')


def _record_fixture():
    import json
    from commander_sim.decks import load, P
    fix = os.path.join(ROOT, 'tests', 'fixtures', 'my_decks_parsed.json')
    files = {'seph': 'sephiroth-phyrexian-reanimator.md', 'veyran': 'veyran-izzet-spellslinger.md',
             'sauron': 'sauron-grixis-amass.md'}
    with open(fix, 'w') as fh: json.dump({k: sorted(load(P + f)) for k, f in files.items()}, fh, indent=0)


if __name__ == '__main__':
    main(sys.argv[1:])
