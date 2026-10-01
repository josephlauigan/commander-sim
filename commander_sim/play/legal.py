"""The rules check for the human seat: is this move legal right now? Each check returns None (legal) or a one-line
reason to show the player. Nothing here changes the game."""
from commander_sim import engine as E
from commander_sim.play import mana

MAIN_STEPS = ('main1', 'main2')


def land_drops(g, p):
    """land drops p has this turn (Exploration, Azusa and similar add more)"""
    return 1 + (E.CI.total(g, 'extra_lands', p) if g.hooks else 0) + getattr(p, 'extra_land_now', 0)


def sorcery_timing(g, p):
    """None if p may do sorcery-speed things now (its own main phase), else why not"""
    if g.active is not p: return "It isn't your turn."
    if getattr(g, 'step', None) not in MAIN_STEPS: return 'You can only do that in one of your main phases.'
    if getattr(g, 'responding', 0): return 'You can only do that when nothing is waiting to resolve (not in response).'
    return None


def check_land(g, p, c):
    if c not in p.hand: return f"{c.name} isn't in your hand."
    if not c.land: return f"{c.name} isn't a land."
    why = sorcery_timing(g, p)
    if why: return why.replace('do that', 'play a land')
    if getattr(p, 'lands_played', 0) >= land_drops(g, p):
        return "You've already played a land this turn." if land_drops(g, p) == 1 else "You've used all your land drops this turn."
    return None


def instant_speed(c):
    return c.instant or 'flash' in c.tags or 'flash' in (c.kws or ())


def base_cost(g, p, c):
    """what c costs to cast (before target-dependent extras). The engine stores a few cards at the cost the AI always
    pays; the person pays the printed cost"""
    if c.name == "Bloodchief's Thirst": return 0, 'B'           # {B}; kicker {2}{B} for a target over mana value 2
    return E.cost_of(p, c)


def check_cast(g, p, c, zone='hand'):
    """can p cast c from zone ('hand' or 'cmd') now? Checks where it is, timing, locks and the mana in the pool"""
    if zone == 'hand' and c not in p.hand: return f"{c.name} isn't in your hand."
    if zone == 'cmd' and not (c is p.cmd and p.cmd_in_zone): return 'Your commander is not in the command zone.'
    if c.land: return f'{c.name} is a land: play it as your land drop instead.'
    if 'ctr' in c.tags: return f'{c.name} counters a spell: cast it when a spell you can counter is cast.'
    if E.silenced(g, p): return "You can't cast spells during this player's turn (Conqueror's Flail)."
    if c.name == 'Return the Favor': return 'Return the Favor targets a spell: cast it in response to one.'
    if not instant_speed(c) and c.name != 'Necromancy':     # Necromancy: "as though it had flash"
        why = sorcery_timing(g, p)
        if why: return why.replace('do that', f'cast {c.name}')
    if not E.castable(g, p, c, zone): return f"Something on the battlefield stops you casting {c.name} right now."
    if 'needsac' in c.tags and not any(m.creature and not m.phased for m in p.perms):
        return f'{c.name} needs a creature to sacrifice as you cast it, and you control none.'
    if c.dsl and not E.additional_cost(g, p, c, dry=True): return f"You can't pay {c.name}'s additional cost."
    tg = spell_targets(g, p, c)
    if tg is not None and not tg and 'wipe' not in c.tags: return f'{c.name} has no legal target right now.'
    if 'rean' in c.tags:
        from commander_sim.play import choices
        if not choices.rean_candidates(g, p, c.tags['rean']): return f'{c.name} has no creature card to return.'
    if c.name == 'Deadly Dispute' and not p.treasures and not any(
            (m.creature or (m.cd is not None and 'A' in m.cd.types)) and not m.phased for m in p.perms):
        return 'Deadly Dispute needs an artifact or creature to sacrifice as you cast it.'
    from commander_sim.play import cards
    why = cards.needs(g, p, c)
    if why: return why
    gen, pips = base_cost(g, p, c)
    why = mana.cost_problem(g, p, gen, pips)
    if why: return f"Can't cast {c.name}. {why}"
    return None


# ------------------------------------------------------------------ targets
TGT_OK = {'c': lambda m, ty: m.creature, 'cp': lambda m, ty: m.creature or 'P' in ty,
          'cap': lambda m, ty: m.creature or 'A' in ty or 'P' in ty, 'ce': lambda m, ty: m.creature or 'E' in ty,
          'nl': lambda m, ty: True, 'p': lambda m, ty: True, 'a': lambda m, ty: 'A' in ty,
          'cna': lambda m, ty: m.creature and 'A' not in ty, 'ae': lambda m, ty: 'A' in ty or 'E' in ty,
          'blue': lambda m, ty: m.cd is not None and 'U' in m.cd.pips}
WHAT = {'c': 'creature', 'cp': 'creature or planeswalker', 'cap': 'creature, artifact or planeswalker',
        'ce': 'creature or enchantment', 'nl': 'nonland permanent', 'p': 'permanent', 'a': 'artifact',
        'cna': 'nonartifact creature', 'ae': 'artifact or enchantment', 'blue': 'blue permanent'}


def player_targetable(q):
    """Teferi's Protection and The One Ring give protection from everything"""
    return q.alive and not getattr(q, 'life_locked', False) and not getattr(q, 'ring_prot', False)


def spell_targets(g, p, c):
    """what spell c may target, by the rules (not by what the AI would pick): a list of permanents and players, or
    None if c doesn't target. Your own permanents count too; hexproof, shroud and protection are respected"""
    t = c.tags
    if 'rem' not in t: return None
    tgt = t.get('tgt', 'c')
    ok = TGT_OK.get(tgt, TGT_OK['c'])
    out = []
    for q in g.players:
        if not q.alive: continue
        for m in q.perms:
            ty = m.cd.types if m.cd is not None else 'C'
            if m.phased or not ok(m, ty): continue
            if 'alsoart' in t and not m.creature and 'A' not in ty: continue
            if E.untargetable(g, m) or E.protected_from(g, m, c.pips): continue
            if 'nonblack' in t and 'B' in E.colors_of(m): continue
            if 'mv4' in t and m.cd is not None and m.cd.cmc > 4: continue
            if 'newonly' in t and not E.entered_since_last_turn(g, p, m): continue
            if c.name == 'Fatal Push' and m.cd is not None and m.cd.cmc > 2:
                from commander_sim.cards.impl import rules
                if m.cd.cmc > 4 or not rules.revolt(g, p): continue
            out.append(m)
    if 'face' in t: out += [q for q in g.players if player_targetable(q)]
    return out


def describe_target(g, p, x):
    """'Sheoldred, the Apocalypse (Prosper, tapped)' / 'Krenko (player, 31 life)'"""
    if isinstance(x, E.Player): return f"{E.NAME(x)} (player, {x.life} life)" + (' (you)' if x is p else '')
    from commander_sim.play.view import token_name
    name = token_name(x) if x.token else x.name
    bits = ['you' if x.owner is p else E.NAME(x.owner)]
    if x.creature: bits.append(f'{E.epow(g, x)}/{E.etgh(g, x)}')
    if x.tapped: bits.append('tapped')
    return f"{name} ({', '.join(bits)})"


def target_extra_cost(g, p, c, x):
    """more mana this target needs: ward (on an opponent's permanent), Bloodchief's Thirst's kicker"""
    gen, pips = 0, ''
    if isinstance(x, E.Perm) and x.owner is not p and x.cd is not None and x.cd.ward: gen += x.cd.ward
    if c.name == "Bloodchief's Thirst" and isinstance(x, E.Perm) and x.cd is not None and x.cd.cmc > 2:
        gen, pips = gen + 2, pips + 'B'                       # kicked: {2}{B} more, any mana value
    return gen, pips


# ------------------------------------------------------------------ equipment
EQUIP_COST = {'sword': 2, 'flail': 2, 'cloak': 2, 'animist': 2, 'helm': 1, 'nim': 4}


def equip_cost(m):
    """the equip cost of equipment m, or None if m isn't equipment"""
    if m.cd is None: return None
    for a in (m.cd.dsl or ()):
        if a.get('static') == 'equip_cost': return int(a.get('mana', 0))
    t = m.cd.tags
    if t.get('prot') == 'boots': return 0 if m.cd.name == 'Lightning Greaves' else 1
    for k, v in EQUIP_COST.items():
        if k in t: return v
    return None


def check_equip(g, p, e, target):
    if e not in p.perms or equip_cost(e) is None: return "That isn't equipment you control."
    why = sorcery_timing(g, p)
    if why: return why.replace('do that', 'equip')
    if target not in p.perms or not target.creature or target.phased: return 'Equip only onto a creature you control.'
    if e.attached is target: return f'{e.name} is already on {target.name}.'
    why = mana.cost_problem(g, p, equip_cost(e), '')
    if why: return f"Can't equip {e.name}. {why}"
    return None


# ------------------------------------------------------------------ counterspells
def alternative_counter_cost(g, p, ctr):
    """a way to cast counterspell ctr without its mana cost right now, in words, or None"""
    t = ctr.tags
    blues = [x for x in p.hand if x is not ctr and 'U' in x.pips]
    if 'fierce' in t and E.commander_out(p): return 'free while your commander is out'
    if 'pact' in t: return 'free now; pay {3}{U}{U} at your next upkeep or lose'
    if 'misstep' in t and p.life > 10: return '2 life'
    if 'fon' in t and g.active is not p and blues: return 'exile a blue card'
    if 'free' in t and blues: return 'exile a blue card and pay 1 life'
    return None


def check_counter(g, p, ctr, spell):
    """can p counter `spell` (on the stack) with ctr from hand now?"""
    if ctr not in p.hand: return f"{ctr.name} isn't in your hand."
    if 'ctr' not in ctr.tags: return f"{ctr.name} isn't a counterspell."
    if not E.counter_ok(ctr, spell): return f"{ctr.name} can't counter {spell.name}."
    if E.silenced(g, p): return "You can't cast spells during this player's turn (Conqueror's Flail)."
    if g.hooks and not E.castable(g, p, ctr): return f"Something on the battlefield stops you casting {ctr.name} right now."
    gen, pips = E.counter_cost(ctr, spell)
    if mana.cost_problem(g, p, gen, pips) is None or alternative_counter_cost(g, p, ctr): return None
    return f"Can't cast {ctr.name}. {mana.cost_problem(g, p, gen, pips)}"


# ------------------------------------------------------------------ casting from the graveyard
FLASHBACK = {'Unburial Rites': (3, 'W'), 'Dread Return': 'sac3'}   # flashback costs the engine's tags leave out


def gy_mode(g, p, c):
    """how p may cast card c from their graveyard now: ('yawg' | 'flashback' | 'sac3' | 'escape', generic, pips), or
    None. Yawgmoth's Will: the normal cost; flashback: its own cost; Dread Return: sacrifice three creatures;
    Underworld Breach: the normal cost plus three other cards from your graveyard exiled"""
    if c not in p.gy or c.land: return None
    if getattr(p, 'yawg', False) and id(c) in getattr(p, 'yawg_gy', {}):
        gen, pips = E.cost_of(p, c); return ('yawg', gen, pips)
    fb = c.tags.get('fb') or FLASHBACK.get(c.name)
    if not fb and (c.instant or c.sorcery):
        from commander_sim.play import cards
        if cards.granted_flashback(g, p, c): fb = E.cost_of(p, c)       # Flashback (the card): its mana cost
    if fb == 'sac3': return ('sac3', 0, '')
    if fb:
        gen, pips = fb if isinstance(fb, tuple) else E.parse_cost(fb)
        return ('flashback', gen, pips)
    if E.has(p, 'breach') and len(p.gy) >= 4:
        gen, pips = E.cost_of(p, c); return ('escape', gen, pips)
    return None


def check_cast_gy(g, p, c):
    mode = gy_mode(g, p, c)
    if mode is None: return f"You can't cast {c.name} from your graveyard right now."
    if not instant_speed(c):
        why = sorcery_timing(g, p)
        if why: return why.replace('do that', f'cast {c.name}')
    if E.silenced(g, p): return "You can't cast spells during this player's turn (Conqueror's Flail)."
    if not E.castable(g, p, c, 'gy'): return f"Something on the battlefield stops you casting {c.name} right now."
    kind, gen, pips = mode
    if kind == 'sac3' and sum(1 for m in p.perms if m.creature and not m.phased) < 3:
        return f"{c.name}'s flashback needs three creatures to sacrifice."
    if 'fblife' in c.tags and kind == 'flashback' and p.life <= int(c.tags['fblife']):
        return f"{c.name}'s flashback costs {c.tags['fblife']} life, which you don't have to spare."
    if 'rean' in c.tags:
        from commander_sim.play import choices
        if not choices.rean_candidates(g, p, c.tags['rean']): return f'{c.name} has no creature card to return.'
    why = mana.cost_problem(g, p, gen, pips)
    if why: return f"Can't cast {c.name} from your graveyard. {why}"
    return None


def check_land_gy(g, p, c):
    if c not in p.gy or not c.land: return f"{c.name} isn't a land in your graveyard."
    if not (getattr(p, 'yawg', False) and id(c) in getattr(p, 'yawg_gy', {})):
        return "You can't play lands from your graveyard right now."
    why = sorcery_timing(g, p)
    if why: return why.replace('do that', 'play a land')
    if getattr(p, 'lands_played', 0) >= land_drops(g, p): return "You've already played a land this turn."
    return None
