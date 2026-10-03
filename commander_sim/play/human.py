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


def stack_view(g):
    """the stack for the page, top first: each item's card, controller and what it targets"""
    out = []
    for it in reversed(g.stack):
        d = {'name': it.card.name if it.card is not None else it.name, 'controller': E.NAME(it.controller),
             'key': it.controller.key, 'kind': it.kind}
        if it.kind != 'spell': d['text'] = it.name
        t = it.ctx.get('counter')
        if t is not None: d['target'] = t.card.name
        elif it.ctx.get('target') is not None: d['target'] = getattr(it.ctx['target'], 'name', '')
        out.append(d)
    return out


AUTOPASS = ('respond', 'stack', 'all')     # stop when I can respond (the default) / on every stack item / every step


def autopass(g, q):
    """q's auto-pass setting: 'respond' (asked only when you could do something, plus opponents' spells, attacks on
    you and the end of each other turn), 'stack' (every spell, ability and trigger too) or 'all' (every step)"""
    ctl = controller_of(g, q)
    mode = getattr(ctl, 'autopass', None)
    return mode if mode in AUTOPASS else 'respond'


def stack_priority(g, q, item):
    """q (the person) gets priority with something on the stack. Your own spell: only when you hold a card that
    copies it, or with auto-pass off; anyone else's: always. An ability or trigger: when you could do something about
    it, or always with auto-pass at 'stack' or 'all'"""
    mode = autopass(g, q)
    top = g.stack[-1] if g.stack else item
    if top.kind != 'spell':
        if top.controller is q and mode != 'all': return
        if mode == 'respond' and not can_respond(g, q): return
        if top.controller is q: who = 'You activate' if top.kind == 'ability' else 'Your trigger:'
        else: who = E.NAME(top.controller) + (' activates' if top.kind == 'ability' else ' has a trigger:')
        respond(g, q, f'{who} {top.name}', spell=top.card, caster=top.controller)
        return
    if top.controller is q and mode != 'all' and not E._copy_window(g, q, top.card): return
    who = 'You cast' if top.controller is q else f'{E.NAME(top.controller)} casts'
    t = top.ctx.get('counter')
    what = f'{top.card.name} (countering {t.name})' if t is not None else top.card.name
    respond(g, q, f'{who} {what}', spell=top.card, caster=top.controller)


def step_priority(g, q, step, defender=None, attackers=()):
    """q (the person) has priority in a step of the turn (engine.step_priority), when their auto-pass setting stops
    there: with 'all', every step; otherwise when attacked, at the end of each other player's turn, and in the declare
    blockers step of a combat they're in when they could do something"""
    a = g.active
    mode = autopass(g, q)
    if mode != 'all':
        if step == 'end': stop = a is not q
        elif step == 'attackers': stop = q is defender
        elif step == 'blockers': stop = (q is a or q is defender) and can_act(g, q)
        else: stop = False
        if not stop: return
    if step == 'end': prompt = 'End of your turn' if a is q else f"End of {E.NAME(a)}'s turn"
    elif step == 'attackers' and q is defender:
        prompt = f'{E.NAME(a)} attacks you with {len(attackers)} creature(s) ({sum(E.epow(g, m) for m in attackers)} power)'
    else:
        prompt = f"{'Your' if a is q else E.NAME(a) + chr(39) + 's'} {E.STEP_NAMES[step]}"
    respond(g, q, prompt)


def can_act(g, q):
    """anything q could do at instant speed now: an instant or flash card they can afford, or an ability"""
    if can_respond(g, q): return True
    from commander_sim.play import abilities, cards
    g.responding = getattr(g, 'responding', 0) + 1           # instant speed: sorcery-speed abilities aren't offered
    try:
        def usable(x, labels):                               # a tapped permanent's {T} abilities don't count
            return any(not (x.tapped and '{T}' in lbl) for lbl in labels)
        for m in q.perms:
            if m.phased or m.cd is None: continue
            if usable(m, [lbl for lbl, _ in abilities.permanent_abilities(g, q, m) + (cards.abilities(g, q, m) or [])]):
                return True
        return any(usable(L, [lbl for lbl, _ in abilities.land_abilities(g, q, L)]) for L in q.lands)
    finally:
        g.responding -= 1


def can_respond(g, q):
    """does q hold anything to do at instant speed right now (an instant or flash card they can afford, an ability
    counter)?"""
    have = E.total_mana(g, q) + mana.pool_of(q).total()
    if any((c.instant or 'flash' in c.tags or 'flash' in getattr(c, 'kws', ())) and c.cmc <= have and not c.land for c in q.hand):
        return True
    return any(m.cd is not None and m.cd.name in E.ABILITY_ANSWERS for m in q.perms)


def respond(g, q, prompt, spell=None, caster=None):
    """q (the person) has priority in response to something (a spell on the stack, attackers, the end of a turn).
    They may tap mana, cast instants and flash spells (a counterspell goes on the stack aimed at a spell there), use
    instant-speed abilities, or pass. Returns once they pass, or once something they did went on the stack (it has
    resolved by then, after its own round of priority)"""
    ctl = controller_of(g, q)
    order = []                                               # who gets priority on it, in turn order after the caster
    if caster is not None:
        order = [{'key': x.key, 'name': E.NAME(x)} for x in g.after(caster) if x.alive]
    g.responding = getattr(g, 'responding', 0) + 1           # something is waiting to resolve: no sorcery-speed play
    try:
        while not g.over and q.alive:
            stack = stack_view(g) if g.stack else ([{'name': spell.name}] if spell is not None else [])
            act = ctl.ask(Request('priority', f'{prompt}. You have priority',
                                  data={'view': build_view(g, q.key), 'stack': stack, 'order': order,
                                        'caster': None if caster is None else E.NAME(caster)}))
            if not isinstance(act, dict): act = {}
            if act.get('do') == 'pass': return None
            n = getattr(g, 'stack_pushes', 0)
            if act.get('do') == 'cast' and g.stack and act.get('zone') != 'cmd':
                c = _hand_card(q, act)
                if c is not None and 'ctr' in c.tags:
                    why = cast_counterspell(g, q, c)
                    if why: ctl.tell('invalid', why); continue
                    return None
                from commander_sim.play import cards
                if c is not None and cards.copy_card(c):          # Return the Favor, Dualcaster Mage: copy the spell
                    top = g.stack[-1]
                    why = legal.check_cast(g, q, c) if c.name != 'Return the Favor' else None
                    why = why or cards.copy_in_response(g, q, c, top.card, top.controller)
                    if why: ctl.tell('invalid', why)
                    continue
            why = apply(g, q, act)
            if why: ctl.tell('invalid', why)
            if getattr(g, 'stack_pushes', 0) != n and g.stack: return None    # it went on the stack and resolved
        return None
    finally:
        g.responding -= 1


def cast_counterspell(g, q, c):
    """q casts counterspell c at a spell on the stack (the top one, or one they pick when there are several it can
    counter). None, or why not"""
    cands = [it for it in reversed(g.stack) if it.controller is not q or len(g.stack) > 1]
    cands = [it for it in cands if E.counter_ok(c, it.card) and E.counterable(g, it)]
    if not cands: return f"{c.name} can't counter anything on the stack."
    it = cands[0]
    if len(cands) > 1:
        k = choose(g, q, 'target', f'{c.name}: counter which spell?',
                   [f"{x.card.name} ({E.NAME(x.controller)})" for x in cands])
        if k is None: return 'Cancelled.'
        it = cands[k]
    why = legal.check_counter(g, q, c, it.card)
    if why: return why
    alt = legal.alternative_counter_cost(g, q, c)
    gen, pips = E.counter_cost(c, it.card)
    if mana.cost_problem(g, q, gen, pips) is None:            # pay from your pool
        mana.pay_from_pool(g, q, gen, pips)
        g.free_counter = True                                 # already paid
    elif not alt: return f"Can't cast {c.name}. {mana.cost_problem(g, q, gen, pips)}"
    try:
        if not E.cast_counter_spell(g, q, c, it): return f"{c.name} couldn't be cast."
    finally:
        g.free_counter = False
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
    if do == 'land' and act.get('zone') == 'gy':
        i = act.get('card')
        c = p.gy[i] if isinstance(i, int) and 0 <= i < len(p.gy) else None
        if c is None: return 'There is no such card in your graveyard.'
        why = legal.check_land_gy(g, p, c)
        if why: return why
        if not (getattr(p, 'yawg', False) and id(c) in getattr(p, 'yawg_gy', {})): E.CI.muld_mark(g, p, 'L')   # Muldrotha
        p.gy.remove(c); ais.play_land_card(g, p, c, 'plays from the graveyard'); E.check_state(g)
        return None
    if do == 'cast' and act.get('zone') == 'gy':
        i = act.get('card')
        c = p.gy[i] if isinstance(i, int) and 0 <= i < len(p.gy) else None
        if c is None: return 'There is no such card in your graveyard.'
        why = legal.check_cast_gy(g, p, c)
        if why: return why
        from commander_sim.play import cards
        if c.name in cards.GY: return cards.GY[c.name](g, p, c)          # Momentary Blink's flashback, Demonic Embrace
        return cast(g, p, c, 'gy')
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
        from commander_sim.play import cards
        if zone == 'hand' and c.name in cards.HAND: return cards.cast_from_hand(g, p, c)
        why = legal.check_cast(g, p, c, zone)
        if why: return why
        return cast(g, p, c, zone)
    return 'Unknown action.'


def choose(g, p, kind, prompt, labels, cancel='cancel', data=None):
    """ask the person to pick one of labels; returns its index, or None if they cancel (the last choice, named by
    `cancel`, e.g. 'no block'). cancel=None: a forced choice with no way out (after five bad answers: the first)"""
    ctl = controller_of(g, p)
    shown = list(labels) + ([cancel] if cancel is not None else [])
    live = getattr(ctl, 'notify', None) is not None              # a person at the browser or terminal
    refs = table_refs(g, p, labels) if live else []
    data = dict(data or {}, **({'refs': refs} if any(refs) else {}))
    if live: data['view'] = build_view(g, p.key)                  # the table as it is now (the step, the pool)
    for _ in range(5):
        ans = ctl.ask(Request(kind, prompt, choices=shown, data=data))
        if cancel is not None and (ans == 'cancel' or ans == len(labels)): return None
        if isinstance(ans, int) and not isinstance(ans, bool) and 0 <= ans < len(labels): return ans
        ctl.tell('invalid', 'Pick one of the numbers' + (', or cancel.' if cancel is not None else '.'))
    return None if cancel is not None else 0


def table_refs(g, p, labels):
    """for the browser: where each choice is on the table, so it can be clicked there. A label naming a permanent
    ('Sheoldred, the Apocalypse (Prosper, tapped)', with or without more after it) -> {'seat', 'perm'}; a player ->
    {'player'}; a card in p's hand -> {'hand'}. None when a label isn't on the table (a mode, a card in a library)"""
    where = {}
    for q in g.players:
        if not q.alive: continue
        where.setdefault(legal.describe_target(g, p, q), {'player': q.key})
        for i, m in enumerate(q.perms):
            where.setdefault(legal.describe_target(g, p, m), {'seat': q.key, 'perm': i})
    from commander_sim.play.choices import card_label
    hand = {}
    for i, c in enumerate(p.hand): hand.setdefault(card_label(c), {'hand': i})
    out = []
    for lb in labels:
        ref = where.get(lb) or hand.get(lb)
        if ref is None and ' (' in lb:                       # 'Archmage Emeritus (Veyran, 2/2) (X = 4)'
            ref = next((r for k, r in where.items() if lb.startswith(k + ' ')), None)
        out.append(ref)
    return out


def abilities_of(g, p, m):
    """[(label, kind, fn)]: what p can activate on permanent m now. Equip, and the abilities its card code offers
    (planeswalker loyalty abilities, Triskelion, Deathrite ...)"""
    from commander_sim.ai import brain
    out = []
    if legal.equip_cost(m) is not None: out.append((f'Equip {{{legal.equip_cost(m)}}}', 'equip', None))
    post = (getattr(g, 'step', None) == 'main2') if g.active is p else None    # None: instant-speed abilities only
    from commander_sim.play import abilities, cards
    for label, f in abilities.permanent_abilities(g, p, m):
        out.append((label, 'extra', f))
    own = cards.abilities(g, p, m)
    if own is not None:                                  # your deck's card: its abilities by the rules
        return out + [(label, 'extra', f) for label, f in own]
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
        E.log(f'  {E.NAME(p)} equips {m.name} to {cre[j].name}', g)
        if E.ability_window(g, p, m, f'equip to {cre[j].name}', target=cre[j]) and cre[j] in p.perms and m in p.perms:
            m.attached = cre[j]
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
    """cast card c from zone (its costs from your pool); restricted mana (Secluded Courtyard) may pay for it if it's a
    creature spell of the named type"""
    prev, mana.SPENDING = mana.SPENDING, c
    try:
        return _cast(g, p, c, zone)
    finally:
        mana.SPENDING = prev


def _cast(g, p, c, zone):
    """cast c (already checked legal): target and extra costs, pay from the pool, then the engine casts and resolves
    it. Choices not yet made by the player (X for now) are made automatically and reported"""
    ctl = controller_of(g, p)
    gen, pips = legal.base_cost(g, p, c)
    ctx = {}
    gymode = legal.gy_mode(g, p, c)[0] if zone == 'gy' else None
    if zone == 'gy':
        _, gen, pips = legal.gy_mode(g, p, c)
        if gymode == 'escape': zone = 'escape'               # Underworld Breach: back to the graveyard afterwards
        if gymode == 'muldrotha':                            # which of its types this uses up (an artifact creature ...)
            ts = E.CI.muld_types(g, p, c)
            names = dict(E.CI.MULD_TYPES)
            k = choose(g, p, 'choose', f'{c.name}: cast it with Muldrotha as which type?', [names[t] for t in ts]) if len(ts) > 1 else 0
            if k is None: return None
            ctx['muld_type'] = ts[k]; zone = 'mgy'
    if 'wipe' in c.tags and 'rem' in c.tags:                 # overload (Cyclonic Rift, Vandalblast)
        og, op = ais.wipe_cost(p, c)
        over = f'overloaded ({mana.cost_text(og, op)}): every one you don\'t control'
        if legal.spell_targets(g, p, c):
            k = choose(g, p, 'choose', f'{c.name}: cast it how?', [f'one target ({mana.cost_text(gen, pips)})', over])
        else:                                                # nothing to target: only the overloaded mode is legal
            k = choose(g, p, 'choose', f'{c.name}: nothing to target, so it can only be cast overloaded', [over])
            k = None if k is None else 1
        if k is None: return None
        if k == 1:
            why = mana.pay_from_pool(g, p, og, op)
            if why: return f"Can't overload {c.name}. {why}"
            E.log(f'  {E.NAME(p)} casts {c.name} overloaded', g)
            E.cast_card(g, p, c, zone, {})
            return None
    if 'mastery' in c.tags:                                  # Mizzix's Mastery: one target, or overloaded
        from commander_sim.play import cards
        k = choose(g, p, 'choose', f'{c.name}: cast it how?',
                   [f'one target ({mana.cost_text(gen, pips)})',
                    'overloaded ({5}{R}{R}{R}): every instant and sorcery card in your graveyard'])
        if k is None: return None
        if k == 1: gen, pips = 5, 'RRR'; ctx['overload'] = True
        else:
            pick = cards.mastery_target(g, p, c)
            if pick is None: return None
            ctx['mastery_pick'] = pick
    from commander_sim.play import cards as mycards
    if c.name in mycards.CAST_TARGET:                         # a tagless spell with a creature target (Act of Treason)
        spec = mycards.CAST_TARGET[c.name]
        prompt, keep = spec if isinstance(spec, tuple) else (spec, None)
        t = mycards.pick_creature(g, p, prompt, optional=True, keep=keep)
        if t is None: return None
        ctx['target'] = t
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
        if c.name == 'Necromancy' and zone == 'hand' and legal.sorcery_timing(g, p):
            ctx['flash_sac'] = True                          # cast as though it had flash: sacrificed at cleanup
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
    life = 0
    if legal.phyrexian(c):                                   # {B/P}: its colour, or 2 life (your choice)
        ways = legal.phyrexian_ways(g, p, c, gen, pips)
        if not ways:
            why = mana.cost_problem(g, p, gen, pips)
            return f"Can't cast {c.name}. {why} Its Phyrexian mana can be paid with 2 life each instead."
        k = 0
        if len(ways) > 1:
            phy = legal.phyrexian(c)
            sym = ''.join('{' + x + '/P}' for x in phy)
            k = choose(g, p, 'choose', f'{c.name}: pay {sym} with mana or life?',
                       [' and '.join(x for x in (''.join('{' + y + '}' for y in phy[paid // 2:]),
                                                 f'{paid} life' if paid else '') if x) for paid, _ in ways])
            if k is None: return None
        life, pips = ways[k]
    why = mana.pay_from_pool(g, p, gen, pips)
    if why: return f"Can't cast {c.name}. {why}"
    if life:
        E.lose_life(g, p, life, p)
        E.log(f'  {E.NAME(p)} pays {life} life for {c.name}\'s Phyrexian mana', g)
    if 'crackle' in c.tags:                                  # {X}{X}{X}{R}{R}: X from what's left in the pool
        top = mana.pool_of(p).total() // 3
        k = choose(g, p, 'choose', f'{c.name}: choose X (it costs {{X}}{{X}}{{X}} more; 5X damage to each of up to X targets)',
                   [f'X = {x}' for x in range(top + 1)], cancel=None)
        mana.pay_from_pool(g, p, 3 * k, ''); ctx['x'] = k
    if 'tokx' in c.tags or 'xtutor' in c.tags or 'xdrain' in c.tags:   # X: up to what's left in the pool
        top = mana.pool_of(p).total()
        x = choose(g, p, 'choose', f'{c.name}: choose X (paid from what is left in your pool)',
                   [f'X = {x}' for x in range(top + 1)], cancel=None) if top else 0
        mana.pay_from_pool(g, p, x, ''); ctx['x'] = x
        if 'xtutor' in c.tags: g.last_x = x
    if c.dsl: E.additional_cost(g, p, c)
    if fodder is not None:
        E.log(f'  {E.NAME(p)} sacrifices {fodder.name} for {c.name}', g); E.die(g, fodder, 'sac')
    if gymode == 'sac3':                                     # Dread Return's flashback: sacrifice three creatures
        from commander_sim.play import choices
        for i in range(3):
            cre = [m for m in p.perms if m.creature and not m.phased]
            choices.sacrifice_creature(g, p, cre, f'{c.name} flashback: sacrifice a creature ({i + 1} of 3)')
    if gymode == 'flashback' and 'fblife' in c.tags: E.lose_life(g, p, int(c.tags['fblife']), p)
    if gymode == 'escape':                                   # exile three other cards from your graveyard
        from commander_sim.play import choices
        for x in choices.pick_cards(g, p, [y for y in p.gy if y is not c], 3, f'Escape {c.name}: exile a card from your graveyard'):
            p.gy.remove(x); p.exile.append(x)
    if ctx.pop('sac_cost', False):
        from commander_sim.play import choices
        choices.sac_artifact_or_creature(g, p, c.name)
    if ctx.pop('discard_cost', False):
        from commander_sim.play import choices
        x = choices.pick_cards(g, p, [y for y in p.hand if y is not c], 1, f'{c.name}: discard a card')[0]
        E.discard_cards(g, p, [x]); ctx['paid_otherwise'] = True
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
    n0 = len(p.perms)
    ais.seph_rean_resolve(g, p, c, ctx)
    if ctx.get('flash_sac'):                                 # Necromancy at instant speed: its enter effects, then gone
        for m in [m for m in p.perms[n0:] if m.cd is cd]:
            E.log(f'    {m.name} is sacrificed (Necromancy was cast at instant speed)', g); E.die(g, m, 'sac')
    dest.append(c)
    E.check_state(g)
    return None
