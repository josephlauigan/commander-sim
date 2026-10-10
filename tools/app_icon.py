"""Draws the iPad app's icon in the loading animation's 8-bit style: the knight's shield, under the five mana colours,
on the page's true black. Run once; the output is committed:

    python3 tools/app_icon.py        # writes ios/icons/commander_ipad-<size>.png (every size Briefcase asks for)

Needs Pillow (only this tool does; the game doesn't).
"""
import os
from PIL import Image, ImageDraw

from knight_dragon import MANA, SWORD, px, rect

N = 32                                                      # the pixel grid; scaled up without smoothing
OUT = os.path.join(os.path.dirname(__file__), '..', 'ios', 'icons')
SIZES = (20, 29, 40, 58, 60, 76, 80, 87, 120, 152, 167, 180, 256, 512, 640, 1024)
BG, RIM = (10, 10, 10), (222, 168, 96)                     # the page's true black and orange highlight
SHIELD, SHIELD_D, EMBLEM = (101, 150, 189), (79, 126, 166), (222, 168, 96)   # its blue


def draw():
    im = Image.new('RGB', (N, N), BG)                       # iOS icons are opaque; iOS rounds the corners itself
    d = ImageDraw.Draw(im)
    for i, c in enumerate(MANA):                            # the five mana colours in an arc
        x, y = 6 + i * 5, (6, 4, 3, 4, 6)[i]
        rect(d, x - 1, y - 1, x + 1, y + 1, c); px(d, x - 1, y - 1, (255, 255, 255))
    x0, x1, y0, ys = 8, 23, 10, 19                          # the shield: straight sides to ys, then a point

    def rows(inset):                                        # (y, left, right)
        out = [(y, x0 + inset, x1 - inset) for y in range(y0 + inset, ys + 1)]
        out += [(ys + k, x0 + k + inset, x1 - k - inset) for k in range(1, 8)]
        return [r for r in out if r[1] <= r[2]]
    for y, a, b in rows(0): rect(d, a, y, b, y, RIM)        # gold rim
    for y, a, b in rows(1):                                 # blue field, the right half in shadow
        rect(d, a, y, b, y, SHIELD)
        if b >= 16: rect(d, max(a, 16), y, b, y, SHIELD_D)
    rect(d, 14, 12, 17, 12, EMBLEM)                         # the emblem: a sword, point down (pommel, grip, guard, blade)
    rect(d, 15, 13, 16, 13, SHIELD_D); px(d, 15, 13, EMBLEM)
    rect(d, 12, 14, 19, 14, EMBLEM)
    rect(d, 15, 15, 16, 22, SWORD); rect(d, 16, 15, 16, 22, (170, 176, 190))
    px(d, 15, 23, SWORD)
    return im


def main():
    os.makedirs(OUT, exist_ok=True)
    im = draw()
    for s in SIZES:
        im.resize((s, s), Image.NEAREST if s % N == 0 else Image.BOX).save(os.path.join(OUT, f'commander_ipad-{s}.png'))
    print(f'wrote {len(SIZES)} icons to {os.path.normpath(OUT)}')


if __name__ == '__main__':
    main()
