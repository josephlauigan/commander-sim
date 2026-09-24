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
A hook on a card that is also in one of the four main decks only runs in pool games (engine.POOL_RULES),
so the original four-deck mode is unchanged.
"""
import engine as E

HOOKS = {}            # card name -> {event: fn}
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
    return name in HOOKS and (E.POOL_RULES or name not in main_cards())


def hooked(g, event):
    """(src permanent, fn) for every permanent on the battlefield with a hook for event"""
    hs = getattr(g, 'hooks', None)
    if not hs: return
    for m in list(hs):
        if m.phased or m.cd is None or m not in m.owner.perms or not m.owner.alive: continue
        fn = HOOKS.get(m.cd.name, {}).get(event)
        if fn is not None and not m.neutered: yield m, fn


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


def load():
    """import the implementation modules (they register themselves)"""
    import impl_common, impl_t1  # noqa: F401


E.CI = __import__('sys').modules[__name__]


# ------------------------------------------------------------------ attack keywords (compiled cards)
def keyword_attack(g, p, atk, d):
    """battle cry, mentor, dethrone, exalted, myriad-free subset; returns new attacking creatures"""
    new = []
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


def _eot(g, m, dp, dt):
    a, b = g.eot_pt.get(id(m), (0, 0)); g.eot_pt[id(m)] = (a + dp, b + dt)
