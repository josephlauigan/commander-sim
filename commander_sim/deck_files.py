"""Where your deck files live: one folder per owner under decklists/ (JD, Avery, Other).

Deck files are looked for in $SIM_DECKS (flat, or with the owner folders), then decklists/<owner>/ in the repository,
then /mnt/project (Claude's project copy)."""
import os
from commander_sim import ROOT

# deck key -> (owner folder in decklists/, file)
FILES = {'seph': ('JD', 'sephiroth-phyrexian-reanimator.md'), 'veyran': ('JD', 'veyran-izzet-spellslinger.md'),
         'sauron': ('JD', 'sauron-grixis-amass.md'), 'yshtola': ('JD', 'yshtola-esper-drain.md'),
         'galadriel': ('Avery', 'galadriel-bant-rebels.md'), 'alela': ('Avery', 'alela-esper-faeries.md'),
         'jodah': ('JD', 'jodah-wubrg-legends.md'),
         'marchesa': ('Other', 'marchesa-grixis-recursion.md'), 'zur': ('Other', 'zur-esper-auras.md')}
OWNERS = tuple(dict.fromkeys(o for o, _ in FILES.values()))


def repo_path(key):
    owner, f = FILES[key]
    return os.path.join(ROOT, 'decklists', owner, f)


def deck_path(key):
    """the deck file for key: $SIM_DECKS (flat or by owner), then decklists/<owner>/, then /mnt/project"""
    owner, f = FILES[key]
    env = os.environ.get('SIM_DECKS', '')
    cands = ([os.path.join(env, f), os.path.join(env, owner, f)] if env else []) + \
        [repo_path(key), os.path.join('/mnt/project', f)]
    return next((c for c in cands if os.path.exists(c)), repo_path(key))


def is_deck_file(path):
    """is path one of the repository's own deck files (not a copy elsewhere)?"""
    return any(os.path.abspath(path) == repo_path(k) for k in FILES)
