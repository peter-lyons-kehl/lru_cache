use core::hash::Hash;

pub mod shared_hash;
pub mod double_key;

trait InsertionIndex: Ord + Copy + Hash {
    const ZERO: Self;
    /** Maximum index. */
    const MAX: Self;
    fn increment(&mut self);
    fn accommodates(size: usize) -> bool;
}
impl InsertionIndex for u8 {
    const ZERO: Self = 0;
    const MAX: Self = u8::MAX;
    fn increment(&mut self) {
        *self += 1;
    }
    fn accommodates(size: usize) -> bool {
        Self::MAX as usize >= size
    }
}
impl InsertionIndex for u16 {
    const ZERO: Self = 0;
    const MAX: Self = u16::MAX;
    fn increment(&mut self) {
        *self += 1;
    }
    fn accommodates(size: usize) -> bool {
        Self::MAX as usize >= size
    }
}
#[cfg(any(target_pointer_width = "16", target_pointer_width = "32"))]
type UsizeAndU32 = u32;

#[cfg(target_pointer_width = "16")]
type UsizeAndU64 = u64;

#[cfg(target_pointer_width = "64")]
type UsizeAndU32 = u64;

#[cfg(target_pointer_width = "64")]
type UsizeAndU64 = u64;
// If there is an 128bit platform, add similar.

impl InsertionIndex for u32 {
    const ZERO: Self = 0;
    const MAX: Self = u32::MAX;
    fn increment(&mut self) {
        *self += 1;
    }
    fn accommodates(size: usize) -> bool {
        Self::MAX as UsizeAndU32 >= size as UsizeAndU32
    }
}
impl InsertionIndex for u64 {
    const ZERO: Self = 0;
    const MAX: Self = u64::MAX;
    fn increment(&mut self) {
        *self += 1;
    }
    fn accommodates(size: usize) -> bool {
        Self::MAX as UsizeAndU64 >= size as UsizeAndU64
    }
}
impl InsertionIndex for u128 {
    const ZERO: Self = 0;
    const MAX: Self = u128::MAX;
    fn increment(&mut self) {
        *self += 1;
    }
    fn accommodates(size: usize) -> bool {
        Self::MAX >= size as Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
