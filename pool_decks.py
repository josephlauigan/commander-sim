"""Per-deck AI configuration for the opponent pools (read by pool_ai via pools.register).

  style      play style for the adaptive AI: aggression, caution, temp (see brain.STYLE)
  key_cards  {card name: cast priority 0-90} overriding the generic tag-based priority
  wish       tutor wish list (first card still in the library and not in hand/play is fetched), or a
             function (g, p) -> list
  cmd_prio   commander cast priority (default 75), cmd_turn: earliest turn to cast it (default 2)
Decks without an entry use pool_ai.DEFAULT_STYLE and the generic priority.
"""
CONFIG = {
    'light-paws-aura-voltron': {'aura_host': 'commander', 'aura_prio': 70, 'cmd_prio': 88, 'cmd_turn': 1,
                                'style': {'temp': 1.0, 'aggression': 0.8, 'caution': 0.4}},
}
