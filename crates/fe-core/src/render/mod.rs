//! CPU drawing of objects with tiny-skia (C# `ObjectRenderer`): shape outlines (`geometry`), shapes, pictures
//! and groups (`draw`), reshaped corners (`warp`), pictures ready to draw (`pixmaps`). An object's drawing goes
//! into its layer's tiles; it is made again only when the object changes. Text, charts, codes and the looks
//! (shadow, glow, 3D...) follow in Phases 9–10; until then such objects keep their saved drawing, which is moved,
//! stretched or turned along with the object.

mod barcode_draw;
mod chart;
pub mod draw;
mod fx;
mod label;
mod qr_draw;
mod vector;
pub use fx::{from_pixmap, to_pixmap};
pub mod geometry;
pub mod pixmaps;
mod text_draw;
pub use text_draw::measure_text;
pub mod warp;

use tiny_skia::{FilterQuality, Pixmap, PixmapPaint, Transform};

use crate::imaging::{PixelRect, TileMap};
use crate::objects::StoredObject;

/// The area the object draws into (its box, corners and border), cut to the canvas.
pub fn drawing_area(obj: &StoredObject, canvas: PixelRect) -> PixelRect {
    // A shadow can reach onto the canvas from an object that is partly outside it.
    let reach = obj.box_().map_or(0.0, |b| obj.looks().reach(b.width, b.height)).ceil();
    draw::visual_bounds(obj, draw::margin_for(obj) + reach).intersect(&canvas.inflate(reach as i32))
}

/// Premultiplied pixmap pixels to straight-alpha tiles placed at `origin`.
pub fn pixmap_to_tiles(pm: &Pixmap, origin: (i32, i32)) -> TileMap {
    let mut data = pm.data().to_vec();
    for p in data.as_chunks_mut::<4>().0 {
        let a = p[3] as u32;
        if a == 0 {
            *p = [0; 4];
        } else if a < 255 {
            for c in &mut p[..3] {
                *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
    let mut tiles = TileMap::empty();
    let rect = PixelRect::new(origin.0, origin.1, pm.width() as i32, pm.height() as i32);
    tiles.write_rect(rect, &data);
    tiles
}

/// Straight-alpha tiles of `rect` to a premultiplied pixmap.
fn tiles_to_pixmap(pixels: &TileMap, rect: PixelRect) -> Option<Pixmap> {
    super::render::pixmaps::to_pixmap(&pixels.to_buffer(rect))
}

/// Draws an object into tiles (only its part on the canvas). `None` when the object (or a part of a group) is
/// of a kind the Rust version cannot draw yet.
pub fn render_object(obj: &StoredObject, canvas: PixelRect) -> Option<TileMap> {
    if !obj.can_draw() {
        return None;
    }
    let area = drawing_area(obj, canvas);
    if area.is_empty() {
        return Some(TileMap::empty());
    }
    let mut pm = Pixmap::new(area.width as u32, area.height as u32)?;
    let b = obj.box_()?;
    let t = draw::local_to_canvas(&b, obj.flips()).post_translate(-area.x as f32, -area.y as f32);
    draw::draw(&mut pm, obj, t)?;
    Some(pixmap_to_tiles(&pm, (area.x, area.y)))
}

/// The drawing of `from` (its layer pixels) moved, stretched and turned so it fits `to` (same object, another
/// box). Used for objects the Rust version cannot draw itself yet, so they keep their look while being
/// transformed. Only the part on (or near) the canvas is kept.
pub fn transform_drawing(pixels: &TileMap, from: &StoredObject, to: &StoredObject, canvas: PixelRect) -> TileMap {
    let (Some(fb), Some(tb)) = (from.box_(), to.box_()) else { return pixels.clone() };
    let source = pixels.tile_bounds().intersect(&pixels.visible_bounds());
    if source.is_empty() {
        return TileMap::empty();
    }
    // Canvas of `from` -> its box space -> canvas of `to`.
    let Some(inverse) = draw::local_to_canvas(&fb, from.flips()).invert() else { return pixels.clone() };
    let stretch =
        Transform::from_scale((tb.width / fb.width.max(1e-9)) as f32, (tb.height / fb.height.max(1e-9)) as f32);
    let map = draw::local_to_canvas(&tb, to.flips()).pre_concat(stretch).pre_concat(inverse);
    let mut corners = [
        tiny_skia::Point::from_xy(source.x as f32, source.y as f32),
        tiny_skia::Point::from_xy(source.right() as f32, source.y as f32),
        tiny_skia::Point::from_xy(source.right() as f32, source.bottom() as f32),
        tiny_skia::Point::from_xy(source.x as f32, source.bottom() as f32),
    ];
    map.map_points(&mut corners);
    let (l, t) = corners.iter().fold((f32::MAX, f32::MAX), |(l, t), p| (l.min(p.x), t.min(p.y)));
    let (r, b) = corners.iter().fold((f32::MIN, f32::MIN), |(r, b), p| (r.max(p.x), b.max(p.y)));
    let area = PixelRect::from_bounds(l as f64, t as f64, r as f64, b as f64).intersect(&canvas.inflate(64));
    let (Some(src), Some(mut pm)) =
        (tiles_to_pixmap(pixels, source), Pixmap::new(area.width.max(1) as u32, area.height.max(1) as u32))
    else {
        return TileMap::empty();
    };
    if area.is_empty() {
        return TileMap::empty();
    }
    let paint = PixmapPaint { quality: FilterQuality::Bicubic, ..PixmapPaint::default() };
    let placed = Transform::from_translate(-area.x as f32, -area.y as f32)
        .pre_concat(map)
        .pre_translate(source.x as f32, source.y as f32);
    pm.draw_pixmap(0, 0, src.as_ref(), &paint, placed, None);
    pixmap_to_tiles(&pm, (area.x, area.y))
}
