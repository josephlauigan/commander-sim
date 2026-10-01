"""The table as one seat is allowed to see it: plain dicts, safe to hand to another thread or to JSON.

Hidden information stays hidden: other players' hands and every library are counts only.
"""
from commander_sim import engine as E


def token_name(m):
    """a token's name as players say it: 'Orc Army', 'Goblin token', 'Treasure'"""
    if m.army: return 'Orc Army'
    if m.name and m.name != 'Token': return m.name
    kinds = sorted(t for t in (m.ttypes or ()) if t)
    return (' '.join(k.title() for k in kinds) + ' token') if kinds else 'Token'


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
        d['mana_pool'] = mana.pool_of(p).text()
        d['mana_sources'] = mana.sources(g, p)
        from commander_sim.play import abilities
        for L, entry in zip(p.lands, d['lands']):
            if abilities.land_abilities(g, p, L): entry['ability'] = True
    return d


def build_view(g, me):
    """the table from seat `me` (a deck key); call on the engine thread"""
    return {'round': g.round, 'active': g.active.key if g.active is not None else None, 'step': getattr(g, 'step', None),
            'over': bool(g.over), 'winner': g.winner.key if getattr(g, 'winner', None) is not None else None,
            'players': [_player(g, p, me) for p in g.players]}
