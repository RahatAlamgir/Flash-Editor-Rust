//! Lays the codewords out in the QR grid: function patterns, data, the best mask, format and version bits.

use super::qr::QrLevel;
use super::qr_tables::FORMAT_BITS;

pub struct Builder {
    version: usize,
    level: QrLevel,
    size: usize,
    codewords: Vec<u8>,
    modules: Vec<Vec<bool>>,
    /// True for finder / timing / alignment / format modules (not data).
    function: Vec<Vec<bool>>,
}

fn bit(value: i32, index: i32) -> bool {
    (value >> index) & 1 != 0
}

impl Builder {
    pub fn new(version: usize, level: QrLevel, codewords: Vec<u8>) -> Self {
        let size = version * 4 + 17;
        Self {
            version,
            level,
            size,
            codewords,
            modules: vec![vec![false; size]; size],
            function: vec![vec![false; size]; size],
        }
    }

    pub fn build(mut self, force_mask: Option<usize>) -> Vec<Vec<bool>> {
        self.draw_function_patterns();
        self.draw_codewords();
        let (mut best, mut lowest) = (0, i32::MAX);
        for mask in 0..8 {
            if force_mask.is_some_and(|m| m != mask) {
                continue;
            }
            self.apply_mask(mask);
            self.draw_format(mask);
            let penalty = self.penalty();
            if penalty < lowest {
                lowest = penalty;
                best = mask;
            }
            self.apply_mask(mask); // applying twice undoes it
        }
        self.apply_mask(best);
        self.draw_format(best);
        self.modules
    }

    fn set(&mut self, x: usize, y: usize, dark: bool) {
        self.modules[y][x] = dark;
        self.function[y][x] = true;
    }

    fn draw_function_patterns(&mut self) {
        let n = self.size;
        for i in 0..n {
            self.set(6, i, i % 2 == 0);
            self.set(i, 6, i % 2 == 0);
        }
        self.finder(3, 3);
        self.finder(n as i32 - 4, 3);
        self.finder(3, n as i32 - 4);
        let positions = self.alignment_positions();
        let count = positions.len();
        for i in 0..count {
            for j in 0..count {
                if (i == 0 && j == 0) || (i == 0 && j == count - 1) || (i == count - 1 && j == 0) {
                    continue; // where the finders are
                }
                self.alignment(positions[i], positions[j]);
            }
        }
        self.draw_format(0); // reserves the format area (the real bits are written after the mask is chosen)
        self.draw_version();
    }

    fn finder(&mut self, cx: i32, cy: i32) {
        let n = self.size as i32;
        for dy in -4i32..=4 {
            for dx in -4i32..=4 {
                let dist = dx.abs().max(dy.abs());
                let (x, y) = (cx + dx, cy + dy);
                if (0..n).contains(&x) && (0..n).contains(&y) {
                    self.set(x as usize, y as usize, dist != 2 && dist != 4);
                }
            }
        }
    }

    fn alignment(&mut self, cx: usize, cy: usize) {
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                let (x, y) = ((cx as i32 + dx) as usize, (cy as i32 + dy) as usize);
                self.set(x, y, dx.abs().max(dy.abs()) != 1);
            }
        }
    }

    fn alignment_positions(&self) -> Vec<usize> {
        if self.version == 1 {
            return Vec::new();
        }
        let count = self.version / 7 + 2;
        let step = if self.version == 32 { 26 } else { (self.version * 4 + count * 2 + 1) / (count * 2 - 2) * 2 };
        let mut result = vec![0; count];
        result[0] = 6;
        let mut pos = self.size - 7;
        for i in (1..count).rev() {
            result[i] = pos;
            pos = pos.wrapping_sub(step);
        }
        result
    }

    fn draw_format(&mut self, mask: usize) {
        let n = self.size;
        let data = FORMAT_BITS[self.level as usize] << 3 | mask as i32;
        let mut rem = data;
        for _ in 0..10 {
            rem = (rem << 1) ^ ((rem >> 9) * 0x537);
        }
        let bits = (data << 10 | rem) ^ 0x5412;
        for i in 0..=5 {
            self.set(8, i as usize, bit(bits, i));
        }
        self.set(8, 7, bit(bits, 6));
        self.set(8, 8, bit(bits, 7));
        self.set(7, 8, bit(bits, 8));
        for i in 9..15 {
            self.set((14 - i) as usize, 8, bit(bits, i));
        }
        for i in 0..8 {
            self.set(n - 1 - i as usize, 8, bit(bits, i));
        }
        for i in 8..15 {
            self.set(8, n - 15 + i as usize, bit(bits, i));
        }
        self.set(8, n - 8, true); // the one module that is always dark
    }

    fn draw_version(&mut self) {
        if self.version < 7 {
            return;
        }
        let v = self.version as i32;
        let mut rem = v;
        for _ in 0..12 {
            rem = (rem << 1) ^ ((rem >> 11) * 0x1F25);
        }
        let bits = v << 12 | rem;
        for i in 0..18 {
            let dark = bit(bits, i);
            let a = self.size - 11 + (i as usize) % 3;
            let b = (i as usize) / 3;
            self.set(a, b, dark);
            self.set(b, a, dark);
        }
    }

    fn draw_codewords(&mut self) {
        let n = self.size as i32;
        let mut i = 0usize;
        let mut right = n - 1;
        while right >= 1 {
            if right == 6 {
                right = 5; // the vertical timing line is skipped
            }
            for vert in 0..n {
                for j in 0..2 {
                    let x = (right - j) as usize;
                    let upward = ((right + 1) & 2) == 0;
                    let y = (if upward { n - 1 - vert } else { vert }) as usize;
                    if !self.function[y][x] && i < self.codewords.len() * 8 {
                        self.modules[y][x] = bit(i32::from(self.codewords[i >> 3]), 7 - (i & 7) as i32);
                        i += 1;
                    }
                }
            }
            right -= 2;
        }
    }

    fn apply_mask(&mut self, mask: usize) {
        for y in 0..self.size {
            for x in 0..self.size {
                let invert = match mask {
                    0 => (x + y) % 2 == 0,
                    1 => y % 2 == 0,
                    2 => x % 3 == 0,
                    3 => (x + y) % 3 == 0,
                    4 => (x / 3 + y / 2) % 2 == 0,
                    5 => x * y % 2 + x * y % 3 == 0,
                    6 => (x * y % 2 + x * y % 3) % 2 == 0,
                    _ => ((x + y) % 2 + x * y % 3) % 2 == 0,
                };
                if !self.function[y][x] && invert {
                    self.modules[y][x] = !self.modules[y][x];
                }
            }
        }
    }

    /// The penalty rules of the standard: long runs, 2 x 2 blocks, finder-like patterns and an unbalanced mix.
    fn penalty(&self) -> i32 {
        let n = self.size;
        let mut result = 0;
        for pass in 0..2 {
            for a in 0..n {
                let get = |b: usize| if pass == 0 { self.modules[a][b] } else { self.modules[b][a] };
                let mut run = 1;
                for b in 1..n {
                    if get(b) == get(b - 1) {
                        run += 1;
                        if run == 5 {
                            result += 3;
                        } else if run > 5 {
                            result += 1;
                        }
                    } else {
                        run = 1;
                    }
                }
                for b in 0..n.saturating_sub(10) {
                    if get(b)
                        && !get(b + 1)
                        && get(b + 2)
                        && get(b + 3)
                        && get(b + 4)
                        && !get(b + 5)
                        && get(b + 6)
                        && !get(b + 7)
                        && !get(b + 8)
                        && !get(b + 9)
                        && !get(b + 10)
                    {
                        result += 40;
                    }
                    if !get(b)
                        && !get(b + 1)
                        && !get(b + 2)
                        && !get(b + 3)
                        && get(b + 4)
                        && !get(b + 5)
                        && get(b + 6)
                        && get(b + 7)
                        && get(b + 8)
                        && !get(b + 9)
                        && get(b + 10)
                    {
                        result += 40;
                    }
                }
            }
        }
        for y in 0..n - 1 {
            for x in 0..n - 1 {
                let c = self.modules[y][x];
                if c == self.modules[y][x + 1] && c == self.modules[y + 1][x] && c == self.modules[y + 1][x + 1] {
                    result += 3;
                }
            }
        }
        let dark = self.modules.iter().flatten().filter(|m| **m).count() as i64;
        let total = (n * n) as i64;
        let diff = (dark * 20 - total * 10).abs();
        // ceil(diff / total) - 1, never below zero, times 10
        let k = (diff + total - 1) / total - 1;
        result + (k.max(0) as i32) * 10
    }
}
