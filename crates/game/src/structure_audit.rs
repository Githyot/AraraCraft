//! Structure audit (thread "Analisar e melhorar todas as estruturas"): loads every structure
//! file the way the game does and checks what can be checked without playing: unknown loot
//! items and mobs, biome names nobody has, buildings that never get placed, buildings that
//! land on each other, and how far the important ones are from spawn.
//!
//! `cargo test -p game --release structure_audit -- --ignored --nocapture` prints the full
//! report; the plain tests guard the data.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use voxel::structures::{Placement, Structures};
use voxel::worldgen::{Dim, Generator};

fn data() -> (voxel::block::BlockRegistry, voxel::item::ItemRegistry, std::sync::Arc<Structures>) {
    let base = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let (reg, items, _) = crate::load_game_data(base).unwrap();
    let structs = crate::load_structures(base, &reg);
    (reg, items, structs)
}

/// Every biome name the generator can report.
fn biome_names() -> HashSet<&'static str> {
    (0..voxel::worldgen::Biome::COUNT as u8).map(|v| voxel::worldgen::Biome::from_u8(v).name()).collect()
}

#[test]
fn structure_loot_and_mobs_exist() {
    let (_, items, structs) = data();
    let mobs = crate::mobs::registry_for_tests();
    let mut bad = Vec::new();
    for s in &structs.list {
        for l in s.def.loot.iter().chain(s.def.chests.iter().flat_map(|c| c.loot.iter())) {
            if items.id(&l.item).is_none() {
                bad.push(format!("{}: loot {}", s.def.name, l.item));
            }
        }
        if let Some(m) = &s.def.spawner {
            if mobs.id(m).is_none() {
                bad.push(format!("{}: spawner mob {m}", s.def.name));
            }
        }
        for (m, _) in &s.def.mobs {
            if mobs.id(m).is_none() {
                bad.push(format!("{}: mob {m}", s.def.name));
            }
        }
    }
    assert!(bad.is_empty(), "unknown names:\n{}", bad.join("\n"));
}

#[test]
fn structure_names_and_biomes_are_real() {
    let (_, _, structs) = data();
    let known = biome_names();
    let mut names = HashSet::new();
    let mut bad = Vec::new();
    for s in &structs.list {
        if !names.insert(s.def.name.clone()) {
            bad.push(format!("two structures called {}", s.def.name));
        }
        for b in &s.def.biomes {
            if !known.contains(b.as_str()) {
                bad.push(format!("{}: biome {b}", s.def.name));
            }
        }
        for p in &s.def.pieces {
            if !structs.list.iter().any(|o| &o.def.name == p) {
                bad.push(format!("{}: piece {p}", s.def.name));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

fn overlap(a: &Placement, b: &Placement, s: &Structures, gap: i32) -> bool {
    let fa = s.list[a.index].turned_footprint(a.turns);
    let fb = s.list[b.index].turned_footprint(b.turns);
    let (ya, yb) = (s.list[a.index].size().y, s.list[b.index].size().y);
    a.origin.x < b.origin.x + fb.x + gap
        && b.origin.x < a.origin.x + fa.x + gap
        && a.origin.z < b.origin.z + fb.y + gap
        && b.origin.z < a.origin.z + fa.y + gap
        && a.origin.y < b.origin.y + yb
        && b.origin.y < a.origin.y + ya
}

/// Buildings of one world in a square, with the pairs that cut into each other.
fn survey(dim: Dim, seed: u64, half: i32) -> (Vec<Placement>, Vec<(usize, usize)>, std::sync::Arc<Structures>) {
    let (reg, _, structs) = data();
    let gen = Generator::new(seed, &reg).with_structures(structs.clone()).with_dim(dim).with_spread(100);
    let all = gen.placements_in(-half, -half, half, half);
    let mut hits = Vec::new();
    for i in 0..all.len() {
        for j in i + 1..all.len() {
            if overlap(&all[i], &all[j], &structs, 0) {
                hits.push((i, j));
            }
        }
    }
    (all, hits, structs)
}

#[test]
fn buildings_do_not_cut_into_each_other() {
    for (dim, half) in [(Dim::Overworld, 2048), (Dim::Nether, 1024), (Dim::End, 1536)] {
        let (all, hits, s) = survey(dim, 12345, half);
        let names: Vec<String> = hits.iter().take(12).map(|&(i, j)| format!("{} x {} at {}", s.list[all[i].index].def.name, s.list[all[j].index].def.name, all[i].origin)).collect();
        assert!(hits.is_empty(), "{dim:?}: {} of {} buildings overlap:\n{}", hits.len(), all.len(), names.join("\n"));
    }
}

#[test]
#[ignore]
fn structure_audit() {
    let mut out = String::new();
    use std::fmt::Write;
    for (dim, half) in [(Dim::Overworld, 4096), (Dim::Nether, 1024), (Dim::End, 2048), (Dim::Aether, 1024), (Dim::Otherside, 1024)] {
        for seed in [12345u64, 777] {
            let (all, hits, s) = survey(dim, seed, half);
            let area = (2.0 * half as f32 / 1000.0).powi(2);
            let _ = writeln!(out, "\n== {dim:?} seed {seed}: {} buildings in {}x{} blocks, {} overlapping pairs", all.len(), 2 * half, 2 * half, hits.len());
            let mut count: HashMap<usize, u32> = HashMap::new();
            for p in &all {
                *count.entry(p.index).or_default() += 1;
            }
            let mut pairs: HashMap<(String, String), u32> = HashMap::new();
            for &(i, j) in &hits {
                let mut k = [s.list[all[i].index].def.name.clone(), s.list[all[j].index].def.name.clone()];
                k.sort();
                *pairs.entry((k[0].clone(), k[1].clone())).or_default() += 1;
            }
            let mut pv: Vec<_> = pairs.into_iter().collect();
            pv.sort_by(|a, b| b.1.cmp(&a.1));
            for ((a, b), n) in pv.iter().take(25) {
                let _ = writeln!(out, "  overlap {n:3} x  {a} / {b}");
            }
            let mut rows: Vec<(String, u32, f32, i32)> = Vec::new();
            for (i, st) in s.list.iter().enumerate() {
                let n = count.get(&i).copied().unwrap_or(0);
                let near = all.iter().filter(|p| p.index == i).map(|p| p.origin.x.abs().max(p.origin.z.abs())).min().unwrap_or(-1);
                let dim_ok = match dim {
                    Dim::Overworld => !st.def.biomes.iter().any(|b| ["Nether Wastes", "Soul Sand Valley", "Crimson Forest", "Warped Forest", "The End", "End Highlands", "Aether", "Golden Grove", "Deeplands", "Echoing Forest"].contains(&b.as_str())),
                    _ => n > 0,
                };
                if dim_ok {
                    rows.push((st.def.name.clone(), n, n as f32 / area, near));
                }
            }
            rows.sort_by(|a, b| a.1.cmp(&b.1));
            for (name, n, per, near) in rows {
                let _ = writeln!(out, "  {name:30} {n:5}  {per:7.2}/km2  nearest {near}");
            }
        }
    }
    let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/structure_audit.txt"));
    let _ = std::fs::write(path, &out);
    println!("{out}");
}

#[test]
#[ignore]
fn structure_lookup_speed() {
    let (reg, _, structs) = data();
    let gen = Generator::new(12345, &reg).with_structures(structs).with_spread(100);
    let t = std::time::Instant::now();
    let mut n = 0;
    for cz in -10..10 {
        for cx in -10..10 {
            n += gen.placements_near(voxel::chunk::ChunkPos::new(cx, cz)).len();
        }
    }
    let first = t.elapsed();
    let t = std::time::Instant::now();
    for cz in -10..10 {
        for cx in -10..10 {
            n += gen.placements_near(voxel::chunk::ChunkPos::new(cx, cz)).len();
        }
    }
    println!("400 chunks: first {first:?}, again {:?} ({n} hits)", t.elapsed());
}

#[test]
#[ignore]
fn mountain_ground() {
    let (reg, _, structs) = data();
    let gen = Generator::new(12345, &reg).with_structures(structs);
    let mut n = 0;
    let mut spreads = Vec::new();
    for i in 0..400_000i32 {
        let (x, z) = ((i * 7919) % 8000 - 4000, (i * 104729) % 8000 - 4000);
        if gen.biome_at(x, z).base() != voxel::worldgen::Biome::Mountains {
            continue;
        }
        n += 1;
        let hs = [gen.height_at(x - 26, z - 26), gen.height_at(x + 26, z - 26), gen.height_at(x - 26, z + 26), gen.height_at(x + 26, z + 26), gen.height_at(x, z)];
        spreads.push(hs.iter().max().unwrap() - hs.iter().min().unwrap());
        if spreads.len() > 3000 {
            break;
        }
    }
    spreads.sort();
    println!("mountain samples {n}: spread p10 {} p50 {} p90 {}", spreads[spreads.len() / 10], spreads[spreads.len() / 2], spreads[spreads.len() * 9 / 10]);
}

#[test]
fn fortress_spawners_make_blazes_and_dungeons_exist() {
    let (reg, _, structs) = data();
    let fort = structs.list.iter().find(|s| s.def.name == "fortaleza_do_nether").unwrap();
    assert_eq!(fort.def.spawner.as_deref(), Some("blaze"));
    let gen = Generator::new(12345, &reg).with_structures(structs.clone()).with_spread(100);
    let all = gen.placements_in(-2048, -2048, 2048, 2048);
    let n = all.iter().filter(|p| structs.list[p.index].def.name.starts_with("sala_de_monstros")).count();
    assert!(n > 20, "only {n} monster rooms in 4x4 km");
}

/// Prints where and how a building is turned (to aim screenshots inside it):
/// `MC_WHAT=name MC_AT=x,z cargo test -p game --release where_is -- --ignored --nocapture`.
#[test]
#[ignore]
fn where_is() {
    let (reg, _, structs) = data();
    let name = std::env::var("MC_WHAT").unwrap_or_else(|_| "fortaleza_subterranea".into());
    let at: Vec<i32> = std::env::var("MC_AT").unwrap_or_else(|_| "0,0".into()).split(',').filter_map(|t| t.parse().ok()).collect();
    let gen = Generator::new(12345, &reg).with_structures(structs.clone());
    for p in gen.placements_in(at[0] - 200, at[1] - 200, at[0] + 200, at[1] + 200) {
        if structs.list[p.index].def.name == name {
            println!("{name} at {} turns {}", p.origin, p.turns);
        }
    }
}

#[test]
fn stronghold_rooms_fill_their_own_chests() {
    let (reg, _, structs) = data();
    let gen = Generator::new(12345, &reg).with_structures(structs.clone());
    let (o, size) = gen.find_structure("fortaleza_subterranea", 0, 0, 40).expect("a stronghold");
    // Every chest in it, with what its list starts with.
    let chest = reg.id("chest").unwrap();
    let mut firsts = HashSet::new();
    let mut seen = 0;
    let pos = |p: glam::IVec3| voxel::chunk::ChunkPos::new(p.x.div_euclid(16), p.z.div_euclid(16));
    let mut chunks: HashMap<(i32, i32), voxel::chunk::Chunk> = HashMap::new();
    for y in 0..12 {
        for z in 0..size.z {
            for x in 0..size.x {
                let p = o + glam::IVec3::new(x, y, z);
                let c = pos(p);
                let chunk = chunks.entry((c.x, c.y)).or_insert_with(|| gen.generate(c));
                if chunk.get_i(p.x.rem_euclid(16) as usize, p.y, p.z.rem_euclid(16) as usize) != chest {
                    continue;
                }
                seen += 1;
                let loot = gen.structure_chest(p).expect("a world chest has loot");
                firsts.insert(loot[0].item.clone());
            }
        }
    }
    assert!(seen >= 6, "only {seen} chests found");
    for want in ["enchanted_book", "bread", "diamond", "bone", "ender_pearl"] {
        assert!(firsts.contains(want), "no room whose loot starts with {want}: {firsts:?}");
    }
    // Deeper and later worlds hold more.
    assert!(gen.loot_scale(glam::IVec3::new(0, -300, 0)) > gen.loot_scale(glam::IVec3::new(0, 60, 0)));
}
