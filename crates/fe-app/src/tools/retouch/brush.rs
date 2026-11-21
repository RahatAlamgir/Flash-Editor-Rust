//! Clone Stamp, Healing Brush, Spot Healing, Dodge / Burn and Blur / Sharpen: one stroke per mouse drag with an
//! operation deciding what happens to the pixels under the brush. The whole drag is one undo step.

use fe_core::document::{Document, LayerId};
use fe_core::imaging::{ColorRgba, PointD};
use fe_core::retouch::{RetouchOp, RetouchStroke, find_spot_source};

use super::RetouchOptions;
use crate::app::FlashApp;
use crate::commands::checks;
use crate::tools::{Pointer, ToolKind};

pub struct Active {
    stroke: RetouchStroke,
    layer: LayerId,
    before: Document,
    name: &'static str,
    spot: bool,
}

fn name_of(tool: ToolKind) -> &'static str {
    match tool {
        ToolKind::CloneStamp => "Clone Stamp",
        ToolKind::HealingBrush => "Healing Brush",
        ToolKind::SpotHealing => "Spot Healing",
        ToolKind::DodgeBurn => "Dodge / Burn",
        _ => "Blur / Sharpen",
    }
}

/// The operation and the stroke strength (0..1) of a stroke starting at `p`; `None` when it cannot start.
fn begin_op(app: &mut FlashApp, tool: ToolKind, p: &Pointer) -> Option<(RetouchOp, f64)> {
    let o: RetouchOptions = app.tools.retouch.options_mut(tool)?.clone();
    let strength = o.strength / 100.0;
    Some(match tool {
        ToolKind::CloneStamp | ToolKind::HealingBrush => {
            let Some(source) = app.tools.retouch.source else {
                app.status = "Alt+click first, on the place you want to copy from.".into();
                return None;
            };
            let t = &mut app.tools.retouch;
            if t.offset.is_none() || !o.aligned {
                t.offset = Some((
                    (source.x - p.pos.x).round_ties_even() as i32,
                    (source.y - p.pos.y).round_ties_even() as i32,
                ));
            }
            let (dx, dy) = t.offset?;
            let op = if tool == ToolKind::CloneStamp {
                RetouchOp::Clone { dx, dy }
            } else {
                RetouchOp::Heal { dx, dy, radius: RetouchOp::heal_radius_for_brush(o.size) }
            };
            (op, strength)
        }
        ToolKind::SpotHealing => (RetouchOp::Highlight { color: ColorRgba::rgb(255, 60, 60), amount: 0.45 }, 1.0),
        ToolKind::DodgeBurn => (RetouchOp::Tone { burn: o.burn != p.mods.alt, range: o.range }, strength),
        _ => {
            if o.sharpen != p.mods.alt {
                (RetouchOp::Sharpen { radius: 2, amount: 0.3 + 1.7 * strength }, 1.0)
            } else {
                (RetouchOp::Blur { radius: (1.0 + 7.0 * strength).round_ties_even() as i32 }, 0.35 + 0.65 * strength)
            }
        }
    })
}

fn add(app: &mut FlashApp, a: &mut Active, at: PointD) {
    if let Some(layer) = app.project_mut().active_page_mut().doc.layer_mut(a.layer) {
        a.stroke.add_point(layer.pixels_mut(), at);
    }
}

/// Moves the cross that shows the place being copied.
fn show_cross(app: &mut FlashApp, tool: ToolKind, at: PointD) {
    if matches!(tool, ToolKind::CloneStamp | ToolKind::HealingBrush) {
        let t = &mut app.tools.retouch;
        t.cross = Some(match t.offset {
            Some((dx, dy)) => PointD::new(at.x + f64::from(dx), at.y + f64::from(dy)),
            None => at,
        });
    }
}

pub fn pointer(app: &mut FlashApp, tool: ToolKind, p: &Pointer) {
    if p.pressed {
        if p.mods.alt && matches!(tool, ToolKind::CloneStamp | ToolKind::HealingBrush) {
            let t = &mut app.tools.retouch;
            t.source = Some(p.pos);
            t.offset = None;
            t.cross = Some(p.pos);
            app.status = "Copy-from point chosen. Now paint where it should be copied to.".into();
            return;
        }
        let Some(id) = checks::editable_layer(app) else { return };
        let Some((op, opacity)) = begin_op(app, tool, p) else { return };
        let Some(o) = app.tools.retouch.options_mut(tool).map(|o| o.settings(opacity)) else { return };
        let canvas = app.doc().bounds();
        let selection = app.doc().selected_area().cloned();
        let Some(layer) = app.doc().layer(id) else { return };
        let stroke = RetouchStroke::new(layer.pixels(), o, op, selection, canvas);
        let mut active = Active {
            stroke,
            layer: id,
            before: app.doc().clone(),
            name: name_of(tool),
            spot: tool == ToolKind::SpotHealing,
        };
        add(app, &mut active, p.pos);
        show_cross(app, tool, p.pos);
        app.tools.retouch.stroke = Some(active);
    }
    let Some(mut a) = app.tools.retouch.stroke.take() else { return };
    if !p.pressed {
        add(app, &mut a, p.pos);
        show_cross(app, tool, p.pos);
    }
    if p.released || !p.down {
        finish(app, tool, a);
    } else {
        app.tools.retouch.stroke = Some(a);
    }
}

fn finish(app: &mut FlashApp, tool: ToolKind, mut a: Active) {
    let canvas = app.doc().bounds();
    if a.spot {
        // The painted area was only tinted; now it is healed from a clean place the program picks.
        let found = find_spot_source(a.stroke.original(), a.stroke.total(), canvas);
        let Some((dx, dy)) = found else {
            app.status =
                "No clean area nearby to heal from. Try a smaller brush or a spot nearer to the middle of the picture."
                    .into();
            app.project_mut().active_page_mut().doc = a.before;
            return;
        };
        let size = app.tools.retouch.spot.size;
        let op = RetouchOp::Heal { dx, dy, radius: RetouchOp::heal_radius_for_brush(size) };
        if let Some(layer) = app.project_mut().active_page_mut().doc.layer_mut(a.layer) {
            a.stroke.reapply(layer.pixels_mut(), op);
        }
    }
    if matches!(tool, ToolKind::CloneStamp | ToolKind::HealingBrush)
        && !app.tools.retouch.clone.aligned
        && !app.tools.retouch.heal.aligned
    {
        app.tools.retouch.cross = app.tools.retouch.source;
    } else if !matches!(tool, ToolKind::CloneStamp | ToolKind::HealingBrush) {
        app.tools.retouch.cross = None;
    }
    app.project_mut().record(a.name, a.before);
}
