use std::cmp::Ordering;

use vpsearch;

use crate::palette::Palette;
use crate::cluster::Cluster;
use crate::histogram::Histogram;

struct ColormapEntry;
impl vpsearch::MetricSpace<ColormapEntry> for [f32; 4] {
    type UserData = ();
    type Distance = f32;

    fn distance(&self, other: &Self, _: &Self::UserData) -> Self::Distance {
        color_dist(self, other).sqrt()
    }
}

type ColormapTree = vpsearch::Tree::<[f32; 4], ColormapEntry>;

pub struct Colormap {
    entries: Vec<[f32; 4]>,
    tree: ColormapTree,
    pub error: f32,
}

impl Colormap {
    pub fn from_clusters(clusters: &Vec::<Cluster>) -> Self {
        assert!(clusters.len() <= 256);

        let mut total_weight = 0f32;

        let mut entries: Vec::<[f32; 4]> = clusters.iter().map(|c|{
            total_weight += c.weight as f32;
            [c.mean[0] as f32, c.mean[1] as f32, c.mean[2] as f32, c.mean[3] as f32]
        }).collect();

        let mut tree = vpsearch::Tree::new(&entries);

        kmeans(clusters, &mut entries, &tree, total_weight);
        tree = vpsearch::Tree::new(&entries);

        let error = kmeans(clusters, &mut entries, &tree, total_weight);
        sort_colors(&mut entries);

        tree = vpsearch::Tree::new(&entries);

        Self{
            entries: entries,
            tree: tree,
            error: error,
        }
    }

    pub fn from_histogram(hist: &Histogram) -> Self {
        assert!(hist.0.len() <= 256);

        let mut entries: Vec::<[f32; 4]> = hist.0.values().map(|e|{
            [e.color[0] as f32, e.color[1] as f32, e.color[2] as f32, e.color[3] as f32]
        }).collect();

        sort_colors(&mut entries);

        let tree = vpsearch::Tree::new(&entries);

        Self{
            entries: entries,
            tree: tree,
            error: 0f32,
        }
    }

    pub fn generate_palette(&self, palette: &mut Palette) {
        palette.count = self.entries.len() as u32;

        for (i, e) in self.entries.iter().enumerate() {
            let c = &mut palette.entries[i];
            c.r = e[0].round().clamp(0.0, 255.0) as u8;
            c.g = e[1].round().clamp(0.0, 255.0) as u8;
            c.b = e[2].round().clamp(0.0, 255.0) as u8;
            c.a = e[3].round().clamp(0.0, 255.0) as u8;
        }
    }

    #[inline(always)]
    pub fn nearest_ind(&self, color: &[f32; 4]) -> (usize, f32) {
        self.tree.find_nearest(color)
    }

    pub fn color(&self, ind: usize) -> &[f32; 4] {
        &self.entries[ind]
    }
}

fn kmeans(clusters: &Vec::<Cluster>, entries: &mut Vec<[f32; 4]>, tree: &ColormapTree, total_weight: f32) -> f32 {
    let mut colors = [[0f32; 4]; 256];
    let mut weights = [0f32; 256];

    let mut total_err = 0f32;

    for cluster in clusters.iter() {
        for entry in cluster.entries.iter() {
            let hist_color = [
                entry.color[0] as f32,
                entry.color[1] as f32,
                entry.color[2] as f32,
                entry.color[3] as f32,
            ];
            let weight = entry.weight as f32;

            let (ind, err) = tree.find_nearest(&hist_color);

            let color = &mut colors[ind];
            add_color(color, &hist_color, weight);

            weights[ind] += weight;
            total_err += err*err;
        }
    }

    for ((pal_c, c), weight) in entries.iter_mut().zip(colors).zip(weights) {
        if weight > 0.0 {
            pal_c[0] = c[0] / weight;
            pal_c[1] = c[1] / weight;
            pal_c[2] = c[2] / weight;
            pal_c[3] = c[3] / weight;
        }
    }

    return total_err / total_weight;
}

fn sort_colors(entries: &mut Vec<[f32; 4]>) {
    entries.sort_unstable_by(|e1, e2| {
        e1[3].partial_cmp(&e2[3]).unwrap_or(Ordering::Equal)
    });
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn add_color(dst: &mut [f32; 4], src: &[f32; 4], weight: f32) {
    unsafe {
        use std::arch::x86_64::*;

        let mut psrc = _mm_loadu_ps(src.as_ptr());
        let mut pdst = _mm_loadu_ps(dst.as_ptr());
        let pweights = _mm_set1_ps(weight);

        psrc = _mm_mul_ps(psrc, pweights);
        pdst = _mm_add_ps(pdst, psrc);

        _mm_storeu_ps(dst.as_mut_ptr(), pdst);
    }
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
fn add_color(dst: &mut [f32; 4], src: &[f32; 4], weight: f32) {
    dst[0] += src[0] * weight;
    dst[1] += src[1] * weight;
    dst[2] += src[2] * weight;
    dst[3] += src[3] * weight;
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn color_dist(c1: &[f32; 4], c2: &[f32; 4]) -> f32 {
    unsafe {
        use std::arch::x86_64::*;

        let pc1 = _mm_loadu_ps(c1.as_ptr());
        let pc2 = _mm_loadu_ps(c2.as_ptr());

        let mut dist = _mm_sub_ps(pc1, pc2);
        dist = _mm_mul_ps(dist, dist);

        let mut tmp = [0f32; 4];
        _mm_storeu_ps(tmp.as_mut_ptr(), dist);

        tmp[0] + tmp[1] + tmp[2] + tmp[3]
    }
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
fn color_dist(c1: &[f32; 4], c2: &[f32; 4]) -> f32 {
    (c1[0] - c2[0]).powi(2) +
    (c1[1] - c2[1]).powi(2) +
    (c1[2] - c2[2]).powi(2) +
    (c1[3] - c2[3]).powi(2)
}
