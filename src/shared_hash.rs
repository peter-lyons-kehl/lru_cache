use super::InsertionIndex;
use core::borrow::Borrow;
use core::hash::{BuildHasher, Hash, Hasher};
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct Idx<I: InsertionIndex> {
    /// The actual hash of [Idx] may NOT be same as `single_hash`, but it will be based on it.
    single_hash: u64,
    idx: I,
}
impl<I: InsertionIndex> Idx<I> {
    fn new(idx: I, single_hash: u64) -> Self {
        Self { idx, single_hash}
    }
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
        state.write_u64(self.single_hash);
    }
}

#[derive(PartialEq, Eq)]
struct Key<K> {
    /// The actual hash of [Key] may NOT be same as `single_hash`, but it will be based on it.
    single_hash: u64,
    key: K,
}
impl<K: Hash> Key<K> {
    fn new(key: K, single_hash: u64) -> Self {
        Self {
            key, single_hash
        }
    }
    /// We consume the hasher, so it's not reused accidentally.
    fn new_from_hasher<H: Hasher>(key: K, mut h: H) -> Self {
        key.hash(&mut h);
        Self::new( key, h.finish())
    }
}
impl<K: Hash> Hash for Key<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.single_hash);
    }
}

// @TODO?:
/// We don't store a single_hash, so that we can transmute &K to KeyRef<K>.
struct KeyRef<'a, K> {
    single_hash: u64, // @TODO Remove single_hash?
    key_ref: &'a K,
}
impl<'a, K: Hash> KeyRef<'a, K> {
    fn new(key_ref: &'a K, single_hash: u64) -> Self {
        Self {
            key_ref, single_hash
        }
    }
    /// We consume the hasher, so it's not reused accidentally.
    fn new_from_hasher<H: Hasher>(key_ref: &'a K, mut h: H) -> Self {
        key_ref.hash(&mut h);
        Self::new( key_ref, h.finish())
    }
}
impl<'a, K: Hash> Hash for KeyRef<'a, K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        /// The following is different to: self.single_hash.hash(state)
        state.write_u64(self.single_hash);
    }
}

/// A bi-modal wrapper. On its own it uses only `ck` part for [PartialEq] and [Hash]. However, see
/// trait for borrowing as comparable by `idx` part, too.
///
/// We intentionally do NOT implement `Borrow<K>``. We don't want to have
/// [DbCache::key_and_idx_to_value] keys accidentally compared to `K`, because
/// [DbCache::key_and_idx_to_value] doesn't use a hash of `K` - it uses its "double hash" instead.
struct KeyAndIdx<K, I: InsertionIndex> {
    key: Key<K>,
    idx: Idx<I>,
}
impl<K, I: InsertionIndex> KeyAndIdx<K, I> {
    fn new(key: Key<K>, idx: Idx<I>) -> Self {
        Self {
            key, idx
        }
    }
}

impl<K, I: InsertionIndex> Hash for KeyAndIdx<K, I> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.idx.single_hash);
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
impl<'a, K, I: InsertionIndex> Borrow<KeyRef<'a, K>> for KeyAndIdx<K, I> {
    fn borrow(&self) -> &KeyRef<'a, K> {
        &self.key
    }
}
impl<K, I: InsertionIndex> Borrow<Idx<I>> for KeyAndIdx<K, I> {
    fn borrow(&self) -> &Idx<I> {
        &self.idx
    }
}

pub struct DhCache<
    K,
    V,
    I: InsertionIndex,
    const MOST_RECENT_FAST: bool,
    const RECYCLE: bool,
> {
    max_size: usize,
    next_insertion_index: I,
    key_and_idx_to_value: HashMap<KeyAndIdx<K, I>, V>,
    /// Always sorted.
    indexes: Vec<Idx<I>>
}

impl<
        K: Hash + Eq,
        V,
        I: InsertionIndex,
        const MOST_RECENT_FAST: bool,
        const RECYCLE: bool,
    > DhCache<K, V, I, MOST_RECENT_FAST, RECYCLE>
{
    pub fn new(max_size: usize) -> Self {
        assert!(I::accommodates(max_size));
        Self {
            max_size,
            next_insertion_index: I::ZERO,
            key_and_idx_to_value: HashMap::with_capacity(max_size),
            indexes: Vec::with_capacity(max_size),
        }
    }

    pub fn put(&mut self, k: K, v: V) {
        debug_assert!(self.key_and_idx_to_value.len() <= self.max_size);
        let key = Key::new_from_hasher(k, self.key_and_idx_to_value.hasher().build_hasher());

        if let Some((old_key_and_idx, old_v)) = self.key_and_idx_to_value.remove_entry(&key) {
            let old_idx_and_key_pos = self
                .indexes
                .binary_search(&old_key_and_idx.idx)
                .unwrap();
            // We always remove the old entry, even if the storage is not full (to our capacity)
            // yet. We could store an Option, and set it to None, which would save the shifting of
            // the rest of items. However, that would help only while storage is not full. But, a
            // cache is beneficial/intended to use once it gets full, so we keep it simple.
            self.indexes.remove(old_idx_and_key_pos);
        } else {
            if self.key_and_idx_to_value.len() == self.max_size {
                // remove the least recently used
                let oldest_idx = self.indexes.remove(0);

                #[cfg(debug_assertions)] {} //@TODO
                let (oldest_key_and_idx, _oldest_value) = self
                    .key_and_idx_to_value
                    .remove_entry(&oldest_idx)
                    .unwrap();
            }
        }
        let idx = Idx::new(self.next_insertion_index, key.single_hash);

        let key_and_idx = KeyAndIdx::new(key, idx);
        self.key_and_idx_to_value
            .insert(key_and_idx, v);

        self.indexes
            .push(idx);

        self.next_insertion_index.increment();
    }

    pub fn get(&mut self, k: &K) -> Option<&V> {
        debug_assert!(self.key_and_idx_to_value.len() <= self.max_size);
        let key = Key::new_from_hasher(k, self.key_and_idx_to_value.hasher().build_hasher());

        if let Some((key_and_idx, v)) = self.key_and_idx_to_value.remove_entry(key) {
            let old_idx_pos = self
                .indexes
                .binary_search(&key_and_idx.idx)
                .unwrap();

            self.indexes.remove(old_idx_pos);

            let idx = Idx::new(self.next_insertion_index, key.single_hash);

            let key_and_idx = KeyAndIdx::new(key, idx);

            self.indexes.push(idx);

            // @TODO
            //
            //self.key_and_idx_to_value.insert(key_and_idx, v);

            self.next_insertion_index.increment();
            return None;
            // @TODO
            
            //return Some(&value_and_index.0);
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
