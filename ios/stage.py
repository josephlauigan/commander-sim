"""Copy what the iPad app needs into ios/src/ before a Briefcase build (the copies are not in git):

    python3 ios/stage.py              # the code, the deck lists, the card cache and the card images you have
    python3 ios/stage.py --images     # first download the images of every card in every deck (once)
    python3 ios/stage.py --no-images  # a smaller app without images (they download on the iPad when online)

  ios/src/commander_sim/                         the shared core (engine, AI, cards, practice server)
  ios/src/commander_ipad/resources/decklists/    your decks (JD/, Avery/) and the opponent tiers (pool/t*/)
  ios/src/commander_ipad/resources/data/         the card cache, the card images, and bundle.json (this build's stamp)

The app copies resources/data into its own writable folder when bundle.json changes (commander_ipad/bootstrap.py).
Without --images the app has the images of the cards already downloaded on this computer; the rest are drawn from
their names when the iPad is offline, and downloaded when it is online.
"""
import argparse
import datetime as dt
import json
import os
import shutil
import subprocess
import sys

IOS = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(IOS)
sys.path.insert(0, REPO)
SKIP = shutil.ignore_patterns('__pycache__', '*.pyc', '*.tmp')


def all_names():
    """every card (and commander) in your decks and in every opponent tier"""
    from commander_sim import poolmode, pools
    from commander_sim.play import images
    keys = list(poolmode.MINE) + [d.key for t in pools.TIERS for d in pools.load_pool(t)]
    return images.game_names([poolmode.seat_spec(k) for k in keys])


def fetch_images():
    from commander_sim.play import images
    names = all_names()
    print(f'Card images for {len(names)} cards (and the tokens they make) ...')
    last = [-1]

    def progress(done, total):
        pct = 100 * done // max(total, 1)
        if pct // 10 != last[0]: last[0] = pct // 10; print(f'  {done}/{total} downloads')
    images.prepare(names, progress)


def copy_tree(src, dst, ignore=SKIP):
    if os.path.exists(dst): shutil.rmtree(dst)
    shutil.copytree(src, dst, ignore=ignore)


def stage(out, with_images=True):
    from commander_sim import DATA, ROOT
    res = os.path.join(out, 'commander_ipad', 'resources')
    copy_tree(os.path.join(REPO, 'commander_sim'), os.path.join(out, 'commander_sim'))
    copy_tree(os.path.join(ROOT, 'decklists'), os.path.join(res, 'decklists'),
              ignore=shutil.ignore_patterns('retired', '__pycache__', '*.tmp'))
    data = os.path.join(res, 'data')
    if os.path.exists(data): shutil.rmtree(data)
    os.makedirs(data)
    for f in ('scryfall_cache.json', 'cards_dsl.json'):
        if os.path.exists(os.path.join(DATA, f)): shutil.copyfile(os.path.join(DATA, f), os.path.join(data, f))
    n = 0
    if with_images and os.path.isdir(os.path.join(DATA, 'images')):
        copy_tree(os.path.join(DATA, 'images'), os.path.join(data, 'images'))
        n = len(os.listdir(os.path.join(data, 'images')))
    try:
        commit = subprocess.run(['git', 'rev-parse', '--short', 'HEAD'], cwd=REPO, capture_output=True, text=True).stdout.strip()
    except OSError:
        commit = ''
    stamp = {'commit': commit, 'staged': dt.datetime.now().isoformat(timespec='seconds'), 'images': n}
    with open(os.path.join(data, 'bundle.json'), 'w', encoding='utf-8') as f: json.dump(stamp, f)
    return stamp


def main(argv=None):
    ap = argparse.ArgumentParser(description='Copy the code, deck lists and card data into ios/src/ for a build.')
    ap.add_argument('--images', action='store_true', help='download every card image first')
    ap.add_argument('--no-images', action='store_true', help='leave the images out (a smaller app; they download '
                    'on the iPad when it is online)')
    ap.add_argument('--out', default=os.path.join(IOS, 'src'), help=argparse.SUPPRESS)      # tests stage elsewhere
    a = ap.parse_args(argv)
    if a.images: fetch_images()
    stamp = stage(a.out, with_images=not a.no_images)
    print(f"Staged commit {stamp['commit'] or '?'} with {stamp['images']} image files into {a.out}")


if __name__ == '__main__':
    main()
