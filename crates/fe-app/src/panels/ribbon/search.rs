//! The "Search commands" box (C# `RibbonBar` search, "tell me what you want to do"): type a few words and the buttons
//! of the ribbon and the items of the File drawer that match are listed under the box. Click one (or Enter for the first,
//! the arrow keys choose) and it runs.

use egui::{Align2, Color32, FontId, Id, Key, Order, Rect, Sense, TextEdit, Ui, UiBuilder, Vec2, pos2, vec2};
use fe_ui_kit::icons::glyphs;
use fe_ui_kit::theme::{colors, metrics};

use super::model::TABS;
use super::{RibbonAction, RibbonState};

/// A command the search can find.
pub struct Hit {
    pub title: String,
    /// The tab (or "File") it is on.
    pub place: &'static str,
    pub glyph: char,
    pub action: RibbonAction,
}

const MAX: usize = 9;

/// The commands that match what was typed: every word has to appear in the name, the tab or the group, best first.
pub fn find(query: &str, anchor: Rect) -> Vec<Hit> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    if words.is_empty() {
        return Vec::new();
    }
    let mut hits: Vec<(Hit, String)> = Vec::new();
    for (tab, t) in TABS.iter().enumerate() {
        for (group, g) in t.groups.iter().enumerate() {
            for (item, i) in g.items.iter().enumerate() {
                let title = i.label.trim_end_matches(" \u{25BE}").to_owned();
                let hay = format!("{title} {} {}", t.name, g.name).to_lowercase();
                hits.push((
                    Hit {
                        title,
                        place: t.name,
                        glyph: i.glyph,
                        action: RibbonAction::Button { tab, group, item, rect: anchor },
                    },
                    hay,
                ));
            }
        }
    }
    for (index, (label, glyph, tip, heading)) in crate::panels::file_drawer::flat_items().into_iter().enumerate() {
        let hay = format!("{label} file {heading} {tip}").to_lowercase();
        hits.push((
            Hit { title: label.to_owned(), place: "File", glyph, action: RibbonAction::DrawerItem(index) },
            hay,
        ));
    }
    let first = &words[0];
    let mut found: Vec<(Hit, String)> =
        hits.into_iter().filter(|(_, hay)| words.iter().all(|w| hay.contains(w.as_str()))).collect();
    found.sort_by_key(|(h, _)| {
        let t = h.title.to_lowercase();
        (
            if t.starts_with(first.as_str()) {
                0
            } else if t.contains(first.as_str()) {
                1
            } else {
                2
            },
            t.len(),
        )
    });
    let mut seen = std::collections::HashSet::new();
    found.into_iter().map(|(h, _)| h).filter(|h| seen.insert((h.title.clone(), h.place))).take(MAX).collect()
}

/// Draws the box and, under it, the results. Returns what was chosen.
pub fn show(ui: &mut Ui, row: Rect, state: &mut RibbonState) -> Option<RibbonAction> {
    let rect = Rect::from_min_size(
        pos2(row.right() - 40.0 - metrics::SEARCH_W, row.top() + 3.0),
        vec2(metrics::SEARCH_W, 24.0),
    );
    let mut child = ui.new_child(UiBuilder::new().max_rect(rect));
    let id = Id::NULL.with("ribbon-search");
    let edit = TextEdit::singleline(&mut state.search)
        .id(id)
        .hint_text(egui::RichText::new("Search commands").color(colors::SUBTLE_TEXT))
        .font(FontId::proportional(12.0))
        .margin(egui::Margin { left: 26, right: 6, top: 4, bottom: 4 })
        .desired_width(metrics::SEARCH_W);
    let response = child
        .add_sized(Vec2::new(metrics::SEARCH_W, 24.0), edit)
        .on_hover_text("Type what you want to do, for example rotate, blur or export");
    ui.painter().text(
        pos2(rect.left() + 9.0, rect.center().y),
        Align2::LEFT_CENTER,
        glyphs::SEARCH,
        glyphs::font(11.0),
        colors::SUBTLE_TEXT,
    );
    if response.changed() {
        state.search_open = true;
        state.search_pick = 0;
    }
    if response.gained_focus() && !state.search.is_empty() {
        state.search_open = true;
    }
    if state.search.is_empty() || !state.search_open {
        return None;
    }
    let hits = find(&state.search, rect);
    if hits.is_empty() {
        return None;
    }
    state.search_pick = state.search_pick.min(hits.len() - 1);
    let mut chosen = None;
    if response.has_focus() {
        ui.input_mut(|i| {
            if i.consume_key(egui::Modifiers::NONE, Key::ArrowDown) {
                state.search_pick = (state.search_pick + 1).min(hits.len() - 1);
            }
            if i.consume_key(egui::Modifiers::NONE, Key::ArrowUp) {
                state.search_pick = state.search_pick.saturating_sub(1);
            }
            if i.consume_key(egui::Modifiers::NONE, Key::Escape) {
                state.search.clear();
                state.search_open = false;
            }
        });
        if ui.input(|i| i.key_pressed(Key::Enter)) {
            chosen = Some(state.search_pick);
        }
    }
    let width = metrics::SEARCH_W + 120.0;
    let area = egui::Area::new(Id::NULL.with("ribbon-search-results"))
        .order(Order::Foreground)
        .fixed_pos(pos2(rect.right() - width, rect.bottom() + 2.0))
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style()).fill(colors::PANEL).show(ui, |ui| {
                ui.set_width(width);
                for (i, hit) in hits.iter().enumerate() {
                    let (r, resp) = ui.allocate_exact_size(vec2(width, 26.0), Sense::click());
                    if resp.hovered() {
                        state.search_pick = i;
                    }
                    if i == state.search_pick {
                        ui.painter().rect_filled(r, 3.0, colors::ACCENT_SOFT);
                    }
                    ui.painter().text(
                        pos2(r.left() + 14.0, r.center().y),
                        Align2::CENTER_CENTER,
                        hit.glyph,
                        glyphs::font(14.0),
                        colors::ICON,
                    );
                    ui.painter().text(
                        pos2(r.left() + 32.0, r.center().y),
                        Align2::LEFT_CENTER,
                        &hit.title,
                        FontId::proportional(12.5),
                        colors::TEXT,
                    );
                    ui.painter().text(
                        pos2(r.right() - 8.0, r.center().y),
                        Align2::RIGHT_CENTER,
                        hit.place,
                        FontId::proportional(11.0),
                        colors::SUBTLE_TEXT,
                    );
                    if resp.clicked() {
                        chosen = Some(i);
                    }
                }
            });
        });
    // A click anywhere else closes the results.
    if ui.input(|i| i.pointer.any_pressed()) && !response.hovered() && !area.response.hovered() {
        state.search_open = false;
    }
    let pick = chosen?;
    let action = hits.into_iter().nth(pick)?.action;
    state.search.clear();
    state.search_open = false;
    ui.memory_mut(|m| m.surrender_focus(id));
    let _ = Color32::WHITE;
    Some(action)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles(query: &str) -> Vec<String> {
        find(query, Rect::NOTHING).into_iter().map(|h| h.title).collect()
    }

    #[test]
    fn a_few_words_find_the_commands_that_have_them_best_first() {
        let blur = titles("blur");
        assert!(blur.first().is_some_and(|t| t.contains("Blur")), "{blur:?}");
        assert!(titles("rotate").iter().any(|t| t == "Rotate"));
        assert!(titles("export pdf").iter().any(|t| t.contains("PDF")));
        assert!(titles("zzzzqq").is_empty());
        assert!(titles("").is_empty());
        assert!(titles("e").len() <= MAX);
    }

    #[test]
    fn file_drawer_commands_are_found_too() {
        let hits = find("batch", Rect::NOTHING);
        assert!(hits.iter().any(|h| h.place == "File" && matches!(h.action, RibbonAction::DrawerItem(_))));
    }
}
