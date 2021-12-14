use crate::quantize::Palette;

macro_rules! color_dist {
    ($c1: expr, $c2: expr) => {
        ($c1[0] - $c2[0]).powi(2) +
        ($c1[1] - $c2[1]).powi(2) +
        ($c1[2] - $c2[2]).powi(2) +
        ($c1[3] - $c2[3]).powi(2)
    };
}

struct ColormapEntry {
    pub color: [f32; 4],
    radius: f32,
}

pub struct Colormap(Vec<ColormapEntry>);

impl Colormap {
    pub fn new(palette: &Palette) -> Self {
        let mut entries = vec![];

        let mut ci = 0;
        entries.resize_with(palette.count as usize, ||{
            let c = palette.entries[ci];
            let e = ColormapEntry{
                color: [c.r as f32, c.g as f32, c.b as f32, c.a as f32],
                radius: 0.0,
            };
            ci += 1;
            e
        });

        let count = entries.len();
        let mut nearest = [f32::MAX; 256];

        assert!(count <= 256);

        for i in 0..count-1 {
            for j in i+1..count {
                let dist = color_dist!(entries[i].color, entries[j].color);

                nearest[i] = nearest[i].min(dist);
                nearest[j] = nearest[j].min(dist);
            }
        }

        for i in 0..count {
            entries[i].radius = nearest[i] / 2.0;
        }

        Self{0: entries}
    }

    pub fn nearest_ind(&self, color: &[f32; 4], last_ind: usize) -> usize {
        let mut best_ind = last_ind;

        let mut best_dist = color_dist!(self.0[last_ind].color, color);
        if best_dist <= self.0[last_ind].radius {
            return last_ind;
        }

        for (i, e) in self.0.iter().enumerate() {
            let dist = color_dist!(e.color, color);

            if dist <= e.radius {
                return i
            }

            if dist < best_dist {
                best_dist = dist;
                best_ind = i;
            }
        }

        best_ind
    }

    pub fn color(&self, ind: usize) -> [f32; 4] {
        self.0[ind].color
    }
}
