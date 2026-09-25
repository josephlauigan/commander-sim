from engine import DB
def load(path):
    txt=open(path).read(); block=txt.split('## Import list')[1].split('```')[1]
    out=[]
    for line in block.strip().splitlines():
        n,name=line.split(' ',1); out+= [name.strip()]*int(n)
    return out
import os
# Your deck .md files are looked for in: $SIM_DECKS, then decklists/mine/ next to this script, then this script's
# folder, then /mnt/project (Claude's project copy). The outside decks live in decklists/pool/ (pools.py).
_here = os.path.dirname(os.path.abspath(__file__))
_cands = [os.environ.get('SIM_DECKS', ''), os.path.join(_here, 'decklists', 'mine'), _here, '/mnt/project']
P = next(d for d in _cands if d and os.path.exists(os.path.join(d, 'sephiroth-phyrexian-reanimator.md'))) + os.sep
DECKS={'seph':load(P+'sephiroth-phyrexian-reanimator.md'),'veyran':load(P+'veyran-izzet-spellslinger.md'),
       'sauron':load(P+'sauron-grixis-amass.md')}
SWAPS=[('Stinkweed Imp','Eternal Witness'),('Phyrexian Metamorph','Heroic Intervention'),('Persist','Dread Return'),
       ('Mind Stone','Counterspell'),("Yawgmoth's Will",'Mnemonic Wall'),('Sheoldred, the Apocalypse','Rune-Scarred Demon'),
       ('Lash of the Balrog',"Assassin's Trophy"),('Evil Reawakened',"Tishana's Tidebinder"),('Swamp','Island')]
def seph_new():
    d=list(DECKS['seph'])
    for o,i in SWAPS: d.remove(o); d.append(i)
    return d
if __name__=='__main__':
    for k,v in DECKS.items():
        miss=[n for n in v if n not in DB]
        print(k,len(v),'missing:',miss)
    n=seph_new(); print('new',len(n),[x for x in n if x not in DB])


# Any listed card without hand-written tags is looked up on Scryfall and auto-tagged (cached locally).
def _ensure_all():
    import cards
    names = sorted({n for v in DECKS.values() for n in v})
    added, missing = cards.ensure_cards(names)
    if missing:
        raise SystemExit('Cards not found on Scryfall (check spelling in the deck file): ' + ', '.join(missing))


_ensure_all()
