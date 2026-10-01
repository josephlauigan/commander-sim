"""Your decks' activated abilities, by the rules. The card code offers these to the AI only when its plan says they are
worth it, with the AI's own targets (Vraska's -2 on the best creature, Mind Stone once there are six lands). For the
human seat they are listed whenever the card has them; timing, tapping, loyalty and mana are checked with the reason,
and you choose the targets. Each entry: card name -> fn(g, p, m) -> [(label, act)], act(g, p, m) -> None or why not.
A card listed here replaces its card-code abilities for the human seat."""
import importlib
from commander_sim import engine as E, ais
from commander_sim.play import legal, mana


def _choose(*a, **k):
    from commander_sim.play.human import choose
    return choose(*a, **k)


def _choices():
    return importlib.import_module('commander_sim.play.choices')


def _mine():
    return importlib.import_module('commander_sim.cards.impl.mine')


def _cost(g, p, gen, pips, what):
    why = mana.cost_problem(g, p, gen, pips)
    return f"Can't activate {what}. {why}" if why else None


def _tapped(m, what=None):
    return f'{what or m.cd.name} is tapped.' if m.tapped else None


# ------------------------------------------------------------------ planeswalkers
def _loyalty(g, p, m, cost):
    """why loyalty ability of m costing `cost` can't be activated now (sorcery speed, once per turn, enough loyalty)"""
    why = legal.sorcery_timing(g, p)
    if why: return why.replace('do that', 'activate a loyalty ability')
    if not _mine()._pw_once(g, m): return f'{m.cd.name} has already used a loyalty ability this turn.'
    if cost < 0 and (m.loyalty or 0) < -cost: return f'{m.cd.name} has only {m.loyalty} loyalty.'
    return None


def _walker(abilities):
    """abilities: [(cost, text, effect(g, p, m) -> None | why)]: the effect runs after the loyalty is paid; one that
    needs a choice asks before paying (a cancelled choice costs nothing) by returning a callable"""
    def offer(g, p, m):
        out = []
        for cost, text, eff in abilities:
            def act(g, p, m, cost=cost, eff=eff, text=text):
                why = _loyalty(g, p, m, cost)
                if why: return why
                ready = eff(g, p, m)
                if isinstance(ready, str): return ready
                if ready is False: return None                     # cancelled
                E.log(f'  {E.NAME(p)} activates {m.cd.name} {cost:+d}: {text}', g)
                _mine()._pw_use(g, m, cost)                        # a walker at 0 loyalty is gone; the effect still happens
                if callable(ready): ready()
                E.check_state(g)
                return None
            out.append((f'{cost:+d}: {text}' if cost else f'0: {text}', act))
        return out
    return offer


def _vraska_minus2(g, p, m):
    cre = [x for q in g.players if q.alive for x in q.perms if x.creature and not x.phased
           and not (x.owner is not p and E.untargetable(g, x))]
    if not cre: return 'There is no creature to target.'
    k = _choose(g, p, 'target', "Vraska -2: which creature becomes a Treasure?", [legal.describe_target(g, p, x) for x in cre])
    if k is None: return False
    t = cre[k]

    def go():
        if t in t.owner.perms:
            q = t.owner; E.leave(g, t); q.treasures += 1
            E.log(f'    {t.name} becomes a Treasure', g)
    return go


def _any_player(g, p, prompt):
    ps = [q for q in g.players if q.alive]
    k = _choose(g, p, 'target', prompt, [E.NAME(q) + (' (you)' if q is p else '') for q in ps])
    return None if k is None else ps[k]


def _vraska_minus9(g, p, m):
    q = _any_player(g, p, 'Vraska -9: which player gets nine poison counters?')
    if q is None: return False

    def go():
        if not _mine().melira(q): q.poison = max(getattr(q, 'poison', 0), 9)
    return go


def _vraska_zero(g, p, m):
    def go():
        E.draw(g, p, 1); E.lose_life(g, p, 1, p); _mine().proliferate_all(g, p)
    return go


def _ralz_plus1(g, p, m):
    return lambda: _choices().scry(g, p, 2, to='gy')


def _ralz_minus1(g, p, m):
    def go():
        for q in g.opps(p):
            if q.hand: E.discard_worst(g, q, 1)
    return go


def _ralz_minus2(g, p, m):
    cs = [c for c in p.gy if c.creature and c.cmc <= 3]
    if not cs: return 'There is no creature card with mana value 3 or less in your graveyard.'
    k = _choose(g, p, 'target', 'Ral Zarek -2: return which creature card?', [_choices().card_label(c) for c in cs])
    if k is None: return False
    c = cs[k]

    def go():
        if c in p.gy: p.gy.remove(c); E.enter(g, p, c)
    return go


def _ralz_minus7(g, p, m):
    opps = g.opps(p)
    if not opps: return 'There is no opponent to target.'
    k = _choose(g, p, 'target', 'Ral Zarek -7: which opponent?', [E.NAME(q) for q in opps])
    if k is None: return False
    q = opps[k]

    def go():
        n = sum(1 for _ in range(5) if g.rng.random() < 0.5)
        q.skip_turns = getattr(q, 'skip_turns', 0) + n
        E.log(f'    five coins: {n} heads, {E.NAME(q)} skips {n} turn(s)', g)
    return go


def _ral_plus2(g, p, m):
    return lambda: _choices().scry(g, p, 1)


def _ral_minus2(g, p, m):
    def go():
        p.ral_copy = E.turn_stamp(g)
    return go


# ------------------------------------------------------------------ prepared back faces (cast a copy of the spell)
PREPARED = {'Blazing Firesinger // Seething Song': (2, 'R', True, 'Seething Song', 'add {R}{R}{R}{R}{R}'),
            'Emeritus of Ideation // Ancestral Recall': (0, 'U', True, 'Ancestral Recall', 'target player draws three'),
            'Sanar, Unfinished Genius // Wild Idea': (3, 'UR', False, 'Wild Idea', 'search for an instant or sorcery'),
            'Emeritus of Conflict // Lightning Bolt': (0, 'R', True, 'Lightning Bolt', '3 damage to any target'),
            'Grave Researcher // Reanimate': (0, 'B', False, 'Reanimate',
                                              'a creature card from a graveyard to the battlefield; lose life equal to its mana value')}


def _prepared(g, p, m):
    gen, pips, instant, spell, text = PREPARED[m.cd.name]

    def act(g, p, m):
        if not (m.data and m.data.get('prepared')): return f'{m.cd.name.split(" //")[0]} is not prepared.'
        if not instant and not E.has(p, 'gandalf'):
            why = legal.sorcery_timing(g, p)
            if why: return why.replace('do that', f'cast {spell} (a sorcery)')
        why = mana.cost_problem(g, p, gen, pips)
        if why: return f"Can't cast {spell}. {why}"
        eff = None
        if spell == 'Ancestral Recall':
            q = _any_player(g, p, 'Ancestral Recall: which player draws three?')
            if q is None: return None
            eff = lambda: E.draw(g, q, 3)
        elif spell == 'Lightning Bolt':
            tg = _choices().damage_targets(g, p, 'R')
            if not tg: return 'There is no target for Lightning Bolt.'
            k = _choose(g, p, 'target', 'Lightning Bolt: 3 damage to which target?', [legal.describe_target(g, p, x) for x in tg])
            if k is None: return None
            eff = lambda x=tg[k]: _damage(g, p, x, 3, 'Lightning Bolt', 'burn')
        elif spell == 'Reanimate':
            pick = _rean_pick(g, p, 'Reanimate')
            if isinstance(pick, str) or pick is None: return pick
            cd, q = pick

            def eff(cd=cd, q=q):
                if cd not in q.gy: return
                if g.hooks and E.CI.gy_response(g, p, max(4, cd.cmc), q): return
                q.gy.remove(cd); E.enter(g, p, cd, orig=q); E.lose_life(g, p, cd.cmc, p)
        elif spell == 'Seething Song':
            eff = lambda: mana.pool_of(p).add('R', 5)
        else:
            eff = lambda: E.tutor(g, p, 'is')
        mana.pay_from_pool(g, p, gen, pips)
        m.data['prepared'] = False
        E.log(f'  {E.NAME(p)} casts a copy of {spell} ({m.cd.name.split(" //")[0]})', g)
        E.cast_copy(g, p, eff)
        return None
    if not (m.data and m.data.get('prepared')): return []
    return [(f'prepared: cast a copy of {spell} ({mana.cost_text(gen, pips)}): {text}', act)]


def _damage(g, p, x, n, source, kind):
    """n damage from source to x, a player or permanent chosen already (the same rules as choices.deal_damage)"""
    if isinstance(x, E.Player):
        if x.alive: E.lose_life(g, x, n, p, kind=kind)
        E.check_state(g); return
    if x not in x.owner.perms: return
    if x.creature and E.etgh(g, x) <= n: E.apply_removal(g, p, x, f'dmg{n}')
    elif x.creature: E.log(f'    {source} deals {n} damage to {x.name} (it survives)', g)
    if x in x.owner.perms and x.cd is not None and 'P' in x.cd.types and x.loyalty is not None:
        x.loyalty -= n
        if x.loyalty <= 0: E.leave(g, x); E.to_zone_card(g, x, 'gy')
    E.check_state(g)


def _rean_pick(g, p, what, kind='reanimate'):
    cands = _choices().rean_candidates(g, p, kind)
    if not cands: return 'There is no creature card in a graveyard to return.'
    k = _choose(g, p, 'target', f'{what}: return which creature card?',
                [f'{_choices().card_label(c)} ({"your" if q is p else E.NAME(q) + chr(39) + "s"} graveyard)' for c, q in cands])
    return None if k is None else cands[k]


# ------------------------------------------------------------------ other permanents
def _mind_stone(g, p, m):
    def act(g, p, m):
        why = _tapped(m) or _cost(g, p, 1, '', 'Mind Stone')
        if why: return why
        mana.pay_from_pool(g, p, 1, ''); m.tapped = True
        E.log(f'  {E.NAME(p)} sacrifices Mind Stone: draw a card', g)
        E.die(g, m, 'sac'); E.draw(g, p, 1)
        return None
    return [('{1}, {T}, sacrifice: draw a card', act)]


def _triskelion(g, p, m):
    def act(g, p, m):
        if m.plus <= 0: return 'Triskelion has no +1/+1 counter to remove.'
        tg = _choices().damage_targets(g, p)
        k = _choose(g, p, 'target', 'Triskelion: 1 damage to which target?', [legal.describe_target(g, p, x) for x in tg])
        if k is None: return None
        m.plus -= 1
        E.log(f'  {E.NAME(p)} removes a counter from Triskelion', g)
        _damage(g, p, tg[k], 1, 'Triskelion', 'triggers')
        if m in p.perms and E.etgh(g, m) <= 0: E.die(g, m, 'sba')
        return None
    return [('remove a +1/+1 counter: 1 damage to any target', act)]


def _aetherflux(g, p, m):
    def act(g, p, m):
        if p.life < 50: return f'You need 50 life to pay (you have {p.life}).'
        tg = _choices().damage_targets(g, p)
        k = _choose(g, p, 'target', 'Aetherflux Reservoir: 50 damage to which target?', [legal.describe_target(g, p, x) for x in tg])
        if k is None: return None
        E.lose_life(g, p, 50, p)
        E.log(f'  {E.NAME(p)} pays 50 life: Aetherflux Reservoir', g)
        if ais.tide_response(g, p, 'aether', 9): return None
        p.stats['aether_shots'] += 1; p.milestone.setdefault('aether', p.turns)
        _damage(g, p, tg[k], 50, 'Aetherflux Reservoir', 'aether')
        return None
    return [('pay 50 life: 50 damage to any target', act)]


def _pteramander(g, p, m):
    n = max(0, 7 - sum(1 for c in p.gy if c.instant or c.sorcery))

    def act(g, p, m):
        why = _cost(g, p, n, 'U', 'Pteramander')
        if why: return why
        mana.pay_from_pool(g, p, n, 'U')
        if m.plus > 0: E.log(f'  {E.NAME(p)} adapts Pteramander (it has counters already: nothing happens)', g)
        else: m.plus += 4; E.log(f'  {E.NAME(p)} adapts Pteramander (4 counters)', g)
        return None
    return [(f'{mana.cost_text(n, "U")}: adapt 4 ({{1}} less per instant and sorcery in your graveyard)', act)]


def _skullport(g, p, m):
    def act(g, p, m):
        cre = [x for x in p.perms if x.creature and not x.phased and x is not m]
        if not cre and not p.treasures: return 'You have no other creature or Treasure to sacrifice.'
        why = _cost(g, p, 1, 'B', 'Skullport Merchant')
        if why: return why
        labels = (['a Treasure'] if p.treasures else []) + [legal.describe_target(g, p, x) for x in cre]
        k = _choose(g, p, 'choose', 'Skullport Merchant: sacrifice what?', labels)
        if k is None: return None
        mana.pay_from_pool(g, p, 1, 'B')
        if p.treasures and k == 0:
            p.treasures -= 1
            if g.hooks: E.CI.fire(g, 'sacrifice', p, 'Treasure')
        else:
            E.die(g, cre[k - (1 if p.treasures else 0)], 'sac')
        E.draw(g, p, 1)
        return None
    return [('{1}{B}, sacrifice another creature or a Treasure: draw a card', act)]


def _ste(g, p, m):
    def act(g, p, m):
        E.log(f'  {E.NAME(p)} sacrifices Sakura-Tribe Elder', g)
        E.die(g, m, 'sac'); E.land_ramp(g, p, 1, True)
        return None
    return [('sacrifice: a basic land onto the battlefield tapped', act)]


ABILITIES = {
    "Vraska, Betrayal's Sting": _walker([(0, 'draw a card, lose 1 life, proliferate', _vraska_zero),
                                         (-2, 'target creature becomes a Treasure', _vraska_minus2),
                                         (-9, "a player's poison counters become nine", _vraska_minus9)]),
    'Ral Zarek, Guest Lecturer': _walker([(1, 'surveil 2', _ralz_plus1),
                                          (-1, 'each opponent discards a card', _ralz_minus1),
                                          (-2, 'a creature card (mana value 3 or less) from your graveyard to the battlefield', _ralz_minus2),
                                          (-7, 'flip five coins: target opponent skips a turn per head', _ralz_minus7)]),
    'Ral, Storm Conduit': _walker([(2, 'scry 1', _ral_plus2),
                                   (-2, 'copy the next instant or sorcery you cast this turn', _ral_minus2)]),
    'Mind Stone': _mind_stone,
    'Triskelion': _triskelion,
    'Aetherflux Reservoir': _aetherflux,
    'Pteramander': _pteramander,
    'Skullport Merchant': _skullport,
    'Sakura-Tribe Elder': _ste,
    'Nim Deathmantle': lambda g, p, m: [],                  # equip {4}: the Equip ability every equipment has
}
for _n in PREPARED: ABILITIES[_n] = _prepared


def abilities(g, p, m):
    """[(label, act)] for your permanent m, or None when its card isn't listed here (its card-code abilities apply)"""
    if m.cd is None or m.cd.name not in ABILITIES: return None
    return ABILITIES[m.cd.name](g, p, m)


# ------------------------------------------------------------------ lands
def _strip(g, p, L):
    def act(g, p, L):
        why = _tapped(L, 'Strip Mine')
        if why: return why
        tg = [(q, x) for q in g.players if q.alive for x in q.lands]
        k = _choose(g, p, 'target', 'Strip Mine: destroy which land?',
                    [f'{x.cd.name} ({"yours" if q is p else E.NAME(q)})' for q, x in tg])
        if k is None: return None
        q, x = tg[k]
        p.lands.remove(L); p.gy.append(L.cd)
        E.log(f'  {E.NAME(p)} sacrifices Strip Mine: destroys {x.cd.name} ({E.NAME(q)})', g)
        if x in q.lands: importlib.import_module('commander_sim.cards.impl.fixes').destroy_land(g, q, x)
        return None
    return [('{T}, sacrifice: destroy target land', act)]


def _lighthouse(g, p, L):
    def act(g, p, L):
        why = _tapped(L, 'Desolate Lighthouse') or _cost(g, p, 1, 'UR', 'Desolate Lighthouse')
        if why: return why
        mana.pay_from_pool(g, p, 1, 'UR'); L.tapped = True
        E.log(f'  {E.NAME(p)} loots with Desolate Lighthouse', g)
        E.draw(g, p, 1); E.discard_worst(g, p, 1)
        return None
    return [('{1}{U}{R}, {T}: draw a card, then discard a card', act)]


def _summit(g, p, L):
    def act(g, p, L):
        why = _tapped(L, 'Spectacle Summit') or _cost(g, p, 2, 'UR', 'Spectacle Summit')
        if why: return why
        mana.pay_from_pool(g, p, 2, 'UR'); L.tapped = True
        _choices().scry(g, p, 1, to='gy')
        return None
    return [('{2}{U}{R}, {T}: surveil 1', act)]


def _baraddur(g, p, L):
    def act(g, p, L):
        why = _tapped(L, 'Barad-dûr')
        if why: return why
        if getattr(g, 'died_turn', None) != E.turn_stamp(g): return 'Barad-dûr: activate only if a creature died this turn.'
        top = max(0, (mana.pool_of(p).total() - 1) // 2)
        if mana.cost_problem(g, p, 0, 'B'): return f"Can't activate Barad-dûr. {mana.cost_problem(g, p, 0, 'B')}"
        k = _choose(g, p, 'choose', 'Barad-dûr: choose X (it costs {X}{X}{B}: amass Orcs X)',
                    [f'X = {x} ({mana.cost_text(2 * x, "B")})' for x in range(1, top + 1)] or ['X = 0 ({B})'])
        if k is None: return None
        x = k + 1 if top else 0
        why = mana.pay_from_pool(g, p, 2 * x, 'B')
        if why: return f"Can't activate Barad-dûr. {why}"
        L.tapped = True
        E.amass(g, p, x); E.log(f'  {E.NAME(p)} uses Barad-dûr: amass Orcs {x}', g)
        return None
    return [('{X}{X}{B}, {T}: amass Orcs X (only if a creature died this turn)', act)]


LANDS = {'Strip Mine': _strip, 'Desolate Lighthouse': _lighthouse, 'Spectacle Summit': _summit, 'Barad-dûr': _baraddur}


def land_abilities(g, p, L):
    return LANDS[L.cd.name](g, p, L) if L.cd.name in LANDS else []
