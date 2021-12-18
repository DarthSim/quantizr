use std::collections::HashMap;

use crate::image::Image;

macro_rules! hist_key {
    ($c: expr) => {
        $c[0] as usize +
        $c[1] as usize * 256 +
        $c[2] as usize * 256 * 256 +
        $c[3] as usize * 256 * 256 * 256
    };
}

#[derive(Clone,Copy)]
pub struct HistogramEntry {
    pub color: [u8; 4],
    pub weight: u32,
}

pub struct Historgram(pub HashMap<usize, HistogramEntry>);

impl Historgram {
    pub fn new() -> Self {
        Self{0: HashMap::new()}
    }

    pub fn add_image(&mut self, image: &Image) {
        for ind in (0..image.width*image.height*4).step_by(4) {
            let pix = &image.data[ind..ind+4];
            let color = [pix[0], pix[1], pix[2], pix[3]];

            let key = hist_key!(color);

            if let Some(e) = self.0.get_mut(&key) {
                e.weight += 1;
            } else {
                self.0.insert(key, HistogramEntry{
                    color: color,
                    weight: 1,
                });
            }
        }
    }
}
