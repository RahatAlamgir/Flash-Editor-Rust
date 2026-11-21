//! How long each effect takes on a 4000 x 3000 picture. Usage: `time_effects`.

use std::time::Instant;

use fe_core::effects;
use fe_core::imaging::{ColorRgba, PixelBuffer};
use fe_core::transform::{self, ResampleMode};

fn main() {
    let mut img = PixelBuffer::new(4000, 3000);
    for y in 0..3000u32 {
        for x in 0..4000u32 {
            img.set_pixel(x, y, ColorRgba::new((x % 251) as u8, (y % 241) as u8, ((x + y) % 239) as u8, 255));
        }
    }
    for e in effects::adjustments().into_iter().chain(effects::filters()) {
        let t = Instant::now();
        let _ = e.apply(&img, &e.defaults());
        println!("{:28} {:6.0} ms", e.name, t.elapsed().as_secs_f64() * 1000.0);
    }
    for (name, mode) in [("Resize half, smooth", ResampleMode::Smooth), ("Resize half, Lanczos", ResampleMode::Lanczos)]
    {
        let t = Instant::now();
        let _ = transform::resize(&img, 2000, 1500, mode);
        println!("{name:28} {:6.0} ms", t.elapsed().as_secs_f64() * 1000.0);
    }
}
