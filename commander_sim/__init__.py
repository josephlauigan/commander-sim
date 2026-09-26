"""Commander pod simulator: measures a Commander deck against pools of outside decks in four-player games.

Run it with `python3 -m commander_sim ...` (see README.md); documents/architecture.md explains the design."""
import os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))    # the repository: decklists/, data/, documents/
DATA = os.path.join(ROOT, 'data')
