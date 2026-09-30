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
         'battlefield': [_perm(g, m) for m in p.perms if m.cd is not None or m.token],
         'lands': [{'name': L.cd.name, 'tapped': bool(L.tapped)} for L in p.lands]}
    if you: d['hand'] = [c.name for c in p.hand]
    return d


def build_view(g, me):
    """the table from seat `me` (a deck key); call on the engine thread"""
    return {'round': g.round, 'active': g.active.key if g.active is not None else None, 'step': getattr(g, 'step', None),
            'over': bool(g.over), 'winner': g.winner.key if getattr(g, 'winner', None) is not None else None,
            'players': [_player(g, p, me) for p in g.players]}
