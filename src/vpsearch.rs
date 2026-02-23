use alloc::boxed::Box;
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use num_traits::Float;

use crate::ord_float::OrdFloat32;

#[derive(Clone)]
struct SearchIdx {
    ind: u8,
    data: [f32; 4],
}

struct SearchVisitor<'a> {
    ind: Option<&'a SearchIdx>,
    distance: f32,
    distance_sq: f32,
}

impl<'a> SearchVisitor<'a> {
    fn new() -> Self {
        Self {
            ind: None,
            distance: f32::MAX,
            distance_sq: f32::MAX,
        }
    }

    fn visit(&mut self, ind: &'a SearchIdx, distance_sq: f32) {
        if self.distance_sq > distance_sq {
            self.ind = Some(ind);
            self.distance = distance_sq.sqrt();
            self.distance_sq = distance_sq;
        }
    }
}

struct SearchNode {
    ind: SearchIdx,
    near: Option<Box<Self>>,
    far: Option<Box<Self>>,
    rest: Vec<SearchIdx>,
    radius: f32,
    radius_sq: f32,
}

impl SearchNode {
    fn new(indexes: &mut Vec<SearchIdx>, weights: &[f32]) -> Option<Box<Self>> {
        if indexes.is_empty() {
            return None;
        }

        if indexes.len() == 1 {
            let node = Self {
                ind: indexes.pop().unwrap(),
                near: None,
                far: None,
                rest: [].into(),
                radius: f32::MAX,
                radius_sq: f32::MAX,
            };

            return Some(Box::new(node));
        }

        // Find the vantage point by the maximum weight
        // and remove it from the list
        let vp_ind = indexes
            .iter()
            .enumerate()
            .map(|(i, ind)| (i, OrdFloat32::from(weights[usize::from(ind.ind)])))
            .max_by_key(|&(_, w)| w)
            .map(|(i, _)| indexes.swap_remove(i))
            .unwrap();

        indexes.sort_by_cached_key(|i| OrdFloat32::from(dist(&vp_ind.data, &i.data)));

        let (near, far, rest, radius_sq) = if indexes.len() < 7 {
            (None, None, indexes.to_vec(), f32::MAX)
        } else {
            let half_idx = indexes.len() / 2;
            let (near_indexes, far_indexes) = indexes.split_at_mut(half_idx);
            let radius_sq = dist(&vp_ind.data, &far_indexes[0].data);

            (
                Self::new(near_indexes.to_vec().as_mut(), weights),
                Self::new(far_indexes.to_vec().as_mut(), weights),
                [].into(),
                radius_sq,
            )
        };

        let node = Self {
            ind: vp_ind,
            near,
            far,
            rest,
            radius: radius_sq.sqrt(),
            radius_sq,
        };

        Some(Box::new(node))
    }

    fn visit<'a>(&'a self, pin: &[f32; 4], nearest: &mut SearchVisitor<'a>) {
        let distance_sq = dist(&self.ind.data, pin);

        nearest.visit(&self.ind, distance_sq);

        if !self.rest.is_empty() {
            for r in self.rest.iter() {
                let distance_sq = dist(&r.data, pin);
                nearest.visit(r, distance_sq);
            }

            return;
        }

        if distance_sq < self.radius_sq {
            if let Some(near) = &self.near {
                near.visit(pin, nearest);
            }
            if distance_sq.sqrt() >= self.radius - nearest.distance
                && let Some(far) = &self.far
            {
                far.visit(pin, nearest);
            }
        } else {
            if let Some(far) = &self.far {
                far.visit(pin, nearest);
            }
            if distance_sq.sqrt() <= self.radius + nearest.distance
                && let Some(near) = &self.near
            {
                near.visit(pin, nearest);
            }
        }
    }
}

pub(crate) struct SearchTree {
    root: Option<Box<SearchNode>>,
}

impl SearchTree {
    pub(crate) fn new(data: &[[f32; 4]], weights: &[f32]) -> Self {
        assert!(weights.len() >= data.len());
        assert!(data.len() <= 256);

        let mut indexes = data
            .iter()
            .enumerate()
            .map(|(i, &d)| SearchIdx {
                ind: i as u8,
                data: d,
            })
            .collect::<Vec<SearchIdx>>();

        let root = SearchNode::new(&mut indexes, weights);

        Self { root }
    }

    pub(crate) fn find_nearest(&self, pin: &[f32; 4]) -> (u8, [f32; 4], f32) {
        if let Some(vantage_point) = &self.root {
            let mut nearest = SearchVisitor::new();

            vantage_point.visit(pin, &mut nearest);

            if let Some(nearest_ind) = nearest.ind {
                (nearest_ind.ind, nearest_ind.data, nearest.distance)
            } else {
                (0, [0f32; 4], f32::MAX)
            }
        } else {
            (0, [0f32; 4], f32::MAX)
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn dist(c1: &[f32; 4], c2: &[f32; 4]) -> f32 {
    unsafe {
        use core::arch::x86_64::*;

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
        use core::arch::aarch64::*;

        let pc1 = vld1q_f32(c1.as_ptr());
        let pc2 = vld1q_f32(c2.as_ptr());

        let mut dist = vsubq_f32(pc1, pc2);
        dist = vmulq_f32(dist, dist);

        vaddvq_f32(dist)
    }
}

#[cfg(not(any(
    target_arch = "x86_64",
    all(target_arch = "aarch64", target_feature = "neon")
)))]
#[inline(always)]
fn dist(c1: &[f32; 4], c2: &[f32; 4]) -> f32 {
    (c1[0] - c2[0]).powi(2)
        + (c1[1] - c2[1]).powi(2)
        + (c1[2] - c2[2]).powi(2)
        + (c1[3] - c2[3]).powi(2)
}
