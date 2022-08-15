use std::cmp::Ordering;

use crate::vpsearch;

use crate::palette::Palette;
use crate::cluster::Cluster;
use crate::histogram::Histogram;

pub(crate) struct Colormap {
    entries: Vec<[f32; 4]>,
    tree: vpsearch::SearchTree,
    pub(crate) error: f32,
}

impl Colormap {
    pub(crate) fn from_clusters(clusters: &Vec::<Cluster>) -> Self {
        assert!(clusters.len() <= 256);

        let mut weights = [0f32; 256];
        let mut total_weight = 0f32;

        let mut entries: Vec::<[f32; 4]> = clusters.iter().enumerate().map(|(i, c)|{
            let weight = c.weight as f32;
            weights[i] = weight;
            total_weight += weight;
            [c.mean[0] as f32, c.mean[1] as f32, c.mean[2] as f32, c.mean[3] as f32]
        }).collect();

        let mut error;

        let mut tree = vpsearch::SearchTree::new(&entries, &weights);
        (error, weights) = kmeans(clusters, &mut entries, &tree, total_weight);

        if error > 0.001 {
            tree = vpsearch::SearchTree::new(&entries, &weights);
            (error, weights) = kmeans(clusters, &mut entries, &tree, total_weight);
        }

        sort_colors(&mut entries);

        tree = vpsearch::SearchTree::new(&entries, &weights);

        Self{
            entries: entries,
            tree: tree,
            error: error,
        }
    }

    pub(crate) fn from_histogram(hist: &Histogram) -> Self {
        assert!(hist.map.len() <= 256);

        let mut weights = [0f32; 256];

        let mut entries: Vec::<[f32; 4]> = hist.map.values().enumerate().map(|(i, e)|{
            weights[i] = e.weight as f32;
            [e.color[0] as f32, e.color[1] as f32, e.color[2] as f32, e.color[3] as f32]
        }).collect();

        sort_colors(&mut entries);

        let tree = vpsearch::SearchTree::new(&entries, &weights);

        Self{
            entries: entries,
            tree: tree,
            error: 0f32,
        }
    }

    pub(crate) fn generate_palette(&self, palette: &mut Palette) {
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
    pub(crate) fn nearest_ind(&self, color: &[f32; 4]) -> (usize, f32) {
        self.tree.find_nearest(color, &self.entries)
    }

    pub(crate) fn color(&self, ind: usize) -> &[f32; 4] {
        &self.entries[ind]
    }
}

fn kmeans(clusters: &Vec::<Cluster>, entries: &mut Vec<[f32; 4]>, tree: &vpsearch::SearchTree, total_weight: f32) -> (f32, [f32; 256]) {
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

            let (ind, err) = tree.find_nearest(&hist_color, entries);

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

    return (total_err / total_weight, weights);
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
