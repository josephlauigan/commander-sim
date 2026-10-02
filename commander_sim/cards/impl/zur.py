"""Zur the Enchanter (your Esper Auras deck, key `zur`).

- Auras that lock a permanent down while they stay attached: Arrest and Prison Sentence (can't attack or block, no
  activated abilities), Luminous Bonds and Bound in Silence (can't attack or block), Encrust (doesn't untap, no
  activated abilities), Kasmina's Transmutation (loses all abilities, base 1/1), and Phyrexian Boon (-1/-2 on a
  non-black creature, +2/+1 on a black one). Cast, they target as they enter (etb_removal); put onto the battlefield
  by Zur, they don't target, so hexproof and ward don't stop them (protection does).
- Zur's attack trigger for this deck: the enchantment the board calls for, and where its Aura goes.
- The deck's other cards that need code: Demonic Embrace, Gift of Immortality, Duelist's Heritage, Bastion
  Protector, Ministrant of Obligation, Recruitment Officer, Azorius Guildmage, The Eternal Wanderer, Prayer of
  Binding, Momentary Blink, Rootborn Defenses, Divine Verdict, Destroy Evil, Disenchant, Vivid Meadow.
- The AI: cast priorities, protection and wipe responses.
"""
from commander_sim.engine import *
from commander_sim import engine as E
from commander_sim.cards import cardimpl as CI
from commander_sim.cards.cardimpl import on
from commander_sim.cards.pool_cards import card, note
from commander_sim.cards.impl import common as IC


def full(name, text): note(name, 'Full', text)


def human(g, p):
    return E.human_choice(g, p)


# ================================================================== Auras that lock a permanent down
LOCKS = {}          # Aura name -> the locks it puts on what it enchants: pacify (can't attack or block), noact (no
                    # activated abilities), frozen (doesn't untap), neuter (loses all abilities; base 1/1)
LOCK_KINDS = {'arrest': ('pacify', 'noact'), 'pacify': ('pacify',), 'encrust': ('frozen', 'noact'),
              'kasmina': ('neuter',)}


def _kasmina_pt(g, p, a, m):
    """base power and toughness 1/1 (counters and other bonuses still apply)"""
    return 1 - (m.pow or 0), 1 - (m.tgh or 0)


def lock_aura(name, kind, tgt, text, extra=''):
    IC.AURA[name] = dict(pow=0, tgh=0, kws=frozenset(), bonus=_kasmina_pt if kind == 'kasmina' else None,
                         umbra=False, prot='', target='opp', on_etb=None, host_ok=None, back=None)
    LOCKS[name] = frozenset(LOCK_KINDS[kind])
    card(name, f'rem={kind} tgt={tgt} etb {extra}'.strip(), types='E', dsl=[])
    full(name, text)


lock_aura('Arrest', 'arrest', 'c', "Aura: the creature can't attack or block, and its activated abilities can't be "
          "activated (mana abilities too)")
lock_aura('Prison Sentence', 'arrest', 'c', "Aura: scry 2; the creature can't attack or block, and its activated "
          "abilities can't be activated")
lock_aura('Luminous Bonds', 'pacify', 'c', "Aura: the creature can't attack or block")
lock_aura('Bound in Silence', 'pacify', 'c', "Aura: the creature can't attack or block")
lock_aura('Encrust', 'encrust', 'ac', "Aura on an artifact or creature: it doesn't untap during its controller's "
          "untap step, and its activated abilities can't be activated")
lock_aura("Kasmina's Transmutation", 'kasmina', 'c', 'Aura: the creature loses all abilities and has base power and '
          'toughness 1/1 (back to normal if the Aura leaves)')


def locked(g, m, kind):
    """is permanent m locked down by an Aura (kind: pacify, noact, frozen, neuter)?"""
    for a in getattr(g, 'auras', None) or ():
        if a.attached is m and a.cd is not None and kind in LOCKS.get(a.cd.name, ()) and not a.phased \
                and a in a.owner.perms:
            return True
    return False


def lock_factor(g, m):
    """how much of m's value is left under the Auras locking it"""
    f = 1.0
    for a in getattr(g, 'auras', None) or ():
        if a.attached is not m or a.cd is None or a.phased or a not in a.owner.perms: continue
        ks = LOCKS.get(a.cd.name)
        if not ks: continue
        if 'neuter' in ks: f = min(f, 0.15)
        elif 'pacify' in ks: f = min(f, 0.3 if 'noact' in ks else 0.4)
        elif 'frozen' in ks: f = min(f, 0.35)
    return f


def lock_attach(g, a, host):
    """Aura a (already on the battlefield) enchants host and locks it"""
    if host is None or host not in host.owner.perms or a not in a.owner.perms: return False
    a.attached = host
    if getattr(g, 'auras', None) is None: g.auras = []
    if a not in g.auras: g.auras.append(a)
    g.dsl_on = True
    g.bf_ver = getattr(g, 'bf_ver', 0) + 1; g.hook_cache = None
    log(f'    {a.cd.name} enchants {host.name} ({NAME(host.owner)})', g)
    if 'neuter' in LOCKS.get(a.cd.name, ()):
        if a.data is None: a.data = {}
        a.data['was'] = (host.neutered, host.fly, host.dt, host.vig, host.life)
        host.neutered = True; host.fly = host.dt = host.vig = host.life = False
    if a.cd.name == 'Prison Sentence':
        from commander_sim.cards.impl import topdeck
        topdeck.scry(g, a.owner, 2)
    return True


def _unlock(g, a, host):
    """Kasmina's Transmutation left: the creature gets its abilities back"""
    was = (a.data or {}).get('was')
    if was is not None and host is not None and host in host.owner.perms:
        host.neutered, host.fly, host.dt, host.vig, host.life = was


IC.ON_DETACH["Kasmina's Transmutation"] = _unlock


def apply_lock(g, actor, m, kind):
    """apply_removal's lock kinds (arrest, pacify, encrust, kasmina): the entering Aura (g.rem_src) enchants m"""
    a = getattr(g, 'rem_src', None)
    if a is None or a.cd is None: return
    lock_attach(g, a, m)


def lock_host(g, p, name, targeted=True, exclude=()):
    """the best opposing permanent for lock Aura `name` (targeted: cast, so hexproof and ward stop it), or None"""
    kind = next(k for k, v in LOCK_KINDS.items() if frozenset(v) == LOCKS[name])
    ok_art = name == 'Encrust'
    best, bv = None, 0.0
    for q in g.opps(p):
        for m in q.perms:
            if m.phased or m in exclude: continue
            if not (m.creature or (ok_art and m.cd is not None and 'A' in m.cd.types)): continue
            if protected_from(g, m, 'W' if name != 'Encrust' and name != "Kasmina's Transmutation" else 'U'): continue
            if targeted and untargetable(g, m): continue
            if any(locked(g, m, k) for k in LOCKS[name]): continue
            v = lock_worth(g, p, m, kind)
            if v > bv: best, bv = m, v
    return best if bv >= 2.0 else None


def lock_worth(g, p, m, kind):
    """what locking m this way takes away from its controller"""
    v = pval(g, m) * lock_factor(g, m)
    acts = activated(m)
    if kind == 'pacify':
        if not m.creature: return 0
        return v * (0.45 + 0.1 * min(4, epow(g, m))) * (0.5 if acts else 1.0)
    if kind == 'arrest':
        return v * (0.55 + 0.08 * min(4, epow(g, m)) + (0.4 if acts else 0))
    if kind == 'encrust':
        return v * (0.5 + (0.5 if acts else 0)) if (m.creature or acts) else 0
    if kind == 'kasmina':
        if not m.creature: return 0
        return v * (0.9 if (m.cd is not None and (CI.live(m.cd.name) or m.cd.dsl or acts)) else 0.6)
    return v


def activated(m):
    """does m have activated abilities worth stopping (mana abilities included)?"""
    if m.cd is None: return False
    t = m.cd.tags
    if 'rock' in t or 'dork' in t or 'clamp' in t: return True
    if m.cd.name in CI.HOOKS and 'options' in CI.HOOKS[m.cd.name]: return True
    if m.cd.name in ('Lightning Greaves', 'Swiftfoot Boots', 'Whispersilk Cloak'): return True
    return any(a.get('type') in ('activated', 'loyalty') for a in (m.cd.dsl or []))


CI.LOCKS = LOCKS
CI.locked = locked
CI.lock_factor = lock_factor
CI.apply_lock = apply_lock


# ------------------------------------------------------------------ Phyrexian Boon: either side of the table
def _boon_bonus(g, p, a, m):
    return (2, 1) if 'B' in colors_of(m) else (-1, -2)


def _boon_host(g, p, spec, a):
    """kill an opposing non-black creature with toughness 2 or less if one is worth it, else your best black creature"""
    if human(g, p) is not None: return None
    kill = [m for q in g.opps(p) for m in q.perms if m.creature and not m.phased and 'B' not in colors_of(m)
            and etgh(g, m) <= 2 and not protected_from(g, m, 'B')
            and not (getattr(g, 'last_cast_etb', False) and untargetable(g, m))]
    best = max(kill, key=lambda m: pval(g, m), default=None)
    if best is not None and pval(g, best) >= 2.5: return best
    mine = [m for m in p.perms if m.creature and not m.phased and 'B' in colors_of(m)]
    return max(mine, key=lambda m: (m.is_cmd * 3 + pval(g, m)), default=None)


def _boon_etb(g, p, a, host):
    if host.creature and etgh(g, host) <= 0: die(g, host, 'sba')       # -1/-2 on an X/2: dies


IC.aura('Phyrexian Boon', bonus=_boon_bonus, host_pick=_boon_host, target='either', on_etb=_boon_etb,
        status=('Full', '+2/+1 on a black creature (Zur), -1/-2 on any other: killing an opposing X/2 or less'))


# ================================================================== the deck's own Auras
IC.aura('Demonic Embrace', 3, 1, ('flying',), status=('Full', '+3/+1 and flying; cast from your graveyard by paying 3 life '
                                                      'and discarding a card on top of its cost'))


@on('Demonic Embrace', 'gy_options')
def _embrace_gy(g, c, p, s, post):
    if post is None or c not in p.gy or p.life <= 8 or not p.hand or not can_pay(g, p, 1, 'BB'): return []
    host = zur_perm(p)
    if host is None or untargetable_by_you(g, host): return []
    worst = min(p.hand, key=lambda x: card_worth(g, p, x))

    def go():
        if c not in p.gy or worst not in p.hand or not can_pay(g, p, 1, 'BB') or host not in p.perms: return False
        pay(g, p, 1, 'BB'); lose_life(g, p, 3, p); discard_cards(g, p, [worst])
        p.gy.remove(c); p.spells_this_turn += 1; on_cast(g, p, c)
        g.attach_to = host
        try:
            enter(g, p, c, was_cast=True)
        finally:
            g.attach_to = None
        log(f'  {NAME(p)} casts Demonic Embrace from the graveyard (3 life, discards {worst.name})', g)
        return True
    return [(2.0 + 0.3 * epow(g, host), 'Demonic Embrace from the graveyard', go)]


IC.aura('Gift of Immortality', status=('Full', 'when the creature dies it returns to the battlefield, and Gift returns '
                                       'attached to it at the next end step (Zur goes to the graveyard to come back)'))


def gift_returns(g, m):
    """Gift of Immortality: called as m (which had Gift attached) would go to its owner's graveyard. True if m came back"""
    due = [x for x in getattr(g, 'gift_dying', []) if x[0] is m]
    if not due: return False
    g.gift_dying = [x for x in g.gift_dying if x[0] is not m]
    owner = m.orig
    n = enter(g, owner, m.cd, orig=owner)
    n.is_cmd = m.cd is owner.cmd
    log(f'    Gift of Immortality returns {m.cd.name} to the battlefield', g)
    for _, gift_owner, gift_cd in due:
        g.zur_due = getattr(g, 'zur_due', []) + [('gift', gift_owner, gift_cd, n)]
    return True


def _gift_host_died(g, a, host):
    if a.cd.name == 'Gift of Immortality' and host.creature and not host.token:
        g.gift_dying = getattr(g, 'gift_dying', []) + [(host, a.owner, a.cd)]


IC.ON_HOST_DIES['Gift of Immortality'] = _gift_host_died


# ------------------------------------------------------------------ Duelist's Heritage
@on("Duelist's Heritage", 'attack')
def _heritage(g, src, p, atk, d):
    """whenever one or more creatures attack, you may have target attacking creature gain double strike"""
    o = src.owner
    if p is not o or not atk: return
    cands = [m for m in atk if m in o.perms and not untargetable_by_you(g, m)]
    if not cands: return
    hc = human(g, o)
    if hc is not None:
        from commander_sim.play import legal
        k = hc.choose(g, o, 'target', "Duelist's Heritage: give an attacking creature double strike?",
                      [legal.describe_target(g, o, m) for m in cands], cancel='no one')
        if k is None: return
        t = cands[k]
    else:
        t = max(cands, key=lambda m: (epow(g, m) * (3 if m.is_cmd else 1) + (2 if m.fly else 0)))
    g.eot_kw.setdefault(id(t), set()).add('double strike')
    log(f"    Duelist's Heritage gives {t.name} double strike", g)
card("Duelist's Heritage", '', types='E', dsl=[])
full("Duelist's Heritage", 'whenever you attack, your best attacker (Zur first) gains double strike')


def untargetable_by_you(g, m):
    """shroud (Lightning Greaves) stops even your own spells and abilities"""
    return any(e.attached is m and e.cd is not None and e.cd.name == 'Lightning Greaves' and e in m.owner.perms
               for e in m.owner.perms) or E.DSLMOD is not None and E.DSLMOD.has_kw(g, m, 'shroud')


# ================================================================== creatures
card('Bastion Protector', 'human pow=3 tgh=3', dsl=[
    {'type': 'static', 'static': 'anthem', 'filter': {'type': 'creature', 'controller': 'you', 'subtype': 'commander'},
     'pow': 2, 'tgh': 2},
    {'type': 'static', 'static': 'keyword', 'keyword': 'indestructible',
     'filter': {'type': 'creature', 'controller': 'you', 'subtype': 'commander'}}])
full('Bastion Protector', 'commander creatures you control get +2/+2 and have indestructible')


@on('Ministrant of Obligation', 'self_dies')
def _ministrant(g, m, cause):
    make_tokens(g, m.owner, 2, 1, fly=True, color='WB', types=('spirit',))
    log('    Ministrant of Obligation: two 1/1 flying Spirits (afterlife 2)', g)
card('Ministrant of Obligation', 'human pow=2 tgh=1', dsl=[])
full('Ministrant of Obligation', 'afterlife 2: two 1/1 white and black flying Spirits when it dies')


@on('Recruitment Officer', 'options')
def _officer(g, src, p, s, post):
    """{3}{W}: look at the top four, take a creature with mana value 3 or less; the rest go to the bottom"""
    if p is not src.owner or post is False or not can_pay(g, p, 3, 'W') or len(p.library) < 1: return []
    if post is None and g.active is p: return []                   # the end-of-turn window: opponents' turns only

    def go():
        if src not in p.perms or not can_pay(g, p, 3, 'W'): return False
        pay(g, p, 3, 'W')
        officer_dig(g, p)
        return True
    return [(1.4, 'Recruitment Officer: dig for a creature', go)]


def officer_dig(g, p):
    top = [p.library.pop() for _ in range(min(4, len(p.library)))]
    cs = [c for c in top if c.creature and c.cmc <= 3]
    hc = human(g, p)
    pick = None
    if hc is not None and cs:
        k = hc.choose(g, p, 'choose', f'Recruitment Officer: take a creature card? (top four: {", ".join(c.name for c in top)})',
                      [c.name for c in cs], cancel='take nothing')
        pick = cs[k] if k is not None else None
    elif cs:
        pick = max(cs, key=lambda c: card_worth(g, p, c))
    rest = [c for c in top if c is not pick]
    g.rng.shuffle(rest)
    for c in rest: p.library.insert(0, c)
    if pick is not None: p.hand.append(pick); p.seen_names.add(pick.name)
    log(f'  {NAME(p)} activates Recruitment Officer' + (f': takes {pick.name}' if pick is not None and hc is not None
                                                          else ': takes a creature' if pick is not None else ''), g)
card('Recruitment Officer', 'human pow=2 tgh=1', dsl=[])
full('Recruitment Officer', '{3}{W}: look at the top four, put a creature card with mana value 3 or less into your '
     'hand, the rest on the bottom in a random order (used in your second main phase or at an opponent\'s end step)')


@on('Azorius Guildmage', 'options')
def _guildmage(g, src, p, s, post):
    """{2}{W}: tap target creature (before your attack: the creature that would block Zur or your best attacker)"""
    if p is not src.owner or post is not False or g.active is not p or not can_pay(g, p, 2, 'W'): return []
    t = guildmage_target(g, p)
    if t is None: return []

    def go():
        if src not in p.perms or t not in t.owner.perms or t.tapped or not can_pay(g, p, 2, 'W'): return False
        pay(g, p, 2, 'W'); t.tapped = True
        log(f'  {NAME(p)} activates Azorius Guildmage: taps {t.name}', g)
        return True
    return [(1.0 + 0.5 * pval(g, t), f'Azorius Guildmage: tap {t.name}', go)]


def guildmage_target(g, p):
    """an untapped opposing creature that could block your best attacker (Zur first) and win the fight"""
    mine = [m for m in p.perms if m.creature and not m.tapped and not m.phased and (not m.sick or m.is_cmd)]
    if not mine: return None
    a = max(mine, key=lambda m: (m.is_cmd * 5 + epow(g, m)))
    from commander_sim import ais
    blockers = [b for q in g.opps(p) for b in q.perms if b.creature and not b.tapped and not b.phased
                and not untargetable(g, b) and ais.can_block(g, b, a) and epow(g, b) >= etgh(g, a) - 0]
    return max(blockers, key=lambda b: epow(g, b), default=None)
card('Azorius Guildmage', 'wizard pow=2 tgh=2', dsl=[])
note('Azorius Guildmage', 'Partial', "{2}{W}: taps the creature that would block your attacker, before combat; "
     "{2}{U} (counter target activated ability) is not modeled: the engine has no window to answer abilities")


# ================================================================== The Eternal Wanderer
@on('The Eternal Wanderer', 'options')
def _wanderer(g, src, p, s, post):
    """+1: exile up to one target artifact or creature until its owner's next end step. 0: a 2/2 double-strike
    Samurai. -4: each player keeps one creature and sacrifices the rest"""
    if p is not src.owner or post is None or src.phased or src.loyalty is None: return []
    if src.data and src.data.get('act') == turn_stamp(g): return []
    out = []
    t = wanderer_target(g, p)

    def act(kind):
        def go():
            if src not in p.perms or (src.data and src.data.get('act') == turn_stamp(g)): return False
            src.data = dict(src.data or {}, act=turn_stamp(g))
            if kind == 'plus':
                src.loyalty += 1
                if t is not None and t in t.owner.perms: wanderer_exile(g, p, t)
                else: log(f'  {NAME(p)} uses The Eternal Wanderer +1', g)
            elif kind == 'zero':
                n = make_tokens(g, p, 1, 2, color='W', types=('samurai',))
                for x in n: x.data = dict(x.data or {}, kws=('double strike',))
                log(f'  {NAME(p)} uses The Eternal Wanderer 0: a 2/2 double-strike Samurai', g)
            else:
                src.loyalty -= 4
                log(f'  {NAME(p)} uses The Eternal Wanderer -4', g)
                wanderer_ult(g, p)
                if src.loyalty <= 0: leave(g, src); to_zone_card(g, src, 'gy')
            return True
        return go
    if t is not None: out.append((1.5 + 0.6 * pval(g, t), f'The Eternal Wanderer +1 (exile {t.name})', act('plus')))
    out.append((1.2, 'The Eternal Wanderer 0 (Samurai)', act('zero')))
    if src.loyalty >= 4:
        gain_v = ult_value(g, p)
        if gain_v >= 6: out.append((gain_v / 2, 'The Eternal Wanderer -4', act('ult')))
    return out


def wanderer_target(g, p):
    cands = [m for q in g.opps(p) for m in q.perms if not m.phased and not untargetable(g, m) and not m.token
             and (m.creature or (m.cd is not None and 'A' in m.cd.types))]
    best = max(cands, key=lambda m: pval(g, m), default=None)
    return best if best is not None and pval(g, best) >= 2.5 else None


def wanderer_exile(g, p, m):
    owner, cd = m.orig, m.phys or m.cd
    log(f'  {NAME(p)} uses The Eternal Wanderer +1: exiles {m.name} until {NAME(owner)}\'s next end step', g)
    apply_removal(g, p, m, 'exile')
    if m not in m.owner.perms and not m.token and cd in owner.exile and cd is not owner.cmd:
        g.zur_due = getattr(g, 'zur_due', []) + [('wanderer', owner, cd, None)]


def keep_one(g, q, me):
    cr = [m for m in q.perms if m.creature and not m.phased]
    if not cr: return None
    if q is me: return max(cr, key=lambda m: (m.is_cmd * 5 + pval(g, m)))
    return max(cr, key=lambda m: pval(g, m))


def ult_value(g, p):
    """what -4 gains: opponents' creatures lost minus yours"""
    v = 0.0
    for q in g.players:
        if not q.alive: continue
        k = keep_one(g, q, p)
        lost = sum(pval(g, m) for m in q.perms if m.creature and not m.phased and m is not k)
        v += lost if q is not p else -1.5 * lost
    return v


def wanderer_ult(g, p):
    for q in g.players:
        if not q.alive: continue
        k = keep_one(g, q, p)
        for m in [m for m in q.perms if m.creature and not m.phased and m is not k]:
            die(g, m, 'sac')


card('The Eternal Wanderer', 'leg', types='P', dsl=[])
full('The Eternal Wanderer', '+1: exiles the best opposing artifact or creature until its owner\'s next end step; 0: a '
     '2/2 double-strike Samurai; -4: each player keeps one creature (you keep Zur); only one creature can attack it')


def zur_end_step(g, p):
    """at the beginning of p's end step: Eternal Wanderer exiles come home; Gift of Immortality reattaches (the next
    end step after the creature returned)"""
    keep = []
    for kind, owner, cd, perm in getattr(g, 'zur_due', []):
        if kind == 'wanderer':
            if owner is not p: keep.append((kind, owner, cd, perm)); continue
            if cd in owner.exile and owner.alive:
                owner.exile.remove(cd); enter(g, owner, cd)
                log(f'    {cd.name} returns to the battlefield (The Eternal Wanderer)', g)
        elif kind == 'gift':
            if cd in owner.gy and perm in perm.owner.perms:
                owner.gy.remove(cd)
                g.attach_to = perm
                try:
                    enter(g, owner, cd)
                finally:
                    g.attach_to = None
                log(f'    Gift of Immortality returns attached to {perm.name}', g)
    g.zur_due = keep


CI.zur_end_step = zur_end_step
CI.gift_returns = gift_returns


# ================================================================== spells and the rest
@on('Prayer of Binding', 'etb')
def _prayer(g, src, p, m):
    if m is not src: return
    o = src.owner
    hc = human(g, o)
    if hc is not None:
        from commander_sim.play import legal
        cands = [x for q in g.opps(o) for x in q.perms if not x.phased and not untargetable(g, x)
                 and not protected_from(g, x, 'W')]
        k = hc.choose(g, o, 'target', 'Prayer of Binding: exile which nonland permanent an opponent controls?',
                      [legal.describe_target(g, o, x) for x in cands], cancel='no target') if cands else None
        if k is not None: IC.oring_exile(g, src, o, lambda x, t=cands[k]: x is t)
    else:
        IC.oring_exile(g, src, o, lambda x: not x.token or x.creature)
    gain(o, 2)


@on('Prayer of Binding', 'leaves')
def _prayer_leaves(g, m):
    IC.oring_return(g, m)
card('Prayer of Binding', 'flash', types='E', dsl=[])
full('Prayer of Binding', 'flash; exiles the best opposing nonland permanent until it leaves the battlefield; you gain 2')


card('Destroy Evil', 'rem=destroy tgt=ce evil', dsl=[])
full('Destroy Evil', 'destroy target creature with toughness 4 or greater, or target enchantment')
card('Disenchant', 'rem=destroy tgt=ae', dsl=[])
full('Disenchant', 'destroy target artifact or enchantment')
card('Divine Verdict', 'rem=destroy tgt=c verdict', dsl=[])
full('Divine Verdict', 'destroy target attacking or blocking creature: cast when you are attacked, on the biggest attacker')
card('Rootborn Defenses', 'prot=indes populate', dsl=[])
full('Rootborn Defenses', 'populate, and your creatures gain indestructible until end of turn (cast against a '
     'destroy wipe or removal on Zur)')


def populate(g, p):
    toks = [m for m in p.perms if m.token and m.creature and not m.phased]
    if not toks: return
    t = max(toks, key=lambda m: (epow(g, m), m.fly))
    n = make_tokens(g, p, 1, t.pow, t.tgh, fly=t.fly, color=t.colors, types=tuple(t.ttypes) or None)
    for x in n:
        x.life, x.dt = t.life, t.dt
        if t.data: x.data = dict(t.data)


def rootborn(g, p):
    """Rootborn Defenses resolves: populate, then indestructible until end of turn"""
    populate(g, p)
    for m in p.perms:
        if m.creature: g.eot_kw.setdefault(id(m), set()).add('indestructible')
    log(f'  {NAME(p)} casts Rootborn Defenses: creatures gain indestructible', g)


# ------------------------------------------------------------------ Momentary Blink
@on('Momentary Blink', 'hand_options')
def _blink_eot(g, c, p, s, post):
    """at an opponent's end step: blink your best enters-the-battlefield creature (Accursed Marauder, Skyclave)"""
    if post is not None or g.active is p or c not in p.hand or not can_pay(g, p, 1, 'W'): return []
    return _blink_option(g, p, c, 1, 'W', 'hand')


@on('Momentary Blink', 'gy_options')
def _blink_fb(g, c, p, s, post):
    if post is not None or g.active is p or c not in p.gy or not can_pay(g, p, 3, 'U'): return []
    return _blink_option(g, p, c, 3, 'U', 'gy')


def _blink_option(g, p, c, gen, pips, zone):
    from commander_sim.cards.impl import t2
    cands = [m for m in p.perms if m.creature and not m.token and not m.is_cmd and t2.blink_value(g, p, m) >= 3]
    t = max(cands, key=lambda m: t2.blink_value(g, p, m), default=None)
    if t is None: return []

    def go():
        if t not in p.perms or not can_pay(g, p, gen, pips): return False
        if zone == 'hand':
            if c not in p.hand: return False
            p.hand.remove(c)
        else:
            if c not in p.gy: return False
            p.gy.remove(c)
        pay(g, p, gen, pips)
        p.spells_this_turn += 1; on_cast(g, p, c)
        (p.gy if zone == 'hand' else p.exile).append(c)
        t2.blink(g, p, t)
        log(f'  {NAME(p)} casts Momentary Blink{" (flashback)" if zone == "gy" else ""}: blinks {t.cd.name}', g)
        return True
    return [(t2.blink_value(g, p, t) - 1.5 - (1.0 if zone == 'gy' else 0), f'Momentary Blink on {t.name}', go)]
card('Momentary Blink', 'fb=3U', dsl=[])
full('Momentary Blink', 'blinks your best enters-the-battlefield creature at an opponent\'s end step (flashback {3}{U}), '
     'or a creature of yours in answer to removal')


# ------------------------------------------------------------------ Vivid Meadow (like Marchesa's Vivid Creek and Marsh)
def _charge(L):
    return (L.data or {}).get('ctr', 2)


def _meadow_tap(g, p, L, used):
    if any(x != 'W' for x in E.TAP_COLS): L.data = dict(L.data or {}, ctr=max(0, _charge(L) - 1))


CI.LAND_COLS['Vivid Meadow'] = lambda g, p, L: p.ident if _charge(L) > 0 else 'W'
CI.ON_TAP['Vivid Meadow'] = _meadow_tap
full('Vivid Meadow', 'enters tapped with two charge counters; {T}: {W}, or remove a counter for any colour')


# ================================================================== Zur's attack trigger (this deck)
def zur_perm(p):
    return next((m for m in p.perms if m.cd is not None and m.cd is p.cmd and not m.phased), None)


def zur_fetch(g, src, p):
    """Zur attacks: search for an enchantment card with mana value 3 or less and put it onto the battlefield"""
    cs = [c for c in searchable(g, p) if 'E' in c.types and c.cmc <= 3]
    if not cs: return
    hc = human(g, p)
    if hc is not None:
        got = hc.search(g, p, lambda c: 'E' in c.types and c.cmc <= 3, 1,
                        'Zur attacks: search for an enchantment with mana value 3 or less (it enters the battlefield)')
        if not got: return
        c = got[0]
    else:
        scored = [(fetch_value(g, p, c, src), c) for c in cs]
        v, c = max(scored, key=lambda x: x[0])
        if v <= 0: return
        p.library.remove(c); g.rng.shuffle(p.library)
    p.milestone.setdefault('zurfetch', p.turns)
    put_enchantment(g, p, c, src)
    log(f'    Zur fetches {c.name}', g)


def put_enchantment(g, p, c, zur):
    """an enchantment put onto the battlefield (not cast): an Aura goes where it's needed, without targeting"""
    if c.name in LOCKS:
        host = lock_host(g, p, c.name, targeted=False) if human(g, p) is None else human_lock_host(g, p, c.name)
        g.aura_put = True
        try:
            a = enter(g, p, c)
        finally:
            g.aura_put = False
        if host is None or not lock_attach(g, a, host):
            leave(g, a); p.gy.append(c)
        return
    if 'aura' in c.subtypes and c.name in IC.AURA:
        g.attach_to = zur if c.name != 'Phyrexian Boon' and human(g, p) is None else None   # the person chooses
        g.aura_put = True
        try:
            enter(g, p, c)
        finally:
            g.attach_to = None; g.aura_put = False
        return
    enter(g, p, c)


def human_lock_host(g, p, name):
    from commander_sim.play import legal
    kind = next(k for k, v in LOCK_KINDS.items() if frozenset(v) == LOCKS[name])
    cands = [m for q in g.opps(p) for m in q.perms if not m.phased
             and (m.creature or (name == 'Encrust' and m.cd is not None and 'A' in m.cd.types))]
    cands += [m for m in p.perms if not m.phased and (m.creature or (name == 'Encrust' and m.cd is not None and 'A' in m.cd.types))]
    if not cands: return None
    k = human(g, p).choose(g, p, 'target', f'{name}: enchant which permanent? (it enters without targeting)',
                           [legal.describe_target(g, p, m) for m in cands], cancel=None)
    return cands[k] if k is not None else None


def fetch_value(g, p, c, zur):
    """how much an enchantment from Zur is worth now"""
    n = c.name
    have = {m.cd.name for m in p.perms if m.cd is not None and not m.phased}
    if n in have and n not in IC.AURA and n not in LOCKS: return 0                 # a second copy of a global (none)
    turn = p.turns
    if n == 'Rhystic Study': return max(2.0, 8.0 - 0.4 * turn)
    if n == 'Necropotence': return (8.5 - 0.3 * turn) if p.life >= 20 and 'Necropotence' not in have else 1.0
    if n == 'Mystic Remora': return max(0.0, 5.0 - 1.2 * (turn - 1))
    if n in LOCKS:
        kind = next(k for k, v in LOCK_KINDS.items() if frozenset(v) == LOCKS[n])
        t = lock_host(g, p, n, targeted=False)
        return lock_worth(g, p, t, kind) if t is not None else 0
    if n == 'Phyrexian Boon':
        kill = [m for q in g.opps(p) for m in q.perms if m.creature and 'B' not in colors_of(m) and etgh(g, m) <= 2
                and not protected_from(g, m, 'B')]
        k = max((pval(g, m) for m in kill), default=0)
        return max(1.2 * k if k >= 2.5 else 0, voltron_value(g, p, zur, 2))
    if zur is None: return 0.5
    if n == 'Ethereal Armor': return voltron_value(g, p, zur, IC.n_ench(p) + 1) + 0.5
    if n == 'All That Glitters':
        k = sum(1 for m in p.perms if m.cd is not None and ('A' in m.cd.types or 'E' in m.cd.types) and not m.phased) + 1
        return voltron_value(g, p, zur, k)
    if n == 'Demonic Embrace': return voltron_value(g, p, zur, 3)
    if n == "Duelist's Heritage":
        if n in have: return 0
        now = epow(g, zur)
        return 0.5 + 0.3 * now + clock_gain(hits_to_kill(g, p, now, 1), hits_to_kill(g, p, now, 2))
    if n == 'Gift of Immortality':
        if any(a.cd.name == n for a in IC.auras_on(g, zur)) if getattr(g, 'auras', None) else False: return 0
        return 2.0 + 1.5 * (len(IC.auras_on(g, zur)) if getattr(g, 'auras', None) else 0)
    if n == 'Spirit Link':
        if any(a.cd.name == n for a in IC.auras_on(g, zur)) if getattr(g, 'auras', None) else False: return 0
        return 1.0 + (2.5 if p.life < 20 or 'Necropotence' in have else 0) + 0.2 * epow(g, zur)
    return 0.5


def zur_mult(p, zur):
    from commander_sim import ais
    return 2 if (ais.double_strike(p, zur) or any(m.cd is not None and m.cd.name == "Duelist's Heritage"
                                                  and not m.phased for m in p.perms)) else 1


def hits_to_kill(g, p, power, mult):
    """the fewest hits of Zur that kill an opponent (21 commander damage, or their life)"""
    return min((-(-min(q.life, 21 - q.cmd_dmg.get(p.key, 0)) // max(1, power * mult)) for q in g.opps(p)), default=99)


def clock_gain(before, after):
    """hits taken off the kill (capped), and a bonus when the next hit becomes lethal. Measured: weighting power this
    way (over engines and locks) wins about 1.5 points more per tier than favouring the engines"""
    return min(12.0, before - after) + (6.0 if after <= 1 < before else 0)


def voltron_value(g, p, zur, plus):
    """power on Zur is worth the hits it takes off the clock (double strike counts)"""
    if zur is None: return 0
    mult, now = zur_mult(p, zur), epow(g, zur)
    return 1.0 + 0.5 * plus + clock_gain(hits_to_kill(g, p, now, mult), hits_to_kill(g, p, now + plus, mult))


CI.zur_fetch = zur_fetch


# ================================================================== the AI: priorities, protection
def zur_prio(g, p, c):
    t = c.tags; n = c.name
    zur = zur_perm(p)
    if c is p.cmd: return 84 if p.turns >= 2 else 60
    if 'rock' in t: return 82 if p.turns <= 5 else 30
    if 'rhystic' in t: return 78
    if 'necro' in t: return 74 if p.life >= 20 else 20
    if 'remora' in t: return 66 if p.turns <= 4 else 8
    if n == 'Esper Sentinel': return 62 if p.turns <= 4 else 30
    if n in LOCKS:
        kind = next(k for k, v in LOCK_KINDS.items() if frozenset(v) == LOCKS[n])
        tg = lock_host(g, p, n, targeted=True)
        return min(80, int(42 + 7 * lock_worth(g, p, tg, kind))) if tg is not None else 0
    if n == 'Phyrexian Boon':
        h = _boon_host(g, p, IC.AURA[n], None)
        if h is None: return 0
        return 55 if h.owner is not p else (40 if zur is not None and h is zur else 20)
    if 'aura' in t or n in IC.AURA:                                   # the Voltron Auras: on Zur, once it's safe
        if zur is None: return 0
        if untargetable_by_you(g, zur) and not greaves_movable(p, zur): return 0
        v = fetch_value(g, p, c, zur)
        return min(70, int(30 + 5 * v))
    if n == "Duelist's Heritage": return 48 if zur is not None else 20
    if n == 'Prayer of Binding':
        best = max((pval(g, m) for q in g.opps(p) for m in q.perms if not m.phased and not untargetable(g, m)), default=0)
        return min(75, int(40 + 6 * best)) if best >= 3 else 0
    if t.get('prot') == 'boots': return 60 if zur is not None else 25
    if n == 'Bastion Protector': return 55 if zur is not None else 35
    if n == 'Notion Thief': return 50
    if n == 'Skyclave Apparition':
        tg = E.legal_targets(g, p, 'exile', 'nl', True, spell=c)
        return 45 + min(30, int(6 * max((pval(g, m) for m in tg), default=0))) if tg else 25
    if n == 'Accursed Marauder':
        nontok = [m for q in g.opps(p) for m in q.perms if m.creature and not m.token and not m.phased]
        return 54 if nontok else 15
    if n == 'Aven Mindcensor': return 30                              # held for flash
    if n == 'Restoration Angel': return 25                            # held for flash
    if n == 'The Eternal Wanderer': return 58
    if n == 'Recruitment Officer': return 52 if p.turns <= 3 else 40
    if n == 'Azorius Guildmage': return 36
    if n == 'Ministrant of Obligation': return 38
    if n == 'Aven Fisher': return 34
    if 'tokx' in t: return 0                                          # Secure the Wastes: at an end step
    if n == 'Triplicate Spirits': return 46
    if 'draw' in t and (c.instant or c.sorcery): return 46
    if c.creature: return 40
    return 0


def greaves_movable(p, zur):
    return any(m.creature and not m.phased and m is not zur for m in p.perms)


def move_greaves_off(g, p, zur):
    """Lightning Greaves' shroud would stop an Aura you cast on Zur: equip it to another creature first ({0})"""
    gr = [e for e in p.perms if e.cd is not None and e.cd.name == 'Lightning Greaves' and e.attached is zur]
    others = [m for m in p.perms if m.creature and not m.phased and m is not zur]
    if not gr or not others: return False
    gr[0].attached = max(others, key=lambda m: pval(g, m))
    log(f'  {NAME(p)} equips Lightning Greaves to {gr[0].attached.name} (to enchant Zur)', g)
    return True


def own_aura_host(g, p, spec, a):
    """where the deck's own Auras go: Zur (moving Lightning Greaves off it first when the Aura is cast)"""
    zur = zur_perm(p)
    if zur is None: return None
    if getattr(g, 'last_cast_etb', False) and not getattr(g, 'aura_put', False) and untargetable_by_you(g, zur):
        if not move_greaves_off(g, p, zur): return None
    return zur


IC.ZUR_HOST = own_aura_host


def zur_protect(g, owner, m, kind, actor, spell):
    """removal at Zur (or another key creature): phase it out with its Auras (Clever Concealment), make it
    indestructible (Rootborn Defenses), or blink it (Momentary Blink, Restoration Angel: not Zur with Auras on it,
    which would lose them)"""
    from commander_sim import ais
    if pval(g, m) < 4 or not m.creature: return False
    auras = IC.auras_on(g, m) if getattr(g, 'auras', None) else []
    for c in list(owner.hand):
        tp = c.tags.get('prot')
        if tp == 'phase' and can_pay(g, owner, c.generic, c.pips, 'convoke' in c.tags):      # Clever Concealment
            ais.pay_card(g, owner, c)
            m.phased = True
            for a in auras: a.phased = True
            log(f'    {NAME(owner)} casts {c.name}: {m.name} phases out', g)
            return True
        if tp == 'indes' and (kind == 'destroy' or kind.startswith('dmg')) and can_pay(g, owner, c.generic, c.pips):
            ais.pay_card(g, owner, c); rootborn(g, owner)                                     # Rootborn Defenses
            return True
    if auras or kind not in ('destroy', 'exile', 'bounce', 'tuck') and not kind.startswith('dmg'): return False
    for c in list(owner.hand):
        if c.name in ('Momentary Blink', 'Restoration Angel') and can_pay(g, owner, c.generic, c.pips):
            from commander_sim.cards.impl import t2
            ais.pay_card(g, owner, c)
            if c.name == 'Restoration Angel':
                if c in owner.gy: owner.gy.remove(c)
                g.resto_target = m                      # its enters trigger blinks the creature under attack
                try:
                    enter(g, owner, c)
                finally:
                    g.resto_target = None
            else:
                t2.blink(g, owner, m)
            log(f'    {NAME(owner)} casts {c.name}: blinks {m.cd.name if m.cd else m.name}', g)
            return True
    return False


def zur_wipe_response(g, q, kind):
    """a wipe: phase out your creatures and the Auras on them (Clever Concealment), or make them indestructible
    (Rootborn Defenses)"""
    from commander_sim import ais
    loss = sum(pval(g, m) for m in q.perms if m.creature or kind in ('rift', 'rebuke'))
    if loss < 6: return None
    for c in list(q.hand):
        if c.tags.get('prot') == 'phase' and can_pay(g, q, c.generic, c.pips, 'convoke' in c.tags):
            ais.pay_card(g, q, c)
            mine = [m for m in q.perms if m.creature]
            for m in q.perms:
                if m.creature or (m.attached is not None and m.attached in mine): m.phased = True
            log(f'    {NAME(q)} casts {c.name}: their creatures phase out', g)
            return 'all'
    if kind in ('destroy', 'dmg13', 'austere', 'nib'):
        for c in list(q.hand):
            if c.tags.get('prot') == 'indes' and can_pay(g, q, c.generic, c.pips):
                ais.pay_card(g, q, c); rootborn(g, q)
                return 'indes'
    return None


CI.zur_prio = zur_prio
CI.zur_protect = zur_protect
CI.zur_wipe_response = zur_wipe_response


# ------------------------------------------------------------------ Divine Verdict: when you're attacked
def verdict_response(g, d, p, atk):
    """d (the AI) is attacked by p's creatures: Divine Verdict destroys the biggest attacker worth it"""
    if E.human_choice(g, d) is not None: return
    vs = [c for c in d.hand if 'verdict' in c.tags]
    if not vs or not can_pay(g, d, 3, 'W'): return
    cands = [m for m in atk if m in m.owner.perms and not untargetable(g, m) and not indestructible(g, m)
             and not protected_from(g, m, 'W')]
    if not cands: return
    t = max(cands, key=lambda m: (epow(g, m) * (2 if m.is_cmd else 1) + pval(g, m)))
    if epow(g, t) < 3 and pval(g, t) < 4: return
    from commander_sim import ais
    c = vs[0]
    ais.pay_card(g, d, c)
    log(f'  {NAME(d)} casts Divine Verdict on attacking {t.name}', g)
    apply_removal(g, d, t, 'destroy', c)


CI.verdict_response = verdict_response
