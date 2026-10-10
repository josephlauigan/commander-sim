"""ONE-TIME extraction (October 2026, the move to Rust): write the audit tables that lived in
commander_sim/pool_audit.py to data/audit.toml, which `cardgen audit` reads. After this ran, the file is the source:
edit it, not the Python. Do not run it again once the file has been edited by hand.

    python3 rust/tools/extract_audit_data.py            (from the repository root)
"""
import json, os, sys, tomllib

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, ROOT)
from commander_sim import pool_audit as A  # noqa: E402


def s(x): return json.dumps(x, ensure_ascii=False)
def lst(xs): return '[' + ', '.join(s(x) for x in xs) + ']'


L = ['# Data for `cardgen audit` (the tables of the former commander_sim/pool_audit.py; edit here).',
     '#',
     '# unsupported_keywords  Scryfall keywords (lower case) the engine does not implement: a card with one is at best',
     '#                       Partial unless a [card_notes] entry or an override note says otherwise. Remove a keyword',
     '#                       when the engine implements it.',
     '# stat_tags             tags that only describe the body; any other tag counts as a role the AI acts on',
     "# deck_only_tags        hand tags whose behaviour lives in the four main decks' AI: an outside deck holding the card",
     '#                       never uses it. [pool_ok] lists the tag values the generic path handles.',
     '# [card_notes."Name"]   status + note for a card; wins over every automatic rule',
     '# [mechanics]           engine-level mechanics: status (Full / Full-auto / Approximate / Partial / Unmodeled) + note',
     '# [[engines.<deck key>]] the key engines of an outside deck (from its "## Sim modeling notes"): label, key cards,',
     '#                       mechanics (keys of [mechanics])',
     '',
     f'unsupported_keywords = {lst(sorted(A.UNSUPPORTED_KW))}',
     f'stat_tags = {lst(sorted(A.STAT_TAGS))}',
     f'deck_only_tags = {lst(sorted(A.DECK_ONLY_TAGS))}',
     '', '[pool_ok]']
for k, v in A.POOL_OK.items(): L.append(f'{k} = {lst(v)}')
L += ['', '[card_notes]']
assert not A.CARD_NOTES
L += ['', '[mechanics]']
for k, (st, note) in A.MECH.items(): L.append(f'{k} = {{ status = {s(st)}, note = {s(note)} }}')
for deck, engines in A.ENGINES.items():
    for label, cards, mechs in engines:
        L += ['', f'[[engines.{deck}]]', f'label = {s(label)}', f'cards = {lst(cards)}', f'mechs = {lst(mechs)}']
text = '\n'.join(L) + '\n'
path = os.path.join(ROOT, 'data', 'audit.toml')
open(path, 'w', encoding='utf-8').write(text)
t = tomllib.loads(text)
assert set(t['unsupported_keywords']) == A.UNSUPPORTED_KW and set(t['stat_tags']) == A.STAT_TAGS
assert {k: (v['status'], v['note']) for k, v in t['mechanics'].items()} == A.MECH
assert {d: [(e['label'], e['cards'], e['mechs']) for e in es] for d, es in t['engines'].items()} == \
    {d: [(a, b, c) for a, b, c in es] for d, es in A.ENGINES.items()}
print(f'{len(A.MECH)} mechanics, {sum(len(v) for v in A.ENGINES.values())} engines -> {path}')
