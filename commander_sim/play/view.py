"""The table as one seat is allowed to see it: plain dicts, safe to hand to another thread or to JSON.

Hidden information stays hidden: other players' hands and every library are counts only.
"""
from commander_sim import engine as E


def token_name(m):
    """a token's name as players say it: 'Orc Army', 'Goblin token', 'Treasure'"""
    if m.army: return 'Orc Army'
    if m.name and m.name != 'Token': return m.name
    kinds = sorted(t for t in (m.ttypes or ()) if t)
    if kinds: return ' '.join(k.title() for k in kinds) + ' token'
    guess = guess_token(m)
    return f'{guess} token' if guess else 'Token'


def guess_token(m):
    """many cards make tokens the engine doesn't name (Krenko's Goblins): the token its owner's deck makes with the
    same power, toughness, colour and flying (then any deck at the table's, for tokens given to you: Beast Within)"""
    try:
        from commander_sim.play import images
    except ImportError:
        return None
    owner = m.orig or m.owner
    decks = [owner.deck_names]
    g = E.CUR_G
    if g is not None: decks.append(tuple(n for q in g.players for n in q.deck_names))
    pw, tg = m.pow, m.tgh
    for names in decks:
        kinds = [t for t in images.token_kinds(names) if (t['power'], t['toughness']) == (pw, tg)]
        if not kinds: continue
        count = {}
        for t in kinds: count[t['name']] = count.get(t['name'], 0) + 1      # how many of the deck's cards make it
        def score(t):        # the engine leaves many tokens colourless, so colour only counts when it has one
            return ((t['colors'] == m.colors) * 2 if m.colors else 0) + (t['flying'] == bool(m.fly)) + count[t['name']] / 100
        return max(kinds, key=score)['name']
    return None


def _perm(g, m):
    d = {'name': token_name(m) if m.token else m.name, 'tapped': bool(m.tapped), 'token': bool(m.token)}
    if m.creature:
        d['pt'] = f'{E.epow(g, m)}/{E.etgh(g, m)}'
        d['sick'] = bool(m.sick)
    if m.plus: d['counters'] = m.plus
    if m.loyalty is not None: d['loyalty'] = m.loyalty
    if m.attached is not None and getattr(m.attached, 'name', None): d['attached_to'] = m.attached.name
    if m.is_cmd: d['commander'] = True
    if m.phased: d['phased_out'] = True
    return d


def _player(g, p, me):
    you = p.key == me
    d = {'key': p.key, 'name': E.NAME(p), 'commander': p.cmd.name, 'you': you, 'alive': bool(p.alive),
         'life': p.life, 'poison': getattr(p, 'poison', 0),
         'commander_damage': {E.NAME(q): n for q in g.players for k, n in p.cmd_dmg.items() if k == q.key and n},
         'hand_count': len(p.hand), 'library': len(p.library),
         'graveyard': [c.name for c in p.gy], 'exile': [c.name for c in p.exile],
         'commander_in_zone': bool(p.cmd_in_zone), 'tax': p.tax, 'treasures': p.treasures,
         'battlefield': [dict(_perm(g, m), i=i) for i, m in enumerate(p.perms)],
         'lands': [{'name': L.cd.name, 'tapped': bool(L.tapped), 'i': i} for i, L in enumerate(p.lands)]}
    if you:
        from commander_sim.play import mana
        d['hand'] = [c.name for c in p.hand]
        from commander_sim.play import cards
        d['hand_land'] = [bool(c.land) for c in p.hand]
        d['hand_special'] = [c.name in cards.HAND for c in p.hand]      # a land with a spell side or channel
        d['mana_pool'] = mana.pool_of(p).text()
        d['mana_sources'] = mana.sources(g, p)
        from commander_sim.play import abilities, legal
        d['graveyard_playable'] = [{'i': i, 'name': c.name, 'how': (legal.gy_mode(g, p, c) or ('land',))[0]}
                                   for i, c in enumerate(p.gy)
                                   if legal.gy_mode(g, p, c) or (c.land and legal.check_land_gy(g, p, c) is None)]
        for L, entry in zip(p.lands, d['lands']):
            if abilities.land_abilities(g, p, L): entry['ability'] = True
    return d


def build_view(g, me):
    """the table from seat `me` (a deck key); call on the engine thread"""
    return {'round': g.round, 'active': g.active.key if g.active is not None else None, 'step': getattr(g, 'step', None),
            'over': bool(g.over), 'winner': g.winner.key if getattr(g, 'winner', None) is not None else None,
            'players': [_player(g, p, me) for p in g.players]}
