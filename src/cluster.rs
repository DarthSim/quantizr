use std::cmp::{Ord,Ordering};
use std::ops::Deref;
use std::os::raw::c_uchar;

use crate::image::Image;

#[repr(C)]
#[derive(Eq)]
pub struct Cluster {
    pub indexes: Vec<usize>,
    widest_chan: u8,
    pub chan_range: c_uchar,
}

impl Ord for Cluster {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.indexes.len().cmp(&other.indexes.len()) {
            Ordering::Equal => self.chan_range.cmp(&other.chan_range),
            o => o,
        }
    }
}

impl PartialOrd for Cluster {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Cluster {
    fn eq(&self, other: &Self) -> bool {
        self.indexes.len() == other.indexes.len() &&
            self.chan_range == other.chan_range
    }
}

impl Cluster {
    pub fn new(indexes: Vec<usize>) -> Self {
        Self{
            indexes: indexes,
            widest_chan: 0,
            chan_range: 0,
        }
    }

    pub fn populate(count: usize) -> Self {
        let mut indexes = Vec::<usize>::with_capacity(count);

        for ind in 0..count {
            indexes.push(ind)
        }

        Self::new(indexes)
    }

    pub fn set_color_range(&mut self, image: &Image) {
        let image_data = image.data.deref();

        let mut min_by_chan: [c_uchar; 4] = [255; 4];
        let mut max_by_chan: [c_uchar; 4] = [0; 4];

        for ind in self.indexes.iter() {
            for ch in 0..=3 {
                let val = image_data[ind*4 + ch as usize];

                if val < min_by_chan[ch as usize] {
                    min_by_chan[ch as usize] = val
                }
                if val > max_by_chan[ch as usize] {
                    max_by_chan[ch as usize] = val
                }
            }
        }

        let mut chan = 0;
        let mut min = min_by_chan[0];
        let mut max = max_by_chan[0];

        for ch in 0..=3 {
            let min_val = min_by_chan[ch as usize];
            let max_val = max_by_chan[ch as usize];

            if max_val - min_val > max-min {
                chan = ch;
                min = min_val;
                max = max_val;
            }
        }

        self.widest_chan = chan;
        self.chan_range = max - min;
    }

    pub fn median(&mut self, image: &Image) -> c_uchar {
        let image_data = image.data.deref();

        let mut values = Vec::<c_uchar>::with_capacity(self.indexes.len());

        for ind in self.indexes.iter() {
            values.push(image_data[ind*4 + self.widest_chan as usize]);
        }

        values.sort();

        let half = values.len() / 2;
        let mut median = values[half] as u16;

        if values.len() % 2 == 0 {
            median = (median + values[half-1] as u16) / 2;
        }

        return median as c_uchar;
    }

    pub fn split(&mut self, median: c_uchar, image: &Image) -> (Cluster, Cluster) {
        let image_data = image.data.deref();

        let mut i: usize = 0;
        let mut lt: usize = 0;
        let mut gt: usize = self.indexes.len() - 1;

        while i <= gt {
            let ind = self.indexes[i];
            let val = image_data[ind*4 + self.widest_chan as usize];

            if val < median {
                if lt != i {
                    self.indexes.swap(lt, i);
                }
                lt += 1;
                i += 1;
            } else if val > median {
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
}
