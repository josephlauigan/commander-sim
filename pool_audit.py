"""Card audit for the opponent pools: how faithfully the sim models every card in the 25 pool decks, and the
status of each deck's key engines (from its '## Sim modeling notes').

    python3 pool_audit.py                    # summary table per deck + engine status
    python3 pool_audit.py --deck yuriko      # every card of one deck (key prefix is enough)
    python3 pool_audit.py --md FILE          # write the tables as Markdown
    python3 pool_audit.py --mine             # your decks: every card that is not Full

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
    'cascade', 'storm', 'affinity', 'delve', 'improvise', 'dredge', 'madness',
    'evoke', 'escape', 'buyback', 'rebound', 'devour', 'toxic', 'infect', 'prowess', 'ninjutsu',
    'commander ninjutsu', 'bestow', 'myriad', 'annihilator', 'backup', 'blitz', 'dash', 'unearth',
    'training', 'riot', 'afterlife', 'fabricate', 'embalm', 'eternalize', 'encore', 'extort',
    'modular', 'crew', 'living weapon', 'reconfigure', 'split second', 'suspend', 'foretell', 'kicker',
    'multikicker', 'entwine', 'overload', 'spectacle', 'surge', 'emerge', 'casualty', 'transmute', 'splice',
    'sunburst', 'shadow', 'horsemanship', 'fear', 'intimidate', 'flanking',
    'bushido', 'provoke', 'rampage', 'soulbond', 'haunt', 'graft', 'bloodthirst', 'amplify', 'exploit',
    'craft', 'disturb', 'daybound', 'nightbound', 'cleave', 'prototype', 'squad', 'offspring', 'gift',
    'forestwalk', 'swampwalk', 'islandwalk', 'mountainwalk', 'plainswalk', 'landwalk',
    'adapt', 'monstrosity', 'channel', 'cycling', 'basic landcycling', 'landcycling', 'forage', 'plot',
}
STAT_TAGS = {'pow', 'tgh', 'fly', 'dt', 'vig', 'lifelink', 'haste', 'trample', 'reach', 'flash', 'convoke', 'leg',
             'human', 'warrior', 'shaman', 'wizard', 'bomb', 'noatk', 'c', 't', 'ck', 'f', 'amt'}
# hand tags whose behaviour lives in the four main decks' AI code: an outside deck holding the card never
# uses it unless the engine's generic path handles the tag (then remove it from this set)
DECK_ONLY_TAGS = {'tide', 'yawg', 'avarice', 'mastery', 'crackle', 'x'}
POOL_OK = {'fill': ('dispute', 'stitcher', 'wayfinder', 'grisly'), 'rean': ('animate', 'reanimate', 'evil'),
           'prot': ('hi', 'phase', 'indes', 'blink', 'boots')}


def deck_only(cd):
    bad = [k for k in cd.tags if k in DECK_ONLY_TAGS]
    bad += [f'{k}={cd.tags[k]}' for k, ok in POOL_OK.items() if k in cd.tags and cd.tags[k] not in ok]
    return bad

# ------------------------------------------------------------------ hand-verified pool cards
# name -> (status, note). Wins over every automatic rule. Add an entry when a card is implemented or checked.
CARD_NOTES = {}

# ------------------------------------------------------------------ mechanics (engine-level)
# key -> (status, note). Update as mechanics are implemented.
MECH = {
    'commander_damage':  ('Full', '21 combat damage from one commander eliminates'),
    'extra_combat': ('Full', 'Aurelia, Karlach, Scourge, Combat Celebrant, Hellkite Charger, Port Razer, Relentless Assault, Seize the Day, World at War, Savage Beating; max 4 combats a turn'),
    'attack_triggers': ('Approximate', 'attack triggers: hooks for the key cards, compiled text for the rest'),
    'isshin_doubling': ('Full', 'attack-caused triggers fire twice (hand tags, compiled triggers, hooks)'),
    'teysa_doubling': ('Full', 'death-caused triggers of your permanents fire twice'),
    'ninjutsu': ('Full', 'unblocked attacker swapped for a Ninja from hand (or Yuriko from the command zone); Satoru, Silver-Fur Master'),
    'yuriko_reveal': ('Full', 'reveal top card: to hand, each opponent loses its mana value'),
    'cost_taxes': ('Full', 'Thalia, Sphere, Thorn, Trinisphere, Glowrider, Vryn Wingmare, Tithe Taker, Aura of Silence, Dovin, GAA'),
    'spell_limits': ('Full', 'Rule of Law, Deafening Silence, Ethersworn Canonist, Archon of Emeria (per turn, any turn)'),
    'drannith_lock': ('Full', 'opponents cast only from hand (commanders, flashback, escape off)'),
    'attack_taxes': ('Full', "Ghostly Prison, Propaganda, Sphere of Safety, Windborn Muse, Baird, Archangel of Tithes; Norn's Annex and Elephant Grass approximated"),
    'attacker_caps': ('Approximate', "Crawlspace and Silent Arbiter attack caps; Silent Arbiter's block cap missing"),
    'undying_persist': ('Full', 'undying / persist keywords; Mikaeus-granted undying'),
    'minus_counters': ('Full', '-1/-1 counters cancel +1/+1 (net counters)'),
    'planeswalkers': ('Approximate', 'hand-written walkers use one ability a turn; attackers go after valuable walkers (pool games); walkers without an implementation use compiled loyalty abilities'),
    'proliferate': ('Full', "+1/+1, loyalty and other counters on yours, -1/-1 on opponents' creatures; Tekuthal doubles"),
    'counter_doubling': ('Full', 'Doubling Season: tokens, counters, walkers enter with double loyalty'),
    'treasure':          ('Full', ''),
    'food': ('Approximate', 'Food tokens: sacrifice fodder (Korvold, Trail of Crumbs, Goose mana, Savvy Hunter); eaten when low'),
    'clue':              ('Full', ''),
    'discard': ('Approximate', 'targeted discard picks the fullest hand; random discard at random; opponents choose their worst card for symmetric discard'),
    'edicts':            ('Approximate', 'the victim sacrifices its lowest-value creature'),
    'tergrid_steal':     ('Full', 'hand-coded (engine.tergrid_steal)'),
    'aluren': ('Approximate', 'free creature spells MV 3 or less; the Chulane loop is a combo'),
    'kinnan_mana': ('Full', 'nonland mana sources make one extra'),
    'isochron_reversal': ('Approximate', 'combo: needs 3+ mana from nonland permanents and a mana sink; loop abstracted'),
    'power_artifact': ('Approximate', 'combo with Basalt/Grim Monolith (Rings + Basalt too); loop abstracted'),
    'thoracle_combo': ('Approximate', 'combo: both spells cast through the counter window; win abstracted'),
    'helm_of_the_host': ('Full', 'hasty copy each combat; with Combat Celebrant a combo'),
    'kiki_combo': ('Approximate', 'combo: Kiki / Reflection + Conscripts / Felidar / Resto; loop abstracted'),
    'auras': ('Approximate', 'Auras attach to the best creature (the commander in voltron decks); bonuses, keywords, protection; hostile Auras modeled as exile'),
    'totem_armor': ('Full', 'umbra armor destroys the Aura instead'),
    'enchantress_draw': ('Full', 'cast triggers (compiled or hooked)'),
    'put_attacking': ('Full', 'Kaalia (hand) and Winota (top six) put creatures onto the battlefield attacking'),
    'korvold_sac': ('Full', 'every sacrifice incl. Treasure/Food/Clue: counter and a card'),
    'blood_moon': ('Full', 'Magus of the Moon: nonbasic lands tap for R'),
    'graveyard_hate': ('Approximate', 'Rest in Peace keeps graveyards empty, Bojuka Bog, Dauthi void exile, Crypt / Lantern in response to reanimation or proactively'),
    'monarch': ('Full', 'end-step draw; combat damage to the monarch takes it'),
    'flicker': ('Approximate', 'blink re-triggers ETBs (Brago, Soulherder, Closet, Displacer, Deadeye, Ephemerate, Resto); Deadeye soulbond read as a paid blink'),
    'meren': ('Full', 'experience counters; end-step return (battlefield or hand)'),
    'recursion_ai': ('Approximate', 'reanimation spells and Deadly Dispute usable by outside decks'),
    'landfall': ('Full', 'landfall triggers, fetch lands crack for a second trigger (pool games)'),
    'extra_land_drops': ('Full', 'Exploration, Azusa, Dryad, Oracle of Mul Daya, Aesi, Druid Class, Explore'),
    'land_recursion': ('Approximate', 'Crucible / Ramunap / Greenwarden land drops from the graveyard; Loam dredge when short on lands'),
    'elf_mana': ('Full', 'Priest, Archdruid, Channeler, Heritage, Circle of Dreams, Marwyn, Cradle, Nykthos (approx)'),
    'craterhoof': ('Full', 'X = creatures, trample, haste; cast / tutored when the board is wide'),
    'creature_tutors': ('Approximate', 'Natural Order, GSZ, Finale, Eldritch Evolution, Chord onto the battlefield; the target is the best card (Craterhoof on a wide board)'),
    'krenko': ('Full', 'tap: Goblins equal to Goblins you control'),
    'etb_damage': ('Full', 'Impact Tremors, Purphoros'),
    'goblin_bombardment': ('Full', 'sacrifice outlet dealing 1 (aristocrat AI decides when)'),
    'thornbite_untap': ('Approximate', 'combo: Krenko + Thornbite Staff + sacrifice outlet'),
    'impulse_draw': ('Approximate', 'exile-and-play cards go to hand until they expire'),
    'prosper_treasure': ('Full', 'Treasure whenever you play a card from exile'),
    'citadel': ('Full', 'hand-coded; outside decks cast it with life to spare'),
    'sanguine_bond': ('Approximate', 'combo: Sanguine Bond + Exquisite Blood; loop abstracted'),
    'top_manipulation': ('Approximate', 'Brainstorm puts back the two worst cards; Top / Scroll Rack / Ponder not modeled'),
    'free_counters': ('Full', "Force of Will, Force of Negation (opponents' turns), Fierce Guardianship, Pact of Negation (upkeep payment), Mental Misstep"),
    'counter_ai': ('Approximate', 'counters by importance; spells that complete a combo count as critical'),
    'yawgmoth': ('Full', 'sacrifice / -1/-1 / draw and proliferate; the undying loop is a combo'),
    'necro_adnaus': ('Full', 'Necropotence end-step life payment; Ad Nauseam cast above 30 life'),
    'gravecrawler_loop': ('Approximate', 'combo with Phyrexian Altar, another Zombie and a death payoff'),
    'zur': ('Full', 'attack: enchantment MV 3 or less onto the battlefield'),
    'survival_pod': ('Approximate', 'Survival discards the worst creature for the best; Pod upgrades by one MV'),
    'skullclamp': ('Full', 'equip to an X/1 for two cards (outside decks too)'),
    'protection_ai': ('Approximate', 'outside decks answer targeted removal and wipes with their protection cards (and sacrifice creatures in response for value)'),
    'light_paws': ('Full', 'Aura cast -> Aura from library onto Light-Paws'),
    'urza': ('Full', 'Construct, artifacts tap for U, {5} free card'),
    'artifact_mana': ('Approximate', 'rocks and Monoliths tap for mana; untap loops are combos'),
    'karn_lattice': ('Approximate', "Karn shuts off opponents' artifact mana; with Lattice a lock (no mana from permanents)"),
    'chulane': ('Full', ''),
    'winota': ('Full', ''),
    'kaalia': ('Full', ''),
    'windgrace': ('Full', 'loyalty abilities incl. ultimate'),
    'lathril': ('Full', 'Elf tokens equal to combat damage'),
    'teysa': ('Full', 'tokens have vigilance and lifelink'),
    'isshin': ('Full', ''),
    'tatyova': ('Full', 'landfall: 1 life and a card'),
    'marwyn': ('Full', ''),
    'aurelia': ('Full', ''),
    'atraxa_pv': ('Full', 'proliferate at end step'),
    'gaa': ('Full', ''),
    'sythis':            ('Full-auto', 'enchantment cast: gain 1, draw'),
    'brago': ('Full', 'combat damage blinks every permanent with a useful ETB'),
    'lathril_tap10': ('Full', 'tap ten Elves: drain 10'),
    'kinnan': ('Full', ''),
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


def card_status(name, mine=False):
    """(status, note) for one card as an outside deck plays it; mine=True: as one of your decks plays it (your decks'
    AI acts on the hand tags in carddb.py)"""
    global _AUDIT
    import engine, scryfall, autotag
    if name in CARD_NOTES: return CARD_NOTES[name]
    import pool_cards
    if name in pool_cards.NOTES: return pool_cards.NOTES[name]
    if _AUDIT is None: _AUDIT = _main_audit()
    cd = engine.DB[name]
    if name in ('Plains', 'Island', 'Swamp', 'Mountain', 'Forest', 'Wastes'): return 'Full', 'basic land'
    rec = scryfall.fetch([name], verbose=False).get(name) or {}
    kws = {k.lower() for k in rec.get('keywords') or []} & UNSUPPORTED_KW
    kwnote = ('keywords not modeled: ' + ', '.join(sorted(kws))) if kws else ''
    if cd.source == 'manual':
        bad = deck_only(cd)
        if bad and not mine:
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
    L = ['| Tier | Deck | ' + ' | '.join(STATUSES) + ' | Engines: full / approx. / partial / unmodeled |',
         '|' + '---|' * (len(STATUSES) + 3)]
    for d, per, cnt, eng, _ in rows:
        e = Counter('full' if RANK[s] >= RANK['Full-auto'] else s.lower() for _, s, _ in eng)
        L.append(f'| {d.tier[1]} | {__import__("pools").short_name(d)} | ' + ' | '.join(str(cnt.get(s, 0)) for s in STATUSES)
                 + f" | {e['full']} / {e['approximate']} / {e['partial']} / {e['unmodeled']} |")
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


def audit_mine(verbose=True):
    """your decks (decklists/mine/): every card that is not Full, as your decks play it"""
    import pools
    pools.register()
    from decks import DECKS
    out = {}
    for k, cards in DECKS.items():
        per = {n: card_status(n, mine=True) for n in sorted(set(cards))}
        out[k] = per
        if verbose:
            bad = [(s, n, note) for n, (s, note) in per.items() if RANK[s] < RANK['Full-auto']]
            print(f'== {k}: {len(per)} unique cards, {len(bad)} not Full')
            for s, n, note in sorted(bad, key=lambda x: (RANK[x[0]], x[1])): print(f'  {s:11s} {n[:40]:40s} {note[:120]}')
    return out


def main():
    args = sys.argv[1:]
    if '--mine' in args:
        audit_mine(); return
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
