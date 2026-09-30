"""Practice mode: play one of your decks by hand against three AI opponents from a tier.

Design and build plan: documents/practice-mode.md. The game runs on the ordinary engine in a worker thread
(session.py); decisions in the human seat go through a controller (controller.py); what the seat may see is built by
view.py. Nothing here changes a simulation: every hook is inert unless a game has a human seat.
"""
