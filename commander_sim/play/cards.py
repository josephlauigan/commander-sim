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


def _outlet(text, effect):
    """a free sacrifice outlet: "Sacrifice a creature: <text>". effect(g, p, m, power) after the sacrifice"""
    def offer(g, p, m):
        def act(g, p, m):
            cre = [x for x in p.perms if x.creature and not x.phased]
            if not cre: return 'You have no creature to sacrifice.'
            k = _choose(g, p, 'choose', f'{m.cd.name}: sacrifice which creature?', [legal.describe_target(g, p, x) for x in cre])
            if k is None: return None
            x = cre[k]; pw = E.epow(g, x)
            E.log(f'  {E.NAME(p)} sacrifices {x.name} to {m.cd.name}', g)
            E.die(g, x, 'sac')
            effect(g, p, m, pw)
            E.check_state(g)
            return None
        return [(f'sacrifice a creature: {text}', act)]
    return offer


def _feeder(g, p, m, pw):
    if m in p.perms: m.plus += 1


def _dementia(g, p, m, pw):
    q = _any_player(g, p, f'Altar of Dementia: which player mills {pw}?') if pw > 0 else None
    if q is not None: E.mill(g, q, pw); E.log(f'    Altar of Dementia: {E.NAME(q)} mills {pw}', g)


def _coalition(g, p, m):
    def act(g, p, m):
        why = _tapped(m)
        if why: return why
        m.tapped = True; m.data = dict(m.data or {}, charge=(m.data or {}).get('charge', 0) + 1)
        E.log(f'  {E.NAME(p)} puts a charge counter on Coalition Relic', g)
        return None
    return [('{T}: put a charge counter on it (each one is a mana of any colour at your next precombat main phase)', act)]


ABILITIES = {
    'Carrion Feeder': _outlet('put a +1/+1 counter on Carrion Feeder', _feeder),
    'Viscera Seer': _outlet('scry 1', lambda g, p, m, pw: _choices().scry(g, p, 1)),
    "Ashnod's Altar": _outlet('add {C}{C}', lambda g, p, m, pw: mana.pool_of(p).add('C', 2)),
    'Altar of Dementia': _outlet("target player mills cards equal to the sacrificed creature's power", _dementia),
    'Coalition Relic': _coalition,
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


# ------------------------------------------------------------------ casting from hand: modes the card code keeps for the AI
def _spell_cast(g, p, c, imp, what=None):
    """the steps of casting c from hand once it is paid for: on the stack, cast triggers, a window for opponents.
    True if it resolves (the caller carries out its effect and puts the card away)"""
    p.hand.remove(c)
    p.spells_this_turn += 1; p.stats['spells_cast'] += 1; p.cast_names.add(c.name)
    E.log(f'  {E.NAME(p)} casts {what or c.name}', g)
    E.on_cast(g, p, c)
    if g.over or not p.alive: return False
    return E.counter_window(g, p, c, imp, {})


def _cast_normally(g, p, c):
    from commander_sim.play import human
    why = legal.check_cast(g, p, c)
    return why or human.cast(g, p, c, 'hand')


def _cycle(n):
    def act(g, p, c):
        why = mana.cost_problem(g, p, n, '')
        if why: return f"Can't cycle {c.name}. {why}"
        mana.pay_from_pool(g, p, n, '')
        p.hand.remove(c); p.gy.append(c)
        E.log(f'  {E.NAME(p)} cycles {c.name}', g)
        E.draw(g, p, 1)
        return None
    return act


def _twinflame(g, p, c):
    why = legal.check_cast(g, p, c)
    if why: return why
    cre = [m for m in p.perms if m.creature and not m.phased]
    if not cre: return 'Twinflame needs a creature you control to target.'
    picked = []
    while True:
        left = [m for m in cre if m not in picked]
        if not left: break
        n = len(picked)
        k = _choose(g, p, 'target', f'Twinflame: target creature #{n + 1} ({mana.cost_text(1 + 2 * n, "R" * (n + 1))} in all)',
                    [legal.describe_target(g, p, m) for m in left], cancel='cancel' if not picked else 'done')
        if k is None: break
        picked.append(left[k])
    if not picked: return None
    extra = len(picked) - 1
    why = mana.pay_from_pool(g, p, 1 + 2 * extra, 'R' * (extra + 1))
    if why: return f"Can't cast Twinflame on {len(picked)} creatures. {why}"
    if not _spell_cast(g, p, c, 3, f"Twinflame copying {', '.join(m.name for m in picked)}"):
        p.gy.append(c); return None
    for m in picked:
        if m not in p.perms: continue
        if m.cd is not None: cp = E.enter_token_copy(g, p, m.cd)
        else:                                                         # a token with no card: the same creature token
            cp = (E.make_tokens(g, p, 1, m.pow, m.tgh, fly=bool(getattr(m, 'fly', False)), sick=False) or [None])[0]
        if cp is not None: cp.sick = False; cp.temp = True               # haste; exiled at the end step
    p.gy.append(c); E.check_state(g)
    return None


def _ephemerate(g, p, c):
    why = legal.check_cast(g, p, c)
    if why: return why
    cre = [m for m in p.perms if m.creature and not m.phased]
    if not cre: return 'Ephemerate needs a creature you control to target.'
    k = _choose(g, p, 'target', 'Ephemerate: exile and return which creature?', [legal.describe_target(g, p, m) for m in cre])
    if k is None: return None
    m = cre[k]
    mana.pay_from_pool(g, p, 0, 'W')
    if not _spell_cast(g, p, c, 3, f'Ephemerate on {m.name}'):
        p.gy.append(c); return None
    p.exile.append(c); p.rebound = getattr(p, 'rebound', []) + [c]
    if m in p.perms: _blink(g, p, m)
    return None


def _blink(g, p, m):
    if m.token: E.leave(g, m); E.log(f'    {m.name} (a token) is exiled for good', g); return
    importlib.import_module('commander_sim.cards.impl.t2').blink(g, p, m)


def ephemerate_rebound(g, p, c):
    """your upkeep: cast Ephemerate from exile for free, on a creature you choose; not cast, it stays in exile"""
    cre = [m for m in p.perms if m.creature and not m.phased]
    if not cre: return
    k = _choose(g, p, 'target', 'Rebound: cast Ephemerate again? Exile and return which creature',
                [legal.describe_target(g, p, m) for m in cre], cancel="don't cast it")
    if k is None: return
    m = cre[k]
    p.exile.remove(c)
    p.spells_this_turn += 1; p.stats['spells_cast'] += 1; p.cast_names.add(c.name)
    E.on_cast(g, p, c)
    E.log(f'  {E.NAME(p)} casts Ephemerate from exile (rebound) on {m.name}', g)
    if not g.over and E.counter_window(g, p, c, 3, {}) and m in p.perms: _blink(g, p, m)
    p.gy.append(c)


def _sokenzan(g, p, c):
    leg = sum(1 for m in p.perms if m.creature and m.cd is not None and 'leg' in m.cd.tags and not m.phased)
    n = max(0, 3 - leg)
    why = mana.cost_problem(g, p, n, 'R')
    if why: return f"Can't channel Sokenzan. {why}"
    mana.pay_from_pool(g, p, n, 'R')
    p.hand.remove(c); p.gy.append(c)
    E.log(f'  {E.NAME(p)} channels Sokenzan: two 1/1 Spirits with haste', g)
    E.make_tokens(g, p, 2, 1, sick=False, types=('spirit',))
    return None


def _insight(g, p, c):
    why = legal.sorcery_timing(g, p)
    if why: return why.replace('do that', 'cast Bloodsoaked Insight (a sorcery)')
    gen = _mine().insight_cost(g, p)
    ok = next((pp for pp in ('BB', 'BR', 'RR') if not mana.cost_problem(g, p, gen, pp)), None)
    if ok is None:
        return (f"Can't cast Bloodsoaked Insight. It costs {{{gen}}}{{B/R}}{{B/R}} ({{1}} less for each life your "
                f"opponents lost this turn). {mana.cost_problem(g, p, gen, 'BB')}")
    opps = g.opps(p)
    if not opps: return 'There is no opponent to target.'
    k = _choose(g, p, 'target', "Bloodsoaked Insight: whose library?", [E.NAME(q) for q in opps])
    if k is None: return None
    q = opps[k]
    mana.pay_from_pool(g, p, gen, ok)
    if not _spell_cast(g, p, c, 2, f'Bloodsoaked Insight on {E.NAME(q)}'):
        p.gy.append(c); return None
    top = [q.library.pop() for _ in range(min(3, len(q.library)))]
    p.hand.extend(top)
    p.impulse_long = (getattr(p, 'impulse_long', None) or []) + [(x, p.turns + 1) for x in top]
    E.log(f'    {", ".join(x.name for x in top)} from {E.NAME(q)}: playable until the end of your next turn', g)
    p.gy.append(c)
    return None


def _disintegrate(g, p, c):
    why = legal.check_cast(g, p, c)
    if why: return why
    tg = _choices().damage_targets(g, p, 'R')
    k = _choose(g, p, 'target', 'Disintegrate: X damage to which target?', [legal.describe_target(g, p, x) for x in tg])
    if k is None: return None
    x = tg[k]
    top = mana.pool_of(p).total() - 1
    j = _choose(g, p, 'choose', 'Disintegrate: choose X (paid from your mana pool)', [f'X = {n}' for n in range(top + 1)])
    if j is None: return None
    why = mana.pay_from_pool(g, p, j, 'R')
    if why: return f"Can't cast Disintegrate with X = {j}. {why}"
    ctx = {'rem_kind': f'dmg{j}'}
    ctx['face' if isinstance(x, E.Player) else 'target'] = x
    p.stats['removal_cast'] += 1
    E.cast_card(g, p, c, 'hand', ctx)
    return None


def _disembowel(g, p, c):
    why = legal.check_cast(g, p, c)
    if why: return why
    tg = [m for q in g.players if q.alive for m in q.perms if m.creature and not m.phased
          and not (m.owner is not p and E.untargetable(g, m))]
    if not tg: return 'There is no creature to target.'
    mv = lambda m: m.cd.cmc if m.cd is not None and not m.token else 0
    k = _choose(g, p, 'target', 'Disembowel: destroy which creature? (X is its mana value)',
                [f'{legal.describe_target(g, p, m)} (X = {mv(m)})' for m in tg])
    if k is None: return None
    t = tg[k]
    why = mana.pay_from_pool(g, p, mv(t), 'B')
    if why: return f"Can't cast Disembowel with X = {mv(t)}. {why}"
    p.stats['removal_cast'] += 1
    E.cast_card(g, p, c, 'hand', {'target': t})
    return None


def _throwdown(g, p, c):
    why = legal.check_cast(g, p, c)
    if why: return why
    fod = [m for m in p.perms if m.creature and not m.phased]
    if not fod: return 'Lethal Throwdown needs a creature to sacrifice as you cast it.'
    tg = [m for q in g.players if q.alive for m in q.perms if not m.phased and (m.creature or (m.cd is not None and 'P' in m.cd.types))
          and not (m.owner is not p and E.untargetable(g, m))]
    k = _choose(g, p, 'target', 'Lethal Throwdown: destroy which creature or planeswalker?', [legal.describe_target(g, p, m) for m in tg])
    if k is None: return None
    j = _choose(g, p, 'choose', 'Lethal Throwdown: sacrifice which creature? (a modified one draws you a card)',
                [legal.describe_target(g, p, m) for m in fod])
    if j is None: return None
    f = fod[j]
    mana.pay_from_pool(g, p, 0, 'B')
    mod = importlib.import_module('commander_sim.cards.impl.marchesa')._modified(g, f)
    E.log(f'  {E.NAME(p)} sacrifices {f.name} for Lethal Throwdown', g)
    E.die(g, f, 'sac'); p.stats['removal_cast'] += 1
    E.cast_card(g, p, c, 'hand', {'target': tg[k], 'modified': mod})
    return None


HAND = {
    'Twinflame': lambda g, p, c: [('strive: token copies of creatures you control ({1}{R}, plus {2}{R} per extra target)', _twinflame)],
    'Ephemerate': lambda g, p, c: [('exile a creature you control, then return it (rebound)', _ephemerate)],
    'Sokenzan, Crucible of Defiance': lambda g, p, c: [('channel ({3}{R}, {1} less per legendary creature): two 1/1 Spirits with haste', _sokenzan)],
    'Bloodsoaked Insight // Sanguine Morass': lambda g, p, c: [("Bloodsoaked Insight: an opponent's top three, playable until the end of your next turn", _insight)],
    'Reconnaissance Mission': lambda g, p, c: [('cast it', _cast_normally), ('cycling {2}: discard it, draw a card', _cycle(2))],
    'Unearth': lambda g, p, c: [('cast it', _cast_normally), ('cycling {2}: discard it, draw a card', _cycle(2))],
    'Disintegrate': lambda g, p, c: [('X damage to any target', _disintegrate)],
    'Disembowel': lambda g, p, c: [('destroy target creature with mana value X', _disembowel)],
    'Lethal Throwdown': lambda g, p, c: [('sacrifice a creature: destroy target creature or planeswalker', _throwdown)],
}


def _you_control_a_creature(g, p, c):
    if not any(m.creature and not m.phased for m in p.perms): return f'{c.name} needs a creature you control to target.'


def _any_creature(g, p, c):
    if not any(m.creature and not m.phased and not (m.owner is not p and E.untargetable(g, m))
               for q in g.players if q.alive for m in q.perms):
        return f'{c.name} has no legal target right now.'


NEEDS = {'Twinflame': _you_control_a_creature, 'Ephemerate': _you_control_a_creature,
         'Disembowel': _any_creature,
         'Lethal Throwdown': lambda g, p, c: (_you_control_a_creature(g, p, c) and
                                              f'{c.name} needs a creature to sacrifice as you cast it.')}


def needs(g, p, c):
    """what c needs before it can be cast at all (the rules check asks this), or None"""
    return NEEDS[c.name](g, p, c) if c.name in NEEDS else None


def cast_from_hand(g, p, c):
    """a card of yours whose casting the card code keeps for the AI: pick how (when there's a choice), then cast it"""
    ways = HAND[c.name](g, p, c)
    k = 0
    if len(ways) > 1:
        k = _choose(g, p, 'choose', f'{c.name}: which?', [w[0] for w in ways])
        if k is None: return None
    return ways[k][1](g, p, c)


# ------------------------------------------------------------------ copying a spell: Return the Favor, Dualcaster Mage
def copy_card(c):
    return c.name in ('Return the Favor', 'Dualcaster Mage')


def copy_in_response(g, p, c, spell, caster):
    """p casts Return the Favor or Dualcaster Mage at `spell` (cast by `caster`, on the stack now). None, or why not"""
    cc = getattr(g, 'cur_cast', None)
    ctx = cc[1] if cc is not None and cc[0] is spell else None
    if c.name == 'Dualcaster Mage':
        if not (spell.instant or spell.sorcery): return f'Dualcaster Mage copies an instant or sorcery spell; {spell.name} is not one.'
        why = mana.cost_problem(g, p, 1, 'RR')
        if why: return f"Can't cast Dualcaster Mage. {why}"
        mana.pay_from_pool(g, p, 1, 'RR')
        E.log(f'  {E.NAME(p)} flashes in Dualcaster Mage to copy {spell.name}', g)
        if E.cast_card(g, p, c, 'hand', {}) and not g.over: E.copy_spell(g, p, spell, dict(ctx) if ctx else None)
        return None
    modes = []
    if spell.instant or spell.sorcery: modes.append(('copy', f'copy {spell.name} (you may choose new targets)'))
    if ctx is not None and (ctx.get('target') is not None or ctx.get('face') is not None):
        modes.append(('redirect', f'change the target of {spell.name}'))
    if not modes: return f'Return the Favor has nothing to do to {spell.name} (not an instant or sorcery, and no single target).'
    labels = [f'{t} (+{{1}})' for _, t in modes] + (['both (+{2})'] if len(modes) == 2 else [])
    k = _choose(g, p, 'choose', 'Return the Favor (spree): which modes?', labels)
    if k is None: return None
    chosen = [modes[k][0]] if k < len(modes) else [m for m, _ in modes]
    why = mana.cost_problem(g, p, len(chosen), 'RR')
    if why: return f"Can't cast Return the Favor. {why}"
    new = None
    if 'redirect' in chosen:
        tg = [x for x in (legal.spell_targets(g, caster, spell) or [])
              if x is not ctx.get('target') and x is not ctx.get('face')]
        if not tg: return f'{spell.name} has no other legal target.'
        j = _choose(g, p, 'target', f'Return the Favor: the new target for {spell.name}', [legal.describe_target(g, p, x) for x in tg])
        if j is None: return None
        new = tg[j]
    mana.pay_from_pool(g, p, len(chosen), 'RR')
    if not _spell_cast(g, p, c, 5, f'Return the Favor ({" and ".join(chosen)}) at {spell.name}'):
        p.gy.append(c); return None
    if new is not None and ctx is not None:
        ctx.pop('target', None); ctx.pop('face', None)
        ctx['face' if isinstance(new, E.Player) else 'target'] = new
        E.log(f'    {spell.name} now targets {legal.describe_target(g, p, new)}', g)
    if 'copy' in chosen: E.copy_spell(g, p, spell, dict(ctx) if ctx else None)
    p.gy.append(c)
    return None



# ------------------------------------------------------------------ choices as your spells resolve (Veyran's deck)
def _look(p, n):
    return [p.library.pop() for _ in range(min(n, len(p.library)))]


def _bottom(p, cards):
    p.library[:0] = cards                                   # the bottom of the library is index 0


def expressive_iteration(g, p, c, ctx):
    """top three: one into your hand, one exiled (you may play it this turn), the last on the bottom"""
    top = _look(p, 3)
    if not top: return
    ch = _choices()
    keep = ch.pick_cards(g, p, top, 1, 'Expressive Iteration: put which card into your hand?')[0]
    top.remove(keep); p.hand.append(keep)
    if top:
        ex = ch.pick_cards(g, p, top, 1, 'Expressive Iteration: exile which card (you may play it this turn)?')[0]
        top.remove(ex); p.hand.append(ex); p.impulse.append(ex)
    _bottom(p, top)
    E.log(f'    {E.NAME(p)} keeps one card, exiles one to play this turn, puts {len(top)} on the bottom', g)


def look_and_take(g, p, n, k, name):
    """look at the top n, put k into your hand, the rest on the bottom (Flow State, Stock Up)"""
    top = _look(p, n)
    picked = _choices().pick_cards(g, p, top, k, f'{name}: put which card into your hand?')
    for x in picked: top.remove(x); p.hand.append(x)
    _bottom(p, top)
    E.log(f'    {E.NAME(p)} takes {len(picked)}, puts {len(top)} on the bottom', g)


def prismari_charm(g, p, c, ctx):
    """choose one: surveil 2, then draw a card; 1 damage to each of one or two targets; return target nonland
    permanent to its owner's hand"""
    modes = ['surveil 2, then draw a card', '1 damage to each of one or two targets', "return target nonland permanent to its owner's hand"]
    k = _choose(g, p, 'choose', 'Prismari Charm: choose one', modes, cancel=None)
    if k == 0:
        _choices().scry(g, p, 2, to='gy'); E.draw(g, p, 1); return
    if k == 1:
        tg = _choices().damage_targets(g, p, 'UR')
        if not tg: return
        a = _choose(g, p, 'target', 'Prismari Charm: 1 damage to which target?', [legal.describe_target(g, p, x) for x in tg], cancel=None)
        first = tg.pop(a)
        b = _choose(g, p, 'target', 'Prismari Charm: a second target?', [legal.describe_target(g, p, x) for x in tg],
                    cancel='no second target') if tg else None
        for x in [first] + ([tg[b]] if b is not None else []): _damage(g, p, x, 1, 'Prismari Charm', 'burn')
        return
    tg = [m for q in g.players if q.alive for m in q.perms if not m.phased and not (m.owner is not p and E.untargetable(g, m))]
    if not tg: return
    j = _choose(g, p, 'target', 'Prismari Charm: return which permanent?', [legal.describe_target(g, p, x) for x in tg], cancel=None)
    E.apply_removal(g, p, tg[j], 'bounce', c)


def jeskas_will(g, p):
    """choose one (both if you control your commander): add {R} for each card in target opponent's hand; exile the
    top three cards of your library, you may play them this turn"""
    opts = ["add {R} for each card in target opponent's hand", 'exile your top three cards: you may play them this turn']
    if E.commander_out(p): opts.append('both')
    k = _choose(g, p, 'choose', "Jeska's Will: choose one" + (' (or both: you control your commander)' if len(opts) == 3 else ''),
                opts, cancel=None)
    if k in (0, 2):
        opps = g.opps(p)
        if opps:
            j = _choose(g, p, 'target', "Jeska's Will: which opponent's hand?", [f'{E.NAME(q)} ({len(q.hand)} cards)' for q in opps],
                        cancel=None)
            n = len(opps[j].hand)
            mana.pool_of(p).add('R', n); E.log(f"    Jeska's Will adds {n} red mana", g)
    if k in (1, 2):
        top = _look(p, 3)
        for x in top: p.hand.append(x); p.seen_names.add(x.name)
        p.impulse += top
        E.log(f"    Jeska's Will exiles {', '.join(x.name for x in top)} (playable this turn)", g)


def crackle(g, p, c, ctx):
    """5X damage to each of up to X targets"""
    x = ctx.get('x', 0)
    if x <= 0: return
    tg = _choices().damage_targets(g, p, 'R')
    picked = []
    while len(picked) < x and tg:
        k = _choose(g, p, 'target', f'Crackle with Power: {5 * x} damage to which target? ({len(picked) + 1} of up to {x})',
                    [legal.describe_target(g, p, t) for t in tg], cancel='no more targets' if picked else None)
        if k is None: break
        picked.append(tg.pop(k))
    for t in picked: _damage(g, p, t, 5 * x, 'Crackle with Power', 'burn')


def mastery_target(g, p, c):
    """Mizzix's Mastery's target as it's cast: an instant or sorcery card in your graveyard (None: cancelled)"""
    cs = [x for x in p.gy if (x.instant or x.sorcery) and x is not c]
    k = _choose(g, p, 'target', "Mizzix's Mastery: exile which instant or sorcery (you cast a copy free)?",
                [_choices().card_label(x) for x in cs])
    return None if k is None else cs[k]


def flashback_target(g, p, c):
    """Flashback (the card): target instant or sorcery card in your graveyard gains flashback this turn"""
    cs = [x for x in p.gy if (x.instant or x.sorcery) and x is not c]
    if not cs: return
    k = _choose(g, p, 'target', 'Flashback: which instant or sorcery gains flashback this turn (its mana cost)?',
                [_choices().card_label(x) for x in cs], cancel=None)
    st = E.turn_stamp(g)
    if getattr(p, 'fb_grant', (None,))[0] != st: p.fb_grant = (st, set())     # last turn's grants have ended
    p.fb_grant[1].add(id(cs[k]))
    E.log(f'    {cs[k].name} gains flashback this turn', g)


def granted_flashback(g, p, c):
    fb = getattr(p, 'fb_grant', None)
    return fb is not None and fb[0] == E.turn_stamp(g) and id(c) in fb[1]


NEEDS['Mizzix\'s Mastery'] = lambda g, p, c: (None if any((x.instant or x.sorcery) for x in p.gy) else
                                              "Mizzix's Mastery needs an instant or sorcery card in your graveyard to target.")
NEEDS['Flashback'] = lambda g, p, c: (None if any((x.instant or x.sorcery) for x in p.gy) else
                                      'Flashback needs an instant or sorcery card in your graveyard to target.')


# ------------------------------------------------------------------ choices as your spells and triggers resolve (Marchesa's deck)
def _creatures(g, p, opp_only=False):
    """creatures p's spell or ability can target (opponents' hexproof and shroud respected)"""
    return [m for q in g.players if q.alive and not (opp_only and q is p) for m in q.perms if m.creature and not m.phased
            and not (m.owner is not p and E.untargetable(g, m))]


def pick_creature(g, p, prompt, opp_only=False, optional=False):
    tg = _creatures(g, p, opp_only)
    if not tg: return None
    k = _choose(g, p, 'target', prompt, [legal.describe_target(g, p, m) for m in tg], cancel='no target' if optional else None)
    return None if k is None else tg[k]


def crux_mode(g, p):
    k = _choose(g, p, 'choose', 'Crux of Fate: choose one', ['destroy all Dragons', 'destroy all non-Dragon creatures'], cancel=None)
    return 'dragons' if k == 0 else 'others'


def dredge(g, p):
    """your draw step: dredge a card in your graveyard instead of drawing? True if you did"""
    imps = [c for c in p.gy if 'dredge' in c.tags]
    if not imps or len(p.library) < 5: return False
    if not _choices().yes_no(g, p, f'Draw step: dredge {imps[0].name} (mill five, return it to your hand) instead of drawing?'):
        return False
    p.gy.remove(imps[0]); p.hand.append(imps[0])
    E.mill(g, p, 5); p.stats['dredged'] += 1
    E.log(f'  {E.NAME(p)} dredges {imps[0].name} (mills five)', g)
    return True


CAST_TARGET = {'Act of Treason': 'Act of Treason: gain control of which creature until end of turn?'}
NEEDS['Act of Treason'] = lambda g, p, c: None if _creatures(g, p) else 'Act of Treason has no creature to target.'
