use std::cmp::Ordering;

use crate::quantize::Palette;
use crate::cluster::Cluster;
use crate::image::Image;

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
    popularity: usize,
    radius: f32,
}

pub struct Colormap(Vec<ColormapEntry>);

impl Colormap {
    pub fn new(clusters: &Vec::<Cluster>) -> Self {
        let mut entries = vec![];

        let mut ci = 0;
        entries.resize_with(clusters.len() as usize, ||{
            let c = &clusters[ci];
            let e = ColormapEntry{
                color: [c.mean.r as f32, c.mean.g as f32, c.mean.b as f32, c.mean.a as f32],
                popularity: c.colors.len(),
                radius: 0.0,
            };
            ci += 1;
            e
        });

        entries.sort_by(|e1, e2| {
            match e1.color[3].partial_cmp(&e2.color[3]).unwrap_or(Ordering::Equal) {
                Ordering::Equal => {
                    e2.popularity.cmp(&e1.popularity)
                },
                o => o,
            }
        });

        let mut res = Self{0: entries};
        res.calc_radiuses();
        res
    }

    fn calc_radiuses(&mut self) {
        let count = self.0.len();
        let mut nearest = [f32::MAX; 256];

        assert!(count <= 256);

        for i in 0..count-1 {
            for j in i+1..count {
                let dist = color_dist!(self.0[i].color, self.0[j].color);

                nearest[i] = nearest[i].min(dist);
                nearest[j] = nearest[j].min(dist);
            }
        }

        for i in 0..count {
            self.0[i].radius = nearest[i] / 2.0;
        }
    }

    pub fn fix(&mut self, image: &Image, indexes: &[u8]) {
        let mut colors = [[0usize; 4]; 256];
        let mut counts = [0usize; 256];

        for point in 0..image.width*image.height {
            let data_point = point*4;

            let pix = &image.data[data_point..data_point+4];
            let r = pix[0] as usize;
            let g = pix[1] as usize;
            let b = pix[2] as usize;
            let a = pix[3] as usize;

            let ind = indexes[point] as usize;

            let color = &mut colors[ind];
            color[0] += r;
            color[1] += g;
            color[2] += b;
            color[3] += a;
            counts[ind] += 1;
        }

        for (i, c) in colors.iter().enumerate() {
            let count = counts[i];

            if count > 0 {
                let pal_c = &mut self.0[i];
                pal_c.color[0] = (c[0] / count) as f32;
                pal_c.color[1] = (c[1] / count) as f32;
                pal_c.color[2] = (c[2] / count) as f32;
                pal_c.color[3] = (c[3] / count) as f32;
            }
        }

        self.calc_radiuses()
    }

    pub fn generate_palette(&self, palette: &mut Palette) {
        palette.count = self.0.len() as u32;

        for (i, e) in self.0.iter().enumerate() {
            let c = &mut palette.entries[i];
            c.r = e.color[0].clamp(0.0, 255.0) as u8;
            c.g = e.color[1].clamp(0.0, 255.0) as u8;
            c.b = e.color[2].clamp(0.0, 255.0) as u8;
            c.a = e.color[3].clamp(0.0, 255.0) as u8;
        }
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
