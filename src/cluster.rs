use std::cmp::Ordering;

use crate::histogram::{Histogram, HistogramEntry};

pub(crate) struct Cluster {
    pub entries: Vec<HistogramEntry>,
    pub mean: [f64; 4],
    pub weight: f64,
    pub chan_diff: f64,
    widest_chan: u8,
}

impl Cluster {
    pub(crate) fn new(entries: Vec<HistogramEntry>) -> Self {
        let mut cluster = Self{
            entries: entries,
            mean: [0.0; 4],
            weight: 0.0,
            chan_diff: 0.0,
            widest_chan: 0,
        };

        cluster.calc_stats();

        cluster
    }

    pub(crate) fn from_histogram(hist: &Histogram) -> Self {
        let mut entries = Vec::with_capacity(hist.map.len());

        for entry in hist.map.values() {
            entries.push(*entry)
        }

        Self::new(entries)
    }

    fn calc_stats(&mut self) {
        self.mean = [0.0; 4];
        self.weight = 0.0;

        if self.entries.is_empty() {
            self.chan_diff = 0.0;
            return
        }

        for e in self.entries.iter() {
            let weight = e.weight as f64;

            self.mean[0] += e.color[0] as f64 * weight;
            self.mean[1] += e.color[1] as f64 * weight;
            self.mean[2] += e.color[2] as f64 * weight;
            self.mean[3] += e.color[3] as f64 * weight;

            self.weight += weight;
        }

        self.mean[0] /= self.weight;
        self.mean[1] /= self.weight;
        self.mean[2] /= self.weight;
        self.mean[3] /= self.weight;

        let mut diff_sum: [f64; 4] = [0f64; 4];

        for e in self.entries.iter() {
            let weight = e.weight as f64;

            diff_sum[0] += (e.color[0] as f64 - self.mean[0]).abs() * weight;
            diff_sum[1] += (e.color[1] as f64 - self.mean[1]).abs() * weight;
            diff_sum[2] += (e.color[2] as f64 - self.mean[2]).abs() * weight;
            diff_sum[3] += (e.color[3] as f64 - self.mean[3]).abs() * weight;
        }

        let (chan, max_diff_sum) = diff_sum.iter().enumerate()
            .max_by(|&(_, a), &(_, b)| a.partial_cmp(&b).unwrap_or(Ordering::Equal))
            .unwrap();

        self.chan_diff = max_diff_sum / self.weight;
        self.widest_chan = chan as u8;
    }

    pub(crate) fn split_into(self, max_colors: usize) -> Vec<Self> {
        let max_colors_f64 = max_colors as f64;

        let mut clusters = Vec::<Cluster>::with_capacity(max_colors);

        clusters.push(self);

        while clusters.len() < max_colors {
            // We want to split bigger clusters in the beginning,
            // and clusters with bigger chan_diff in the end
            let weight_ratio = 0.75 - (clusters.len() as f64 + 1.0) / max_colors_f64 / 2.0;

            // Get the best cluster to split
            let to_split_opt = clusters.iter().enumerate()
                .filter(|(_, c)| c.chan_diff > 0.0)
                .map(|(i, c)|{
                    let priority = c.chan_diff * c.weight.powf(weight_ratio);
                    (i, priority)
                })
                .max_by(|&(_, a), &(_, b)| a.partial_cmp(&b).unwrap_or(Ordering::Equal))
                .map(|(i, _)| clusters.swap_remove(i) );

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

    fn split(&mut self) -> (Cluster, Cluster) {
        let widest_chan = self.widest_chan as usize;
        let widest_chan_mean = self.mean[widest_chan];

        let mut i: usize = 0;
        let mut lt: usize = 0;
        let mut gt: usize = self.entries.len() - 1;

        let mut lt_weight: usize = 0;
        let mut gt_weight: usize = 0;

        while i <= gt {
            let entry = &self.entries[i];
            let val = entry.color[widest_chan] as f64;

            if val < widest_chan_mean {
                lt_weight += entry.weight as usize;
                if lt != i {
                    self.entries.swap(lt, i);
                }
                lt += 1;
                i += 1;
            } else if val > widest_chan_mean {
                gt_weight += entry.weight as usize;
                self.entries.swap(gt, i);
                gt -= 1;
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
