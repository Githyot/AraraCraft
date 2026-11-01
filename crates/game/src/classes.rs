//! The six ways to fight (plan/classes-analise.md): Guerreiro, Arqueiro, Atirador, Assassino,
//! Mago and Invocador. Like Terraria there is no class to pick at the start: the weapon in hand,
//! the armour set and the accessories make the class, and the three loadouts (rpg.rs) let a player
//! keep one outfit per class and swap in a click.
//!
//! This file holds what the game and the balance sheet (balance.rs) share: which class every
//! weapon belongs to and the era it first shows up in, which era each armour set opens, and what a
//! set does for each class. On top of that, for the player: the class being played right now
//! ("estilo"), a line of what the class is about, and the class path of the current era (the
//! era's weapons, the class's set and emblem, with what the player already has), drawn as a panel
//! beside the stats sheet (C and the inventory).

use crate::rpg::{ALL_CLASSES, CLASSES as STAT_CLASSES, M, MAGIC, MELEE, RANGED, ROGUE, SETS, SUMMON};
use crate::Game;

/// The eras of the progression (the same six as progress.rs).
pub const STAGES: [&str; 6] = ["Pré-chefe", "Início", "Pré-hardmode", "Hardmode", "Pós-mecânicos", "Fim de jogo"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    /// Swords, axes, spears, yoyos, boomerangs, flails (melee stats).
    Warrior,
    /// Bows and the crossbow (ranged stats).
    Archer,
    /// Guns (ranged stats).
    Gunner,
    /// Daggers, knives, shurikens, javelins, grenades (the assassin's own stats, assassin.rs).
    /// Backstabs and stealth strikes come on top and are not counted here.
    Assassin,
    Mage,
    /// Minions (with the whip of the stage) and sentries.
    Summoner,
}

pub const CLASSES: [Class; 6] = [Class::Warrior, Class::Archer, Class::Gunner, Class::Assassin, Class::Mage, Class::Summoner];

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::Warrior => "Guerreiro",
            Class::Archer => "Arqueiro",
            Class::Gunner => "Atirador",
            Class::Assassin => "Assassino",
            Class::Mage => "Mago",
            Class::Summoner => "Invocador",
        }
    }

    /// The stat class of rpg.rs whose damage and crit it uses.
    pub fn stat(self) -> usize {
        match self {
            Class::Warrior => MELEE,
            Class::Archer | Class::Gunner => RANGED,
            Class::Assassin => ROGUE,
            Class::Mage => MAGIC,
            Class::Summoner => SUMMON,
        }
    }
}


/// Every weapon: its class and the stage it can first be had in (by the cheapest way: a
/// recipe, a shop, a chest or a boss).
pub const GEAR: &[(&str, Class, u8)] = &[
    // Pré-chefe: wood to gold and platinum, gems, the first shops.
    ("wooden_sword", Warrior, 0),
    ("stone_sword", Warrior, 0),
    ("copper_sword", Warrior, 0),
    ("bronze_sword", Warrior, 0),
    ("iron_sword", Warrior, 0),
    ("lead_sword", Warrior, 0),
    ("silver_sword", Warrior, 0),
    ("tungsten_sword", Warrior, 0),
    ("golden_sword", Warrior, 0),
    ("platinum_sword", Warrior, 0),
    ("stone_greatsword", Warrior, 0),
    ("iron_greatsword", Warrior, 0),
    ("golden_greatsword", Warrior, 0),
    ("stone_halberd", Warrior, 0),
    ("iron_halberd", Warrior, 0),
    ("golden_halberd", Warrior, 0),
    ("wooden_spear", Warrior, 0),
    ("stone_spear", Warrior, 0),
    ("copper_spear", Warrior, 0),
    ("iron_spear", Warrior, 0),
    ("golden_spear", Warrior, 0),
    ("skyroot_sword", Warrior, 0),
    ("holystone_sword", Warrior, 0),
    ("wood_yoyo", Warrior, 0),
    ("wood_boomerang", Warrior, 0),
    ("iron_flail", Warrior, 0),
    ("enchanted_boomerang", Warrior, 0),
    ("enchanted_sword", Warrior, 0),
    ("sky_blade", Warrior, 0),
    ("cactus_sword", Warrior, 0),
    ("bone_sword", Warrior, 0),
    ("ice_blade", Warrior, 1),
    ("light_blade", Warrior, 1),
    ("falcon_blade", Warrior, 1),
    ("frost_brand", Warrior, 3),
    ("star_saber", Warrior, 0),
    ("bow", Archer, 0),
    ("crossbow", Archer, 0),
    ("copper_bow", Archer, 0),
    ("silver_bow", Archer, 0),
    ("platinum_bow", Archer, 0),
    ("flintlock_pistol", Gunner, 0),
    ("shuriken", Assassin, 0),
    ("throwing_knife", Assassin, 0),
    ("javelin", Assassin, 0),
    ("bone_javelin", Assassin, 0),
    ("grenade", Assassin, 0),
    ("iron_dagger", Assassin, 0),
    ("blowpipe", Gunner, 0),
    ("revolver", Gunner, 0),
    ("topaz_wand", Mage, 0),
    ("ruby_wand", Mage, 0),
    ("sapphire_wand", Mage, 0),
    ("amethyst_wand", Mage, 0),
    ("emerald_wand", Mage, 0),
    ("fire_staff", Mage, 0),
    ("frost_staff", Mage, 0),
    ("thunder_tome", Mage, 0),
    ("flame_tome", Mage, 0),
    ("chain_tome", Mage, 0),
    ("wisp_staff", Summoner, 0),
    ("jelly_staff", Summoner, 0),
    ("leather_whip", Summoner, 0),
    ("storm_totem_staff", Summoner, 0),
    ("frost_crystal_staff", Summoner, 0),
    ("ballista_rod", Summoner, 0),
    // Início: the first bosses, diamond, the meteor, the evil biomes.
    ("diamond_sword", Warrior, 1),
    ("diamond_greatsword", Warrior, 1),
    ("diamond_halberd", Warrior, 1),
    ("diamond_spear", Warrior, 1),
    ("zanite_sword", Warrior, 1),
    ("umbral_sword", Warrior, 1),
    ("sanguinite_sword", Warrior, 1),
    ("umbral_yoyo", Warrior, 1),
    ("sanguinite_yoyo", Warrior, 1),
    ("umbral_flail", Warrior, 1),
    ("sanguinite_flail", Warrior, 1),
    ("jungle_yoyo", Warrior, 1),
    ("thorn_chakram", Warrior, 1),
    ("ice_boomerang", Warrior, 1),
    ("umbral_bow", Archer, 1),
    ("sanguinite_bow", Archer, 1),
    ("hive_bow", Archer, 1),
    ("musket", Gunner, 1),
    ("star_cannon", Gunner, 1),
    ("minishark", Gunner, 1),
    ("boomstick", Gunner, 1),
    ("guardian_staff", Mage, 1),
    ("aqua_scepter", Mage, 1),
    ("creeping_thorn", Mage, 1),
    ("meteor_blaster", Mage, 1),
    ("ruin_thorn", Mage, 1),
    ("blight_rod", Mage, 1),
    ("poison_knife", Assassin, 1),
    ("umbral_dagger", Assassin, 1),
    ("worm_staff", Mage, 1),
    ("wasp_staff", Summoner, 1),
    ("thorn_whip", Summoner, 1),
    ("spider_staff", Summoner, 1),
    // Pré-hardmode: the hive, the dungeon, the Nether, the Underworld's metal.
    ("magmarite_sword", Warrior, 2),
    ("netherite_sword", Warrior, 2),
    ("netherite_spear", Warrior, 2),
    ("gravitite_sword", Warrior, 2),
    ("valkyrie_lance", Warrior, 2),
    ("resonarium_sword", Warrior, 2),
    ("hive_blade", Warrior, 2),
    ("blue_katana", Warrior, 2),
    ("night_edge", Warrior, 2),
    ("mace", Warrior, 2),
    ("trident", Warrior, 2),
    ("bone_flail", Warrior, 2),
    ("sun_flail", Warrior, 2),
    ("magma_yoyo", Warrior, 2),
    ("flame_boomerang", Warrior, 2),
    ("rapid_gun", Gunner, 2),
    ("handgun", Gunner, 2),
    ("phoenix_blaster", Gunner, 2),
    ("quad_barrel_shotgun", Gunner, 2),
    ("magmarite_bow", Archer, 2),
    ("batwing_bow", Archer, 2),
    ("magmarite_knife", Assassin, 2),
    ("magmarite_dagger", Assassin, 2),
    ("water_bolt", Mage, 2),
    ("demon_scythe", Mage, 2),
    ("hive_staff", Mage, 2),
    ("flower_of_fire", Mage, 2),
    ("colossus_staff", Mage, 2),
    ("void_tome", Mage, 2),
    ("imp_staff", Summoner, 2),
    ("bone_whip", Summoner, 2),
    ("raven_staff", Summoner, 2),
    ("ember_tower_staff", Summoner, 2),
    // Hardmode: the six new metals, the first invasions, the queen of the jellies.
    ("cobalt_sword", Warrior, 3),
    ("palladium_sword", Warrior, 3),
    ("mythril_sword", Warrior, 3),
    ("orichalcum_sword", Warrior, 3),
    ("adamantite_sword", Warrior, 3),
    ("titanium_sword", Warrior, 3),
    ("eclipse_scythe", Warrior, 3),
    ("crystal_yoyo", Warrior, 3),
    ("cutlass", Warrior, 3),
    ("cobalt_bow", Archer, 3),
    ("mythril_bow", Archer, 3),
    ("adamantite_bow", Archer, 3),
    ("marrow_bow", Archer, 3),
    ("frost_bow", Archer, 3),
    ("storm_bow", Archer, 3),
    ("adamantite_rifle", Gunner, 3),
    ("mythril_javelin", Assassin, 3),
    ("mythril_dagger", Assassin, 3),
    ("ice_spike_staff", Mage, 3),
    ("blizzard_staff", Mage, 3),
    ("crystal_storm", Mage, 3),
    ("cursed_flames", Mage, 3),
    ("magnet_sphere", Mage, 3),
    ("shadow_beam", Mage, 3),
    ("explosive_trap_rod", Summoner, 3),
    ("lightning_aura_rod", Summoner, 3),
    ("flame_sentry_rod", Summoner, 3),
    ("crystal_jelly_staff", Summoner, 3),
    ("frost_whip", Summoner, 3),
    ("megashark", Gunner, 3),
    ("flamethrower", Gunner, 3),
    ("shotgun", Gunner, 3),
    ("clockwork_rifle", Gunner, 3),
    ("uzi", Gunner, 3),
    ("onyx_blaster", Gunner, 3),
    ("ogre_club", Warrior, 3),
    // Pós-mecânicos: radiant and verdite, the vine queen, the idol, the tide duke, the empress.
    ("radiant_sword", Warrior, 4),
    ("verdite_sword", Warrior, 4),
    ("radiant_blade", Warrior, 4),
    ("true_night_edge", Warrior, 4),
    ("true_radiant_blade", Warrior, 4),
    ("terra_blade", Warrior, 4),
    ("vine_blade", Warrior, 4),
    ("horseman_blade", Warrior, 4),
    ("stone_fist", Warrior, 4),
    ("tide_boomerang", Warrior, 4),
    ("prism_saber", Warrior, 4),
    ("bubble_rifle", Gunner, 4),
    ("laser_rifle", Gunner, 4),
    ("radiant_bow", Archer, 4),
    ("verdite_bow", Archer, 4),
    ("pulse_bow", Archer, 4),
    ("tide_bow", Archer, 4),
    ("verdite_knife", Assassin, 4),
    ("verdite_dagger", Assassin, 4),
    ("meteor_staff", Mage, 4),
    ("razorblade_typhoon", Mage, 4),
    ("spirit_staff", Mage, 4),
    ("sun_stone_staff", Mage, 4),
    ("blade_staff", Summoner, 4),
    ("ballista_cane", Summoner, 4),
    ("storm_whip", Summoner, 4),
    ("sniper_rifle", Gunner, 4),
    ("coin_gun", Gunner, 4),
    ("elf_gun", Gunner, 4),
    ("rocket_launcher", Gunner, 4),
    ("grenade_launcher", Gunner, 4),
    ("tactical_shotgun", Gunner, 4),
    ("venus_magnum", Gunner, 4),
    ("chain_gun", Gunner, 4),
    ("paladin_hammer", Warrior, 4),
    ("betsy_wrath", Warrior, 4),
    ("razorpine", Warrior, 4),
    ("spooky_staff", Summoner, 4),
    // Fim de jogo: the pillars, lumite and the Celestial Herald.
    ("lumite_sword", Warrior, 5),
    ("celestial_blade", Warrior, 5),
    ("celestial_wrath", Warrior, 5),
    ("apex_blade", Warrior, 5),
    ("solar_eruption", Warrior, 5),
    ("nebula_blade", Warrior, 5),
    ("stardust_yoyo", Warrior, 5),
    ("vortex_beater", Gunner, 5),
    ("lumite_minigun", Gunner, 5),
    ("lumite_bow", Archer, 5),
    ("phoenix_bow", Archer, 5),
    ("skysplit_bow", Archer, 5),
    ("sun_javelin", Assassin, 5),
    ("lumite_dagger", Assassin, 5),
    ("star_tome", Mage, 5),
    ("lunar_flare", Mage, 5),
    ("prism", Mage, 5),
    ("star_dragon_staff", Summoner, 5),
    ("star_whip", Summoner, 5),
];

/// Armour sets: the stage each can first be worn in.
pub const SET_STAGE: &[(&str, u8)] = &[
    ("leather", 0),
    ("chainmail", 0),
    ("iron", 0),
    ("golden", 0),
    ("diamond", 1),
    ("netherite", 2),
    ("copper", 0),
    ("tin", 0),
    ("bronze", 0),
    ("lead", 0),
    ("silver", 0),
    ("tungsten", 0),
    ("platinum", 0),
    ("meteor", 1),
    ("umbral", 1),
    ("sanguinite", 1),
    ("jungle", 1),
    ("fossil", 1),
    ("ninja", 1),
    ("nightshade", 2),
    ("shroud", 3),
    ("eclipse", 4),
    ("gladiator", 1),
    ("hive", 2),
    ("necro", 2),
    ("magmarite", 2),
    ("cobalt", 3),
    ("palladium", 3),
    ("mythril", 3),
    ("orichalcum", 3),
    ("adamantite", 3),
    ("titanium", 3),
    ("radiant", 4),
    ("verdite", 4),
    ("spectral", 4),
    ("sporite", 4),
    ("beetle", 4),
    ("turtle", 4),
    ("solar", 5),
    ("vortex", 5),
    ("nebula", 5),
    ("stardust", 5),
    ("lumite", 5),
    ("zanite", 1),
    ("pyromancer", 1),
    ("troubadour", 0),
    ("minstrel", 1),
    ("maestro", 3),
    ("virtuoso", 5),
    ("cryomancer", 1),
    ("electromancer", 1),
    ("priest", 1),
    ("voidwalker", 1),
    ("bloodmage", 1),
    ("cultist", 1),
    ("archevoker", 1),
    ("gravitite", 2),
    ("valkyrie", 2),
    ("phoenix", 2),
    ("resonarium", 2),
];

/// How much the best armour set a class can wear by `stage` multiplies its damage per second.
pub fn set_factor(class: Class, stage: u8) -> (f32, &'static str) {
    let k = class.stat();
    let base_crit = crate::rpg::Stats::default().crit[k];
    let mut best = (1.0 + 0.5 * base_crit, "");
    for (prefix, s) in SET_STAGE {
        if *s > stage {
            continue;
        }
        let Some(set) = SETS.iter().find(|x| x.prefix == *prefix) else { continue };
        // (A set's minion slots are already in `SLOTS`.)
        let (mut dmg, mut crit, mut speed) = (1.0, base_crit, 1.0);
        for m in set.mods {
            match *m {
                M::Dmg(c, v) if c == k || c == ALL_CLASSES => dmg += v,
                M::Crit(c, v) if (c == k || c == ALL_CLASSES) && class != Summoner => crit += v,
                M::MeleeSpeed(v) if class == Warrior => speed += v,
                // Follows the weapon in hand: whatever the class.
                M::HeldDmg(v) => dmg += v,
                _ => {}
            }
        }
        let f = dmg * (1.0 + 0.5 * crit) * speed;
        if f > best.0 {
            best = (f, prefix);
        }
    }
    best
}

use Class::*;

impl Class {
    /// What the class is about, in a line, as it plays in a 3D world.
    pub fn fantasy(self) -> &'static str {
        match self {
            Warrior => "Na frente da luta: espadas, ioiôs, manguais e lanças. Suba até o monstro.",
            Archer => "Puxe o arco até o fim: o tiro cheio é crítico. Mire acima de quem está longe.",
            Gunner => "Tiro reto e rápido: munição do Armeiro, bom contra o que voa.",
            Assassin => "Adagas pelas costas e facas em leque: fique na sombra, encha a furtividade e golpeie primeiro.",
            Mage => "Varinhas, cajados e livros gastam mana. Magias que perseguem acham o alvo.",
            Summoner => "Lacaios lutam por você; o chicote marca o alvo e as sentinelas guardam o lugar.",
        }
    }

    /// Colour of the class name on the panels.
    pub fn color(self) -> [u8; 4] {
        match self {
            Warrior => [235, 120, 90, 255],
            Archer => [150, 210, 110, 255],
            Gunner => [220, 190, 120, 255],
            Assassin => [200, 160, 230, 255],
            Mage => [120, 170, 255, 255],
            Summoner => [110, 225, 210, 255],
        }
    }

    /// The emblem every class looks for (rpg.rs accessories).
    pub fn emblem(self) -> &'static str {
        match self.stat() {
            MELEE => "warrior_emblem",
            RANGED => "ranger_emblem",
            MAGIC => "arcane_emblem",
            ROGUE => "assassin_emblem",
            _ => "summoner_emblem",
        }
    }
}

/// The class of a weapon, if it is one.
pub fn of_weapon(name: &str) -> Option<Class> {
    GEAR.iter().find(|g| g.0 == name).map(|g| g.1)
}

/// How much a list of stat changes leans to each stat class (melee, ranged, magic, summon, rogue).
pub fn lean(mods: &[M]) -> [f32; STAT_CLASSES] {
    let mut w = [0.0; STAT_CLASSES];
    for m in mods {
        match *m {
            M::Dmg(c, v) if c < STAT_CLASSES => w[c] += v,
            M::Dmg(_, v) => w.iter_mut().for_each(|x| *x += v),
            M::Crit(c, v) if c < STAT_CLASSES => w[c] += 0.5 * v,
            M::Crit(_, v) => w.iter_mut().for_each(|x| *x += 0.5 * v),
            M::MeleeSpeed(v) => w[MELEE] += v,
            M::AmmoSave(v) => w[RANGED] += 0.5 * v,
            M::Mana(v) => w[MAGIC] += 0.01 * v,
            M::ManaCost(v) | M::ManaRegen(v) if v < 1.0 => w[MAGIC] += v,
            M::Minions(n) => w[SUMMON] += 0.1 * n as f32,
            M::Stealth(v) => w[ROGUE] += 0.001 * v,
            M::Backstab(v) => w[ROGUE] += 0.5 * v,
            _ => {}
        }
    }
    w
}

/// The stat class a set is made for, if it leans to one (a set for everyone gives `None`).
pub fn set_class(prefix: &str) -> Option<usize> {
    let set = SETS.iter().find(|s| s.prefix == prefix)?;
    let w = lean(set.mods);
    let (k, top) = w.iter().enumerate().fold((0, 0.0), |a, (k, v)| if *v > a.1 { (k, *v) } else { a });
    let second = w.iter().enumerate().filter(|(i, _)| *i != k).map(|(_, v)| *v).fold(0.0, f32::max);
    (top > 0.05 && top > 2.0 * second).then_some(k)
}

/// One line of the class path panel.
pub type Line = (String, [u8; 4]);

const HAVE: [u8; 4] = [130, 225, 130, 255];
const WANT: [u8; 4] = [200, 200, 200, 255];
const GREY: [u8; 4] = [150, 150, 150, 255];
const WARN: [u8; 4] = [240, 170, 90, 255];

/// Longest line of the class panel, in letters: it sits beside the stats sheet, so it stays narrow.
const WIDTH: usize = 22;

/// Adds `text` to the panel, broken into lines of at most `WIDTH` letters at the spaces.
fn wrap(out: &mut Vec<Line>, text: &str, color: [u8; 4]) {
    let mut line = String::new();
    for word in text.split(' ') {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > WIDTH {
            out.push((std::mem::take(&mut line), color));
            line.push_str("  ");
        } else if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.trim().is_empty() {
        out.push((line, color));
    }
}

/// Display names of the stat classes, for the warnings.
const STAT_NAMES: [&str; STAT_CLASSES] = ["corpo a corpo", "à distância", "magia", "invocação", "assassino"];

impl Game {
    /// The class of the weapon in hand, if any.
    pub(crate) fn held_class(&self) -> Option<Class> {
        of_weapon(&self.held_key())
    }

    /// Keeps the class being played: the class of the last weapon held (rpg.rs calls this
    /// every frame).
    pub(crate) fn update_style(&mut self) {
        if let Some(c) = self.held_class() {
            self.t5.rpg.style = Some(c);
        }
    }

    /// The era the player is in, from the progression road.
    pub(crate) fn era(&self) -> u8 {
        self.progress.era().min(STAGES.len() as u8 - 1)
    }

    fn owns(&self, name: &str) -> bool {
        let Some(id) = self.items.id(name) else { return false };
        self.surv.inv.slots.iter().flatten().any(|s| s.item == id) || self.surv.armor.iter().flatten().any(|s| s.item == id) || self.wears(name)
    }

    fn class_item_name(&self, name: &str) -> String {
        self.items.id(name).map(|id| self.items.get(id).display_name.clone()).unwrap_or_else(|| name.to_string())
    }

    /// The class path panel: the class being played and what the current era has for it.
    pub(crate) fn class_lines(&self) -> Vec<Line> {
        let mut out = Vec::new();
        let Some(class) = self.held_class().or(self.t5.rpg.style) else {
            out.push(("Classe: nenhuma ainda".to_string(), WANT));
            out.push(("A arma na mão faz a classe.".to_string(), GREY));
            out.push(("Para começar:".to_string(), GREY));
            for c in CLASSES {
                let first = GEAR.iter().find(|g| g.1 == c && g.2 == 0 && !g.0.starts_with("wood")).map(|g| self.class_item_name(g.0)).unwrap_or_default();
                out.push((format!("{}: {}", c.name(), first), c.color()));
            }
            return out;
        };
        let era = self.era();
        out.push((format!("Classe: {}", class.name()), class.color()));
        wrap(&mut out, class.fantasy(), GREY);
        out.push((format!("Nesta era ({}):", STAGES[era as usize]), WANT));
        // The era's weapons: the ones owned first, at most three named.
        let weapons: Vec<&str> = GEAR.iter().filter(|g| g.1 == class && g.2 == era).map(|g| g.0).collect();
        let mut owned: Vec<&str> = weapons.iter().copied().filter(|w| self.owns(w)).collect();
        if !owned.is_empty() {
            owned.truncate(2);
            let names: Vec<String> = owned.iter().map(|w| self.class_item_name(w)).collect();
            wrap(&mut out, &format!("+ Arma: {}", names.join(", ")), HAVE);
        } else if weapons.is_empty() {
            out.push(("  Arma: a da era anterior ainda serve".to_string(), GREY));
        } else {
            let names: Vec<String> = weapons.iter().rev().take(3).map(|w| self.class_item_name(w)).collect();
            wrap(&mut out, &format!("- Arma: {}", names.join(" ou ")), WANT);
        }
        let (_, set) = set_factor(class, era);
        if !set.is_empty() {
            let worn = self.t5.rpg.stats.set.is_some_and(|i| SETS[i].prefix == set);
            let name = crate::rpg::set_name(set);
            if worn {
                out.push((format!("+ Conjunto {name}"), HAVE));
            } else {
                wrap(&mut out, &format!("- Conjunto {name} (as 4 peças)"), WANT);
            }
        }
        let emblem = class.emblem();
        if self.wears(emblem) {
            out.push((format!("+ {}", self.class_item_name(emblem)), HAVE));
        } else {
            out.push((format!("- {}", self.class_item_name(emblem)), WANT));
        }
        // Gear that pulls the other way.
        if let Some(i) = self.t5.rpg.stats.set {
            if let Some(k) = set_class(SETS[i].prefix) {
                if k != class.stat() {
                    wrap(&mut out, &format!("Seu conjunto é de {}", STAT_NAMES[k]), WARN);
                }
            }
        }
        let mut lean_acc = [0.0f32; STAT_CLASSES];
        for st in self.t5.acc[..self.acc_slots()].iter().flatten() {
            if let Some(a) = crate::rpg::accessory(&self.items.get(st.item).name) {
                let w = lean(a.mods);
                for k in 0..4 {
                    lean_acc[k] += w[k];
                }
            }
        }
        let mine = lean_acc[class.stat()];
        if let Some((k, v)) = lean_acc.iter().enumerate().filter(|(k, _)| *k != class.stat()).fold(None, |a: Option<(usize, f32)>, (k, v)| if a.map_or(true, |a| *v > a.1) { Some((k, *v)) } else { a }) {
            if v >= 0.15 && v > mine + 0.1 {
                wrap(&mut out, &format!("Seus acessórios ajudam mais {}", STAT_NAMES[k]), WARN);
            }
        }
        out
    }

    /// Width and height of the class path panel.
    pub(crate) fn class_panel_size(&self) -> (f32, f32) {
        let s = self.ui.scale;
        let lines = self.class_lines();
        let tw = lines.iter().map(|(t, _)| self.ui.text_width(t, s)).fold(self.ui.text_width("Caminho da classe", s), f32::max);
        (tw + 8.0 * s, (lines.len() as f32 * 9.0 + 16.0) * s)
    }

    /// The class path panel, drawn with its right edge at `right`.
    pub(crate) fn draw_class_panel(&mut self, right: f32, y: f32) {
        let s = self.ui.scale;
        let lines = self.class_lines();
        let tw = lines.iter().map(|(t, _)| self.ui.text_width(t, s)).fold(self.ui.text_width("Caminho da classe", s), f32::max);
        let (pw, ph) = (tw + 8.0 * s, (lines.len() as f32 * 9.0 + 16.0) * s);
        let x = (right - pw).max(2.0 * s);
        self.ui.rect(x, y, pw, ph, [20, 16, 28, 215]);
        self.ui.frame(x, y, pw, ph, s, [110, 90, 150, 255]);
        self.ui.text(x + 4.0 * s, y + 3.0 * s, "Caminho da classe", [255, 230, 140, 255]);
        for (k, (t, c)) in lines.iter().enumerate() {
            self.ui.text(x + 4.0 * s, y + (13.0 + k as f32 * 9.0) * s, t, *c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_sets_lean_the_right_way() {
        assert_eq!(set_class("adamantite"), Some(MELEE));
        assert_eq!(set_class("cobalt"), Some(RANGED));
        assert_eq!(set_class("mythril"), Some(MAGIC));
        assert_eq!(set_class("orichalcum"), Some(SUMMON));
        assert_eq!(set_class("lumite"), None);
    }

    #[test]
    fn every_class_has_a_first_weapon() {
        for c in CLASSES {
            assert!(GEAR.iter().any(|g| g.1 == c && g.2 == 0), "{} sem arma no começo", c.name());
        }
        assert_eq!(of_weapon("topaz_wand"), Some(Mage));
        assert_eq!(of_weapon("dirt"), None);
    }
}
