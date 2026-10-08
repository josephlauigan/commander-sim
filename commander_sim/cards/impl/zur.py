"""Card rules first written for the Zur deck (removed 2026-10-07), kept for the cards other lists still run (mostly
your Y'shtola deck, which also uses the protection and wipe responses here).

- Auras that lock a permanent down while they stay attached: Arrest and Prison Sentence (can't attack or block, no
  activated abilities), Luminous Bonds and Bound in Silence (can't attack or block), Encrust (doesn't untap, no
  activated abilities). Cast, they target as they enter (etb_removal); put onto the battlefield by Zur, they don't
  target, so hexproof and ward don't stop them (protection does). The 'kasmina' kind (loses all abilities, base 1/1)
  is still here, though no list runs Kasmina's Transmutation now.
- Zur's attack trigger for your Y'shtola deck's 99 (what to fetch: cards/impl/yshtola.py). The pool's Zur deck uses
  t5.py's version.
- Other cards that need code: Bastion Protector, The Eternal Wanderer, Prayer of Binding, Rootborn Defenses,
  Disenchant, and Azorius Guildmage (no list runs it; the stack tests use it).
- Protection and wipe responses (Clever Concealment, Rootborn Defenses, Restoration Angel), used by Y'shtola's AI.
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


# Azorius Guildmage is in no list now; it stays because the stack tests use it as the card that counters an activated
# ability (the engine's ABILITY_ANSWERS and partials.py share that code with Tishana's Tidebinder).
@on('Azorius Guildmage', 'options')
def _guildmage(g, src, p, s, post):
    """{2}{W}: tap target creature (before your attack: the creature that would block your best attacker)"""
    if p is not src.owner or post is not False or g.active is not p or not can_pay(g, p, 2, 'W'): return []
    t = guildmage_target(g, p)
    if t is None: return []

    def go():
        if src not in p.perms or t not in t.owner.perms or t.tapped or not can_pay(g, p, 2, 'W'): return False
        pay(g, p, 2, 'W')
        log(f'  {NAME(p)} activates Azorius Guildmage: tap {t.name}', g)
        if ability_window(g, p, src, f'tap {t.name}', target=t): t.tapped = True
        return True
    return [(1.0 + 0.5 * pval(g, t), f'Azorius Guildmage: tap {t.name}', go)]


def guildmage_target(g, p):
    """an untapped opposing creature that could block your best attacker (your commander first) and win the fight
    (also Galadriel's tappers)"""
    mine = [m for m in p.perms if m.creature and not m.tapped and not m.phased and (not m.sick or m.is_cmd)]
    if not mine: return None
    a = max(mine, key=lambda m: (m.is_cmd * 5 + epow(g, m)))
    from commander_sim import ais
    blockers = [b for q in g.opps(p) for b in q.perms if b.creature and not b.tapped and not b.phased
                and not untargetable(g, b) and ais.can_block(g, b, a) and epow(g, b) >= etgh(g, a) - 0]
    return max(blockers, key=lambda b: epow(g, b), default=None)
card('Azorius Guildmage', 'wizard pow=2 tgh=2', dsl=[])
full('Azorius Guildmage', "{2}{W}: taps the creature that would block your attacker, before combat; {2}{U}: counters "
     "an opponent's important activated ability on the stack")


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
                log(f'  {NAME(p)} uses The Eternal Wanderer +1', g)
                if ability_window(g, p, src, '+1', target=t) and t is not None and t in t.owner.perms: wanderer_exile(g, p, t)
            elif kind == 'zero':
                log(f'  {NAME(p)} uses The Eternal Wanderer 0', g)
                if not ability_window(g, p, src, '0'): return True
                n = make_tokens(g, p, 1, 2, color='W', types=('samurai',))
                for x in n: x.data = dict(x.data or {}, kws=('double strike',))
                log(f'  {NAME(p)} uses The Eternal Wanderer 0: a 2/2 double-strike Samurai', g)
            else:
                src.loyalty -= 4
                log(f'  {NAME(p)} uses The Eternal Wanderer -4', g)
                dead = src.loyalty <= 0
                if dead: leave(g, src); to_zone_card(g, src, 'gy')
                if ability_window(g, p, src.cd if dead else src, '-4', imp=8): wanderer_ult(g, p)
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
    """at the beginning of p's end step: Eternal Wanderer exiles come home"""
    keep = []
    for kind, owner, cd, perm in getattr(g, 'zur_due', []):
        if kind == 'wanderer':
            if owner is not p: keep.append((kind, owner, cd, perm)); continue
            if cd in owner.exile and owner.alive:
                owner.exile.remove(cd); enter(g, owner, cd)
                log(f'    {cd.name} returns to the battlefield (The Eternal Wanderer)', g)
    g.zur_due = keep


CI.zur_end_step = zur_end_step


# ================================================================== spells and the rest
@on('Prayer of Binding', 'etb')
def _prayer(g, src, p, m):
    if m is not src: return
    o = src.owner
    if not trigger_window(g, o, src, 'exile a permanent; gain 2 life', imp=5): return
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


card('Disenchant', 'rem=destroy tgt=ae', dsl=[])
full('Disenchant', 'destroy target artifact or enchantment')
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


# ================================================================== Zur's attack trigger (your Y'shtola deck's 99)
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
        g.attach_to = None                                     # the AI's host pick, or the person chooses
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
    """how much an enchantment from Zur is worth now (Y'shtola's values: cards/impl/yshtola.py)"""
    return CI.yshtola_fetch_value(g, p, c, zur)


CI.zur_fetch = zur_fetch


# ================================================================== the AI: protection and wipe responses (Y'shtola's)
def zur_protect(g, owner, m, kind, actor, spell):
    """removal at a key creature: phase it out with its Auras (Clever Concealment), make it indestructible (Rootborn
    Defenses), or blink it with Restoration Angel (not a creature with Auras on it, which would lose them)"""
    from commander_sim import ais
    if pval(g, m) < 4 or not m.creature: return False
    auras = IC.auras_on(g, m) if getattr(g, 'auras', None) else []
    for c in list(owner.hand):
        tp = c.tags.get('prot')
        if tp == 'phase' and can_pay(g, owner, c.generic, c.pips, 'convoke' in c.tags):      # Clever Concealment
            if not ais.pay_card(g, owner, c): return False
            m.phased = True
            for a in auras: a.phased = True
            log(f'    {NAME(owner)} casts {c.name}: {m.name} phases out', g)
            return True
        if tp == 'indes' and (kind == 'destroy' or kind.startswith('dmg')) and can_pay(g, owner, c.generic, c.pips):
            if not ais.pay_card(g, owner, c): return False                                   # Rootborn Defenses
            rootborn(g, owner)
            return True
    if auras or kind not in ('destroy', 'exile', 'bounce', 'tuck') and not kind.startswith('dmg'): return False
    for c in list(owner.hand):
        if c.name == 'Restoration Angel' and can_pay(g, owner, c.generic, c.pips):
            if not ais.pay_card(g, owner, c): return False
            if c in owner.gy: owner.gy.remove(c)
            g.resto_target = m                          # its enters trigger blinks the creature under attack
            try:
                enter(g, owner, c)
            finally:
                g.resto_target = None
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
            if not ais.pay_card(g, q, c): return None
            mine = [m for m in q.perms if m.creature]
            for m in q.perms:
                if m.creature or (m.attached is not None and m.attached in mine): m.phased = True
            log(f'    {NAME(q)} casts {c.name}: their creatures phase out', g)
            return 'all'
    if kind in ('destroy', 'dmg13', 'austere', 'nib'):
        for c in list(q.hand):
            if c.tags.get('prot') == 'indes' and can_pay(g, q, c.generic, c.pips):
                if not ais.pay_card(g, q, c): return None
                rootborn(g, q)
                return 'indes'
    return None
