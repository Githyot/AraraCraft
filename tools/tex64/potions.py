"""Potion icons: glass you can see into, liquid with a surface, a shine and a few bubbles
(HP 2026-11-01, "analise todas as poções"; plan/pocoes-analise.md).

Every potion used to be the same round bottle in another colour. Now the family shows in the
shape, so a glance at the hotbar tells what a bottle is for:

* drinkable (Minecraft's potions): a round flask with a cork;
* splash: a squat, pear-shaped bottle with a cloth wrapped round the neck to throw it by;
* lingering: a round flask with a long thin neck, a glass stopper and mist curling in the neck;
* buff potions of the Terraria side: a tall vial sealed with red wax;
* healing potions: a heart-shaped bottle, bigger with each tier (the super one has a gold cap);
* mana potions: a teardrop bottle with a star-cut stopper;
* travel and odd potions (recall, teleport, wormhole, gravity, battle, calm, fishing...):
  a cone-shaped lab flask.

Drawn on a 32x32 grid (the pixel size pixel.py gives every item) and grown to 64x64.
"""

import os
import re

import numpy as np

from style import hexrgb

G = 32
Y, X = np.mgrid[0:G, 0:G].astype(np.float64) + 0.5
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def ell(cx, cy, rx, ry):
    return ((X - cx) / rx) ** 2 + ((Y - cy) / ry) ** 2 <= 1.0


def box(x0, y0, x1, y1):
    return (X >= x0) & (X < x1) & (Y >= y0) & (Y < y1)


def c(h):
    return np.array(hexrgb(h), dtype=float) if isinstance(h, str) else np.asarray(h, dtype=float)


def shade(col, k):
    """Lighter (k > 1) or darker; darker leans cool, lighter warm, like the rest of our art."""
    col = c(col)
    if k >= 1:
        out = col + (255 - col) * (k - 1) + np.array([6, 4, -4]) * (k - 1)
    else:
        out = col * k + np.array([-4, 0, 10]) * (1 - k)
    return np.clip(out, 0, 255)


def edge_of(m):
    p = np.pad(m, 1)
    return m & ~(p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:])


def grow(m):
    p = np.pad(m, 1)
    return m | p[:-2, 1:-1] | p[2:, 1:-1] | p[1:-1, :-2] | p[1:-1, 2:]


GLASS_HI = c("#e8f6fa")
GLASS = c("#b4ccd6")
GLASS_AIR = c("#7f99a8")
GLASS_AIR_D = c("#5f7686")


class Bottle:
    def __init__(self):
        self.img = np.zeros((G, G, 4))
        self.vessel = np.zeros((G, G), bool)

    def put(self, m, col):
        self.img[m, :3] = c(col)
        self.img[m, 3] = 255

    def glass(self, body, level, liquid, empty=False, mist=None):
        """`body`: the inside of the glass; liquid below row `level`."""
        liquid = c(liquid)
        self.vessel |= body
        rim = edge_of(body)
        inner = body & ~rim
        air = inner & (Y < level)
        wet = inner & (Y >= level)
        if empty:
            air, wet = inner, np.zeros_like(inner)
        # Air in the glass: a cool grey-blue, darker to the right, tinted by what's below.
        tint = GLASS_AIR * 0.85 + liquid * 0.15 if not empty else GLASS_AIR
        x0, x1 = X[body].min(), X[body].max()
        t = (X - x0) / max(x1 - x0, 1)
        self.put(air & (t < 0.6), tint)
        self.put(air & (t >= 0.6), GLASS_AIR_D * 0.85 + liquid * 0.15 if not empty else GLASS_AIR_D)
        if mist is not None:
            self.put(air & mist, shade(liquid, 1.25))
        if wet.any():
            ys = Y[wet]
            depth = (Y - level) / max(ys.max() - level, 1)
            self.put(wet, liquid)
            self.put(wet & (t < 0.38), shade(liquid, 1.18))
            self.put(wet & ((t > 0.72) | (depth > 0.72)), shade(liquid, 0.72))
            self.put(wet & (t > 0.8) & (depth > 0.55), shade(liquid, 0.55))
            # The surface: a bright line where the liquid meets the air (the meniscus).
            surf = wet & (Y < level + 1)
            self.put(surf, shade(liquid, 1.45))
            self.put(surf & (t > 0.75), shade(liquid, 1.15))
            # Bubbles rising.
            for bx, by in ((0.55, 0.45), (0.42, 0.7), (0.62, 0.25)):
                xx = int(x0 + bx * (x1 - x0))
                yy = int(level + 2 + by * (ys.max() - level - 3))
                if 0 <= yy < G and wet[yy, xx]:
                    self.img[yy, xx, :3] = shade(liquid, 1.6)
        # The glass wall: light where the liquid is (it glows through), pale above.
        self.put(rim & (Y < level), GLASS)
        self.put(rim & (Y >= level), shade(liquid, 1.3) * 0.5 + GLASS * 0.5)
        if empty:
            self.put(rim, GLASS)
        # The shine: a tall streak on the upper left of the bulge and a dot on the right.
        ys, xs = np.nonzero(inner)
        if len(ys):
            top, bot = ys.min(), ys.max()
            left = {}
            for yy, xx in zip(ys, xs):
                left[yy] = min(left.get(yy, 99), xx)
            mid = (top + bot) // 2
            for yy in range(max(top + 1, mid - 3), min(bot, mid + 2)):
                if yy in left:
                    self.img[yy, left[yy] + 1, :3] = GLASS_HI
            right = {}
            for yy, xx in zip(ys, xs):
                right[yy] = max(right.get(yy, -1), xx)
            yy = mid + 3
            if yy in right and right[yy] - 1 >= 0:
                self.img[yy, right[yy] - 1, :3] = shade(GLASS_HI, 0.9)

    def cork(self, m, col="#9a6a3a"):
        self.put(m, col)
        self.put(m & (X >= X[m].max() - 0.5), shade(col, 0.7))
        self.put(m & (Y < Y[m].min() + 1) & (X < X[m].max() - 0.5), shade(col, 1.3))
        self.vessel |= m

    def done(self, line):
        """A crisp outline in the liquid's own dark colour, then grown to 64x64."""
        a = self.img[..., 3] > 0
        out = grow(a) & ~a
        self.put(out, line)
        big = np.repeat(np.repeat(self.img, 2, 0), 2, 1)
        return big


def outline_col(liquid):
    return shade(c(liquid) * 0.35 + c("#2a3440") * 0.65, 0.75)


# --- shapes -----------------------------------------------------------------------------------

def flask(liquid, empty=False, cork="#9a6a3a", full=0.68):
    b = Bottle()
    body = ell(16, 21, 9, 8.5) | box(13, 8, 19, 14)
    lip = box(12, 7, 20, 9)
    b.glass(body | lip, 21 - 8.5 + 17 * (1 - full) if not empty else 99, liquid, empty)
    b.cork(box(13, 3, 19, 7), cork)
    return b.done(outline_col(liquid if not empty else "#6a8494"))


def splash(liquid):
    b = Bottle()
    # Pear-shaped: wide and low, narrowing to a short neck.
    body = ell(16, 23, 10, 6.5) | (ell(16, 18, 6.5, 6) & (Y > 13)) | box(14, 9, 18, 14)
    b.glass(body, 16.5, liquid)
    # A cloth wound round the neck, its ends hanging out to the right: something to throw by.
    cloth = c("#c9b28a")
    b.put(box(13, 10, 19, 13), cloth)
    b.put(box(13, 12, 19, 13), shade(cloth, 0.75))
    b.put(box(19, 11, 21, 12) | box(20, 12, 22, 14), shade(cloth, 0.9))
    b.put(box(21, 13, 22, 15), shade(cloth, 0.7))
    b.vessel |= box(13, 10, 22, 15)
    b.cork(box(14, 6, 18, 9), "#7a5a3a")
    return b.done(outline_col(liquid))


def lingering(liquid):
    b = Bottle()
    body = ell(16, 22, 8.5, 8) | box(14, 6, 18, 15)
    # Mist curling up the long neck.
    mist = ((X >= 15) & (X < 16) & ((Y.astype(int) % 4) < 2)) | ((X >= 16) & (X < 17) & ((Y.astype(int) % 4) >= 2))
    b.glass(body, 17, liquid, mist=mist & (Y < 15))
    # A glass stopper: a small ball.
    stop = ell(16, 4, 2.6, 2.6)
    b.put(stop, GLASS)
    b.put(stop & (X > 16.5), GLASS_AIR)
    b.img[3, 15, :3] = GLASS_HI
    b.vessel |= stop
    return b.done(outline_col(liquid))


def vial(liquid):
    b = Bottle()
    body = box(11, 9, 21, 27) | ell(16, 26, 5, 3.5)
    b.glass(body, 13, liquid)
    # Red sealing wax over the top, with a drip down the side.
    wax = c("#a8242c")
    m = box(10, 5, 22, 9)
    b.put(m, wax)
    b.put(m & (Y < 6), shade(wax, 1.35))
    b.put(m & (X >= 20), shade(wax, 0.7))
    b.put(box(11, 9, 12, 12), shade(wax, 0.85))
    b.vessel |= m
    return b.done(outline_col(liquid))


def heart(liquid, tier):
    """Healing: the bottle is a heart, larger with each tier."""
    b = Bottle()
    s = [0.72, 0.85, 1.0, 1.0][tier]
    cy = 20 + (1 - s) * 6
    r = 5.0 * s
    lobes = ell(16 - r * 0.95, cy - r * 0.2, r, r * 1.05) | ell(16 + r * 0.95, cy - r * 0.2, r, r * 1.05)
    point = (np.abs(X - 16) <= (cy + r * 2.1 - Y) * 0.9) & (Y >= cy - r * 0.2) & (Y <= cy + r * 2.1)
    # The neck rises from the dip between the two lobes.
    neck_top = cy - r * 1.2 - 3
    body = lobes | point | box(15, neck_top, 17, cy - r * 0.2)
    b.glass(body, cy - r * 0.35, liquid)
    cap = "#d8b440" if tier == 3 else "#9a6a3a"
    b.cork(box(13, neck_top - 3, 19, neck_top), cap)
    if tier == 3:
        b.put(box(15, neck_top - 5, 17, neck_top - 3), "#f0d870")
        b.vessel |= box(15, neck_top - 5, 17, neck_top - 3)
    return b.done(outline_col(liquid))


def teardrop(liquid, tier):
    """Mana: a teardrop with a star-cut stopper."""
    b = Bottle()
    s = [0.75, 0.88, 1.0][tier]
    cy = 22
    r = 7.5 * s
    drop = ell(16, cy, r, r) | ((np.abs(X - 16) <= (Y - (cy - r * 2.0)) * 0.55) & (Y < cy) & (Y >= cy - r * 2.0))
    body = drop | box(14.5, cy - r * 2.0 - 2, 17.5, cy - r)
    b.glass(body, cy - r * 0.6, liquid)
    top = cy - r * 2.0 - 2
    star = (box(15, top - 5, 17, top) | box(13, top - 3.5, 19, top - 1.5))
    b.put(star, "#e8e0a0")
    b.put(star & (X >= 16.5), "#b8a860")
    b.vessel |= star
    return b.done(outline_col(liquid))


def cone(liquid):
    """A lab flask for travel and odd potions."""
    b = Bottle()
    w = lambda y: 2.0 + (y - 11) * 0.62  # noqa: E731
    body = (np.abs(X - 16) <= w(Y)) & (Y >= 11) & (Y < 28) | box(14, 6, 18, 12)
    b.glass(body, 18, liquid)
    b.cork(box(13, 3, 19, 6))
    return b.done(outline_col(liquid))


def imbue(liquid):
    """Weapon flasks: a flat, square flask with a metal cap, like a hip flask for a blade."""
    b = Bottle()
    body = box(9, 11, 23, 28) & ~(box(9, 11, 10, 12) | box(22, 11, 23, 12) | box(9, 27, 10, 28) | box(22, 27, 23, 28))
    body |= box(14, 7, 18, 11)
    b.glass(body, 15, liquid)
    cap = box(13, 3, 19, 7)
    b.put(cap, "#9aa0a8")
    b.put(cap & (X >= 17.5), "#6a7078")
    b.put(cap & (Y < 4), "#d0d6dc")
    b.vessel |= cap
    return b.done(outline_col(liquid))


def restoration():
    """Two liquids that don't mix: life below, mana floating on top."""
    b = Bottle()
    body = ell(16, 21, 9, 8.5) | box(13, 8, 19, 14)
    b.glass(body, 14, "#d02a3a")
    top = body & ~edge_of(body) & (Y >= 14) & (Y < 18)
    b.put(top, "#3a6ae0")
    b.put(top & (Y < 15), shade("#3a6ae0", 1.45))
    b.cork(box(13, 4, 19, 8))
    return b.done(outline_col("#8a3060"))


def experience():
    b = Bottle()
    body = ell(16, 21, 9, 8.5) | box(13, 8, 19, 14)
    b.glass(body, 15, "#6ad040")
    for x, y in ((13, 19), (18, 23), (16, 26), (20, 18)):
        b.img[y, x, :3] = c("#f4ffb0")
    b.cork(box(13, 4, 19, 8), "#6a8a3a")
    return b.done(outline_col("#6ad040"))


# --- which shape for which item ---------------------------------------------------------------

def effect_colors():
    text = open(os.path.join(ROOT, "crates", "game", "src", "effects.rs")).read()
    consts = dict((n, int(v)) for n, v in re.findall(r"pub const ([A-Z_]+): u8 = (\d+);", text))
    colors = {}
    for name, rgb in re.findall(r"([A-Z_]+) => \(\"[^\"]+\", \[(\d+, \d+, \d+)\]", text):
        if name in consts:
            colors[consts[name]] = [int(v) for v in rgb.split(",")]
    potions = []
    for key, body in re.findall(r'p!\("([a-z_]+)", "[^"]+", \[([^\]]*)\]\)', text):
        effs = re.findall(r"\(([A-Z_]+), (\d+), [\d.]+\)", body)
        potions.append((key, [consts[e] for e, _ in effs]))
    return colors, potions


# Potions of the Terraria side (brewed from an awkward potion, stations.rs): their own vial.
TERRARIA_BUFFS = {
    "ironskin", "mana_regeneration", "magic_power", "archery", "wrath", "rage", "endurance", "lifeforce", "thorns", "titan",
    "summoning", "spelunker", "hunter", "shine", "builder", "warmth", "inferno", "mining", "heartreach", "dangersense",
    "water_walking", "flipper", "ammo_reservation", "biome_sight",
}


def liquid_of(effs, colors):
    if not effs:
        return [70, 110, 210]
    col = np.mean([colors[e] for e in effs], axis=0)
    # Keep every liquid rich: a pale mix gets more colour, a near-black one some light.
    lum = col @ np.array([0.3, 0.59, 0.11])
    if lum > 180:
        col = col * 0.8
    if lum < 50:
        col = col + 40
    return list(np.clip(col, 0, 255))


REGISTRY = {}


def register():
    colors, potions = effect_colors()
    for key, effs in potions:
        col = liquid_of(effs, colors)
        stem = key.removeprefix("long_").removeprefix("strong_")
        if key == "water":
            col = [60, 110, 230]
        if key in ("mundane", "thick", "awkward"):
            col = {"mundane": [110, 130, 200], "thick": [150, 140, 210], "awkward": [80, 100, 190]}[key]
        if stem.startswith("flask_"):
            drink = lambda col=col: imbue(col)  # noqa: E731
        elif stem in TERRARIA_BUFFS:
            drink = lambda col=col: vial(col)  # noqa: E731
        else:
            drink = lambda col=col: flask(col)  # noqa: E731
        REGISTRY[f"potion_{key}"] = drink
        REGISTRY[f"splash_potion_{key}"] = lambda col=col: splash(col)
        REGISTRY[f"lingering_potion_{key}"] = lambda col=col: lingering(col)
    REGISTRY["glass_bottle"] = lambda: flask("#7f99a8", empty=True)
    REGISTRY["honey_bottle"] = lambda: flask("#e0a020", full=0.8)
    REGISTRY["dragon_breath"] = lambda: lingering("#d070c0")
    REGISTRY["ominous_bottle"] = lambda: lingering("#1a7a58")
    REGISTRY["experience_bottle"] = experience
    heal = "#d8243a"
    for t, n in enumerate(["lesser_healing_potion", "healing_potion", "greater_healing_potion", "super_healing_potion"]):
        REGISTRY[n] = lambda t=t: heart("#e02a6a" if t == 3 else heal, t)
    for t, n in enumerate(["lesser_mana_potion", "mana_potion", "greater_mana_potion"]):
        REGISTRY[n] = lambda t=t: teardrop("#2a5ae0", t)
    REGISTRY["super_mana_potion"] = lambda: teardrop("#7a3ae8", 2)
    REGISTRY["restoration_potion"] = restoration
    for n, col in {
        "recall_potion": "#3a8ad0", "teleport_potion": "#b04ad4", "wormhole_potion": "#2aa898", "gravity_potion": "#7a6ad8",
        "battle_potion": "#c8302a", "calming_potion": "#4a8ad8", "crate_potion": "#c89a20", "fishing_potion": "#2a8ad0",
        "sonar_potion": "#2ab098", "return_potion": "#5ac0e0",
    }.items():
        REGISTRY[n] = lambda col=col: cone(col)


register()
