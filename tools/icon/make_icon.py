#!/usr/bin/env python3
"""AraraCraft app icon: HP's ReelHouse hyacinth macaw mascot perched on a pixel grass block.

Writes tools/icon/icon.svg, renders it with the Chromium that Playwright ships, then writes
assets/icon/icon_{16..1024}.png and icon.ico.

    python3 tools/icon/make_icon.py
"""
import os, random, subprocess, sys, json
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
OUT = os.path.join(ROOT, "assets", "icon")

# ---- the grass block (2:1 pixel-art projection, 16x16 texels per face) -------------------
CX, TOP, W = 256, 196, 300         # top vertex, block width
HW, HH = W / 2, W / 4               # half width, half height of the top rhombus
SIDE = 158                          # height of the vertical edges
N = 16

rng = random.Random(7)
GRASS = ["#5fae3a", "#6cbd43", "#4f9c31", "#78c94c", "#589f35"]
DIRT = ["#8a5a3b", "#7a4e33", "#966545", "#6f462d", "#9d6b48"]
STONE_FLECK = "#a7a39d"


def shade(hexc, k):
    r, g, b = (int(hexc[i:i + 2], 16) for i in (1, 3, 5))
    return "#%02x%02x%02x" % tuple(max(0, min(255, int(c * k))) for c in (r, g, b))


def poly(pts, fill):
    return '<polygon points="%s" fill="%s"/>' % (" ".join("%.2f,%.2f" % p for p in pts), fill)


def top_pt(u, v):  # u along the right edge, v along the left edge, 0..1
    return (CX + (u - v) * HW, TOP + (u + v) * HH)


def left_pt(u, y):  # u from the left corner to the front corner, y down 0..1
    x0, y0 = CX - HW, TOP + HH
    return (x0 + u * HW, y0 + u * HH + y * SIDE)


def right_pt(u, y):  # u from the front corner to the right corner
    x0, y0 = CX, TOP + 2 * HH
    return (x0 + u * HW, y0 - u * HH + y * SIDE)


def face(fn, colour_at, k):
    out = []
    for j in range(N):
        for i in range(N):
            a, b = i / N, (i + 1) / N
            c, d = j / N, (j + 1) / N
            pts = [fn(a, c), fn(b, c), fn(b, d), fn(a, d)]
            out.append(poly(pts, shade(colour_at(i, j), k)))
    return out


def grass_top(i, j):
    return rng.choice(GRASS)


drip = [rng.choice([2, 3, 3, 4, 4, 5]) for _ in range(N)]


def side(i, j):
    if j < drip[i]:
        return rng.choice(GRASS)
    if rng.random() < 0.06:
        return STONE_FLECK
    return rng.choice(DIRT)


block = []
block += face(lambda u, v: top_pt(u, v), grass_top, 1.0)
block += face(lambda u, y: left_pt(u, y), side, 0.86)
block += face(lambda u, y: right_pt(u, y), side, 0.68)
# crisp edge lines, a touch lighter on the top rim like a lit bevel
P = lambda pts: " ".join("%.2f,%.2f" % p for p in pts)
block.append('<polygon points="%s" fill="none" stroke="#1c2a12" stroke-width="5" stroke-linejoin="round"/>'
             % P([top_pt(0, 0), top_pt(1, 0), right_pt(1, 1), right_pt(0, 1), left_pt(0, 1), left_pt(0, 0)]))
block.append('<polyline points="%s" fill="none" stroke="#1c2a12" stroke-width="3" opacity="0.55"/>'
             % P([left_pt(0, 0), top_pt(1, 1), top_pt(1, 0)]))
block.append('<line x1="%.2f" y1="%.2f" x2="%.2f" y2="%.2f" stroke="#1c2a12" stroke-width="3" opacity="0.55"/>'
             % (*top_pt(1, 1), *right_pt(0, 1)))
block.append('<polyline points="%s" fill="none" stroke="#b9ef8a" stroke-width="3" opacity="0.7"/>'
             % P([top_pt(0, 0.04), top_pt(0.04, 0.04), top_pt(0.96, 0.04)]))

# ---- the macaw: HP's ReelHouse mascot (top hat, medal, sleepy eyes, open beak), waving -------
BLUE, BLUE_D, BLUE_DD, BLUE_L, BLUE_LL = "#2f6be3", "#2152c2", "#14307a", "#5c95f2", "#8fb9ff"
YEL, YEL_D = "#ffd23f", "#e0a923"
BEAK, BEAK_L, BEAK_D = "#454b57", "#6b7282", "#2a2e37"
GREEN = "#2fa84f"
OUTLINE = 'stroke="%s" stroke-width="6" stroke-linejoin="round" stroke-linecap="round"' % BLUE_DD


def eye(cx, cy):
    return f"""
<circle cx="{cx}" cy="{cy}" r="31" fill="{YEL}" {OUTLINE}/>
<circle cx="{cx}" cy="{cy}" r="22" fill="#ffffff"/>
<circle cx="{cx}" cy="{cy + 5}" r="12" fill="#151922"/>
<circle cx="{cx + 4}" cy="{cy + 1}" r="4" fill="#ffffff"/>
<path d="M{cx - 23} {cy + 1} A 23 23 0 0 1 {cx + 23} {cy + 1} Z" fill="{BLUE}"/>
<path d="M{cx - 23} {cy + 1} L {cx + 23} {cy + 1}" stroke="{BLUE_DD}" stroke-width="5" stroke-linecap="round"/>
<path d="M{cx - 16} {cy - 40} Q {cx} {cy - 46}, {cx + 16} {cy - 40}" fill="none" stroke="{BLUE_DD}" stroke-width="5" stroke-linecap="round"/>
"""


bird = f"""
<g transform="translate(256 272) scale(0.8) translate(-256 -304)">
<!-- tail -->
<path d="M292 262 C 330 290, 372 318, 404 344 C 392 356, 378 356, 366 350 C 336 330, 306 306, 284 286 Z" fill="{BLUE_D}" {OUTLINE}/>
<path d="M300 280 C 330 302, 356 322, 380 340" fill="none" stroke="{BLUE}" stroke-width="6" stroke-linecap="round"/>
<!-- head tufts -->
<path d="M214 46 C 204 26, 202 8, 206 -6 C 220 6, 228 24, 230 40 Z" fill="{BLUE}" {OUTLINE}/>
<path d="M234 38 C 232 18, 236 2, 244 -8 C 254 6, 256 22, 252 36 Z" fill="{BLUE}" {OUTLINE}/>
<!-- left wing, resting -->
<path d="M190 212 C 162 236, 152 276, 160 306 C 170 300, 176 300, 182 304 C 186 296, 194 294, 200 296 C 206 268, 206 236, 190 212 Z" fill="{BLUE_D}" {OUTLINE}/>
<!-- body -->
<ellipse cx="256" cy="246" rx="80" ry="64" fill="{BLUE}" {OUTLINE}/>
<ellipse cx="256" cy="258" rx="52" ry="42" fill="{BLUE_L}"/>
<!-- head -->
<circle cx="256" cy="124" r="94" fill="{BLUE}" {OUTLINE}/>
<path d="M190 70 C 204 50, 228 40, 252 38" fill="none" stroke="{BLUE_LL}" stroke-width="8" stroke-linecap="round" opacity="0.6"/>
<!-- cheeks -->
<ellipse cx="180" cy="156" rx="15" ry="9" fill="#e98bd0" opacity="0.75"/>
<ellipse cx="332" cy="156" rx="15" ry="9" fill="#e98bd0" opacity="0.75"/>
{eye(214, 116)}
{eye(298, 116)}
<!-- open mouth with tongue, yellow corners -->
<path d="M226 178 C 228 222, 284 222, 286 178 Z" fill="#3b1622" {OUTLINE}/>
<ellipse cx="258" cy="206" rx="18" ry="11" fill="#ff7c9c"/>
<path d="M222 176 C 224 186, 228 192, 234 196" fill="none" stroke="{YEL}" stroke-width="7" stroke-linecap="round"/>
<path d="M290 176 C 288 186, 284 192, 278 196" fill="none" stroke="{YEL}" stroke-width="7" stroke-linecap="round"/>
<!-- beak -->
<path d="M252 136 C 282 134, 304 150, 300 172 C 296 190, 282 200, 270 206 C 274 194, 272 186, 262 182
         C 246 178, 226 182, 216 172 C 210 154, 226 138, 252 136 Z" fill="{BEAK}" {OUTLINE}/>
<path d="M232 152 C 244 146, 260 145, 274 148" fill="none" stroke="{BEAK_L}" stroke-width="6" stroke-linecap="round"/>
<!-- top hat, tilted -->
<g transform="rotate(14 300 40)">
  <ellipse cx="300" cy="44" rx="42" ry="10" fill="#1d2029" {OUTLINE}/>
  <path d="M278 44 L 280 2 C 290 -2, 312 -2, 322 2 L 324 44 Z" fill="#1d2029" {OUTLINE}/>
  <rect x="279" y="30" width="44" height="6" fill="{GREEN}"/>
  <rect x="279" y="36" width="44" height="5" fill="{YEL}"/>
  <path d="M288 8 L 288 26" stroke="#4a5060" stroke-width="5" stroke-linecap="round"/>
</g>
<!-- medal on a green and yellow ribbon -->
<path d="M226 214 L 256 270 L 286 214" fill="none" stroke="{GREEN}" stroke-width="14" stroke-linejoin="round"/>
<path d="M226 214 L 256 270 L 286 214" fill="none" stroke="{YEL}" stroke-width="5" stroke-linejoin="round"/>
<circle cx="256" cy="278" r="17" fill="{YEL}" stroke="{YEL_D}" stroke-width="5"/>
<path d="M256 268 L 259 275 L 266 275 L 260 280 L 262 287 L 256 283 L 250 287 L 252 280 L 246 275 L 253 275 Z" fill="#fff3b0"/>
<!-- raised wing, waving -->
<path d="M326 214 C 350 190, 372 160, 388 128 C 400 136, 404 148, 400 158 C 414 158, 420 172, 410 182
         C 420 190, 414 206, 398 206 C 384 226, 362 240, 340 246 Z" fill="{BLUE_D}" {OUTLINE}/>
<path d="M346 226 C 364 212, 380 194, 392 174" fill="none" stroke="{BLUE}" stroke-width="6" stroke-linecap="round"/>
<path d="M418 118 C 428 124, 432 134, 432 144" fill="none" stroke="{BLUE_L}" stroke-width="5" stroke-linecap="round"/>
<path d="M432 104 C 446 114, 452 128, 450 142" fill="none" stroke="{BLUE_L}" stroke-width="5" stroke-linecap="round"/>
<!-- feet -->
<g fill="{BEAK}" stroke="{BEAK_D}" stroke-width="4" stroke-linejoin="round">
  <path d="M220 302 c -4 8, 0 14, 7 14 c 3 -4, 9 -4, 12 0 c 7 0, 10 -7, 6 -14 Z"/>
  <path d="M267 302 c -4 8, 0 14, 7 14 c 3 -4, 9 -4, 12 0 c 7 0, 10 -7, 6 -14 Z"/>
</g>
</g>
"""

svg = ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">\n'
       + "\n".join(block) + bird + "</svg>\n")
svg_path = os.path.join(HERE, "icon.svg")
open(svg_path, "w").write(svg)

# ---- render with Playwright's Chromium ------------------------------------------------------
big = os.path.join(OUT, "icon_1024.png")
js = """
const { chromium } = require('playwright');
(async () => {
  const [svgPath, out] = process.argv.slice(1);
  const svg = require('fs').readFileSync(svgPath, 'utf8');
  const b = await chromium.launch();
  const p = await b.newPage({ viewport: { width: 512, height: 512 }, deviceScaleFactor: 2 });
  await p.setContent('<html><body style="margin:0;background:transparent">' + svg + '</body></html>');
  await p.locator('svg').screenshot({ path: out, omitBackground: true });
  await b.close();
})();
"""
subprocess.run(["node", "-e", js, svg_path, big], check=True)

img = Image.open(big).convert("RGBA")
for s in (16, 24, 32, 48, 64, 128, 256, 512):
    small = img.resize((s, s), Image.LANCZOS)
    small.save(os.path.join(OUT, "icon_%d.png" % s))
img.resize((256, 256), Image.LANCZOS).save(
    os.path.join(OUT, "icon.ico"), sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
print("wrote", OUT)
