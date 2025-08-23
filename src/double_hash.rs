use super::InsertionIndex;
use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use std::collections::HashMap;

struct Idx<I: InsertionIndex> {
    /// The actual hash of [Idx] may NOT be same as `hash`, but it will be based on it.
    hash: u64,
    idx: I,
}
impl<I: InsertionIndex> PartialEq for Idx<I> {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
    fn ne(&self, other: &Self) -> bool {
        self.idx != other.idx
    }
}
impl<I: InsertionIndex> Eq for Idx<I> {}
impl<I: InsertionIndex> PartialOrd for Idx<I> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.idx.partial_cmp(&other.idx)
    }
    fn ge(&self, other: &Self) -> bool {
        self.idx.ge(&other.idx)
    }
    fn gt(&self, other: &Self) -> bool {
        self.idx.gt(&other.idx)
    }
    fn le(&self, other: &Self) -> bool {
        self.idx.le(&other.idx)
    }
    fn lt(&self, other: &Self) -> bool {
        self.idx.lt(&other.idx)
    }
}
impl<I: InsertionIndex> Ord for Idx<I> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.idx.cmp(&other.idx)
    }
}
impl<I: InsertionIndex + Hash> Hash for Idx<I> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

#[derive(PartialEq, Eq)]
struct Key<K> {
    /// The actual hash of [Idx] may NOT be same as `hash`, but it will be based on it.
    hash: u64,
    key: K,
}
impl<K: Hash> Hash for Key<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

/// A bi-modal wrapper. On its own it uses only `ck` part for [PartialEq] and [Hash]. However, see
/// trait for borrowing as comparable by `idx` part, too.
struct KeyAndIdx<K, I: InsertionIndex> {
    key: Key<K>,
    idx: Idx<I>,
}
impl<K, I: InsertionIndex> KeyAndIdx<K, I> {
    fn new() -> Self {
        panic!()
    }
}
type KeyU8 = Key<u8>;
type OptionKeyU8 = Option<KeyU8>; // size not optimized

type KeyVecU8 = Key<Vec<u8>>;
type OptionKeyVecU8 = Option<KeyVecU8>; // size optimized

type KeyU16AndIdxU8 = KeyAndIdx<u16, u8>;
type OptionKeyU16AndIdxU8 = Option<KeyU16AndIdxU8>; // size not optimized

type KeyVecU16AndIdxU8 = KeyAndIdx<Vec<u16>, u8>;
type OptionKeyVecU16AndIdxU8 = Option<KeyVecU16AndIdxU8>; // size optimized

impl<K, I: InsertionIndex> Hash for KeyAndIdx<K, I> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.idx.hash);
    }
}
impl<K: PartialEq, I: InsertionIndex> PartialEq for KeyAndIdx<K, I> {
    fn eq(&self, other: &Self) -> bool {
        debug_assert_eq!(self.key == other.key, self.idx == other.idx);
        self.key == other.key
    }
    fn ne(&self, other: &Self) -> bool {
        debug_assert_eq!(self.key != other.key, self.idx != other.idx);
        self.key != other.key
    }
}
impl<K: Eq, I: InsertionIndex> Eq for KeyAndIdx<K, I> {}

impl<K, I: InsertionIndex> Borrow<Key<K>> for KeyAndIdx<K, I> {
    fn borrow(&self) -> &Key<K> {
        &self.key
    }
}
impl<K, I: InsertionIndex> Borrow<Idx<I>> for KeyAndIdx<K, I> {
    fn borrow(&self) -> &Idx<I> {
        &self.idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
