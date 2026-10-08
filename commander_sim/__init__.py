"""Commander pod simulator: measures a Commander deck against pools of outside decks in four-player games.

Run it with `python3 -m commander_sim ...` (see README.md); documents/architecture.md explains the design."""
import os

# Where the files live. On this computer: the repository (decklists/, data/, documents/). The iPad app (ios/) sets
# COMMANDER_SIM_ROOT to the deck lists bundled with it and COMMANDER_SIM_DATA to its own writable folder (card cache,
# images, saved games) before it imports anything from here.
ROOT = os.environ.get('COMMANDER_SIM_ROOT') or os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.environ.get('COMMANDER_SIM_DATA') or os.path.join(ROOT, 'data')
