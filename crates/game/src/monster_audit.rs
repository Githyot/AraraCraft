//! The monster audit (plan/monstros-analise.md): one line per hostile or neutral mob with what
//! it has and what it lacks. Run with
//! `cargo test -p game monster_audit -- --ignored --nocapture > audit.tsv`.

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn print_monsters() {
        let mut reg = crate::mobs::registry_for_tests();
        let fauna = crate::embedded::FAUNA_MODELS_RON;
        let fauna_names: Vec<String> = fauna.lines().filter_map(|l| l.trim().strip_prefix("name: \"").map(|r| r.split('"').next().unwrap_or("").to_string())).collect();
        let _ = reg.apply_models(fauna);
        let tex = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs/vanilla/textures/entity");
        println!("AUDIT\tname\tdisplay\tkind\tboss\thp\tatk\tparts\tlayers\tfauna\tstyle\tvoice\ttier\tdrops\tspawn\tbiomes\tflags\tskin");
        for d in &reg.defs {
            if d.kind == voxel::mob::Kind::Passive && !d.boss {
                continue;
            }
            let rig = crate::anim::Rig::new(d);
            let skin = d.skin.clone().unwrap_or(d.name.clone());
            let has_skin = tex.join(format!("{skin}.png")).exists();
            let tier = crate::mob_tiers::tier(&d.name).map(|(s, r)| format!("{s}:{r:?}")).unwrap_or("-".into());
            let mut flags = Vec::new();
            for (b, n) in [(d.ranged, "ranged"), (d.explodes, "explodes"), (d.flies, "flies"), (d.climbs, "climbs"), (d.burns_in_daylight, "burns"), (d.aquatic, "aquatic"), (d.town, "town"), (d.lit, "lit"), (d.hops, "hops")] {
                if b {
                    flags.push(n);
                }
            }
            println!(
                "AUDIT\t{}\t{}\t{:?}\t{}\t{}\t{}\t{}\t{}\t{}\t{:?}\t{}\t{}\t{}\t{:?}\t{}\t{}\t{}",
                d.name,
                d.display_name,
                d.kind,
                d.boss,
                d.health,
                d.attack,
                d.parts.len(),
                d.layers.len(),
                fauna_names.contains(&d.name),
                rig.style,
                crate::mob_sounds::has_voice(&d.name),
                tier,
                d.drops.len(),
                d.spawn,
                d.biomes.join(","),
                flags.join(","),
                if has_skin { "ok" } else { "MISSING" },
            );
        }
    }
}
