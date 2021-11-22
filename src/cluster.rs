use std::cmp::{Ord,Ordering};

use crate::color::Color;
use crate::image::Image;

pub struct Cluster {
    pub colors: Vec<[u8; 4]>,
    pub mean: Color,
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
    pub fn new(colors: Vec<[u8; 4]>) -> Self {
        Self{
            colors: colors,
            widest_chan: 0,
            mean: Color::default(),
            priority: 0,
        }
    }

    pub fn populate(image: &Image) -> Self {
        let mut colors = vec![];

        let mut ind = 0;
        colors.resize_with(image.width* image.height, || {
            let pix = &image.data[ind..ind+4];
            ind += 4;

            [pix[0], pix[1], pix[2], pix[3]]
        });

        Self::new(colors)
    }

    pub fn calc_mean_and_priority(&mut self) {
        if self.colors.is_empty() {
            self.mean = Color::default();
            self.priority = 0;
            return
        }

        self.calc_mean();

        let mut diff_sum: [usize; 4] = [0; 4];
        let mean = self.mean.as_slice();

        for c in self.colors.iter() {
            diff_sum[0] += diff(c[0], mean[0]) as usize;
            diff_sum[1] += diff(c[1], mean[1]) as usize;
            diff_sum[2] += diff(c[2], mean[2]) as usize;
            diff_sum[3] += diff(c[3], mean[3]) as usize;
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

        let chan_diff = max_diff_sum as f64 / self.colors.len() as f64;

        self.priority = (chan_diff * (self.colors.len() as f64).sqrt()) as u64;
        self.widest_chan = chan as u8;
    }

    pub fn split(&mut self) -> (Cluster, Cluster) {
        let widest_chan = self.widest_chan as usize;
        let widest_chan_mean = self.mean.as_slice()[widest_chan];

        let mut i: usize = 0;
        let mut lt: usize = 0;
        let mut gt: usize = self.colors.len() - 1;

        while i <= gt {
            let val = self.colors[i][widest_chan];

            if val < widest_chan_mean {
                if lt != i {
                    self.colors.swap(lt, i);
                }
                lt += 1;
                i += 1;
            } else if val > widest_chan_mean {
                self.colors.swap(gt, i);
                gt -= 1;
            } else {
                i += 1;
            }
        }

        let mut split_pos = i;
        if lt < self.colors.len() - i {
            split_pos = lt;
        }

        let (sp1, sp2) = self.colors.split_at(split_pos);

        (Self::new(sp1.to_vec()), Self::new(sp2.to_vec()))
    }

    pub fn calc_mean(&mut self) {
        let mut rsum: usize = 0;
        let mut gsum: usize = 0;
        let mut bsum: usize = 0;
        let mut asum: usize = 0;

        for c in self.colors.iter() {
            rsum += c[0] as usize;
            gsum += c[1] as usize;
            bsum += c[2] as usize;
            asum += c[3] as usize;
        }

        let count = self.colors.len();

        self.mean = Color::new(
            (rsum / count) as u8,
            (gsum / count) as u8,
            (bsum / count) as u8,
            (asum / count) as u8,
        )
    }
}

fn diff(a: u8, b: u8) -> u8 {
    if a > b {
        return a - b
    }

    b - a
}
