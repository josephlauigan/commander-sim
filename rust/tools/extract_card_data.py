"""ONE-TIME extraction (October 2026, the move to Rust): write the card definitions that lived in Python code into
data files the Rust `cardgen` reads. After this ran, the data files are the source: edit them, not the Python.
Do not run it again once the data files have been edited by hand (it would overwrite them).

    python3 rust/tools/extract_card_data.py            (from the repository root)

It loads the Python package the way `python3 -m commander_sim.tools.export_cards` does (every deck, the pools,
every card-code module in cardimpl's import order), then writes:

  data/carddb.txt           cards/carddb.py's DB_TEXT (the hand-tagged cards), plus the Jace Token alela.py adds
  data/card_overrides.toml  per card, everything cards/impl/*.py added with card() / full() / note() and the tag
                            edits it ran once the card data was loaded (pool_cards.POST: rules.set_tags, Galadriel's
                            stay-home creatures, the keyword data for hand-tagged cards); the final state after every
                            module loaded
  data/python_hooks.txt     the events each card's Python implementation handled (cardimpl.HOOKS) and whether it had
                            a hand-written cast priority (SPELL_PRIO): a frozen record, carried into cards.json
  data/my_decks.toml        your decks: the deck file, display name, commander and colour identity of each

and checks that the files say exactly what the Python state is (read back, compared field by field).
"""
import json, os, re, sys, tomllib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, ROOT)
DATA = os.path.join(ROOT, 'data')

from commander_sim.tools import export_cards as X  # noqa: E402

E, pools, DECKS = X.load_everything()
from commander_sim import ais, deck_files  # noqa: E402
from commander_sim.cards import pool_cards as PC, cardimpl as CI, carddb  # noqa: E402
from commander_sim.cards.impl import galadriel  # noqa: E402

MAIN = CI.main_cards()
ALWAYS = ('pain', 'nonblack', 'labyrinth')      # rules.set_tags adds these to your decks' cards too


# ------------------------------------------------------------------ TOML writing
def tstr(s):
    return json.dumps(s, ensure_ascii=False)        # a JSON string is a TOML basic string


def tkey(k):
    return k if re.fullmatch(r'[A-Za-z0-9_-]+', k) else tstr(k)


def tval(v):
    if v is True: return 'true'
    if v is False: return 'false'
    if isinstance(v, int): return str(v)
    if isinstance(v, str): return tstr(v)
    if isinstance(v, (list, tuple)): return '[' + ', '.join(tval(x) for x in v) + ']'
    if isinstance(v, dict): return '{ ' + ', '.join(f'{tkey(k)} = {tval(x)}' for k, x in v.items()) + ' }' if v else '{}'
    raise TypeError(f'{v!r} has no TOML form')


# ------------------------------------------------------------------ what the Python state holds
def post_edits():
    """name -> {'add_tags': [...], 'pool_add_tags': [...], 'pool_remove_tags': [...]} from pool_cards.POST"""
    out = {}
    for fn in PC.POST:
        q = fn.__qualname__
        if q == 'set_tags.<locals>.fn':
            cv = dict(zip(fn.__code__.co_freevars, (c.cell_contents for c in fn.__closure__)))
            e = out.setdefault(cv['name'], {})
            for k in cv['remove']: e.setdefault('pool_remove_tags', []).append(k)
            for k in cv['add']:
                e.setdefault('add_tags' if k.partition('=')[0] in ALWAYS else 'pool_add_tags', []).append(k)
        elif q == '_noatk':
            for n in galadriel.NOATK: out.setdefault(n, {}).setdefault('add_tags', []).append('noatk')
        elif q in ('_post_veyran', '_post_sauron'):
            pass                                     # keyword data for hand-tagged cards: read from the cards below
        else:
            raise SystemExit(f'unknown pool_cards.POST function {q}: extend this script')
    return out


def manual_extras():
    """keyword data the code set on hand-tagged cards (mine._post_veyran / _post_sauron, fixes' Tergrid menace):
    the final CD's kws / ward / start_loyalty against a fresh one from DB_TEXT"""
    out = {}
    base = {}
    for line in carddb.DB_TEXT.strip().splitlines():
        n, t, c, g = [x.strip() for x in line.split('|')]
        base[n] = E.CD(n, t, c, g)
    for n, cd in E.DB.items():
        if cd.source != 'manual': continue
        e = {}
        printed = set()
        rec = __import__('commander_sim.cards.scryfall', fromlist=['x']).load_cache().get(n.lower())
        if rec and 'Changeling' in (rec.get('keywords') or []): printed.add('changeling')
        if set(cd.kws) - printed: e['kws'] = sorted(set(cd.kws) - printed)
        if cd.ward: e['ward'] = cd.ward
        if cd.start_loyalty is not None: e['loyalty'] = str(cd.start_loyalty)
        if e: out[n] = e
    return out


def entries():
    edits = post_edits()
    extras = manual_extras()
    names = set(PC.CARDS) | set(PC.NOTES) | set(edits) | set(extras)
    out = {}
    for n in sorted(names):
        e = {}
        spec = PC.CARDS.get(n)
        if spec is not None:
            assert set(spec) <= {'tags', 'types', 'cost', 'dsl', 'kws', 'ward'}, (n, spec)
            assert spec.get('tags') is not None or any(k in spec for k in ('dsl', 'types', 'cost')), n
            if spec.get('types'): e['types'] = spec['types']
            if spec.get('cost'): e['cost'] = spec['cost']
            if spec.get('tags') is not None: e['tags'] = spec['tags']
            if 'kws' in spec: e['kws'] = sorted(spec['kws'])
            if 'ward' in spec: e['ward'] = spec['ward']
            if 'dsl' in spec: e['dsl'] = spec['dsl'] or []
        x = extras.get(n, {})
        if x:
            assert spec is None or (n in MAIN and E.DB[n].source == 'manual'), n
            e.update(x)
        e.update(edits.get(n, {}))
        if n in PC.NOTES:
            e['status'], e['note'] = PC.NOTES[n]
        out[n] = e
    return out


HEAD_OVERRIDES = '''\
# Card overrides: one table per card, by exact card name. Edit by hand; then `cardgen export` (rust/crates/cardgen).
# Extracted once from the Python card code (cards/impl/*.py: card(), full(), note() and the tag edits run after
# loading) by rust/tools/extract_card_data.py; this file is now the source.
#
# Keys (all optional):
#   types, cost, tags, dsl   an override: the card is rebuilt from these (source "pool"); a missing key keeps the
#                            card's own (types / cost / tags from Scryfall or carddb.txt). tags uses carddb.txt's
#                            vocabulary; dsl is the card's compiled abilities (the ability language, see
#                            cardgen's dsl_parse.rs); dsl = [] means none. An override with abilities also gets the
#                            AI hint tags they imply. Your decks' hand-tagged cards (carddb.txt) ignore an
#                            override: your decks' AI is built around their tags.
#   kws, ward                keyword data: replace the card's keywords (lower case) / its ward cost
#   loyalty                  a planeswalker's starting loyalty (a string, as printed)
#   add_tags                 tags added once the cards are loaded ("noatk", "pain", "dork=A")
#   pool_add_tags            the same, but not to cards in your decks
#   pool_remove_tags         tags removed, but not from cards in your decks
#                            (the three tag edits apply to the cards the decks play, not to cache-only cards)
#   status, note             the audit's verdict (Full / Approximate / Partial / Unmodeled) and what is modeled
'''


def write_overrides(ents, path):
    order = ('types', 'cost', 'tags', 'kws', 'ward', 'loyalty', 'dsl', 'add_tags', 'pool_add_tags',
             'pool_remove_tags', 'status', 'note')
    L = [HEAD_OVERRIDES]
    for n, e in ents.items():
        assert set(e) <= set(order), (n, e)
        L.append(f'[{tstr(n)}]')
        for k in order:
            if k not in e: continue
            v = e[k]
            if k == 'dsl' and v:
                L.append('dsl = [')
                for a in v: L.append('  ' + tval(a) + ',')
                L.append(']')
            else:
                L.append(f'{k} = {tval(v)}')
        L.append('')
    with open(path, 'w', encoding='utf-8') as fh: fh.write('\n'.join(L))


def write_carddb(path):
    lines = carddb.DB_TEXT.strip().splitlines()
    jace = E.DB.get('Jace Token')
    assert jace is not None
    head = ('# Hand-tagged cards, one per line:  name | types | cost | tags\n'
            '#   types: L land, C creature, I instant, S sorcery, A artifact, E enchantment, P planeswalker (AC: an\n'
            '#          artifact creature); cost: generic then coloured pips ("2UU", "-" for a land, "0")\n'
            '#   tags:  the roles the AI reads (pow=4 fly leg rock=1:C ...); see rust/crates/sim-core/src/tag.rs\n'
            '# These win over Scryfall data. Extracted once from commander_sim/cards/carddb.py (DB_TEXT); this file is\n'
            '# now the source. A later line for the same name replaces an earlier one.\n')
    with open(path, 'w', encoding='utf-8') as fh:
        fh.write(head + '\n'.join(lines) + '\n'
                 '# a planeswalker token (Alela\'s empowered Jace): no card, it ceases to exist when it leaves\n'
                 'Jace Token|P|0|tokpw\n')


def write_hooks(path):
    rows = []
    for n in sorted(set(CI.HOOKS) | set(CI.SPELL_PRIO)):
        ev = sorted(CI.HOOKS.get(n, {}))
        rows.append(f"{n}|{' '.join(ev)}|{'prio' if n in CI.SPELL_PRIO else ''}")
    head = ('# FROZEN record of the Python card code (commander_sim/cards/impl, October 2026): per card, the events its\n'
            '# Python implementation handled and "prio" when it had a hand-written cast priority. cardgen copies it into\n'
            '# cards.json (python_hooks, spell_prio): the work list for porting card code to Rust (cardgen audit\n'
            '# compares it with the Rust registry). New cards need no line.\n')
    with open(path, 'w', encoding='utf-8') as fh: fh.write(head + '\n'.join(rows) + '\n')


def write_my_decks(path):
    L = ['# Your decks: the deck key, its file under decklists/, the name the logs use, the commander the simulator',
         '# plays and the colour identity of the seat. Extracted once from deck_files.py, engine.py and ais.py.', '']
    for k, (owner, f) in deck_files.FILES.items():
        L += [f'[{k}]', f'file = {tstr(owner + "/" + f)}', f'name = {tstr(E.NAME(X._Seat(k)))}',
              f'commander = {tstr(ais.CMDS[k])}', f'ident = {tstr(E.IDENT[k])}', '']
    with open(path, 'w', encoding='utf-8') as fh: fh.write('\n'.join(L))


def check(ents):
    """read the files back and compare with the Python state"""
    with open(os.path.join(DATA, 'card_overrides.toml'), 'rb') as fh: t = tomllib.load(fh)
    assert list(t) == list(ents)
    for n, e in ents.items():
        got = t[n]
        for k, v in e.items():
            w = json.loads(json.dumps(v))
            assert got[k] == w, (n, k, got[k], w)
            if k == 'dsl': assert json.dumps(got[k]) == json.dumps(w), n          # key order too
    for n, spec in PC.CARDS.items():
        if 'dsl' in spec: assert json.dumps(t[n]['dsl']) == json.dumps(spec['dsl'] or []), n
        assert t[n].get('tags') == spec.get('tags'), n
    for n, (s, note) in PC.NOTES.items(): assert (t[n]['status'], t[n]['note']) == (s, note), n
    with open(os.path.join(DATA, 'my_decks.toml'), 'rb') as fh: d = tomllib.load(fh)
    assert {k: v['commander'] for k, v in d.items()} == {k: ais.CMDS[k] for k in deck_files.FILES}


def main():
    ents = entries()
    write_carddb(os.path.join(DATA, 'carddb.txt'))
    write_overrides(ents, os.path.join(DATA, 'card_overrides.toml'))
    write_hooks(os.path.join(DATA, 'python_hooks.txt'))
    write_my_decks(os.path.join(DATA, 'my_decks.toml'))
    check(ents)
    print(f'{len(ents)} override entries, {len(CI.HOOKS)} cards with Python hooks -> data/')


if __name__ == '__main__':
    main()
