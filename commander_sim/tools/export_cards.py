"""Write every card definition and deck list the simulator plays to JSON, for the Rust engine (rust/).

    python3 -m commander_sim.tools.export_cards [--out DIR]       (default: data/)

data/cards.json: every card in engine.DB once all decks and pools are loaded (your decks, the five tiers), sorted by
name. Each card's fields are the CD's own (engine.CD): name, types, cost, tags (string values, or true for a bare
tag), the compiled abilities (dsl), Scryfall keywords and subtypes, protection, ward, colour identity, and where
the definition came from. `derived` repeats what the CD works out from those (land, creature, cmc, ...), so the
Rust loader can check it computes the same. `phyrexian` is the colours of its Phyrexian mana symbols and
`land_etb_fx` a land's "when this land enters" scry / gain life, both read from the card cache. `python_hooks` lists the events a card's Python implementation
handles (cards/impl/*.py), and `spell_prio` whether it has a hand-written cast priority: the work list for porting
cards by hand.

data/decks.json: your decks (key, display name, commander, the 99) and the pool decks (tier as well).

The export is deterministic: the same code and card cache give the same files.
"""
import argparse, json, os, sys
from commander_sim import DATA


def load_everything():
    from commander_sim import engine as E, pools, ais
    from commander_sim.decks import DECKS
    from commander_sim.cards import sources
    pools.register()
    names = set(ais.CMDS.values())
    for cards in DECKS.values(): names |= set(cards)
    for d in pools.load_pool(): names |= set(d.cards) | {d.commander}
    sources.ensure_cards(sorted(names), verbose=False)
    return E, pools, DECKS


def card_record(E, cd):
    CI = E.CI
    hooks = sorted(CI.HOOKS.get(cd.name, {})) if CI is not None else []
    return {
        'name': cd.name,
        'types': cd.types,
        'generic': cd.generic,
        'pips': cd.pips,
        'tags': cd.tags,
        'pow': cd.pow,
        'tgh': cd.tgh,
        'bomb': cd.bomb,
        'dsl': cd.dsl,
        'start_loyalty': cd.start_loyalty,
        'kws': sorted(cd.kws),
        'subtypes': sorted(cd.subtypes),
        'protfrom': cd.protfrom,
        'ward': cd.ward,
        'identity': getattr(cd, 'identity', None),
        'game_changer': cd.game_changer,
        'source': cd.source,
        'unparsed': cd.unparsed,
        'derived': {'cmc': cd.cmc, 'land': cd.land, 'creature': cd.creature, 'instant': cd.instant,
                    'sorcery': cd.sorcery, 'perm': cd.perm},
        'phyrexian': E.phyrexian(cd),
        'land_etb_fx': [list(x) for x in E.land_etb_fx(cd)] if cd.land else [],
        'python_hooks': hooks,
        'spell_prio': CI is not None and cd.name in CI.SPELL_PRIO,
    }


class _Seat:
    """just enough of a Player for engine.NAME"""
    def __init__(self, key): self.key = key


def export(out_dir):
    E, pools, DECKS = load_everything()
    from commander_sim import ais
    cards = [card_record(E, E.DB[n]) for n in sorted(E.DB)]
    decks = {
        'mine': [{'key': k, 'name': E.NAME(_Seat(k)), 'commander': ais.CMDS[k], 'cards': sorted(v)}
                 for k, v in sorted(DECKS.items())],
        'pool': [{'tier': d.tier, 'key': d.key, 'name': pools.short_name(d), 'commander': d.commander,
                  'cards': sorted(d.cards)} for d in pools.load_pool()],
    }
    os.makedirs(out_dir, exist_ok=True)
    paths = []
    for fname, body in (('cards.json', {'version': 1, 'cards': cards}), ('decks.json', {'version': 1, **decks})):
        path = os.path.join(out_dir, fname)
        with open(path, 'w') as fh:
            json.dump(body, fh, indent=1, sort_keys=False, ensure_ascii=False)
            fh.write('\n')
        paths.append(path)
    return paths, len(cards), len(decks['mine']) + len(decks['pool'])


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('--out', default=DATA, help='output folder (default: data/)')
    a = ap.parse_args(argv)
    paths, n_cards, n_decks = export(a.out)
    print(f'{n_cards} cards, {n_decks} decks -> ' + ', '.join(paths))


if __name__ == '__main__':
    main(sys.argv[1:])
