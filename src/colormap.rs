use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use num_traits::Float;

use crate::ord_float::OrdFloat32;

use crate::vpsearch;

use crate::cluster::Cluster;
use crate::histogram::Histogram;
use crate::palette::Palette;

pub(crate) struct Colormap {
    palette: Palette,
    tree: vpsearch::SearchTree,
    pub(crate) error: f32,
}

impl Colormap {
    pub(crate) fn from_clusters(clusters: &Vec<Cluster>) -> Self {
        assert!(clusters.len() <= 256);

        let size = clusters.len();
        let mut entries = [[0f32; 4]; 256];
        let mut weights = [0f32; 256];
        let mut total_weight = 0f32;

        clusters.iter().enumerate().for_each(|(i, c)| {
            entries[i] = [c.mean[0], c.mean[1], c.mean[2], c.mean[3]];

            let weight = c.weight;
            weights[i] = weight;
            total_weight += weight;
        });

        let entries_sl = &mut entries[..size];
        let mut error;

        let mut tree = vpsearch::SearchTree::new(entries_sl, &weights);
        (error, weights) = kmeans(clusters, entries_sl, &tree, total_weight);

        if error > 0.001 {
            tree = vpsearch::SearchTree::new(entries_sl, &weights);
            (error, weights) = kmeans(clusters, entries_sl, &tree, total_weight);
        }

        round_and_clamp_colors(entries_sl);
        sort_colors(entries_sl, &mut weights);

        tree = vpsearch::SearchTree::new(entries_sl, &weights);

        Self {
            palette: entries[..size].into(),
            tree,
            error,
        }
    }

    pub(crate) fn from_histogram(hist: &Histogram) -> Self {
        assert!(hist.map.len() <= 256);

        let size = hist.map.len();
        let mut entries = [[0f32; 4]; 256];
        let mut weights = [0f32; 256];

        hist.map.values().enumerate().for_each(|(i, e)| {
            entries[i] = [
                e.color[0] as f32,
                e.color[1] as f32,
                e.color[2] as f32,
                e.color[3] as f32,
            ];
            weights[i] = e.weight as f32;
        });

        let entries_sl = &mut entries[..size];

        sort_colors(entries_sl, &mut weights);

        let tree = vpsearch::SearchTree::new(entries_sl, &weights);

        Self {
            palette: entries[..size].into(),
            tree,
            error: 0f32,
        }
    }

    pub(crate) fn get_palette(&self) -> &Palette {
        &self.palette
    }

    #[inline(always)]
    pub(crate) fn nearest_ind(&self, color: &[f32; 4]) -> (u8, [f32; 4], f32) {
        self.tree.find_nearest(color)
    }
}

fn kmeans(
    clusters: &Vec<Cluster>,
    entries: &mut [[f32; 4]],
    tree: &vpsearch::SearchTree,
    total_weight: f32,
) -> (f32, [f32; 256]) {
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

            let (ind, _, err) = tree.find_nearest(&hist_color);

            let color = &mut colors[usize::from(ind)];
            add_color(color, &hist_color, weight);

            weights[ind as usize] += weight;
            total_err += err * err;
        }
    }

    for ((ec, c), weight) in entries.iter_mut().zip(colors).zip(weights) {
        if weight > 0.0 {
            ec[0] = c[0] / weight;
            ec[1] = c[1] / weight;
            ec[2] = c[2] / weight;
            ec[3] = c[3] / weight;
        }
    }

    (total_err / total_weight, weights)
}

fn round_and_clamp_colors(entries: &mut [[f32; 4]]) {
    for entry in entries.iter_mut() {
        entry[0] = entry[0].round().clamp(0.0, 255.0);
        entry[1] = entry[1].round().clamp(0.0, 255.0);
        entry[2] = entry[2].round().clamp(0.0, 255.0);
        entry[3] = entry[3].round().clamp(0.0, 255.0);
    }
}

/// Sort colors by alpha channel for better PNG compression.
/// Weights are sorted along with the colors.
fn sort_colors(entries: &mut [[f32; 4]], weights: &mut [f32]) {
    assert!(weights.len() >= entries.len());

    let mut indexes: Vec<usize> = (0..entries.len()).collect();
    indexes.sort_by_cached_key(|&i| OrdFloat32::from(entries[i][3]));

    for i in 0..indexes.len() {
        if indexes[i] != i {
            let mut current = i;

            loop {
                let target = indexes[current];

                indexes[current] = current;

                if indexes[target] == target {
                    break;
                }

                entries.swap(current, target);
                weights.swap(current, target);

                current = target;
            }
        }
    }

    // entries.sort_unstable_by_key(|e| OrdFloat32::from(e[3]));
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn add_color(dst: &mut [f32; 4], src: &[f32; 4], weight: f32) {
    unsafe {
        use core::arch::x86_64::*;

        let mut psrc = _mm_loadu_ps(src.as_ptr());
        let mut pdst = _mm_loadu_ps(dst.as_ptr());
        let pweights = _mm_set1_ps(weight);

        psrc = _mm_mul_ps(psrc, pweights);
        pdst = _mm_add_ps(pdst, psrc);

        _mm_storeu_ps(dst.as_mut_ptr(), pdst);
    }
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
fn add_color(dst: &mut [f32; 4], src: &[f32; 4], weight: f32) {
    unsafe {
        use core::arch::aarch64::*;

        let mut psrc = vld1q_f32(src.as_ptr());
        let mut pdst = vld1q_f32(dst.as_ptr());
        let pweights = vmovq_n_f32(weight);

        psrc = vmulq_f32(psrc, pweights);
        pdst = vaddq_f32(pdst, psrc);

        vst1q_f32(dst.as_mut_ptr(), pdst);
    }
}

#[cfg(not(any(
    target_arch = "x86_64",
    all(target_arch = "aarch64", target_feature = "neon")
)))]
#[inline(always)]
fn add_color(dst: &mut [f32; 4], src: &[f32; 4], weight: f32) {
    dst[0] += src[0] * weight;
    dst[1] += src[1] * weight;
    dst[2] += src[2] * weight;
    dst[3] += src[3] * weight;
}
