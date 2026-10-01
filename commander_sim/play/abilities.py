"""Activated abilities of your decks' cards that the engine keeps in deck AI code rather than in card hooks (the AI
uses them through its own plans). For the human seat they are abilities like any other: listed on the permanent or
land, checked for timing, tapping and mana, then carried out. Each effect returns None or the reason it can't."""
from commander_sim import engine as E, ais
from commander_sim.play import legal, mana


def _tap_ok(m, creature=False):
    if m.tapped: return f'{m.cd.name} is tapped.'
    if creature and getattr(m, 'sick', False): return f'{m.cd.name} has summoning sickness (it came under your control this turn).'
    return None


def _pay(g, p, gen, pips, what):
    why = mana.pay_from_pool(g, p, gen, pips)
    return f"Can't activate {what}. {why}" if why else None


# ------------------------------------------------------------------ permanents
def archivist(g, p, m):
    """Jace's Archivist: {U}, {T}: each player discards their hand, then draws as many as the most discarded"""
    why = _tap_ok(m, creature=True) or _pay(g, p, 0, 'U', "Jace's Archivist")
    if why: return why
    m.tapped = True
    n = max(len(q.hand) for q in g.players if q.alive)
    E.log(f"  {E.NAME(p)} activates Jace's Archivist: everyone discards and draws {n}", g)
    for q in g.players:
        if q.alive: E.discard_cards(g, q, list(q.hand))
    for q in g.players:
        if q.alive: E.draw(g, q, n)
    E.check_state(g)
    return None


def assault(g, p, m):
    """Aggravated Assault: {3}{R}{R}: untap your creatures; after this main phase, an additional combat (followed by a
    main phase). Sorcery speed"""
    why = legal.sorcery_timing(g, p)
    if why: return why.replace('do that', 'activate Aggravated Assault')
    why = _pay(g, p, 3, 'RR', 'Aggravated Assault')
    if why: return why
    for x in p.perms:
        if x.creature: x.tapped = False
    E.log(f'  {E.NAME(p)} activates Aggravated Assault: creatures untap, an extra combat follows', g)
    if g.step == 'main1':
        p.extra_combats += 1                          # it comes right after this turn's combat
    else:
        ais.combat(g, p)                              # main 2: the extra combat now, then this main phase continues
    return None


def tortured(g, p, m):
    """Tortured Existence: {B}, discard a creature card: return target creature card from your graveyard to your hand"""
    from commander_sim.play import choices
    tg = [c for c in p.gy if c.creature]
    fodder = [c for c in p.hand if c.creature]
    if not tg: return 'There is no creature card in your graveyard to return.'
    if not fodder: return 'You have no creature card in hand to discard.'
    why = mana.cost_problem(g, p, 0, 'B')
    if why: return f"Can't activate Tortured Existence. {why}"
    from commander_sim.play.human import choose
    k = choose(g, p, 'target', 'Tortured Existence: return which creature card?', [choices.card_label(c) for c in tg])
    if k is None: return None                                # targets first, then the costs
    mana.pay_from_pool(g, p, 0, 'B')
    d = choices.pick_cards(g, p, fodder, 1, 'Tortured Existence: discard a creature card')[0]
    E.discard_cards(g, p, [d])
    if tg[k] in p.gy: p.gy.remove(tg[k]); p.hand.append(tg[k])
    E.log(f'  {E.NAME(p)} uses Tortured Existence: {d.name} for {tg[k].name}', g)
    return None


PERMANENT = {'archivist': ("{U}, {T}: each player discards their hand and draws that many", archivist),
             'assault': ('{3}{R}{R}: untap your creatures; an additional combat (sorcery speed)', assault)}


def loop_abilities(g, p, m):
    """Sephiroth's infinite loops as a table shortcut: on each piece of a loop that is assembled, 'start the loop'
    (the engine plays it out as its end result, after a window for opponents to answer a key piece)"""
    import importlib
    mine = importlib.import_module('commander_sim.cards.impl.mine')
    if p.key != 'seph' or m.cd is None or m.cd.name not in mine.LOOP_CARDS: return []
    out = []
    for name, keys, kills in mine.seph_loops(g, p):
        groups = next(gr for n, gr, _ in mine.LOOPS if n == name)
        if not any(m.cd.name in grp for grp in groups) and m.cd.name not in mine.OUTLETS: continue

        def go(g, p, m, name=name, keys=keys, kills=kills):
            mine.run_loop(g, p, name, keys, kills); return None
        out.append((f'start the loop: {name}' + ('' if kills else ' (no payoff on the battlefield yet)'), go))
    return out


def permanent_abilities(g, p, m):
    """[(label, fn)] for permanent m"""
    if m.cd is None: return []
    out = [(label, fn) for tag, (label, fn) in PERMANENT.items() if tag in m.cd.tags]
    if m.cd.tags.get('fill') == 'tortured':
        out.append(('{B}, discard a creature card: return a creature card from your graveyard to your hand', tortured))
    return out + loop_abilities(g, p, m)


# ------------------------------------------------------------------ lands
def passage(g, p, L):
    """Rogue's Passage: {4}, {T}: target creature can't be blocked this turn"""
    why = _tap_ok(L)
    if why: return why
    if mana.cost_problem(g, p, 4, ''): return f"Can't activate Rogue's Passage. {mana.cost_problem(g, p, 4, '')}"
    from commander_sim.play.human import choose
    cre = [m for q in g.players if q.alive for m in q.perms if m.creature and not m.phased
           and not (m.owner is not p and E.untargetable(g, m))]
    if not cre: return 'There is no creature to target.'
    k = choose(g, p, 'target', "Rogue's Passage: which creature can't be blocked this turn?",
               [legal.describe_target(g, p, x) for x in cre])
    if k is None: return None
    mana.pay_from_pool(g, p, 4, ''); L.tapped = True
    m = cre[k]; m.data = m.data or {}; m.data['unbl'] = E.turn_stamp(g)
    E.log(f"  {E.NAME(p)} activates Rogue's Passage: {m.name} can't be blocked this turn", g)
    return None


def grounds(g, p, L):
    """Scavenger Grounds: {2}, {T}, sacrifice a Desert: exile all graveyards"""
    why = _tap_ok(L)
    if why: return why
    deserts = [x for x in p.lands if x.cd.tags.get('desert')]
    why = _pay(g, p, 2, '', 'Scavenger Grounds')
    if why: return why
    L.tapped = True
    d = L if L in deserts else deserts[0]            # it is a Desert itself: sacrifice it (the usual choice)
    p.lands.remove(d); p.gy.append(d.cd)
    for q in g.players:
        if q.alive: q.exile.extend(q.gy); q.gy = []
    E.log(f'  {E.NAME(p)} activates Scavenger Grounds: all graveyards are exiled', g)
    return None


LAND = {'passage': ("{4}, {T}: target creature can't be blocked this turn", passage),
        'desert': ('{2}, {T}, sacrifice a Desert: exile all graveyards', grounds)}


def land_abilities(g, p, L):
    """[(label, fn)] for land L"""
    out = []
    for tag, (label, fn) in LAND.items():
        if L.cd.tags.get(tag) and not (tag == 'desert' and L.cd.name != 'Scavenger Grounds'): out.append((label, fn))
    return out
