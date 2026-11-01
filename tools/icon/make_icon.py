#!/usr/bin/env python3
"""AraraCraft app icon: our own hyacinth macaw mascot perched on a pixel grass block.

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

# ---- the macaw (flat, round, friendly) -----------------------------------------------------
BLUE, BLUE_D, BLUE_DD, BLUE_L, BLUE_LL = "#2c6be0", "#1f50b5", "#173c8c", "#5b9cf5", "#9cc8ff"
YEL, YEL_D = "#ffd23f", "#f0a92a"
BEAK, BEAK_L, BEAK_D = "#3a3f4b", "#5d6474", "#22252d"
OUTLINE = 'stroke="%s" stroke-width="7" stroke-linejoin="round"' % BLUE_DD

bird = f"""
<g transform="translate(256 272) scale(0.8) translate(-256 -304)">
<!-- tail: two long feathers draped over the front of the block -->
<path d="M268 292 C 300 330, 318 380, 312 452 C 300 440, 290 400, 262 300 Z" fill="{BLUE_D}" {OUTLINE}/>
<path d="M248 292 C 280 340, 286 400, 270 470 C 256 452, 250 400, 236 300 Z" fill="{BLUE}" {OUTLINE}/>
<path d="M256 330 C 268 370, 270 410, 266 450" fill="none" stroke="{BLUE_L}" stroke-width="5" stroke-linecap="round"/>

<!-- folded wing (left) -->
<path d="M178 190 C 140 220, 140 280, 186 304 C 196 270, 200 230, 178 190 Z" fill="{BLUE_D}" {OUTLINE}/>

<!-- body and head as one blob -->
<path d="M256 34 C 330 34, 352 92, 344 140 C 360 180, 358 250, 322 286
         C 296 308, 216 308, 190 286 C 154 250, 152 180, 168 140 C 160 92, 182 34, 256 34 Z"
      fill="{BLUE}" {OUTLINE}/>
<!-- crest -->
<path d="M232 44 C 222 18, 238 6, 252 14 C 256 0, 280 2, 278 24 C 292 18, 304 30, 290 46 Z"
      fill="{BLUE}" {OUTLINE}/>
<!-- belly and a soft top light -->
<ellipse cx="256" cy="236" rx="60" ry="56" fill="{BLUE_L}"/>
<path d="M214 248 C 226 268, 286 268, 298 248" fill="none" stroke="{BLUE_LL}" stroke-width="5" stroke-linecap="round" opacity="0.8"/>
<path d="M196 92 C 204 62, 232 48, 258 48" fill="none" stroke="{BLUE_LL}" stroke-width="9" stroke-linecap="round" opacity="0.7"/>

<!-- raised wing (right), waving -->
<path d="M330 200 C 360 168, 384 128, 396 84 C 410 96, 414 112, 408 126 C 424 126, 430 144, 418 156
         C 430 164, 426 184, 408 186 C 392 214, 366 236, 340 244 Z" fill="{BLUE_D}" {OUTLINE}/>
<path d="M352 214 C 372 196, 390 172, 400 150" fill="none" stroke="{BLUE}" stroke-width="6" stroke-linecap="round"/>

<!-- eyes: yellow bare skin rings, big friendly pupils -->
<g {OUTLINE}>
  <circle cx="218" cy="112" r="40" fill="{YEL}"/>
  <circle cx="294" cy="112" r="40" fill="{YEL}"/>
</g>
<circle cx="220" cy="112" r="29" fill="#ffffff"/>
<circle cx="292" cy="112" r="29" fill="#ffffff"/>
<circle cx="228" cy="118" r="17" fill="#141821"/>
<circle cx="284" cy="118" r="17" fill="#141821"/>
<circle cx="234" cy="110" r="6" fill="#ffffff"/>
<circle cx="290" cy="110" r="6" fill="#ffffff"/>

<!-- beak: a big dark hook with the yellow strip of the lower jaw peeking out on both sides -->
<path d="M204 186 C 214 226, 298 226, 308 186 C 304 236, 276 246, 256 246 C 236 246, 208 236, 204 186 Z"
      fill="{BEAK_D}" {OUTLINE}/>
<path d="M206 190 C 218 222, 294 222, 306 190" fill="none" stroke="{YEL}" stroke-width="10" stroke-linecap="round"/>
<path d="M256 138 C 288 138, 304 156, 300 180 C 296 202, 278 216, 264 230 C 259 236, 253 236, 248 230
         C 234 216, 216 202, 212 180 C 208 156, 224 138, 256 138 Z" fill="{BEAK}" {OUTLINE}/>
<path d="M230 158 C 240 150, 254 148, 266 150" fill="none" stroke="{BEAK_L}" stroke-width="7" stroke-linecap="round"/>
<path d="M256 196 C 258 208, 258 216, 256 224" fill="none" stroke="{BEAK_D}" stroke-width="4" stroke-linecap="round" opacity="0.7"/>

<!-- feet gripping the block -->
<g fill="{BEAK}" stroke="{BEAK_D}" stroke-width="4" stroke-linejoin="round">
  <path d="M220 294 c -4 8, 0 14, 7 14 c 3 -4, 9 -4, 12 0 c 7 0, 10 -7, 6 -14 Z"/>
  <path d="M267 294 c -4 8, 0 14, 7 14 c 3 -4, 9 -4, 12 0 c 7 0, 10 -7, 6 -14 Z"/>
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
