use crate::ord_float::OrdFloat32;

use crate::histogram::{Histogram, HistogramEntry};

pub(crate) struct Cluster<'clust> {
    pub entries: Vec<&'clust HistogramEntry>,
    pub mean: [f32; 4],
    pub weight: f32,
    pub chan_diff: f32,
    widest_chan: u8,
}

impl<'clust> Cluster<'clust> {
    pub(crate) fn new(entries: Vec<&'clust HistogramEntry>) -> Self {
        let mut cluster = Self {
            entries,
            mean: [0.0; 4],
            weight: 0.0,
            chan_diff: 0.0,
            widest_chan: 0,
        };

        cluster.calc_stats();

        cluster
    }

    pub(crate) fn from_histogram(hist: &'clust Histogram) -> Self {
        let mut entries = Vec::with_capacity(hist.map.len());

        for entry in hist.map.values() {
            entries.push(entry)
        }

        Self::new(entries)
    }

    fn calc_stats(&mut self) {
        self.mean = [0.0; 4];
        self.weight = 0.0;

        if self.entries.is_empty() {
            self.chan_diff = 0.0;
            return;
        }

        let mut sum = [0u64; 4];
        let mut weight = 0u64;

        for e in self.entries.iter() {
            add_weighted(&mut sum, &e.color, e.weight);
            weight += u64::from(e.weight);
        }

        self.mean = weighted_mean(&sum, weight);
        self.weight = weight as f32;

        let mut tmp;
        let mut diff_sum: [f32; 4] = [0f32; 4];

        for &e in self.entries.iter() {
            let weight = e.weight as f32;

            tmp = [
                e.color[0] as f32,
                e.color[1] as f32,
                e.color[2] as f32,
                e.color[3] as f32,
            ];
            add_diff(&mut diff_sum, &tmp, &self.mean, weight);
        }

        let (chan, max_diff_sum) = diff_sum
            .iter()
            .enumerate()
            .max_by_key(|&(_, &d)| OrdFloat32::from(d))
            .unwrap();

        self.chan_diff = max_diff_sum / self.weight;
        self.widest_chan = chan as u8;
    }

    pub(crate) fn split_into(self, max_colors: usize) -> Vec<Self> {
        let max_colors_f32 = max_colors as f32;

        let mut clusters = Vec::<Self>::with_capacity(max_colors);

        clusters.push(self);

        while clusters.len() < max_colors {
            // We want to split bigger clusters in the beginning,
            // and clusters with bigger chan_diff in the end
            let weight_ratio = 0.75 - (clusters.len() as f32 + 1.0) / max_colors_f32 / 2.0;

            // Get the best cluster to split
            let to_split_opt = clusters
                .iter()
                .enumerate()
                .filter(|(_, c)| c.chan_diff > 0.0)
                .map(|(i, c)| {
                    let priority = c.chan_diff * c.weight.powf(weight_ratio);
                    (i, priority)
                })
                .max_by_key(|&(_, p)| OrdFloat32::from(p))
                .map(|(i, _)| clusters.swap_remove(i));

            // If nothing there, this means everything is ready
            let mut to_split = match to_split_opt {
                Some(c) => c,
                None => break,
            };

            let (mut c1, mut c2) = to_split.split();

            if c1.entries.is_empty() {
                c2.chan_diff = 0.0;
                clusters.push(c2);
                continue;
            }

            if c2.entries.is_empty() {
                c1.chan_diff = 0.0;
                clusters.push(c1);
                continue;
            }

            clusters.push(c1);
            clusters.push(c2);
        }

        clusters
    }

    fn split(&mut self) -> (Self, Self) {
        let widest_chan = self.widest_chan as usize;
        let widest_chan_mean = self.mean[widest_chan];

        let mut i: usize = 0;
        let mut lt: usize = 0;
        let mut gt: usize = self.entries.len();

        let mut lt_weight: usize = 0;
        let mut gt_weight: usize = 0;

        while i < gt {
            let entry = self.entries[i];
            let val = entry.color[widest_chan] as f32;

            if val < widest_chan_mean {
                lt_weight += entry.weight as usize;
                if lt != i {
                    self.entries.swap(lt, i);
                }
                lt += 1;
                i += 1;
            } else if val > widest_chan_mean {
                gt_weight += entry.weight as usize;
                gt -= 1;
                self.entries.swap(gt, i);
            } else {
                i += 1;
            }
        }

        let mut split_pos = i;
        if lt_weight > gt_weight {
            split_pos = lt;
        }

        let (sp1, sp2) = self.entries.split_at(split_pos);

        (Self::new(sp1.to_vec()), Self::new(sp2.to_vec()))
    }
}

// Exact integer sums: f32 ones round past 2^24
#[inline(always)]
pub(crate) fn add_weighted(sum: &mut [u64; 4], color: &[u8; 4], weight: u32) {
    let weight = u64::from(weight);
    sum[0] += u64::from(color[0]) * weight;
    sum[1] += u64::from(color[1]) * weight;
    sum[2] += u64::from(color[2]) * weight;
    sum[3] += u64::from(color[3]) * weight;
}

#[inline(always)]
pub(crate) fn weighted_mean(sum: &[u64; 4], weight: u64) -> [f32; 4] {
    let weight = weight as f64;
    [
        (sum[0] as f64 / weight) as f32,
        (sum[1] as f64 / weight) as f32,
        (sum[2] as f64 / weight) as f32,
        (sum[3] as f64 / weight) as f32,
    ]
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn add_diff(dst: &mut [f32; 4], a: &[f32; 4], b: &[f32; 4], weight: f32) {
    unsafe {
        use std::arch::x86_64::*;

        let pa = _mm_loadu_ps(a.as_ptr());
        let pb = _mm_loadu_ps(b.as_ptr());
        let mut pdst = _mm_loadu_ps(dst.as_ptr());
        let pweights = _mm_set1_ps(weight);

        let mut diff = _mm_sub_ps(pa, pb);

        // Abs
        let mask = _mm_castsi128_ps(_mm_srli_epi32(_mm_set1_epi32(-1), 1));
        diff = _mm_and_ps(mask, diff);

        diff = _mm_mul_ps(diff, pweights);
        pdst = _mm_add_ps(pdst, diff);

        _mm_storeu_ps(dst.as_mut_ptr(), pdst);
    }
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
fn add_diff(dst: &mut [f32; 4], a: &[f32; 4], b: &[f32; 4], weight: f32) {
    unsafe {
        use std::arch::aarch64::*;

        let pa = vld1q_f32(a.as_ptr());
        let pb = vld1q_f32(b.as_ptr());
        let mut pdst = vld1q_f32(dst.as_ptr());
        let pweights = vmovq_n_f32(weight);

        let mut diff = vsubq_f32(pa, pb);
        diff = vabsq_f32(diff);

        diff = vmulq_f32(diff, pweights);
        pdst = vaddq_f32(pdst, diff);

        vst1q_f32(dst.as_mut_ptr(), pdst);
    }
}

#[cfg(not(any(
    target_arch = "x86_64",
    all(target_arch = "aarch64", target_feature = "neon")
)))]
#[inline(always)]
fn add_diff(dst: &mut [f32; 4], a: &[f32; 4], b: &[f32; 4], weight: f32) {
    dst[0] += (a[0] - b[0]).abs() * weight;
    dst[1] += (a[1] - b[1]).abs() * weight;
    dst[2] += (a[2] - b[2]).abs() * weight;
    dst[3] += (a[3] - b[3]).abs() * weight;
}

#[cfg(test)]
mod tests {
    use super::Cluster;
    use crate::histogram::HistogramEntry;

    fn assert_means_within_values(cluster: &Cluster) {
        let (min_col, max_col) =
            cluster
                .entries
                .iter()
                .fold(([u8::MAX; 4], [u8::MIN; 4]), |(min, max), e| {
                    let mut min = min;
                    let mut max = max;
                    for ch in 0..4 {
                        min[ch] = min[ch].min(e.color[ch]);
                        max[ch] = max[ch].max(e.color[ch]);
                    }
                    (min, max)
                });

        let iter = cluster.mean.iter().zip(min_col).zip(max_col).enumerate();
        for (ch, ((mean, min), max)) in iter {
            assert!(
                (min as f32..=max as f32).contains(mean),
                "channel {ch} mean {mean} outside {min}..={max}"
            );
            if min == max {
                assert_eq!(*mean, min as f32, "channel {ch} mean of equal values");
            }
        }

        if min_col == max_col {
            assert_eq!(cluster.chan_diff, 0.0);
        }
    }

    fn split_and_check(entries: &[HistogramEntry], total_weight: f32, max_colors: usize) {
        let root = Cluster::new(entries.iter().collect());
        assert_eq!(root.weight, total_weight);

        // Split the root cluster into smaller clusters and check each one
        let clusters = root.split_into(max_colors);
        assert!((1..=max_colors).contains(&clusters.len()));

        for cluster in &clusters {
            assert_means_within_values(cluster);
        }
    }

    fn histogram_entries(colors: &[([u8; 4], u32)]) -> Vec<HistogramEntry> {
        colors
            .iter()
            .map(|&(color, weight)| HistogramEntry { color, weight })
            .collect()
    }

    /// Test with a known near-uniform static histogram that used to panic in `split`
    #[test]
    fn near_uniform_static() {
        let hist = histogram_entries(&[
            ([250, 246, 240, 255], 261911),
            ([251, 248, 243, 255], 193),
            ([249, 244, 237, 255], 6),
            ([249, 245, 238, 255], 5),
            ([249, 246, 239, 255], 4),
            ([250, 245, 238, 255], 4),
            ([250, 247, 242, 255], 4),
            ([250, 247, 241, 255], 3),
            ([250, 246, 239, 255], 2),
            ([250, 247, 240, 255], 2),
            ([250, 246, 241, 255], 2),
            ([249, 246, 240, 255], 2),
            ([250, 245, 239, 255], 1),
            ([250, 245, 240, 255], 1),
            ([249, 245, 239, 255], 1),
            ([249, 244, 238, 255], 1),
            ([250, 246, 242, 255], 1),
            ([251, 248, 242, 255], 1),
        ]);
        let total_weight: f32 = hist.iter().map(|entry| entry.weight as f32).sum();
        split_and_check(&hist, total_weight, 16);
    }

    /// Test with a dynamically generated near-uniform histogram
    #[test]
    fn near_uniform_dynamic() {
        let shades: [([u8; 4], u32); 5] = [
            ([251, 248, 243, 255], 193),
            ([249, 244, 237, 255], 6),
            ([250, 247, 242, 255], 4),
            ([250, 246, 239, 255], 2),
            ([251, 248, 242, 255], 1),
        ];
        for side in [1024u32, 2048, 4096] {
            let scale = side * side / (512 * 512);
            let shaded: u32 = shades.iter().map(|&(_, w)| w * scale).sum();
            let mut colors = vec![([250, 246, 240, 255], side * side - shaded)];
            colors.extend(shades.iter().map(|&(c, w)| (c, w * scale)));

            let hist = histogram_entries(&colors);

            for max_colors in [2, 4, 16, 128, 256] {
                split_and_check(&hist, (side * side) as f32, max_colors);
            }
        }
    }

    /// Test with a randomly generated near-uniform histogram
    #[test]
    fn near_uniform_random() {
        let mut seed: u64 = 0x1a26e;
        let mut rand = |n: u32| {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((seed >> 33) as u32) % n
        };

        const SIDE_OPTIONS: [u32; 4] = [512, 1024, 2048, 4096];
        const MAX_COLORS_OPTIONS: [usize; 5] = [2, 4, 16, 128, 256];

        for _ in 0..300 {
            let side = SIDE_OPTIONS[rand(4) as usize];
            let base = [rand(256) as u8, rand(256) as u8, rand(256) as u8, 255];
            let mut colors = vec![];
            let mut shaded = 0;
            for _ in 0..1 + rand(12) {
                let mut shade = base;
                for v in &mut shade {
                    *v = v.saturating_add_signed(rand(5) as i8 - 2);
                }
                let weight = 1 + rand(side);
                shaded += weight;
                colors.push((shade, weight));
            }
            colors.push((base, side * side - shaded));
            colors.sort_unstable_by_key(|&(c, _)| c);
            colors.dedup_by(|a, b| {
                let same = a.0 == b.0;
                if same {
                    b.1 += a.1;
                }
                same
            });

            let hist: Vec<HistogramEntry> = histogram_entries(&colors);
            let max_colors = MAX_COLORS_OPTIONS[rand(5) as usize];
            split_and_check(&hist, (side * side) as f32, max_colors);
        }
    }
}
