//! Gun item icons drawn from the 3D gun itself (guns.rs), like the armour icons (armor_icon.rs):
//! the same boxes and colours, seen from the left side and a little from above so the barrel
//! points right like a Terraria icon, cropped to fill the slot. The pictures go to
//! tools/tex64/guns3d/ and tools/gen_atirador.py turns them into chunky pixel art (pixel.py).
//! Re-run after changing a gun:
//!
//! ```text
//! cargo test -p game write_gun_icons -- --ignored
//! python3 tools/gen_atirador.py
//! ```

use glam::Vec3;

use crate::guns;

const OUT: usize = 64;
const GRID: usize = 32;
const SS: usize = 4;
const MARGIN: f32 = 1.0;

/// A box hit by a ray: distance, the face's normal and the material.
fn hit(o: Vec3, d: Vec3, lo: Vec3, hi: Vec3) -> Option<(f32, Vec3)> {
    let mut t0 = f32::MIN;
    let mut t1 = f32::MAX;
    let mut n = Vec3::ZERO;
    for a in 0..3 {
        if d[a].abs() < 1e-6 {
            if o[a] < lo[a] || o[a] > hi[a] {
                return None;
            }
            continue;
        }
        let (mut ta, mut tb) = ((lo[a] - o[a]) / d[a], (hi[a] - o[a]) / d[a]);
        let mut na = Vec3::ZERO;
        na[a] = -d[a].signum();
        if ta > tb {
            std::mem::swap(&mut ta, &mut tb);
        }
        if ta > t0 {
            t0 = ta;
            n = na;
        }
        t1 = t1.min(tb);
    }
    (t0 <= t1 && t1 > 0.0).then_some((t0, n))
}

/// The icon of gun `i`: OUT × OUT RGBA.
pub fn render(i: usize) -> Vec<u8> {
    let boxes: Vec<guns_box::B> = guns_box::all(i);
    let d = Vec3::new(1.0, -0.42, 0.24).normalize();
    let right = d.cross(Vec3::Y).normalize();
    let up = right.cross(d).normalize();
    // Long guns lie on the slot's diagonal, rising to the right, so they fill it; pistols stay
    // nearly level.
    let tilt: f32 = if guns::two_handed(i) { 0.5 } else { 0.12 };
    let (right, up) = (right * tilt.cos() - up * tilt.sin(), up * tilt.cos() + right * tilt.sin());
    let light = Vec3::new(-0.3, 1.0, 0.5).normalize();
    // Fit the picture: the corners of every box on the screen's axes.
    let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for b in &boxes {
        for k in 0..8 {
            let p = Vec3::new(if k & 1 == 0 { b.lo.x } else { b.hi.x }, if k & 2 == 0 { b.lo.y } else { b.hi.y }, if k & 4 == 0 { b.lo.z } else { b.hi.z });
            let (sx, sy) = (p.dot(right), p.dot(up));
            x0 = x0.min(sx);
            x1 = x1.max(sx);
            y0 = y0.min(sy);
            y1 = y1.max(sy);
        }
    }
    let span = (x1 - x0).max(y1 - y0);
    let cells = GRID as f32 - 2.0 * MARGIN;
    let (cx, cy) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
    let n = GRID * SS;
    let mut cell = vec![[0f32; 4]; GRID * GRID];
    let mut cface = vec![usize::MAX; GRID * GRID];
    for gy in 0..GRID {
        for gx in 0..GRID {
            let mut acc = [0f32; 4];
            let mut seen: Vec<(usize, usize)> = Vec::new();
            for sy in 0..SS {
                for sx in 0..SS {
                    let px = (gx * SS + sx) as f32 + 0.5;
                    let py = (gy * SS + sy) as f32 + 0.5;
                    let u = (px / n as f32 - 0.5) * GRID as f32 / cells * span;
                    let v = (0.5 - py / n as f32) * GRID as f32 / cells * span;
                    let o = right * (cx + u) + up * (cy + v) - d * 200.0;
                    let mut best: Option<(f32, Vec3, usize)> = None;
                    for (k, b) in boxes.iter().enumerate() {
                        if let Some((t, nn)) = hit(o, d, b.lo, b.hi) {
                            if best.is_none_or(|bb| t < bb.0) {
                                best = Some((t, nn, k));
                            }
                        }
                    }
                    if let Some((_, nn, k)) = best {
                        let mat = boxes[k].mat;
                        let c = guns::base(mat);
                        let glow = matches!(mat, guns::EMBER | guns::CYAN | guns::VIOLET);
                        let shade = if glow { 1.1 } else { 0.62 + 0.45 * nn.dot(light).max(0.0) + if nn.y > 0.5 { 0.06 } else { 0.0 } };
                        for j in 0..3 {
                            acc[j] += c[j] * shade;
                        }
                        acc[3] += 1.0;
                        let face = k * 6 + if nn.x != 0.0 { 0 } else if nn.y != 0.0 { 2 } else { 4 } + (nn.max_element() > 0.0) as usize;
                        match seen.iter_mut().find(|(f, _)| *f == face) {
                            Some(e) => e.1 += 1,
                            None => seen.push((face, 1)),
                        }
                    }
                }
            }
            if acc[3] * 2.0 >= (SS * SS) as f32 {
                cell[gy * GRID + gx] = [acc[0] / acc[3], acc[1] / acc[3], acc[2] / acc[3], 255.0];
                cface[gy * GRID + gx] = seen.iter().max_by_key(|e| e.1).unwrap().0;
            }
        }
    }
    // A thin dark line where one part meets another, so the barrel, the stock and the grip
    // keep their edges at this size.
    let mut lined = cell.clone();
    for y in 0..GRID {
        for x in 0..GRID {
            let i = y * GRID + x;
            if cell[i][3] == 0.0 {
                continue;
            }
            let other = |j: usize| cell[j][3] > 0.0 && cface[j] / 6 != cface[i] / 6 && (cell[j][0] - cell[i][0]).abs() + (cell[j][1] - cell[i][1]).abs() > 40.0;
            if (x > 0 && other(i - 1)) || (y > 0 && other(i - GRID)) {
                for j in 0..3 {
                    lined[i][j] *= 0.75;
                }
            }
        }
    }
    let mut out = vec![0u8; OUT * OUT * 4];
    let k = OUT / GRID;
    for y in 0..OUT {
        for x in 0..OUT {
            let c = lined[(y / k) * GRID + x / k];
            let o = (y * OUT + x) * 4;
            for j in 0..4 {
                out[o + j] = c[j].round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}

/// Every box of a gun at rest in gun space.
mod guns_box {
    use glam::Vec3;

    pub struct B {
        pub lo: Vec3,
        pub hi: Vec3,
        pub mat: u8,
    }

    pub fn all(i: usize) -> Vec<B> {
        crate::guns::model(i)
            .into_iter()
            .flat_map(|p| {
                let place = p.place;
                p.boxes.into_iter().map(move |b| {
                    let (a, c) = (place.transform_point3(b.lo), place.transform_point3(b.hi));
                    B { lo: a.min(c), hi: a.max(c), mat: b.mat }
                })
            })
            .collect()
    }
}

#[test]
fn every_gun_icon_fills_its_slot() {
    for i in 0..guns::COUNT {
        let px = render(i);
        let filled = px.chunks(4).filter(|p| p[3] > 0).count();
        assert!(filled > OUT * OUT / 12, "{} icon nearly empty ({filled})", guns::GUNS[i].item);
    }
}

#[test]
#[ignore]
fn write_gun_icons() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = root.join("tools/tex64/guns3d");
    std::fs::create_dir_all(&dir).unwrap();
    for (i, g) in guns::GUNS.iter().enumerate() {
        let px = render(i);
        image::save_buffer(dir.join(format!("{}.png", g.item)), &px, OUT as u32, OUT as u32, image::ColorType::Rgba8).unwrap();
    }
}
