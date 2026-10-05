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


choose = _choose


def pick_exile(g, p, pool, n, prompt, default=()):
    """'you may exile n cards from your graveyard': no, the suggested n, or n picked one at a time. The cards, or []"""
    names = ', '.join(c.name for c in default)
    k = _choose(g, p, 'choose', prompt, [f'yes: {names}', 'yes, and I pick the cards'], cancel='no')
    if k is None: return []
    if k == 0: return list(default)
    picked = []
    while len(picked) < n:
        left = [c for c in pool if c not in picked]
        j = _choose(g, p, 'choose', f'Exile which card? ({len(picked) + 1} of {n})', [c.name for c in left], cancel=None)
        picked.append(left[j])
    return picked


def pick_blue(g, p, blues, what):
    """an alternative cost that exiles a blue card from your hand (Force of Will, Force of Negation): which one"""
    if len(blues) == 1: return blues[0]
    k = _choose(g, p, 'choose', f'{what}: exile which blue card from your hand?', [c.name for c in blues], cancel=None)
    return blues[k]


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
                if not E.ability_window(g, p, m.cd if m not in p.perms else m, f'{cost:+d}'): return None
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
        E.cast_copy(g, p, eff, name=spell, instant=instant)
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
        E.die(g, m, 'sac')
        if E.ability_window(g, p, m.cd, 'draw a card'): E.draw(g, p, 1)
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
        if E.ability_window(g, p, m, '1 damage', target=tg[k]): _damage(g, p, tg[k], 1, 'Triskelion', 'triggers')
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
        if not E.ability_window(g, p, m, '50 damage', imp=9, target=tg[k]): return None
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
        if not E.ability_window(g, p, m, 'adapt 4'): return None
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
        if E.ability_window(g, p, m, 'draw a card'): E.draw(g, p, 1)
        return None
    return [('{1}{B}, sacrifice another creature or a Treasure: draw a card', act)]


def _ste(g, p, m):
    def act(g, p, m):
        E.log(f'  {E.NAME(p)} sacrifices Sakura-Tribe Elder', g)
        E.die(g, m, 'sac')
        if E.ability_window(g, p, m.cd, 'search for a basic land'): E.land_ramp(g, p, 1, True)
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
            if E.ability_window(g, p, m, text): effect(g, p, m, pw)
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
        m.tapped = True
        E.log(f'  {E.NAME(p)} puts a charge counter on Coalition Relic', g)
        if E.ability_window(g, p, m, 'a charge counter') and m in p.perms:
            m.data = dict(m.data or {}, charge=(m.data or {}).get('charge', 0) + 1)
        return None
    return [('{T}: put a charge counter on it (each one is a mana of any colour at your next precombat main phase)', act)]


def _citadel(g, p, m):
    """Bolas's Citadel: play the top card of your library (a spell for life equal to its mana value instead of its
    mana cost); {T}, sacrifice ten nonland permanents: each opponent loses 10 life"""
    out = []
    top = p.library[-1] if p.library else None

    def play_top(g, p, m):
        if not p.library or p.library[-1] is not top: return 'The top card of your library has changed.'
        if top.land:
            why = legal.sorcery_timing(g, p)
            if why: return why.replace('do that', 'play a land')
            if getattr(p, 'lands_played', 0) >= legal.land_drops(g, p): return "You've already played a land this turn."
            p.library.pop(); ais.play_land_card(g, p, top, 'plays (from the top, Citadel)')
            E.check_state(g); return None
        if not legal.instant_speed(top):
            why = legal.sorcery_timing(g, p)
            if why: return why.replace('do that', f'cast {top.name}')
        if 'ctr' in top.tags: return f'{top.name} counters a spell: it can only be cast in response to one.'
        if any(k in top.tags for k in ('x', 'tokx', 'xtutor', 'xdrain', 'crackle', 'rean', 'deluge')):
            return f"{top.name} has a choice of X or a graveyard target the Citadel path doesn't support; draw it instead."
        if p.life <= top.cmc: return f'Casting {top.name} costs {top.cmc} life; you have {p.life}.'
        if not E.castable(g, p, top, 'lib'): return f'Something on the battlefield stops you casting {top.name} right now.'
        from commander_sim.play import human
        return human.cast(g, p, top, 'citadel')
    if top is not None:
        what = f'play {top.name}' if top.land else f'cast {top.name} for {top.cmc} life'
        out.append((f'{what} from the top of your library', play_top))

    def boom(g, p, m):
        why = _tapped(m, "Bolas's Citadel")
        if why: return why
        fod = [x for x in p.perms if not x.phased and x is not m]
        if len(fod) + 1 < 10: return f'You need ten nonland permanents to sacrifice; you have {len(fod) + 1}.'
        from commander_sim.play import choices
        picked = [m]
        while len(picked) < 10:
            left = [x for x in fod if x not in picked]
            k = _choose(g, p, 'choose', f"Bolas's Citadel: sacrifice which permanent? ({len(picked) + 1} of 10)",
                        [legal.describe_target(g, p, x) for x in left], cancel=None)
            picked.append(left[k])
        m.tapped = True
        for x in picked: E.die(g, x, 'sac')
        for q in g.opps(p): E.lose_life(g, q, 10, p)
        E.log(f"  {E.NAME(p)} sacrifices ten permanents to Bolas's Citadel: each opponent loses 10", g)
        E.check_state(g); return None
    out.append(('{T}, sacrifice ten nonland permanents: each opponent loses 10 life', boom))
    return out


ABILITIES = {
    "Bolas's Citadel": _citadel,
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
        E.log(f'  {E.NAME(p)} sacrifices Strip Mine: destroy {x.cd.name} ({E.NAME(q)})', g)
        if E.ability_window(g, p, L.cd, f'destroy {x.cd.name}') and x in q.lands:
            importlib.import_module('commander_sim.cards.impl.fixes').destroy_land(g, q, x)
        return None
    return [('{T}, sacrifice: destroy target land', act)]


def _lighthouse(g, p, L):
    def act(g, p, L):
        why = _tapped(L, 'Desolate Lighthouse') or _cost(g, p, 1, 'UR', 'Desolate Lighthouse')
        if why: return why
        mana.pay_from_pool(g, p, 1, 'UR'); L.tapped = True
        E.log(f'  {E.NAME(p)} loots with Desolate Lighthouse', g)
        if E.ability_window(g, p, L.cd, 'draw, then discard'): E.draw(g, p, 1); E.discard_worst(g, p, 1)
        return None
    return [('{1}{U}{R}, {T}: draw a card, then discard a card', act)]


def _summit(g, p, L):
    def act(g, p, L):
        why = _tapped(L, 'Spectacle Summit') or _cost(g, p, 2, 'UR', 'Spectacle Summit')
        if why: return why
        mana.pay_from_pool(g, p, 2, 'UR'); L.tapped = True
        if E.ability_window(g, p, L.cd, 'surveil 1'): _choices().scry(g, p, 1, to='gy')
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
        E.log(f'  {E.NAME(p)} uses Barad-dûr: amass Orcs {x}', g)
        if E.ability_window(g, p, L.cd, f'amass Orcs {x}'): E.amass(g, p, x)
        return None
    return [('{X}{X}{B}, {T}: amass Orcs X (only if a creature died this turn)', act)]


def _mistrise(g, p, L):
    def act(g, p, L):
        why = _tapped(L, 'Mistrise Village') or _cost(g, p, 0, 'U', 'Mistrise Village')
        if why: return why
        mana.pay_from_pool(g, p, 0, 'U'); L.tapped = True
        p.mistrise_next = E.turn_stamp(g)
        E.log(f"  {E.NAME(p)} activates Mistrise Village: the next spell this turn can't be countered", g)
        return None
    return [("{U}, {T}: the next spell you cast this turn can't be countered", act)]


LANDS = {'Strip Mine': _strip, 'Desolate Lighthouse': _lighthouse, 'Spectacle Summit': _summit, 'Barad-dûr': _baraddur,
         'Mistrise Village': _mistrise}


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
    'Clever Concealment': lambda g, p, c: [('phase out any number of your nonland permanents (convoke)', lambda g, p, c: _concealment(g, p, c))],
    'Rootborn Defenses': lambda g, p, c: [('populate; your creatures gain indestructible until end of turn', lambda g, p, c: _rootborn(g, p, c))],
    'Momentary Blink': lambda g, p, c: [('exile a creature you control, then return it', lambda g, p, c: _momentary(g, p, c))],
    'Triplicate Spirits': lambda g, p, c: [('three 1/1 white flying Spirits (convoke)', lambda g, p, c: _triplicate(g, p, c))],
}


def _you_control_a_creature(g, p, c):
    if not any(m.creature and not m.phased for m in p.perms): return f'{c.name} needs a creature you control to target.'


def _any_creature(g, p, c):
    if not any(m.creature and not m.phased and not (m.owner is not p and E.untargetable(g, m))
               for q in g.players if q.alive for m in q.perms):
        return f'{c.name} has no legal target right now.'


NEEDS = {'Twinflame': _you_control_a_creature, 'Ephemerate': _you_control_a_creature,
         'Momentary Blink': _you_control_a_creature,
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


# ------------------------------------------------------------------ Zur the Enchanter's deck
def _zur():
    return importlib.import_module('commander_sim.cards.impl.zur')


def convoke_pay(g, p, c, gen, pips):
    """convoke: tap your creatures to help pay (each pays {1} or one coloured pip of its colour), the rest from your
    pool. None once paid, or why not (nothing is tapped or spent then)"""
    tapped = []
    while gen or pips:
        left = [m for m in p.perms if m.creature and not m.tapped and not m.phased and m not in tapped]
        if not left: break
        k = _choose(g, p, 'choose', f'{c.name} (convoke): tap a creature to help pay? Still to pay: {mana.cost_text(gen, pips)}',
                    [legal.describe_target(g, p, m) for m in left], cancel='pay the rest with mana')
        if k is None: break
        m = left[k]
        col = next((x for x in pips if x in E.colors_of(m)), None)
        if col: pips = pips.replace(col, '', 1)
        elif gen: gen -= 1
        else: continue
        tapped.append(m)
    why = mana.cost_problem(g, p, gen, pips)
    if why: return f"Can't cast {c.name}. {why}"
    for m in tapped: m.tapped = True
    mana.pay_from_pool(g, p, gen, pips)
    if tapped: E.log(f'  {E.NAME(p)} taps {", ".join(m.name for m in tapped)} to convoke {c.name}', g)
    return None


def _timing(g, p, c):
    if E.silenced(g, p): return "You can't cast spells during this player's turn (Conqueror's Flail)."
    if not legal.instant_speed(c):
        why = legal.sorcery_timing(g, p)
        if why: return why.replace('do that', f'cast {c.name}')
    return None


def _concealment(g, p, c):
    why = _timing(g, p, c)
    if why: return why
    picked = []
    while True:
        left = [m for m in p.perms if not m.phased and m not in picked and m.attached is None]
        if not left: break
        k = _choose(g, p, 'target', f'Clever Concealment: phase out which of your permanents? ({len(picked)} so far)',
                    [legal.describe_target(g, p, m) for m in left], cancel='done' if picked else 'cancel')
        if k is None: break
        picked.append(left[k])
    if not picked: return None
    why = convoke_pay(g, p, c, 2, 'WW')
    if why: return why
    if not _spell_cast(g, p, c, 4, f'Clever Concealment on {", ".join(m.name for m in picked)}'):
        p.gy.append(c); return None
    for m in picked:
        if m in p.perms:
            m.phased = True
            for x in p.perms + [y for q in g.players for y in q.perms]:
                if x.attached is m: x.phased = True              # what's attached phases out with it
    p.gy.append(c)
    E.log(f'    {", ".join(m.name for m in picked)} phase out', g)
    return None


def _rootborn(g, p, c):
    why = _timing(g, p, c) or mana.cost_problem(g, p, 2, 'W')
    if why: return why if why.startswith(('You', 'Cast', 'It')) and 'costs' not in why else f"Can't cast {c.name}. {why}"
    mana.pay_from_pool(g, p, 2, 'W')
    if not _spell_cast(g, p, c, 3): p.gy.append(c); return None
    _zur().rootborn(g, p)
    p.gy.append(c)
    return None


def _momentary(g, p, c, zone='hand'):
    gen, pips = (1, 'W') if zone == 'hand' else (3, 'U')
    why = _timing(g, p, c)
    if why: return why
    cre = [m for m in p.perms if m.creature and not m.phased]
    if not cre: return 'Momentary Blink needs a creature you control to target.'
    k = _choose(g, p, 'target', 'Momentary Blink: exile and return which creature you control?',
                [legal.describe_target(g, p, m) for m in cre])
    if k is None: return None
    why = mana.cost_problem(g, p, gen, pips)
    if why: return f"Can't cast Momentary Blink{' (flashback)' if zone == 'gy' else ''}. {why}"
    mana.pay_from_pool(g, p, gen, pips)
    m = cre[k]
    if zone == 'hand':
        if not _spell_cast(g, p, c, 3, f'Momentary Blink on {m.name}'): p.gy.append(c); return None
        p.gy.append(c)
    else:
        p.gy.remove(c)
        p.spells_this_turn += 1; p.stats['spells_cast'] += 1; p.cast_names.add(c.name)
        E.log(f'  {E.NAME(p)} casts Momentary Blink (flashback) on {m.name}', g)
        E.on_cast(g, p, c)
        ok = not g.over and E.counter_window(g, p, c, 3, {})
        p.exile.append(c)
        if not ok: return None
    if m in p.perms: _blink(g, p, m)
    return None


def _triplicate(g, p, c):
    why = _timing(g, p, c)
    if why: return why
    why = convoke_pay(g, p, c, 4, 'WW')
    if why: return why
    if not _spell_cast(g, p, c, 3): p.gy.append(c); return None
    E.make_tokens(g, p, 3, 1, fly=True, color='W', types=('spirit',))
    p.gy.append(c)
    return None


def _embrace_gy(g, p, c):
    """Demonic Embrace from your graveyard: its cost, 3 life and a discard"""
    why = legal.sorcery_timing(g, p)
    if why: return why.replace('do that', 'cast Demonic Embrace')
    others = list(p.hand)
    if not others: return 'Demonic Embrace from your graveyard needs a card in hand to discard.'
    cre = [m for q in g.players if q.alive for m in q.perms
           if m.creature and not m.phased and not (m.owner is not p and E.untargetable(g, m))
           and not _zur().untargetable_by_you(g, m) and not E.protected_from(g, m, 'B')]
    if not cre: return 'Demonic Embrace has no creature to enchant.'
    k = _choose(g, p, 'target', 'Demonic Embrace: enchant which creature?', [legal.describe_target(g, p, m) for m in cre])
    if k is None: return None
    j = _choose(g, p, 'choose', 'Demonic Embrace: discard which card?', [x.name for x in others])
    if j is None: return None
    why = mana.cost_problem(g, p, 1, 'BB')
    if why: return f"Can't cast Demonic Embrace. {why}"
    mana.pay_from_pool(g, p, 1, 'BB')
    E.lose_life(g, p, 3, p); E.discard_cards(g, p, [others[j]])
    p.gy.remove(c)
    p.spells_this_turn += 1; p.stats['spells_cast'] += 1; p.cast_names.add(c.name)
    E.log(f'  {E.NAME(p)} casts Demonic Embrace from the graveyard (3 life, discards {others[j].name})', g)
    E.on_cast(g, p, c)
    if g.over or not E.counter_window(g, p, c, 3, {}): return None
    host = cre[k]
    g.attach_to = host if host in host.owner.perms else None
    try:
        E.enter(g, p, c, was_cast=True)
    finally:
        g.attach_to = None
    E.check_state(g)
    return None


GY = {'Momentary Blink': lambda g, p, c: _momentary(g, p, c, 'gy'), 'Demonic Embrace': _embrace_gy}


def _guildmage_tap(g, p, m):
    why = _cost(g, p, 2, 'W', 'Azorius Guildmage')
    if why: return why
    cre = [x for q in g.players if q.alive for x in q.perms if x.creature and not x.phased
           and not (x.owner is not p and E.untargetable(g, x))]
    if not cre: return 'There is no creature to tap.'
    k = _choose(g, p, 'target', 'Azorius Guildmage: tap which creature?', [legal.describe_target(g, p, x) for x in cre])
    if k is None: return None
    mana.pay_from_pool(g, p, 2, 'W')
    E.log(f'  {E.NAME(p)} activates Azorius Guildmage: tap {cre[k].name}', g)
    if E.ability_window(g, p, m, f'tap {cre[k].name}', target=cre[k]): cre[k].tapped = True
    return None


def _officer(g, p, m):
    why = _cost(g, p, 3, 'W', 'Recruitment Officer')
    if why: return why
    mana.pay_from_pool(g, p, 3, 'W')
    if E.ability_window(g, p, m, 'look at the top four'): _zur().officer_dig(g, p)
    return None


def _wanderer_plus(g, p, m):
    cands = [x for q in g.players if q.alive for x in q.perms if not x.phased and (x.creature or (x.cd is not None and 'A' in x.cd.types))
             and not (x.owner is not p and E.untargetable(g, x))]
    k = _choose(g, p, 'target', 'The Eternal Wanderer +1: exile which artifact or creature until its owner\'s next end step?',
                [legal.describe_target(g, p, x) for x in cands], cancel='no target') if cands else None
    if k is None: return lambda: None
    t = cands[k]
    return lambda: _zur().wanderer_exile(g, p, t) if t in t.owner.perms else None


def _wanderer_zero(g, p, m):
    def go():
        for x in E.make_tokens(g, p, 1, 2, color='W', types=('samurai',)): x.data = dict(x.data or {}, kws=('double strike',))
    return go


def _wanderer_minus4(g, p, m):
    mine = [x for x in p.perms if x.creature and not x.phased]
    keep = None
    if mine:
        k = _choose(g, p, 'choose', 'The Eternal Wanderer -4: which of your creatures do you keep? (the rest are sacrificed)',
                    [legal.describe_target(g, p, x) for x in mine], cancel=None)
        keep = mine[k]

    def go():
        for q in g.players:
            if not q.alive: continue
            k2 = keep if q is p else _zur().keep_one(g, q, p)
            for x in [x for x in q.perms if x.creature and not x.phased and x is not k2]: E.die(g, x, 'sac')
    return go



def _guildmage_counter(g, p, m):
    abil = [it for it in reversed(g.stack) if it.kind == 'ability']
    if not abil: return 'There is no activated ability on the stack to counter.'
    why = _cost(g, p, 2, 'U', 'Azorius Guildmage')
    if why: return why
    k = 0
    if len(abil) > 1:
        k = _choose(g, p, 'target', 'Azorius Guildmage: counter which ability?', [f'{it.name} ({E.NAME(it.controller)})' for it in abil])
        if k is None: return None
    t = abil[k]
    mana.pay_from_pool(g, p, 2, 'U')
    E.log(f'  {E.NAME(p)} activates Azorius Guildmage: counter {t.name}', g)
    if E.ability_window(g, p, m, f'counter {t.name}', target=t) and t in g.stack: t.countered = True
    return None


ABILITIES['Azorius Guildmage'] = lambda g, p, m: [('{2}{W}: tap target creature', _guildmage_tap)] + (
    [('{2}{U}: counter target activated ability', _guildmage_counter)] if any(it.kind == 'ability' for it in g.stack) else [])
ABILITIES['Recruitment Officer'] = lambda g, p, m: [('{3}{W}: look at the top four, take a creature card with mana value 3 or less', _officer)]
ABILITIES['The Eternal Wanderer'] = _walker([(1, "exile up to one target artifact or creature until its owner's next end step", _wanderer_plus),
                                             (0, 'create a 2/2 white Samurai with double strike', _wanderer_zero),
                                             (-4, 'each player keeps one creature and sacrifices the rest', _wanderer_minus4)])


def _niv_draw(g, p, m):
    why = _tapped(m) or ("Niv-Mizzet has summoning sickness (it came under your control this turn)." if m.sick else None)
    if why: return why
    m.tapped = True
    E.log(f'  {E.NAME(p)} activates Niv-Mizzet, the Firemind: draw a card', g)
    if E.ability_window(g, p, m, 'draw a card'): E.draw(g, p, 1)
    return None


def _torch_fiend(g, p, m):
    why = _cost(g, p, 0, 'R', 'Torch Fiend')
    if why: return why
    arts = [x for q in g.players if q.alive for x in q.perms if not x.phased and x.cd is not None and 'A' in x.cd.types
            and not (x.owner is not p and (E.untargetable(g, x) or E.protected_from(g, x, 'R')))]
    if not arts: return 'There is no artifact to target.'
    k = _choose(g, p, 'target', 'Torch Fiend: destroy which artifact?', [legal.describe_target(g, p, x) for x in arts])
    if k is None: return None
    mana.pay_from_pool(g, p, 0, 'R')
    E.log(f'  {E.NAME(p)} sacrifices Torch Fiend: destroy {arts[k].name}', g)
    E.die(g, m, 'sac')
    if E.ability_window(g, p, m.cd, f'destroy {arts[k].name}', target=arts[k]) and arts[k] in arts[k].owner.perms:
        E.apply_removal(g, p, arts[k], 'destroy')
    return None


def _relic_legend(g, p, m):
    legs = [x for x in p.perms if x.creature and not x.tapped and not x.phased and x.cd is not None
            and _mine().is_legendary(g, x)]
    if not legs: return 'You have no untapped legendary creature to tap.'
    k = _choose(g, p, 'choose', 'Relic of Legends: tap which legendary creature?', [legal.describe_target(g, p, x) for x in legs])
    if k is None: return None
    j = _choose(g, p, 'choose', 'Relic of Legends: which colour?', list(p.ident) or ['C'])
    if j is None: return None
    legs[k].tapped = True
    col = (list(p.ident) or ['C'])[j]
    mana.pool_of(p).add(col, 1)
    E.log(f'  {E.NAME(p)} taps {legs[k].name} for Relic of Legends: {{{col}}}', g)
    return None


ABILITIES['Niv-Mizzet, the Firemind'] = lambda g, p, m: [('{T}: draw a card', _niv_draw)]
ABILITIES['Torch Fiend'] = lambda g, p, m: [('{R}, sacrifice Torch Fiend: destroy target artifact', _torch_fiend)]
ABILITIES['Relic of Legends'] = lambda g, p, m: [('tap an untapped legendary creature you control: one mana of any colour', _relic_legend)]

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


def pick_creature(g, p, prompt, opp_only=False, optional=False, keep=None):
    tg = [m for m in _creatures(g, p, opp_only) if keep is None or keep(g, p, m)]
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


# ------------------------------------------------------------------ Galadriel's deck (Bant Rebels)
def _gal():
    return importlib.import_module('commander_sim.cards.impl.galadriel')


def _creature_tap_ok(m):
    return _tapped(m) or (f'{m.cd.name} has summoning sickness (it came under your control this turn).' if m.sick else None)


def _gal_pay(g, p, m, gen, pips, what):
    """pay a creature's ability from your pool (Secluded Courtyard's mana may pay if it's of the named type).
    None once paid, or why not"""
    prev, mana.SPENDING = mana.SPENDING, m
    try:
        why = _cost(g, p, gen, pips, what)
        if why: return why
        mana.pay_from_pool(g, p, gen, pips)
    finally:
        mana.SPENDING = prev
    return None


def _pick_x(g, p, what, lo=0):
    """X for an ability, up to the mana in your pool"""
    n = mana.pool_of(p).total() + sum(getattr(p, a, 0) for a in ('floatR', 'floatU', 'floatC', 'floatA', 'floatG', 'floatB'))
    if n < lo: return None
    xs = list(range(lo, n + 1))
    k = _choose(g, p, 'choose', f'{what}: X = ? (your pool has {n} mana)', [f'X = {x}' for x in xs])
    return None if k is None else xs[k]


def _searcher_act(cost, cap):
    def act(g, p, m):
        why = _creature_tap_ok(m) or _gal_pay(g, p, m, cost, '', m.cd.name)
        if why: return why
        m.tapped = True
        E.log(f'  {E.NAME(p)} activates {m.cd.name}', g)
        if E.ability_window(g, p, m, f'search for a Rebel (mana value {cap} or less)'): _gal().put_rebel(g, p, m.cd.name, cap)
        return None
    return act


def _lin_search(g, p, m):
    why = _creature_tap_ok(m)
    if why: return why
    x = _pick_x(g, p, 'Lin Sivvi')
    if x is None: return None
    why = _gal_pay(g, p, m, x, '', 'Lin Sivvi')
    if why: return why
    m.tapped = True
    E.log(f'  {E.NAME(p)} activates Lin Sivvi, X = {x}', g)
    if E.ability_window(g, p, m, f'search for a Rebel (mana value {x} or less)'): _gal().put_rebel(g, p, 'Lin Sivvi', x)
    return None


def _lin_bottom(g, p, m):
    cs = _gal().rebel_cards(p, p.gy, 99) + [c for c in p.gy if not c.perm and 'rebel' in c.subtypes]
    if not cs: return 'There is no Rebel card in your graveyard.'
    k = _choose(g, p, 'target', 'Lin Sivvi: put which Rebel card from your graveyard on the bottom of your library?',
                [c.name for c in cs])
    if k is None: return None
    why = _gal_pay(g, p, m, 3, '', 'Lin Sivvi')
    if why: return why
    c = cs[k]
    E.log(f'  {E.NAME(p)} activates Lin Sivvi: {c.name} to the bottom of the library', g)
    if E.ability_window(g, p, m, f'put {c.name} on the bottom of the library', target=c) and c in p.gy:
        p.gy.remove(c); p.library.insert(0, c)
    return None


def _revivalist(g, p, m):
    why = _creature_tap_ok(m)
    if why: return why
    if not _gal().rebel_cards(p, p.gy, 5): return 'There is no Rebel permanent card with mana value 5 or less in your graveyard.'
    why = _gal_pay(g, p, m, 6, '', 'Ramosian Revivalist')
    if why: return why
    m.tapped = True
    E.log(f'  {E.NAME(p)} activates Ramosian Revivalist', g)
    if E.ability_window(g, p, m, 'return a Rebel from your graveyard'): _gal().put_rebel(g, p, 'Ramosian Revivalist', 5, 'gy')
    return None


def _bringer(colour, word):
    def act(g, p, m):
        why = _creature_tap_ok(m)
        if why: return why
        cre = [x for q in g.players if q.alive for x in q.perms if x.creature and not x.phased and colour in E.colors_of(x)
               and not (x.owner is not p and E.untargetable(g, x)) and not E.protected_from(g, x, 'W')]
        if not cre: return f'There is no {word} creature to target.'
        k = _choose(g, p, 'target', f'{m.cd.name}: exile which {word} creature?', [legal.describe_target(g, p, x) for x in cre])
        if k is None: return None
        _gal().bringer_use(g, p, m, cre[k])
        return None
    return act


def _ballista(g, p, m):
    why = _creature_tap_ok(m)
    if why: return why
    fighting = set(getattr(g, 'in_combat', ()) or ()) | set(getattr(g, 'blocking', ()) or ())
    cre = [x for x in fighting if x in x.owner.perms and not (x.owner is not p and E.untargetable(g, x))
           and not E.protected_from(g, x, 'W')]
    if not cre: return 'Ballista Squad needs an attacking or blocking creature to target.'
    k = _choose(g, p, 'target', 'Ballista Squad: X damage to which attacking or blocking creature?', [legal.describe_target(g, p, x) for x in cre])
    if k is None: return None
    x = _pick_x(g, p, 'Ballista Squad (plus {W})', 0)
    if x is None: return None
    why = _gal_pay(g, p, m, x, 'W', 'Ballista Squad')
    if why: return why
    m.tapped = True
    t = cre[k]
    E.log(f'  {E.NAME(p)} activates Ballista Squad: {x} damage to {t.name}', g)
    if E.ability_window(g, p, m, f'{x} damage to {t.name}', target=t) and t in t.owner.perms:
        if E.no_damage(g, t): E.log(f'    the damage to {t.name} is prevented', g)
        elif E.etgh(g, t) <= x: E.apply_removal(g, p, t, f'dmg{x}')
        else: E.log(f'    {t.name} survives', g)
    return None


def _tapper_act(gen, pips, tough):
    def act(g, p, m):
        why = _creature_tap_ok(m)
        if why: return why
        cre = [x for q in g.players if q.alive for x in q.perms if x.creature and not x.phased and E.etgh(g, x) <= tough
               and not (x.owner is not p and E.untargetable(g, x))]
        if not cre: return 'There is no creature it can tap.'
        k = _choose(g, p, 'target', f'{m.cd.name}: tap which creature?', [legal.describe_target(g, p, x) for x in cre])
        if k is None: return None
        why = _gal_pay(g, p, m, gen, pips, m.cd.name)
        if why: return why
        m.tapped = True
        t = cre[k]
        E.log(f'  {E.NAME(p)} activates {m.cd.name}: tap {t.name}', g)
        if E.ability_window(g, p, m, f'tap {t.name}', target=t) and t in t.owner.perms: t.tapped = True
        return None
    return act


def _maskwood(g, p, m):
    why = _tapped(m) or _cost(g, p, 3, '', 'Maskwood Nexus')
    if why: return why
    mana.pay_from_pool(g, p, 3, '')
    m.tapped = True
    E.log(f'  {E.NAME(p)} activates Maskwood Nexus', g)
    if E.ability_window(g, p, m, 'a 2/2 Shapeshifter with changeling'): _gal().shapeshifter(g, p)
    return None


def _mirror(g, p, m):
    x = _pick_x(g, p, 'Mirror Entity', 0)
    if x is None: return None
    why = _gal_pay(g, p, m, x, '', 'Mirror Entity')
    if why: return why
    E.log(f'  {E.NAME(p)} activates Mirror Entity, X = {x}', g)
    if E.ability_window(g, p, m, f'your creatures become {x}/{x}'): _gal().mirror(g, p, x)
    return None


def _elspeth_plus(g, p, m):
    return lambda: E.make_tokens(g, p, 3, 1, color='W', types=('soldier',))


def _elspeth_minus3(g, p, m):
    def go():
        for q in g.players:
            for x in list(q.perms):
                if x.creature and not x.phased and E.epow(g, x) >= 4: E.die(g, x, 'destroy')
    return go


def _elspeth_minus7(g, p, m):
    return lambda: E.CI.elspeth_emblem(g, p)


def _turn_up(g, p, m):
    why = _cost(g, p, 0, 'W', 'turn Whipcorder face up')
    if why: return why
    mana.pay_from_pool(g, p, 0, 'W')
    _gal().turn_face_up(g, p, m)
    return None


def _morph_cast(g, p, c):
    why = _timing(g, p, c) or _cost(g, p, 3, '', 'cast it face down')
    if why: return why.replace("Can't activate", "Can't cast")
    mana.pay_from_pool(g, p, 3, '')
    if not _spell_cast(g, p, c, 2, 'a face-down creature (morph)'): p.gy.append(c); return None
    _gal().enter_face_down(g, p, c)
    return None


for _n, (_cost_n, _cap) in (('Ramosian Sergeant', (3, 2)), ('Ramosian Lieutenant', (4, 3)), ('Ramosian Captain', (5, 4)),
                            ('Defiant Vanguard', (5, 4)), ('Ramosian Commander', (6, 5))):
    ABILITIES[_n] = (lambda c, k: lambda g, p, m: [(f'{{{c}}}, {{T}}: search for a Rebel permanent card with mana value {k} '
                                                    f'or less, put it onto the battlefield', _searcher_act(c, k))])(_cost_n, _cap)
ABILITIES['Lin Sivvi, Defiant Hero'] = lambda g, p, m: [
    ('{X}, {T}: search for a Rebel permanent card with mana value X or less, put it onto the battlefield', _lin_search),
    ('{3}: put target Rebel card from your graveyard on the bottom of your library', _lin_bottom)]
ABILITIES['Ramosian Revivalist'] = lambda g, p, m: [
    ('{6}, {T}: return a Rebel permanent card with mana value 5 or less from your graveyard to the battlefield', _revivalist)]
ABILITIES['Lawbringer'] = lambda g, p, m: [('{T}, sacrifice it: exile target red creature', _bringer('R', 'red'))]
ABILITIES['Lightbringer'] = lambda g, p, m: [('{T}, sacrifice it: exile target black creature', _bringer('B', 'black'))]
ABILITIES['Ballista Squad'] = lambda g, p, m: [('{X}{W}, {T}: X damage to target attacking or blocking creature', _ballista)]
ABILITIES['Errant Doomsayers'] = lambda g, p, m: [('{T}: tap target creature with toughness 2 or less', _tapper_act(0, '', 2))]
ABILITIES['Whipcorder'] = lambda g, p, m: ([('turn it face up ({W})', _turn_up)] if (m.data or {}).get('facedown')
                                           else [('{W}, {T}: tap target creature', _tapper_act(0, 'W', 999))])
ABILITIES['Maskwood Nexus'] = lambda g, p, m: [('{3}, {T}: a 2/2 blue Shapeshifter with changeling', _maskwood)]
ABILITIES['Mirror Entity'] = lambda g, p, m: [('{X}: until end of turn, your creatures have base power and toughness '
                                               'X/X and gain all creature types', _mirror)]
ABILITIES["Elspeth, Sun's Champion"] = _walker([(1, 'create three 1/1 white Soldiers', _elspeth_plus),
                                                (-3, 'destroy all creatures with power 4 or greater', _elspeth_minus3),
                                                (-7, 'emblem: your creatures get +2/+2 and have flying', _elspeth_minus7)])
HAND['Whipcorder'] = lambda g, p, c: [('cast it ({W}{W})', _cast_normally),
                                      ('cast it face down as a 2/2 creature for {3} (morph)', _morph_cast)]



# ------------------------------------------------------------------ Necropotence (Zur's deck)
def _necro(g, p, m):
    if ais.blocked(g, p, 'Necropotence'): return 'Disruptor Flute names Necropotence: its ability can\'t be activated.'
    most = min(p.life, len(p.library), 30)
    if most < 1: return 'You have no life or no library to pay with.'
    ns = list(range(1, most + 1))
    k = _choose(g, p, 'choose', f'Necropotence: pay how much life? (you have {p.life}; the cards come to your hand at '
                                f'your next end step)', [f'pay {n} life' for n in ns])
    if k is None: return None
    n = ns[k]
    E.lose_life(g, p, n, p)
    E.log(f'  {E.NAME(p)} pays {n} life to Necropotence', g)
    if E.ability_window(g, p, m, f'exile the top {n} card(s) face down') and not g.over: ais.necro_exile(g, p, n)
    E.check_state(g)
    return None


ABILITIES['Necropotence'] = lambda g, p, m: [('pay 1 life (any number of times): exile the top card of your library '
                                              'face down; put it into your hand at the beginning of your next end step', _necro)]


# ------------------------------------------------------------------ Y'shtola's deck (Esper Drain)
def _ysh():
    return importlib.import_module('commander_sim.cards.impl.yshtola')


def _own_hand(p):
    """the cards in your hand (not the ones taken from opponents, which are in exile)"""
    stolen = getattr(p, 'stolen', None) or {}
    return [c for c in p.hand if id(c) not in stolen]


def _syphon_mage(g, p, m):
    why = _creature_tap_ok(m) or ('You have no card in hand to discard.' if not _own_hand(p) else None) \
        or _cost(g, p, 2, 'B', 'Urborg Syphon-Mage')
    if why: return why
    cs = _own_hand(p)
    k = _choose(g, p, 'choose', 'Urborg Syphon-Mage: discard which card?', [_choices().card_label(c) for c in cs])
    if k is None: return None
    mana.pay_from_pool(g, p, 2, 'B'); m.tapped = True
    E.discard_cards(g, p, [cs[k]])
    _ysh().syphon(g, p, m)
    return None


def _inheritance(g, p, m):
    why = _cost(g, p, 5, 'B', 'Ill-Gotten Inheritance')
    if why: return why
    opps = g.opps(p)
    if not opps: return 'There is no opponent to target.'
    k = _choose(g, p, 'target', 'Ill-Gotten Inheritance: 4 damage to which opponent?',
                [f'{E.NAME(q)} ({q.life} life)' for q in opps])
    if k is None: return None
    mana.pay_from_pool(g, p, 5, 'B')
    _ysh().inheritance_sac(g, p, m, opps[k])
    return None


def _jesters_cap(g, p, m):
    why = _tapped(m) or _cost(g, p, 2, '', "Jester's Cap")
    if why: return why
    ps = [q for q in g.players if q.alive]
    k = _choose(g, p, 'target', "Jester's Cap: search which player's library?",
                [E.NAME(q) + (' (you)' if q is p else '') for q in ps])
    if k is None: return None
    mana.pay_from_pool(g, p, 2, '')
    _ysh().cap_use(g, p, m, ps[k])
    return None


ABILITIES['Urborg Syphon-Mage'] = lambda g, p, m: [
    ('{2}{B}, {T}, discard a card: each other player loses 2 life; you gain the life lost this way', _syphon_mage)]
ABILITIES['Ill-Gotten Inheritance'] = lambda g, p, m: [
    ('{5}{B}, sacrifice it: 4 damage to target opponent; you gain 4 life', _inheritance)]
ABILITIES["Jester's Cap"] = lambda g, p, m: [
    ("{2}, {T}, sacrifice it: search target player's library for three cards and exile them", _jesters_cap)]
CAST_TARGET['Take Up the Shield'] = ('Take Up the Shield: which creature gets a +1/+1 counter, lifelink and indestructible?',
                                     lambda g, p, m: not (m.owner is p and _zur().untargetable_by_you(g, m)))
NEEDS['Take Up the Shield'] = lambda g, p, c: None if _creatures(g, p) else 'Take Up the Shield has no creature to target.'


def _narset_minus2(g, p, m):
    return lambda: importlib.import_module('commander_sim.cards.impl.common').narset_dig(g, p, m)


ABILITIES['Narset, Parter of Veils'] = _walker([(-2, 'look at the top four; take a noncreature, nonland card', _narset_minus2)])
