"""Combat for the human seat: declaring attackers (and which player they attack) and declaring blockers.

The engine still resolves combat (damage, first strike, trample, triggers); only the declarations are the person's.
Limitations for now: all attackers in one combat attack the same player, and planeswalkers can't be attacked
directly; each attacker is blocked by at most one creature (menace needs two, and only the first deals damage).
"""
from commander_sim import engine as E, ais
from commander_sim.play import legal
from commander_sim.play.controller import controller_of, Request
from commander_sim.play.human import choose


def can_attack(g, p, m):
    """could creature m attack now (by the rules, not by whether it should)"""
    if not m.creature or m.tapped or m.phased or m not in p.perms: return False
    if m.sick and not (p.haste_all or ais.has_haste(g, m)): return False
    if ais.kw(m, 'defender'): return False
    if getattr(g, 'auras', None) and E.CI.locked(g, m, 'pacify'): return False      # Arrest, Luminous Bonds
    return True


def attack_candidates(g, p):
    if E.chasm(p): return []                                  # Glacial Chasm: your creatures can't attack
    return [m for m in p.perms if can_attack(g, p, m)]


def human_attack(g, p, ncomb=1):
    """ask which creatures attack and whom: (defending player, attackers), or None for no attack"""
    cands = attack_candidates(g, p)
    opps = [q for q in g.opps(p) if q.alive]
    if not cands or not opps: return None
    ctl = controller_of(g, p)
    labels = [legal.describe_target(g, p, m) for m in cands]
    extra = '' if ncomb == 1 else f' (combat {ncomb})'
    from commander_sim.play.human import table_refs
    if getattr(ctl, 'notify', None) is not None:
        from commander_sim.play.view import build_view
        data = {'refs': table_refs(g, p, labels), 'view': build_view(g, p.key)}
    else: data = None
    for _ in range(5):
        ans = ctl.ask(Request('attack', f'Declare attackers{extra}: pick any of your creatures, or none', choices=labels,
                              data=data))
        if ans in (None, 'none', []) or ans is False: return None
        if not isinstance(ans, (list, tuple)) or not all(isinstance(i, int) and 0 <= i < len(cands) for i in ans):
            ctl.tell('invalid', 'Pick attackers by their numbers, or none.'); continue
        atk = [cands[i] for i in dict.fromkeys(ans)]
        if len(opps) == 1: d = opps[0]
        else:
            k = choose(g, p, 'target', f'Attack which player with {len(atk)} creature(s)?',
                       [legal.describe_target(g, p, q) for q in opps])
            if k is None: continue                             # changed their mind: declare again
            d = opps[k]
        return d, atk
    return None


def human_blocks(g, p, atk, d, unbl):
    """ask the defending person which creature blocks each attacker; returns {attacker: blocker}"""
    assign, used = {}, set()
    blockers = [m for m in d.perms if m.creature and not m.tapped and not m.phased]
    total = sum(E.epow(g, a) for a in atk if a in p.perms)
    ctl = controller_of(g, d)
    from commander_sim.play.human import table_refs
    show = getattr(ctl, 'notify', None) is not None
    attackers = table_refs(g, d, [legal.describe_target(g, d, a) for a in atk]) if show else []
    for a in sorted(atk, key=lambda m: -E.epow(g, m)):
        if a in unbl or a not in p.perms: continue
        cands = [b for b in blockers if b not in used and ais.can_block(g, b, a)]
        if not cands: continue
        menace = ais.kw(a, 'menace')
        if menace and len(cands) < 2: continue
        what = legal.describe_target(g, d, a) + (' (menace: needs two blockers)' if menace else '')
        extra = {'attackers': attackers, 'attacker': table_refs(g, d, [legal.describe_target(g, d, a)])[0]} if show else None
        k = choose(g, d, 'block', f'{E.NAME(p)} attacks you ({total} damage in all). Block {what} with?',
                   [legal.describe_target(g, d, b) for b in cands], cancel='no block', data=extra)
        if k is None: continue
        b = cands[k]
        if menace:
            rest = [x for x in cands if x is not b]
            k2 = choose(g, d, 'block', f'Second blocker for {a.name} (menace)?',
                        [legal.describe_target(g, d, x) for x in rest], cancel='no block')
            if k2 is None:
                ctl.tell('invalid', f'{a.name} has menace: it needs two blockers, so it is unblocked.'); continue
            used.add(rest[k2])
        assign[a] = b; used.add(b)
        E.log(f'  {E.NAME(d)} blocks {a.name} with {b.name}', g)
    return assign
