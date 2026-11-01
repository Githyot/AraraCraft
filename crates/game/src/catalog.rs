//! The item catalogue behind the creative screen and the bag sort: which tab and which section
//! every item belongs to, where it comes from (Minecraft, Terraria, a mod, or our own), the order
//! things are listed in, and the search that finds them.
//!
//! Tabs follow Minecraft Java 26.2's creative tabs (building, coloured, natural, functional and
//! redstone blocks, tools and utilities, combat, food and drinks, ingredients, spawn eggs), with
//! four more on the right side for what the Terraria side and the mods add: magic and summons,
//! accessories and looks, furniture and decoration, and the player's favourites. Inside a tab
//! items come in sections (Madeiras, Pedras, Espadas, Arcos e bestas...), each starting on its
//! own row, and families stay together the way Java lists them (oak log, oak wood, stripped oak
//! log, ..., oak planks, oak stairs, oak slab, oak fence...). Blocks of a mod that fit no
//! material section get a section named after the mod.
//!
//! Search (`matches`): Portuguese or English names, with or without accents, every word must
//! match ("espada diam" finds the diamond sword), plurals work ("espadas"), section and tab names
//! count ("arcos", "poções", "minérios"), and a few everyday words ("luz", "arma", "comida").
//! `@terraria`, `@minecraft`, `@mods`, `@nosso` or `@aether` (any mod's name) keep one origin;
//! `#armaduras` keeps one section or tab.

use std::collections::HashMap;

use voxel::block::{Shape, ToolKind};
use voxel::item::{ItemId, ItemRegistry};
use voxel::BlockRegistry;

use crate::embedded;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Tab {
    Building,
    Colored,
    Natural,
    Functional,
    Redstone,
    Saved,
    Search,
    Tools,
    Combat,
    Food,
    Ingredients,
    Eggs,
    Inventory,
    Magic,
    Accessories,
    Furniture,
    Favorites,
}

/// Tabs on top (7), below (6) and down the right side (4); the index is what `Survival::ctab` keeps.
pub const TABS: [Tab; 17] = [
    Tab::Building,
    Tab::Colored,
    Tab::Natural,
    Tab::Functional,
    Tab::Redstone,
    Tab::Saved,
    Tab::Search,
    Tab::Tools,
    Tab::Combat,
    Tab::Food,
    Tab::Ingredients,
    Tab::Eggs,
    Tab::Inventory,
    Tab::Magic,
    Tab::Accessories,
    Tab::Furniture,
    Tab::Favorites,
];

/// Tabs that list items (all but the saved hotbars, search and the player's own inventory).
pub const ITEM_TABS: [Tab; 14] = [
    Tab::Building,
    Tab::Colored,
    Tab::Natural,
    Tab::Functional,
    Tab::Redstone,
    Tab::Furniture,
    Tab::Tools,
    Tab::Combat,
    Tab::Magic,
    Tab::Accessories,
    Tab::Food,
    Tab::Ingredients,
    Tab::Eggs,
    Tab::Favorites,
];

impl Tab {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Tab::Building => "Blocos de construção",
            Tab::Colored => "Blocos coloridos",
            Tab::Natural => "Blocos naturais",
            Tab::Functional => "Blocos funcionais",
            Tab::Redstone => "Blocos de redstone",
            Tab::Saved => "Barras salvas",
            Tab::Search => "Buscar",
            Tab::Tools => "Ferramentas e utilidades",
            Tab::Combat => "Combate",
            Tab::Food => "Comidas e bebidas",
            Tab::Ingredients => "Ingredientes",
            Tab::Eggs => "Ovos de criatura",
            Tab::Inventory => "Inventário",
            Tab::Magic => "Magia e invocação",
            Tab::Accessories => "Acessórios e visual",
            Tab::Furniture => "Móveis e decoração",
            Tab::Favorites => "Favoritos",
        }
    }

    /// Item shown on the tab (the search tab draws a magnifier).
    pub(crate) fn icon(self) -> &'static str {
        match self {
            Tab::Building => "bricks",
            Tab::Colored => "cyan_wool",
            Tab::Natural => "grass_block",
            Tab::Functional => "crafting_table",
            Tab::Redstone => "redstone",
            Tab::Saved => "bookshelf",
            Tab::Search => "",
            Tab::Tools => "diamond_pickaxe",
            Tab::Combat => "diamond_sword",
            Tab::Food => "apple",
            Tab::Ingredients => "iron_ingot",
            Tab::Eggs => "zombie_spawn_egg",
            Tab::Inventory => "chest",
            Tab::Magic => "amethyst_wand",
            Tab::Accessories => "angel_wings",
            Tab::Furniture => "oak_chair",
            Tab::Favorites => "nether_star",
        }
    }

    /// Sections in the order they are listed; sections not named here (a mod's own, or
    /// "Terraria" for blocks that fit no material) come after these.
    fn sections(self) -> &'static [&'static str] {
        match self {
            Tab::Building => &["Madeiras", "Pedras", "Tijolos e ladrilhos", "Arenitos", "Lama e argila", "Metais e gemas", "Cobre", "Prismarinho", "Nether", "End", "Vidros", "Outros blocos"],
            Tab::Colored => &["Lã", "Carpetes", "Terracota", "Terracota vitrificada", "Concreto", "Concreto em pó", "Vidros coloridos", "Velas", "Estandartes", "Caixas de shulker", "Outros coloridos"],
            Tab::Natural => &["Terra e areia", "Pedras", "Minérios", "Troncos e folhas", "Flores", "Plantas e cogumelos", "Água e corais", "Gelo e neve", "Nether", "End", "Outros naturais"],
            Tab::Functional => &["Iluminação", "Estações de trabalho", "Armazenamento", "Placas", "Cabeças", "Escadas de mão e andaimes", "Outros funcionais"],
            Tab::Redstone => &["Fios e sinais", "Mecanismos", "Trilhos", "Lâmpadas e explosivos", "Armadilhas e sensores", "Outros de redstone"],
            Tab::Furniture => &["Cadeiras e bancos", "Mesas", "Estantes e armários", "Camas", "Estátuas e troféus", "Vasos e enfeites", "Quadros"],
            Tab::Tools => &["Picaretas", "Machados", "Pás", "Enxadas", "Ganchos", "Pesca", "Baldes", "Transporte", "Navegação e informação", "Música", "Invocar chefes", "Sacolas e caixas", "Chaves", "Mochilas e cofres", "Construção", "Diversão", "Utilidades"],
            Tab::Combat => &["Espadas", "Lanças e alabardas", "Corpo a corpo", "Arcos e bestas", "Flechas e munição", "Armas de fogo", "Arremesso", "Escudos", "Armaduras", "Armaduras Terraria", "Armaduras de mods", "Armaduras de animais", "Outros de combate"],
            Tab::Magic => &["Armas mágicas", "Invocação", "Tomos e pergaminhos", "Glifos", "Mana"],
            Tab::Accessories => &["Movimento", "Atributos e combate", "Defesa e imunidades", "Informação", "Acessórios", "Asas", "Mascotes", "Montarias", "Roupas (visual)", "Tinturas"],
            Tab::Food => &["Frutas e vegetais", "Carnes e peixes", "Pratos e doces", "Bebidas", "Poções", "Poções arremessáveis", "Poções persistentes", "Poções do Terraria", "Melhorias permanentes"],
            Tab::Ingredients => &["Minérios brutos e lingotes", "Gemas e cristais", "Moedas", "Materiais básicos", "Materiais de criaturas", "Ingredientes de poções", "Corantes", "Livros e papel", "Moldes de forja", "Fragmentos de cerâmica", "Peixes de pesca", "Outros ingredientes"],
            Tab::Eggs => &["Minecraft", "AraraCraft", "Terraria"],
            _ => &[],
        }
    }
}

/// Where an item comes from.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Origin {
    Minecraft,
    Ours,
    Terraria,
    /// Index into `MODS`.
    Mod(u8),
    Plugin,
}

/// Mod modules of tools/moddata/ and the name the player sees.
pub const MODS: &[(&str, &str)] = &[
    ("aether", "The Aether"),
    ("aircraft", "Immersive Aircraft"),
    ("apotheosis", "Apotheosis"),
    ("aquaculture", "Aquaculture"),
    ("better_combat", "Better Combat"),
    ("bomd", "Bosses of Mass Destruction"),
    ("distant_lands", "Distant Lands"),
    ("ecologics", "Ecologics"),
    ("immersive_melodies", "Immersive Melodies"),
    ("macaws", "Macaw's"),
    ("otherside", "Deeper and Darker"),
    ("seasons", "Serene Seasons"),
    ("silent_gear", "Silent Gear"),
    ("snow_real_magic", "Snow! Real Magic!"),
    ("spellbooks", "Iron's Spells"),
    ("yungs_caves", "YUNG's Cave Biomes"),
    ("zz_dim_eggs", "Aether e Otherside"),
    ("zz_furniture", "Móveis das madeiras"),
    ("zz_mobs", "Criaturas (Alex's Mobs...)"),
];

impl Origin {
    pub fn label(self) -> &'static str {
        match self {
            Origin::Minecraft => "Minecraft",
            Origin::Ours => "AraraCraft",
            Origin::Terraria => "Terraria",
            Origin::Mod(i) => MODS.get(i as usize).map(|m| m.1).unwrap_or("Mod"),
            Origin::Plugin => "Plugin",
        }
    }

    /// The tooltip line: "Mod: The Aether", "Terraria"...
    pub fn tooltip(self) -> String {
        match self {
            Origin::Mod(_) => format!("Mod: {}", self.label()),
            o => o.label().to_string(),
        }
    }

    /// Words `@...` in a search matches.
    fn keys(self) -> String {
        match self {
            Origin::Minecraft => "minecraft mc vanilla".into(),
            Origin::Ours => "araracraft arara nosso nossos original".into(),
            Origin::Terraria => "terraria tr".into(),
            Origin::Mod(i) => format!("mods mod {} {}", norm(MODS[i as usize].0), norm(MODS[i as usize].1)),
            Origin::Plugin => "plugin plugins".into(),
        }
    }
}

/// Origin filter of the creative screen (button "Origem").
pub const ORIGIN_FILTERS: [&str; 5] = ["Tudo", "Minecraft", "Terraria", "Mods", "AraraCraft"];

pub fn origin_passes(filter: usize, o: Origin) -> bool {
    match filter {
        1 => o == Origin::Minecraft,
        2 => o == Origin::Terraria,
        3 => matches!(o, Origin::Mod(_) | Origin::Plugin),
        4 => o == Origin::Ours,
        _ => true,
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub tab: Tab,
    /// Index into the tab's `Catalog::sections`.
    pub sec: u16,
    pub origin: Origin,
    /// Place in its tab (lower first).
    pub order: u64,
    /// Lower-case, accent-free text the search looks in.
    hay: String,
    /// The display name, normalised (for "starts with" ranking and A-Z order).
    pub key: String,
}

pub struct Catalog {
    /// Per item id; None for items no tab shows.
    pub entries: Vec<Option<Entry>>,
    pub sections: HashMap<Tab, Vec<String>>,
    /// Items of each tab in catalogue order.
    pub lists: HashMap<Tab, Vec<ItemId>>,
    /// How many items the registry had when this was built (plugins add more later).
    pub built_for: usize,
}

/// Lower-case, without accents, `_` as a space.
pub fn norm(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars().flat_map(|c| c.to_lowercase()) {
        o.push(match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            '_' | '-' => ' ',
            c => c,
        });
    }
    o
}

/// Every `name: "..."` in a data file.
fn names_in(text: &str) -> impl Iterator<Item = &str> {
    text.split("name: \"").skip(1).filter_map(|s| s.split('"').next())
}

/// Where each item name comes from, by the data file that first names it (load order).
fn origins() -> HashMap<String, Origin> {
    let mut m: HashMap<String, Origin> = HashMap::new();
    let files: [(&str, Origin); 17] = [
        (embedded::BLOCKS_RON, Origin::Minecraft),
        (embedded::ITEMS_RON, Origin::Minecraft),
        (embedded::FASE6_BLOCKS_RON, Origin::Minecraft),
        (embedded::FASE6_ITEMS_RON, Origin::Minecraft),
        (embedded::T5_BLOCKS_RON, Origin::Terraria),
        (embedded::T5_ITEMS_RON, Origin::Terraria),
        (embedded::DEEP_BLOCKS_RON, Origin::Ours),
        (embedded::DEEP_ITEMS_RON, Origin::Ours),
        (embedded::TR_BLOCKS_RON, Origin::Terraria),
        (embedded::TR_ITEMS_RON, Origin::Terraria),
        (embedded::AGUAS_BLOCKS_RON, Origin::Ours),
        (embedded::VULCAO_BLOCKS_RON, Origin::Ours),
        (embedded::VULCAO_ITEMS_RON, Origin::Ours),
        (embedded::TR3_BLOCKS_RON, Origin::Terraria),
        (embedded::TR3_ITEMS_RON, Origin::Terraria),
        (embedded::RPG_ITEMS_RON, Origin::Terraria),
        (embedded::LOTE11_ITEMS_RON, Origin::Terraria),
    ];
    for (text, o) in files {
        for n in names_in(text) {
            m.entry(n.to_string()).or_insert(o);
        }
    }
    // The mod pack: tools/gen_mod_data.py writes which module made each name.
    if let Ok(by_mod) = ron::from_str::<HashMap<String, Vec<String>>>(embedded::MODS_ORIGINS_RON) {
        for (module, names) in by_mod {
            let Some(i) = MODS.iter().position(|m| m.0 == module) else { continue };
            for n in names {
                m.entry(n).or_insert(Origin::Mod(i as u8));
            }
        }
    }
    m
}

const COLORS: [&str; 16] = [
    "white", "light_gray", "gray", "black", "brown", "red", "orange", "yellow", "lime", "green", "cyan", "light_blue", "blue", "purple", "magenta", "pink",
];

/// Woods, so their blocks land in "Madeiras" (mod woods too).
const WOODS: &[&str] = &[
    "oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "mangrove", "cherry", "pale_oak", "bamboo", "crimson", "warped", "skyroot", "golden_oak", "ipe", "araucaria", "buriti",
    "jabuticaba", "coconut", "walnut", "maple", "echo", "palm", "ebonwood", "shadewood", "pearlwood", "boreal", "rich_mahogany", "mahogany", "dynasty", "spooky",
];

/// Suffixes in the order a family is listed (Java's order for a wood set).
const VARIANTS: &[&str] = &[
    "_log", "_stem", "_wood", "_hyphae", "_planks", "_mosaic", "_stairs", "_slab", "_fence_gate", "_fence", "_wall", "_door", "_trapdoor", "_pressure_plate", "_button", "_hanging_sign", "_sign",
    "_window", "_roof", "_chest_boat", "_boat", "_raft", "_leaves", "_sapling", "_helmet", "_cap", "_hat", "_hood", "_chestplate", "_tunic", "_robe", "_leggings", "_pants", "_boots", "_shoes", "_pane",
];

fn has(n: &str, keys: &[&str]) -> bool {
    keys.iter().any(|k| n.contains(k))
}

fn ends(n: &str, keys: &[&str]) -> bool {
    keys.iter().any(|k| n.ends_with(k))
}

/// The family a name belongs to and its place in the family: "stripped_oak_log" → ("oak", 2).
fn family(name: &str) -> (String, usize) {
    let stripped = name.strip_prefix("stripped_");
    let n = stripped.unwrap_or(name);
    for (i, v) in VARIANTS.iter().enumerate() {
        if let Some(base) = n.strip_suffix(v) {
            // Stripped logs come after the wood, like Java.
            return (base.to_string(), i * 2 + 1 + usize::from(stripped.is_some()) * 4);
        }
    }
    (n.to_string(), 0)
}

/// What the catalogue needs to know about an item beyond the registry.
struct Tables {
    gear: HashMap<&'static str, (crate::balance::Class, u8)>,
    acc: Vec<&'static str>,
    wings: Vec<&'static str>,
    pets: Vec<&'static str>,
    mounts: Vec<&'static str>,
    dyes: Vec<&'static str>,
    summon_staffs: Vec<&'static str>,
    hidden: Vec<&'static str>,
    hooks: Vec<&'static str>,
    boss_summons: Vec<&'static str>,
    rods: Vec<&'static str>,
    baits: Vec<&'static str>,
    arrows: Vec<&'static str>,
}

impl Tables {
    fn new() -> Tables {
        let mut t = Tables {
            gear: crate::balance::GEAR.iter().map(|g| (g.0, (g.1, g.2))).collect(),
            acc: crate::rpg::ACCESSORIES.iter().map(|a| a.name).collect(),
            wings: crate::wings::WINGS.iter().map(|w| w.item).chain(crate::tr_pets::WINGS.iter().map(|w| w.0)).collect(),
            pets: crate::companions::PETS.iter().map(|p| p.item).chain(crate::tr_pets::PETS.iter().map(|p| p.0)).collect(),
            mounts: crate::companions::MOUNTS.iter().map(|m| m.item).chain(crate::tr_pets::MOUNTS.iter().map(|m| m.0)).collect(),
            dyes: crate::vanity::DYES.iter().map(|d| d.0).collect(),
            summon_staffs: crate::rpg::MINIONS.iter().map(|m| m.staff).chain(crate::sentries::SENTRIES.iter().map(|s| s.staff)).collect(),
            hidden: crate::rpg::MINIONS.iter().map(|m| m.sprite).chain(crate::sentries::SENTRIES.iter().map(|s| s.sprite)).collect(),
            hooks: crate::hook::HOOKS.iter().map(|h| h.0).collect(),
            boss_summons: crate::bosses::SUMMONS.iter().map(|s| s.0).chain(crate::tr_bosses::SUMMONS.iter().map(|s| s.0)).chain(crate::boss_ai::SUMMONS.iter().map(|s| s.0)).collect(),
            rods: crate::tr_fishing::RODS.iter().map(|r| r.0).collect(),
            baits: crate::tr_fishing::BAITS.iter().map(|b| b.0).collect(),
            arrows: crate::tr3_tools::ARROWS.iter().map(|a| a.0).collect(),
        };
        t.summon_staffs.extend(["leather_whip", "thorn_whip", "bone_whip"]);
        t
    }
}

/// Tab and section of one item; None when no tab shows it.
fn classify(reg: &BlockRegistry, items: &ItemRegistry, id: ItemId, origin: Origin, t: &Tables, listed: &std::collections::HashSet<String>) -> Option<(Tab, String)> {
    use crate::balance::Class;
    let it = items.get(id);
    let n = it.name.as_str();
    let sec = |tab: Tab, s: &str| Some((tab, s.to_string()));
    // Pictures of minions and turrets, the bobber and Silent Gear's assembled tools are not items to hand out.
    if n == "fishing_bobber" || t.hidden.contains(&n) || n.starts_with("minion_") || n.starts_with("gear_") && it.tool.is_some() && !n.ends_with("_stick_string") {
        return None;
    }
    if n.ends_with("_spawn_egg") {
        return Some((Tab::Eggs, origin.label().to_string()));
    }
    // Blocks that only exist placed (orientations, lit forms) stay out, like Java.
    if let Some(b) = it.places {
        let bl = reg.get(b);
        if !bl.in_inventory && !matches!(it.look, voxel::item::Look::Sprite(_)) && !listed.contains(n) {
            return None;
        }
    }
    // Weapons by the class their stats use.
    if t.rods.contains(&n) || t.baits.contains(&n) || ends(n, &["_bait", "fishing_line", "strong_line"]) || n.contains("fishing_rod") || matches!(origin, Origin::Mod(3)) && ends(n, &["_hook", "worm"]) {
        return sec(Tab::Tools, "Pesca");
    }
    if n == "brush" {
        return sec(Tab::Tools, "Construção");
    }
    if let Some(&(class, _)) = t.gear.get(n).filter(|_| !n.starts_with("music_disc")) {
        return match class {
            Class::Warrior if has(n, &["spear", "halberd", "glaive", "lance", "javelin", "trident", "pike"]) => sec(Tab::Combat, "Lanças e alabardas"),
            Class::Warrior if it.tool.as_ref().is_some_and(|t| t.kind == ToolKind::Sword) => sec(Tab::Combat, "Espadas"),
            Class::Warrior if it.tool.as_ref().is_some_and(|t| t.kind == ToolKind::Axe) && !has(n, &["war_axe", "battle"]) => sec(Tab::Tools, "Machados"),
            Class::Warrior if it.tool.as_ref().is_some_and(|t| t.kind == ToolKind::Pickaxe) => sec(Tab::Tools, "Picaretas"),
            Class::Warrior => sec(Tab::Combat, "Corpo a corpo"),
            Class::Archer => sec(Tab::Combat, "Arcos e bestas"),
            Class::Gunner => sec(Tab::Combat, "Armas de fogo"),
            Class::Assassin => sec(Tab::Combat, "Assassino"),
            Class::Mage => sec(Tab::Magic, "Armas mágicas"),
            Class::Summoner => sec(Tab::Magic, "Invocação"),
        };
    }
    if t.summon_staffs.contains(&n) {
        return sec(Tab::Magic, "Invocação");
    }
    if let Some(tool) = &it.tool {
        return match tool.kind {
            ToolKind::Sword if has(n, &["spear", "halberd", "glaive", "lance", "trident", "pike"]) => sec(Tab::Combat, "Lanças e alabardas"),
            ToolKind::Sword => sec(Tab::Combat, "Espadas"),
            ToolKind::Pickaxe => sec(Tab::Tools, "Picaretas"),
            ToolKind::Axe => sec(Tab::Tools, "Machados"),
            ToolKind::Shovel => sec(Tab::Tools, "Pás"),
            ToolKind::Hoe => sec(Tab::Tools, "Enxadas"),
        };
    }
    if t.wings.contains(&n) || n.ends_with("_wings") {
        return sec(Tab::Accessories, "Asas");
    }
    if t.acc.contains(&n) {
        let s = if has(n, &["boots", "fins", "claws", "climbing", "balloon", "bottle", "horseshoe", "cape", "cloak", "parachute", "flippers", "skates"]) {
            "Movimento"
        } else if has(n, &["meter", "watch", "radar", "finder", "vane", "clock", "compass", "tally", "map", "kit", "lens"]) && !has(n, &["marksman"]) {
            "Informação"
        } else if has(n, &["shield", "ward", "scarf", "ankh", "buckler", "turtle", "charm", "veil", "mirror", "bandage", "medicine", "polish", "vitamin", "obsidian", "honey", "frog", "mask"]) {
            "Defesa e imunidades"
        } else if has(n, &["emblem", "glove", "gauntlet", "quiver", "lens", "stone", "ring", "necklace", "pendant", "cuffs", "band", "blossom", "gland", "heart", "socket"]) {
            "Atributos e combate"
        } else {
            "Acessórios"
        };
        return sec(Tab::Accessories, s);
    }
    if t.pets.contains(&n) {
        return sec(Tab::Accessories, "Mascotes");
    }
    if t.mounts.contains(&n) {
        return sec(Tab::Accessories, "Montarias");
    }
    if t.dyes.contains(&n) {
        return sec(Tab::Accessories, "Tinturas");
    }
    if n.starts_with("vanity_") || n == "vanity_mirror" {
        return sec(Tab::Accessories, if n == "vanity_mirror" { "Acessórios" } else { "Roupas (visual)" });
    }
    if it.armor.is_some() {
        if has(n, &["horse_armor", "wolf_armor"]) {
            return sec(Tab::Combat, "Armaduras de animais");
        }
        return sec(
            Tab::Combat,
            match origin {
                Origin::Terraria => "Armaduras Terraria",
                Origin::Mod(_) | Origin::Plugin => "Armaduras de mods",
                _ => "Armaduras",
            },
        );
    }
    if t.hooks.contains(&n) || n.ends_with("_hook") && it.places.is_none() && origin != Origin::Mod(3) {
        return sec(Tab::Tools, "Ganchos");
    }
    if t.boss_summons.contains(&n) {
        return sec(Tab::Tools, "Invocar chefes");
    }
    if n.starts_with("tipped_arrow") || t.arrows.contains(&n) || ends(n, &["arrow", "_bolt_ammo", "musket_ball", "_shot", "_bullet", "rocket"]) && n != "firework_rocket" {
        return sec(Tab::Combat, "Flechas e munição");
    }
    // Potions before food (some potions are also drinks).
    if n.starts_with("splash_potion") {
        return sec(Tab::Food, "Poções arremessáveis");
    }
    if n.starts_with("lingering_potion") {
        return sec(Tab::Food, "Poções persistentes");
    }
    if n.starts_with("potion_") {
        return sec(Tab::Food, "Poções");
    }
    if n.ends_with("_potion") || n == "restoration_potion" {
        return sec(Tab::Food, "Poções do Terraria");
    }
    if ["life_fruit", "aegis_fruit", "ancestral_heart", "celestial_socket", "mana_crystal", "vital_crystal", "arcane_shard", "gummy_worm", "artisan_loaf", "galaxy_pearl", "ambrosia", "torch_gods_favor", "demon_heart"].contains(&n) {
        return sec(Tab::Food, "Melhorias permanentes");
    }
    if it.food.is_some() || has(n, &["honey_bottle", "milk_bucket"]) {
        let s = if has(n, &["bottle", "milk", "juice", "tea", "coffee", "ale", "drink", "soda", "wine", "mead", "cocoa_cup", "smoothie"]) {
            "Bebidas"
        } else if has(n, &["beef", "pork", "chicken", "mutton", "rabbit", "cod", "salmon", "fish", "steak", "meat", "fillet", "tropical", "puffer", "crab", "shrimp", "rotten_flesh", "spider_eye", "egg", "bacon", "sausage", "trout", "carp", "eel", "squid", "lobster", "koi", "minnow", "guppy", "sunfish", "jerky", "venison", "calamari", "sushi", "tilapia", "piranha", "pirarucu", "arctic_char", "perch", "tuna", "bass", "bluegill", "halibut", "pollock", "grouper", "gar", "catfish"]) {
            "Carnes e peixes"
        } else if has(n, &["apple", "carrot", "potato", "beetroot", "melon", "berr", "fruit", "coconut", "walnut", "chorus", "kelp", "mango", "banana", "jabuticaba", "pinhao", "buriti", "acai", "pear", "peach", "lemon", "orange", "grape", "cherry", "plum", "truffle", "mushroom", "seed", "nut", "pumpkin", "corn", "tomato", "lettuce", "onion", "pepper", "pineapple", "fig", "lime", "date", "olive", "acorn", "cactus"]) && !has(n, &["pie", "stew", "soup", "cake", "cookie", "salad", "bread", "golden_"]) {
            "Frutas e vegetais"
        } else {
            "Pratos e doces"
        };
        return sec(Tab::Food, s);
    }
    // Blocks.
    if let Some(b) = it.places {
        let bl = reg.get(b);
        return Some(block_section(n, bl, origin));
    }
    // Plain items by name.
    let s: (Tab, &str) = if n.starts_with("gear_") && origin != Origin::Terraria {
        return Some((Tab::Ingredients, origin.label().to_string()));
    } else if n == "wolf_armor" || n.ends_with("horse_armor") {
        (Tab::Combat, "Armaduras de animais")
    } else if ["bomb", "dynamite", "sticky_bomb", "grenade"].contains(&n) {
        (Tab::Combat, "Arremesso")
    } else if has(n, &["medallion", "battle_standard", "_summon", "snow_globe"]) {
        (Tab::Tools, "Invocar chefes")
    } else if ["stick", "flint", "brick", "nether_brick", "resin_brick", "clay_ball", "wheat", "coal", "charcoal", "bowl", "popped_chorus_fruit", "resin_clump", "skyroot_stick", "sulfur", "driftwood", "tin_can", "spooky_wood", "sturdy_fossil", "broken_hero_sword"].contains(&n) {
        (Tab::Ingredients, "Materiais básicos")
    } else if origin == Origin::Terraria && has(n, &["king", "minnow", "eel", "trout", "lantern_fish", "carp", "guppy", "sunfish", "koi", "obsidian_fish"]) {
        (Tab::Ingredients, "Peixes de pesca")
    } else if n.starts_with("glyph_") {
        (Tab::Magic, "Glifos")
    } else if has(n, &["scroll", "tome", "spellbook", "grimoire", "glyph_book", "arcane_ink", "arcane_essence", "blank_book"]) || n.starts_with("sigil_") && origin == Origin::Mod(14) {
        (Tab::Magic, "Tomos e pergaminhos")
    } else if has(n, &["_wand", "_staff", "magic_missile"]) {
        (Tab::Magic, "Armas mágicas")
    } else if n.starts_with("mana_") || has(n, &["arcane_shard", "fallen_star", "ether_crystal"]) {
        (Tab::Magic, "Mana")
    } else if n == "shield" || n.ends_with("_shield") {
        (Tab::Combat, "Escudos")
    } else if has(n, &["totem_of_undying", "wind_charge", "snowball", "trap_rod", "aura_rod", "flame_sentry_rod", "rifle_scope", "sharpening"]) || n == "egg" {
        (Tab::Combat, "Outros de combate")
    } else if has(n, &["bucket", "sponge"]) && !n.ends_with("_block") {
        (Tab::Tools, "Baldes")
    } else if has(n, &["boat", "minecart", "saddle", "airship", "biplane", "gyrodyne", "quadrocopter", "warship", "parachute", "aircraft_", "hull", "propeller", "engine", "rotary_cannon", "golf_cart", "witchs_broom", "reindeer_bells", "drill_containment", "carrot_on_a_stick", "_raft"]) {
        (Tab::Tools, "Transporte")
    } else if has(n, &["compass", "clock", "map", "spyglass", "atlas", "depth_meter", "pda", "radar", "magic_mirror", "conch", "calendar", "bestiary", "recall", "teleport"]) && !n.ends_with("_potion") {
        (Tab::Tools, "Navegação e informação")
    } else if has(n, &["music_disc", "lute", "flute", "cello", "didgeridoo", "bagpipe", "piano", "drum", "handpan", "ancient_horn", "goat_horn"]) {
        (Tab::Tools, "Música")
    } else if n.starts_with("bag_") || has(n, &["_crate", "present", "goodie_bag", "party_kit"]) {
        (Tab::Tools, "Sacolas e caixas")
    } else if n.ends_with("_key") || n.starts_with("key_of") || n.ends_with("trial_key") {
        (Tab::Tools, "Chaves")
    } else if has(n, &["backpack", "void_bag", "money_trough", "piggy"]) {
        (Tab::Tools, "Mochilas e cofres")
    } else if has(n, &["paint_", "toolbelt", "toolbox", "extendo", "architect", "rod_of_discord", "bug_net", "wrench", "sprayer", "purification_powder", "vile_powder", "solution", "flint_and_steel", "fire_charge", "shears", "brush", "torch_gods"]) {
        (Tab::Tools, "Construção")
    } else if has(n, &["kite_", "golf_", "confetti", "_license", "firework_rocket", "voodoo", "daring_stone", "explorer_hat"]) {
        (Tab::Tools, "Diversão")
    } else if has(n, &["lead", "name_tag", "armor_stand", "writable_book", "written_book", "bundle", "experience_bottle", "ender_eye", "ender_pearl", "item_frame", "painting"]) && !has(n, &["lead_ingot", "raw_lead", "lead_nugget"]) {
        (Tab::Tools, "Utilidades")
    } else if n.starts_with("raw_") || ends(n, &["_ingot", "_nugget", "netherite_scrap", "_scrap"]) {
        (Tab::Ingredients, "Minérios brutos e lingotes")
    } else if n.ends_with("_coin") {
        (Tab::Ingredients, "Moedas")
    } else if n.ends_with("_dye") || n == "ink_sac" || n == "glow_ink_sac" || n == "cocoa_beans" || n == "bone_meal" {
        (Tab::Ingredients, "Corantes")
    } else if n.ends_with("smithing_template") {
        (Tab::Ingredients, "Moldes de forja")
    } else if n.ends_with("pottery_sherd") {
        (Tab::Ingredients, "Fragmentos de cerâmica")
    } else if has(n, &["book", "paper", "parchment"]) {
        (Tab::Ingredients, "Livros e papel")
    } else if has(n, &["glass_bottle", "sugar", "fermented", "glistering", "rabbit_foot", "blaze_powder", "magma_cream", "glowstone_dust", "phantom_membrane", "dragon_breath", "ghast_tear", "nether_wart", "breeze_rod", "_powder", "ominous_bottle"]) {
        (Tab::Ingredients, "Ingredientes de poções")
    } else if has(n, &["diamond", "emerald", "ruby", "sapphire", "topaz", "amethyst", "quartz", "lapis", "gem", "crystal", "shard", "prism", "amber", "pearl", "echo", "heart_of", "nether_star", "lens", "fragment", "core", "idol", "tablet", "sigil", "salvage", "resonarium"]) {
        (Tab::Ingredients, "Gemas e cristais")
    } else if has(n, &["bone", "string", "leather", "feather", "slime", "gel", "scale", "membrane", "tear", "hide", "scute", "fang", "fur", "wing", "tooth", "skin", "proboscis", "shell", "horn", "tissue", "gland", "soul_", "ectoplasm", "rod", "cream", "spine", "comb", "beeswax", "dust", "honeycomb", "husk", "mandible", "spore", "vine", "petal", "egg", "head", "gunpowder", "flint", "stick", "clay_ball", "brick", "wheat", "eye", "fin", "claw", "silk", "web", "cobweb", "venom", "blood", "thorn", "flesh", "critter_"]) {
        (Tab::Ingredients, "Materiais de criaturas")
    } else {
        (Tab::Ingredients, "Outros ingredientes")
    };
    // Things of a mod that land in the catch-all go under the mod's name.
    if s.1.starts_with("Outros") {
        if let Origin::Mod(_) | Origin::Plugin = origin {
            return Some((s.0, origin.label().to_string()));
        }
    }
    sec(s.0, s.1)
}

fn block_section(n: &str, bl: &voxel::block::Block, origin: Origin) -> (Tab, String) {
    let out = |tab: Tab, s: &str| (tab, s.to_string());
    let modded = matches!(origin, Origin::Mod(_) | Origin::Plugin);
    let by_mod = |tab: Tab, fallback: &str| if modded { (tab, origin.label().to_string()) } else { (tab, fallback.to_string()) };
    let shape = bl.shape;
    // Redstone.
    let redstone = (matches!(shape, Shape::Wire | Shape::Lever | Shape::Button | Shape::Plate | Shape::Repeater | Shape::Piston | Shape::Comparator | Shape::Hopper | Shape::Rail)
        || has(n, &["redstone", "copper_bulb", "observer", "target", "daylight_detector", "tripwire", "note_block", "sculk_sensor", "dispenser", "dropper", "crafter", "lightning_rod", "dart_", "_timer", "logic_", "teleporter", "dungeon_spikes"])
        || n.ends_with("_trap")
        || n == "tnt")
        && n != "redstone_ore"
        && !n.ends_with("deepslate_redstone_ore");
    if redstone {
        let s = if shape == Shape::Rail || n.contains("rail") {
            "Trilhos"
        } else if matches!(shape, Shape::Wire | Shape::Lever | Shape::Button | Shape::Plate) || has(n, &["redstone_torch", "redstone_block", "target", "daylight", "tripwire", "sculk_sensor", "lightning_rod"]) || n == "redstone" {
            "Fios e sinais"
        } else if matches!(shape, Shape::Piston | Shape::Repeater | Shape::Comparator | Shape::Hopper) || has(n, &["observer", "dispenser", "dropper", "crafter", "note_block"]) {
            "Mecanismos"
        } else if has(n, &["lamp", "tnt", "bulb"]) {
            "Lâmpadas e explosivos"
        } else if has(n, &["trap", "dart", "timer", "logic", "sensor", "teleporter", "spikes"]) {
            "Armadilhas e sensores"
        } else {
            "Outros de redstone"
        };
        return out(Tab::Redstone, s);
    }
    // Furniture and decoration.
    if matches!(shape, Shape::Chair | Shape::Bench) || has(n, &["_chair", "_stool", "_sofa", "_throne", "_bench"]) && !has(n, &["work", "smithing"]) {
        return out(Tab::Furniture, "Cadeiras e bancos");
    }
    if shape == Shape::Table || has(n, &["_table", "_desk"]) && !has(n, &["crafting", "enchanting", "cartography", "fletching", "smithing", "war_table", "bewitching", "salvag", "apothe", "gem_cut", "altar", "work"]) {
        return out(Tab::Furniture, "Mesas");
    }
    if shape == Shape::Shelf || has(n, &["_drawer", "cabinet", "dresser", "wardrobe", "bookcase", "cupboard", "_shelf"]) {
        return out(Tab::Furniture, "Estantes e armários");
    }
    if shape == Shape::Bed || n.ends_with("_bed") {
        return out(Tab::Furniture, "Camas");
    }
    if has(n, &["statue", "trophy", "monolith", "bust", "relic", "gargoyle"]) {
        return out(Tab::Furniture, "Estátuas e troféus");
    }
    if has(n, &["painting", "item_frame", "_frame_deco", "_rug", "tapestry", "mirror"]) && !n.ends_with("portal_frame") {
        return out(Tab::Furniture, "Quadros");
    }
    if matches!(shape, Shape::Pot) || has(n, &["flower_pot", "vase", "decorated_pot", "_urn", "globe", "chime", "fountain", "jar", "candelabra", "bowl_deco", "_clock", "gnome"]) {
        return out(Tab::Furniture, "Vasos e enfeites");
    }
    // Coloured blocks: Java lists them by kind, then colour.
    let colored = COLORS.iter().any(|c| n.starts_with(&format!("{c}_"))) && shape != Shape::Cross && !n.ends_with("_tulip") && n != "red_sand" && n != "red_sandstone" && !n.starts_with("red_nether") && !n.starts_with("red_mushroom") && !n.starts_with("brown_mushroom")
        && !has(n, &["ice", "soil", "aercloud", "_flower", "_berry"]);
    if colored || has(n, &["glass", "candle", "wool", "terracotta", "concrete", "shulker_box"]) || matches!(shape, Shape::Banner | Shape::TallBanner) {
        let s = if n.ends_with("_carpet") {
            "Carpetes"
        } else if n.ends_with("_wool") {
            "Lã"
        } else if n.contains("glazed_terracotta") {
            "Terracota vitrificada"
        } else if n.contains("terracotta") {
            "Terracota"
        } else if n.ends_with("concrete_powder") {
            "Concreto em pó"
        } else if n.ends_with("concrete") {
            "Concreto"
        } else if n.contains("stained_glass") {
            "Vidros coloridos"
        } else if n.contains("glass") {
            return out(Tab::Building, "Vidros");
        } else if n.contains("candle") {
            "Velas"
        } else if n.contains("banner") || matches!(shape, Shape::Banner | Shape::TallBanner) {
            "Estandartes"
        } else if n.contains("shulker_box") {
            "Caixas de shulker"
        } else if colored {
            "Outros coloridos"
        } else {
            return by_mod(Tab::Building, "Outros blocos");
        };
        return out(Tab::Colored, s);
    }
    // Functional blocks.
    if matches!(shape, Shape::Torch | Shape::Lantern) || has(n, &["torch", "lantern", "lamp", "campfire", "candle", "glowstone", "sea_lantern", "froglight", "shroomlight", "light_block", "chandelier", "sconce"]) && !has(n, &["torchflower", "sniffer_egg", "heavy_core", "conduit", "coconut"]) {
        return out(Tab::Functional, "Iluminação");
    }
    if has(n, &["crafting", "furnace", "smoker", "anvil", "smithing", "grindstone", "stonecutter", "cartography", "fletching", "brewing", "enchanting", "cauldron", "composter", "lectern", "workbench", "forge", "altar", "sawmill", "keg", "station", "_press", "war_table", "bewitching", "salvag", "gem_cutting", "inscription", "incubator", "freezer", "kiln", "crystal_ball", "ammo_box", "sundial", "extractinator", "hellforge", "dye_vat", "cooking_pot", "bottle", "imbuing", "tinkerer", "autohammer", "mythril_anvil", "orichalcum_anvil", "ancient_manipulator", "lunar"]) && !has(n, &["ore", "brick", "glass"]) || n == "loom" {
        return out(Tab::Functional, "Estações de trabalho");
    }
    if has(n, &["chest", "barrel", "crate", "safe", "piggy_bank", "bank", "hopper_box", "vault", "storage"]) {
        return out(Tab::Functional, "Armazenamento");
    }
    if shape == Shape::Sign || n.ends_with("_sign") {
        return out(Tab::Functional, "Placas");
    }
    if shape == Shape::Skull || has(n, &["_head", "skull"]) {
        return out(Tab::Functional, "Cabeças");
    }
    if shape == Shape::Ladder || has(n, &["ladder", "scaffolding", "rope"]) {
        return out(Tab::Functional, "Escadas de mão e andaimes");
    }
    if matches!(shape, Shape::Beacon | Shape::Portal | Shape::Anvil | Shape::Cage | Shape::Orb) || has(n, &["beacon", "conduit", "respawn_anchor", "lodestone", "bell", "jukebox", "spawner", "portal", "end_rod", "beehive", "bee_nest", "cake", "heart_lantern", "banner_stand", "pylon", "sunflower_vase", "chain", "cobweb", "sponge", "command_block", "heavy_core", "life_crystal", "golf_", "bookshelf", "windmill", "creaking_heart", "sculk_shrieker", "sculk_catalyst", "dragon_egg"]) {
        if !matches!(shape, Shape::Cube) || has(n, &["beacon", "conduit", "respawn", "lodestone", "jukebox", "spawner", "beehive", "pylon", "command_block", "bookshelf", "life_crystal", "creaking_heart", "sculk_shrieker", "sculk_catalyst"]) {
            return out(Tab::Functional, "Outros funcionais");
        }
    }
    // Natural blocks.
    let natural_names = [
        "grass_block", "dirt", "coarse_dirt", "podzol", "mycelium", "rooted_dirt", "farmland", "dirt_path", "sand", "red_sand", "gravel", "mud", "clay", "stone", "deepslate", "tuff",
        "calcite", "granite", "diorite", "andesite", "bedrock", "obsidian", "crying_obsidian", "netherrack", "soul_sand", "soul_soil", "basalt", "blackstone", "magma_block", "end_stone",
        "ice", "packed_ice", "blue_ice", "snow", "snow_block", "powder_snow", "moss_block", "moss_carpet", "pale_moss_block", "pale_moss_carpet", "dripstone_block", "pointed_dripstone",
        "amethyst_block", "budding_amethyst", "cactus", "pumpkin", "melon", "lily_pad", "sugar_cane", "bamboo", "hay_block", "sponge", "wet_sponge", "sculk", "glowstone", "ancient_debris",
        "honey_block", "bee_nest", "mushroom_stem", "red_mushroom_block", "brown_mushroom_block", "nether_wart_block", "warped_wart_block", "chorus_plant", "chorus_flower", "dried_kelp_block",
        "cobweb", "suspicious_sand", "suspicious_gravel", "infested_stone", "sniffer_egg", "turtle_egg", "frogspawn", "mossy_cobblestone", "carved_pumpkin", "jack_o_lantern",
    ];
    let natural = matches!(shape, Shape::Cross | Shape::Model | Shape::Crystal | Shape::Layer)
        || natural_names.contains(&n)
        || ends(n, &["_ore", "_log", "_stem", "_leaves", "_sapling", "_roots", "_nylium", "_coral", "_coral_block", "_coral_fan", "_mushroom", "_fungus", "_propagule", "_bud", "_cluster", "_moss", "_grass", "_vines", "_vine", "_hyphae"])
        || n.starts_with("raw_") && n.ends_with("_block")
        || has(n, &["_ore", "infested_", "grass_block", "_dirt", "_sand", "glow_mushroom_block", "_soil"]) && !has(n, &["sandstone", "brick", "polished", "smooth", "cut_"]);
    // Wood logs belong to the natural tab; the rest of a wood set to building.
    if natural && !has(n, &["planks", "stairs", "slab", "fence", "door", "wall", "button", "plate", "brick", "polished", "smooth", "tile", "chiseled", "pillar"]) {
        let s = if has(n, &["_ore", "ancient_debris"]) || n.starts_with("raw_") && n.ends_with("_block") {
            "Minérios"
        } else if has(n, &["_log", "_leaves", "_sapling", "_roots", "_wood", "_stem", "_hyphae", "propagule"]) && !has(n, &["mushroom_stem", "melon_stem", "pumpkin_stem"]) {
            "Troncos e folhas"
        } else if has(n, &["dirt", "grass_block", "sand", "gravel", "mud", "clay", "podzol", "mycelium", "farmland", "path", "soil"]) {
            "Terra e areia"
        } else if has(n, &["ice", "snow"]) {
            "Gelo e neve"
        } else if has(n, &["nether", "crimson", "warped", "soul", "basalt", "blackstone", "magma", "shroomlight", "glowstone", "obsidian", "ancient", "wart"]) {
            "Nether"
        } else if has(n, &["end_stone", "chorus", "end_"]) {
            "End"
        } else if has(n, &["stone", "deepslate", "tuff", "calcite", "granite", "diorite", "andesite", "bedrock", "dripstone", "amethyst", "cobble", "sculk", "rock", "slate"]) {
            "Pedras"
        } else if has(n, &["tulip", "dandelion", "poppy", "orchid", "allium", "bluet", "daisy", "cornflower", "lily_of", "rose", "flower", "peony", "lilac", "sunflower", "petals", "torchflower", "pitcher", "wither_rose", "blossom", "bloom", "daffodil", "lavender", "violet", "iris", "lotus", "hibiscus", "marigold", "orchid", "aster"]) {
            "Flores"
        } else if has(n, &["coral", "kelp", "seagrass", "lily", "sea_pickle", "sponge", "pickle", "algae", "reed", "cattail", "water"]) {
            "Água e corais"
        } else if matches!(shape, Shape::Cross | Shape::Model | Shape::Layer) || has(n, &["mushroom", "fungus", "vine", "grass", "fern", "bush", "moss", "cactus", "pumpkin", "melon", "bamboo", "sugar_cane", "leaf", "berry", "crop", "plant", "root", "lichen", "hay", "honey", "bee_nest", "cobweb"]) {
            "Plantas e cogumelos"
        } else {
            return by_mod(Tab::Natural, "Outros naturais");
        };
        if modded && !matches!(s, "Minérios" | "Troncos e folhas" | "Flores" | "Plantas e cogumelos") {
            return out(Tab::Natural, origin.label());
        }
        return out(Tab::Natural, s);
    }
    // Building blocks, by material.
    if modded {
        let wood = WOODS.iter().any(|w| n.starts_with(w)) || has(n, &["planks", "_log", "stripped_"]);
        // Mod sets of the base game's woods and stones (Macaw's gates, snowy twins) join those.
        if !(wood && !matches!(origin, Origin::Mod(i) if MODS[i as usize].0 == "aether" || MODS[i as usize].0 == "otherside")) {
            return out(Tab::Building, origin.label());
        }
    }
    let s = if WOODS.iter().any(|w| n.starts_with(&format!("{w}_")) || n.starts_with(&format!("stripped_{w}_"))) && !has(n, &["mushroom"]) || has(n, &["planks", "stripped_", "bamboo_"]) {
        "Madeiras"
    } else if has(n, &["copper"]) {
        "Cobre"
    } else if has(n, &["prismarine"]) {
        "Prismarinho"
    } else if has(n, &["sandstone"]) {
        "Arenitos"
    } else if has(n, &["nether", "quartz", "blackstone", "basalt", "soul", "crimson", "warped", "gilded"]) {
        "Nether"
    } else if has(n, &["end_stone", "purpur", "end_"]) {
        "End"
    } else if has(n, &["mud", "clay", "packed_"]) {
        "Lama e argila"
    } else if has(n, &["brick", "tile"]) {
        "Tijolos e ladrilhos"
    } else if has(n, &["stone", "cobble", "granite", "diorite", "andesite", "deepslate", "tuff", "calcite", "slate", "marble", "limestone", "basalt", "dripstone", "resin"]) {
        "Pedras"
    } else if n.ends_with("_block") || has(n, &["iron_bars", "chain", "gold", "iron", "diamond", "emerald", "lapis", "netherite", "amethyst", "metal"]) {
        "Metais e gemas"
    } else if origin == Origin::Terraria || origin == Origin::Ours {
        return out(Tab::Building, origin.label());
    } else {
        "Outros blocos"
    };
    out(Tab::Building, s)
}

impl Catalog {
    pub fn build(reg: &BlockRegistry, items: &ItemRegistry) -> Catalog {
        let origin_of = origins();
        let tables = Tables::new();
        // Items the item files name themselves (a block item listed there is meant to be handed out).
        let listed: std::collections::HashSet<String> = [embedded::ITEMS_RON, embedded::FASE6_ITEMS_RON, embedded::T5_ITEMS_RON, embedded::TR_ITEMS_RON, embedded::TR3_ITEMS_RON, embedded::MODS_ITEMS_RON]
            .iter()
            .flat_map(|t| names_in(t).map(|s| s.to_string()))
            .collect();
        let n = items.items.len();
        let mut entries: Vec<Option<Entry>> = vec![None; n];
        let mut sections: HashMap<Tab, Vec<String>> = HashMap::new();
        for tab in ITEM_TABS {
            sections.insert(tab, tab.sections().iter().map(|s| s.to_string()).collect());
        }
        // Families listed where their first member stands.
        let mut first: HashMap<String, usize> = HashMap::new();
        for (i, it) in items.items.iter().enumerate().skip(1) {
            first.entry(family(&it.name).0).or_insert(i);
        }
        let mut raw: Vec<(ItemId, Tab, String, Origin)> = Vec::new();
        for id in 1..n as ItemId {
            let it = items.get(id);
            let o = origin_of.get(&it.name).copied().unwrap_or(Origin::Plugin);
            if let Some((tab, s)) = classify(reg, items, id, o, &tables, &listed) {
                raw.push((id, tab, s, o));
            }
        }
        // Sections the tables don't name: Terraria, ours, then each mod in its order.
        let mut extra: Vec<(Tab, String, Origin)> = Vec::new();
        for (_, tab, s, o) in &raw {
            if !sections[tab].contains(s) && !extra.iter().any(|e| e.0 == *tab && e.1 == *s) {
                extra.push((*tab, s.clone(), *o));
            }
        }
        extra.sort_by_key(|e| e.2);
        for (tab, s, _) in extra {
            sections.get_mut(&tab).unwrap().push(s);
        }
        for (id, tab, s, o) in raw {
            let it = items.get(id);
            let sec = sections[&tab].iter().position(|x| *x == s).unwrap_or(0) as u16;
            let (fam, var) = family(&it.name);
            let fam_at = first.get(&fam).copied().unwrap_or(id as usize) as u64;
            // Tools by tier, weapons by the stage they come in, colours in Java's order.
            let rank: u64 = if let Some(&(_, stage)) = tables.gear.get(it.name.as_str()) {
                stage as u64 * 1000 + it.tool.as_ref().map(|t| t.tier as u64).unwrap_or(0)
            } else if let Some(t) = &it.tool {
                t.tier as u64 * 10_000 + t.durability as u64
            } else if tab == Tab::Colored {
                COLORS.iter().position(|c| it.name.starts_with(&format!("{c}_"))).map(|p| p as u64 + 1).unwrap_or(0)
            } else {
                0
            };
            let order = (sec as u64) << 52 | rank.min(0xF_FFFF) << 32 | fam_at.min(0xFFFF) << 16 | (var as u64).min(0xFF) << 8;
            let section = &sections[&tab][sec as usize];
            let mut hay = format!("{} | {} | {} | {} | {}", norm(&it.display_name), norm(&it.name), norm(section), norm(tab.title()), norm(&o.tooltip()));
            hay.push_str(&synonyms(it, reg, tab, section));
            entries[id as usize] = Some(Entry { tab, sec, origin: o, order, hay, key: norm(&it.display_name) });
        }
        let mut lists: HashMap<Tab, Vec<ItemId>> = HashMap::new();
        for (id, e) in entries.iter().enumerate() {
            if let Some(e) = e {
                lists.entry(e.tab).or_default().push(id as ItemId);
            }
        }
        for (_, v) in lists.iter_mut() {
            v.sort_by_key(|&i| (entries[i as usize].as_ref().unwrap().order, i));
        }
        Catalog { entries, sections, lists, built_for: n }
    }

    pub fn entry(&self, id: ItemId) -> Option<&Entry> {
        self.entries.get(id as usize).and_then(|e| e.as_ref())
    }

    pub fn tab_of(&self, id: ItemId) -> Option<Tab> {
        self.entry(id).map(|e| e.tab)
    }

    pub fn section_name(&self, e: &Entry) -> &str {
        self.sections.get(&e.tab).and_then(|v| v.get(e.sec as usize)).map(|s| s.as_str()).unwrap_or("")
    }

    /// A sort key that lists things like the creative screen does (for the bag sort).
    pub fn rank(&self, id: ItemId) -> (usize, u64, ItemId) {
        match self.entry(id) {
            Some(e) => (ITEM_TABS.iter().position(|t| *t == e.tab).unwrap_or(99), e.order, id),
            None => (100, 0, id),
        }
    }

    /// Items of a tab, in order.
    pub fn list(&self, tab: Tab) -> &[ItemId] {
        self.lists.get(&tab).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Does the item fit the query? Some(rank) when it does (lower ranks are listed first).
    pub fn matches(&self, id: ItemId, q: &Query) -> Option<u8> {
        let e = self.entry(id)?;
        for o in &q.origins {
            if !norm(&e.origin.keys()).split(' ').any(|k| k.starts_with(o.as_str())) && !e.origin.keys().contains(o.as_str()) {
                return None;
            }
        }
        for s in &q.sections {
            let sec = norm(self.section_name(e));
            let tab = norm(e.tab.title());
            if !word_hit(&sec, s) && !word_hit(&tab, s) {
                return None;
            }
        }
        if q.words.is_empty() {
            return Some(3);
        }
        for w in &q.words {
            if !word_hit(&e.hay, w) {
                return None;
            }
        }
        // Names that start with the first word come first, then ones with a word starting so.
        let first = &q.words[0];
        Some(if e.key.starts_with(first.as_str()) {
            0
        } else if e.key.split(' ').any(|k| k.starts_with(first.as_str())) {
            1
        } else if e.key.contains(first.as_str()) {
            2
        } else {
            3
        })
    }
}

/// `w` in `hay`, or its singular ("espadas" → "espada", "poções" → "poção", "flores" → "flor").
fn word_hit(hay: &str, w: &str) -> bool {
    if hay.contains(w) {
        return true;
    }
    let n = w.chars().count();
    if n > 3 {
        for cut in ["oes", "aes", "ns", "es", "s"] {
            if let Some(stem) = w.strip_suffix(cut) {
                let stem = match cut {
                    "oes" | "aes" => format!("{stem}ao"),
                    "ns" => format!("{stem}m"),
                    _ => stem.to_string(),
                };
                if stem.chars().count() >= 3 && hay.contains(stem.as_str()) {
                    return true;
                }
            }
        }
    }
    false
}

/// Everyday words an item answers to besides its names.
fn synonyms(it: &voxel::item::Item, reg: &BlockRegistry, tab: Tab, section: &str) -> String {
    let mut s = String::new();
    let n = it.name.as_str();
    if it.places.is_some_and(|b| reg.get(b).light > 0) || section == "Iluminação" {
        s.push_str(" | luz iluminacao luminoso");
    }
    if matches!(tab, Tab::Combat) || section == "Armas mágicas" || section == "Invocação" {
        s.push_str(" | arma");
    }
    if it.armor.is_some() {
        s.push_str(" | armadura protecao defesa");
    }
    if it.food.is_some() {
        s.push_str(" | comida alimento comer");
    }
    if it.tool.is_some() {
        s.push_str(" | ferramenta");
    }
    if it.places.is_some() {
        s.push_str(" | bloco");
    }
    if n.contains("potion") {
        s.push_str(" | pocao");
    }
    if n.ends_with("_ore") {
        s.push_str(" | minerio");
    }
    if n.ends_with("_ingot") {
        s.push_str(" | lingote barra");
    }
    if n.contains("gold") || n.contains("golden") {
        s.push_str(" | ouro");
    }
    if n.contains("iron") {
        s.push_str(" | ferro");
    }
    s
}

/// A search as typed, split into plain words, `@origins` and `#sections`.
#[derive(Default, Debug, Clone)]
pub struct Query {
    pub words: Vec<String>,
    pub origins: Vec<String>,
    pub sections: Vec<String>,
}

impl Query {
    pub fn parse(text: &str) -> Query {
        let mut q = Query::default();
        for w in norm(text).split_whitespace() {
            if let Some(o) = w.strip_prefix('@') {
                if !o.is_empty() {
                    q.origins.push(o.to_string());
                }
            } else if let Some(s) = w.strip_prefix('#') {
                if !s.is_empty() {
                    q.sections.push(s.to_string());
                }
            } else {
                q.words.push(w.to_string());
            }
        }
        q
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty() && self.origins.is_empty() && self.sections.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cat() -> (BlockRegistry, ItemRegistry, Catalog) {
        let (reg, items, _) = crate::game_data(embedded::BLOCKS_RON, embedded::ITEMS_RON, embedded::RECIPES_RON).unwrap();
        let c = Catalog::build(&reg, &items);
        (reg, items, c)
    }

    fn tab_sec(c: &Catalog, items: &ItemRegistry, name: &str) -> (Tab, String) {
        let id = items.id(name).unwrap_or_else(|| panic!("no item {name}"));
        let e = c.entry(id).unwrap_or_else(|| panic!("{name} is in no tab"));
        (e.tab, c.section_name(e).to_string())
    }

    #[test]
    fn items_land_in_sensible_places() {
        let (_, items, c) = cat();
        let cases: &[(&str, Tab, &str)] = &[
            ("oak_planks", Tab::Building, "Madeiras"),
            ("stone_bricks", Tab::Building, "Tijolos e ladrilhos"),
            ("oak_log", Tab::Natural, "Troncos e folhas"),
            ("diamond_ore", Tab::Natural, "Minérios"),
            ("red_wool", Tab::Colored, "Lã"),
            ("torch", Tab::Functional, "Iluminação"),
            ("furnace", Tab::Functional, "Estações de trabalho"),
            ("chest", Tab::Functional, "Armazenamento"),
            ("repeater", Tab::Redstone, "Mecanismos"),
            ("diamond_pickaxe", Tab::Tools, "Picaretas"),
            ("diamond_sword", Tab::Combat, "Espadas"),
            ("bow", Tab::Combat, "Arcos e bestas"),
            ("arrow", Tab::Combat, "Flechas e munição"),
            ("iron_helmet", Tab::Combat, "Armaduras"),
            ("apple", Tab::Food, "Frutas e vegetais"),
            ("cooked_beef", Tab::Food, "Carnes e peixes"),
            ("potion_healing", Tab::Food, "Poções"),
            ("healing_potion", Tab::Food, "Poções do Terraria"),
            ("iron_ingot", Tab::Ingredients, "Minérios brutos e lingotes"),
            ("ruby_wand", Tab::Magic, "Armas mágicas"),
            ("wisp_staff", Tab::Magic, "Invocação"),
            ("angel_wings", Tab::Accessories, "Asas"),
            ("running_boots", Tab::Accessories, "Movimento"),
            ("grappling_hook", Tab::Tools, "Ganchos"),
            ("zombie_spawn_egg", Tab::Eggs, "Minecraft"),
            ("minishark", Tab::Combat, "Armas de fogo"),
        ];
        let mut bad = Vec::new();
        for (n, tab, s) in cases {
            let got = tab_sec(&c, &items, n);
            if got.0 != *tab || got.1 != *s {
                bad.push(format!("{n}: {:?}/{} (wanted {tab:?}/{s})", got.0, got.1));
            }
        }
        assert!(bad.is_empty(), "{}", bad.join("\n"));
    }

    #[test]
    fn catch_alls_stay_small() {
        // Most items find a real section: the "Outros" ones must not swallow the catalogue.
        let (_, items, c) = cat();
        let mut others = 0;
        let mut shown = 0;
        for id in 1..items.items.len() as ItemId {
            if let Some(e) = c.entry(id) {
                shown += 1;
                if c.section_name(e).starts_with("Outros") {
                    others += 1;
                }
            }
        }
        assert!(shown > 2500, "only {shown} items shown");
        assert!(others * 100 / shown < 8, "{others} of {shown} items in catch-all sections");
    }

    #[test]
    fn search_understands_portuguese() {
        let (_, items, c) = cat();
        let find = |q: &str| -> Vec<String> {
            let q = Query::parse(q);
            let mut v: Vec<(u8, u64, ItemId)> = (1..items.items.len() as ItemId).filter_map(|i| c.matches(i, &q).map(|r| (r, c.entry(i).unwrap().order, i))).collect();
            v.sort();
            v.into_iter().map(|x| items.get(x.2).name.clone()).collect()
        };
        assert!(find("picareta diamante").contains(&"diamond_pickaxe".to_string()));
        assert!(find("pocao cura").iter().any(|n| n.contains("healing")));
        assert!(find("espadas").contains(&"iron_sword".to_string()));
        assert!(find("diamond pickaxe").contains(&"diamond_pickaxe".to_string()));
        assert_eq!(find("tocha").first().map(|s| s.as_str()), Some("torch"));
        let t = find("@terraria");
        assert!(t.contains(&"minishark".to_string()) && !t.contains(&"iron_sword".to_string()));
        assert!(find("@aether").iter().all(|n| matches!(c.entry(items.id(n).unwrap()).unwrap().origin, Origin::Mod(0) | Origin::Mod(16))));
        assert!(find("#armaduras ferro").contains(&"iron_chestplate".to_string()));
        assert!(find("luz").contains(&"lantern".to_string()));
    }

    #[test]
    fn wood_sets_stay_together() {
        let (_, items, c) = cat();
        let list = c.list(Tab::Building);
        let at = |n: &str| list.iter().position(|&i| items.get(i).name == n).unwrap();
        assert!(at("oak_planks") < at("oak_stairs") && at("oak_stairs") < at("oak_slab") && at("oak_slab") < at("oak_fence"));
        assert!(at("oak_fence") < at("spruce_planks"));
    }

    /// `cargo test -p game catalog_report -- --ignored --nocapture`: every tab and section with
    /// its items, to read through (also written to target/catalogo.txt).
    #[test]
    #[ignore]
    fn catalog_report() {
        let (_, items, c) = cat();
        let mut out = String::new();
        for tab in ITEM_TABS {
            let list = c.list(tab);
            out += &format!("\n=== {} ({})\n", tab.title(), list.len());
            let mut cur = u16::MAX;
            for &id in list {
                let e = c.entry(id).unwrap();
                if e.sec != cur {
                    cur = e.sec;
                    let n = list.iter().filter(|&&i| c.entry(i).unwrap().sec == cur).count();
                    out += &format!("\n-- {} ({n})\n", c.section_name(e));
                }
                out += &format!("{} [{}], ", items.get(id).display_name, items.get(id).name);
            }
            out += "\n";
        }
        let hidden: Vec<_> = (1..items.items.len() as ItemId).filter(|&i| c.entry(i).is_none()).map(|i| items.get(i).name.clone()).collect();
        out += &format!("\n=== Escondidos ({}): {}\n", hidden.len(), hidden.join(" "));
        let _ = std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/catalogo.txt"), &out);
        println!("{out}");
    }
}
