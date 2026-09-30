"""The rules check for the human seat: is this move legal right now? Each check returns None (legal) or a one-line
reason to show the player. Nothing here changes the game."""
from commander_sim import engine as E
from commander_sim.play import mana

MAIN_STEPS = ('main1', 'main2')


def land_drops(g, p):
    """land drops p has this turn (Exploration, Azusa and similar add more)"""
    return 1 + (E.CI.total(g, 'extra_lands', p) if g.hooks else 0) + getattr(p, 'extra_land_now', 0)


def sorcery_timing(g, p):
    """None if p may do sorcery-speed things now (its own main phase), else why not"""
    if g.active is not p: return "It isn't your turn."
    if getattr(g, 'step', None) not in MAIN_STEPS: return 'You can only do that in one of your main phases.'
    return None


def check_land(g, p, c):
    if c not in p.hand: return f"{c.name} isn't in your hand."
    if not c.land: return f"{c.name} isn't a land."
    why = sorcery_timing(g, p)
    if why: return why.replace('do that', 'play a land')
    if getattr(p, 'lands_played', 0) >= land_drops(g, p):
        return "You've already played a land this turn." if land_drops(g, p) == 1 else "You've used all your land drops this turn."
    return None


def instant_speed(c):
    return c.instant or 'flash' in c.tags or 'flash' in (c.kws or ())


def base_cost(g, p, c):
    """what c costs to cast (before target-dependent extras). The engine stores a few cards at the cost the AI always
    pays; the person pays the printed cost"""
    if c.name == "Bloodchief's Thirst": return 0, 'B'           # {B}; kicker {2}{B} for a target over mana value 2
    return E.cost_of(p, c)


def check_cast(g, p, c, zone='hand'):
    """can p cast c from zone ('hand' or 'cmd') now? Checks where it is, timing, locks and the mana in the pool"""
    if zone == 'hand' and c not in p.hand: return f"{c.name} isn't in your hand."
    if zone == 'cmd' and not (c is p.cmd and p.cmd_in_zone): return 'Your commander is not in the command zone.'
    if c.land: return f'{c.name} is a land: play it as your land drop instead.'
    if not instant_speed(c):
        why = sorcery_timing(g, p)
        if why: return why.replace('do that', f'cast {c.name}')
    if not E.castable(g, p, c, zone): return f"Something on the battlefield stops you casting {c.name} right now."
    gen, pips = base_cost(g, p, c)
    why = mana.cost_problem(g, p, gen, pips)
    if why: return f"Can't cast {c.name}. {why}"
    return None


# ------------------------------------------------------------------ targets
TGT_OK = {'c': lambda m, ty: m.creature, 'cp': lambda m, ty: m.creature or 'P' in ty,
          'cap': lambda m, ty: m.creature or 'A' in ty or 'P' in ty, 'ce': lambda m, ty: m.creature or 'E' in ty,
          'nl': lambda m, ty: True, 'p': lambda m, ty: True, 'a': lambda m, ty: 'A' in ty,
          'cna': lambda m, ty: m.creature and 'A' not in ty, 'ae': lambda m, ty: 'A' in ty or 'E' in ty,
          'blue': lambda m, ty: m.cd is not None and 'U' in m.cd.pips}
WHAT = {'c': 'creature', 'cp': 'creature or planeswalker', 'cap': 'creature, artifact or planeswalker',
        'ce': 'creature or enchantment', 'nl': 'nonland permanent', 'p': 'permanent', 'a': 'artifact',
        'cna': 'nonartifact creature', 'ae': 'artifact or enchantment', 'blue': 'blue permanent'}


def player_targetable(q):
    """Teferi's Protection and The One Ring give protection from everything"""
    return q.alive and not getattr(q, 'life_locked', False) and not getattr(q, 'ring_prot', False)


def spell_targets(g, p, c):
    """what spell c may target, by the rules (not by what the AI would pick): a list of permanents and players, or
    None if c doesn't target. Your own permanents count too; hexproof, shroud and protection are respected"""
    t = c.tags
    if 'rem' not in t: return None
    tgt = t.get('tgt', 'c')
    ok = TGT_OK.get(tgt, TGT_OK['c'])
    out = []
    for q in g.players:
        if not q.alive: continue
        for m in q.perms:
            ty = m.cd.types if m.cd is not None else 'C'
            if m.phased or not ok(m, ty): continue
            if 'alsoart' in t and not m.creature and 'A' not in ty: continue
            if E.untargetable(g, m) or E.protected_from(g, m, c.pips): continue
            if 'nonblack' in t and 'B' in E.colors_of(m): continue
            if 'mv4' in t and m.cd is not None and m.cd.cmc > 4: continue
            if c.name == 'Fatal Push' and m.cd is not None and m.cd.cmc > 2:
                from commander_sim.cards.impl import rules
                if m.cd.cmc > 4 or not rules.revolt(g, p): continue
            out.append(m)
    if 'face' in t: out += [q for q in g.players if player_targetable(q)]
    return out


def describe_target(g, p, x):
    """'Sheoldred, the Apocalypse (Prosper, tapped)' / 'Krenko (player, 31 life)'"""
    if isinstance(x, E.Player): return f"{E.NAME(x)} (player, {x.life} life)" + (' (you)' if x is p else '')
    from commander_sim.play.view import token_name
    name = token_name(x) if x.token else x.name
    bits = ['you' if x.owner is p else E.NAME(x.owner)]
    if x.creature: bits.append(f'{E.epow(g, x)}/{E.etgh(g, x)}')
    if x.tapped: bits.append('tapped')
    return f"{name} ({', '.join(bits)})"


def target_extra_cost(g, p, c, x):
    """more mana this target needs: ward (on an opponent's permanent), Bloodchief's Thirst's kicker"""
    gen, pips = 0, ''
    if isinstance(x, E.Perm) and x.owner is not p and x.cd is not None and x.cd.ward: gen += x.cd.ward
    if c.name == "Bloodchief's Thirst" and isinstance(x, E.Perm) and x.cd is not None and x.cd.cmc > 2:
        gen, pips = gen + 2, pips + 'B'                       # kicked: {2}{B} more, any mana value
    return gen, pips


# ------------------------------------------------------------------ equipment
EQUIP_COST = {'sword': 2, 'flail': 2, 'cloak': 2, 'animist': 2, 'helm': 1, 'nim': 4}


def equip_cost(m):
    """the equip cost of equipment m, or None if m isn't equipment"""
    if m.cd is None: return None
    for a in (m.cd.dsl or ()):
        if a.get('static') == 'equip_cost': return int(a.get('mana', 0))
    t = m.cd.tags
    if t.get('prot') == 'boots': return 0 if m.cd.name == 'Lightning Greaves' else 1
    for k, v in EQUIP_COST.items():
        if k in t: return v
    return None


def check_equip(g, p, e, target):
    if e not in p.perms or equip_cost(e) is None: return "That isn't equipment you control."
    why = sorcery_timing(g, p)
    if why: return why.replace('do that', 'equip')
    if target not in p.perms or not target.creature or target.phased: return 'Equip only onto a creature you control.'
    if e.attached is target: return f'{e.name} is already on {target.name}.'
    why = mana.cost_problem(g, p, equip_cost(e), '')
    if why: return f"Can't equip {e.name}. {why}"
    return None
