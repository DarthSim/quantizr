use std::cmp::Ordering;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub(crate) struct OrdFloat32(f32);

impl From<f32> for OrdFloat32 {
    #[inline(always)]
    fn from(value: f32) -> Self {
        debug_assert!(value.is_finite(), "OrdFloat32 only accepts finite values");
        OrdFloat32(value)
    }
}

impl Eq for OrdFloat32 {}

impl Ord for OrdFloat32 {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
    }
}
