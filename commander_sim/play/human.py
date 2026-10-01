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


def respond(g, q, prompt, spell=None):
    """q (the person) has priority in response to something (a spell on the stack, attackers, the end of a turn).
    They may tap mana, cast instants and flash spells, use instant-speed abilities, or pass. Returns the counterspell
    they cast at `spell` (the engine then counters it), or None when they pass"""
    ctl = controller_of(g, q)
    stack = [{'name': spell.name}] if spell is not None else []
    while not g.over and q.alive:
        act = ctl.ask(Request('priority', f'{prompt}. You have priority', data={'view': build_view(g, q.key), 'stack': stack}))
        if not isinstance(act, dict): act = {}
        if act.get('do') == 'pass': return None
        if act.get('do') == 'cast' and spell is not None and act.get('zone') != 'cmd':
            c = _hand_card(q, act)
            if c is not None and 'ctr' in c.tags:
                why = legal.check_counter(g, q, c, spell)
                if why: ctl.tell('invalid', why); continue
                return c
        why = apply(g, q, act)
        if why: ctl.tell('invalid', why)
    return None


def humans(g):
    """the human seats still in the game"""
    ctl = getattr(g, 'controllers', None)
    return [p for p in g.players if ctl and p.key in ctl and getattr(ctl[p.key], 'human', False) and p.alive]


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
        if 'land' in act:
            i = act.get('land')
            if not isinstance(i, int) or not 0 <= i < len(p.lands): return "You don't control that land."
            return use_land(g, p, p.lands[i])
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


def choose(g, p, kind, prompt, labels, cancel='cancel'):
    """ask the person to pick one of labels; returns its index, or None if they cancel (the last choice, named by
    `cancel`, e.g. 'no block'). cancel=None: a forced choice with no way out (after five bad answers: the first)"""
    ctl = controller_of(g, p)
    shown = list(labels) + ([cancel] if cancel is not None else [])
    for _ in range(5):
        ans = ctl.ask(Request(kind, prompt, choices=shown))
        if cancel is not None and (ans == 'cancel' or ans == len(labels)): return None
        if isinstance(ans, int) and not isinstance(ans, bool) and 0 <= ans < len(labels): return ans
        ctl.tell('invalid', 'Pick one of the numbers' + (', or cancel.' if cancel is not None else '.'))
    return None if cancel is not None else 0


def abilities_of(g, p, m):
    """[(label, kind, fn)]: what p can activate on permanent m now. Equip, and the abilities its card code offers
    (planeswalker loyalty abilities, Triskelion, Deathrite ...)"""
    from commander_sim.ai import brain
    out = []
    if legal.equip_cost(m) is not None: out.append((f'Equip {{{legal.equip_cost(m)}}}', 'equip', None))
    post = (getattr(g, 'step', None) == 'main2') if g.active is p else None    # None: instant-speed abilities only
    from commander_sim.play import abilities
    for label, f in abilities.permanent_abilities(g, p, m):
        out.append((label, 'extra', f))
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
    if kind == 'extra': return fn(g, p, m)
    if not fn(): return f"{label}: that can't be done right now."
    E.check_state(g)
    return None


def use_land(g, p, L):
    """an activated ability of land L (beyond tapping for mana)"""
    from commander_sim.play import abilities
    abil = abilities.land_abilities(g, p, L)
    if not abil: return f'{L.cd.name} has no ability to activate beyond mana (tap it for mana with tap).'
    k = choose(g, p, 'choose', f'{L.cd.name}: which ability?', [a[0] for a in abil])
    if k is None: return None
    return abil[k][1](g, p, L)


def cast(g, p, c, zone):
    """cast c (already checked legal): target and extra costs, pay from the pool, then the engine casts and resolves
    it. Choices not yet made by the player (X for now) are made automatically and reported"""
    ctl = controller_of(g, p)
    gen, pips = legal.base_cost(g, p, c)
    ctx = {}
    if 'wipe' in c.tags and 'rem' in c.tags:                 # overload (Cyclonic Rift, Vandalblast)
        og, op = ais.wipe_cost(p, c)
        k = choose(g, p, 'choose', f'{c.name}: cast it how?',
                   [f'one target ({mana.cost_text(gen, pips)})', f'overloaded ({mana.cost_text(og, op)}): every one you don\'t control'])
        if k is None: return None
        if k == 1:
            why = mana.pay_from_pool(g, p, og, op)
            if why: return f"Can't overload {c.name}. {why}"
            E.log(f'  {E.NAME(p)} casts {c.name} overloaded', g)
            E.cast_card(g, p, c, zone, {})
            return None
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
    if 'rean' in c.tags:                                     # reanimation: its target is chosen as it's cast
        from commander_sim.play import choices
        pick = choices.choose_rean(g, p, c)
        if pick is None: return None
        ctx['rean_target'], ctx['rean_src'] = pick
        ctx['rean_value'] = max(4, pick[0].cmc)              # how hard opponents try to stop it
    if c.name == 'Deadly Dispute': ctx['sac_cost'] = True
    if 'deluge' in c.tags:                                   # Toxic Deluge: pay X life, all creatures get -X/-X
        top = max(0, min(p.life - 1, 20))
        k = choose(g, p, 'choose', f'{c.name}: choose X (you pay X life; creatures with toughness X or less die)',
                   [f'X = {x}' for x in range(top + 1)])
        if k is None: return None
        ctx['deluge_x'] = k
    if c.name == 'Bitter Triumph':                           # additional cost: discard a card, or pay 3 life
        others = [x for x in p.hand if x is not c]
        k = choose(g, p, 'choose', f'{c.name}: pay its extra cost how?',
                   ['pay 3 life'] + (['discard a card'] if others else []))
        if k is None: return None
        if k == 1: ctx['discard_cost'] = True
    fodder = None
    if 'needsac' in c.tags:                                  # Diabolic Intent: sacrifice a creature as you cast it
        cre = [m for m in p.perms if m.creature and not m.phased]
        if not cre: return f'{c.name} needs a creature to sacrifice as you cast it, and you control none.'
        k = choose(g, p, 'choose', f'{c.name}: sacrifice which creature?', [legal.describe_target(g, p, m) for m in cre])
        if k is None: return None
        fodder = cre[k]
    if c.dsl and not E.additional_cost(g, p, c, dry=True): return f"You can't pay {c.name}'s additional cost."
    why = mana.pay_from_pool(g, p, gen, pips)
    if why: return f"Can't cast {c.name}. {why}"
    if 'tokx' in c.tags or 'xtutor' in c.tags:              # X: everything left in the pool (automatic for now)
        x = mana.pool_of(p).total(); mana.pool_of(p).empty(); ctx['x'] = x
        if 'xtutor' in c.tags: g.last_x = x
        ctl.tell('auto', f'X = {x} (the rest of your mana pool)')
    if c.dsl: E.additional_cost(g, p, c)
    if fodder is not None:
        E.log(f'  {E.NAME(p)} sacrifices {fodder.name} for {c.name}', g); E.die(g, fodder, 'sac')
    if ctx.pop('sac_cost', False):
        from commander_sim.play import choices
        choices.sac_artifact_or_creature(g, p, c.name)
    if ctx.pop('discard_cost', False):
        from commander_sim.play import choices
        x = choices.pick_cards(g, p, [y for y in p.hand if y is not c], 1, f'{c.name}: discard a card')[0]
        E.discard_cards(g, p, [x]); ctx['paid_otherwise'] = True
    if 'phyU' in c.tags and 'U' in c.pips and pips.count('U') < c.pips.count('U'):
        E.lose_life(g, p, 2, p)                              # {U/P} paid with 2 life
    if 'rean_target' in ctx: return cast_rean(g, p, c, zone, ctx)
    E.cast_card(g, p, c, zone, ctx)
    return None


def cast_rean(g, p, c, zone, ctx):
    """a reanimation spell (paid): the same steps the AI's reanimation takes, with the person's target. Auras like
    Animate Dead are modelled as the effect only, as the AI's are"""
    from commander_sim import ais
    (p.hand if zone == 'hand' else p.gy).remove(c)
    p.spells_this_turn += 1; p.stats['spells_cast'] += 1; p.cast_names.add(c.name)
    E.on_cast(g, p, c)
    cd, src = ctx['rean_target'], ctx['rean_src']
    E.log(f'  {E.NAME(p)} casts {c.name} targeting {cd.name}' + (f" in {E.NAME(src)}'s graveyard" if src is not p else ''), g)
    dest = p.gy if zone == 'hand' else p.exile
    if g.over or not p.alive: return None
    if not E.counter_window(g, p, c, ctx['rean_value'], {}) or ais.sauron_grounds_response(g, p, ctx['rean_value']) \
            or (g.hooks and E.CI.gy_response(g, p, ctx['rean_value'], src)):
        dest.append(c); return None
    ais.seph_rean_resolve(g, p, c, ctx)
    dest.append(c)
    E.check_state(g)
    return None
