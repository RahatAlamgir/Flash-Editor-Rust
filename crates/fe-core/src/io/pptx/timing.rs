//! The animations of a slide as PowerPoint's `<p:timing>`: one "click" step after another, inside each step the effects that
//! start together or after each other. Each effect is written with PowerPoint's own behaviors (visibility, fade, wipe, fly,
//! zoom, spin, scale), so the slide plays in PowerPoint.

use std::fmt::Write;

use crate::document::TransitionDirection;
use crate::document::animation::{Animation, Effect, EffectClass, Start, Timeline, timeline};

/// Writes `<p:timing>` for the animations; `spids(layer)` gives the shape ids of a layer on the slide. Empty when there is
/// nothing to animate.
pub fn timing_xml(animations: &[Animation], spids: &dyn Fn(crate::document::LayerId) -> Vec<u32>) -> String {
    let timeline = timeline(animations);
    if animations.iter().all(|a| spids(a.layer).is_empty()) {
        return String::new();
    }
    let mut ids = Ids(2);
    let mut steps = String::new();
    for step in 0..=timeline.clicks {
        let body = step_xml(animations, &timeline, step, spids, &mut ids);
        if body.is_empty() {
            continue;
        }
        let id = ids.next();
        let start = if step == 0 {
            "<p:cond delay=\"indefinite\"/><p:cond evt=\"onBegin\" delay=\"0\"><p:tn val=\"2\"/></p:cond>"
        } else {
            "<p:cond delay=\"indefinite\"/>"
        };
        let _ = write!(
            steps,
            "<p:par><p:cTn id=\"{id}\" fill=\"hold\"><p:stCondLst>{start}</p:stCondLst><p:childTnLst>{body}</p:childTnLst></p:cTn></p:par>"
        );
    }
    if steps.is_empty() {
        return String::new();
    }
    format!(
        "<p:timing><p:tnLst><p:par><p:cTn id=\"1\" dur=\"indefinite\" restart=\"never\" nodeType=\"tmRoot\"><p:childTnLst><p:seq concurrent=\"1\" nextAc=\"seek\"><p:cTn id=\"2\" dur=\"indefinite\" nodeType=\"mainSeq\"><p:childTnLst>{steps}</p:childTnLst></p:cTn><p:prevCondLst><p:cond evt=\"onPrev\" delay=\"0\"><p:tgtEl><p:sldTgt/></p:tgtEl></p:cond></p:prevCondLst><p:nextCondLst><p:cond evt=\"onNext\" delay=\"0\"><p:tgtEl><p:sldTgt/></p:tgtEl></p:cond></p:nextCondLst></p:seq></p:childTnLst></p:cTn></p:par></p:tnLst></p:timing>"
    )
}

struct Ids(u32);

impl Ids {
    fn next(&mut self) -> u32 {
        self.0 += 1;
        self.0
    }
}

fn ms(seconds: f32) -> i64 {
    (seconds * 1000.0).round() as i64
}

/// The effects of one click step, grouped by the time they begin.
fn step_xml(
    animations: &[Animation],
    timeline: &Timeline,
    step: usize,
    spids: &dyn Fn(crate::document::LayerId) -> Vec<u32>,
    ids: &mut Ids,
) -> String {
    let mut begins: Vec<f32> = Vec::new();
    for t in timeline.items.iter().filter(|t| t.step == step) {
        if !begins.contains(&t.begin) {
            begins.push(t.begin);
        }
    }
    let mut out = String::new();
    for begin in begins {
        let mut effects = String::new();
        for (n, t) in timeline.items.iter().filter(|t| t.step == step && t.begin == begin).enumerate() {
            let a = &animations[t.index];
            let shapes = spids(a.layer);
            if shapes.is_empty() {
                continue;
            }
            let node = match (a.start, n, step) {
                (Start::OnClick, 0, s) if s > 0 => "clickEffect",
                (Start::AfterPrevious, ..) => "afterEffect",
                _ => "withEffect",
            };
            effects.push_str(&effect_xml(a, node, &shapes, ids));
        }
        if effects.is_empty() {
            continue;
        }
        let id = ids.next();
        let _ = write!(
            out,
            "<p:par><p:cTn id=\"{id}\" fill=\"hold\"><p:stCondLst><p:cond delay=\"{}\"/></p:stCondLst><p:childTnLst>{effects}</p:childTnLst></p:cTn></p:par>",
            ms(begin)
        );
    }
    out
}

/// PowerPoint's effect number and the direction number of an effect.
fn preset(a: &Animation) -> (u32, u32) {
    let side = match a.direction {
        TransitionDirection::Left => 8,
        TransitionDirection::Right => 2,
        TransitionDirection::Up => 1,
        TransitionDirection::Down => 4,
    };
    match a.effect {
        Effect::Appear | Effect::Disappear => (1, 0),
        Effect::FadeIn | Effect::FadeOut => (10, 0),
        Effect::FlyIn | Effect::FlyOut => (2, side),
        Effect::FloatIn | Effect::FloatOut => (42, 0),
        Effect::ZoomIn | Effect::ZoomOut => (53, 16),
        Effect::WipeIn | Effect::WipeOut => (22, side),
        Effect::GrowTurn => (31, 0),
        Effect::Pulse => (26, 0),
        Effect::Spin => (8, 0),
        Effect::GrowShrink => (6, 0),
        Effect::Teeter => (32, 0),
    }
}

fn target(spid: u32) -> String {
    format!("<p:tgtEl><p:spTgt spid=\"{spid}\"/></p:tgtEl>")
}

fn set_visibility(ids: &mut Ids, spid: u32, shown: bool, delay_ms: i64) -> String {
    let id = ids.next();
    format!(
        "<p:set><p:cBhvr><p:cTn id=\"{id}\" dur=\"1\" fill=\"hold\"><p:stCondLst><p:cond delay=\"{delay_ms}\"/></p:stCondLst></p:cTn>{}<p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val=\"{}\"/></p:to></p:set>",
        target(spid),
        if shown { "visible" } else { "hidden" }
    )
}

fn fade(ids: &mut Ids, spid: u32, duration: i64, entering: bool, filter: &str) -> String {
    let id = ids.next();
    format!(
        "<p:animEffect transition=\"{}\" filter=\"{filter}\"><p:cBhvr><p:cTn id=\"{id}\" dur=\"{duration}\"/>{}</p:cBhvr></p:animEffect>",
        if entering { "in" } else { "out" },
        target(spid)
    )
}

/// Moves one value (`ppt_x`, `ppt_y`, `ppt_w`, `ppt_h`) from one formula to another.
fn slide_value(ids: &mut Ids, spid: u32, duration: i64, name: &str, from: &str, to: &str) -> String {
    let id = ids.next();
    format!(
        "<p:anim calcmode=\"lin\" valueType=\"num\"><p:cBhvr additive=\"base\"><p:cTn id=\"{id}\" dur=\"{duration}\" fill=\"hold\"/>{}<p:attrNameLst><p:attrName>{name}</p:attrName></p:attrNameLst></p:cBhvr><p:tavLst><p:tav tm=\"0\"><p:val><p:strVal val=\"{from}\"/></p:val></p:tav><p:tav tm=\"100000\"><p:val><p:strVal val=\"{to}\"/></p:val></p:tav></p:tavLst></p:anim>",
        target(spid)
    )
}

/// The edge-to-place movement of a fly effect: where the shape starts (entering) or ends (leaving).
fn fly(ids: &mut Ids, spid: u32, duration: i64, direction: TransitionDirection, entering: bool) -> String {
    let (x_edge, y_edge) = match direction {
        TransitionDirection::Left => (Some("0-#ppt_w/2"), None),
        TransitionDirection::Right => (Some("1+#ppt_w/2"), None),
        TransitionDirection::Up => (None, Some("0-#ppt_h/2")),
        TransitionDirection::Down => (None, Some("1+#ppt_h/2")),
    };
    let mut out = String::new();
    for (name, edge, home) in [("ppt_x", x_edge, "#ppt_x"), ("ppt_y", y_edge, "#ppt_y")] {
        let edge = edge.unwrap_or(home);
        let (from, to) = if entering { (edge, home) } else { (home, edge) };
        out.push_str(&slide_value(ids, spid, duration, name, from, to));
    }
    out
}

fn wipe_filter(direction: TransitionDirection) -> &'static str {
    match direction {
        TransitionDirection::Left => "wipe(left)",
        TransitionDirection::Right => "wipe(right)",
        TransitionDirection::Up => "wipe(up)",
        TransitionDirection::Down => "wipe(down)",
    }
}

fn scale_by(ids: &mut Ids, spid: u32, duration: i64, percent: i64) -> String {
    let id = ids.next();
    format!(
        "<p:animScale><p:cBhvr><p:cTn id=\"{id}\" dur=\"{duration}\" autoRev=\"1\" fill=\"hold\"/>{}</p:cBhvr><p:by x=\"{}\" y=\"{}\"/></p:animScale>",
        target(spid),
        percent * 1000,
        percent * 1000
    )
}

fn rotate_by(ids: &mut Ids, spid: u32, duration: i64, degrees: i64, auto_reverse: bool) -> String {
    let id = ids.next();
    format!(
        "<p:animRot by=\"{}\"><p:cBhvr><p:cTn id=\"{id}\" dur=\"{duration}\"{} fill=\"hold\"/>{}<p:attrNameLst><p:attrName>r</p:attrName></p:attrNameLst></p:cBhvr></p:animRot>",
        degrees * 60000,
        if auto_reverse { " autoRev=\"1\"" } else { "" },
        target(spid)
    )
}

/// One effect on its shapes.
fn effect_xml(a: &Animation, node: &str, shapes: &[u32], ids: &mut Ids) -> String {
    let duration = ms(a.length()).max(1);
    let (preset_id, subtype) = preset(a);
    let class = match a.effect.class() {
        EffectClass::Entrance => "entr",
        EffectClass::Emphasis => "emph",
        EffectClass::Exit => "exit",
    };
    let id = ids.next();
    let mut inner = String::new();
    for &spid in shapes {
        match a.effect {
            Effect::Appear => inner.push_str(&set_visibility(ids, spid, true, 0)),
            Effect::Disappear => inner.push_str(&set_visibility(ids, spid, false, 0)),
            Effect::FadeIn => {
                inner.push_str(&set_visibility(ids, spid, true, 0));
                inner.push_str(&fade(ids, spid, duration, true, "fade"));
            }
            Effect::FadeOut => {
                inner.push_str(&fade(ids, spid, duration, false, "fade"));
                inner.push_str(&set_visibility(ids, spid, false, duration));
            }
            Effect::FlyIn => {
                inner.push_str(&set_visibility(ids, spid, true, 0));
                inner.push_str(&fly(ids, spid, duration, a.direction, true));
            }
            Effect::FlyOut => {
                inner.push_str(&fly(ids, spid, duration, a.direction, false));
                inner.push_str(&set_visibility(ids, spid, false, duration));
            }
            Effect::FloatIn => {
                inner.push_str(&set_visibility(ids, spid, true, 0));
                inner.push_str(&slide_value(ids, spid, duration, "ppt_y", "#ppt_y+.06", "#ppt_y"));
                inner.push_str(&fade(ids, spid, duration, true, "fade"));
            }
            Effect::FloatOut => {
                inner.push_str(&slide_value(ids, spid, duration, "ppt_y", "#ppt_y", "#ppt_y-.06"));
                inner.push_str(&fade(ids, spid, duration, false, "fade"));
                inner.push_str(&set_visibility(ids, spid, false, duration));
            }
            Effect::ZoomIn | Effect::GrowTurn => {
                inner.push_str(&set_visibility(ids, spid, true, 0));
                inner.push_str(&slide_value(ids, spid, duration, "ppt_w", "0", "#ppt_w"));
                inner.push_str(&slide_value(ids, spid, duration, "ppt_h", "0", "#ppt_h"));
                inner.push_str(&fade(ids, spid, duration, true, "fade"));
            }
            Effect::ZoomOut => {
                inner.push_str(&slide_value(ids, spid, duration, "ppt_w", "#ppt_w", "0"));
                inner.push_str(&slide_value(ids, spid, duration, "ppt_h", "#ppt_h", "0"));
                inner.push_str(&fade(ids, spid, duration, false, "fade"));
                inner.push_str(&set_visibility(ids, spid, false, duration));
            }
            Effect::WipeIn => {
                inner.push_str(&set_visibility(ids, spid, true, 0));
                inner.push_str(&fade(ids, spid, duration, true, wipe_filter(a.direction)));
            }
            Effect::WipeOut => {
                inner.push_str(&fade(ids, spid, duration, false, wipe_filter(a.direction)));
                inner.push_str(&set_visibility(ids, spid, false, duration));
            }
            Effect::Pulse => inner.push_str(&scale_by(ids, spid, duration, 15)),
            Effect::GrowShrink => inner.push_str(&scale_by(ids, spid, duration, 50)),
            Effect::Spin => inner.push_str(&rotate_by(ids, spid, duration, 360, false)),
            Effect::Teeter => inner.push_str(&rotate_by(ids, spid, duration / 4, 8, true)),
        }
    }
    format!(
        "<p:par><p:cTn id=\"{id}\" presetID=\"{preset_id}\" presetClass=\"{class}\" presetSubtype=\"{subtype}\" fill=\"hold\" nodeType=\"{node}\"><p:stCondLst><p:cond delay=\"{}\"/></p:stCondLst><p:childTnLst>{inner}</p:childTnLst></p:cTn></p:par>",
        ms(a.delay.max(0.0))
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;
    use crate::imaging::ColorRgba;

    #[test]
    fn every_effect_makes_well_formed_timing_with_unique_ids() {
        let layer = Document::blank(10, 10, ColorRgba::WHITE).layers()[0].id();
        let animations: Vec<Animation> = Effect::ALL
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let mut a = Animation::new(layer, *e);
                a.start = Start::ALL[i % 3];
                a
            })
            .collect();
        let xml = timing_xml(&animations, &|_| vec![4, 5]);
        let wrapped = format!("<root xmlns:p=\"urn:p\">{xml}</root>");
        let doc = roxmltree::Document::parse(&wrapped).expect("well-formed");
        let mut seen = std::collections::HashSet::new();
        for node in doc.descendants().filter(|n| n.has_tag_name("cTn")) {
            let id: u32 = node.attribute("id").expect("id").parse().expect("number");
            assert!(seen.insert(id), "id {id} used twice");
        }
        assert!(
            xml.contains("presetClass=\"entr\"")
                && xml.contains("presetClass=\"emph\"")
                && xml.contains("presetClass=\"exit\"")
        );
        assert!(xml.contains("spid=\"4\"") && xml.contains("spid=\"5\""));
        assert_eq!("", timing_xml(&animations, &|_| Vec::new()));
    }
}
