use std::cmp::{Ord,Ordering};

use crate::histogram::{Historgram, HistogramEntry};

pub struct Cluster {
    pub entries: Vec<HistogramEntry>,
    pub mean: [u8; 4],
    pub weight: usize,
    pub priority: u64,
    widest_chan: u8,
}

impl Ord for Cluster {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for Cluster {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Cluster {}

impl PartialEq for Cluster {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Cluster {
    pub fn new(entries: Vec<HistogramEntry>) -> Self {
        Self{
            entries: entries,
            mean: [0; 4],
            priority: 0,
            weight: 0,
            widest_chan: 0,
        }
    }

    pub fn from_histogram(hist: &Historgram) -> Self {
        let mut entries = Vec::with_capacity(hist.0.len());

        for entry in hist.0.values() {
            entries.push(*entry)
        }

        Self::new(entries)
    }

    pub fn calc_widest_and_priority(&mut self) {
        let mut diff_sum: [usize; 4] = [0; 4];

        for e in self.entries.iter() {
            let weight = e.weight as usize;

            diff_sum[0] += diff(e.color[0], self.mean[0]) as usize * weight;
            diff_sum[1] += diff(e.color[1], self.mean[1]) as usize * weight;
            diff_sum[2] += diff(e.color[2], self.mean[2]) as usize * weight;
            diff_sum[3] += diff(e.color[3], self.mean[3]) as usize * weight;
        }

        let mut chan = 0;
        let mut max_diff_sum = 0;

        for ch in 0..=3usize {
            let d = diff_sum[ch];

            if d > max_diff_sum {
                chan = ch;
                max_diff_sum = d;
            }
        }

        let chan_diff = max_diff_sum as f64 / self.weight as f64;

        self.priority = (chan_diff * (self.weight as f64).sqrt()) as u64;
        self.widest_chan = chan as u8;
    }

    pub fn split(&mut self) -> (Cluster, Cluster) {
        let widest_chan = self.widest_chan as usize;
        let widest_chan_mean = self.mean[widest_chan];

        let mut i: usize = 0;
        let mut lt: usize = 0;
        let mut gt: usize = self.entries.len() - 1;

        let mut lt_weight: usize = 0;
        let mut gt_weight: usize = 0;

        while i <= gt {
            let val = self.entries[i].color[widest_chan];

            if val < widest_chan_mean {
                if lt != i {
                    self.entries.swap(lt, i);
                }
                lt += 1;
                i += 1;
                lt_weight += self.entries[i].weight as usize;
            } else if val > widest_chan_mean {
                self.entries.swap(gt, i);
                gt -= 1;
                gt_weight += self.entries[i].weight as usize;
            } else {
                i += 1;
            }
        }

        let mut split_pos = i;
        if lt_weight < gt_weight {
            split_pos = lt;
        }

        let (sp1, sp2) = self.entries.split_at(split_pos);

        (Self::new(sp1.to_vec()), Self::new(sp2.to_vec()))
    }

    pub fn calc_mean_and_weight(&mut self) {
        self.weight = 0;

        if self.entries.is_empty() {
            self.mean = [0; 4];
            return
        }

        let mut rsum: usize = 0;
        let mut gsum: usize = 0;
        let mut bsum: usize = 0;
        let mut asum: usize = 0;

        for e in self.entries.iter() {
            let weight = e.weight as usize;

            rsum += e.color[0] as usize * weight;
            gsum += e.color[1] as usize * weight;
            bsum += e.color[2] as usize * weight;
            asum += e.color[3] as usize * weight;

            self.weight += weight;
        }

        self.mean = [
            (rsum / self.weight) as u8,
            (gsum / self.weight) as u8,
            (bsum / self.weight) as u8,
            (asum / self.weight) as u8,
        ]
    }
}

fn diff(a: u8, b: u8) -> u8 {
    if a > b {
        return a - b
    }

    b - a
}
