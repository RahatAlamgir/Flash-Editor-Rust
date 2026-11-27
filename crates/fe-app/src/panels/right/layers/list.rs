//! The layer list: top layer first. Click selects, Ctrl+click adds / removes, Shift+click picks a range,
//! double-click renames, right-click opens the layer menu, and dragging moves layers (a line shows where they
//! will land; several selected layers move together).

use egui::{Modifiers, Stroke, Ui};
use fe_core::document::LayerId;
use fe_core::document::layer_order::{RowSpan, gap_at};
use fe_ui_kit::theme::colors;

use super::menu::{self, MenuItem};
use super::row::{self, RowLook};
use crate::app::FlashApp;
use crate::commands::layers as cmd;

enum Action {
    Click(LayerId, Modifiers),
    StartRename(LayerId),
    Rename(LayerId, String),
    CancelRename,
    ToggleVisible(LayerId, bool),
    ToggleLock(LayerId, bool),
    ToggleMask(LayerId),
    ToggleExpand(LayerId),
    PickPart(LayerId, usize),
    DragStart(LayerId),
    Menu(LayerId, MenuItem),
}

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let settled = !ui.input(|i| i.pointer.any_down());
    let ctx = ui.ctx().clone();
    let mut actions = Vec::new();
    let mut spans: Vec<RowSpan> = Vec::new();
    let list_rect = ui.max_rect();
    egui::ScrollArea::vertical().id_salt("layers-list").auto_shrink(false).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let doc = &app.projects[app.active_project].active_page().doc;
        let can_merge_down = doc.active_index().is_some_and(|i| i > 0);
        let selected_count = doc.selected_ids().len();
        for (index, layer) in doc.layers().iter().rev().enumerate() {
            let id = layer.id();
            let thumbnail = app.thumbnails.layer(&ctx, doc, layer, [84, 56], settled);
            let renaming = match &mut app.layers_ui.renaming {
                Some((rid, text)) if *rid == id => Some(text),
                _ => None,
            };
            let is_renaming = renaming.is_some();
            let mask = app.thumbnails.mask(&ctx, doc, layer, [60, 56], settled);
            let is_group = layer.object().is_some_and(|o| o.kind() == "group");
            let open = is_group && app.layers_ui.expanded.contains(&id);
            let group_open = is_group.then_some(open);
            let events =
                row::show(ui, RowLook { layer, selected: doc.is_selected(id), thumbnail, mask, renaming, group_open });
            let Some(response) = events.row else { continue };
            spans.push(RowSpan { index, top: response.rect.top() as f64, bottom: response.rect.bottom() as f64 });
            if events.toggle_visible {
                actions.push(Action::ToggleVisible(id, !layer.visible));
            } else if events.toggle_lock {
                actions.push(Action::ToggleLock(id, !layer.locked));
            } else if events.toggle_mask {
                actions.push(Action::ToggleMask(id));
            } else if events.toggle_expand {
                actions.push(Action::ToggleExpand(id));
            } else if response.double_clicked() && !is_renaming {
                actions.push(Action::StartRename(id));
            } else if response.clicked() || response.secondary_clicked() && !doc.is_selected(id) {
                actions.push(Action::Click(id, ui.input(|i| i.modifiers)));
            } else if response.drag_started_by(egui::PointerButton::Primary) && !is_renaming {
                actions.push(Action::DragStart(id));
            }
            match events.rename_done {
                Some(true) => {
                    if let Some((_, text)) = &app.layers_ui.renaming {
                        actions.push(Action::Rename(id, text.clone()));
                    }
                }
                Some(false) => actions.push(Action::CancelRename),
                None => {}
            }
            let info = menu::MenuInfo {
                visible: layer.visible,
                locked: layer.locked,
                selected_count: if doc.is_selected(id) { selected_count } else { 1 },
                can_merge_down,
                has_mask: layer.mask().is_some(),
                can_clip: !layer.is_adjustment()
                    && doc.index_of(id).is_some_and(|i| i > 0 && !doc.layers()[i - 1].is_adjustment()),
            };
            response.context_menu(|ui| {
                if let Some(item) = menu::show(ui, &info) {
                    actions.push(Action::Menu(id, item));
                }
            });
            if open && let Some(parts) = layer.object().and_then(|o| o.group_members()) {
                let active_part = (doc.active_id() == Some(id))
                    .then(|| crate::commands::parts::active_part_of(&app.tools, doc))
                    .flatten();
                for (i, part) in parts.iter().enumerate().rev() {
                    if super::part_row::show(ui, &part.name, active_part == Some(i)) {
                        actions.push(Action::PickPart(id, i));
                    }
                }
            }
        }
    });
    let layer_count = app.doc().layers().len();
    drag_feedback(ui, app, &spans, layer_count, list_rect);
    for action in actions {
        apply(app, action);
    }
}

/// While layers are dragged: work out the gap under the pointer, draw the drop line, and drop on release.
fn drag_feedback(ui: &Ui, app: &mut FlashApp, spans: &[RowSpan], count: usize, list: egui::Rect) {
    let Some((moved, _)) = app.layers_ui.drag.clone() else { return };
    let pointer = ui.input(|i| i.pointer.interact_pos());
    let gap = pointer.map_or(0, |p| gap_at(spans, p.y as f64, count));
    if let Some(span) = spans.iter().find(|s| s.index == gap).or(spans.last()) {
        let y = if spans.iter().any(|s| s.index == gap) { span.top } else { span.bottom } as f32;
        ui.painter().with_clip_rect(list).hline(list.left()..=list.right(), y, Stroke::new(2.0, colors::ACCENT));
    }
    app.layers_ui.drag = Some((moved.clone(), gap));
    if !ui.input(|i| i.pointer.primary_down()) {
        app.layers_ui.drag = None;
        cmd::move_to_gap(app, moved, gap);
    }
}

fn apply(app: &mut FlashApp, action: Action) {
    match action {
        Action::Click(id, mods) => click(app, id, mods),
        Action::StartRename(id) => {
            let name = app.doc().layer(id).map(|l| l.name.clone()).unwrap_or_default();
            app.layers_ui.renaming = Some((id, name));
        }
        Action::Rename(id, name) => {
            app.layers_ui.renaming = None;
            cmd::rename(app, id, name);
        }
        Action::CancelRename => app.layers_ui.renaming = None,
        Action::ToggleVisible(id, visible) => cmd::set_visible(app, id, visible),
        Action::ToggleLock(id, locked) => cmd::set_locked(app, id, locked),
        Action::ToggleExpand(id) => {
            if !app.layers_ui.expanded.remove(&id) {
                app.layers_ui.expanded.insert(id);
            }
        }
        Action::PickPart(id, index) => {
            app.project_mut().active_page_mut().doc.set_active(Some(id));
            crate::commands::parts::select_part(app, Some((id, index)));
            app.tools.activate(crate::tools::ToolKind::Move);
        }
        Action::ToggleMask(id) => {
            app.project_mut().active_page_mut().doc.set_active(Some(id));
            crate::commands::mask::toggle(app);
        }
        Action::DragStart(id) => {
            let doc = app.doc();
            let moved = if doc.is_selected(id) { doc.selected_ids() } else { vec![id] };
            app.layers_ui.drag = Some((moved, 0));
        }
        Action::Menu(id, item) => {
            if !app.doc().is_selected(id) {
                app.project_mut().active_page_mut().doc.set_active(Some(id));
            }
            menu::run(app, id, item);
        }
    }
}

/// Selection rules of the list (C# `LayersPanel`).
fn click(app: &mut FlashApp, id: LayerId, mods: Modifiers) {
    let anchor = app.layers_ui.anchor;
    let doc = &mut app.project_mut().active_page_mut().doc;
    if mods.command {
        let mut selected = doc.selected_ids();
        if selected.contains(&id) && selected.len() > 1 {
            selected.retain(|s| *s != id);
            let active = doc.active_id().filter(|a| *a != id).or(selected.last().copied());
            doc.set_selection(&selected, active);
        } else if !selected.contains(&id) {
            selected.push(id);
            doc.set_selection(&selected, Some(id));
        }
        app.layers_ui.anchor = Some(id);
    } else if mods.shift
        && let (Some(a), Some(b)) = (anchor.and_then(|a| doc.index_of(a)), doc.index_of(id))
    {
        let range: Vec<LayerId> = doc.layers()[a.min(b)..=a.max(b)].iter().map(|l| l.id()).collect();
        doc.set_selection(&range, Some(id));
    } else {
        doc.set_active(Some(id));
        app.layers_ui.anchor = Some(id);
    }
}
