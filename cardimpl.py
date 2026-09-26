"""Hand-written card implementations (Python hooks), for engines the ability language can't express:
commander abilities, locks, taxes, combos. Registered by card name with @on(name, event).

Events the engine fires (fn signatures):
  etb(g, p, m)                          m entered under p's control
  leaves(g, m)                          m left the battlefield (any way)
  dies(g, src, m, cause)                any permanent m died (src: the hooked permanent listening)
  cast(g, src, caster, c)               any player cast c
  attack(g, src, p, atk, d)             p attacked d with atk; return a list of new attacking creatures (or None)
  blocks(g, src, p, atk, d, assign)     blockers declared (assign: attacker -> blocker); may change atk
  combat_damage(g, src, p, a, d, dmg)   attacker a dealt dmg combat damage to player d
  upkeep(g, src, p) / end_step(g, src, p)   p's upkeep / end step (every hooked permanent hears every player's)
  draw(g, src, p)                       p drew a card
  landfall(g, src, p)                   a land entered under p's control
  sacrifice(g, src, p, what)            p sacrificed a permanent (what: Perm, or 'Treasure' / 'Food' / 'Clue')
  options(g, src, p, s, post)           -> [(utility, label, fn)]: activated abilities the adaptive AI may use
  cost(g, src, caster, c)               -> generic mana added to (+) or removed from (-) c's cost
  can_cast(g, src, caster, c, zone)     -> False to forbid casting c (spell limits, locks)
  attack_tax(g, src, attacker, d)       -> generic mana per attacking creature for attacks on d
  attack_cap(g, src, attacker, d)       -> max creatures that can attack d (None: no cap)
  trigger_copies(g, src, p, kind, x)    -> extra copies of a triggered ability (kind 'attack' / 'dies')
Zone hooks for cards not on the battlefield (ninjutsu from hand, recursion from the graveyard):
  hand_blocks(g, card, p, atk, d, assign)   gy_options(g, card, p, s, post)
"""
import engine as E

HOOKS = {}            # card name -> {event: fn}
SPELL_PRIO = {}       # card name -> number or fn(g, p, c): cast priority 0-90 for outside decks (0 = not now)
LAND_ETB = {}         # land name -> fn(g, p, land) when it enters as a land drop (Bojuka Bog ...)


def gy_response(g, reanimator, value, src_player=None):
    """an opponent answers a reanimation spell by exiling the graveyard (Tormod's Crypt, Soul-Guide Lantern ...)"""
    if value < 5: return False
    for src, fn in list(hooked(g, 'gy_hate')):
        if src.owner is not reanimator and fn(g, src, reanimator, src_player or reanimator): return True
    return False
DYN_MANA = {}         # card name -> fn(g, p, perm) -> amount of mana its tap ability makes (Priest of Titania ...)
ON_TAP = {}           # card name -> fn(g, p, perm, amount used) after it is tapped for mana (Heritage Druid ...)


def dyn_mana(g, p, m):
    return DYN_MANA[m.cd.name](g, p, m) if live(m.cd.name) else 1


def count_type(g, p, t, everyone=False):
    """permanents of subtype t controlled by p (or by anyone)"""
    ps = [q for q in g.players if q.alive] if everyone else [p]
    return sum(1 for q in ps for m in q.perms if not m.phased and E.has_type(m, t))


def on(name, *events):
    def deco(fn):
        for ev in events:
            HOOKS.setdefault(name, {})[ev] = fn
        return fn
    return deco


_MAIN = None


def main_cards():
    global _MAIN
    if _MAIN is None:
        from decks import DECKS
        _MAIN = frozenset(n for v in DECKS.values() for n in v)
    return _MAIN


def live(name):
    return name in HOOKS


def hooked(g, event):
    """(src permanent, fn) for every permanent on the battlefield with a hook for event"""
    if not g.hooks: return
    cache = g.hook_cache
    if cache is None: cache = g.hook_cache = {}
    lst = cache.get(event)
    if lst is None:
        lst = cache[event] = [(m, HOOKS[m.cd.name][event]) for m in g.hooks if event in HOOKS.get(m.cd.name, {})]
    for m, fn in lst:
        if m.phased or m.neutered or not m.owner.alive or m not in m.owner.perms: continue
        yield m, fn


def fire(g, event, *args):
    out = []
    for src, fn in hooked(g, event):
        reps = 1 + total(g, 'trigger_copies', src.owner, 'dies', args[0]) if event == 'dies' else 1   # Teysa
        for _ in range(reps):
            r = fn(g, src, *args)
            if r: out.append(r)
            if g.over: break
        if g.over: break
    return out


def granted_kw(g, m, kw):
    """static keyword grants from hooked permanents (Teysa: tokens have vigilance and lifelink)"""
    for src, fn in hooked(g, 'grant_kw'):
        if fn(g, src, m, kw): return True
    return False


def total(g, event, *args):
    return sum(fn(g, src, *args) or 0 for src, fn in hooked(g, event))


def allowed(g, caster, c, zone='hand'):
    for src, fn in hooked(g, 'can_cast'):
        if fn(g, src, caster, c, zone) is False: return False
    return True


def hand_cards(p, event):
    return [(c, HOOKS[c.name][event]) for c in list(p.hand) if c.name in HOOKS and event in HOOKS[c.name] and live(c.name)]


def gy_cards(p, event):
    return [(c, HOOKS[c.name][event]) for c in list(p.gy) if c.name in HOOKS and event in HOOKS[c.name] and live(c.name)]


PVAL = {}             # card name -> fixed threat value (how much opponents want it gone)
combo_options = None  # set by impl_combos
combo_imp = None
LOCK_EVENTS = ('cost', 'can_cast', 'min_cost', 'uncounterable', 'no_lifegain', 'no_graveyard')
ENGINE_EVENTS = ('options', 'cast', 'dies', 'etb', 'attack', 'upkeep', 'end_step', 'landfall', 'draw', 'sacrifice',
                 'combat_damage', 'trigger_copies', 'extra_lands', 'land_mana', 'grant_kw', 'discard', 'land_gy')


_TV = {}


def threat_value(g, m):
    return _cached_threat(g, m) + (piece_threat(g, m) if piece_threat is not None else 0)


piece_threat = None


def _cached_threat(g, m):
    """pool games: what a permanent is worth removing (beyond the engine's tag-based value); fixed per card name"""
    cd = m.cd
    if cd is None: return 0
    v = _TV.get(cd.name)
    if v is None:
        v = _TV[cd.name] = _threat_value(cd)
    return v


def spell_importance(g, p, c):
    """pool games: how much opponents want to counter spell c (0-9); 6-7 is the usual counter threshold"""
    if c.name in SPELL_IMP:
        v = SPELL_IMP[c.name]
        return v(g, p, c) if callable(v) else v
    v = 0.0
    if c.perm:
        v = max(float(c.bomb), _threat_value(c) + 1.0)
        if c.creature: v = max(v, 0.9 * c.pow + (1.0 if 'fly' in c.tags else 0) - 0.5)
        if c is p.cmd: v = max(v + 1.0, 6.0)
    else:
        t = c.tags
        if t.get('tut') or 'seal' in t: v = 5.0
        if 'draw' in t: v = max(v, 1.5 * int(t['draw']) if str(t['draw']).isdigit() else 3.0)
        if c.name in HOOKS and 'resolve' in HOOKS[c.name]: v = max(v, 4.0)
    return min(9.0, v)


SPELL_IMP = {}        # card name -> fixed spell importance (or fn(g, p, c))


def _threat_value(cd):
    if cd.name in PVAL: return PVAL[cd.name]
    v = 0.0
    h = HOOKS.get(cd.name) if live(cd.name) else None
    if h:
        if 'attack_tax' in h or 'attack_cap' in h: v = max(v, 4.5)
        if any(e in h for e in LOCK_EVENTS): v = max(v, 5.0)
        if any(e in h for e in ENGINE_EVENTS): v = max(v, 3.0)
    if cd.dsl and not cd.creature:
        n = sum(1.5 if a.get('type') in ('triggered', 'static', 'replacement') else 1.0 if a.get('type') == 'activated' else 0
                for a in cd.dsl if a.get('static') != 'note')
        v = max(v, min(6.0, 1.0 + n))
    return v


def turn_start(g, p):
    """start of p's turn: animated lands revert, land upkeep triggers, rebound spells, Pact of Negation payments"""
    import impl_lands
    if getattr(g, 'animated', None): impl_lands.revert_animated(g)
    import impl_rules2
    impl_rules2.uncrew(g, p)
    if getattr(p, 'drain_mana', 0):                   # Mana Drain: the countered spell's mana value, colourless
        p.floatC = getattr(p, 'floatC', 0) + p.drain_mana; p.drain_mana = 0
    for q in g.players:                               # Baubles: draw at the beginning of the next upkeep
        n = getattr(q, 'delayed_draws', 0)
        if n and q.alive: q.delayed_draws = 0; E.draw(g, q, n)
    impl_lands.land_upkeep(g, p)
    n = getattr(p, 'pacts', 0)
    while n > 0:
        n -= 1
        if E.can_pay(g, p, 3, 'UU'): E.pay(g, p, 3, 'UU')
        else:
            E.log(f'  {E.NAME(p)} can\'t pay for Pact of Negation and loses', g)
            p.life = 0; p.last_src = None; E.check_state(g); break
    p.pacts = 0
    if getattr(p, 'sagas', None):
        import impl_common; impl_common.saga_step(g, p)
    reb = getattr(p, 'rebound', None)
    if reb:
        p.rebound = []
        for c in reb:
            if c in p.exile and 'rebound' in HOOKS.get(c.name, {}): HOOKS[c.name]['rebound'](g, p, c)


def adjust_mana(g, p, U):
    """mana locks and bonuses (Karn + Lattice, Collector Ouphe, Cursed Totem, Kinnan, Urza)"""
    if total(g, 'mana_lock', p): return []
    if total(g, 'no_creature_mana', p):
        U = [u for u in U if not (isinstance(u[0], E.Perm) and u[0].creature)]
    bonus = total(g, 'nonland_mana_bonus', p)
    if bonus:
        for u in U:
            if isinstance(u[0], E.Perm) or u[0] == 'T': u[2] += bonus
    for src, fn in hooked(g, 'extra_mana'):
        if src.owner is p: U += fn(g, src, p, U)
    if any(c.name == 'Elvish Spirit Guide' for c in p.hand):
        import impl_partials; U += impl_partials.hand_mana(g, p)
    return U


def blood_moon(g):
    return bool(list(hooked(g, 'blood_moon')))


def become_monarch(g, p):
    if getattr(g, 'monarch', None) is p or not p.alive: return
    g.monarch = p
    E.log(f'    {E.NAME(p)} becomes the monarch', g)
    if g.hooks: fire(g, 'monarch', p)


def load():
    """import the implementation modules (they register themselves)"""
    import impl_common, impl_t1, impl_t2, impl_t3, impl_t4, impl_t5, impl_combos, impl_topdeck, impl_fixes, impl_lands, impl_partials, impl_rules, impl_rules2, impl_mine  # noqa: F401


E.CI = __import__('sys').modules[__name__]


# ------------------------------------------------------------------ attack keywords (compiled cards)
def keyword_attack(g, p, atk, d):
    """battle cry, mentor, dethrone, exalted, myriad-free subset; returns new attacking creatures"""
    new = []
    for m in atk:                                   # creature lands: Raging Ravine grows, Hive exiles a card
        if m.token and m.data and 'land' in m.data:
            if m.name == 'Raging Ravine': m.plus += 1
            elif m.name == 'Hive of the Eye Tyrant' and d.gy:
                x = max(d.gy, key=lambda c: (c.creature, c.cmc)); d.gy.remove(x); d.exile.append(x)
    if not any(m.cd is not None and m.cd.kws for m in atk) and not any(
            m.cd is not None and 'exalted' in m.cd.kws for m in p.perms):
        return new
    for m in list(atk):
        if m.cd is None or m not in p.perms: continue
        k = m.cd.kws
        if 'battle cry' in k:
            for x in atk:
                if x is not m: _eot(g, x, 1, 0)
        if 'mentor' in k:
            lesser = [x for x in atk if x is not m and x in p.perms and E.epow(g, x) < E.epow(g, m)]
            if lesser: max(lesser, key=lambda x: E.epow(g, x)).plus += 1
        if 'dethrone' in k and d.life >= max(q.life for q in g.players if q.alive):
            m.plus += 1
    if len(atk) == 1:
        n = sum(1 for x in p.perms if x.cd is not None and 'exalted' in x.cd.kws and not x.phased)
        if n: _eot(g, atk[0], n, n)
    return new


PROWESS = {'Eris, Roar of the Storm', 'Harmonic Prodigy'}     # hand-tagged cards with prowess (no keyword data)


def prowess(g, p, c):
    """prowess: +1/+1 until end of turn whenever you cast a noncreature spell"""
    for m in p.perms:
        if m.phased or not m.creature: continue
        if (m.cd is not None and ('prowess' in m.cd.kws or m.cd.name in PROWESS)) or (m.data and m.data.get('prowess')):
            _eot(g, m, 1, 1)


def _eot(g, m, dp, dt):
    a, b = g.eot_pt.get(id(m), (0, 0)); g.eot_pt[id(m)] = (a + dp, b + dt)
