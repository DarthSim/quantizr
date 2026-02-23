use core::cmp::Ordering;

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub(crate) struct OrdFloat32(f32);

impl From<f32> for OrdFloat32 {
    #[inline(always)]
    fn from(value: f32) -> Self {
        OrdFloat32(value)
    }
}

impl PartialOrd for OrdFloat32 {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    #[inline]
    fn gt(&self, other: &Self) -> bool {
        !self.le(other)
    }

    #[inline]
    fn ge(&self, other: &Self) -> bool {
        other.le(self)
    }

    #[inline]
    fn lt(&self, other: &Self) -> bool {
        !other.le(self)
    }

    #[inline]
    fn le(&self, other: &Self) -> bool {
        // We consider NaN to be less than any other value and equal to itself.
        // This meand that NaN is always less than or equal to any value.
        self.0.is_nan() || (self.0 <= other.0)
    }
}

impl Ord for OrdFloat32 {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        if self.lt(other) {
            Ordering::Less
        } else if self.gt(other) {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl PartialEq for OrdFloat32 {
    #[inline]
    fn eq(&self, other: &OrdFloat32) -> bool {
        if self.0.is_nan() {
            other.0.is_nan()
        } else {
            self.0 == other.0
        }
    }
}

impl Eq for OrdFloat32 {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ordfloat32_gt() {
        // Finite values
        assert!(OrdFloat32(2.0).gt(&OrdFloat32(1.0)));
        assert!(!OrdFloat32(1.0).gt(&OrdFloat32(2.0)));
        assert!(!OrdFloat32(1.0).gt(&OrdFloat32(1.0)));
        // NaN values
        assert!(OrdFloat32(1.0).gt(&OrdFloat32(f32::NAN)));
        assert!(!OrdFloat32(f32::NAN).gt(&OrdFloat32(1.0)));
        assert!(!OrdFloat32(f32::NAN).gt(&OrdFloat32(f32::NAN)));
        // Infinite values
        assert!(OrdFloat32(f32::INFINITY).gt(&OrdFloat32(1.0)));
        assert!(!OrdFloat32(1.0).gt(&OrdFloat32(f32::INFINITY)));
        assert!(!OrdFloat32(f32::INFINITY).gt(&OrdFloat32(f32::INFINITY)));
        // Negative infinite values
        assert!(OrdFloat32(1.0).gt(&OrdFloat32(f32::NEG_INFINITY)));
        assert!(!OrdFloat32(f32::NEG_INFINITY).gt(&OrdFloat32(1.0)));
        assert!(!OrdFloat32(f32::NEG_INFINITY).gt(&OrdFloat32(f32::NEG_INFINITY)));
        // Infinites vs NaN
        assert!(OrdFloat32(f32::INFINITY).gt(&OrdFloat32(f32::NAN)));
        assert!(!OrdFloat32(f32::NAN).gt(&OrdFloat32(f32::INFINITY)));
        // Negative infinites vs NaN
        assert!(OrdFloat32(f32::NEG_INFINITY).gt(&OrdFloat32(f32::NAN)));
        assert!(!OrdFloat32(f32::NAN).gt(&OrdFloat32(f32::NEG_INFINITY)));
    }

    #[test]
    fn test_ordfloat32_ge() {
        // Finite values
        assert!(OrdFloat32(2.0).ge(&OrdFloat32(1.0)));
        assert!(!OrdFloat32(1.0).ge(&OrdFloat32(2.0)));
        assert!(OrdFloat32(1.0).ge(&OrdFloat32(1.0)));
        // NaN values
        assert!(OrdFloat32(1.0).ge(&OrdFloat32(f32::NAN)));
        assert!(!OrdFloat32(f32::NAN).ge(&OrdFloat32(1.0)));
        assert!(OrdFloat32(f32::NAN).ge(&OrdFloat32(f32::NAN)));
        // Infinite values
        assert!(OrdFloat32(f32::INFINITY).ge(&OrdFloat32(1.0)));
        assert!(!OrdFloat32(1.0).ge(&OrdFloat32(f32::INFINITY)));
        assert!(OrdFloat32(f32::INFINITY).ge(&OrdFloat32(f32::INFINITY)));
        // Negative infinite values
        assert!(OrdFloat32(1.0).ge(&OrdFloat32(f32::NEG_INFINITY)));
        assert!(!OrdFloat32(f32::NEG_INFINITY).ge(&OrdFloat32(1.0)));
        assert!(OrdFloat32(f32::NEG_INFINITY).ge(&OrdFloat32(f32::NEG_INFINITY)));
        // Infinites vs NaN
        assert!(OrdFloat32(f32::INFINITY).ge(&OrdFloat32(f32::NAN)));
        assert!(!OrdFloat32(f32::NAN).ge(&OrdFloat32(f32::INFINITY)));
        // Negative infinites vs NaN
        assert!(OrdFloat32(f32::NEG_INFINITY).ge(&OrdFloat32(f32::NAN)));
        assert!(!OrdFloat32(f32::NAN).ge(&OrdFloat32(f32::NEG_INFINITY)));
    }

    #[test]
    fn test_ordfloat32_lt() {
        // Finite values
        assert!(OrdFloat32(1.0).lt(&OrdFloat32(2.0)));
        assert!(!OrdFloat32(2.0).lt(&OrdFloat32(1.0)));
        assert!(!OrdFloat32(1.0).lt(&OrdFloat32(1.0)));
        // NaN values
        assert!(OrdFloat32(f32::NAN).lt(&OrdFloat32(1.0)));
        assert!(!OrdFloat32(1.0).lt(&OrdFloat32(f32::NAN)));
        assert!(!OrdFloat32(f32::NAN).lt(&OrdFloat32(f32::NAN)));
        // Infinite values
        assert!(OrdFloat32(1.0).lt(&OrdFloat32(f32::INFINITY)));
        assert!(!OrdFloat32(f32::INFINITY).lt(&OrdFloat32(1.0)));
        assert!(!OrdFloat32(f32::INFINITY).lt(&OrdFloat32(f32::INFINITY)));
        // Negative infinite values
        assert!(OrdFloat32(f32::NEG_INFINITY).lt(&OrdFloat32(1.0)));
        assert!(!OrdFloat32(1.0).lt(&OrdFloat32(f32::NEG_INFINITY)));
        assert!(!OrdFloat32(f32::NEG_INFINITY).lt(&OrdFloat32(f32::NEG_INFINITY)));
        // Infinites vs NaN
        assert!(OrdFloat32(f32::NAN).lt(&OrdFloat32(f32::INFINITY)));
        assert!(!OrdFloat32(f32::INFINITY).lt(&OrdFloat32(f32::NAN)));
        // Negative infinites vs NaN
        assert!(OrdFloat32(f32::NAN).lt(&OrdFloat32(f32::NEG_INFINITY)));
        assert!(!OrdFloat32(f32::NEG_INFINITY).lt(&OrdFloat32(f32::NAN)));
    }

    #[test]
    fn test_ordfloat32_le() {
        // Finite values
        assert!(OrdFloat32(1.0).le(&OrdFloat32(2.0)));
        assert!(!OrdFloat32(2.0).le(&OrdFloat32(1.0)));
        assert!(OrdFloat32(1.0).le(&OrdFloat32(1.0)));
        // NaN values
        assert!(OrdFloat32(f32::NAN).le(&OrdFloat32(1.0)));
        assert!(!OrdFloat32(1.0).le(&OrdFloat32(f32::NAN)));
        assert!(OrdFloat32(f32::NAN).le(&OrdFloat32(f32::NAN)));
        // Infinite values
        assert!(OrdFloat32(1.0).le(&OrdFloat32(f32::INFINITY)));
        assert!(!OrdFloat32(f32::INFINITY).le(&OrdFloat32(1.0)));
        assert!(OrdFloat32(f32::INFINITY).le(&OrdFloat32(f32::INFINITY)));
        // Negative infinite values
        assert!(OrdFloat32(f32::NEG_INFINITY).le(&OrdFloat32(1.0)));
        assert!(!OrdFloat32(1.0).le(&OrdFloat32(f32::NEG_INFINITY)));
        assert!(OrdFloat32(f32::NEG_INFINITY).le(&OrdFloat32(f32::NEG_INFINITY)));
        // Infinites vs NaN
        assert!(OrdFloat32(f32::NAN).le(&OrdFloat32(f32::INFINITY)));
        assert!(!OrdFloat32(f32::INFINITY).le(&OrdFloat32(f32::NAN)));
        // Negative infinites vs NaN
        assert!(OrdFloat32(f32::NAN).le(&OrdFloat32(f32::NEG_INFINITY)));
        assert!(!OrdFloat32(f32::NEG_INFINITY).le(&OrdFloat32(f32::NAN)));
    }
}
