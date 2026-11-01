//! Buttons of the survival inventory, Terraria style, so sorting and putting away don't depend on
//! knowing the keys: over the bag "Ordenar" (O) and "Guardar nos baús perto" (X); over an open
//! chest "Pegar tudo", "Depositar tudo", "Guardar iguais" and "Ordenar baú". Favourite slots
//! (Alt+click) and the hotbar are never moved by them. Sorting follows the creative catalogue
//! (catalog.rs), so a sorted bag lists things in the same order as the creative tabs.

use voxel::item::{Inventory, ItemStack};

use crate::audio::{Material, Sfx};
use crate::survival::{Container, Open, INV_SIZE};
use crate::Game;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Btn {
    SortBag,
    StackNearby,
    LootAll,
    DepositAll,
    StackHere,
    SortChest,
}

impl Btn {
    fn tip(self) -> &'static str {
        match self {
            Btn::SortBag => "Ordenar mochila (O)",
            Btn::StackNearby => "Guardar nos baús perto (X)",
            Btn::LootAll => "Pegar tudo",
            Btn::DepositAll => "Depositar tudo (menos barra e favoritos)",
            Btn::StackHere => "Guardar iguais: só o que o baú já tem",
            Btn::SortChest => "Ordenar baú",
        }
    }
}

/// The 12×12 picture of a button, as rows of 'X' (dark) and 'o' (light).
fn glyph(b: Btn) -> [&'static str; 10] {
    match b {
        Btn::SortBag | Btn::SortChest => [
            "..........",
            "XXXXXXXXX.",
            "..........",
            "XXXXXXX...",
            "..........",
            "XXXXX.....",
            "..........",
            "XXX.......",
            "..........",
            "X.........",
        ],
        Btn::StackNearby => [
            "....X.....",
            "....X.....",
            "..X.X.X...",
            "...XXX....",
            "....X.....",
            "XXXXXXXXX.",
            "XooooooX..",
            "XXXXXXXXX.",
            "XooooooX..",
            "XXXXXXXXX.",
        ],
        Btn::LootAll => [
            "XXXXXXXXX.",
            "XooooooX..",
            "XXXXXXXXX.",
            "....X.....",
            "....X.....",
            "....X.....",
            "..X.X.X...",
            "...XXX....",
            "....X.....",
            "..........",
        ],
        Btn::DepositAll => [
            "....X.....",
            "...XXX....",
            "..X.X.X...",
            "....X.....",
            "....X.....",
            "....X.....",
            "XXXXXXXXX.",
            "XooooooX..",
            "XXXXXXXXX.",
            "..........",
        ],
        Btn::StackHere => [
            "XXX..XXX..",
            "XoX..XoX..",
            "XXX..XXX..",
            "..........",
            "....X.....",
            "...XXX....",
            "..X.X.X...",
            "XXXXXXXXX.",
            "XooooooX..",
            "XXXXXXXXX.",
        ],
    }
}

impl Game {
    /// Draws the buttons that fit the open screen and runs the one clicked. Returns the hovered
    /// button's tooltip.
    pub(crate) fn draw_inventory_buttons(&mut self, x0: f32, y0: f32, extra: f32, mouse: (f32, f32)) -> Option<&'static str> {
        let s = self.ui.scale;
        let mut tip = None;
        let mut row = |game: &mut Game, btns: &[Btn], y: f32| {
            for (k, &b) in btns.iter().enumerate() {
                let x = x0 + (176.0 - 8.0 - 13.0 * (btns.len() - k) as f32 + 1.0) * s;
                if game.inventory_button(b, x, y0 + y * s, mouse) {
                    tip = Some(b.tip());
                }
            }
        };
        row(self, &[Btn::SortBag, Btn::StackNearby], 71.5 + extra);
        if let Open::Chest(p) = self.surv.open {
            if self.apoth_table(p).is_none() && matches!(self.surv.containers.get(&p), Some(Container::Chest(_))) {
                row(self, &[Btn::LootAll, Btn::DepositAll, Btn::StackHere, Btn::SortChest], 4.0);
            }
        }
        tip
    }

    /// One button; true when the mouse is over it.
    fn inventory_button(&mut self, b: Btn, x: f32, y: f32, mouse: (f32, f32)) -> bool {
        let s = self.ui.scale;
        let size = 12.0 * s;
        let over = mouse.0 >= x && mouse.0 < x + size && mouse.1 >= y && mouse.1 < y + size;
        self.ui.rect(x, y, size, size, [55, 55, 55, 255]);
        self.ui.rect(x + s, y + s, size - 2.0 * s, size - 2.0 * s, if over { [170, 180, 225, 255] } else { [150, 150, 150, 255] });
        self.ui.rect(x + s, y + s, size - 2.0 * s, s, [220, 220, 220, 255]);
        for (r, line) in glyph(b).iter().enumerate() {
            for (c, ch) in line.chars().enumerate() {
                let col = match ch {
                    'X' => [40, 40, 50, 255],
                    'o' => [235, 205, 120, 255],
                    _ => continue,
                };
                self.ui.rect(x + (1.5 + c as f32) * s, y + (1.0 + r as f32) * s, s, s, col);
            }
        }
        if over && self.clicked {
            self.clicked = false;
            self.press_inventory_button(b);
        }
        over
    }

    fn press_inventory_button(&mut self, b: Btn) {
        match b {
            Btn::SortBag => self.sort_backpack(),
            Btn::StackNearby => self.quick_stack(),
            Btn::LootAll => self.loot_all(),
            Btn::DepositAll => self.deposit_all(false),
            Btn::StackHere => self.deposit_all(true),
            Btn::SortChest => self.sort_open_chest(),
        }
    }

    fn open_chest_mut(&mut self) -> Option<&mut Inventory> {
        let Open::Chest(p) = self.surv.open else { return None };
        match self.surv.containers.get_mut(&p)? {
            Container::Chest(i) => Some(i),
            _ => None,
        }
    }

    /// Everything in the open chest into the bag (hotbar first, like picking things up).
    pub(crate) fn loot_all(&mut self) {
        let Some(chest) = self.open_chest_mut() else { return };
        let stacks: Vec<(usize, ItemStack)> = chest.slots.iter().enumerate().filter_map(|(i, s)| s.map(|s| (i, s))).collect();
        let mut moved = 0;
        for (i, st) in stacks {
            let rest = self.surv.inv.add(&self.items, st);
            if rest.map(|r| r.count) != Some(st.count) {
                moved += 1;
            }
            if let Some(chest) = self.open_chest_mut() {
                chest.slots[i] = rest;
            }
        }
        if moved > 0 {
            self.sfx(Sfx::Click);
        } else {
            self.toast("A mochila está cheia", 1.5);
        }
    }

    /// The bag (not the hotbar, not favourites) into the open chest. `same_only`: only what the
    /// chest already holds some of (Terraria's quick stack).
    pub(crate) fn deposit_all(&mut self, same_only: bool) {
        let favs: Vec<bool> = (0..INV_SIZE).map(|k| self.is_favorite(k)).collect();
        let mut moved = 0;
        for k in 9..INV_SIZE {
            if favs[k] {
                continue;
            }
            let Some(st) = self.surv.inv.slots[k] else { continue };
            let items = &self.items;
            let Open::Chest(p) = self.surv.open else { return };
            let Some(Container::Chest(chest)) = self.surv.containers.get_mut(&p) else { return };
            if same_only && !chest.slots.iter().flatten().any(|s| s.stacks_with(&st)) {
                continue;
            }
            let rest = chest.add(items, st);
            if rest.map(|r| r.count) != Some(st.count) {
                moved += 1;
            }
            self.surv.inv.slots[k] = rest;
        }
        if moved > 0 {
            self.sfx(Sfx::Place(Material::Wood));
        } else if same_only {
            self.toast("O baú não tem nada igual ao da mochila", 1.5);
        } else {
            self.toast("Nada para guardar (ou o baú está cheio)", 1.5);
        }
    }

    /// The open chest merged and sorted in catalogue order.
    pub(crate) fn sort_open_chest(&mut self) {
        let cat = self.catalog();
        let items = &self.items;
        let Open::Chest(p) = self.surv.open else { return };
        let Some(Container::Chest(chest)) = self.surv.containers.get_mut(&p) else { return };
        let n = chest.slots.len();
        let mut merged = Inventory::new(n);
        for st in chest.slots.iter_mut().filter_map(|s| s.take()) {
            // Everything fits back: merging only ever frees slots.
            let _ = merged.add(items, st);
        }
        let mut stacks: Vec<ItemStack> = merged.slots.into_iter().flatten().collect();
        stacks.sort_by_key(|s| (cat.rank(s.item), std::cmp::Reverse(s.count)));
        for (i, st) in stacks.into_iter().enumerate() {
            chest.slots[i] = Some(st);
        }
        self.sfx(Sfx::Click);
    }
}
