use std::cmp::{Ord,Ordering};
use std::ops::Deref;
use std::os::raw::c_uchar;

use crate::color::Color;
use crate::image::Image;

pub struct Cluster {
    pub indexes: Vec<usize>,
    pub mean: Color,
    pub chan_range: c_uchar,
    widest_chan: u8,
}

impl Ord for Cluster {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.chan_range.cmp(&other.chan_range) {
            Ordering::Equal => self.indexes.len().cmp(&other.indexes.len()),
            o => o,
        }
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
        self.chan_range == other.chan_range &&
            self.indexes.len() == other.indexes.len()
    }
}

impl Cluster {
    pub fn new(indexes: Vec<usize>) -> Self {
        Self{
            indexes: indexes,
            widest_chan: 0,
            chan_range: 0,
            mean: Color::default(),
        }
    }

    pub fn populate(count: usize) -> Self {
        let mut indexes = Vec::<usize>::with_capacity(count);

        for ind in 0..count {
            indexes.push(ind)
        }

        Self::new(indexes)
    }

    pub fn calc_mean_and_range(&mut self, image: &Image) {
        if self.indexes.is_empty() {
            self.mean = Color::default();
            self.chan_range = 0;
            return
        }

        self.calc_mean(image);

        let image_data = image.data.deref();

        let mut max_diff: [c_uchar; 4] = [0; 4];
        let mean = self.mean.as_slice();

        for ind in self.indexes.iter() {
            for ch in 0..=3usize {
                let val = image_data[ind*4 + ch];
                let d = diff(val, mean[ch]);

                if d > max_diff[ch] {
                    max_diff[ch] = d
                }
            }
        }

        let mut chan = 0;
        let mut max = max_diff[0];

        for ch in 0..=3usize {
            let max_val = max_diff[ch];

            if max_val > max {
                chan = ch;
                max = max_val;
            }
        }

        self.widest_chan = chan as u8;
        self.chan_range = max;
    }

    pub fn split(&mut self, image: &Image) -> (Cluster, Cluster) {
        let image_data = image.data.deref();

        let mean = self.mean.as_slice();
        let widest_chan = self.widest_chan as usize;

        let mut i: usize = 0;
        let mut lt: usize = 0;
        let mut gt: usize = self.indexes.len() - 1;

        while i <= gt {
            let ind = self.indexes[i];
            let val = image_data[ind*4 + widest_chan];

            if val < mean[widest_chan] {
                if lt != i {
                    self.indexes.swap(lt, i);
                }
                lt += 1;
                i += 1;
            } else if val > mean[widest_chan] {
                self.indexes.swap(gt, i);
                gt -= 1;
            } else {
                i += 1;
            }
        }

        let mut split_pos = lt;
        if lt < self.indexes.len() - i {
            split_pos = i;
        }

        let (sp1, sp2) = self.indexes.split_at(split_pos);

        (Self::new(sp1.to_vec()), Self::new(sp2.to_vec()))
    }

    fn calc_mean(&mut self, image: &Image) {
        let image_data = image.data.deref();

        let mut rsum: usize = 0;
        let mut gsum: usize = 0;
        let mut bsum: usize = 0;
        let mut asum: usize = 0;

        for ind in self.indexes.iter() {
            rsum += image_data[ind*4 + 0] as usize;
            gsum += image_data[ind*4 + 1] as usize;
            bsum += image_data[ind*4 + 2] as usize;
            asum += image_data[ind*4 + 3] as usize;
        }

        let count = self.indexes.len();

        self.mean = Color::new(
            (rsum / count) as c_uchar,
            (gsum / count) as c_uchar,
            (bsum / count) as c_uchar,
            (asum / count) as c_uchar,
        )
    }
}

fn diff(a: c_uchar, b: c_uchar) -> c_uchar {
    if a > b {
        return a - b
    }

    b - a
}
