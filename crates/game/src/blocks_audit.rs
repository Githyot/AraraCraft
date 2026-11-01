//! Writes one line per base block (shape, light, textures, sound material...) for the block
//! review in plan/blocos-analise.md: `MC_AUDIT=out.tsv cargo test -p game dump_blocks -- --ignored`.

#[cfg(test)]
mod tests {
    use crate::audio::Material;
    use crate::embedded;

    /// Every picture a block names is one of ours in packs/vanilla/textures/block (so new blocks
    /// never show the missing-texture checkerboard), and each is 64x64.
    #[test]
    fn every_block_picture_exists() {
        let (reg, _, _) = crate::game_data(embedded::BLOCKS_RON, embedded::ITEMS_RON, embedded::RECIPES_RON).unwrap();
        let dir = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../packs/vanilla/textures/block"));
        let mut missing = Vec::new();
        for b in reg.blocks.iter().skip(1).filter(|b| b.variant == 0) {
            for &f in &b.faces {
                let t = reg.textures[f as usize].as_str();
                if t.contains('/') || crate::flora_tex::parse(t).is_some() {
                    continue;
                }
                let p = dir.join(format!("{t}.png"));
                match image::image_dimensions(&p) {
                    Ok((64, 64)) => {}
                    Ok(d) => missing.push(format!("{} ({t} is {}x{})", b.name, d.0, d.1)),
                    Err(_) => missing.push(format!("{} ({t})", b.name)),
                }
            }
        }
        missing.dedup();
        assert!(missing.is_empty(), "block pictures missing or not 64x64: {missing:?}");
    }

    #[test]
    #[ignore]
    fn dump_blocks() {
        let (reg, items, _) = crate::game_data(embedded::BLOCKS_RON, embedded::ITEMS_RON, embedded::RECIPES_RON).unwrap();
        let mut out = String::from("name\tdisplay\tshape\ttransp\tlight\tsolid\thardness\ttool\ttier\tgravity\tinv\tmaterial\tflora\tmodel\thas_item\ttextures\n");
        for b in reg.blocks.iter().skip(1) {
            if b.variant != 0 {
                continue;
            }
            let mut tex: Vec<&str> = b.faces.iter().map(|&f| reg.textures[f as usize].as_str()).collect();
            tex.dedup();
            out += &format!(
                "{}\t{}\t{:?}\t{:?}\t{}\t{}\t{}\t{:?}\t{}\t{}\t{}\t{:?}\t{}\t{}\t{}\t{}\n",
                b.name,
                b.display_name,
                b.shape,
                b.transparency,
                b.light,
                b.solid,
                b.hardness,
                b.tool,
                b.tier,
                b.gravity,
                b.in_inventory,
                Material::of(&b.name),
                voxel::flora::kind_of(&b.name).map(|k| format!("{k:?}")).unwrap_or_default(),
                b.model.is_some() || voxel::props::block_quads(b).is_some(),
                items.id(&b.name).is_some(),
                tex.join(",")
            );
        }
        let path = std::env::var("MC_AUDIT").unwrap_or_else(|_| "blocks_audit.tsv".into());
        std::fs::write(path, out).unwrap();
    }
}
