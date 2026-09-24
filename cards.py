"""Card sources, in priority order:
  1. cards_dsl.json      explicit ability data you (or an LLM) wrote for a card -- always wins
  2. carddb.py           hand-verified tags for the audited cards
  3. Scryfall + compiler Oracle text compiled into ability data (dsl_parse), run by the interpreter (dsl);
                         falls back to regex tags (autotag) when nothing compiles
Set SIM_DSL_ALL=1 (or pass --dsl-all to compare.py) to run *every* card from Scryfall text instead of hand tags.
"""
import os, sys
import engine
import dsl


def _tagdict(s):
    return dict(x.split('=', 1) if '=' in x else (x, True) for x in s.split())


def _from_record(name, rec):
    import autotag
    r = autotag.autotag(rec)
    face = rec['card_faces'][0] if rec.get('card_faces') and rec.get('layout') not in ('split',) else rec
    oracle = face.get('oracle_text', rec.get('oracle_text', ''))
    cd = dsl.build_cd(name, r['types'], r['cost'], _tagdict(r['tags']), oracle, face.get('loyalty') or rec.get('loyalty'))
    if not cd.dsl:                              # nothing compiled: use the regex tagger's tags instead
        cd = engine.CD(name, r['types'], r['cost'], r['tags'])
        cd.source = 'scryfall'; cd.unparsed = r['unparsed'] + ['note: ' + x for x in r['notes']]
    cd.game_changer = r['game_changer']
    cd.identity = ''.join(rec.get('color_identity') or [])
    return cd


def _from_override(name, ov, base):
    ab = ov.get('abilities', [])
    types = ov.get('types') or (base.types if base else 'C')
    cost = ov.get('cost') or (f'{base.generic or ""}{base.pips}' if base else '0')
    tags = {k: v for k, v in (base.tags.items() if base else []) if k in dsl.SAFE_TAGS}
    tags.update(_tagdict(ov.get('tags', '')))
    tags.update(dsl.hints(ab, types))
    cd = engine.CD(name, types, cost or '0', ' '.join(k if v is True else f'{k}={v}' for k, v in tags.items()))
    cd.dsl = ab or None; cd.source = 'cards_dsl.json'; cd.start_loyalty = ov.get('loyalty')
    if base is not None: cd.identity = getattr(base, 'identity', None)
    return cd


def ensure_cards(names, verbose=True):
    """Make sure every name is in engine.DB. Returns (added, not_found)."""
    names = list(dict.fromkeys(names))
    dsl_all = os.environ.get('SIM_DSL_ALL') == '1'
    ov = dsl.overrides()
    added = []
    for n in names:                              # explicit ability data always wins
        if n in ov and engine.DB.get(n) is not None and engine.DB[n].source == 'cards_dsl.json': continue
        if n in ov and (n in engine.DB or not ov[n].get('needs_scryfall')):
            if n in engine.DB or ov[n].get('types'):
                engine.DB[n] = _from_override(n, ov[n], engine.DB.get(n)); added.append(engine.DB[n])
    want = [n for n in names if n not in engine.DB or
            (dsl_all and engine.DB[n].source == 'manual' and n not in ('Forest', 'Island', 'Plains', 'Swamp', 'Mountain'))]
    if want:
        import scryfall
        recs = scryfall.fetch(want, verbose)
        for n in want:
            rec = recs.get(n)
            if rec is None: continue
            engine.DB[n] = _from_record(n, rec); added.append(engine.DB[n])
            if n in ov: engine.DB[n] = _from_override(n, ov[n], engine.DB[n])
    return added, [n for n in names if n not in engine.DB]


def describe(cd):
    tags = ' '.join(k if v is True else f'{k}={v}' for k, v in cd.tags.items())
    s = f'{cd.name} [{cd.source}] {cd.types} {cd.generic or ""}{cd.pips} :: {tags or "(no tags)"}'
    for a in (cd.dsl or []):
        what = a.get('event') or a.get('static') or a.get('replace') or ''
        s += f"\n      ability: {a['type']} {what} -> " + ', '.join(e.get('do', '?') for e in a.get('effects', []))
    for u in cd.unparsed: s += f'\n      not modeled: {u}'
    return s


def identity(cd):
    ident = getattr(cd, 'identity', None)
    if ident is not None: return set(ident)
    s = set(cd.pips)
    if cd.land and cd.tags.get('c') not in (None, 'A', 'C'): s |= set(cd.tags['c'])
    return s & set('WUBRG')


def game_changers(names):
    import scryfall
    names = list(dict.fromkeys(names))
    recs = scryfall.fetch(names, verbose=True)
    if len(recs) < len(names) * 0.9: return None          # offline or incomplete: don't report a misleading count
    return sorted(n for n, r in recs.items() if r.get('game_changer'))
