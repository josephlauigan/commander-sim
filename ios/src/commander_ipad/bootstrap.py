"""Start-up for the iPad app, kept free of Toga so it can be tested on any computer (tests/test_ios_app.py).

The app bundle holds a copy of the commander_sim package and, under resources/, the deck lists and the data folder
(card cache, card images) that ios/stage.py put there. An iOS app's bundle is read-only, so the data folder is copied
into the app's own writable folder, and the core is pointed at both through COMMANDER_SIM_ROOT and COMMANDER_SIM_DATA
before anything from it is imported.
"""
import json
import os
import shutil
import sys

STAMP = 'bundle.json'           # in the bundled data folder: which build staged it (ios/stage.py writes it)
INDEX = 'index.json'            # the card images' index (commander_sim/play/images.py)


def _read(path):
    try:
        with open(path, encoding='utf-8') as f: return json.load(f)
    except (OSError, ValueError):
        return None


def _write(path, data):
    tmp = path + '.tmp'
    with open(tmp, 'w', encoding='utf-8') as f: json.dump(data, f)
    os.replace(tmp, path)


def sync_data(bundled, data):
    """copy the bundled data folder into the writable one when this build's stamp differs from the last one copied.
    Files are replaced (the card cache is the engine's card data, so a new build's must win); images are copied only
    when missing, and the images index is merged, so images downloaded since stay. Saved games are never touched.
    Returns whether it copied."""
    os.makedirs(data, exist_ok=True)
    stamp = _read(os.path.join(bundled, STAMP))
    if stamp is not None and _read(os.path.join(data, STAMP)) == stamp: return False
    for name in os.listdir(bundled):
        src, dst = os.path.join(bundled, name), os.path.join(data, name)
        if name == STAMP: continue
        if not os.path.isdir(src):
            shutil.copyfile(src, dst)
            continue
        os.makedirs(dst, exist_ok=True)
        for f in os.listdir(src):
            if f == INDEX:
                idx = _read(os.path.join(dst, f)) or {}
                idx.update(_read(os.path.join(src, f)) or {})
                _write(os.path.join(dst, f), idx)
            elif not os.path.exists(os.path.join(dst, f)):
                shutil.copyfile(os.path.join(src, f), os.path.join(dst, f))
    if stamp is not None: _write(os.path.join(data, STAMP), stamp)
    return True


def prepare(app_dir, data_dir):
    """point the core at the bundled deck lists and the writable data folder, copying the bundled data in first"""
    if 'commander_sim' in sys.modules:
        raise RuntimeError('commander_sim was imported before its folders were set')
    res = os.path.join(app_dir, 'resources')
    sync_data(os.path.join(res, 'data'), data_dir)
    os.environ['COMMANDER_SIM_ROOT'] = res
    os.environ['COMMANDER_SIM_DATA'] = data_dir


def start(app_dir, data_dir, fetch_images=True):
    """prepare, then start the practice server on this device: returns (server, url)"""
    prepare(app_dir, data_dir)
    from commander_sim.play import server
    return server.start(fetch_images=fetch_images)
