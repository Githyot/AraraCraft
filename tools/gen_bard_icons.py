"""16x16 designs of the bard's accessories (crates/game/src/bard.rs).

Writes packs/vanilla/textures/item/NAME.png; then run
    python3 tools/tex64/build.py --only metronome,bard_songbook,tuning_fork,echo_shell,conductor_baton
to grow them to 64x64 in the house style.
"""
from pathlib import Path

from PIL import Image

OUT = Path(__file__).resolve().parent.parent / "packs/vanilla/textures/item"

ICONS = {
    # A wooden pyramid with a brass pendulum and weight.
    "metronome": (
        {
            "o": (58, 34, 20), "w": (128, 78, 42), "W": (164, 104, 58), "l": (196, 136, 80),
            "b": (150, 112, 40), "B": (212, 170, 70), "Y": (246, 214, 120), "d": (40, 30, 24),
        },
        [
            "................",
            ".......oo.......",
            "......oBYo......",
            "......owBo......",
            ".....owBWlo.....",
            ".....owBWlo.....",
            "....owwBbWlo....",
            "....owWBWWlo....",
            "...owwYYYWWlo...",
            "...owwYBbWWlo...",
            "..owwwWWWWWllo..",
            "..owdddddddllo..",
            ".owwwWWWWWWWllo.",
            ".oooooooooooooo.",
            "................",
            "................",
        ],
    ),
    # A red leather book with a gold clef on the cover and a ribbon.
    "bard_songbook": (
        {
            "o": (54, 18, 20), "r": (126, 32, 36), "R": (170, 50, 50), "L": (204, 84, 70),
            "p": (232, 220, 190), "P": (250, 242, 220), "g": (190, 146, 50), "G": (240, 200, 90),
        },
        [
            "................",
            "...oooooooooo...",
            "..oRRRRRRRRRLo..",
            "..oRRRRGRRRRLo..",
            "..oRRRGRGRRRLo..",
            "..oRRRGGRRRRLo..",
            "..oRRRRGRRRRLo..",
            "..oRRRGGGRRRLo..",
            "..oRRGRGRGRRLo..",
            "..oRRRGGGRRRLo..",
            "..oRRRRGRRRRLo..",
            "..oRRRGGRRRRLo..",
            "..orrrrrrrrrro..",
            "..oPpPpPpPpPpo..",
            "...ooooooooRo...",
            "...........R....",
        ],
    ),
    # A steel tuning fork, two tall prongs on a stem.
    "tuning_fork": (
        {
            "o": (44, 50, 64), "s": (120, 130, 150), "S": (176, 186, 204), "H": (232, 238, 248),
        },
        [
            "................",
            "....oo....oo....",
            "...oHSo..oHSo...",
            "...oHSo..oHSo...",
            "...oHSo..oHSo...",
            "...oHSo..oHSo...",
            "...oHSo..oHSo...",
            "...oHSo..oHSo...",
            "...oHSoooosSo...",
            "....oSSSSSSo....",
            ".....oossoo.....",
            ".......oSo......",
            ".......oSo......",
            ".......oSo......",
            ".......oso......",
            "........o.......",
        ],
    ),
    # A spiral shell, pearly inside, tied with a string.
    "echo_shell": (
        {
            "o": (70, 46, 50), "c": (196, 150, 126), "C": (226, 188, 160), "L": (246, 222, 200),
            "p": (226, 170, 186), "P": (246, 210, 220), "s": (220, 220, 220),
        },
        [
            "................",
            "................",
            ".....oooooo.....",
            "....oCCLLCCo....",
            "...oCcooocCCo...",
            "..oCcoPPPocCo...",
            "..oCopPooPocCo..",
            "..oCopPoPPocCo..",
            "..oCcopPPpoCCo..",
            "...oCcooooCCo...",
            "....oCCccCCo....",
            ".....oCCCCo.....",
            "......oooo.s....",
            "...........s....",
            "..........s.....",
            "................",
        ],
    ),
    # A dark wooden baton with a gold star at its tip and a gold grip.
    "conductor_baton": (
        {
            "o": (40, 26, 22), "e": (70, 46, 36), "E": (110, 74, 52), "g": (180, 132, 40),
            "G": (236, 196, 84), "Y": (255, 236, 150),
        },
        [
            "............G...",
            "...........GYG..",
            "..........GYYYG.",
            "...........GYG..",
            "..........oEG...",
            ".........oEo....",
            "........oEo.....",
            ".......oEo......",
            "......oEo.......",
            ".....oEo........",
            "....ogGo........",
            "...ogGgo........",
            "..ogGgo.........",
            "..ogeo..........",
            "...oo...........",
            "................",
        ],
    ),
    # A golden frame harp: pillar on the left, curved neck, slanting soundbox, strings.
    "golden_harp": (
        {"o": (70, 44, 14), "g": (188, 140, 40), "G": (246, 206, 100), "w": (120, 54, 26), "W": (160, 78, 36), "s": (236, 228, 200)},
        [
            "..oGGo..........",
            "..oGGGGoo.......",
            "..oGo.oGGGoo....",
            "..oGos.s.oGGGo..",
            "..oGos.s.s.ooGo.",
            "..oGos.s.s.s.oGo",
            "..oGos.s.s.s.oWo",
            "..oGos.s.s.sowWo",
            "..oGos.s.s.oWWo.",
            "..oGos.s.soWWo..",
            "..oGos.s.oWWo...",
            "..oGos.soWWo....",
            "..oGosoWWWo.....",
            "..oGooWWWo......",
            ".oGGGGGgo.......",
            ".ooooooo........",
        ],
    ),
    # A curved war horn bound in brass, bell up at the right.
    "war_horn": (
        {"o": (60, 40, 22), "h": (206, 182, 132), "H": (236, 220, 180), "d": (150, 112, 70), "b": (190, 140, 50), "B": (236, 196, 96), "k": (30, 20, 14)},
        [
            "...........oooo.",
            "..........okkkko",
            "..........oHHHho",
            "..........ohHhdo",
            ".........obBBbo.",
            ".........ohHhdo.",
            "........ohHhdo..",
            ".......ohHhdo...",
            "......obBBbo....",
            ".....ohHhdo.....",
            "...oohHhdo......",
            ".oohHhhdo.......",
            "obBbhddo........",
            "obBboo..........",
            ".oo.............",
            "................",
        ],
    ),
    # A crystal violin with its bow across.
    "crystal_violin": (
        {"o": (40, 60, 90), "c": (130, 180, 222), "C": (190, 228, 250), "e": (30, 22, 18), "s": (220, 224, 232), "h": (236, 232, 220)},
        [
            ".............oo.",
            "............oCo.",
            "...........oeo..",
            "..........oeo...",
            ".........oeo....",
            "....oooooeo.....",
            "...oCCCCeCo.....",
            "..oCcCceCcCo....",
            "..ocCcecCcco....",
            "...oCCeCCCo.....",
            "..oCcecCcCCo....",
            ".sscCCcCcCcos...",
            "..oCcCCcCCo.hs..",
            "...oooooooo...s.",
            "...............s",
            "................",
        ],
    ),
    # A red lacquered war drum with a rope and a stick.
    "war_drum": (
        {"o": (60, 14, 12), "r": (150, 30, 26), "R": (196, 52, 40), "k": (232, 214, 176), "K": (210, 188, 144), "y": (210, 180, 110), "b": (236, 196, 96), "w": (110, 70, 40)},
        [
            "............ww..",
            "...........ww...",
            "...oooooooww....",
            "..okkkkkkkwko...",
            ".okKkkkkkkkKko..",
            ".obkkkkkkkkkbo..",
            ".oRRRRRRRRRRRo..",
            ".oRrRRRRRRRrRo..",
            ".oyyyyyyyyyyyo..",
            ".oRrRRRRRRRrRo..",
            ".oRRRRRRRRRRRo..",
            ".obkkkkkkkkkbo..",
            "..okkkkkkkkko...",
            "...ooooooooo....",
            "................",
            "................",
        ],
    ),
    # A portative organ: keys at the bottom, silver pipes rising behind.
    "pipe_organ": (
        {"o": (40, 30, 30), "s": (150, 156, 168), "S": (226, 232, 240), "k": (20, 18, 22), "r": (130, 30, 26), "i": (236, 230, 214)},
        [
            "................",
            "..........oo....",
            "........oooSo...",
            "......oooSoSo...",
            "....oooSoSoSoo..",
            "..oooSoSoSoSoSo.",
            "..oSoSoSoSoSoSo.",
            "..oSoSoSoSoSoSo.",
            "..oSoSoSoSoSoSo.",
            "..ooooooooooooo.",
            "..orrrrrrrrrrro.",
            "..oikikiikikiio.",
            "..oikikiikikiio.",
            "..oiiiiiiiiiiio.",
            "..ooooooooooooo.",
            "................",
        ],
    ),
    # A star lyre: dark arms, a golden bar with stars, strings.
    "star_lyre": (
        {"o": (14, 12, 34), "n": (40, 36, 90), "N": (70, 64, 140), "g": (190, 146, 50), "G": (250, 216, 110), "s": (230, 226, 210)},
        [
            "..G.........G...",
            ".GGG.......GGG..",
            "..GgggggggggG...",
            "..onssssssssno..",
            ".onNs.s.s.s.Nno.",
            ".onNs.s.s.s.Nno.",
            "..onss.s.s.sno..",
            "..onss.s.s.sno..",
            "...onss.s.sno...",
            "...onNs.s.sNno..",
            "...onNs.s.sNno..",
            "..onnnnnnnnnno..",
            "..onNNNNNNNNno..",
            "..ognnnnnnnngo..",
            "...oooooooooo...",
            "................",
        ],
    ),
}


def main():
    for name, (pal, rows) in ICONS.items():
        assert len(rows) == 16 and all(len(r) == 16 for r in rows), name
        im = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
        for y, row in enumerate(rows):
            for x, ch in enumerate(row):
                if ch != ".":
                    im.putpixel((x, y), pal[ch] + (255,))
        im.save(OUT / f"{name}.png")
        print("wrote", name)


if __name__ == "__main__":
    main()
