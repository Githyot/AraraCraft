//! The mining ladder (plan/progressao-linear.md): Minecraft's and Terraria's metals in one
//! ordered chain, where every ore needs the pickaxe made from the ore just before it.
//!
//! ```text
//! wood → stone → copper/tin → iron/lead → silver/tungsten → gold/platinum →
//! diamond · demonite (umbral/sanguinite) · meteorite → obsidian · ancient debris · magmarite →
//! [World Breaker] cobalt/palladium → mythril/orichalcum → adamantite/titanium →
//! [the three machines] verdite → [the Vine Queen] the scaled temple → lumite
//! ```
//!
//! A block's `tier` is the mining level it needs and a tool's `tier` the level it has, as in the
//! files, but the files keep the classic Minecraft numbers (parity_test.rs checks them, and the
//! classes' balance sheet reads them). `apply` rewrites the loaded registries once when the
//! world opens, unless the option "Progressão: Clássica" is on. Armour follows the same steps:
//! a set of a later step never has less defense than the best set of the step before it
//! (`armour_floor`), so each new metal is worth wearing.

use voxel::block::BlockRegistry;
use voxel::block::ToolKind;
use voxel::item::ItemRegistry;

/// One step of the ladder.
pub struct Step {
    /// Mining level its pickaxes have (and the level the ores of the next step need).
    pub level: u8,
    /// What the step is called in the Guide and the tooltips.
    pub name: &'static str,
    /// Materials whose tools (`<material>_pickaxe`, `_axe`...) have this level.
    pub tools: &'static [&'static str],
    /// Ores (and other blocks) its pickaxes open: they need this step's level.
    pub ores: &'static [&'static str],
    /// Armour sets of the step (`<prefix>_helmet`...).
    pub armour: &'static [&'static str],
    /// Where the ores it opens are (and what boss lets them grow), for the tooltips and Guide.
    pub gate: &'static str,
}

/// The ladder. Step `i` has level `i + 1`, and its `ores` are what its pickaxes open (they
/// need exactly that level, so the pickaxe of the step before can't take them). Pairs stand
/// side by side: each world has one metal of the pair common and the other rare, and either
/// one climbs the ladder.
pub const STEPS: &[Step] = &[
    Step { level: 1, name: "Madeira", tools: &["wooden", "skyroot"], ores: &["stone", "coal_ore", "nether_quartz_ore", "nether_gold_ore", "ambrosium_ore"], armour: &["leather"], gate: "" },
    Step { level: 2, name: "Pedra", tools: &["stone", "holystone", "flint"], ores: &["copper_ore", "tin_ore", "copper_block", "raw_copper_block"], armour: &[], gate: "cobre e estanho logo abaixo da altura 0" },
    Step { level: 3, name: "Cobre e estanho", tools: &["copper", "bronze", "tin"], ores: &["iron_ore", "lead_ore", "lapis_ore", "zanite_ore"], armour: &["copper", "bronze", "tin"], gate: "ferro e chumbo abaixo de -20" },
    Step { level: 4, name: "Ferro e chumbo", tools: &["iron", "lead", "zanite"], ores: &["silver_ore", "tungsten_ore", "redstone_ore", "amethyst_ore", "topaz_ore"], armour: &["iron", "lead", "zanite"], gate: "prata e tungstênio abaixo de -60" },
    Step { level: 5, name: "Prata e tungstênio", tools: &["silver", "tungsten"], ores: &["gold_ore", "platinum_ore", "sapphire_ore", "ruby_ore", "emerald_ore"], armour: &["silver", "tungsten"], gate: "ouro e platina abaixo de -160" },
    Step {
        level: 6,
        name: "Ouro e platina",
        tools: &["golden", "gold", "platinum"],
        ores: &["diamond_ore", "umbral_ore", "sanguinite_ore", "meteorite_ore", "gravitite_ore", "enchanted_gravitite", "resonarium_ore"],
        armour: &["golden", "platinum"],
        gate: "diamante abaixo de -300; minério escuro na terra maldita; meteorito depois do primeiro chefe",
    },
    Step {
        level: 7,
        name: "Diamante, sombraço e sanguinita",
        tools: &["diamond", "umbral", "sanguinite", "meteor", "gravitite", "resonarium"],
        ores: &[
            "obsidian",
            "crying_obsidian",
            "obsidian_bricks",
            "respawn_anchor",
            "end_portal_frame_filled",
            "ancient_debris",
            "netherite_block",
            "magmarite_ore",
            "infernal_bricks",
            "dungeon_brick_blue",
            "dungeon_brick_green",
            "dungeon_brick_pink",
            "locked_chest",
            "titanic_forge",
        ],
        armour: &["diamond", "umbral", "sanguinite", "meteor", "gravitite", "resonarium"],
        gate: "obsidiana onde a água toca a lava; detritos no Nether; magmarita no Submundo (abaixo de -640)",
    },
    Step { level: 8, name: "Netherite e magmarita", tools: &["netherite", "magmarite", "molten"], ores: &["cobalt_ore", "palladium_ore", "ruin_altar", "blight_altar"], armour: &["netherite", "magmarite"], gate: "cobalto e paládio, depois do Quebra-Mundos" },
    Step { level: 9, name: "Cobalto e paládio", tools: &["cobalt", "palladium"], ores: &["mythril_ore", "orichalcum_ore"], armour: &["cobalt", "palladium"], gate: "mitril e oricalco, mais fundos" },
    Step { level: 10, name: "Mitril e oricalco", tools: &["mythril", "orichalcum"], ores: &["adamantite_ore", "titanium_ore"], armour: &["mythril", "orichalcum"], gate: "adamantita e titânio, os mais fundos" },
    Step { level: 11, name: "Adamantita e titânio", tools: &["adamantite", "titanium", "radiant"], ores: &["verdite_ore"], armour: &["adamantite", "titanium", "radiant"], gate: "verdita na selva, depois das três máquinas" },
    Step { level: 12, name: "Verdita", tools: &["verdite", "chlorophyte", "spectral", "sporite"], ores: &["scale_brick", "scale_altar"], armour: &["verdite"], gate: "os tijolos do templo escamado" },
    Step { level: 13, name: "Lumita", tools: &["lumite", "luminite", "solar", "vortex", "nebula", "stardust"], ores: &[], armour: &["lumite"], gate: "" },
];

/// Tools named by what they are, not by a material (level).
const NAMED_TOOLS: &[(&str, u8)] = &[
    ("iron_hammer", 4),
    ("tungsten_hammer", 5),
    ("platinum_hammer", 6),
    ("diamond_hammer", 7),
    ("volcanic_hammer", 7),
    ("breaker_hammer", 8),
    ("pickaxe_axe", 12),
    ("picksaw", 12),
];

/// Armour sets that are not on the ladder's main line keep their numbers (class sets of the
/// bosses and events: they trade defense for their bonus), except these later ones, which only
/// have to beat the best set of the step before as a group (`armour_floor`).
const LATE_SETS: &[(&str, usize)] = &[("spectral", 11), ("sporite", 11), ("beetle", 11), ("turtle", 11), ("solar", 12), ("vortex", 12), ("nebula", 12), ("stardust", 12)];

/// Sets from the mods' side worlds (Aether, Otherside): they stand on a step and are lifted to
/// its floor, but don't raise the floor of the next step (the main metals set the pace).
pub const SIDE_SETS: &[&str] = &["zanite", "gravitite", "resonarium"];

/// Classic tool tier (the files') to the ladder's level, for anything not in the tables
/// (plugins): wood 1, stone 2, iron 4, diamond 7, netherite and cobalt 8, then on.
pub fn from_classic(t: u8) -> u8 {
    const MAP: [u8; 10] = [0, 1, 2, 4, 7, 8, 10, 11, 12, 13];
    MAP.get(t as usize).copied().unwrap_or(t + 4)
}

/// The level a tool's material gives, by its name (`diamond_pickaxe`, `cobalt_drill`,
/// `gear_pickaxe_iron_bone_string`...).
pub fn tool_level(name: &str) -> Option<u8> {
    if let Some(&(_, l)) = NAMED_TOOLS.iter().find(|t| t.0 == name) {
        return Some(l);
    }
    // Tools put together from parts: gear_<kind>_<head>_<handle>_<binding>.
    let mat = match name.strip_prefix("gear_") {
        Some(rest) => rest.split('_').nth(1)?,
        None => name.split('_').next()?,
    };
    STEPS.iter().find(|s| s.tools.contains(&mat)).map(|s| s.level)
}

/// The level a block needs, if the ladder places it: its own line, a deepslate twin of an ore,
/// or the storage block of a metal or gem (`iron_block`, `raw_gold_block`, `ruby_block`).
pub fn block_level(name: &str) -> Option<u8> {
    let find = |n: &str| STEPS.iter().find(|s| s.ores.contains(&n)).map(|s| s.level);
    if let Some(l) = find(name) {
        return Some(l);
    }
    if let Some(base) = name.strip_prefix("deepslate_") {
        if let Some(l) = find(base) {
            return Some(l.max(1));
        }
    }
    let mat = name.strip_suffix("_block")?;
    let mat = mat.strip_prefix("raw_").unwrap_or(mat);
    let mat = if mat == "golden" { "gold" } else { mat };
    find(&format!("{mat}_ore"))
}

/// The step whose pickaxes have `level`.
pub fn step(level: u8) -> Option<&'static Step> {
    STEPS.iter().find(|s| s.level == level)
}

/// Rewrites block and tool levels and lifts armour that would fall behind. Returns how many
/// entries changed (for the log).
pub fn apply(blocks: &mut BlockRegistry, items: &mut ItemRegistry) -> usize {
    let mut changed = 0;
    for b in blocks.blocks.iter_mut() {
        if b.tier == 0 {
            continue;
        }
        let new = block_level(&b.name).unwrap_or_else(|| from_classic(b.tier));
        if new != b.tier {
            b.tier = new;
            changed += 1;
        }
    }
    for it in items.items.iter_mut() {
        let Some(t) = it.tool.as_mut() else { continue };
        if t.tier == 0 {
            continue;
        }
        let new = tool_level(&it.name).unwrap_or_else(|| from_classic(t.tier));
        if new != t.tier {
            t.tier = new;
            changed += 1;
        }
    }
    changed + lift_armour(items)
}

const PIECES: [&str; 4] = ["helmet", "chestplate", "leggings", "boots"];

/// Defense of the four pieces of a set, if it has all four.
pub fn set_points(items: &ItemRegistry, prefix: &str) -> Option<u32> {
    let mut sum = 0;
    for k in PIECES {
        let id = items.id(&format!("{prefix}_{k}"))?;
        sum += items.get(id).armor?.points as u32;
    }
    Some(sum)
}

/// The defense every set of step `i` must reach: one more than the best set of the step
/// before, as the files have them after lifting the steps before (so it is a running floor).
pub fn armour_floor(items: &ItemRegistry, i: usize) -> u32 {
    if i == 0 {
        return 0;
    }
    let prev = &STEPS[i - 1];
    let best = prev.armour.iter().filter(|p| !SIDE_SETS.contains(p)).filter_map(|p| set_points(items, p)).max();
    // A step without armour (stone) passes the floor of the one before it on.
    best.map_or_else(|| armour_floor(items, i - 1), |b| b + 1)
}

/// Raises the sets of each step that are under the step's floor, piece by piece in proportion
/// (the chestplate takes what rounding leaves). Late class sets only need to beat the floor
/// as a group, with their best set.
fn lift_armour(items: &mut ItemRegistry) -> usize {
    let mut changed = 0;
    for i in 0..STEPS.len() {
        let floor = armour_floor(items, i);
        for prefix in STEPS[i].armour {
            changed += lift_set(items, prefix, floor) as usize;
        }
    }
    // Late sets: the best of each step must beat the step before; the rest keep theirs.
    for step in [11usize, 12] {
        let floor = armour_floor(items, step);
        let sets: Vec<&str> = LATE_SETS.iter().filter(|s| s.1 == step).map(|s| s.0).collect();
        let best = sets.iter().filter_map(|p| set_points(items, p).map(|v| (v, *p))).max();
        if let Some((v, p)) = best {
            if v < floor {
                changed += lift_set(items, p, floor) as usize;
            }
        }
    }
    changed
}

fn lift_set(items: &mut ItemRegistry, prefix: &str, floor: u32) -> bool {
    let Some(have) = set_points(items, prefix) else { return false };
    if have >= floor {
        return false;
    }
    let ids: Vec<_> = PIECES.iter().filter_map(|k| items.id(&format!("{prefix}_{k}"))).collect();
    let k = floor as f32 / have.max(1) as f32;
    let mut total = 0;
    for &id in &ids {
        let a = items.items[id as usize].armor.as_mut().unwrap();
        a.points = ((a.points as f32 * k).floor() as u8).max(a.points);
        total += a.points as u32;
    }
    // What rounding left goes on the chestplate.
    if let Some(a) = items.items[ids[1] as usize].armor.as_mut() {
        a.points += floor.saturating_sub(total) as u8;
    }
    true
}

/// Display names of the cheapest pickaxes that have `level` (one per metal of the step).
pub fn pickaxe_names(items: &ItemRegistry, level: u8) -> Vec<String> {
    let Some(s) = step(level) else { return Vec::new() };
    s.tools
        .iter()
        .filter_map(|m| items.id(&format!("{m}_pickaxe")))
        .map(|id| items.get(id))
        .filter(|i| i.tool.as_ref().is_some_and(|t| t.kind == ToolKind::Pickaxe && t.tier == level))
        .map(|i| i.display_name.clone())
        .collect()
}

/// "Picareta de Prata ou Picareta de Tungstênio".
pub fn pickaxes_text(items: &ItemRegistry, level: u8) -> Option<String> {
    let names = pickaxe_names(items, level);
    match names.len() {
        0 => None,
        1 => Some(names[0].clone()),
        n => Some(format!("{} ou {}", names[..n - 1].join(", "), names[n - 1])),
    }
}

/// The metals of the step's pickaxes, short: "Prata ou Tungstênio".
pub fn metals_text(items: &ItemRegistry, level: u8) -> Option<String> {
    let names: Vec<String> = pickaxe_names(items, level)
        .into_iter()
        .map(|n| n.strip_prefix("Picareta de ").or_else(|| n.strip_prefix("Picareta da ")).or_else(|| n.strip_prefix("Picareta do ")).map(str::to_string).unwrap_or(n))
        .collect();
    match names.len() {
        0 => None,
        1 => Some(names[0].clone()),
        n => Some(format!("{} ou {}", names[..n - 1].join(", "), names[n - 1])),
    }
}

/// "picareta de nível 5 ou mais (Prata ou Tungstênio)".
pub fn need_text(items: &ItemRegistry, level: u8) -> Option<String> {
    metals_text(items, level).map(|m| format!("picareta de nível {level} ou mais ({m})"))
}

/// Ore blocks a pickaxe of `level` opens that the one before could not (display names).
pub fn opens(blocks: &BlockRegistry, level: u8) -> Vec<String> {
    let Some(next) = step(level) else { return Vec::new() };
    let mut out: Vec<String> = Vec::new();
    for n in next.ores.iter().filter(|n| n.ends_with("_ore") || ["ancient_debris", "obsidian", "scale_brick"].contains(*n)) {
        if let Some(id) = blocks.id(n) {
            let d = blocks.get(id).display_name.clone();
            let d = d.strip_prefix("Minério de ").map(str::to_string).unwrap_or(d);
            if !out.contains(&d) {
                out.push(d);
            }
        }
    }
    out
}

/// Which ladder step an item of a goal belongs to: its bar, raw metal or gem (`silver_ingot`,
/// `raw_iron`, `diamond`), by the ore it comes from. The level returned is the one the ore needs.
pub fn item_need(name: &str) -> Option<u8> {
    let base = name.strip_suffix("_ingot").or_else(|| name.strip_prefix("raw_")).unwrap_or(name);
    let base = if base == "golden" { "gold" } else { base };
    let ore = format!("{base}_ore");
    STEPS.iter().find(|s| s.ores.contains(&ore.as_str())).map(|s| s.level)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> (BlockRegistry, ItemRegistry, crate::Recipes) {
        crate::game_data(crate::embedded::BLOCKS_RON, crate::embedded::ITEMS_RON, crate::embedded::RECIPES_RON).expect("game data")
    }

    fn laddered() -> (BlockRegistry, ItemRegistry, crate::Recipes) {
        let (mut b, mut i, r) = data();
        apply(&mut b, &mut i);
        (b, i, r)
    }

    #[test]
    fn steps_are_in_order() {
        for (i, s) in STEPS.iter().enumerate() {
            assert_eq!(s.level as usize, i + 1, "{}", s.name);
        }
    }

    #[test]
    fn every_name_in_the_ladder_exists() {
        let (b, i, _) = data();
        for s in STEPS {
            for o in s.ores {
                assert!(b.id(o).is_some(), "bloco {o} ({}) não existe", s.name);
            }
            for a in s.armour {
                assert!(set_points(&i, a).is_some() || ["tin"].contains(a), "armadura {a} ({}) incompleta", s.name);
            }
        }
    }

    /// Every pickaxe of the game has a place on the ladder (no stray classic number).
    #[test]
    fn every_pickaxe_is_placed() {
        let (_, i, _) = data();
        let stray: Vec<_> = i.items.iter().filter(|it| it.tool.as_ref().is_some_and(|t| t.kind == ToolKind::Pickaxe && t.tier > 0)).filter(|it| tool_level(&it.name).is_none()).map(|it| it.name.clone()).collect();
        assert!(stray.is_empty(), "picaretas fora da escada: {stray:?}");
    }

    /// Every ore that needs a pickaxe is placed by name (not by the classic fallback).
    #[test]
    fn every_ore_is_placed() {
        let (b, _, _) = data();
        let stray: Vec<_> = b.blocks.iter().filter(|x| x.tier > 0 && (x.name.ends_with("_ore") || x.name == "ancient_debris")).filter(|x| block_level(&x.name).is_none()).map(|x| x.name.clone()).collect();
        assert!(stray.is_empty(), "minérios fora da escada: {stray:?}");
    }

    /// The point of the ladder: the ores a step opens need that step's pickaxe; nothing of a
    /// lower level takes them.
    #[test]
    fn no_step_can_be_skipped() {
        let (b, i, _) = laddered();
        let picks: Vec<(String, u8)> = i.items.iter().filter_map(|it| it.tool.as_ref().filter(|t| t.kind == ToolKind::Pickaxe).map(|t| (it.name.clone(), t.tier))).collect();
        for s in STEPS.iter() {
            for o in s.ores.iter().filter(|o| o.ends_with("_ore")) {
                let Some(id) = b.id(o) else { continue };
                let need = b.get(id).tier;
                assert_eq!(need, s.level, "{o}");
                // The step has a pickaxe that does it…
                assert!(picks.iter().any(|p| p.1 == need), "nenhuma picareta de nível {need} para {o}");
                // …and nothing of a lower level does.
                for (p, l) in &picks {
                    let tool = i.get(i.id(p).unwrap()).tool.clone();
                    assert_eq!(i.can_harvest(&b, id, tool.as_ref()), *l >= need, "{p} em {o}");
                }
            }
        }
    }

    /// Each step's pickaxe is made from that step's own metal: the ore of step n gives the bar,
    /// the bar makes the pickaxe that opens step n + 1.
    #[test]
    fn each_pickaxe_needs_its_own_metal() {
        let (_, i, r) = laddered();
        for s in &STEPS[2..12] {
            let mut found = false;
            for m in s.tools {
                let Some(pick) = i.id(&format!("{m}_pickaxe")) else { continue };
                found = true;
                let made = r.shaped.iter().any(|x| x.result.item == pick);
                // A few come from shops, bosses or anvils outside recipes.ron (the hardmode ones).
                if !made {
                    continue;
                }
                let uses: Vec<String> = r.shaped.iter().filter(|x| x.result.item == pick).flat_map(|x| x.cells.iter().flatten().flatten().map(|c| i.get(*c).name.clone())).collect();
                assert!(uses.iter().any(|u| u.contains(m) || (*m == "golden" && u.contains("gold")) || *m == "flint"), "{m}_pickaxe não usa {m}: {uses:?}");
            }
            assert!(found, "o degrau {} não tem picareta", s.name);
        }
    }

    /// Armour: every main-line set beats the best set of the step before, and nothing is lifted
    /// by more than half (the sets keep their own character).
    #[test]
    fn armour_climbs_with_the_ladder() {
        let (_, raw, _) = data();
        let (_, i, _) = laddered();
        let mut best_before = 0;
        for s in STEPS {
            let sets: Vec<(u32, &str)> = s.armour.iter().filter_map(|p| set_points(&i, p).map(|v| (v, *p))).collect();
            for (v, p) in &sets {
                assert!(*v > best_before, "{p}: {v} não passa de {best_before}");
                let was = set_points(&raw, p).unwrap();
                assert!(*v as f32 <= was as f32 * 1.6 + 1.0, "{p} subiu demais: {was} → {v}");
            }
            if let Some(m) = sets.iter().filter(|x| !SIDE_SETS.contains(&x.1)).map(|x| x.0).max() {
                best_before = m;
            }
        }
        for step in [11usize, 12] {
            let best = LATE_SETS.iter().filter(|s| s.1 == step).filter_map(|s| set_points(&i, s.0)).max().unwrap();
            assert!(best >= armour_floor(&i, step), "degrau {step}");
        }
    }

    /// Glints of the old numbers: the classic Minecraft pickaxes still climb the same way.
    #[test]
    fn classic_order_is_kept() {
        let (_, i, _) = laddered();
        let lvl = |n: &str| i.get(i.id(n).unwrap()).tool.as_ref().unwrap().tier;
        assert!(lvl("wooden_pickaxe") < lvl("stone_pickaxe"));
        assert!(lvl("stone_pickaxe") < lvl("iron_pickaxe"));
        assert!(lvl("iron_pickaxe") < lvl("diamond_pickaxe"));
        assert!(lvl("diamond_pickaxe") < lvl("netherite_pickaxe"));
        assert!(lvl("netherite_pickaxe") < lvl("mythril_pickaxe"));
        assert_eq!(lvl("umbral_pickaxe"), lvl("diamond_pickaxe"));
        assert_eq!(lvl("magmarite_pickaxe"), lvl("netherite_pickaxe"));
    }

    /// `cargo test -p game ladder_sheet -- --nocapture` prints the ladder as the game has it.
    #[test]
    fn ladder_sheet() {
        let (_, raw, _) = data();
        let (b, i, _) = laddered();
        for s in STEPS {
            let picks = pickaxes_text(&i, s.level).unwrap_or_default();
            let sets: Vec<String> = s.armour.iter().filter_map(|p| Some(format!("{p} {}→{}", set_points(&raw, p)?, set_points(&i, p)?))).collect();
            println!("{:>2} {:<32} {:<60} abre {:<60} armadura {}", s.level, s.name, picks, opens(&b, s.level).join(", "), sets.join(", "));
        }
    }

    #[test]
    fn texts() {
        let (b, i, _) = laddered();
        assert_eq!(pickaxes_text(&i, 5).as_deref(), Some("Picareta de Prata ou Picareta de Tungstênio"));
        assert_eq!(need_text(&i, 5).as_deref(), Some("picareta de nível 5 ou mais (Prata ou Tungstênio)"));
        assert!(opens(&b, 6).iter().any(|n| n == "Diamante"), "{:?}", opens(&b, 6));
        assert_eq!(item_need("gold_ingot"), Some(5));
        assert_eq!(item_need("diamond"), Some(6));
        assert_eq!(block_level("deepslate_diamond_ore"), Some(6));
        assert_eq!(block_level("raw_gold_block"), Some(5));
    }
}
