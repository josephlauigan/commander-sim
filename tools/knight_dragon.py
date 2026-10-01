"""Draws the practice table's loading animation: an 8-bit knight raising a shield against a fire-breathing dragon,
under the five mana colours. Run once; the output is committed:

    python3 tools/knight_dragon.py        # writes commander_sim/play/static/knight-dragon.gif

Needs Pillow (only this tool does; the game doesn't).
"""
import math
import os
from PIL import Image, ImageDraw

W, H, SCALE, FRAMES = 128, 72, 4, 12
OUT = os.path.join(os.path.dirname(__file__), '..', 'commander_sim', 'play', 'static', 'knight-dragon.gif')

SKY, STAR, GROUND, GRASS = (18, 28, 52), (200, 210, 255), (52, 38, 30), (40, 92, 52)
STEEL, STEEL_D, VISOR, PLUME = (176, 184, 196), (112, 120, 136), (30, 30, 40), (200, 40, 48)
SHIELD, SHIELD_D, EMBLEM, SWORD = (40, 88, 168), (26, 58, 116), (232, 196, 106), (220, 224, 232)
DRAGON, DRAGON_D, BELLY, WING, EYE = (176, 36, 36), (116, 20, 24), (232, 150, 70), (140, 28, 40), (255, 230, 90)
FIRE = [(255, 244, 160), (255, 196, 60), (244, 110, 30), (200, 50, 20)]
MANA = [(248, 240, 200), (80, 140, 230), (110, 80, 130), (230, 70, 50), (70, 170, 90)]   # W U B R G
STARS = [(7, 5), (22, 14), (41, 4), (58, 11), (77, 6), (96, 15), (113, 8), (120, 20), (12, 24), (66, 22)]


def px(d, x, y, c):
    d.point((x, y), fill=c)


def rect(d, x0, y0, x1, y1, c):
    d.rectangle((x0, y0, x1, y1), fill=c)


def knight(d, f):
    bx, by = 22, 42                                         # feet at (bx, by + 20)
    brace = 1 if 4 <= f % FRAMES <= 10 else 0               # he leans into the shield while the fire hits
    rect(d, bx + 2, by + 14, bx + 4, by + 20, STEEL_D)      # legs
    rect(d, bx + 7, by + 14, bx + 9, by + 20, STEEL_D)
    rect(d, bx + 1, by + 19, bx + 4, by + 20, VISOR); rect(d, bx + 7, by + 19, bx + 10, by + 20, VISOR)
    rect(d, bx + 1, by + 6, bx + 10, by + 14, STEEL)        # body
    rect(d, bx + 1, by + 12, bx + 10, by + 13, STEEL_D)     # belt
    rect(d, bx + 3, by, bx + 8, by + 5, STEEL)              # helmet
    rect(d, bx + 5, by + 2, bx + 8, by + 3, VISOR)          # visor slit
    rect(d, bx + 4, by - 3, bx + 5, by - 1, PLUME); px(d, bx + 3, by - 2, PLUME); px(d, bx + 2, by - 1, PLUME)
    rect(d, bx - 3, by + 4, bx - 2, by + 12, STEEL_D)       # sword arm, sword raised behind
    rect(d, bx - 3, by - 8, bx - 2, by + 3, SWORD); rect(d, bx - 5, by + 3, bx, by + 3, EMBLEM)
    sx = bx + 10 + brace                                    # the shield, held towards the dragon
    rect(d, sx, by + 2, sx + 5, by + 14, SHIELD); rect(d, sx + 4, by + 2, sx + 5, by + 14, SHIELD_D)
    rect(d, sx + 1, by + 15, sx + 4, by + 15, SHIELD); px(d, sx + 2, by + 16, SHIELD)
    rect(d, sx + 2, by + 5, sx + 3, by + 11, EMBLEM); rect(d, sx + 1, by + 7, sx + 4, by + 8, EMBLEM)
    return sx + 6, by + 8                                   # where the fire lands


def dragon(d, f):
    bob = round(2 * math.sin(2 * math.pi * f / FRAMES))
    x, y = 84, 20 + bob
    up = (f // 2) % 2 == 0                                  # wings flap every other frame
    if up:                                                  # wings
        d.polygon([(x + 10, y + 6), (x + 26, y - 12), (x + 30, y - 4), (x + 22, y + 8)], fill=WING)
        d.polygon([(x + 12, y + 6), (x + 16, y - 14), (x + 22, y - 6), (x + 18, y + 8)], fill=DRAGON_D)
    else:
        d.polygon([(x + 10, y + 5), (x + 32, y + 8), (x + 36, y + 2), (x + 22, y + 3)], fill=WING)
        d.polygon([(x + 12, y + 5), (x + 26, y + 12), (x + 30, y + 6), (x + 18, y + 3)], fill=DRAGON_D)
    rect(d, x + 6, y + 4, x + 26, y + 13, DRAGON)           # body
    rect(d, x + 8, y + 11, x + 24, y + 13, BELLY)
    d.polygon([(x + 26, y + 6), (x + 38, y + 14), (x + 40, y + 20), (x + 34, y + 14), (x + 26, y + 12)], fill=DRAGON)   # tail
    px(d, x + 40, y + 21, DRAGON_D); px(d, x + 41, y + 20, DRAGON_D)
    rect(d, x + 1, y, x + 8, y + 7, DRAGON)                 # neck and head
    rect(d, x - 6, y - 2, x + 2, y + 4, DRAGON)
    rect(d, x - 8, y + 1, x - 4, y + 4, DRAGON_D)           # jaw
    px(d, x - 2, y - 1, EYE)
    px(d, x - 1, y - 4, DRAGON_D); px(d, x + 1, y - 4, DRAGON_D); px(d, x, y - 3, DRAGON_D)   # horns
    rect(d, x + 10, y + 13, x + 12, y + 17, DRAGON_D); rect(d, x + 20, y + 13, x + 22, y + 17, DRAGON_D)  # claws
    return x - 9, y + 2                                     # the mouth


def fire(d, f, mouth, target):
    phase = f % FRAMES
    if phase < 3: return                                    # drawing breath
    reach = min(1.0, (phase - 2) / 4)
    (mx, my), (tx, ty) = mouth, target
    n = int(28 * reach)
    for i in range(n):
        t = i / 27
        cx = mx + (tx - mx) * t
        cy = my + (ty - my) * t + math.sin(t * 9 + f) * 1.5
        r = 1 + t * 3
        col = FIRE[min(3, int(t * 3 + (i + f) % 2))]
        d.ellipse((cx - r, cy - r, cx + r, cy + r), fill=col)
    if reach >= 1.0:                                        # sparks off the shield
        for k in range(6):
            a = (k * 60 + f * 23) % 360
            sx = tx + 2 + round(4 * math.cos(math.radians(a))); sy = ty + round(6 * math.sin(math.radians(a)))
            px(d, sx, sy, FIRE[k % 2])


def mana(d, f):
    for i, c in enumerate(MANA):
        x = 40 + i * 12
        y = 6 + round(1.5 * math.sin(2 * math.pi * (f / FRAMES) + i))
        d.ellipse((x - 3, y - 3, x + 3, y + 3), fill=c, outline=(20, 20, 20))
        px(d, x - 1, y - 1, (255, 255, 255))


def frame(f):
    im = Image.new('RGB', (W, H), SKY)
    d = ImageDraw.Draw(im)
    for i, (x, y) in enumerate(STARS):
        if (f + i) % 5: px(d, x, y, STAR)                    # twinkle
    rect(d, 0, H - 10, W, H, GROUND); rect(d, 0, H - 10, W, H - 9, GRASS)
    mana(d, f)
    target = knight(d, f)
    mouth = dragon(d, f)
    fire(d, f, mouth, target)
    return im.resize((W * SCALE, H * SCALE), Image.NEAREST)


def main():
    frames = [frame(f) for f in range(FRAMES)]
    frames[0].save(OUT, save_all=True, append_images=frames[1:], duration=110, loop=0, optimize=True)
    print(f'wrote {os.path.normpath(OUT)} ({W * SCALE}x{H * SCALE}, {FRAMES} frames)')


if __name__ == '__main__':
    main()
