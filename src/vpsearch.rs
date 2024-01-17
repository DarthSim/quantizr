use std::cmp::Ordering;

#[derive(Clone,Copy)]
struct SearchIdx {
    ind: usize,
    distance_sq: f32,
    weight: f32,
}

struct SearchVisitor {
    ind: usize,
    distance: f32,
    distance_sq: f32,
}

impl SearchVisitor {
    fn new() -> Self {
        Self {
            ind: 0,
            distance: f32::MAX,
            distance_sq: f32::MAX,
        }
    }

    fn visit(&mut self, ind: usize, distance_sq: f32) {
        if self.distance_sq > distance_sq {
            self.ind = ind;
            self.distance = distance_sq.sqrt();
            self.distance_sq = distance_sq;
        }
    }
}

struct SearchNode {
    ind: usize,
    near: Option<Box<Self>>,
    far: Option<Box<Self>>,
    rest: Box<[SearchIdx]>,
    radius: f32,
    radius_sq: f32,
}

impl SearchNode {
    fn new(indexes: &mut [SearchIdx], data: &[[f32; 4]]) -> Option<Box<Self>> {
        if indexes.is_empty() {
            return None;
        }

        if indexes.len() == 1 {
            let node = Self{
                ind: indexes[0].ind,
                near: None,
                far: None,
                rest: [].into(),
                radius: f32::MAX,
                radius_sq: f32::MAX,
            };

            return Some(Box::new(node))
        }

        let vp_pos = indexes.iter().enumerate()
            .max_by(|&(_, a), &(_, b)| {
                a.weight.partial_cmp(&b.weight).unwrap_or(Ordering::Equal)
            })
            .map(|(i, _)| i )
            .unwrap();

        indexes.swap(0, vp_pos);

        let vp_ind = indexes[0].ind;
        let vp_data = &data[vp_ind];

        let indexes = &mut indexes[1..];

        for i in indexes.iter_mut() {
            i.distance_sq = dist(vp_data, &data[i.ind]);
        }

        indexes.sort_unstable_by(|a, b| {
            a.distance_sq.partial_cmp(&b.distance_sq).unwrap_or(Ordering::Equal)
        });

        let (near, far, rest, radius_sq) = if indexes.len() < 7 {
            let rest = &indexes[0..];
            (None, None, rest.into(), f32::MAX)
        } else {
            let half_idx = indexes.len()/2;
            let (near_indexes, far_indexes) = indexes.split_at_mut(half_idx);
            let radius_sq = far_indexes[0].distance_sq;

            (
                Self::new(near_indexes, data),
                Self::new(far_indexes, data),
                [].into(),
                radius_sq
            )
        };


        let node = Self{
            ind: vp_ind,
            near: near,
            far: far,
            rest: rest,
            radius: radius_sq.sqrt(),
            radius_sq: radius_sq,
        };

        Some(Box::new(node))
    }

    fn visit(&self, pin: &[f32; 4], data: &[[f32; 4]], nearest: &mut SearchVisitor) {
        let vp_data = &data[self.ind];
        let distance_sq = dist(vp_data, pin);

        nearest.visit(self.ind, distance_sq);

        if !self.rest.is_empty() {
            for r in self.rest.iter() {
                let r_data = &data[r.ind];
                let distance_sq = dist(r_data, pin);

                nearest.visit(r.ind, distance_sq);
            }

            return;
        }

        if distance_sq < self.radius_sq {
            if let Some(near) = &self.near {
                near.visit(pin, data, nearest);
            }
            if distance_sq.sqrt() >= self.radius - nearest.distance {
                if let Some(far) = &self.far {
                    far.visit(pin, data, nearest);
                }
            }
        } else {
            if let Some(far) = &self.far {
                far.visit(pin, data, nearest);
            }
            if distance_sq.sqrt() <= self.radius + nearest.distance {
                if let Some(near) = &self.near {
                    near.visit(pin, data, nearest);
                }
            }
        }
    }
}

pub(crate) struct SearchTree {
    root: Option<Box<SearchNode>>,
    min_data_len: usize,
}

impl SearchTree {
    pub(crate) fn new(data: &[[f32; 4]], weights: &[f32]) -> Self {
        assert!(weights.len() >= data.len());

        let mut indexes = (0..data.len()).map(|i| {
            SearchIdx{ind: i, distance_sq: 0f32, weight: weights[i]}
        }).collect::<Vec<SearchIdx>>();

        let root = SearchNode::new(indexes.as_mut_slice(), data);

        Self{ root: root, min_data_len: data.len()}
    }

    pub(crate) fn find_nearest(&self, pin: &[f32; 4], data: &[[f32; 4]]) -> (usize, f32) {
        assert!(data.len() >= self.min_data_len);

        if let Some(vantage_point) = &self.root {
            let mut nearest = SearchVisitor::new();
            vantage_point.visit(pin, data, &mut nearest);
            (nearest.ind, nearest.distance)
        } else {
            (0, f32::MAX)
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn dist(c1: &[f32; 4], c2: &[f32; 4]) -> f32 {
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

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
fn dist(c1: &[f32; 4], c2: &[f32; 4]) -> f32 {
    unsafe {
        use std::arch::aarch64::*;

        let pc1 = vld1q_f32(c1.as_ptr());
        let pc2 = vld1q_f32(c2.as_ptr());

        let mut dist = vsubq_f32(pc1, pc2);
        dist = vmulq_f32(dist, dist);

        vaddvq_f32(dist)
    }
}

#[cfg(not(any(target_arch = "x86_64", all(target_arch = "aarch64", target_feature = "neon"))))]
#[inline(always)]
fn dist(c1: &[f32; 4], c2: &[f32; 4]) -> f32 {
    (c1[0] - c2[0]).powi(2) +
    (c1[1] - c2[1]).powi(2) +
    (c1[2] - c2[2]).powi(2) +
    (c1[3] - c2[3]).powi(2)
}
