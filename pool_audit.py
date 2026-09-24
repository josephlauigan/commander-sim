"""Card audit for the opponent pools: how faithfully the sim models every card in the 25 pool decks, and the
status of each deck's key engines (from its '## Sim modeling notes').

    python3 pool_audit.py                    # summary table per deck + engine status
    python3 pool_audit.py --deck yuriko      # every card of one deck (key prefix is enough)
    python3 pool_audit.py --md FILE          # write the tables as Markdown

Statuses (same meaning as CARD_AUDIT.md, with the automatic ones split out):
  Full         hand-verified against Oracle text (CARD_AUDIT.md 'Modeled', or CARD_NOTES below)
  Full-auto    every Oracle line compiled by the ability language and no ability keyword dropped; not hand-verified
  Approximate  modeled with a simplification (regex tags, or a note below)
  Partial      main effect in, some text or keywords ignored
  Unmodeled    the sim never casts it, or it does nothing when cast
"""
import re, sys
from collections import Counter

# ------------------------------------------------------------------ keywords the engine does not implement
# Scryfall keyword names (lower case). A card whose keyword is listed here is at best Partial unless a
# CARD_NOTES entry says otherwise. Remove a keyword from this set when the engine implements it.
UNSUPPORTED_KW = {
    'hexproof', 'indestructible', 'shroud', 'menace', 'first strike', 'double strike', 'ward', 'protection',
    'undying', 'persist', 'exalted', 'cascade', 'storm', 'affinity', 'delve', 'improvise', 'dredge', 'madness',
    'evoke', 'escape', 'buyback', 'rebound', 'devour', 'mentor', 'toxic', 'infect', 'prowess', 'ninjutsu',
    'commander ninjutsu', 'bestow', 'dethrone', 'myriad', 'annihilator', 'backup', 'blitz', 'dash', 'unearth',
    'battle cry', 'training', 'riot', 'afterlife', 'fabricate', 'embalm', 'eternalize', 'encore', 'extort',
    'modular', 'crew', 'living weapon', 'reconfigure', 'split second', 'suspend', 'foretell', 'kicker',
    'multikicker', 'entwine', 'overload', 'spectacle', 'surge', 'emerge', 'casualty', 'transmute', 'splice',
    'sunburst', 'totem armor', 'umbra armor', 'shadow', 'horsemanship', 'fear', 'intimidate', 'flanking',
    'bushido', 'provoke', 'rampage', 'soulbond', 'haunt', 'graft', 'bloodthirst', 'amplify', 'exploit',
    'craft', 'disturb', 'daybound', 'nightbound', 'cleave', 'prototype', 'squad', 'offspring', 'gift',
    'forestwalk', 'swampwalk', 'islandwalk', 'mountainwalk', 'plainswalk', 'landwalk', 'changeling',
    'adapt', 'monstrosity', 'channel', 'cycling', 'basic landcycling', 'landcycling', 'forage', 'plot',
}
STAT_TAGS = {'pow', 'tgh', 'fly', 'dt', 'vig', 'lifelink', 'haste', 'trample', 'reach', 'flash', 'convoke', 'leg',
             'human', 'warrior', 'shaman', 'wizard', 'bomb', 'noatk', 'c', 't', 'ck', 'f', 'amt'}
# hand tags whose behaviour lives in the four main decks' AI code: an outside deck holding the card never
# uses it unless the engine's generic path handles the tag (then remove it from this set)
DECK_ONLY_TAGS = {'prot', 'clamp', 'fill', 'rean', 'tide', 'yawg', 'avarice', 'mastery', 'crackle', 'x',
                  'skate', 'rabble', 'seal', 'jeska', 'intuition', 'adnaus', 'citadel', 'stampede', 'drawcre'}

# ------------------------------------------------------------------ hand-verified pool cards
# name -> (status, note). Wins over every automatic rule. Add an entry when a card is implemented or checked.
CARD_NOTES = {}

# ------------------------------------------------------------------ mechanics (engine-level)
# key -> (status, note). Update as mechanics are implemented.
MECH = {
    'commander_damage':  ('Full', '21 combat damage from one commander eliminates'),
    'extra_combat':      ('Partial', 'extra combat phases exist (max 4 per turn); most granting cards not compiled'),
    'attack_triggers':   ('Partial', 'interpreter fires attack triggers; many texts not compiled'),
    'isshin_doubling':   ('Unmodeled', 'attack-trigger doubling'),
    'teysa_doubling':    ('Unmodeled', 'death-trigger doubling'),
    'ninjutsu':          ('Unmodeled', ''),
    'yuriko_reveal':     ('Unmodeled', ''),
    'cost_taxes':        ('Partial', "only 'spells your opponents cast cost {N} more' compiles"),
    'spell_limits':      ('Unmodeled', 'one spell per turn (Rule of Law, Deafening Silence, Canonist, Archon of Emeria)'),
    'drannith_lock':     ('Unmodeled', ''),
    'attack_taxes':      ('Unmodeled', 'Ghostly Prison, Propaganda, Sphere of Safety, Norn\'s Annex, Windborn Muse'),
    'attacker_caps':     ('Unmodeled', 'Crawlspace, Silent Arbiter'),
    'undying_persist':   ('Unmodeled', 'only Mikaeus-granted undying (hand tag) exists'),
    'minus_counters':    ('Unmodeled', '-1/-1 counters and cancellation with +1/+1'),
    'planeswalkers':     ('Partial', 'loyalty abilities work; nobody attacks planeswalkers'),
    'proliferate':       ('Partial', 'only +1/+1 counters, not loyalty'),
    'counter_doubling':  ('Partial', 'token doubling compiles; counter doubling only for +1/+1 wording'),
    'treasure':          ('Full', ''),
    'food':              ('Unmodeled', ''),
    'clue':              ('Full', ''),
    'discard':           ('Approximate', 'opponents discard at random'),
    'edicts':            ('Approximate', 'the victim sacrifices its lowest-value creature'),
    'tergrid_steal':     ('Full', 'hand-coded (engine.tergrid_steal)'),
    'aluren':            ('Unmodeled', ''),
    'kinnan_mana':       ('Unmodeled', ''),
    'isochron_reversal': ('Unmodeled', ''),
    'power_artifact':    ('Unmodeled', ''),
    'thoracle_combo':    ('Unmodeled', "Thassa's Oracle + Demonic Consultation / Tainted Pact"),
    'helm_of_the_host':  ('Unmodeled', ''),
    'kiki_combo':        ('Unmodeled', 'Kiki-Jiki / Reflection + Zealous Conscripts / Felidar Guardian / Resto'),
    'auras':             ('Unmodeled', 'Auras attaching to creatures ("enchanted creature gets/has")'),
    'totem_armor':       ('Unmodeled', ''),
    'enchantress_draw':  ('Partial', "compiles as a cast trigger where the wording matches"),
    'put_attacking':     ('Unmodeled', 'Kaalia / Winota put a creature onto the battlefield attacking'),
    'korvold_sac':       ('Unmodeled', ''),
    'blood_moon':        ('Unmodeled', 'Magus of the Moon'),
    'graveyard_hate':    ('Unmodeled', 'Rest in Peace, Bojuka Bog, Dauthi Voidwalker, Tormod\'s Crypt'),
    'monarch':           ('Unmodeled', ''),
    'flicker':           ('Partial', 'blink effect exists; Brago / Closet / Deadeye / Soulherder not compiled'),
    'meren':             ('Unmodeled', ''),
    'recursion_ai':      ('Unmodeled', 'reanimation / graveyard filling only used by the main decks\' AI'),
    'landfall':          ('Approximate', 'landfall triggers fire on every land drop'),
    'extra_land_drops':  ('Unmodeled', 'Exploration, Burgeoning, Azusa, Dryad, Oracle of Mul Daya'),
    'land_recursion':    ('Unmodeled', 'Crucible, Ramunap, Life from the Loam, Greenwarden'),
    'elf_mana':          ('Unmodeled', 'mana scaling with Elf count (Priest of Titania, Archdruid, Cradle)'),
    'craterhoof':        ('Unmodeled', ''),
    'creature_tutors':   ('Approximate', 'tutors pick by priority; no combo-aware wish lists yet'),
    'krenko':            ('Approximate', 'X in Krenko\'s activation is a fixed 3'),
    'etb_damage':        ('Partial', 'Impact Tremors-style triggers compile only for some wordings'),
    'goblin_bombardment':('Unmodeled', 'sacrifice outlet with damage'),
    'thornbite_untap':   ('Unmodeled', ''),
    'impulse_draw':      ('Approximate', 'exile-and-play effects become draws'),
    'prosper_treasure':  ('Unmodeled', ''),
    'citadel':           ('Partial', 'hand-coded, but outside decks never cast it (no priority)'),
    'sanguine_bond':     ('Unmodeled', 'lifegain triggers never fire'),
    'top_manipulation':  ('Unmodeled', 'scry / Brainstorm / Top / Scroll Rack do nothing'),
    'free_counters':     ('Approximate', 'Force of Will / Fierce Guardianship; Pact, Force of Negation not free'),
    'counter_ai':        ('Partial', 'outside decks counter by importance; combo pieces not weighted'),
    'yawgmoth':          ('Unmodeled', ''),
    'necro_adnaus':      ('Partial', 'Necropotence works; Ad Nauseam never cast by outside decks'),
    'gravecrawler_loop': ('Unmodeled', ''),
    'zur':               ('Unmodeled', 'regex reads Zur as an ETB tutor'),
    'survival_pod':      ('Unmodeled', 'Survival of the Fittest, Birthing Pod'),
    'skullclamp':        ('Unmodeled', 'Skullclamp is only activated by Sephiroth\'s AI'),
    'protection_ai':     ('Unmodeled', 'Heroic Intervention, Teferi\'s Protection, Boots equips: main decks only'),
    'light_paws':        ('Unmodeled', 'regex reads Light-Paws as a tutor to hand'),
    'urza':              ('Unmodeled', ''),
    'artifact_mana':     ('Approximate', 'rocks tap for mana; untap loops not modeled'),
    'karn_lattice':      ('Unmodeled', ''),
    'chulane':           ('Partial', 'draw on creature cast; land drop part missing'),
    'winota':            ('Unmodeled', ''),
    'kaalia':            ('Unmodeled', ''),
    'windgrace':         ('Partial', 'loyalty abilities partly compiled'),
    'lathril':           ('Partial', 'token-on-damage compiled; tap-ten drain cost not payable'),
    'teysa':             ('Unmodeled', ''),
    'isshin':            ('Unmodeled', ''),
    'tatyova':           ('Full-auto', 'landfall: gain 1 life and draw'),
    'marwyn':            ('Partial', 'taps for G only, no counters'),
    'aurelia':           ('Unmodeled', ''),
    'atraxa_pv':         ('Approximate', 'hand tag: proliferates +1/+1 counters at end step'),
    'gaa':               ('Partial', 'opponent tax compiled; own cost reduction missing'),
    'sythis':            ('Full-auto', 'enchantment cast: gain 1, draw'),
    'brago':             ('Unmodeled', ''),
    'lathril_tap10':     ('Unmodeled', ''),
    'kinnan':            ('Unmodeled', ''),
}

# ------------------------------------------------------------------ engines per deck (from '## Sim modeling notes')
# deck key -> [(engine, [key cards], [mechanics])]
ENGINES = {
    'isshin-mardu-attack-triggers': [
        ("Isshin's attack-trigger doubling", ['Isshin, Two Heavens as One'], ['isshin_doubling']),
        ('token-on-attack creatures', ['Adeline, Resplendent Cathar', 'Hero of Bladehold', 'Brimaz, King of Oreskos',
                                       'Legion Warboss', 'Goblin Rabblemaster', 'Tilonalli\'s Summoner'], ['attack_triggers']),
        ('Hellrider / Brutal Hordechief per-attacker damage', ['Hellrider', 'Brutal Hordechief'], ['attack_triggers']),
        ('extra combats', ['Aurelia, the Warleader', 'Karlach, Fury of Avernus', 'Scourge of the Throne'], ['extra_combat']),
    ],
    'lathril-golgari-elves': [
        ('Elf-scaling mana', ['Priest of Titania', 'Elvish Archdruid'], ['elf_mana']),
        ("Lathril's tokens and tap-ten drain", ['Lathril, Blade of the Elves'], ['lathril', 'lathril_tap10']),
        ('lords and overruns', ['Elvish Archdruid', 'Imperious Perfect', 'Elvish Clancaller', 'Overwhelming Stampede',
                                'Ezuri, Renegade Leader', 'Shaman of the Pack'], []),
        ('Skullclamp', ['Skullclamp'], ['skullclamp']),
    ],
    'light-paws-aura-voltron': [
        ("Light-Paws Aura chain", ['Light-Paws, Emperor\'s Voice'], ['light_paws', 'auras']),
        ('enchantress draw', ['Sram, Senior Edificer', 'Mesa Enchantress', 'Kor Spiritdancer',
                              'Eidolon of Countless Battles'], ['enchantress_draw']),
        ('totem armor', ['Hyena Umbra', 'Mammoth Umbra', 'Felidar Umbra'], ['totem_armor', 'auras']),
        ('commander damage', [], ['commander_damage']),
    ],
    'tatyova-simic-landfall': [
        ('Tatyova landfall draw', ['Tatyova, Benthic Druid'], ['tatyova']),
        ('extra land drops', ['Exploration', 'Burgeoning', 'Azusa, Lost but Seeking', 'Dryad of the Ilysian Grove',
                              'Oracle of Mul Daya'], ['extra_land_drops']),
        ('landfall token output', ['Avenger of Zendikar', 'Scute Swarm', 'Rampaging Baloths'], ['landfall']),
        ('land recursion', ['Crucible of Worlds', 'Ramunap Excavator', 'Life from the Loam', 'Ancient Greenwarden'],
         ['land_recursion']),
    ],
    'teysa-orzhov-aristocrats': [
        ("Teysa's death-trigger doubling", ['Teysa Karlov'], ['teysa_doubling', 'teysa']),
        ('drain payoffs', ['Blood Artist', 'Zulaport Cutthroat', 'Cruel Celebrant', 'Bastion of Remembrance',
                           'Falkenrath Noble'], []),
        ('forced sacrifice', ['Grave Pact', 'Dictate of Erebos', 'Butcher of Malakir'], ['edicts']),
        ('Skullclamp', ['Skullclamp'], ['skullclamp']),
    ],
    'brago-azorius-blink-control': [
        ('flicker ETB re-use', ['Brago, King Eternal', 'Conjurer\'s Closet', 'Soulherder', 'Deadeye Navigator',
                                'Eldrazi Displacer'], ['flicker', 'brago']),
        ('counterspell decisions', ['Counterspell', 'Absorb', 'Cryptic Command', 'Memory Lapse'], ['counter_ai']),
        ('monarch', ['Palace Jailer'], ['monarch']),
    ],
    'kaalia-mardu-creature-cheat': [
        ("Kaalia's put-onto-battlefield-attacking", ['Kaalia of the Vast'], ['put_attacking', 'kaalia']),
        ('commander protection', ['Lightning Greaves', 'Swiftfoot Boots', 'Mother of Runes', 'Giver of Runes',
                                  'Selfless Spirit', 'Deflecting Swat'], ['protection_ai']),
        ('big flier combat', ['Baneslayer Angel', 'Lyra Dawnbringer', 'Thundermaw Hellkite', 'Terror of the Peaks'], []),
    ],
    'lord-windgrace-jund-lands': [
        ('Windgrace loyalty and land recursion', ['Lord Windgrace'], ['planeswalkers', 'windgrace']),
        ('landfall token makers', ['Avenger of Zendikar', 'Scute Swarm', 'Rampaging Baloths', 'Omnath, Locus of Rage',
                                   'Retreat to Kazandu'], ['landfall']),
        ('Titania elementals', ['Titania, Protector of Argoth'], []),
        ('Life from the Loam dredge', ['Life from the Loam'], ['land_recursion']),
    ],
    'meren-golgari-recursion': [
        ("Meren's end-step reanimation", ['Meren of Clan Nel Toth'], ['meren']),
        ('ETB removal re-use', ['Ravenous Chupacabra', 'Shriekmaw', 'Nekrataal', 'Acidic Slime'], []),
        ('sac outlets', ['Viscera Seer', 'Carrion Feeder', 'Ashnod\'s Altar'], []),
        ('Grave Pact', ['Grave Pact'], ['edicts']),
        ('Survival / Birthing Pod', ['Survival of the Fittest', 'Birthing Pod'], ['survival_pod']),
    ],
    'sythis-selesnya-enchantress': [
        ('enchantress draw', ['Argothian Enchantress', 'Enchantress\'s Presence', 'Mesa Enchantress',
                              'Verduran Enchantress', 'Satyr Enchanter', 'Setessan Champion'], ['enchantress_draw']),
        ('attack taxes and caps', ['Ghostly Prison', 'Sphere of Safety', 'Norn\'s Annex', 'Crawlspace',
                                   'Silent Arbiter', 'Windborn Muse', 'Baird, Steward of Argive'],
         ['attack_taxes', 'attacker_caps']),
        ('Sigil of the Empty Throne', ['Sigil of the Empty Throne'], []),
        ('Starfield of Nyx', ['Starfield of Nyx'], []),
        ('Sythis cast trigger', ['Sythis, Harvest\'s Hand'], ['sythis']),
    ],
    'atraxa-superfriends': [
        ('planeswalker loyalty and ultimates', ['Jace, the Mind Sculptor', 'Teferi, Hero of Dominaria',
                                                'Elspeth, Sun\'s Champion', 'Karn Liberated', 'Oko, Thief of Crowns'],
         ['planeswalkers']),
        ('proliferate', ['Atraxa, Praetors\' Voice', 'Evolution Sage', 'Flux Channeler', 'Tekuthal, Inquiry Dominus'],
         ['proliferate', 'atraxa_pv']),
        ('Doubling Season', ['Doubling Season'], ['counter_doubling']),
        ('attack taxes', ['Ghostly Prison', 'Propaganda'], ['attack_taxes']),
    ],
    'aurelia-boros-extra-combats': [
        ('additional combat phases', ['Aurelia, the Warleader', 'Relentless Assault', 'Seize the Day', 'World at War',
                                      'Combat Celebrant', 'Hellkite Charger', 'Port Razer', 'Karlach, Fury of Avernus',
                                      'Scourge of the Throne'], ['extra_combat', 'aurelia']),
        ('attack triggers per combat', ['Hero of Bladehold', 'Adeline, Resplendent Cathar', 'Legion Warboss'],
         ['attack_triggers']),
        ('equipment', ['Embercleave', 'Sword of Fire and Ice', 'Shadowspear', 'Sword of Hearth and Home'], []),
        ('Helm of the Host + Combat Celebrant', ['Helm of the Host', 'Combat Celebrant'], ['helm_of_the_host']),
    ],
    'korvold-jund-sacrifice': [
        ('Korvold sacrifice draw', ['Korvold, Fae-Cursed King'], ['korvold_sac']),
        ('Treasure / Food / Clue tokens', ['Gilded Goose', 'Trail of Crumbs', 'Witch\'s Oven', 'Tireless Provisioner'],
         ['treasure', 'food', 'clue']),
        ('per-sacrifice damage', ['Mayhem Devil', 'Goblin Bombardment', 'Blood Artist', 'Zulaport Cutthroat'],
         ['goblin_bombardment']),
        ('Gravecrawler + Phyrexian Altar loop', ['Gravecrawler', 'Phyrexian Altar'], ['gravecrawler_loop']),
    ],
    'marwyn-mono-green-elves': [
        ('Elf-scaling mana', ['Priest of Titania', 'Elvish Archdruid', 'Circle of Dreams Druid'], ['elf_mana', 'marwyn']),
        ("Gaea's Cradle", ["Gaea's Cradle"], ['elf_mana']),
        ('Craterhoof overrun', ['Craterhoof Behemoth'], ['craterhoof']),
        ('creature tutors', ['Natural Order', 'Chord of Calling', 'Green Sun\'s Zenith', 'Finale of Devastation',
                             'Eldritch Evolution'], ['creature_tutors']),
    ],
    'tergrid-mono-black-disruption': [
        ("Tergrid's steal", ['Tergrid, God of Fright // Tergrid\'s Lantern'], ['tergrid_steal']),
        ('edicts', ['Chainer\'s Edict', 'Liliana\'s Triumph', 'Sheoldred\'s Edict', 'Innocent Blood', 'Plaguecrafter',
                    'Fleshbag Marauder'], ['edicts']),
        ('discard', ['Hymn to Tourach', 'Mind Twist', 'Liliana of the Veil', 'Tinybones, Trinket Thief'], ['discard']),
        ('Grave Pact / Dictate', ['Grave Pact', 'Dictate of Erebos'], ['edicts']),
        ('Cabal Coffers + Urborg', ['Cabal Coffers', 'Urborg, Tomb of Yawgmoth'], []),
    ],
    'chulane-bant-value-combo': [
        ('Chulane cast trigger', ['Chulane, Teller of Tales'], ['chulane']),
        ('Aluren free casting', ['Aluren'], ['aluren']),
        ('self-bounce creatures', ['Shrieking Drake', 'Whitemane Lion', 'Kor Skyfisher', 'Man-o\'-War'], []),
        ('creature tutors', ['Chord of Calling', 'Green Sun\'s Zenith', 'Finale of Devastation', 'Survival of the Fittest'],
         ['creature_tutors']),
    ],
    'gaa-azorius-stax-control': [
        ('static cost increases', ['Grand Arbiter Augustin IV', 'Thalia, Guardian of Thraben', 'Sphere of Resistance',
                                   'Thorn of Amethyst', 'Glowrider', 'Vryn Wingmare', 'Lavinia, Azorius Renegade'],
         ['cost_taxes', 'gaa']),
        ('one-spell-per-turn limits', ['Rule of Law', 'Deafening Silence', 'Ethersworn Canonist', 'Archon of Emeria'],
         ['spell_limits']),
        ('Drannith lock', ['Drannith Magistrate'], ['drannith_lock']),
        ('Isochron + Reversal', ['Isochron Scepter', 'Dramatic Reversal'], ['isochron_reversal']),
        ('counterspells', ['Counterspell', 'Mana Drain', 'Force of Will', 'Flusterstorm', 'Absorb'], ['counter_ai']),
    ],
    'krenko-mono-red-goblins': [
        ("Krenko's token activation", ['Krenko, Mob Boss'], ['krenko']),
        ('ETB damage', ['Impact Tremors', 'Purphoros, God of the Forge'], ['etb_damage']),
        ('Goblin Bombardment', ['Goblin Bombardment'], ['goblin_bombardment']),
        ('Kiki-Jiki + Conscripts', ['Kiki-Jiki, Mirror Breaker', 'Zealous Conscripts',
                                    'Fable of the Mirror-Breaker // Reflection of Kiki-Jiki'], ['kiki_combo']),
        ('Thornbite Staff loop', ['Thornbite Staff'], ['thornbite_untap']),
        ('Goblin tutors', ['Goblin Lackey', 'Goblin Recruiter', 'Goblin Matron', 'Muxus, Goblin Grandee'],
         ['creature_tutors']),
    ],
    'prosper-rakdos-exile-treasure': [
        ('Prosper Treasure on exile-cast', ['Prosper, Tome-Bound'], ['prosper_treasure']),
        ('impulse draw', ['Reckless Impulse', 'Light Up the Stage', 'Outpost Siege', 'Valakut Exploration'],
         ['impulse_draw']),
        ('Treasure payoffs', ['Mayhem Devil', 'Storm-Kiln Artist'], ['treasure']),
        ("Bolas's Citadel", ["Bolas's Citadel"], ['citadel']),
        ('Sanguine Bond + Exquisite Blood', ['Sanguine Bond', 'Exquisite Blood'], ['sanguine_bond']),
    ],
    'yuriko-dimir-ninjas': [
        ('ninjutsu', ['Yuriko, the Tiger\'s Shadow', 'Ink-Eyes, Servant of Oni', 'Fallen Shinobi', 'Silver-Fur Master',
                      'Mistblade Shinobi', 'Moon-Circuit Hacker'], ['ninjutsu']),
        ('Yuriko reveal damage', ['Yuriko, the Tiger\'s Shadow'], ['yuriko_reveal']),
        ('top-of-library manipulation', ['Brainstorm', 'Scroll Rack', "Sensei's Divining Top", 'Vampiric Tutor'],
         ['top_manipulation']),
        ('free counterspells', ['Fierce Guardianship', 'Force of Negation'], ['free_counters']),
    ],
    'kinnan-simic-mana-combo': [
        ("Kinnan's extra mana", ['Kinnan, Bonder Prodigy'], ['kinnan_mana']),
        ('Basalt / Freed / Pemmin loops', ['Basalt Monolith', 'Freed from the Real', "Pemmin's Aura"], ['kinnan_mana']),
        ('Kinnan activation', ['Kinnan, Bonder Prodigy'], ['kinnan']),
        ('Isochron + Reversal', ['Isochron Scepter', 'Dramatic Reversal'], ['isochron_reversal']),
        ('free counterspells', ['Force of Will', 'Fierce Guardianship', 'Pact of Negation'], ['free_counters']),
    ],
    'urza-mono-blue-artifacts': [
        ('artifact fast mana', ['Mana Vault', 'Grim Monolith', 'Basalt Monolith', 'Mox Opal', 'Chrome Mox'],
         ['artifact_mana']),
        ('Urza tapping artifacts, Construct', ['Urza, Lord High Artificer'], ['urza']),
        ('Isochron + Reversal', ['Isochron Scepter', 'Dramatic Reversal'], ['isochron_reversal']),
        ('Power Artifact / Rings loops', ['Power Artifact', 'Rings of Brighthearth'], ['power_artifact']),
        ('Karn + Lattice lock', ['Karn, the Great Creator', 'Mycosynth Lattice'], ['karn_lattice']),
        ('taxing artifacts', ['Sphere of Resistance', 'Thorn of Amethyst', 'Trinisphere'], ['cost_taxes']),
        ('free counterspells', ['Force of Will', 'Fierce Guardianship', 'Pact of Negation'], ['free_counters']),
    ],
    'winota-boros-humans-cheat': [
        ("Winota's attack trigger", ['Winota, Joiner of Forces'], ['put_attacking', 'winota']),
        ('Kiki-Jiki copy combos', ['Kiki-Jiki, Mirror Breaker', 'Zealous Conscripts', 'Felidar Guardian',
                                   'Restoration Angel'], ['kiki_combo']),
        ('stax Humans', ['Thalia, Guardian of Thraben', 'Thalia, Heretic Cathar', 'Drannith Magistrate',
                         'Magus of the Moon'], ['cost_taxes', 'drannith_lock', 'blood_moon']),
        ('protection spells', ['Flawless Maneuver', "Teferi's Protection", 'Deflecting Swat', 'Selfless Spirit'],
         ['protection_ai']),
    ],
    'yawgmoth-mono-black-aristocrats': [
        ('Yawgmoth sac / -1/-1 / draw', ['Yawgmoth, Thran Physician'], ['yawgmoth', 'minus_counters']),
        ('undying and counter cancellation', ['Mikaeus, the Unhallowed', 'Geralf\'s Messenger', 'Butcher Ghoul'],
         ['undying_persist', 'minus_counters']),
        ('death-trigger drains', ['Blood Artist', 'Zulaport Cutthroat', 'Bastion of Remembrance', 'Syr Konrad, the Grim'],
         []),
        ('Gravecrawler + Altar', ['Gravecrawler', 'Phyrexian Altar'], ['gravecrawler_loop']),
        ('Necropotence / Ad Nauseam / Citadel', ['Necropotence', 'Ad Nauseam', "Bolas's Citadel"],
         ['necro_adnaus', 'citadel']),
    ],
    'zur-esper-enchantment-control': [
        ("Zur's attack trigger", ['Zur the Enchanter'], ['zur']),
        ('Necropotence', ['Necropotence'], ['necro_adnaus']),
        ("Thassa's Oracle + Consultation / Pact", ["Thassa's Oracle", 'Demonic Consultation', 'Tainted Pact'],
         ['thoracle_combo']),
        ('free counterspells', ['Force of Will', 'Fierce Guardianship', 'Pact of Negation', 'Force of Negation'],
         ['free_counters']),
        ('graveyard exile', ['Rest in Peace', 'Dauthi Voidwalker'], ['graveyard_hate']),
    ],
}

RANK = {'Full': 4, 'Full-auto': 3, 'Approximate': 2, 'Partial': 1, 'Unmodeled': 0}
STATUSES = ('Full', 'Full-auto', 'Approximate', 'Partial', 'Unmodeled')


def _main_audit():
    st = {}
    try:
        for line in open(__import__('os').path.join(__import__('os').path.dirname(__file__), 'CARD_AUDIT.md')):
            m = re.match(r'^\| (.+?) \| (Modeled|Approximate|Partial|Not modeled|Unverified) \| (.*?) \|', line)
            if m: st.setdefault(m.group(1), (m.group(2), m.group(3)))
    except OSError:
        pass
    return st


_AUDIT = None


def card_status(name):
    """(status, note) for one card as an outside deck plays it"""
    global _AUDIT
    import engine, scryfall, autotag
    if name in CARD_NOTES: return CARD_NOTES[name]
    if _AUDIT is None: _AUDIT = _main_audit()
    cd = engine.DB[name]
    if name in ('Plains', 'Island', 'Swamp', 'Mountain', 'Forest', 'Wastes'): return 'Full', 'basic land'
    rec = scryfall.fetch([name], verbose=False).get(name) or {}
    kws = {k.lower() for k in rec.get('keywords') or []} & UNSUPPORTED_KW
    kwnote = ('keywords not modeled: ' + ', '.join(sorted(kws))) if kws else ''
    if cd.source == 'manual':
        bad = sorted(k for k in cd.tags if k in DECK_ONLY_TAGS)
        if bad:
            return 'Unmodeled', f"hand tags ({', '.join(bad)}) are only acted on by the main decks' AI"
        s, note = _AUDIT.get(name, ('Modeled', 'hand-tagged'))
        s = {'Modeled': 'Full', 'Not modeled': 'Unmodeled', 'Unverified': 'Approximate'}.get(s, s)
        return s, note
    if cd.land:
        util = [u for u in autotag.autotag(rec)['unparsed']] if rec else []
        if util: return 'Partial', 'land utility ignored: ' + '; '.join(u[:70] for u in util)
        return 'Full', 'mana only'
    functional = {k for k in cd.tags if k not in STAT_TAGS}
    abil = [a for a in (cd.dsl or []) if a.get('static') != 'note' and a.get('type') != 'additional_cost']
    unparsed = list(cd.unparsed)
    oracle = (rec.get('oracle_text') or '') or ' '.join(f.get('oracle_text') or '' for f in rec.get('card_faces') or [])
    if not abil and not functional:
        if cd.creature:
            if not rules_text(oracle): return 'Full', 'vanilla creature (keywords only)'
            return 'Partial', 'body only, abilities not modeled: ' + ('; '.join(u[:70] for u in unparsed[:3]) or rules_text(oracle)[:120]) + (('; ' + kwnote) if kwnote else '')
        return 'Unmodeled', '; '.join(u[:70] for u in unparsed[:3]) or 'no effect recognised'
    notes = [u[:80] for u in unparsed[:3]] + ([kwnote] if kwnote else [])
    if unparsed or kws: return 'Partial', '; '.join(notes)
    if cd.source == 'scryfall+dsl': return 'Full-auto', 'compiled from Oracle text'
    return 'Approximate', 'regex tags: ' + ' '.join(sorted(functional))


SIMPLE_KW = ('flying', 'deathtouch', 'vigilance', 'lifelink', 'haste', 'trample', 'reach', 'flash')


def rules_text(oracle):
    """Oracle text minus reminder text and lines made only of keywords the engine models"""
    t = re.sub(r'\([^)]*\)', '', oracle or '')
    keep = [l for l in t.split('\n') if l.strip() and not all(w.strip().lower() in SIMPLE_KW for w in l.split(','))]
    return ' / '.join(keep)


def engine_status(deck, cards, mechs):
    worst, notes = 'Full', []
    for c in cards:
        if c not in deck.cards: continue
        s, n = card_status(c)
        if RANK[s] < RANK[worst]: worst = s
        if RANK[s] < RANK['Full-auto']: notes.append(f'{c}: {s}')
    for m in mechs:
        s, n = MECH[m]
        if RANK[s] < RANK[worst]: worst = s
        if RANK[s] < RANK['Full-auto']: notes.append(f'{m}: {s}' + (f' ({n})' if n else ''))
    return worst, notes


def audit(decks=None):
    import pools
    pools.register()
    decks = decks or pools.load_pool()
    out = []
    for d in decks:
        uniq = sorted(set(d.cards))
        per = {n: card_status(n) for n in uniq}
        cnt = Counter(s for s, _ in per.values())
        eng = [(label, *engine_status(d, cs, ms)) for label, cs, ms in ENGINES.get(d.key, [])]
        missing = [c for _, cs, _ in ENGINES.get(d.key, []) for c in cs if c not in d.cards]
        out.append((d, per, cnt, eng, missing))
    return out


def markdown(rows):
    L = ['| Tier | Deck | ' + ' | '.join(STATUSES) + ' | Engines at Full/Full-auto |', '|' + '---|' * (len(STATUSES) + 3)]
    for d, per, cnt, eng, _ in rows:
        ok = sum(1 for _, s, _ in eng if RANK[s] >= RANK['Full-auto'])
        L.append(f'| {d.tier[1]} | {__import__("pools").short_name(d)} | ' + ' | '.join(str(cnt.get(s, 0)) for s in STATUSES)
                 + f' | {ok}/{len(eng)} |')
    L.append('')
    for d, per, cnt, eng, _ in rows:
        L.append(f'**{d.title}** (Tier {d.tier[1]})')
        L.append('')
        L.append('| Engine | Status | What is missing |')
        L.append('|---|---|---|')
        for label, s, notes in eng:
            L.append(f'| {label} | {s} | {"; ".join(notes) or "-"} |')
        L.append('')
    return '\n'.join(L)


def main():
    args = sys.argv[1:]
    rows = audit()
    if '--deck' in args:
        key = args[args.index('--deck') + 1]
        for d, per, cnt, eng, missing in rows:
            if not d.key.startswith(key): continue
            print(f'== {d.title}')
            for n in sorted(per, key=lambda n: (RANK[per[n][0]], n)):
                print(f'  {per[n][0]:11s} {n[:40]:40s} {per[n][1][:150]}')
        return
    if '--md' in args:
        open(args[args.index('--md') + 1], 'w').write(markdown(rows) + '\n')
    print(f"{'deck':34s} " + ' '.join(f'{s[:9]:>9s}' for s in STATUSES) + '   engines ok')
    for d, per, cnt, eng, missing in rows:
        ok = sum(1 for _, s, _ in eng if RANK[s] >= RANK['Full-auto'])
        print(f'{d.key[:34]:34s} ' + ' '.join(f'{cnt.get(s, 0):9d}' for s in STATUSES) + f'   {ok}/{len(eng)}')
        if missing: print(f'    (engine cards not in this list: {", ".join(missing)})')
    tot = Counter()
    for _, per, _, _, _ in rows: tot.update(s for s, _ in per.values())
    print('all decks (card-slots, unique per deck): ' + ', '.join(f'{s} {tot[s]}' for s in STATUSES))


if __name__ == '__main__':
    main()
