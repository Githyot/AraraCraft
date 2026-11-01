//! The creative inventory, laid out like Java's: seven tabs on top (building, coloured, natural,
//! functional and redstone blocks, saved hotbars, search), six below (tools and utilities,
//! combat, food and drinks, ingredients, spawn eggs and the player's own inventory) and four down
//! the right side for the Terraria and mod content (magic and summons, accessories and looks,
//! furniture and decoration, favourites). Which item goes where is catalog.rs.
//!
//! Item tabs show a 9×5 grid that scrolls; each section starts on its own row under a thin line,
//! and the list on the left names the tab's sections (click one to jump there), with buttons for
//! the origin filter (Minecraft, Terraria, mods, ours) and the order (by section or A to Z).
//! Alt+click or F over an item marks it as a favourite. The inventory tab shows the player in 3D
//! with armour, second hand, the rows and a bin.

use std::sync::Arc;

use voxel::item::{ItemId, ItemStack};
use winit::keyboard::KeyCode;

pub use crate::catalog::{Tab, TABS};
use crate::catalog::{Catalog, Query, ORIGIN_FILTERS};
use crate::survival::{item_icon, rect_around, Open, Slot};
use crate::ui::WHITE;
use crate::{Game, Screen};

/// One row of the creative grid: up to nine items, and the section it opens (if it does).
#[derive(Clone, Debug, Default)]
pub struct Row {
    pub items: [Option<ItemId>; 9],
    pub head: Option<u16>,
}

/// Panel size in interface pixels, and the side list's width.
const PANEL: (f32, f32) = (195.0, 136.0);
const SIDE_W: f32 = 140.0;
const SIDE_GAP: f32 = 4.0;
/// How far the side tabs stick out of the panel.
const SIDE_TAB: f32 = 28.0;

impl Tab {
    /// Left edge of a tab in panel pixels and where it hangs: 0 on top, 1 below, 2 at the right.
    fn place(i: usize) -> (f32, f32, u8) {
        if i >= 13 {
            return (PANEL.0 - 4.0, 4.0 + (i - 13) as f32 * 29.0, 2);
        }
        let bottom = i >= 7;
        let col = if bottom { i - 7 } else { i };
        // The last tabs of each row sit at the right edge, like the classic screen.
        let x = match (bottom, col) {
            (false, 5) => 140.0,
            (false, 6) | (true, 5) => 169.0,
            _ => col as f32 * 29.0,
        };
        (x, if bottom { PANEL.1 - 4.0 } else { -28.0 }, u8::from(bottom))
    }

    fn lists_items(self) -> bool {
        !matches!(self, Tab::Saved | Tab::Inventory)
    }
}

impl Game {
    /// The catalogue, built the first time it is needed and again when plugins add items.
    pub(crate) fn catalog(&self) -> Arc<Catalog> {
        let mut g = self.catalog_cache.lock().unwrap();
        match g.as_ref() {
            Some(c) if c.built_for == self.items.items.len() => c.clone(),
            _ => {
                let c = Arc::new(Catalog::build(&self.reg, &self.items));
                *g = Some(c.clone());
                c
            }
        }
    }

    /// Which tab an item belongs to (None: not shown, like the hidden orientations of a block).
    pub(crate) fn creative_tab_of(&self, id: ItemId) -> Option<Tab> {
        self.catalog().tab_of(id)
    }

    /// A tab's items in order (all of them, no filter).
    pub(crate) fn creative_list(&self, tab: Tab) -> Vec<ItemId> {
        self.catalog().list(tab).to_vec()
    }

    pub(crate) fn is_creative_favorite(&self, id: ItemId) -> bool {
        let n = &self.items.get(id).name;
        self.settings.creative_favorites.iter().any(|f| f == n)
    }

    pub(crate) fn toggle_creative_favorite(&mut self, id: ItemId) {
        let n = self.items.get(id).name.clone();
        let label = self.items.get(id).display_name.clone();
        if let Some(i) = self.settings.creative_favorites.iter().position(|f| *f == n) {
            self.settings.creative_favorites.remove(i);
            self.toast(format!("{label}: fora dos favoritos"), 1.5);
        } else {
            self.settings.creative_favorites.push(n);
            self.toast(format!("{label}: nos favoritos (aba Favoritos, à direita)"), 2.0);
        }
        self.settings.save(&self.base.join("settings.ron"));
        self.surv.crows = None;
    }

    /// The rows of a tab as the grid shows them, with the search, origin filter and order.
    pub(crate) fn creative_rows(&mut self, tab: Tab) -> Arc<Vec<Row>> {
        let cat = self.catalog();
        let key = format!(
            "{tab:?}|{}|{}|{}|{}|{}",
            self.surv.csearch,
            self.settings.creative_origin,
            self.settings.creative_sort,
            self.settings.creative_favorites.len(),
            cat.built_for
        );
        if let Some((k, rows)) = &self.surv.crows {
            if *k == key {
                return rows.clone();
            }
        }
        let rows = Arc::new(build_rows(self, &cat, tab));
        self.surv.crows = Some((key, rows.clone()));
        rows
    }

    /// An old creative world keeps its blocks: the hotbar of block ids becomes item stacks.
    pub(crate) fn fill_creative_hotbar(&mut self) {
        if self.survival || self.surv.inv.slots[..9].iter().any(|s| s.is_some()) {
            return;
        }
        for i in 0..9 {
            if let Some(it) = self.items.of_block(self.hotbar[i]) {
                self.surv.inv.slots[i] = Some(ItemStack::new(it, 1));
            }
        }
    }

    pub(crate) fn creative_screen_open(&self) -> bool {
        self.screen == Screen::Inventory && !self.survival && self.surv.open == Open::Craft
    }

    fn open_creative_tab(&mut self, i: usize) {
        if self.surv.ctab != i {
            self.surv.ctab = i.min(TABS.len() - 1);
            self.surv.cscroll = 0;
            self.surv.csec_scroll = 0;
        }
    }

    /// Keys while the search tab has the text box: letters go into it. Returns true if used.
    pub(crate) fn creative_key(&mut self, code: KeyCode) -> bool {
        // The search box of the survival item list has the keyboard.
        if self.screen == Screen::Inventory && self.survival && self.surv.bsearch_on {
            match code {
                KeyCode::Escape | KeyCode::Enter => self.surv.bsearch_on = false,
                KeyCode::Backspace => {
                    self.surv.bsearch.pop();
                }
                _ => {}
            }
            self.surv.book_scroll = 0;
            return true;
        }
        if !self.creative_screen_open() || code == KeyCode::Escape {
            return false;
        }
        let ctrl = self.keys.contains(&KeyCode::ControlLeft) || self.keys.contains(&KeyCode::ControlRight);
        // Ctrl+F (or Tab) jumps to the search from any tab.
        if ctrl && code == KeyCode::KeyF {
            let i = TABS.iter().position(|t| *t == Tab::Search).unwrap();
            self.open_creative_tab(i);
            return true;
        }
        if TABS[self.surv.ctab] != Tab::Search {
            // F over an item of the grid: favourite on or off.
            if code == KeyCode::KeyF {
                if let Some(id) = self.surv.chover {
                    self.toggle_creative_favorite(id);
                    return true;
                }
            }
            return false;
        }
        match code {
            KeyCode::Backspace => {
                if ctrl {
                    self.surv.csearch.clear();
                } else {
                    self.surv.csearch.pop();
                }
                self.surv.cscroll = 0;
            }
            KeyCode::Delete => {
                self.surv.csearch.clear();
                self.surv.cscroll = 0;
            }
            _ => {}
        }
        true
    }

    pub(crate) fn creative_type(&mut self, text: &str) {
        if self.screen == Screen::Inventory && self.survival && self.surv.bsearch_on {
            for c in text.chars() {
                if !c.is_control() && self.surv.bsearch.chars().count() < 20 {
                    self.surv.bsearch.push(c);
                }
            }
            self.surv.book_scroll = 0;
            return;
        }
        if !self.creative_screen_open() || TABS[self.surv.ctab] != Tab::Search {
            return;
        }
        let ctrl = self.keys.contains(&KeyCode::ControlLeft) || self.keys.contains(&KeyCode::ControlRight);
        for c in text.chars() {
            if !c.is_control() && !ctrl && self.surv.csearch.chars().count() < 32 {
                self.surv.csearch.push(c);
            }
        }
        self.surv.cscroll = 0;
    }

    pub(crate) fn creative_scroll(&mut self, up: bool) {
        // Over the side list the wheel scrolls the list of sections.
        if self.surv.cside_hover {
            self.surv.csec_scroll = if up { self.surv.csec_scroll.saturating_sub(1) } else { self.surv.csec_scroll + 1 };
            return;
        }
        self.surv.cscroll = if up { self.surv.cscroll.saturating_sub(1) } else { self.surv.cscroll + 1 };
    }

    pub(crate) fn draw_creative_inventory(&mut self, w: f32, h: f32, mouse: (f32, f32)) {
        let s = self.ui.scale;
        let (pw, ph) = (PANEL.0 * s, PANEL.1 * s);
        let tab = TABS[self.surv.ctab.min(TABS.len() - 1)];
        // The side list goes left of the panel when the screen is wide enough; the panel moves
        // right so the two (and the side tabs) are centred together.
        let group = SIDE_W + SIDE_GAP + PANEL.0 + SIDE_TAB;
        let side = w >= (group + 8.0) * s;
        let x0 = if side { ((w - group * s) / 2.0).floor() + (SIDE_W + SIDE_GAP) * s } else { ((w - pw - SIDE_TAB * s) / 2.0).floor() };
        let y0 = ((h - ph) / 2.0).floor();
        let at = |x: f32, y: f32| (x0 + x * s, y0 + y * s);
        let inside = |p: (f32, f32), wd: f32, ht: f32| mouse.0 >= p.0 && mouse.0 < p.0 + wd * s && mouse.1 >= p.1 && mouse.1 < p.1 + ht * s;
        let shift = self.keys.contains(&KeyCode::ShiftLeft) || self.keys.contains(&KeyCode::ShiftRight);
        let alt = self.keys.contains(&KeyCode::AltLeft) || self.keys.contains(&KeyCode::AltRight);
        let (click, rclick) = (self.clicked, self.rclicked);

        let hole = (tab == Tab::Inventory).then(|| {
            let (hx, hy) = at(73.0, 6.0);
            (hx, hy, 32.0 * s, 43.0 * s)
        });
        self.surv.figure_box = hole;
        rect_around(&mut self.ui, (0.0, 0.0, w, h), hole, [0, 0, 0, 110]);

        // Tabs behind the panel first, then the panel, then the open tab joined to it.
        let mut over_tab: Option<usize> = None;
        for pass in 0..2 {
            for i in 0..TABS.len() {
                let on = i == self.surv.ctab;
                if (pass == 1) != on {
                    continue;
                }
                let (tx, ty, edge) = Tab::place(i);
                let (x, y) = at(tx, ty);
                let (tw, th) = if edge == 2 { (32.0 * s, 26.0 * s) } else { (26.0 * s, 32.0 * s) };
                // A pack's tab picture when it has one: number 1 joins the left edge, 7 the right.
                let col = if edge == 1 { i - 7 } else { i };
                let n = if tx >= 169.0 { 7 } else { col + 1 };
                let name = format!(
                    "sprites/container/creative_inventory/tab_{}_{}_{n}",
                    if edge == 1 { "bottom" } else { "top" },
                    if on { "selected" } else { "unselected" }
                );
                if edge == 2 || !self.ui.sprite(&name, x, y, tw, th) {
                    let bg = if on { [198, 198, 198, 255] } else { [160, 160, 160, 255] };
                    self.ui.rect(x, y, tw, th, bg);
                    self.ui.rect(x, y, tw, s, [255, 255, 255, 255]);
                    self.ui.rect(x + tw - s, y, s, th, [85, 85, 85, 255]);
                    match edge {
                        1 => {
                            self.ui.rect(x, y, s, th, [255, 255, 255, 255]);
                            self.ui.rect(x, y + th - s, tw, s, [85, 85, 85, 255]);
                        }
                        2 => self.ui.rect(x, y + th - s, tw, s, [85, 85, 85, 255]),
                        _ => self.ui.rect(x, y, s, th, [255, 255, 255, 255]),
                    }
                }
                let (ix, iy) = match edge {
                    1 => (x + 5.0 * s, y + 10.0 * s),
                    2 => (x + 10.0 * s, y + 5.0 * s),
                    _ => (x + 5.0 * s, y + 7.0 * s),
                };
                self.draw_tab_icon(TABS[i], ix, iy);
                if mouse.0 >= x && mouse.0 < x + tw && mouse.1 >= y && mouse.1 < y + th {
                    over_tab = Some(i);
                }
            }
            if pass == 0 {
                let bg = match tab {
                    Tab::Inventory => "container/creative_inventory/tab_inventory",
                    Tab::Search => "container/creative_inventory/tab_item_search",
                    _ => "container/creative_inventory/tab_items",
                };
                self.surv.cart = self.ui.has_sprite(bg);
                if self.surv.cart {
                    // The pack's picture, cut around the hole the 3D figure shows through.
                    sprite_around(&mut self.ui, bg, (x0, y0, pw, ph), hole, s);
                } else {
                    // The panel, with the classic bevel, around the hole the 3D figure shows through.
                    rect_around(&mut self.ui, (x0, y0, pw, ph), hole, [198, 198, 198, 255]);
                    self.ui.rect(x0, y0, pw, s, [255, 255, 255, 255]);
                    self.ui.rect(x0, y0, s, ph, [255, 255, 255, 255]);
                    self.ui.rect(x0 + pw - s, y0, s, ph, [85, 85, 85, 255]);
                    self.ui.rect(x0, y0 + ph - s, pw, s, [85, 85, 85, 255]);
                }
            }
        }

        let mut hovered: Option<Slot> = None;
        let mut tip: Option<ItemId> = None;
        let mut slot_at = |game: &mut Game, slot: Slot, p: (f32, f32)| {
            draw_slot(game, p.0, p.1);
            if inside(p, 18.0, 18.0) {
                hovered = Some(slot);
                game.ui.rect(p.0 + s, p.1 + s, 16.0 * s, 16.0 * s, [255, 255, 255, 80]);
            }
            if let Some(st) = game.slot_get(slot) {
                item_icon(&mut game.ui, &game.reg, &game.items, game.item_base, &st, p.0 + s, p.1 + s, 16.0 * s);
            }
        };
        let dark = [64, 64, 64, 255];
        self.surv.chover = None;
        self.surv.cside_hover = false;
        let mut over_side = false;

        match tab {
            Tab::Inventory => {
                // Helmet and chestplate left of the figure, leggings and boots right, like the classic screen.
                let spots = [(54.0, 6.0), (54.0, 33.0), (108.0, 6.0), (108.0, 33.0)];
                for (i, &(sx, sy)) in spots.iter().enumerate() {
                    let p = at(sx, sy);
                    slot_at(self, Slot::Armor(i), p);
                    if self.surv.armor[i].is_none() {
                        slot_outline(self, i, p.0, p.1);
                    }
                }
                let op = at(35.0, 20.0);
                slot_at(self, Slot::Offhand, op);
                if self.surv.offhand.is_none() {
                    slot_outline(self, 4, op.0, op.1);
                }
                for i in 9..36 {
                    let k = i - 9;
                    slot_at(self, Slot::Inv(i), at(9.0 + (k % 9) as f32 * 18.0, 54.0 + (k / 9) as f32 * 18.0));
                }
                // Bin: drop the held stack here to delete it (shift-click empties the inventory).
                let (bx, by) = at(173.0, 112.0);
                if !self.surv.cart {
                    self.ui.rect(bx, by, 18.0 * s, 18.0 * s, [85, 55, 55, 255]);
                    self.ui.rect(bx + s, by + s, 16.0 * s, 16.0 * s, [184, 131, 131, 255]);
                    for k in 0..12 {
                        let d = (k as f32 + 2.0) * s;
                        self.ui.rect(bx + d, by + d, 2.0 * s, s, [60, 40, 40, 255]);
                        self.ui.rect(bx + 16.0 * s - d, by + d, 2.0 * s, s, [60, 40, 40, 255]);
                    }
                }
                if inside((bx, by), 18.0, 18.0) && click {
                    if shift {
                        self.surv.inv.slots.iter_mut().for_each(|x| *x = None);
                    }
                    self.surv.cursor = None;
                    self.clicked = false;
                }
            }
            Tab::Saved => {
                self.ui.label(x0 + 8.0 * s, y0 + 6.0 * s, "Clique carrega, Shift+clique salva", dark, s);
                let saved = self.settings.saved_hotbars.clone();
                for r in 0..5 {
                    let row: Vec<String> = saved.get(r).cloned().unwrap_or_default();
                    let y = 18.0 + r as f32 * 18.0;
                    for c in 0..9 {
                        let p = at(9.0 + c as f32 * 18.0, y);
                        draw_slot(self, p.0, p.1);
                        if let Some(id) = row.get(c).and_then(|n| self.items.id(n)) {
                            item_icon(&mut self.ui, &self.reg, &self.items, self.item_base, &ItemStack::new(id, 1), p.0 + s, p.1 + s, 16.0 * s);
                        }
                    }
                    let rp = at(9.0, y);
                    if inside(rp, 162.0, 18.0) {
                        self.ui.rect(rp.0, rp.1, 162.0 * s, 18.0 * s, [255, 255, 255, 50]);
                        if click {
                            self.clicked = false;
                            if shift {
                                let names: Vec<String> = (0..9).map(|i| self.surv.inv.slots[i].map(|st| self.items.get(st.item).name.clone()).unwrap_or_default()).collect();
                                while self.settings.saved_hotbars.len() <= r {
                                    self.settings.saved_hotbars.push(Vec::new());
                                }
                                self.settings.saved_hotbars[r] = names;
                                self.settings.save(&self.base.join("settings.ron"));
                                self.toast("Barra salva", 1.5);
                            } else if !row.is_empty() {
                                for i in 0..9 {
                                    self.surv.inv.slots[i] = row.get(i).and_then(|n| self.items.id(n)).map(|id| ItemStack::new(id, self.items.get(id).max_stack.max(1)));
                                }
                            }
                        }
                    }
                }
            }
            t => {
                let rows = self.creative_rows(t);
                let max_scroll = rows.len().saturating_sub(5);
                self.surv.cscroll = self.surv.cscroll.min(max_scroll);
                let cat = self.catalog();
                // The section of the first row shown (the header names it).
                let cur_sec = (0..=self.surv.cscroll.min(rows.len().saturating_sub(1))).rev().find_map(|r| rows.get(r).and_then(|row| row.head));
                let title_room = if t == Tab::Search { 72.0 } else { 160.0 } * s;
                let title = match (t, cur_sec) {
                    (Tab::Search, _) => t.title().to_string(),
                    (_, Some(k)) if self.settings.creative_sort == 0 => format!("{} » {}", t.title(), cat.sections.get(&t).and_then(|v| v.get(k as usize)).cloned().unwrap_or_default()),
                    _ => t.title().to_string(),
                };
                // When "Tab » Section" is too long: the tab's name if the side list marks the
                // section, else the section.
                let title = if self.ui.text_width(&title, s) <= title_room || !title.contains(" » ") {
                    title
                } else if side {
                    t.title().to_string()
                } else {
                    title.split(" » ").nth(1).unwrap_or_default().to_string()
                };
                let title = fit_text(&self.ui, &title, title_room, s);
                self.ui.label(x0 + 8.0 * s, y0 + 6.0 * s, &title, dark, s);
                if t == Tab::Search {
                    let (bx, by) = at(82.0, 5.0);
                    if !self.surv.cart {
                        self.ui.rect(bx, by, 89.0 * s, 12.0 * s, [0, 0, 0, 255]);
                        self.ui.frame(bx, by, 89.0 * s, 12.0 * s, s, [160, 160, 160, 255]);
                    }
                    let blink = if (self.now * 2.0) as i32 % 2 == 0 { "_" } else { "" };
                    if self.surv.csearch.is_empty() {
                        self.ui.text(bx + 2.0 * s, by + 2.0 * s, &format!("{blink}nome, @mod, #seção"), [120, 120, 120, 255]);
                    } else {
                        // The end of a long search stays in view.
                        let mut shown = self.surv.csearch.clone();
                        while self.ui.text_width(&format!("{shown}_"), s) > 85.0 * s && !shown.is_empty() {
                            shown.remove(0);
                        }
                        self.ui.text(bx + 2.0 * s, by + 2.0 * s, &format!("{shown}{blink}"), WHITE);
                    }
                }
                let mut grid_hover: Option<ItemId> = None;
                for r in 0..5 {
                    let row = rows.get(r + self.surv.cscroll);
                    for c in 0..9 {
                        let p = at(9.0 + c as f32 * 18.0, 18.0 + r as f32 * 18.0);
                        draw_slot(self, p.0, p.1);
                        let Some(id) = row.and_then(|row| row.items[c]) else { continue };
                        if inside(p, 18.0, 18.0) {
                            grid_hover = Some(id);
                            self.ui.rect(p.0 + s, p.1 + s, 16.0 * s, 16.0 * s, [255, 255, 255, 80]);
                        }
                        item_icon(&mut self.ui, &self.reg, &self.items, self.item_base, &ItemStack::new(id, 1), p.0 + s, p.1 + s, 16.0 * s);
                        if t != Tab::Favorites && self.is_creative_favorite(id) {
                            draw_star(&mut self.ui, p.0 + 12.0 * s, p.1 + s, s, [255, 205, 60, 255]);
                        }
                    }
                    // A line over each new section (not over the first row shown).
                    if r > 0 && row.is_some_and(|row| row.head.is_some()) && self.settings.creative_sort == 0 {
                        let (lx, ly) = at(9.0, 18.0 + r as f32 * 18.0);
                        self.ui.rect(lx, ly - s, 162.0 * s, s, [70, 90, 160, 255]);
                    }
                }
                if rows.is_empty() {
                    let msg = match t {
                        Tab::Favorites => "Alt+clique ou F num item: favorito",
                        Tab::Search => "Nada encontrado",
                        _ => "Nada com este filtro de origem",
                    };
                    let msg = fit_text(&self.ui, msg, 160.0 * s, s);
                    self.ui.label(x0 + 10.0 * s, y0 + 24.0 * s, &msg, [90, 90, 90, 255], s);
                }
                // Scroll bar.
                let (sx, sy) = at(175.0, 18.0);
                if !self.surv.cart {
                    self.ui.rect(sx, sy, 14.0 * s, 90.0 * s, [139, 139, 139, 255]);
                    self.ui.rect(sx, sy, 14.0 * s, s, [55, 55, 55, 255]);
                    self.ui.rect(sx, sy, s, 90.0 * s, [55, 55, 55, 255]);
                }
                let k = if max_scroll > 0 { self.surv.cscroll as f32 / max_scroll as f32 } else { 0.0 };
                let ty = sy + s + k * (90.0 - 17.0) * s;
                let thumb_art = if max_scroll > 0 { "sprites/container/creative_inventory/scroller" } else { "sprites/container/creative_inventory/scroller_disabled" };
                if !self.ui.sprite(thumb_art, sx + s, ty, 12.0 * s, 15.0 * s) {
                    let thumb = if max_scroll > 0 { [198, 198, 198, 255] } else { [150, 150, 150, 255] };
                    self.ui.rect(sx + s, ty, 12.0 * s, 15.0 * s, thumb);
                    self.ui.rect(sx + s, ty, 12.0 * s, s, [255, 255, 255, 255]);
                    self.ui.rect(sx + 12.0 * s, ty, s, 15.0 * s, [85, 85, 85, 255]);
                }
                if (self.rdown || self.mouse_held[0]) && mouse.0 >= sx && mouse.0 < sx + 14.0 * s && mouse.1 >= sy && mouse.1 < sy + 90.0 * s && max_scroll > 0 {
                    let f = ((mouse.1 - sy) / (90.0 * s)).clamp(0.0, 1.0);
                    self.surv.cscroll = (f * max_scroll as f32).round() as usize;
                }
                if side {
                    over_side = self.draw_creative_side(t, &rows, cur_sec, (x0 - (SIDE_W + SIDE_GAP) * s, y0 - 28.0 * s, SIDE_W * s, ph + 56.0 * s), mouse);
                }
                if let Some(id) = grid_hover {
                    tip = Some(id);
                    self.surv.chover = Some(id);
                    let max = self.items.get(id).max_stack.max(1);
                    // Items are endless in creative, so they come one at a time; middle click
                    // (or shift-click into the inventory) gives a full stack.
                    if let Some(k) = self.inv_number_key.take() {
                        self.surv.inv.slots[k] = Some(ItemStack::new(id, 1));
                    }
                    if self.mclicked && self.surv.cursor.is_none() {
                        self.mclicked = false;
                        self.surv.cursor = Some(ItemStack::new(id, max));
                    }
                    if click && alt {
                        self.clicked = false;
                        self.toggle_creative_favorite(id);
                    } else if click || rclick {
                        self.clicked = false;
                        self.rclicked = false;
                        match self.surv.cursor {
                            // Holding something: clicking the list throws it away.
                            Some(c) if !(c.item == id && c.count < max) => self.surv.cursor = None,
                            Some(c) => self.surv.cursor = Some(ItemStack { count: c.count + 1, ..c }),
                            None if shift => {
                                let _ = self.surv.inv.add(&self.items, ItemStack::new(id, max));
                            }
                            None => self.surv.cursor = Some(ItemStack::new(id, 1)),
                        }
                    }
                } else if (click || rclick) && self.surv.cursor.is_some() && inside(at(9.0, 18.0), 162.0, 90.0) {
                    self.surv.cursor = None;
                    self.clicked = false;
                }
            }
        }
        // Hotbar row, on every tab but the saved ones.
        if tab != Tab::Saved {
            for i in 0..9 {
                slot_at(self, Slot::Inv(i), at(9.0 + i as f32 * 18.0, 112.0));
            }
        }
        // Clicks on the player's own slots work like the survival screen.
        if let Some(slot) = hovered {
            if let Some(st) = self.slot_get(slot) {
                tip = Some(st.item);
            }
            if self.clicked || self.rclicked {
                let right = self.rclicked && !self.clicked;
                self.click_slot(slot, right, shift);
            }
            if let Some(k) = self.inv_number_key.take() {
                let cur = self.slot_get(slot);
                let hot = self.surv.inv.slots[k];
                self.surv.inv.slots[k] = cur;
                if slot != Slot::Inv(k) {
                    self.slot_set(slot, hot);
                }
            }
        }
        self.inv_number_key = None;
        if let Some(i) = over_tab {
            if self.clicked {
                self.clicked = false;
                self.open_creative_tab(i);
            }
            if tip.is_none() {
                let t = TABS[i].title();
                self.tooltip(t, mouse);
            }
        }
        // Clicking outside the window throws the held stack (right click: one item).
        let in_panel = mouse.0 >= x0 && mouse.0 < x0 + pw + SIDE_TAB * s && mouse.1 >= y0 - 28.0 * s && mouse.1 < y0 + ph + 28.0 * s;
        if (self.clicked || self.rclicked) && !in_panel && over_tab.is_none() && !over_side {
            if let Some(c) = self.surv.cursor {
                let n = if self.rclicked && !self.clicked { 1 } else { c.count };
                self.surv.cursor = (c.count > n).then_some(ItemStack { count: c.count - n, ..c });
                self.throw_stack(ItemStack { count: n, ..c });
            }
        }
        // What the keys do, under the panel when it fits.
        if tab.lists_items() {
            let hint = "Clique: 1  Shift/Meio: pilha  Alt+clique ou F: favorito  Ctrl+F: buscar";
            let mut ts = s;
            while ts > 1.0 && self.ui.text_width(hint, ts) > w - 8.0 * s {
                ts -= 1.0;
            }
            let tw = self.ui.text_width(hint, ts);
            // Over the panel's top tabs there is room; under it the hotbar would be covered.
            let hy = y0 - 28.0 * s - 12.0 * ts;
            if hy >= 0.0 {
                self.ui.text_scaled((w - tw) / 2.0, hy, hint, [220, 220, 220, 255], ts);
            }
        }
        if let Some(id) = tip {
            let st = ItemStack::new(id, 1);
            let mut lines = vec![(self.items.get(id).display_name.clone(), WHITE)];
            lines.extend(self.item_details(&st));
            // Where it is listed and where it comes from (blue, like a mod's name in Java).
            let cat = self.catalog();
            if let Some(e) = cat.entry(id) {
                lines.push((format!("{} » {}", e.tab.title(), cat.section_name(e)), [170, 170, 170, 255]));
                lines.push((e.origin.tooltip(), [110, 140, 255, 255]));
            }
            if self.is_creative_favorite(id) {
                lines.push(("* Favorito".to_string(), [255, 205, 60, 255]));
            }
            self.draw_tooltip_lines(&lines, mouse);
        }
        if let Some(c) = self.surv.cursor {
            item_icon(&mut self.ui, &self.reg, &self.items, self.item_base, &c, mouse.0 - 8.0 * s, mouse.1 - 8.0 * s, 16.0 * s);
        }
    }

    /// The list left of the panel: origin and order buttons, then the tab's sections (click one
    /// to jump to it). Returns whether the mouse is over it.
    fn draw_creative_side(&mut self, tab: Tab, rows: &[Row], cur_sec: Option<u16>, area: (f32, f32, f32, f32), mouse: (f32, f32)) -> bool {
        let s = self.ui.scale;
        let (x, y, w, h) = area;
        let over = mouse.0 >= x && mouse.0 < x + w && mouse.1 >= y && mouse.1 < y + h;
        self.surv.cside_hover = over;
        self.ui.rect(x, y, w, h, [198, 198, 198, 255]);
        self.ui.rect(x, y, w, s, [255, 255, 255, 255]);
        self.ui.rect(x, y, s, h, [255, 255, 255, 255]);
        self.ui.rect(x + w - s, y, s, h, [85, 85, 85, 255]);
        self.ui.rect(x, y + h - s, w, s, [85, 85, 85, 255]);
        let dark = [64, 64, 64, 255];
        let (bx, bw, bh) = (x + 4.0 * s, w - 8.0 * s, 14.0 * s);
        let mut cy = y + 4.0 * s;
        // Origin filter and order: click goes forward, right click back.
        let origin = format!("Origem: {}", ORIGIN_FILTERS[self.settings.creative_origin.min(ORIGIN_FILTERS.len() - 1)]);
        let ob = self.ui.button_text(bx, cy, bw, bh, &origin, mouse, s);
        cy += bh + 2.0 * s;
        let order = if self.settings.creative_sort == 0 { "Ordem: por seção" } else { "Ordem: A a Z" };
        let sb = self.ui.button_text(bx, cy, bw, bh, order, mouse, s);
        cy += bh + 4.0 * s;
        let (click, rclick) = (self.clicked, self.rclicked);
        if (click || rclick) && ob {
            let n = ORIGIN_FILTERS.len();
            self.settings.creative_origin = if rclick && !click { (self.settings.creative_origin + n - 1) % n } else { (self.settings.creative_origin + 1) % n };
            self.surv.cscroll = 0;
            self.surv.csec_scroll = 0;
            self.settings.save(&self.base.join("settings.ron"));
            self.clicked = false;
            self.rclicked = false;
        }
        if (click || rclick) && sb {
            self.settings.creative_sort = 1 - self.settings.creative_sort.min(1);
            self.surv.cscroll = 0;
            self.settings.save(&self.base.join("settings.ron"));
            self.clicked = false;
            self.rclicked = false;
        }
        let line = 11.0 * s;
        let room = ((y + h - 4.0 * s - cy) / line).floor().max(1.0) as usize;
        // Search and A-Z have no sections: the list says how to search instead.
        if tab == Tab::Search || tab == Tab::Favorites || self.settings.creative_sort != 0 {
            let total: usize = rows.iter().map(|r| r.items.iter().flatten().count()).sum();
            let mut lines: Vec<(String, [u8; 4])> = vec![(format!("{total} itens"), dark)];
            if tab == Tab::Search {
                for l in ["Busca pelo nome,", "com ou sem acento,", "em PT ou EN.", "Várias palavras:", "todas valem.", "", "Origem:", "@terraria @mods", "@minecraft @aether", "Seção:", "#espadas #poções", "", "Ctrl+F: buscar", "Del: limpar"] {
                    lines.push((l.to_string(), [90, 90, 90, 255]));
                }
            } else if tab == Tab::Favorites {
                for l in ["Alt+clique ou F", "num item marca", "ou desmarca."] {
                    lines.push((l.to_string(), [90, 90, 90, 255]));
                }
            }
            for (k, (l, c)) in lines.iter().enumerate().take(room) {
                let l = fit_text(&self.ui, l, w - 8.0 * s, s);
                self.ui.label(x + 5.0 * s, cy + k as f32 * line + 2.0 * s, &l, *c, s);
            }
            return over;
        }
        // Sections of this tab: name and how many items, the one in view marked.
        let cat = self.catalog();
        let names = cat.sections.get(&tab).cloned().unwrap_or_default();
        let mut secs: Vec<(u16, usize, usize)> = Vec::new(); // (section, first row, items)
        for (r, row) in rows.iter().enumerate() {
            let n = row.items.iter().flatten().count();
            match row.head {
                Some(k) => secs.push((k, r, n)),
                None => {
                    if let Some(last) = secs.last_mut() {
                        last.2 += n;
                    }
                }
            }
        }
        self.surv.csec_scroll = self.surv.csec_scroll.min(secs.len().saturating_sub(room));
        // Keep the section in view visible in the list.
        if let Some(i) = secs.iter().position(|e| Some(e.0) == cur_sec) {
            if !over {
                if i < self.surv.csec_scroll {
                    self.surv.csec_scroll = i;
                } else if i >= self.surv.csec_scroll + room {
                    self.surv.csec_scroll = i + 1 - room;
                }
            }
        }
        for (k, &(sec, first, n)) in secs.iter().enumerate().skip(self.surv.csec_scroll).take(room) {
            let ly = cy + (k - self.surv.csec_scroll) as f32 * line;
            let hot = mouse.0 >= x && mouse.0 < x + w && mouse.1 >= ly && mouse.1 < ly + line;
            if Some(sec) == cur_sec {
                self.ui.rect(x + 2.0 * s, ly, w - 4.0 * s, line, [120, 135, 200, 255]);
            } else if hot {
                self.ui.rect(x + 2.0 * s, ly, w - 4.0 * s, line, [225, 225, 235, 255]);
            }
            let count = n.to_string();
            let cw = self.ui.text_width(&count, s);
            let name = names.get(sec as usize).cloned().unwrap_or_default();
            let name = fit_text(&self.ui, &name, w - cw - 12.0 * s, s);
            let col = if Some(sec) == cur_sec { WHITE } else { dark };
            self.ui.label(x + 5.0 * s, ly + 2.0 * s, &name, col, s);
            self.ui.label(x + w - cw - 5.0 * s, ly + 2.0 * s, &count, if Some(sec) == cur_sec { WHITE } else { [110, 110, 110, 255] }, s);
            if hot && self.clicked {
                self.clicked = false;
                self.surv.cscroll = first.min(rows.len().saturating_sub(5));
            }
        }
        // Arrows when the list goes on above or below.
        if self.surv.csec_scroll > 0 {
            arrow(&mut self.ui, x + w - 9.0 * s, cy - 6.0 * s, s, true);
        }
        if self.surv.csec_scroll + room < secs.len() {
            arrow(&mut self.ui, x + w - 9.0 * s, y + h - 6.0 * s, s, false);
        }
        over
    }

    fn draw_tab_icon(&mut self, tab: Tab, x: f32, y: f32) {
        let s = self.ui.scale;
        if tab == Tab::Search {
            // A magnifying glass.
            let c = [60, 60, 60, 255];
            self.ui.frame(x + 2.0 * s, y + 2.0 * s, 9.0 * s, 9.0 * s, 2.0 * s, c);
            self.ui.rect(x + 4.0 * s, y + 4.0 * s, 5.0 * s, 5.0 * s, [170, 210, 230, 255]);
            for k in 0..4 {
                self.ui.rect(x + (10.0 + k as f32) * s, y + (10.0 + k as f32) * s, 3.0 * s, 2.0 * s, [110, 80, 50, 255]);
            }
            return;
        }
        if let Some(id) = self.items.id(tab.icon()) {
            item_icon(&mut self.ui, &self.reg, &self.items, self.item_base, &ItemStack::new(id, 1), x, y, 16.0 * s);
        }
    }

    fn tooltip(&mut self, text: &str, mouse: (f32, f32)) {
        let s = self.ui.scale;
        let tw = self.ui.text_width(text, s);
        let tx = (mouse.0 + 8.0 * s).min(self.ui.width - tw - 8.0 * s);
        self.ui.rect(tx, mouse.1 - 14.0 * s, tw + 6.0 * s, 12.0 * s, [16, 0, 16, 235]);
        self.ui.text(tx + 3.0 * s, mouse.1 - 12.0 * s, text, WHITE);
    }
}

/// The rows of the grid: by section (each from a new row), A to Z, or search results.
fn build_rows(game: &Game, cat: &Catalog, tab: Tab) -> Vec<Row> {
    let filter = game.settings.creative_origin;
    let passes = |id: ItemId| cat.entry(id).is_some_and(|e| crate::catalog::origin_passes(filter, e.origin));
    let flat = |ids: Vec<ItemId>| -> Vec<Row> {
        ids.chunks(9)
            .map(|c| {
                let mut row = Row::default();
                for (k, &id) in c.iter().enumerate() {
                    row.items[k] = Some(id);
                }
                row
            })
            .collect()
    };
    let by_name = |ids: &mut Vec<ItemId>| ids.sort_by(|a, b| cat.entry(*a).map(|e| &e.key).cmp(&cat.entry(*b).map(|e| &e.key)));
    match tab {
        Tab::Search => {
            let q = Query::parse(&game.surv.csearch);
            let mut hits: Vec<(u8, usize, u64, ItemId)> = (1..game.items.items.len() as ItemId)
                .filter(|&i| passes(i))
                .filter_map(|i| cat.matches(i, &q).map(|r| (r, cat.rank(i).0, cat.entry(i).map(|e| e.order).unwrap_or(0), i)))
                .collect();
            hits.sort();
            let mut ids: Vec<ItemId> = hits.into_iter().map(|h| h.3).collect();
            if game.settings.creative_sort != 0 {
                by_name(&mut ids);
            }
            flat(ids)
        }
        Tab::Favorites => {
            let mut ids: Vec<ItemId> = game.settings.creative_favorites.iter().filter_map(|n| game.items.id(n)).filter(|&i| passes(i)).collect();
            if game.settings.creative_sort != 0 {
                by_name(&mut ids);
            }
            flat(ids)
        }
        t => {
            let mut ids: Vec<ItemId> = cat.list(t).iter().copied().filter(|&i| passes(i)).collect();
            if game.settings.creative_sort != 0 {
                by_name(&mut ids);
                return flat(ids);
            }
            // Each section starts a row of its own.
            let mut rows: Vec<Row> = Vec::new();
            let mut cur: Option<u16> = None;
            let mut k = 9;
            for id in ids {
                let sec = cat.entry(id).map(|e| e.sec).unwrap_or(0);
                if cur != Some(sec) {
                    cur = Some(sec);
                    rows.push(Row { head: Some(sec), ..Default::default() });
                    k = 0;
                } else if k == 9 {
                    rows.push(Row::default());
                    k = 0;
                }
                rows.last_mut().unwrap().items[k] = Some(id);
                k += 1;
            }
            rows
        }
    }
}

/// `text` cut with "..." so it fits in `room`.
fn fit_text(ui: &crate::ui::UiBatch, text: &str, room: f32, s: f32) -> String {
    if ui.text_width(text, s) <= room {
        return text.to_string();
    }
    let mut t: String = text.to_string();
    while !t.is_empty() && ui.text_width(&format!("{t}..."), s) > room {
        t.pop();
    }
    format!("{}...", t.trim_end())
}

/// A small arrow (up or down) saying a list goes on.
fn arrow(ui: &mut crate::ui::UiBatch, x: f32, y: f32, s: f32, up: bool) {
    for k in 0..3 {
        let row = if up { k } else { 2 - k } as f32;
        let half = k as f32;
        ui.rect(x + (2.0 - half) * s, y + row * s, (1.0 + 2.0 * half) * s, s, [64, 64, 64, 255]);
    }
}

/// A little five-point star (favourites), 4×4 interface pixels.
fn draw_star(ui: &mut crate::ui::UiBatch, x: f32, y: f32, s: f32, c: [u8; 4]) {
    let dark = [90, 60, 0, 255];
    ui.rect(x + 1.0 * s - s * 0.5, y - s * 0.5, 3.0 * s, 5.0 * s, dark);
    ui.rect(x - s * 0.5, y + 1.0 * s - s * 0.5, 5.0 * s, 2.0 * s, dark);
    ui.rect(x + 1.5 * s, y, s, 4.0 * s, c);
    ui.rect(x, y + 1.0 * s, 4.0 * s, s, c);
    ui.rect(x + 0.5 * s, y + 2.0 * s, 3.0 * s, s, c);
}

/// An empty slot with the classic sunken bevel.
fn draw_slot(game: &mut Game, x: f32, y: f32) {
    if game.surv.cart {
        return;
    }
    let s = game.ui.scale;
    game.ui.rect(x, y, 18.0 * s, 18.0 * s, [55, 55, 55, 255]);
    game.ui.rect(x + s, y + s, 17.0 * s, 17.0 * s, [255, 255, 255, 255]);
    game.ui.rect(x + s, y + s, 16.0 * s, 16.0 * s, [139, 139, 139, 255]);
}

/// Faint outline of what goes in an empty armour slot: helmet, chestplate, leggings, boots,
/// then the shield of the second hand.
fn slot_outline(game: &mut Game, kind: usize, x: f32, y: f32) {
    let s = game.ui.scale;
    let art = ["helmet", "chestplate", "leggings", "boots", "shield"][kind.min(4)];
    if game.ui.sprite(&format!("sprites/container/slot/{art}"), x + s, y + s, 16.0 * s, 16.0 * s) {
        return;
    }
    let c = [110, 110, 110, 255];
    let mut r = |px: f32, py: f32, w: f32, h: f32| game.ui.rect(x + px * s, y + py * s, w * s, h * s, c);
    match kind {
        0 => {
            r(4.0, 4.0, 10.0, 1.0);
            r(3.0, 5.0, 1.0, 8.0);
            r(14.0, 5.0, 1.0, 8.0);
            r(4.0, 13.0, 3.0, 1.0);
            r(11.0, 13.0, 3.0, 1.0);
        }
        1 => {
            r(3.0, 3.0, 4.0, 1.0);
            r(11.0, 3.0, 4.0, 1.0);
            r(7.0, 4.0, 4.0, 1.0);
            r(3.0, 4.0, 1.0, 11.0);
            r(14.0, 4.0, 1.0, 11.0);
            r(4.0, 15.0, 10.0, 1.0);
        }
        2 => {
            r(4.0, 3.0, 10.0, 1.0);
            r(4.0, 4.0, 1.0, 11.0);
            r(13.0, 4.0, 1.0, 11.0);
            r(8.0, 6.0, 2.0, 1.0);
            r(8.0, 6.0, 1.0, 9.0);
            r(9.0, 6.0, 1.0, 9.0);
        }
        3 => {
            r(4.0, 6.0, 1.0, 8.0);
            r(7.0, 6.0, 1.0, 6.0);
            r(4.0, 14.0, 4.0, 1.0);
            r(10.0, 6.0, 1.0, 6.0);
            r(13.0, 6.0, 1.0, 8.0);
            r(10.0, 14.0, 4.0, 1.0);
        }
        _ => {
            r(4.0, 3.0, 10.0, 1.0);
            r(4.0, 3.0, 1.0, 8.0);
            r(13.0, 3.0, 1.0, 8.0);
            r(5.0, 11.0, 2.0, 2.0);
            r(11.0, 11.0, 2.0, 2.0);
            r(7.0, 13.0, 4.0, 1.0);
        }
    }
}

/// A pack's 256×256 container picture over the panel, leaving out the `hole` rectangle.
fn sprite_around(ui: &mut crate::ui::UiBatch, name: &str, panel: (f32, f32, f32, f32), hole: Option<(f32, f32, f32, f32)>, s: f32) {
    let (x0, y0, pw, ph) = panel;
    // Screen rectangle -> the same part of the picture (one panel pixel is 1/256 of it).
    let mut part = |x: f32, y: f32, w: f32, h: f32| {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let f = |v: f32| v / s / 256.0;
        let p = [f(x - x0), f(y - y0), f(x - x0 + w), f(y - y0 + h)];
        ui.sprite_part(name, p, x, y, w, h, crate::ui::WHITE);
    };
    match hole {
        None => part(x0, y0, pw, ph),
        Some((hx, hy, hw, hh)) => {
            part(x0, y0, pw, hy - y0);
            part(x0, hy + hh, pw, y0 + ph - hy - hh);
            part(x0, hy, hx - x0, hh);
            part(hx + hw, hy, x0 + pw - hx - hw, hh);
        }
    }
}
