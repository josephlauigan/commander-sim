"""Jodah, the Unifier (JD's WUBRG Legends deck, key `jodah`, decklists/JD/jodah-wubrg-legends.md).

- Jodah: legendary creatures you control get +X/+X (X = the legendary creatures you control); whenever you cast a
  legendary spell from your hand, exile from the top until a legendary nonland card with lesser mana value, cast it
  free, the rest to the bottom in a random order.
- The AI: the outside decks' generic priorities, with legendary spells first while Jodah is out (the more expensive,
  the more the cascade can find).
"""
import importlib
from commander_sim.engine import *
from commander_sim import engine as E
from commander_sim.cards import cardimpl as CI
from commander_sim.cards.cardimpl import on
from commander_sim.cards.pool_cards import card, note
from commander_sim.cards.impl import common as IC

JODAH = 'Jodah, the Unifier'


def full(name, text): note(name, 'Full', text)


def human(g, p):
    return E.human_choice(g, p)


def legendary(g, m):
    """a legendary permanent: printed, a legendary token (Kaldra, Voja), or the Ring-bearer"""
    if m.cd is None: return bool(m.data and m.data.get('legendary'))
    return importlib.import_module('commander_sim.cards.impl.mine').is_legendary(g, m)


def legend_card(c):
    return 'leg' in c.tags and not c.land


def jodahs(p):
    return sum(1 for m in p.perms if m.cd is not None and m.cd.name == JODAH and not m.phased)


# ------------------------------------------------------------------ Jodah: the anthem
def _jodah_pt(g, m):
    p = m.owner
    k = jodahs(p)
    if not k or not legendary(g, m): return 0, 0
    x = k * sum(1 for y in p.perms if y.creature and not y.phased and legendary(g, y))
    return x, x
IC.CREATURE_PT.append(_jodah_pt)


@on(JODAH, 'etb')
def _jodah_etb(g, src, p, m):
    if m is src: g.selfpt = True                     # the engine reads legends' size from CREATURE_PT from here on


# ------------------------------------------------------------------ Jodah: the legend cascade
@on(JODAH, 'cast')
def _jodah_cast(g, src, caster, c):
    o = src.owner
    if caster is not o or not legend_card(c): return
    cur = getattr(g, 'cur_cast', None)
    if cur is None or cur[0] is not c or cur[2] != 'hand': return          # only spells cast from your hand
    if not trigger_window(g, o, src, f'cascade into a legend with mana value below {c.cmc}', imp=4): return
    jodah_cascade(g, o, c.cmc)


def jodah_cascade(g, o, mv):
    """exile from the top until a legendary nonland card with mana value below mv; cast it free (you may); the rest
    go to the bottom in a random order"""
    return cascade(g, o, mv, 'Jodah')


def cascade(g, o, mv, who):
    """Jodah's legend cascade (who 'Jodah') or plain cascade (Maelstrom Nexus, Maelstrom Wanderer): exile from the top
    until a nonland card (legendary, for Jodah) with mana value below mv; cast it free (you may: the AI passes on
    counterspells and on the protection it keeps for responses); the rest go to the bottom in a random order"""
    seen, hit = [], None
    while o.library:
        x = o.library.pop()
        if not x.land and x.cmc < mv and (who != 'Jodah' or legend_card(x)):
            hit = x; break
        seen.append(x)
    cast = False
    if hit is not None:
        hc = human(g, o)
        want = hc.yes_no(g, o, f'{who}: cast {hit.name} without paying its mana cost?') if hc is not None else \
            'ctr' not in hit.tags and hit.name not in CI.RESPONSE_ONLY.get(o.key, ())
        if want and castable(g, o, hit, 'lib'):
            g.last_x = 0                                  # cast free: X is 0
            if who == 'Jodah':
                o.milestone.setdefault('jodah', o.turns)
                o.stats['jodah_cascades'] += 1
            else:
                o.stats['jr_cascade_hits'] += 1
            log(f'    {who} reveals {hit.name} ({len(seen)} other cards): cast free', g)
            cast_card(g, o, hit, 'lib', {})
            cast = True
        else:
            log(f'    {who} reveals {hit.name} ({len(seen)} other cards): not cast', g)
            seen.append(hit)
    else:                                             # nothing cheaper: the whole library is exiled, then goes back
        log(f'    {who} exiles {len(seen)} cards and finds no {"legendary " if who == "Jodah" else ""}nonland card '
            f'with mana value below {mv}: they all go to the bottom in a random order (the library is shuffled)', g)
    g.rng.shuffle(seen)
    o.library[:0] = seen                              # to the bottom, random order
    return cast


full(JODAH, 'legends you control get +X/+X (X = your legendary creatures); casting a legendary spell from hand cascades '
     'into a legendary nonland card with lesser mana value, cast free')


# ================================================================== the AI
_AI = set(__import__('os').environ.get('JODAH_AI', 'first,tutor').split(','))   # audit/jodah: switch fixes off for A/B


def jodah_payable(g, p):
    """Jodah is in the command zone (or in hand: Command Beacon) and can be cast now"""
    return (p.cmd_in_zone or p.cmd in p.hand) and can_pay(g, p, *cost_of(p, p.cmd))


def jodah_prio(g, p, c):
    """the generic priorities; with Jodah out, legendary spells first, the dearer the better (more to cascade into).
    With Jodah castable now, Jodah comes first: a legend cast before it gives up its cascade, and any other spell
    that leaves too little mana for Jodah puts it off a turn."""
    from commander_sim.ai import pool_ai
    if c.name in PROTECT and not (legend_card(c) and jodahs(p)): return 0         # kept for removal and wipes
    v = pool_ai.generic_prio(g, p, c) or 0
    if c is p.cmd: return max(v, 86)
    if not v and c.dsl and E.DSLMOD is not None: v = int(E.DSLMOD.card_value(g, p, c) * 10)
    if legend_card(c) and jodahs(p):
        v = max(v, 40) + 4 + min(16, 2 * c.cmc)
    elif legend_card(c) and c.creature:
        v = max(v, 35)
    if 'first' in _AI and v > 15 and jodah_payable(g, p):
        gen, pips = cost_of(p, p.cmd); cg, cp = cost_of(p, c)
        if legend_card(c) or not can_pay(g, p, gen + cg, pips + cp): v = 15
    return min(90, v)


CI.jodah_prio = jodah_prio


def jodah_tutor(g, p, kind, okn):
    """the card a tutor finds: before Jodah, the mana that casts it (Coalition Relic fixes, Sisay's Ring doesn't); with
    Jodah out or on the way, the dearest legend not already in hand (its cascade can find any cheaper legend); the
    third Kaldra piece; Toxic Deluge or Force of Will under pressure (instant and sorcery tutors)"""
    if 'tutor' not in _AI: return None
    have = {m.cd.name for m in p.perms if m.cd is not None}
    hand = {c.name for c in p.hand}
    mana = len(p.lands) + sum(1 for m in p.perms if m.cd is not None and ('rock' in m.cd.tags or 'dork' in m.cd.tags))
    out = jodahs(p) > 0
    order = []
    if _AI & {'ptutor', 'ptutorall'} and out and not hand & PROTECTION and not untargetable(g, _the_jodah(p)) and \
            ('ptutorall' in _AI or importlib.import_module('commander_sim.ai.brain').removal_risk(g, p) >= 0.3):
        order += [n for n in PROTECTION_ORDER if n not in have]              # Jodah out and exposed: protect it
    if not out and p.cmd_in_zone and mana < 4:
        order += ['Coalition Relic', 'Star Compass', 'Moss Diamond', 'Fyndhorn Elder']
    kaldra = [n for n in KALDRA if n not in have | hand]
    if len(kaldra) == 1: order += kaldra
    big = sorted((c for c in E.searchable(g, p) if legend_card(c) and c.creature and c.name not in hand),
                 key=lambda c: (-c.cmc, c.name))
    if not any(legend_card(c) and c.cmc >= 6 for c in p.hand):
        order += [c.name for c in big if c.cmc >= 6 and c.name != 'Szadek, Lord of Secrets']
    if kind == 'is':
        under = sum(epow(g, m) for q in g.opps(p) for m in q.perms if m.creature and not m.phased) >= 15
        order += (['Toxic Deluge'] if under else []) + ['Demonic Tutor', 'Force of Will']
    if kind in ('art', 'ench', 'ae'):
        order += ['Blackblade Reforged', 'Privileged Position', 'Coalition Relic', 'Court of Ardenvale']
    return next((n for n in order if n in okn), None)


CI.jodah_tutor = jodah_tutor


# ================================================================== protecting Jodah (audit/jodah, 10-09)
# Cards the AI keeps for removal aimed at Jodah or a wipe that would hit it. name: (cost, free while Jodah is out,
# what it stops, what it does). Stops: 'tgt' targeted removal, 'destroy' destroy and damage (targeted or a wipe),
# 'both', 'all' (anything). Does: grant hexproof ('hex') and/or indestructible ('ind') to Jodah or (_all) to every
# creature (Flawless Maneuver, Unbreakable Formation) or permanent (Heroic Intervention, Lazotep Plating) until end of
# turn; 'swat' sends the removal at an opponent's permanent (the 'swat' cards also guard a key legend, see
# jodah_protect); 'coat' puts Mithril Coat on Jodah; 'phase' is Teferi's Protection. Listed cheapest first: free ones,
# then by mana (Bolt Bend: {R} with a creature of power 4 or more, as with Jodah out); Teferi's Protection last.
PROTECT = {
    'Flawless Maneuver':     ((2, 'W'), True, 'destroy', 'ind_all'),
    'Deflecting Swat':       ((2, 'R'), True, 'tgt', 'swat'),
    'Bolt Bend':             ((3, 'R'), False, 'tgt', 'swat'),
    "Tamiyo's Safekeeping":  ((0, 'G'), False, 'both', 'hexind'),
    "Loran's Escape":        ((0, 'W'), False, 'both', 'hexind'),
    'Snakeskin Veil':        ((0, 'G'), False, 'tgt', 'hex'),
    'Royal Treatment':       ((0, 'G'), False, 'tgt', 'hex'),
    'Dark Endurance':        ((1, 'B'), False, 'destroy', 'ind'),
    'Heroic Intervention':   ((1, 'G'), False, 'both', 'hexind_all'),
    'Lazotep Plating':       ((1, 'U'), False, 'tgt', 'hex_all'),
    'Unbreakable Formation': ((2, 'W'), False, 'destroy', 'ind_all'),
    'Mithril Coat':          ((3, ''), False, 'destroy', 'coat'),
    "Teferi's Protection":   ((2, 'W'), False, 'all', 'phase'),
}
CI.RESPONSE_ONLY['jodah'] = set(PROTECT)            # never cast for their interpreter value (brain.card_utility)
WIPE_DESTROY = ('destroy', 'dmg13', 'austere', 'austere2', 'nib')     # wipes that indestructible survives


def _the_jodah(p):
    return next((m for m in p.perms if m.cd is not None and m.cd.name == JODAH and not m.phased), None)


def _grant(g, ms, kws):
    for m in ms:
        g.eot_kw.setdefault(id(m), set()).update(kws)


def _saved(p, name, wipe=False):
    p.stats['jprot_saves'] += 1
    p.stats[f"jprot_{'wipe_' if wipe else ''}{name}"] += 1


def prot_cost(g, p, name):
    """a protection card's mana cost now: Bolt Bend costs {3} less with a creature of power 4 or more"""
    gen, pips = PROTECT[name][0]
    if name == 'Bolt Bend' and any(m.creature and not m.phased and epow(g, m) >= 4 for m in p.perms): gen = 0
    return gen, pips


def _cast_protection(g, p, c):
    """cast protection card c from hand in response (free if it can be): True if it resolves"""
    (gen, pips), free_cmd = prot_cost(g, p, c.name), PROTECT[c.name][1]
    free = free_cmd and commander_out(p)
    if c not in p.hand or not castable(g, p, c) or not (free or can_pay(g, p, gen, pips)): return False
    p.hand.remove(c)
    if not free: pay(g, p, gen, pips)
    p.spells_this_turn += 1; p.stats['spells_cast'] += 1; p.cast_names.add(c.name)
    log(f'  {NAME(p)} casts {c.name}' + (' (free)' if free else ''), g)
    on_cast(g, p, c)
    ok = counter_window(g, p, c, 6, {})
    if c.name == 'Mithril Coat' and ok:
        e = enter(g, p, c); e.attached = _the_jodah(p) or _best_legend(g, p)
    else:
        (p.exile if c.name == E.TEFERIS_PROTECTION else p.gy).append(c)
    return ok


def _swat_targets(g, p, kind):
    """where Deflecting Swat or Bolt Bend can send the removal: any opponent's permanent it can target"""
    return [x for q in g.opps(p) for x in q.perms
            if not x.phased and not untargetable(g, x) and (x.creature or not kind.startswith('dmg'))]


def _apply(g, p, j, name, does, spell=None, actor=None, kind=None):
    """the protection's effect on Jodah j (and the rest of p's board); False if it found nothing to do (Deflecting
    Swat with no new target left)"""
    if does == 'phase': E.teferis_protection(g, p); return True
    if does == 'swat':
        alt = _swat_targets(g, p, kind)
        if not alt: return False
        t = max(alt, key=lambda x: (pval(g, x), x.owner is actor))
        log(f'    {name}: {spell.name} now targets {t.name} ({NAME(t.owner)})', g)
        apply_removal(g, actor, t, kind, spell); return True
    kws = {'hex': {'hexproof'}, 'ind': {'indestructible'}, 'hexind': {'hexproof', 'indestructible'}}.get(does.split('_')[0], set())
    scope = [j]
    if does.endswith('_all'):
        scope = [m for m in p.perms if not m.phased and (m.creature or does != 'ind_all')]
    _grant(g, scope, kws)
    if name in ('Snakeskin Veil', 'Royal Treatment'): j.plus += 1            # the counter / the Royal Role's +1/+1
    if name == "Tamiyo's Safekeeping": gain(p, 2)
    if name == 'Lazotep Plating': amass(g, p, 1)
    if name == "Loran's Escape": importlib.import_module('commander_sim.cards.impl.topdeck').scry(g, p, 1)
    return True


def _stops(stops, kind, targeted):
    destroyish = kind == 'destroy' or kind.startswith('dmg') or (not targeted and kind in WIPE_DESTROY)
    return stops == 'all' or (stops in ('tgt', 'both') and targeted) or (stops in ('destroy', 'both') and destroyish)


def _rune_save(g, p, j, spell):
    """Giver of Runes (another creature: anything) or Mother of Runes (a coloured spell): tap for protection"""
    for name in ('Giver of Runes', 'Mother of Runes'):
        for m in _mine_named(p, name):
            if m is j or m.tapped or m.sick: continue
            if name == 'Mother of Runes' and not (spell is not None and set(spell.pips) & set('WUBRG')): continue
            m.tapped = True
            log(f'    {NAME(p)} taps {name}: Jodah gains protection', g)
            return name
    return None


def _plaza_save(g, p, j):
    """Plaza of Heroes: {3}, {T}, exile it: a legendary creature gains hexproof and indestructible until end of turn"""
    if 'noplaza' in _AI: return False                                        # audit/jodah A/B: as before 10-09
    IL = importlib.import_module('commander_sim.cards.impl.lands')
    for L in [L for L in p.lands if L.cd.name == 'Plaza of Heroes' and not L.tapped]:
        if not IL.can_pay_without(g, p, L, 3, '') or not IL.pay_without(g, p, L, 3, ''): continue
        p.lands.remove(L); p.exile.append(L.cd)
        log(f'    {NAME(p)} exiles Plaza of Heroes: Jodah gains hexproof and indestructible', g)
        if ability_window(g, p, L.cd, 'hexproof and indestructible for Jodah'):
            _grant(g, [j], {'hexproof', 'indestructible'}); return True
        return False
    return False


def key_legend(g, p, m):
    """a legendary creature other than Jodah worth a redirect (Deflecting Swat, Bolt Bend) while Jodah is not out to
    need it: value 5 or more (a bomb, or power 8 or more)"""
    return m.creature and legendary(g, m) and m.cd is not None and m.cd.name != JODAH and not jodahs(p) and pval(g, m) >= 5


def jodah_protect(g, p, m, kind, actor, spell=None):
    """removal aimed at Jodah: the cheapest answer that stops it (free spells, Giver or Mother of Runes, then by mana,
    Plaza of Heroes, Teferi's Protection last). Aimed at a key legend: a redirect only. True if it is safe"""
    if m.cd is None or kind in ('edict', 'wipe') or actor is None or actor is p: return False
    other = m.cd.name != JODAH
    if other and not key_legend(g, p, m): return False
    shrink = kind.startswith('shrink') or kind == 'zero'              # -X/-X: indestructible doesn't help
    n = None if other else _rune_save(g, p, m, spell)
    if n: _saved(p, n); return True
    for name, (cost, free, stops, does) in PROTECT.items():
        c = next((c for c in p.hand if c.name == name), None)
        if c is None or not _stops(stops, kind, True) or (shrink and stops == 'destroy') or (other and does != 'swat'):
            continue
        if does == 'swat' and (spell is None or not _swat_targets(g, p, kind)): continue
        if name == E.TEFERIS_PROTECTION and p.life_locked: continue
        if not _cast_protection(g, p, c) or not _apply(g, p, m, name, does, spell, actor, kind): return False
        _saved(p, name + (' (legend)' if other else ''))
        return True
    if not other and _plaza_save(g, p, m): _saved(p, 'Plaza of Heroes'); return True
    return False


def jodah_wipe_response(g, p, kind, caster):
    """a wipe that would take Jodah: indestructible (destroy and damage wipes) or Teferi's Protection. 'all', 'indes'
    (the whole board is indestructible) or None (Jodah alone may be safe)"""
    j = _the_jodah(p)
    if j is None or caster is p or not importlib.import_module('commander_sim.ais').wipe_hit(g, caster, kind)(j, p):
        return None
    for name, (cost, free, stops, does) in PROTECT.items():
        if does in ('swat',) or not _stops(stops, kind, False): continue
        c = next((c for c in p.hand if c.name == name), None)
        if c is None or (name == E.TEFERIS_PROTECTION and p.life_locked): continue
        if not _cast_protection(g, p, c): return None
        _apply(g, p, j, name, does); _saved(p, name, wipe=True)
        return 'all' if does == 'phase' else 'indes' if does in ('ind_all', 'hexind_all') else None
    if kind in WIPE_DESTROY and _plaza_save(g, p, j): _saved(p, 'Plaza of Heroes', wipe=True)
    return None


CI.jodah_protect = jodah_protect
HOLD = [float(x) for x in __import__('os').environ.get('JODAH_HOLD', '3,6').split(',')]   # audit/jodah A/B


def jodah_hold(g, p):
    """(card, value) of keeping mana up for a protection spell while Jodah is out (switch hold): worth more the likelier
    an opponent holds instant removal; free spells (Flawless Maneuver, Deflecting Swat) need nothing kept"""
    if 'hold' not in _AI or not jodahs(p): return None, 0.0
    cs = [c for c in p.hand if c.name in PROTECT and not (PROTECT[c.name][1] and commander_out(p))
          and can_pay(g, p, *prot_cost(g, p, c.name))]
    if not cs: return None, 0.0
    c = min(cs, key=lambda c: prot_cost(g, p, c.name)[0] + len(prot_cost(g, p, c.name)[1]))
    return c, HOLD[0] + HOLD[1] * importlib.import_module('commander_sim.ai.brain').removal_risk(g, p)


CI.jodah_hold = jodah_hold
PROTECTION_ORDER = ('Lightning Greaves', 'Swiftfoot Boots', 'Mithril Coat', 'Giver of Runes', 'Mother of Runes',
                    'Flawless Maneuver', 'Deflecting Swat', 'Bolt Bend', 'Heroic Intervention', "Tamiyo's Safekeeping",
                    "Loran's Escape", "Teferi's Protection", 'Snakeskin Veil', 'Royal Treatment', 'Lazotep Plating',
                    'Unbreakable Formation', 'Dark Endurance')       # what the tutors find (switch ptutor), best first
PROTECTION = set(PROTECTION_ORDER)
CI.jodah_wipe_response = jodah_wipe_response


def jodah_options(g, p, post):
    """main phase: Swiftfoot Boots, Lightning Greaves or Mithril Coat onto Jodah when they are elsewhere (the generic
    equip only moves unattached equipment)"""
    j = _the_jodah(p)
    if j is None or post is None: return []
    o = []
    for e in p.perms:
        if e.cd is None or e.phased or e.attached is j: continue
        n = {'Swiftfoot Boots': 1, 'Lightning Greaves': 0, 'Mithril Coat': 3}.get(e.cd.name)
        if n is None or not can_pay(g, p, n, '') or (e.cd.name != 'Mithril Coat' and untargetable(g, j)): continue
        if e.cd.name == 'Mithril Coat' and indestructible(g, j): continue
        o.append((4.0 + 0.2 * pval(g, j) - 0.5 * n, f'equip {e.cd.name} to Jodah',
                  lambda e=e, n=n: j in p.perms and can_pay(g, p, n, '') and equip_to(g, p, e, j, n)))
    return o


CI.jodah_options = jodah_options


def _runes_home(g, p, m):
    """Jodah's AI keeps Mother and Giver of Runes home (untapped, to protect Jodah) instead of attacking with them"""
    if p.key == 'jodah': m.noatk = True
for _n in ('Mother of Runes', 'Giver of Runes'): CI.AS_ENTERS[_n] = _runes_home


@on('Mithril Coat', 'etb')
def _coat_etb(g, src, p, m):
    """attach it to a legendary creature you control: Jodah first"""
    if m is src and src.attached is None:
        src.attached = _the_jodah(src.owner) or _best_legend(g, src.owner)


card('Mithril Coat', 'leg flash prot=coat', types='A', dsl=[
    {'type': 'static', 'static': 'self_keyword', 'keyword': 'indestructible'},
    {'type': 'static', 'static': 'equip_keyword', 'keyword': 'indestructible'},
    {'type': 'static', 'static': 'equip_cost', 'mana': 3}])
full('Mithril Coat', 'flash; indestructible; attaches to your legendary creature as it enters (Jodah first); the '
     "equipped creature is indestructible; equip {3}. Jodah's AI casts it with Jodah out, or in response to "
     'destroy removal or a wipe')
for _n, _t, _s in (("Tamiyo's Safekeeping", 'G', 'hexproof and indestructible until end of turn; you gain 2 life'),
                   ("Loran's Escape", 'W', 'hexproof and indestructible until end of turn; scry 1'),
                   ('Snakeskin Veil', 'G', 'a +1/+1 counter and hexproof until end of turn'),
                   ('Dark Endurance', '1B', '+2/+0 and indestructible until end of turn (the +2/+0 is not modeled)'),
                   ('Lazotep Plating', '1U', 'amass Zombies 1; you and your permanents gain hexproof until end of turn')):
    card(_n, 'prot=jodah', types='I', dsl=[])
    full(_n, _s + " (Jodah's AI casts it in response to removal aimed at Jodah or a wipe)")
card('Royal Treatment', 'prot=jodah', types='I', dsl=[])
note('Royal Treatment', 'Approximate', 'hexproof until end of turn; the Royal Role is a +1/+1 counter (its ward {1} is '
     "not modeled); Jodah's AI casts it in response to removal aimed at Jodah")
card('Bolt Bend', 'prot=jodah', types='I', dsl=[])
full('Bolt Bend', "costs {3} less with a creature of power 4 or more ({R} with Jodah out); changes the target of a "
     "removal spell to an opponent's best permanent (Jodah's AI casts it in response to removal aimed at Jodah, or at a "
     'key legend while Jodah is not out)')


# ================================================================== the rework candidates (audit/jodah, 10-09)
# ------------------------------------------------------------------ Command Beacon
@on('Command Beacon', 'land_options')
def _beacon(g, L, p, s, post):
    """{T}, sacrifice: the commander from the command zone to hand, where casting it pays no commander tax. Used on
    your turn when Jodah is castable from hand after the sacrifice, and the tax is 4 or more, or it is 2 and Jodah
    can't be cast from the command zone this turn"""
    if post is None or L.tapped or g.active is not p or not p.cmd_in_zone or p.tax < 2: return []
    IL = importlib.import_module('commander_sim.cards.impl.lands')
    gen, pips = cost_of(p, p.cmd)
    E.PAY_FOR = p.cmd                                  # Plaza of Heroes' colours count
    try:
        after, now = IL.can_pay_without(g, p, L, gen - p.tax, pips), can_pay(g, p, gen, pips)
    finally:
        E.PAY_FOR = None
    if not after or (now and p.tax < 4): return []

    def go():
        if L not in p.lands or L.tapped or not p.cmd_in_zone: return False
        IL.sac_land(g, p, L); p.stats['jr_beacon'] += 1
        log(f'  {NAME(p)} sacrifices Command Beacon: {p.cmd.name} to hand (tax {p.tax} saved)', g)
        if ability_window(g, p, L.cd, f'{p.cmd.name} to hand') and p.cmd_in_zone:
            p.cmd_in_zone = False; p.hand.append(p.cmd)
        return True
    return [(9.5, f'Command Beacon: {p.cmd.name} to hand', go)]
full('Command Beacon', '{T}: {C}; {T}, sacrifice: your commander from the command zone to hand (cast from there '
     'without commander tax). The AI uses it when the tax is 4 or more, or 2 and it makes the commander castable now')


def _jodah_from_hand(g, p, c):
    cur = getattr(g, 'cur_cast', None)
    if cur is not None and cur[2] == 'hand': p.stats['jr_jodah_from_hand'] += 1     # audit/jodah: Beacon's payoff
CI.SELF_CAST[JODAH] = _jodah_from_hand


# ------------------------------------------------------------------ Maelstrom Nexus, Maelstrom Wanderer
@on('Maelstrom Nexus', 'cast')
def _nexus(g, src, caster, c):
    """the first spell you cast each turn has cascade (a legend from hand with Jodah out cascades twice)"""
    o = src.owner
    log_ = getattr(o, 'turn_casts', None)
    if caster is not o or not log_ or log_[0] != turn_stamp(g) or log_[1][0] is not c: return
    if not trigger_window(g, o, src, f'cascade (mana value below {c.cmc})', imp=4): return
    o.stats['jr_nexus'] += 1
    cascade(g, o, c.cmc, 'Maelstrom Nexus')
card('Maelstrom Nexus', '', types='E', dsl=[])
CI.SPELL_PRIO['Maelstrom Nexus'] = 50
full('Maelstrom Nexus', 'the first spell you cast each turn has cascade (it stacks with Jodah\'s legend cascade)')


def _wanderer_cast(g, p, c):
    """cascade, cascade (cast triggers: they resolve before the Wanderer)"""
    for _ in range(2):
        if not p.alive or g.over or not trigger_window(g, p, None, f'{c.name}: cascade', imp=4): continue
        p.stats['jr_wanderer'] += 1
        cascade(g, p, c.cmc, 'Maelstrom Wanderer')
CI.SELF_CAST['Maelstrom Wanderer'] = _wanderer_cast
note('Maelstrom Wanderer', 'Full', 'creatures you control have haste; cascade, cascade')


# ------------------------------------------------------------------ Sisay, Weatherlight Captain
def _sisay_pt(g, p, m):
    """+1/+1 for each colour among your other legendary permanents"""
    cols = set()
    for x in p.perms:
        if x is not m and not x.phased and legendary(g, x): cols |= colors_of(x)
    return len(cols), len(cols)
IC.SELF_PT['Sisay, Weatherlight Captain'] = _sisay_pt
CI.AS_ENTERS['Sisay, Weatherlight Captain'] = lambda g, p, m: setattr(g, 'selfpt', True)


def sisay_pick(g, p, power):
    """a legendary permanent card with mana value below Sisay's power: the tutor wish list's pick (the third Kaldra
    piece, the dearest legendary creature), else the dearest legend"""
    ok = [c for c in E.searchable(g, p) if 'leg' in c.tags and (c.perm or c.creature) and not c.land and c.cmc < power]
    if not ok: return None
    name = jodah_tutor(g, p, 'leg', {c.name for c in ok})
    return next((c for c in ok if c.name == name), None) or \
        max(ok, key=lambda c: (c.creature, c.cmc, card_worth(g, p, c)))


@on('Sisay, Weatherlight Captain', 'options')
def _sisay(g, src, p, s, post):
    """{W}{U}{B}{R}{G}: a legendary permanent card with mana value below Sisay's power onto the battlefield. On your
    turn, never while Jodah could be cast with the same mana"""
    if p is not src.owner or post is None or g.active is not p or not can_pay(g, p, 0, 'WUBRG'): return []
    if jodah_payable(g, p): return []
    t = sisay_pick(g, p, epow(g, src))
    if t is None: return []

    def go():
        if src not in p.perms or not can_pay(g, p, 0, 'WUBRG') or t not in p.library: return False
        pay(g, p, 0, 'WUBRG'); p.stats['jr_sisay'] += 1
        log(f'  {NAME(p)} activates Sisay: {t.name}', g)
        if not ability_window(g, p, src, f'search for {t.name}') or t not in p.library: return True
        p.library.remove(t); g.rng.shuffle(p.library)
        a = agent_for(g, p)
        if a is not None: agent_take(g, a, p, t)
        else: enter(g, p, t)
        return True
    return [(min(7.0, 1.5 + 0.6 * t.cmc), f'Sisay: {t.name}', go)]
card('Sisay, Weatherlight Captain', 'leg human pow=2 tgh=2', dsl=[])
full('Sisay, Weatherlight Captain', '+1/+1 for each colour among your other legendary permanents; {W}{U}{B}{R}{G}: a '
     "legendary permanent card with mana value below Sisay's power onto the battlefield (the tutor wish list's pick, "
     'else the dearest legend; never while Jodah could be cast instead)')
# ------------------------------------------------------------------ Shalai, Voice of Plenty (packages C and D)
SHALAI = 'Shalai, Voice of Plenty'


@on(SHALAI, 'grant_kw')
def _shalai_hexproof(g, src, m, kw):
    """your planeswalkers and other creatures have hexproof (Shalai herself doesn't)"""
    return kw == 'hexproof' and m is not src and m.owner is src.owner and (
        m.creature or (m.cd is not None and 'P' in m.cd.types))


@on(SHALAI, 'player_hexproof')
def _shalai_you(g, src, q):
    return q is src.owner


@on(SHALAI, 'options')
def _shalai_counters(g, src, p, s, post):
    """{4}{G}{G}: a +1/+1 counter on each creature you control; with mana left late (second main phase, or the end of
    the turn before yours) and two or more creatures"""
    cs = [m for m in p.perms if m.creature and not m.phased]
    if post is False or len(cs) < 2 or not can_pay(g, p, 4, 'GG'): return []

    def go():
        if src not in p.perms or not can_pay(g, p, 4, 'GG'): return False
        pay(g, p, 4, 'GG')
        if not ability_window(g, p, src, '+1/+1 counters'): return True
        for m in p.perms:
            if m.creature and not m.phased: m.plus += 1
        p.stats['jr_shalai_counters'] += 1
        log(f'  {NAME(p)} activates Shalai: a +1/+1 counter on each creature', g)
        return True
    return [(1.0 + 0.5 * len(cs), 'Shalai: +1/+1 counters', go)]


card(SHALAI, 'leg pow=3 tgh=4 fly', dsl=[])
CI.PVAL[SHALAI] = 6.0                         # opponents' removal goes to her once she shields the rest
full(SHALAI, 'flying; you, your planeswalkers and your other creatures have hexproof (opponents\' targeted removal, '
     'burn to the face and "target player" effects pass you by); {4}{G}{G}: a +1/+1 counter on each creature you '
     'control (used with mana left in the second main phase or at the end of the turn before yours)')
# Feed the Cycle (package B's on-theme removal): the additional cost is always paid as {B}, so it plays as {1}{B}{B}
card('Feed the Cycle', 'rem=destroy tgt=cp', types='I', cost='1BB', dsl=[])
note('Feed the Cycle', 'Approximate', 'instant: destroy target creature or planeswalker; the additional cost is always '
     'paid as {B} (forage, exiling three cards from your graveyard, is not modeled), so it costs {1}{B}{B}')


# ------------------------------------------------------------------ Venat, Heart of Hydaelyn // Hydaelyn (10-09)
VENAT = 'Venat, Heart of Hydaelyn'


def hydaelyn(m):
    return bool(m.data and m.data.get('hydaelyn'))


@on(VENAT, 'cast')
def _venat_draw(g, src, caster, c):
    """Venat: whenever you cast a legendary spell, draw a card; only once each turn"""
    o = src.owner
    if caster is not o or hydaelyn(src) or not legend_card(c) or o.flag_turn.get('venat') == turn_stamp(g): return
    if not trigger_window(g, o, src, 'draw a card') or not once_per_turn(g, o, 'venat'): return
    draw(g, o, 1); o.stats['jr_venat_draws'] += 1


@on(VENAT, 'options')
def _sundering(g, src, p, s, post):
    """Hero's Sundering: {7}, {T}, as a sorcery: exile target nonland permanent, then Venat transforms. On the best
    opposing nonland permanent, when it is worth 3 or more; never while Jodah could be cast instead"""
    if p is not src.owner or post is None or g.active is not p or hydaelyn(src) or src.tapped or src.sick \
            or not can_pay(g, p, 7, '') or (p.cmd.name == JODAH and jodah_payable(g, p)): return []
    t = IC.best_opp_nonland(g, p)
    if t is None or pval(g, t) < 3: return []

    def go():
        if src not in p.perms or src.tapped or t not in t.owner.perms or not can_pay(g, p, 7, ''): return False
        pay(g, p, 7, ''); src.tapped = True; p.stats['jr_venat_sunder'] += 1
        log(f"  {NAME(p)} activates Hero's Sundering: exile {t.name}, transform Venat", g)
        if not ability_window(g, p, src, f'exile {t.name}, transform', target=t): return True
        if t not in t.owner.perms or untargetable(g, t): return True              # no legal target: it does nothing
        apply_removal(g, p, t, 'exile', src.cd)
        if src in p.perms:
            if src.data is None: src.data = {}
            src.data['hydaelyn'] = True; src.pow, src.tgh = 4, 4
            log('    Venat transforms into Hydaelyn, the Mothercrystal', g)
        return True
    return [(1.0 + pval(g, t) * (1.25 if t.owner is s.leader else 1.0), f"Hero's Sundering -> {t.name}", go)]


@on(VENAT, 'grant_kw')
def _hydaelyn_indestructible(g, src, m, kw):
    return kw == 'indestructible' and m is src and hydaelyn(src)


@on(VENAT, 'crew')
def _blessing(g, src, p):
    """Hydaelyn, beginning of combat on your turn: a +1/+1 counter on another creature you control, indestructible
    until your next turn, a card if it is legendary. Jodah first, else the most valuable (a legend counts 2 more)"""
    if p is not src.owner or not hydaelyn(src) or not once_per_turn(g, p, 'blessing'): return
    p.stats['jr_hyd_turns'] += 1
    cs = [m for m in p.perms if m.creature and m is not src and not m.phased]
    if not cs: return
    t = max(cs, key=lambda m: (m.cd is not None and m.cd.name == JODAH, pval(g, m) + (2 if legendary(g, m) else 0)))
    if not trigger_window(g, p, src, f'+1/+1 counter, indestructible: {t.name}') or t not in p.perms: return
    t.plus += 1
    if t.data is None: t.data = {}
    t.data['indestr_until'] = (p, p.turns)                       # engine.indestructible
    log(f'    Hydaelyn blesses {t.name}: a +1/+1 counter, indestructible until {NAME(p)}\'s next turn', g)
    if legendary(g, t): draw(g, p, 1); p.stats['jr_hyd_draws'] += 1


card(VENAT, 'leg wizard pow=3 tgh=3', dsl=[], kws=())           # not the back face's indestructible
full(VENAT, 'whenever you cast a legendary spell, draw a card (once each turn); {7}, {T}, as a sorcery: exile the '
     'best opposing nonland permanent (worth 3 or more; Jodah is cast first when it can be), then it transforms into '
     'Hydaelyn: a 4/4 indestructible that at the beginning of your combat puts a +1/+1 counter on another creature of '
     'yours (Jodah first, else the most valuable, legends preferred), which is indestructible until your next turn and '
     'draws a card if legendary')


# ------------------------------------------------------------------ Tymna the Weaver (10-09)
TYMNA = 'Tymna the Weaver'


@on(TYMNA, 'combat_damage')
def _tymna_hit(g, src, p, a, d, dmg):
    if p is src.owner and d is not p: p.flag_turn[f'hit{g.players.index(d)}'] = turn_stamp(g)     # (bookkeeping)


@on(TYMNA, 'main2')
def _tymna(g, src, p):
    """at the beginning of your postcombat main phase: you may pay X life to draw X (X = opponents dealt combat
    damage this turn). The AI pays when the life left is at least Necropotence's floor (what the table could hit it
    for plus 6, at least 10) and the library has more than X cards"""
    st = turn_stamp(g)
    if p is not src.owner or p.flag_turn.get('tymna') == st: return
    x = sum(1 for i, q in enumerate(g.players) if q is not p and q.alive and p.flag_turn.get(f'hit{i}') == st)
    if not x: return
    if not trigger_window(g, p, src, f'pay {x} life, draw {x}') or not once_per_turn(g, p, 'tymna'): return
    if p.life < x or p.life_locked: return
    hc = human(g, p)
    if not (hc.yes_no(g, p, f'Tymna the Weaver: pay {x} life to draw {x}?') if hc is not None else
            p.life - x >= importlib.import_module('commander_sim.ais').necro_floor(g, p) and len(p.library) > x): return
    lose_life(g, p, x, p); draw(g, p, x); p.stats['jr_tymna_draws'] += x
    log(f'    {NAME(p)} pays {x} life to Tymna the Weaver and draws {x}', g)


card(TYMNA, 'leg human pow=2 tgh=2 lifelink', dsl=[])
full(TYMNA, 'lifelink; at the beginning of your postcombat main phase, pay X life to draw X (X = opponents dealt combat '
     'damage this turn): the AI pays while the life left stays at Necropotence\'s floor or above. Partner is ignored')


# ------------------------------------------------------------------ Garland, Royal Kidnapper (10-09)
GARLAND = 'Garland, Royal Kidnapper'


def garland_pick(g, p, q):
    """q's creature Garland's trigger takes: the most valuable one p can target"""
    cs = [m for m in q.perms if m.creature and not untargetable(g, m) and not protected_from(g, m, 'UB')]
    return max(cs, key=lambda m: pval(g, m), default=None)


@on(GARLAND, 'etb')
def _garland_etb(g, src, p, m):
    """When Garland enters, target opponent becomes the monarch: the one whose best creature is most worth stealing
    (then the one with the smaller board, the less likely to lose the crown)"""
    if m is not src: return
    o = src.owner
    opps = [q for q in g.opps(o) if q.alive and not player_hexproof(g, q)]
    if not opps: return
    hc = human(g, o)
    if hc is not None:
        q = opps[hc.choose(g, o, 'target', 'Garland: which opponent becomes the monarch?', [NAME(q) for q in opps], cancel=None)]
    else:
        def worth(q):
            t = garland_pick(g, o, q)
            return (pval(g, t) if t is not None else -1, -board_power(g, q))
        q = max(opps, key=worth)
    if not trigger_window(g, o, src, f'{NAME(q)} becomes the monarch', imp=4) or not q.alive: return
    o.stats['jr_garland_etb'] += 1
    CI.become_monarch(g, q)


@on(GARLAND, 'monarch')
def _garland_steal(g, src, q):
    """Whenever an opponent becomes the monarch, gain control of target creature that player controls for as long as
    they're the monarch (garland_check ends it)"""
    o = src.owner
    if q is o or not q.alive: return
    hc = human(g, o)
    t = importlib.import_module('commander_sim.play.cards').pick_creature(
        g, o, f'Garland: gain control of which of {NAME(q)}\'s creatures?', keep=lambda g, p, m: m.owner is q) \
        if hc is not None else garland_pick(g, o, q)
    if t is None or not trigger_window(g, o, src, f'gain control of {t.name}', imp=6): return
    if t not in q.perms or untargetable(g, t) or getattr(g, 'monarch', None) is not q: return   # the duration is over
    if importlib.import_module('commander_sim.ais').protect_response(g, q, t, 'steal', o, src.cd) or t not in q.perms: return
    importlib.import_module('commander_sim.cards.impl.marchesa').steal(g, o, t, until_eot=False)
    g.garland = (getattr(g, 'garland', None) or []) + [(o, t, q, g.round)]
    o.stats['jr_garland_steals'] += 1; o.stats[f'jr_garland_took:{t.name}'] += 1


@on(GARLAND, 'combat_damage')
def _garland_hit(g, src, p, a, d, dmg):
    """(bookkeeping: combat damage by the creatures Garland took)"""
    if p is src.owner and any(t is a for _, t, _, _ in getattr(g, 'garland', None) or ()):
        p.stats['jr_garland_dmg'] += dmg
        if a.data is None: a.data = {}
        if not a.data.get('garland_hit'): a.data['garland_hit'] = True; p.stats['jr_garland_hit'] += 1


def garland_check(g):
    """Garland's control effects end once the player a creature came from is no longer the monarch (or has left
    the game); a creature whose owner has left the game leaves with them"""
    keep = []
    for o, t, q, r in g.garland:
        if t not in o.perms: o.stats['jr_garland_gone'] += 1; continue           # it left (or changed control again)
        if q.alive and getattr(g, 'monarch', None) is q: keep.append((o, t, q, r)); continue
        o.perms.remove(t); g.bf_ver = getattr(g, 'bf_ver', 0) + 1
        o.stats['jr_garland_back'] += 1; o.stats['jr_garland_rounds'] += g.round - r
        if getattr(g, 'monarch', None) is o: o.stats['jr_garland_back_me'] += 1      # (you took the crown back)
        if t.orig.alive:
            t.owner = t.orig; t.orig.perms.append(t)
            log(f'    {t.name} returns to {NAME(t.orig)} (Garland)', g)
    g.garland = keep
CI.garland_check = garland_check


card(GARLAND, 'leg human knight pow=3 tgh=4 garland', dsl=[])
full(GARLAND, 'enters: target opponent becomes the monarch (the one whose best creature is most worth stealing); '
     'whenever an opponent becomes the monarch, gain control of their most valuable creature while they stay the '
     'monarch (it returns when they lose the crown or leave the game); creatures you control but don\'t own get +2/+2 '
     'and can\'t be sacrificed')


# ================================================================== the 99: cards that need code
def _attached(g, m, name):
    return [e for e in m.owner.perms if e.cd is not None and e.cd.name == name and e.attached is m and not e.phased]


def _mine_named(p, name):
    return [m for m in p.perms if m.cd is not None and m.cd.name == name and not m.phased]


def _best_legend(g, p, exclude=()):
    cs = [m for m in p.perms if m.creature and not m.phased and legendary(g, m) and m not in exclude]
    return max(cs, key=lambda m: (not m.sick, pval(g, m)), default=None)


# ------------------------------------------------------------------ Kaldra
card('Sword of Kaldra', 'leg', types='A', dsl=[{'type': 'static', 'static': 'equip_bonus', 'pow': 5, 'tgh': 5},
                                              {'type': 'static', 'static': 'equip_cost', 'mana': 4}])
card('Shield of Kaldra', 'leg', types='A', dsl=[{'type': 'static', 'static': 'equip_keyword', 'keyword': 'indestructible'},
                                               {'type': 'static', 'static': 'equip_cost', 'mana': 4}])
card('Helm of Kaldra', 'leg', types='A', dsl=[{'type': 'static', 'static': 'equip_keyword', 'keyword': 'first strike'},
                                             {'type': 'static', 'static': 'equip_keyword', 'keyword': 'trample'},
                                             {'type': 'static', 'static': 'equip_keyword', 'keyword': 'haste'},
                                             {'type': 'static', 'static': 'equip_cost', 'mana': 2}])
KALDRA = ('Sword of Kaldra', 'Shield of Kaldra', 'Helm of Kaldra')


def kaldra_exile(g, src, victim):
    """Sword of Kaldra: the equipped creature dealt damage to victim: exile it. True if it was exiled"""
    if src.cd is None and not (src.data and src.data.get('kaldra')) and not _attached(g, src, 'Sword of Kaldra'): return False
    if not _attached(g, src, 'Sword of Kaldra'): return False
    if victim not in victim.owner.perms or protected_from(g, victim, ''): return False
    log(f'    Sword of Kaldra exiles {victim.name}', g)
    victim.owner.lost_names[victim.name] += 1
    exile_perm(g, victim)
    return True
CI.kaldra_exile = kaldra_exile


def _kaldra_indestructible(g, m):
    """Shield of Kaldra: the Kaldra equipment are indestructible"""
    return m.cd is not None and m.cd.name in KALDRA and bool(_mine_named(m.owner, 'Shield of Kaldra'))
for _n in KALDRA: CI.SELF_REGEN[_n] = lambda g, m: _kaldra_indestructible(g, m)


@on('Helm of Kaldra', 'options')
def _kaldra_assemble(g, src, p, s, post):
    """{1}: with Helm, Sword and Shield of Kaldra all out, create Kaldra (a legendary 4/4 Avatar) wearing all three"""
    if p is not src.owner or post is None or not can_pay(g, p, 1, ''): return []
    pieces = [next(iter(_mine_named(p, n)), None) for n in KALDRA]
    if None in pieces or any(m.data and m.data.get('kaldra') for m in p.perms if m.cd is None): return []

    def go():
        if not can_pay(g, p, 1, '') or any(x not in p.perms for x in pieces): return False
        pay(g, p, 1, '')
        if not ability_window(g, p, src, 'create Kaldra'): return True
        k = make_tokens(g, p, 1, 4, 4, color='', types=('avatar',), data={'legendary': True, 'kaldra': True})
        if k:
            for e in pieces: e.attached = k[0]
            k[0].data['indestr'] = True
            log(f'  {NAME(p)} creates Kaldra wearing Sword, Shield and Helm', g)
        return True
    return [(9.0, 'Helm of Kaldra: create Kaldra', go)]


for _n, _t in (('Sword of Kaldra', 'equipped creature +5/+5; exiles any creature it deals damage to; equip {4}'),
               ('Shield of Kaldra', 'equipped creature and the Kaldra equipment are indestructible; equip {4}'),
               ('Helm of Kaldra', 'first strike, trample and haste; equip {2}; {1}: with all three Kaldra pieces, '
                                  'create Kaldra (legendary 4/4) wearing them')):
    full(_n, _t)


# ------------------------------------------------------------------ Blackblade Reforged
def _blackblade_pt(g, m):
    k = len(_attached(g, m, 'Blackblade Reforged'))
    return (k * len(m.owner.lands),) * 2 if k else (0, 0)
IC.CREATURE_PT.append(_blackblade_pt)
card('Blackblade Reforged', 'leg', types='A', dsl=[])


@on('Blackblade Reforged', 'etb')
def _blackblade_etb(g, src, p, m):
    if m is src: g.selfpt = True


@on('Blackblade Reforged', 'options')
def _blackblade_equip(g, src, p, s, post):
    """equip a legendary creature {3}, any other {7}"""
    if p is not src.owner or post is None or g.active is not p: return []
    if src.attached is not None and src.attached in p.perms: return []
    t = _best_legend(g, p)
    n = 3
    if t is None:
        cs = [m for m in p.perms if m.creature and not m.phased and not m.noatk]
        t = max(cs, key=lambda m: pval(g, m), default=None); n = 7
    if t is None or not can_pay(g, p, n, ''): return []

    def go():
        if not can_pay(g, p, n, '') or t not in p.perms: return False
        return equip_to(g, p, src, t, n)
    return [(1.5 + 0.25 * len(p.lands) - 0.2 * n, f'equip Blackblade Reforged to {t.name}', go)]
full('Blackblade Reforged', 'equipped creature +1/+1 per land you control; equip a legend {3}, anything else {7}')


# ------------------------------------------------------------------ planeswalkers
def _surveil(g, p, n):
    importlib.import_module('commander_sim.cards.impl.topdeck').scry(g, p, n, 'gy')


def _dakkon_exile(g, p, src):
    t = IC.best_opp_creature(g, p)
    if t is not None: apply_removal(g, p, t, 'exile')


def _dakkon_ult(g, p, src):
    arts = [c for c in p.hand + p.gy if 'A' in c.types and not c.land]
    if not arts: return
    c = max(arts, key=lambda c: (c.cmc, card_worth(g, p, c)))
    (p.hand if c in p.hand else p.gy).remove(c)
    enter(g, p, c)


IC.walker('Dakkon, Shadow Slayer', [
    (1, 'surveil 2', IC.always(1.2), lambda g, p, src: _surveil(g, p, 2)),
    (-3, 'exile a creature', lambda g, p, src: (lambda t: pval(g, t) - 1 if t is not None and pval(g, t) >= 3 else None)(
        IC.best_opp_creature(g, p)), _dakkon_exile),
    (-6, 'an artifact onto the battlefield', lambda g, p, src: 6.0 if any('A' in c.types and c.cmc >= 5 for c in p.hand + p.gy) else None,
     _dakkon_ult),
], status=('Full', 'enters with loyalty equal to your lands; +1 surveil 2, -3 exile the best opposing creature, -6 your '
           'best artifact from hand or graveyard onto the battlefield'))
CI.AS_ENTERS['Dakkon, Shadow Slayer'] = lambda g, p, m: setattr(m, 'loyalty', len(p.lands))


def _mord_draw(g, p, src):
    draw(g, p, 2)
    if p.hand:
        x = min(p.hand, key=lambda c: card_worth(g, p, c))
        p.hand.remove(x); p.library.insert(0, x)


def _mord_dog(g, p, src):
    t = make_tokens(g, p, 1, 0, 0, color='U', types=('dog', 'illusion'), data={'mord_dog': True})


IC.TOKEN_PT.append(lambda g, m: ((2 * len(m.owner.hand),) * 2) if m.data.get('mord_dog') else (0, 0))


def _mord_ult(g, p, src):
    p.hand, p.library = p.library, p.hand
    g.rng.shuffle(p.library)
    p.no_max_hand = True


IC.walker('Mordenkainen', [
    (2, 'draw two, one to the bottom', IC.always(2.5), _mord_draw),
    (-2, 'a Dog Illusion (twice your hand)', lambda g, p, src: 0.6 * len(p.hand) if len(p.hand) >= 3 else None, _mord_dog),
    (-10, 'swap hand and library', lambda g, p, src: 9.0, _mord_ult),
], status=('Full', '+2 draw two then put one on the bottom, -2 a Dog Illusion with power and toughness twice your hand '
           'size, -10 exchange hand and library (no maximum hand size)'))


def _w6_land(g, p, src):
    ls = [c for c in p.gy if c.land]
    if ls: x = ls[0]; p.gy.remove(x); p.hand.append(x)


def _w6_ping(g, p, src):
    t = IC.best_opp_creature(g, p, lambda m: etgh(g, m) <= 1)
    if t is not None: apply_removal(g, p, t, 'dmg1')
    elif g.opps(p): lose_life(g, min(g.opps(p), key=lambda q: q.life), 1, p, kind='burn', damage=True)


IC.walker('Wrenn and Six', [
    (1, 'a land back to hand', lambda g, p, src: 1.5 if any(c.land for c in p.gy) else 0.8, _w6_land),
    (-1, '1 damage', lambda g, p, src: (lambda t: pval(g, t) if t is not None and pval(g, t) >= 1.5 else None)(
        IC.best_opp_creature(g, p, lambda m: etgh(g, m) <= 1)), _w6_ping),
    (-7, 'emblem: retrace', lambda g, p, src: 6.0, lambda g, p, src: setattr(p, 'w6_retrace', True)),
], status=('Approximate', '+1 a land from graveyard to hand, -1 one damage (an X/1 or a player), -7 emblem (retrace '
           'recorded, not used)'))


def _spark_attach(g, p, src):
    t = _best_legend(g, p) or max([m for m in p.perms if m.creature and not m.phased], key=lambda m: epow(g, m), default=None)
    if t is not None: src.attached = t; t.plus += 1


IC.walker('The Aetherspark', [
    (1, 'attach, +1/+1 counter', lambda g, p, src: 2.0 if any(m.creature for m in p.perms) else 0.5, _spark_attach),
    (-5, 'draw two', lambda g, p, src: 3.5, lambda g, p, src: draw(g, p, 2)),
    (-10, 'add ten mana', lambda g, p, src: 5.0 if any(c.cmc >= 8 for c in p.hand) else None,
     lambda g, p, src: setattr(p, 'floatA', p.floatA + 10)),
], status=('Full', '+1 attach to your best creature with a +1/+1 counter, -5 draw two, -10 add ten mana; combat damage '
           'by the equipped creature on your turn adds that much loyalty'))


@on('The Aetherspark', 'combat_damage')
def _spark_loyalty(g, src, p, a, d, dmg):
    if p is src.owner and src.attached is a and src.loyalty is not None: src.loyalty += dmg


def _carth_extra(g, src, p):
    return 1 if p is src.owner else 0
CI.HOOKS.setdefault('Carth the Lion', {})['loyalty_extra'] = _carth_extra


def _carth_look(g, o):
    top = o.library[-7:]
    pw = [c for c in top if 'P' in c.types]
    rest = [c for c in top]
    del o.library[-len(top):]
    if pw:
        c = max(pw, key=lambda c: card_worth(g, o, c)); rest.remove(c); o.hand.append(c)
        log(f'    Carth finds {c.name}', g)
    g.rng.shuffle(rest); o.library[:0] = rest


@on('Carth the Lion', 'etb')
def _carth_etb(g, src, p, m):
    if m is src and trigger_window(g, src.owner, src, 'look at seven for a planeswalker'): _carth_look(g, src.owner)


@on('Carth the Lion', 'dies')
def _carth_pw_dies(g, src, m, cause):
    if m.owner is src.owner and m.cd is not None and 'P' in m.cd.types and trigger_window(g, src.owner, src, 'look at seven'):
        _carth_look(g, src.owner)
card('Carth the Lion', 'leg pow=3 tgh=5', dsl=[])
full('Carth the Lion', 'on entry and when your planeswalker dies, a planeswalker from the top seven to hand; your '
     'loyalty abilities cost an extra [+1]')


# ------------------------------------------------------------------ creatures
def discover(g, o, n, src_name):
    """exile from the top until a nonland card with mana value n or less: cast it free (or to hand); rest to the bottom"""
    seen, hit = [], None
    while o.library:
        x = o.library.pop()
        if not x.land and x.cmc <= n: hit = x; break
        seen.append(x)
    if hit is not None:
        if castable(g, o, hit, 'lib') and not ('ctr' in hit.tags and not g.stack):
            g.last_x = 0
            log(f'    {src_name}: discover {n} casts {hit.name}', g)
            cast_card(g, o, hit, 'lib', {})
        else:
            o.hand.append(hit)
    g.rng.shuffle(seen); o.library[:0] = seen


@on('Caparocti Sunborn', 'attack')
def _caparocti(g, src, p, atk, d):
    if p is not src.owner or src not in atk: return None
    cand = sorted([m for m in p.perms if not m.tapped and not m.phased and m not in atk and m is not src
                   and (m.creature or (m.cd is not None and 'A' in m.cd.types))], key=lambda m: pval(g, m))
    if len(cand) < 2 or not trigger_window(g, p, src, 'tap two: discover 3'): return None
    for m in cand[:2]: m.tapped = True
    discover(g, p, 3, 'Caparocti Sunborn')
    return None
card('Caparocti Sunborn', 'leg pow=4 tgh=4', dsl=[])
full('Caparocti Sunborn', 'attacking: tap your two least valuable untapped artifacts or creatures to discover 3')


def _regen(cost_g, cost_p):
    def fn(g, m):
        p = m.owner
        if E.human_choice(g, p) is None and not can_pay(g, p, cost_g, cost_p): return False
        if not can_pay(g, p, cost_g, cost_p): return False
        pay(g, p, cost_g, cost_p); m.tapped = True
        log(f'    {m.name} regenerates', g)
        return True
    return fn
CI.SELF_REGEN['Cromat'] = _regen(0, 'BG')
CI.SELF_REGEN['Korlash, Heir to Blackblade'] = _regen(1, 'B')


@on('Cromat', 'crew')
def _cromat_combat(g, src, p):
    """before attacking: {U}{R} for flying when the defenders have no fliers to block, {R}{W} pumps with spare mana"""
    if p is not src.owner or src.tapped or src.sick: return
    opp_fly = any(m.fly for q in g.opps(p) for m in q.perms if m.creature and not m.tapped)
    if not src.fly and not opp_fly and can_pay(g, p, 0, 'UR') and total_mana(g, p) >= 4:
        pay(g, p, 0, 'UR'); g.eot_kw.setdefault(id(src), set()).add('flying'); g.dsl_on = True
        log('  Cromat gains flying', g)
    while total_mana(g, p) >= 4 and can_pay(g, p, 0, 'RW'):
        pay(g, p, 0, 'RW'); a, b = g.eot_pt.get(id(src), (0, 0)); g.eot_pt[id(src)] = (a + 1, b + 1); g.dsl_on = True
card('Cromat', 'leg pow=5 tgh=5', dsl=[])
note('Cromat', 'Approximate', '{B}{G} regenerate, {U}{R} flying before an attack the defenders can\'t block in the air, '
     '{R}{W} +1/+1 with spare mana; the blocker-destroy and top-of-library abilities are not used')
card('Korlash, Heir to Blackblade', 'leg pow=0 tgh=0', dsl=[])
IC.SELF_PT['Korlash, Heir to Blackblade'] = lambda g, p, m: ((sum(1 for L in p.lands if 'swamp' in
    importlib.import_module('commander_sim.ais').land_types(L.cd.name)[0]),) * 2)


CI.AS_ENTERS['Korlash, Heir to Blackblade'] = lambda g, p, m: setattr(g, 'selfpt', True)   # before its 0/0 is checked
full('Korlash, Heir to Blackblade', 'power and toughness equal to your Swamps; {1}{B} regenerate; grandeur needs a '
     'second copy (never in a singleton deck)')


@on('Dragonlord Dromoka', 'can_cast')
def _dromoka(g, src, caster, c, zone):
    return not (caster is not src.owner and g.active is src.owner)
card('Dragonlord Dromoka', 'leg pow=5 tgh=7 fly lifelink unc', dsl=[])
full('Dragonlord Dromoka', "can't be countered; flying, lifelink; opponents can't cast spells during your turn")


def _hydra_prio(g, p, c):
    x = total_mana(g, p) - 2
    return 40 + min(30, 5 * x) if x >= 3 else 0
CI.SPELL_PRIO['Genesis Hydra'] = _hydra_prio
card('Genesis Hydra', 'xtutor', types='C', dsl=[])


@on('Genesis Hydra', 'etb')
def _hydra_etb(g, src, p, m):
    """X (the spare mana it was cast with; 0 when cast free): the top X cards, a nonland permanent with mana value X or
    less onto the battlefield, the rest shuffled in; the Hydra gets X +1/+1 counters. The reveal happens as it
    resolves rather than as it is cast (only a counterspell tells the two apart)"""
    if m is not src: return
    o = src.owner
    x, g.last_x = int(getattr(g, 'last_x', 0) or 0), 0
    if x > 0:
        top = o.library[-x:]; del o.library[-x:]
        ok = [y for y in top if not y.land and y.cmc <= x and (y.perm or y.creature)]
        if ok:
            y = max(ok, key=lambda y: (y.cmc, card_worth(g, o, y))); top.remove(y)
            log(f'    Genesis Hydra puts {y.name} onto the battlefield', g)
            enter(g, o, y)
        o.library.extend(top); g.rng.shuffle(o.library)
    if src in o.perms:
        src.plus += x
        if etgh(g, src) <= 0: die(g, src, 'sba')
full('Genesis Hydra', 'X = spare mana: a nonland permanent with mana value X or less from the top X onto the '
     'battlefield, and X +1/+1 counters')


def _lagrella_etb(g, src, p, m):
    if m is not src or src not in src.owner.perms: return
    o = src.owner
    if not trigger_window(g, o, src, 'exile a creature of each opponent', imp=5) or src not in o.perms: return
    IC.oring_exile(g, src, o, lambda x: x.creature, per_opponent=True, creature_only=True)
CI.HOOKS.setdefault('Lagrella, the Magpie', {})['etb'] = _lagrella_etb
CI.HOOKS['Lagrella, the Magpie']['leaves'] = lambda g, m: IC.oring_return(g, m)
card('Lagrella, the Magpie', 'leg pow=2 tgh=3 rem=exile tgt=c', dsl=[])
note('Lagrella, the Magpie', 'Full', "exiles the best creature of each opponent until it leaves (yours aren't exiled, "
     'so the +1/+1 counters clause never applies)')


@on('Mirri, Weatherlight Duelist', 'attack_cap')
def _mirri_cap(g, src, attacker, d):
    return 1 if d is src.owner and src.tapped else None


@on('Mirri, Weatherlight Duelist', 'blocks')
def _mirri_blocks(g, src, p, atk, d, assign):
    """Mirri attacks: each opponent blocks with at most one creature"""
    if p is not src.owner or src not in atk: return
    used = {}
    for a in sorted(list(assign), key=lambda a: -epow(g, a)):
        b = assign.get(a)
        if b is None or b.owner is not d: continue
        if d in used and used[d] is not b: assign[a] = None
        else: used[d] = b
card('Mirri, Weatherlight Duelist', 'leg pow=3 tgh=2 fs', dsl=[{'type': 'static', 'static': 'self_keyword', 'keyword': 'first strike'}])
full('Mirri, Weatherlight Duelist', "first strike; attacking: each opponent blocks with at most one creature; while "
     "tapped, no more than one creature can attack you")


@on('Razia, Boros Archangel', 'options')
def _razia(g, src, p, s, post):
    return []
card('Razia, Boros Archangel', 'leg pow=6 tgh=3 fly vig haste', dsl=[])
note('Razia, Boros Archangel', 'Approximate', 'flying, vigilance, haste; the redirect-3-damage ability is not used')


@on("Sol'kanar the Swamp King", 'cast')
def _solkanar(g, src, caster, c):
    if 'B' in c.pips: gain(src.owner, 1)
card("Sol'kanar the Swamp King", 'leg pow=5 tgh=5 swampwalk', dsl=[])
full("Sol'kanar the Swamp King", 'swampwalk; you gain 1 life whenever any player casts a black spell')

card('Szadek, Lord of Secrets', 'leg pow=5 tgh=5 fly', dsl=[])
full('Szadek, Lord of Secrets', 'flying; combat damage to a player becomes +1/+1 counters and that player mills as many')


@on('Tolsimir Wolfblood', 'options')
def _tolsimir(g, src, p, s, post):
    if p is not src.owner or src.tapped or src.sick or post is None: return []
    if any(m.data and m.data.get('voja') for m in p.perms if m.cd is None): return []

    def go():
        if src.tapped: return False
        src.tapped = True
        if not ability_window(g, p, src, 'create Voja'): return True
        make_tokens(g, p, 1, 2, 2, color='GW', types=('wolf',), data={'legendary': True, 'voja': True})
        return True
    return [(2.5, 'Tolsimir: create Voja', go)]
card('Tolsimir Wolfblood', 'leg pow=3 tgh=4', dsl=[
    {'type': 'static', 'static': 'anthem', 'pow': 1, 'tgh': 1, 'filter': {'type': 'creature', 'controller': 'you', 'other': True, 'color': 'G'}},
    {'type': 'static', 'static': 'anthem', 'pow': 1, 'tgh': 1, 'filter': {'type': 'creature', 'controller': 'you', 'other': True, 'color': 'W'}}])
full('Tolsimir Wolfblood', 'your other green creatures +1/+1 and other white creatures +1/+1; {T}: Voja, a legendary '
     '2/2 Wolf (one at a time)')


@on('Urza, Powerstone Prodigy', 'options')
def _urza_loot(g, src, p, s, post):
    if p is not src.owner or src.tapped or src.sick or post is None or not can_pay(g, p, 1, '') or not p.hand: return []

    def go():
        if src.tapped or not can_pay(g, p, 1, ''): return False
        pay(g, p, 1, ''); src.tapped = True
        if not ability_window(g, p, src, 'draw, then discard'): return True
        draw(g, p, 1)
        if p.hand:
            arts = [c for c in p.hand if 'A' in c.types and card_worth(g, p, c) < 30]
            x = min(arts or p.hand, key=lambda c: card_worth(g, p, c))
            discard_cards(g, p, [x])
            if 'A' in x.types and once_per_turn(g, p, 'urza_stone'): IC.make_artifact_tokens(g, p, 'Powerstone')
        return True
    return [(1.0 if post else 0.4, 'Urza: loot', go)]
card('Urza, Powerstone Prodigy', 'leg pow=1 tgh=3 vig', dsl=[])
full('Urza, Powerstone Prodigy', 'vigilance; {1}, {T}: draw then discard; discarding an artifact makes a Powerstone '
     '(once a turn)')


@on('King Darien XLVIII', 'options')
def _darien(g, src, p, s, post):
    if p is not src.owner or post is None or g.active is not p or not can_pay(g, p, 3, 'GW'): return []

    def go():
        if not can_pay(g, p, 3, 'GW'): return False
        pay(g, p, 3, 'GW')
        if not ability_window(g, p, src, 'a counter and a Soldier'): return True
        src.plus += 1; make_tokens(g, p, 1, 1, 1, color='W', types=('soldier',))
        return True
    return [(1.2 if post else 0.3, 'King Darien: counter and a Soldier', go)]
card('King Darien XLVIII', 'leg pow=2 tgh=3', dsl=[{'type': 'static', 'static': 'anthem', 'pow': 1, 'tgh': 1,
                                                  'filter': {'type': 'creature', 'controller': 'you', 'other': True}}])
note('King Darien XLVIII', 'Approximate', 'other creatures +1/+1; {3}{G}{W} a +1/+1 counter and a 1/1 Soldier with spare '
     'mana; the sacrifice-for-protection ability is not used')


@on('Sisters of Stone Death', 'blocks')
def _sisters(g, src, p, atk, d, assign):
    """{B}{G}: exile a creature blocking the Sisters"""
    if p is not src.owner or src not in atk: return
    b = assign.get(src)
    if b is not None and b in d.perms and can_pay(g, p, 0, 'BG') and not untargetable(g, b):
        pay(g, p, 0, 'BG'); log(f'  Sisters of Stone Death exile {b.name}', g)
        d.lost_names[b.name] += 1; exile_perm(g, b); assign[src] = None
        src.data = dict(src.data or {}, sisters=(src.data or {}).get('sisters', []) + [(b.cd, d)] if b.cd is not None else [])
card('Sisters of Stone Death', 'leg pow=7 tgh=5', dsl=[])
note('Sisters of Stone Death', 'Approximate', '{B}{G} exiles a creature that blocks them; the lure and the {2}{B} '
     'reanimation of exiled creatures are not used')



# ------------------------------------------------------------------ spells
@IC.spell("Cartographer's Survey", prio=lambda g, p, c: 46 if len(p.lands) <= 6 else 20, tags='', types='S',
          status=('Full', 'up to two lands from the top seven onto the battlefield tapped; the rest to the bottom'))
def _survey(g, p, c, ctx):
    top = p.library[-7:]; del p.library[-7:]
    lands = sorted([x for x in top if x.land], key=lambda x: -len(x.tags.get('c', '')))[:2]
    for x in lands:
        top.remove(x); p.lands.append(Land(x, True)); landfall(g, p)
    g.rng.shuffle(top); p.library[:0] = top


@IC.spell('Experimental Augury', prio=36, tags='', types='I',
          status=('Full', 'the best of the top three to hand, the rest to the bottom; proliferate'))
def _augury(g, p, c, ctx):
    top = p.library[-3:]; del p.library[-3:]
    if top:
        x = max(top, key=lambda y: card_worth(g, p, y)); top.remove(x); p.hand.append(x)
    p.library[:0] = top
    importlib.import_module('commander_sim.cards.impl.mine').proliferate_all(g, p)


def _despair_one(g, p, q, kind):
    pool = [m for m in q.perms if not m.phased and (m.creature if kind == 'C' else (m.cd is not None and kind in m.cd.types))]
    if pool:
        die(g, min(pool, key=lambda m: pval(g, m)), 'sac')
    else:
        lose_life(g, q, 2, p); draw(g, p, 1)


@IC.spell('Invoke Despair', prio=lambda g, p, c: 50 if g.opps(p) else 0, tags='', types='S',
          status=('Full', 'the most threatening opponent sacrifices a creature, an enchantment and a planeswalker '
                          '(their least valuable); for each they can\'t, they lose 2 and you draw'))
def _invoke(g, p, c, ctx):
    q = ctx.get('target') or (max(g.opps(p), key=lambda o: threat(g, p, o)) if g.opps(p) else None)
    if q is None or not q.alive: return
    for k in ('C', 'E', 'P'): _despair_one(g, p, q, k)
    check_state(g)


@on("Kaervek's Purge", 'hand_options')
def _purge(g, c, p, s, post):
    """{X}{B}{R}: destroy target creature with mana value X; it deals its power to its controller"""
    if post is None or g.active is not p or c not in p.hand: return []
    tg = [m for m in legal_targets(g, p, 'destroy', 'c', spell=c) if m.cd is not None and can_pay(g, p, m.cd.cmc, 'BR')]
    if not tg: return []
    t = max(tg, key=lambda m: pval(g, m) + 0.2 * epow(g, m))
    if pval(g, t) < 3: return []

    def go():
        x = t.cd.cmc
        if c not in p.hand or not can_pay(g, p, x, 'BR') or t not in t.owner.perms: return False
        pay(g, p, x, 'BR'); p.hand.remove(c); p.spells_this_turn += 1; on_cast(g, p, c)
        ok = counter_window(g, p, c, 4, {})
        p.gy.append(c)
        if ok and t in t.owner.perms and not untargetable(g, t):
            q, power = t.owner, epow(g, t)
            apply_removal(g, p, t, 'destroy', c)
            if t not in q.perms and power: lose_life(g, q, power, p, kind='burn', damage=True)
        return True
    return [(pval(g, t) * 0.5 - 0.8, f"Kaervek's Purge on {t.name}", go)]
card("Kaervek's Purge", '', types='S', dsl=[])
full("Kaervek's Purge", 'X = the target\'s mana value: destroys it and deals its power to its controller')


@on('Profane Tutor', 'hand_options')
def _profane(g, c, p, s, post):
    """suspend 2 for {1}{B}: two upkeeps later, a free Demonic Tutor"""
    if post is None or g.active is not p or c not in p.hand or not can_pay(g, p, 1, 'B'): return []

    def go():
        if c not in p.hand or not can_pay(g, p, 1, 'B'): return False
        pay(g, p, 1, 'B'); p.hand.remove(c); p.exile.append(c)
        p.suspended = getattr(p, 'suspended', []) + [[c, 2]]
        log(f'  {NAME(p)} suspends Profane Tutor', g)
        return True
    return [(2.2, 'suspend Profane Tutor', go)]


def suspend_upkeep(g, p):
    """the beginning of p's upkeep: a time counter off each suspended card; at zero it's cast free"""
    for e in list(getattr(p, 'suspended', []) or []):
        c = e[0]
        if c not in p.exile: p.suspended.remove(e); continue
        e[1] -= 1
        if e[1] > 0: continue
        p.suspended.remove(e); p.exile.remove(c)
        log(f'  {NAME(p)} casts {c.name} from suspend', g)
        cast_card(g, p, c, 'lib', {})
CI.suspend_upkeep = suspend_upkeep


@IC.spell('Profane Tutor', tags='tut=any', types='S',
          status=('Full', 'suspend 2 for {1}{B}, then cast free: search for any card'))
def _profane_resolve(g, p, c, ctx):
    tutor(g, p, 'any')
CI.SPELL_PRIO['Profane Tutor'] = 0                    # never hard-cast (no mana cost): suspended from hand

card('Dissipate', 'ctr=any ctrexile', types='I', dsl=[])
full('Dissipate', 'counter target spell; it is exiled instead of going to the graveyard')
card('Desertion', 'ctr=any', types='I', dsl=[])
full('Desertion', 'counter target spell; a countered artifact or creature spell enters under your control')


# ------------------------------------------------------------------ Mirari, Memory Jar, Court of Ardenvale
@on('Mirari', 'cast')
def _mirari(g, src, caster, c):
    """whenever you cast an instant or sorcery: pay {3} to copy it (the AI copies spells worth a second resolution)"""
    o = src.owner
    if caster is not o or not (c.instant or c.sorcery) or not can_pay(g, o, 3, ''): return
    t = c.tags
    worth = any(k in t for k in ('draw', 'rem', 'tut', 'drawcre')) or c.name in (
        'Fact or Fiction', 'Invoke Despair', 'Demonic Tutor', 'Cartographer\'s Survey', 'Experimental Augury')
    if not worth or 'ctr' in t: return
    hc = human(g, o)
    if hc is not None and not hc.yes_no(g, o, f'Mirari: pay {{3}} to copy {c.name}?'): return
    if not trigger_window(g, o, src, f'pay {{3}}: copy {c.name}'): return
    pay(g, o, 3, '')
    copy_spell(g, o, c)
card('Mirari', 'leg', types='A', dsl=[])
CI.SPELL_PRIO['Mirari'] = lambda g, p, c: 44 if sum(1 for x in p.hand if x.instant or x.sorcery) >= 2 else 26
full('Mirari', 'whenever you cast an instant or sorcery, pay {3} to copy it (draw, removal and tutors)')


@on('Memory Jar', 'options')
def _jar(g, src, p, s, post):
    """{T}, sacrifice: everyone sets their hand aside and draws seven; at the end step they discard and get the old
    hands back. Used with a small hand and mana left to spend the seven"""
    if p is not src.owner or post is not False or g.active is not p or len(p.hand) > 2 or total_mana(g, p) < 3: return []

    def go():
        if src not in p.perms: return False
        leave(g, src); p.gy.append(src.cd)
        if not ability_window(g, p, src.cd, 'wheel for seven'): return True
        due = []
        for q in g.players:
            if not q.alive: continue
            held = list(q.hand); q.hand.clear(); q.exile.extend(held)
            due.append((q, held))
            draw(g, q, 7)
        g.jar_due = (getattr(g, 'jar_due', None) or []) + due
        log(f'  {NAME(p)} cracks Memory Jar', g)
        return True
    return [(3.0 + 0.4 * (7 - len(p.hand)), 'Memory Jar', go)]


def jar_end(g):
    due, g.jar_due = g.jar_due, []
    for q, held in due:
        if not q.alive: continue
        discard_cards(g, q, list(q.hand))
        for c in held:
            if c in q.exile: q.exile.remove(c); q.hand.append(c)
CI.jar_end = jar_end
card('Memory Jar', '', types='A', dsl=[])
CI.SPELL_PRIO['Memory Jar'] = 38
full('Memory Jar', '{T}, sacrifice: every player sets their hand aside and draws seven; at the end step they discard '
     'and take the old hands back (cracked with a small hand and mana to use the seven)')


@on('Court of Ardenvale', 'etb')
def _court_etb(g, src, p, m):
    if m is src and trigger_window(g, src.owner, src, 'become the monarch'): CI.become_monarch(g, src.owner)


@on('Court of Ardenvale', 'upkeep')
def _court(g, src, p):
    o = src.owner
    if p is not o: return
    cs = [c for c in o.gy if (c.perm or c.creature) and not c.land and c.cmc <= 3]
    if not cs or not trigger_window(g, o, src, 'return a permanent card'): return
    c = max(cs, key=lambda c: card_worth(g, o, c))
    if c not in o.gy: return
    o.gy.remove(c)
    if getattr(g, 'monarch', None) is o:
        enter(g, o, c); log(f'    Court of Ardenvale returns {c.name} to the battlefield', g)
    else:
        o.hand.append(c); log(f'    Court of Ardenvale returns {c.name} to hand', g)
card('Court of Ardenvale', '', types='E', dsl=[])
CI.SPELL_PRIO['Court of Ardenvale'] = 50
full('Court of Ardenvale', 'you become the monarch; each upkeep, a permanent card with mana value 3 or less from your '
     'graveyard to hand, or onto the battlefield while you are the monarch')


# ------------------------------------------------------------------ mana
card('Fyndhorn Elder', 'pow=1 tgh=1 dork=G', dsl=[])
CI.DYN_MANA['Fyndhorn Elder'] = lambda g, p, m: 2
full('Fyndhorn Elder', '{T}: {G}{G}')
card("Sisay's Ring", 'rock=2:C', types='A', dsl=[])
full("Sisay's Ring", '{T}: {C}{C}')
