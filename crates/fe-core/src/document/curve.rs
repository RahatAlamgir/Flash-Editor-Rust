//! Tone curves and levels as 256-entry lookup tables. Same math as the C# `CurveMath`
//! (monotone cubic curve, so it never overshoots between its points).

use crate::imaging::color_math::round_even;

/// One point of a tone curve: input `x` and output `y`, both 0..=255.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurvePoint {
    pub x: f64,
    pub y: f64,
}

impl CurvePoint {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// The straight line from (0, 0) to (255, 255): changes nothing.
pub const STRAIGHT: [CurvePoint; 2] = [CurvePoint::new(0.0, 0.0), CurvePoint::new(255.0, 255.0)];

/// True when the curve changes nothing (missing, empty or the straight line).
pub fn is_straight(points: Option<&[CurvePoint]>) -> bool {
    match points {
        None => true,
        Some(p) => p.is_empty() || p == STRAIGHT,
    }
}

/// The 256-entry table of a curve.
pub fn build_table(points: Option<&[CurvePoint]>) -> [u8; 256] {
    let mut table = [0u8; 256];
    let Some(points) = points.filter(|_| !is_straight(points)) else {
        table.iter_mut().enumerate().for_each(|(i, v)| *v = i as u8);
        return table;
    };
    let mut sorted = points.to_vec();
    sorted.sort_by(|a, b| a.x.total_cmp(&b.x));
    // Points on the same whole x: the last one wins.
    let mut list: Vec<CurvePoint> = Vec::new();
    for p in sorted {
        let x = round_even(p.x);
        match list.last_mut() {
            Some(last) if last.x == x => *last = CurvePoint::new(x, p.y),
            _ => list.push(CurvePoint::new(x, p.y)),
        }
    }
    if list[0].x > 0.0 {
        list.insert(0, CurvePoint::new(0.0, list[0].y));
    }
    let last = list[list.len() - 1];
    if last.x < 255.0 {
        list.push(CurvePoint::new(255.0, last.y));
    }
    if list.len() == 1 {
        list.push(CurvePoint::new(list[0].x + 1.0, list[0].y));
    }
    let n = list.len();
    let xs: Vec<f64> = list.iter().map(|p| p.x).collect();
    let ys: Vec<f64> = list.iter().map(|p| p.y.clamp(0.0, 255.0)).collect();
    let delta: Vec<f64> = (0..n - 1).map(|i| (ys[i + 1] - ys[i]) / (xs[i + 1] - xs[i])).collect();
    let mut m = vec![0.0; n];
    m[0] = delta[0];
    m[n - 1] = delta[n - 2];
    for i in 1..n - 1 {
        m[i] = if delta[i - 1] * delta[i] <= 0.0 { 0.0 } else { (delta[i - 1] + delta[i]) / 2.0 };
    }
    for i in 0..n - 1 {
        if delta[i] == 0.0 {
            m[i] = 0.0;
            m[i + 1] = 0.0;
            continue;
        }
        let (a, b) = (m[i] / delta[i], m[i + 1] / delta[i]);
        let s = a * a + b * b;
        if s > 9.0 {
            let t = 3.0 / s.sqrt();
            m[i] = t * a * delta[i];
            m[i + 1] = t * b * delta[i];
        }
    }
    let mut seg = 0;
    for (x, out) in table.iter_mut().enumerate() {
        let x = x as f64;
        while seg < n - 2 && x > xs[seg + 1] {
            seg += 1;
        }
        let h = xs[seg + 1] - xs[seg];
        let t = (x - xs[seg]) / h;
        let (t2, t3) = (t * t, t * t * t);
        let y = (2.0 * t3 - 3.0 * t2 + 1.0) * ys[seg]
            + (t3 - 2.0 * t2 + t) * h * m[seg]
            + (-2.0 * t3 + 3.0 * t2) * ys[seg + 1]
            + (t3 - t2) * h * m[seg + 1];
        *out = round_even(y).clamp(0.0, 255.0) as u8;
    }
    table
}

/// The table of a Levels setting: input black / white points, gamma (>1 lightens), output range.
pub fn levels_table(in_black: f64, in_white: f64, gamma: f64, out_black: f64, out_white: f64) -> [u8; 256] {
    let mut table = [0u8; 256];
    let range = (in_white - in_black).max(1.0);
    let g = 1.0 / gamma.clamp(0.05, 20.0);
    for (i, out) in table.iter_mut().enumerate() {
        let v = ((i as f64 - in_black) / range).clamp(0.0, 1.0).powf(g);
        *out = round_even(out_black + v * (out_white - out_black)).clamp(0.0, 255.0) as u8;
    }
    table
}

/// The curve with a point added; a point closer than 4 steps on x is replaced.
pub fn with_point(points: Option<&[CurvePoint]>, point: CurvePoint) -> Vec<CurvePoint> {
    let base = points.filter(|p| !p.is_empty()).unwrap_or(&STRAIGHT);
    let mut list: Vec<CurvePoint> = base.iter().copied().filter(|p| (p.x - point.x).abs() >= 4.0).collect();
    list.push(CurvePoint::new(round_even(point.x).clamp(0.0, 255.0), round_even(point.y).clamp(0.0, 255.0)));
    list.sort_by(|a, b| a.x.total_cmp(&b.x));
    list
}

/// The curve without point `index`. The first and last points always stay.
pub fn without_point(points: &[CurvePoint], index: usize) -> Vec<CurvePoint> {
    if index == 0 || index + 1 >= points.len() {
        return points.to_vec();
    }
    points.iter().enumerate().filter(|&(i, _)| i != index).map(|(_, p)| *p).collect()
}
