"""Card data for the opponent pools: tag / ability overrides and audit notes.

CARDS[name] = dict(
    tags='...',          replaces the card's tags (same vocabulary as carddb.py); types= / cost= optional
    dsl=[...],           replaces its compiled abilities (ability language, see dsl_parse.py); [] = none
    status=(status, note))   audit status for pool decks: Full / Approximate / Partial / Unmodeled
Overrides apply to every card except the main decks' hand-tagged cards (carddb.py), which keep their tags (the
main decks' AI is built around them); the note is still used by the audit.
Python implementations (cardimpl hooks) record their notes with note() from the impl_* modules.
"""
NOTES = {}           # name -> (status, note): read by pool_audit


def note(name, status, text):
    NOTES[name] = (status, text)


CARDS = {}
POST = []            # fn() run after the overrides are applied (tag edits that must wait for the card data)


def card(name, tags=None, status=None, **kw):
    CARDS[name] = dict(tags=tags, **kw)
    if status: note(name, *status)


def apply(verbose=False):
    """rebuild engine.DB entries for overridden pool cards (called by pools.register)"""
    import engine, dsl, cardimpl
    main = cardimpl.main_cards()
    for name, spec in CARDS.items():
        base = engine.DB.get(name)
        if base is None or (name in main and base.source == 'manual'): continue   # your hand-tagged cards keep their tags
        if getattr(base, 'pool_override', False): continue
        types = spec.get('types') or base.types
        cost = spec.get('cost') or (f'{base.generic or ""}{base.pips}' or '0')
        tags = spec['tags'] if spec.get('tags') is not None else ' '.join(
            k if v is True else f'{k}={v}' for k, v in base.tags.items())
        ab = spec.get('dsl', base.dsl)
        if 'dsl' in spec and spec['dsl']:
            extra = dsl.hints(spec['dsl'], types)
            tags = ' '.join([tags] + [k if v is True else f'{k}={v}' for k, v in extra.items() if k not in tags.split()])
        cd = engine.CD(name, types, cost, tags)
        cd.dsl = ab or None
        cd.unparsed = [] if 'dsl' in spec or spec.get('tags') is not None else list(base.unparsed)
        for a in ('identity', 'kws', 'protfrom', 'ward', 'game_changer', 'start_loyalty', 'subtypes'):
            if hasattr(base, a): setattr(cd, a, getattr(base, a))
        if 'kws' in spec: cd.kws = frozenset(spec['kws'])
        if 'ward' in spec: cd.ward = spec['ward']
        cd.source = 'pool'
        cd.pool_override = True
        engine.DB[name] = cd
    for fn in POST: fn()
