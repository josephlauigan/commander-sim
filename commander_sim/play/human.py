"""The human seat's turn: priority in the main phases, and the actions a player takes (tap for mana, play a land,
cast a spell, pass). Each action is checked by legal.py first; an illegal one is refused with the reason and the
player keeps priority.

Actions are dicts, the same from the text client and (later) the browser:
  {'do': 'tap', 'source': id, 'colour': 'B'}      a mana source from mana.sources()
  {'do': 'land', 'card': i}                       the i-th card in hand as your land drop
  {'do': 'cast', 'card': i}                       the i-th card in hand
  {'do': 'cast', 'zone': 'cmd'}                   your commander from the command zone
  {'do': 'use', 'perm': i}                        an ability of your i-th permanent (Equip, loyalty, ...)
  {'do': 'pass'}
Follow-up questions (which target, which ability) come back as 'target' / 'choose' requests: answer with the index
of a choice, or 'cancel'.
"""
from commander_sim import engine as E, ais
from commander_sim.play import mana, legal
from commander_sim.play.controller import controller_of, Request
from commander_sim.play.view import build_view

STEP_NAMES = {'main1': 'Main phase 1', 'main2': 'Main phase 2'}


def is_human(g, p):
    ctl = getattr(g, 'controllers', None)
    return bool(ctl) and p.key in ctl and getattr(ctl[p.key], 'human', False)


def human_main(g, p, post):
    """p's main phase, played by the person: act until they pass"""
    ctl = controller_of(g, p)
    while not g.over and p.alive:
        step = STEP_NAMES.get(getattr(g, 'step', None), 'Your turn')
        act = ctl.ask(Request('priority', f'{step}: you have priority', data={'view': build_view(g, p.key)}))
        if not isinstance(act, dict): act = {}
        if act.get('do') == 'pass': return
        why = apply(g, p, act)
        if why: ctl.tell('invalid', why)


def _hand_card(p, act):
    i = act.get('card')
    if not isinstance(i, int) or not 0 <= i < len(p.hand): return None
    return p.hand[i]


def apply(g, p, act):
    """carry out one action; returns None, or the reason it isn't legal (nothing changes then)"""
    do = act.get('do')
    if do == 'tap':
        return mana.tap(g, p, act.get('source', -1), act.get('colour'))
    if do == 'land':
        c = _hand_card(p, act)
        if c is None: return 'There is no such card in your hand.'
        why = legal.check_land(g, p, c)
        if why: return why
        p.hand.remove(c)
        ais.play_land_card(g, p, c)
        E.check_state(g)
        return None
    if do == 'use':
        i = act.get('perm')
        if not isinstance(i, int) or not 0 <= i < len(p.perms): return "You don't control that."
        return use(g, p, p.perms[i])
    if do == 'cast':
        zone = 'cmd' if act.get('zone') == 'cmd' else 'hand'
        c = p.cmd if zone == 'cmd' else _hand_card(p, act)
        if c is None: return 'There is no such card in your hand.'
        why = legal.check_cast(g, p, c, zone)
        if why: return why
        return cast(g, p, c, zone)
    return 'Unknown action.'


def choose(g, p, kind, prompt, labels):
    """ask the person to pick one of labels; returns its index, or None if they cancel"""
    ctl = controller_of(g, p)
    for _ in range(5):
        ans = ctl.ask(Request(kind, prompt, choices=list(labels) + ['cancel']))
        if ans == 'cancel' or ans == len(labels): return None
        if isinstance(ans, int) and not isinstance(ans, bool) and 0 <= ans < len(labels): return ans
        ctl.tell('invalid', 'Pick one of the numbers, or cancel.')
    return None                                               # five bad answers in a row: treated as cancel


def abilities_of(g, p, m):
    """[(label, kind, fn)]: what p can activate on permanent m now. Equip, and the abilities its card code offers
    (planeswalker loyalty abilities, Triskelion, Deathrite ...)"""
    from commander_sim.ai import brain
    out = []
    if legal.equip_cost(m) is not None: out.append((f'Equip {{{legal.equip_cost(m)}}}', 'equip', None))
    post = getattr(g, 'step', None) == 'main2'
    if g.hooks:
        s = brain.Situation(g, p)
        for src, fn in E.CI.hooked(g, 'options'):
            if src is m:
                for u, label, f in fn(g, src, p, s, post) or []:
                    out.append((label, 'hook', f))
    return out


def use(g, p, m):
    abil = abilities_of(g, p, m)
    if not abil: return f'{m.name} has no ability you can activate right now.'
    k = choose(g, p, 'choose', f'{m.name}: which ability?', [a[0] for a in abil])
    if k is None: return None
    label, kind, fn = abil[k]
    if kind == 'equip':
        cre = [x for x in p.perms if x.creature and not x.phased and x is not m]
        if not cre: return 'You have no creature to equip.'
        j = choose(g, p, 'target', f'Equip {m.name} onto which creature?', [legal.describe_target(g, p, x) for x in cre])
        if j is None: return None
        why = legal.check_equip(g, p, m, cre[j])
        if why: return why
        mana.pay_from_pool(g, p, legal.equip_cost(m), '')
        m.attached = cre[j]
        E.log(f'  {E.NAME(p)} equips {m.name} to {cre[j].name}', g)
        return None
    if not fn(): return f"{label}: that can't be done right now."
    E.check_state(g)
    return None


def cast(g, p, c, zone):
    """cast c (already checked legal): extra costs, pay from the pool, then the engine casts and resolves it. Choices
    not yet made by the player (X, a creature to sacrifice, targets) are made by the AI and reported as automatic"""
    from commander_sim.ai import brain
    ctl = controller_of(g, p)
    gen, pips = legal.base_cost(g, p, c)
    ctx = {}
    tgts = legal.spell_targets(g, p, c)
    if tgts is not None:                                     # targets are chosen before the spell is paid for
        if not tgts: return f'{c.name} has no legal target right now.'
        what = legal.WHAT.get(c.tags.get('tgt', 'c'), 'target') + (' or player' if 'face' in c.tags else '')
        k = choose(g, p, 'target', f'{c.name}: choose a target ({what})', [legal.describe_target(g, p, x) for x in tgts])
        if k is None: return None
        x = tgts[k]
        eg, ep = legal.target_extra_cost(g, p, c, x)
        gen, pips = gen + eg, pips + ep
        why = mana.cost_problem(g, p, gen, pips)
        if why: return f"Can't cast {c.name} on that target. {why}"
        if isinstance(x, E.Player): ctx['face'] = x
        else: ctx['target'] = x
    fodder = None
    if 'needsac' in c.tags:                                  # Diabolic Intent: sacrifice a creature
        fodder = brain.spare_creature(g, p)
        if fodder is None: return f'{c.name} needs a creature to sacrifice as it is cast, and you have none you can spare.'
    if c.dsl and not E.additional_cost(g, p, c, dry=True): return f"You can't pay {c.name}'s additional cost."
    why = mana.pay_from_pool(g, p, gen, pips)
    if why: return f"Can't cast {c.name}. {why}"
    if 'tokx' in c.tags or 'xtutor' in c.tags:              # X: everything left in the pool (automatic for now)
        x = mana.pool_of(p).total(); mana.pool_of(p).empty(); ctx['x'] = x
        if 'xtutor' in c.tags: g.last_x = x
        ctl.tell('auto', f'X = {x} (the rest of your mana pool)')
    if c.dsl: E.additional_cost(g, p, c)
    if fodder is not None:
        E.die(g, fodder, 'sac'); ctl.tell('auto', f'Sacrificed {fodder.name} for {c.name}')
    if 'phyU' in c.tags and 'U' in c.pips and pips.count('U') < c.pips.count('U'):
        E.lose_life(g, p, 2, p)                              # {U/P} paid with 2 life
    E.cast_card(g, p, c, zone, ctx)
    return None
