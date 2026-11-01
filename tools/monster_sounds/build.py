#!/usr/bin/env python3
"""Builds the monster voices: python3 tools/monster_sounds/build.py [--sheets DIR]

Reads families.py, takes every recording it names from the downloaded sources (paths in
families.ROOTS), trims the silence, keeps it under 3 s, cuts the shrill top (low-pass 5 kHz, lower for bright clips) and
the rumble (high-pass 50 Hz), evens the loudness, and writes Ogg Vorbis mono 22 kHz clips to
assets/sounds/monsters/. Then writes crates/game/src/monster_clips.rs (the clip list and the
families) for monster_sounds.rs.
"""
import glob, os, re, subprocess, sys
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "../.."))
sys.path.insert(0, HERE)
from families import FAMILIES, ROOTS

OUT = os.path.join(REPO, "assets/sounds/monsters")
R = 22050
MAX = 3.0


def resolve(spec):
    src, pat = spec.split(":", 1)
    hits = sorted(glob.glob(os.path.join(ROOTS[src], pat), recursive=True))
    if not hits:
        raise SystemExit(f"nothing matches {spec}")
    return [(src, h) for h in hits]


def clip_name(src, path):
    rel = os.path.relpath(path, ROOTS[src])
    parts = [p for p in re.split(r"[/\\]", rel) if p not in ("sounds", "**")]
    base = "_".join(parts[-2:]) if src in ("mg", "oc") else parts[-1]
    base = re.sub(r"\.(ogg|wav)$", "", base, flags=re.I)
    base = re.sub(r"^(mobs_mc_|mobs_|extra_mobs_|factions_|vl_)", "", base)
    base = re.sub(r"[^a-z0-9]+", "_", base.lower()).strip("_")
    base = base.replace("_ocg_", "_")
    return f"{src}_{base}"


def brightness(x):
    """Effective pitch of a clip as the game hears it (mob_sounds.rs: played at 44.1 kHz through
    two 3 kHz low-pass stages), from how fast the wave changes."""
    y = np.interp(np.arange(len(x) * 2) / 2.0, np.arange(len(x)), x)
    k = 1.0 - np.exp(-2 * np.pi * 3000.0 / 44100.0)
    for _ in range(2):
        out = np.empty_like(y)
        a = 0.0
        for i, v in enumerate(y):
            a += (v - a) * k
            out[i] = a
        y = out
    e = float((y ** 2).sum()) + 1e-12
    d = float((np.diff(y) ** 2).sum())
    return 44100.0 / np.pi * np.arcsin(min(1.0, np.sqrt(d / e) / 2.0))


def process(path, dest):
    # Bright clips get a lower cut until they sit well below the game's 2.5 kHz limit.
    for cut in (5000, 3500, 2500, 1800, 1300):
        x = decode(path, cut)
        if brightness(x) < 2200:
            break
    finish(x, dest)


def decode(path, cut):
    raw = subprocess.run(["ffmpeg", "-v", "quiet", "-i", path, "-ac", "1", "-ar", str(R),
                          "-af", f"highpass=f=50,lowpass=f={cut},lowpass=f={cut}", "-f", "f32le", "-"],
                         capture_output=True, check=True).stdout
    x = np.frombuffer(raw, dtype=np.float32).copy()
    if len(x) == 0:
        raise SystemExit(f"empty {path}")
    return x


def finish(x, dest):
    # Trim quiet ends (-40 dB of the peak).
    env = np.abs(x)
    th = env.max() * 0.01
    idx = np.where(env > th)[0]
    a, b = max(0, idx[0] - int(R * 0.01)), min(len(x), idx[-1] + int(R * 0.05))
    x = x[a:b]
    if len(x) > R * MAX:
        x = x[: int(R * MAX)]
        fade = int(R * 0.3)
        x[-fade:] *= np.linspace(1, 0, fade)
    # Even loudness: RMS of the loudest 100 ms to -14 dBFS, peaks no higher than -1 dB.
    w = int(R * 0.1)
    if len(x) > w:
        loud = np.sqrt(np.convolve(x ** 2, np.ones(w) / w, "valid").max())
    else:
        loud = np.sqrt(np.mean(x ** 2))
    g = 0.2 / max(loud, 1e-4)
    g = min(g, 0.89 / max(np.abs(x).max(), 1e-4))
    x = (x * g).astype(np.float32)
    subprocess.run(["ffmpeg", "-v", "quiet", "-y", "-f", "f32le", "-ar", str(R), "-ac", "1", "-i", "-",
                    "-c:a", "libvorbis", "-q:a", "3", dest], input=x.tobytes(), check=True)


def main():
    os.makedirs(OUT, exist_ok=True)
    clips = {}  # name -> source path
    fams = {}
    for fam, calls in FAMILIES.items():
        fams[fam] = {}
        for call in ("idle", "hurt", "death", "attack"):
            names = []
            for spec in calls.get(call, []):
                for src, p in resolve(spec):
                    n = clip_name(src, p)
                    if clips.get(n, p) != p:
                        raise SystemExit(f"two sources named {n}: {clips[n]} and {p}")
                    clips[n] = p
                    if n not in names:
                        names.append(n)
            fams[fam][call] = names
    for n, p in sorted(clips.items()):
        dest = os.path.join(OUT, n + ".ogg")
        if not os.path.exists(dest) or "--force" in sys.argv:
            process(p, dest)
    # Remove clips no family uses any more.
    for f in os.listdir(OUT):
        if f.endswith(".ogg") and f[:-4] not in clips:
            os.remove(os.path.join(OUT, f))
    order = sorted(clips)
    with open(os.path.join(REPO, "crates/game/src/monster_clips.rs"), "w") as f:
        f.write("// Written by tools/monster_sounds/build.py from tools/monster_sounds/families.py: do not edit.\n\n")
        f.write("/// Every monster recording, by name (Ogg Vorbis, mono, 22 kHz).\n")
        f.write("pub static CLIPS: &[(&str, &[u8])] = &[\n")
        for n in order:
            f.write(f'    ("{n}", include_bytes!("../../../assets/sounds/monsters/{n}.ogg")),\n')
        f.write("];\n\n")
        f.write("/// The voice families: (name, idle, hurt, death, attack), clips by index into CLIPS.\n")
        f.write("pub static FAMILIES: &[(&str, &[u16], &[u16], &[u16], &[u16])] = &[\n")
        for fam, c in fams.items():
            ids = lambda k: "&[" + ", ".join(str(order.index(n)) for n in c[k]) + "]"
            f.write(f'    ("{fam}", {ids("idle")}, {ids("hurt")}, {ids("death")}, {ids("attack")}),\n')
        f.write("];\n")
    total = sum(os.path.getsize(os.path.join(OUT, n + ".ogg")) for n in order)
    print(f"{len(order)} clips, {len(fams)} families, {total / 1e6:.1f} MB")
    if "--sheets" in sys.argv:
        d = sys.argv[sys.argv.index("--sheets") + 1]
        os.makedirs(d, exist_ok=True)
        for fam, c in fams.items():
            ps = [os.path.join(OUT, n + ".ogg") for k in ("idle", "hurt", "death", "attack") for n in c[k]][:16]
            subprocess.run([sys.executable, "/tmp/claude-0/snd/sheet.py", os.path.join(d, fam + ".png")] + ps)


if __name__ == "__main__":
    main()
