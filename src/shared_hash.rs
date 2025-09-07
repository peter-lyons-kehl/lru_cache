use super::InsertionIndex;
use core::borrow::Borrow;
use core::hash::{BuildHasher, Hash, Hasher};
use core::mem;
use hash_injector::{Flags, SignalledInjectionBuildHasher};
use std::collections::HashMap;
use std::hash::RandomState;

#[derive(Clone, Copy)]
struct Idx<I: InsertionIndex, const F: Flags> {
    hash: u64,
    idx: I,
}
impl<I: InsertionIndex, const F: Flags> Idx<I, F> {
    fn new(idx: I, hash: u64) -> Self {
        Self { idx, hash }
    }
}
impl<I: InsertionIndex, const F: Flags> PartialEq for Idx<I, F> {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
    fn ne(&self, other: &Self) -> bool {
        self.idx != other.idx
    }
}
impl<I: InsertionIndex, const F: Flags> Eq for Idx<I, F> {}
impl<I: InsertionIndex, const F: Flags> PartialOrd for Idx<I, F> {
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
impl<I: InsertionIndex, const F: Flags> Ord for Idx<I, F> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.idx.cmp(&other.idx)
    }
}
impl<I: InsertionIndex + Hash, const F: Flags> Hash for Idx<I, F> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_injector::signal_inject_hash::<H, F>(state, self.hash);
    }
}

#[derive(PartialEq, Eq)]
struct Key<K, const F: Flags> {
    /// `hash` is listed before `k`, so that it can short-circuit the derived [PartialEq]
    /// implementation by comparing `hash` first.
    hash: u64,
    k: K,
}
impl<K: Hash, const F: Flags> Key<K, F> {
    fn new(k: K, hash: u64) -> Self {
        Self { k, hash }
    }
    /// We consume the hasher, so that it's not reused accidentally.
    fn new_from_hasher<H: Hasher>(key: K, mut h: H) -> Self {
        key.hash(&mut h);
        Self::new(key, h.finish())
    }
}
impl<K: Hash, const F: Flags> Hash for Key<K, F> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_injector::signal_inject_hash::<H, F>(state, self.hash);
    }
}

/// A bi-modal wrapper. On its own it uses only `ck` part for [PartialEq] and [Hash]. However, see
/// trait for borrowing as comparable by `idx` part, too.
///
/// We intentionally do NOT implement `Borrow<K>``. We don't want to have
/// [DbCache::key_and_idx_to_value] keys accidentally compared to `K`, because
/// [DbCache::key_and_idx_to_value] doesn't use a hash of `K` - it uses its "double hash" instead.
struct KeyAndIdx<K, I: InsertionIndex, const F: Flags> {
    key: Key<K, F>,
    idx: Idx<I, F>,
}
impl<K, I: InsertionIndex, const F: Flags> KeyAndIdx<K, I, F> {
    fn new(key: Key<K, F>, idx: Idx<I, F>) -> Self {
        Self { key, idx }
    }
}

impl<K, I: InsertionIndex, const F: Flags> Hash for KeyAndIdx<K, I, F> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_injector::signal_inject_hash::<H, F>(state, self.idx.hash);
    }
}
impl<K: PartialEq, I: InsertionIndex, const F: Flags> PartialEq for KeyAndIdx<K, I, F> {
    fn eq(&self, other: &Self) -> bool {
        debug_assert_eq!(self.key == other.key, self.idx == other.idx);
        self.key == other.key
    }
    fn ne(&self, other: &Self) -> bool {
        debug_assert_eq!(self.key != other.key, self.idx != other.idx);
        self.key != other.key
    }
}
impl<K: Eq, I: InsertionIndex, const F: Flags> Eq for KeyAndIdx<K, I, F> {}

impl<K, I: InsertionIndex, const F: Flags> Borrow<Key<K, F>> for KeyAndIdx<K, I, F> {
    fn borrow(&self) -> &Key<K, F> {
        &self.key
    }
}
impl<K, I: InsertionIndex, const F: Flags> Borrow<Idx<I, F>> for KeyAndIdx<K, I, F> {
    fn borrow(&self) -> &Idx<I, F> {
        &self.idx
    }
}

// @TODO move PartialEq, Eq and Hash to #[derive()]
/// Needed, because we can't implement both `Borrow<Idx<I>>` and `Borrow<K>` for `KeyAndIdx<K, I>`,
/// as they could conflict.
#[repr(transparent)]
struct Kwrap<K> {
    k: K,
}
impl<K: PartialEq> PartialEq for Kwrap<K> {
    fn eq(&self, other: &Self) -> bool {
        self.k == other.k
    }
    fn ne(&self, other: &Self) -> bool {
        self.k != other.k
    }
}
impl<K: Eq> Eq for Kwrap<K> {}
impl<K: Hash> Hash for Kwrap<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.k.hash(state)
    }
}

impl<'a, K, I: InsertionIndex, const F: Flags> Borrow<Kwrap<K>> for KeyAndIdx<K, I, F> {
    fn borrow(&self) -> &Kwrap<K> {
        unsafe { mem::transmute(&self.key.k) }
    }
}

type SignalledBuildHasher<const F: Flags> =
    SignalledInjectionBuildHasher<<RandomState as BuildHasher>::Hasher, RandomState, F>;
pub struct DhCache<
    K,
    V,
    I: InsertionIndex,
    const MOST_RECENT_FAST: bool,
    const RECYCLE: bool,
    const F: Flags,
> {
    max_size: usize,
    next_insertion_index: I,
    key_and_idx_to_value: HashMap<KeyAndIdx<K, I, F>, V, SignalledBuildHasher<F>>,
    /// Always sorted.
    indexes: Vec<Idx<I, F>>,
}

impl<
    K: Hash + Eq,
    V,
    I: InsertionIndex,
    const MOST_RECENT_FAST: bool,
    const RECYCLE: bool,
    const F: Flags,
> DhCache<K, V, I, MOST_RECENT_FAST, RECYCLE, F>
{
    pub fn new(max_size: usize) -> Self {
        assert!(I::accommodates(max_size));

        let random_state = RandomState::new();
        let build_hasher = SignalledInjectionBuildHasher::new(random_state);
        Self {
            max_size,
            next_insertion_index: I::ZERO,
            key_and_idx_to_value: HashMap::with_capacity_and_hasher(max_size, build_hasher),
            indexes: Vec::with_capacity(max_size),
        }
    }

    pub fn put(&mut self, k: K, v: V) {
        debug_assert!(self.key_and_idx_to_value.len() <= self.max_size);
        debug_assert_eq!(self.key_and_idx_to_value.len(), self.indexes.len());

        let key = Key::new_from_hasher(k, self.key_and_idx_to_value.hasher().build_hasher());

        if let Some((old_key_and_idx, _old_v)) = self.key_and_idx_to_value.remove_entry(&key) {
            let old_idx_and_key_pos = self.indexes.binary_search(&old_key_and_idx.idx).unwrap();
            // We always remove the old entry, even if the storage is not full (to our capacity)
            // yet. We could store an Option, and set it to None, which would save the shifting of
            // the rest of items. However, that would help only while storage is not full. But, a
            // cache is beneficial/intended to use once it gets full, so we keep it simple.
            self.indexes.remove(old_idx_and_key_pos);
        } else {
            if self.key_and_idx_to_value.len() == self.max_size {
                // remove the least recently used
                let oldest_idx = self.indexes.remove(0);

                #[cfg(debug_assertions)]
                {} //@TODO
                let (oldest_key_and_idx, _oldest_value) =
                    self.key_and_idx_to_value.remove_entry(&oldest_idx).unwrap();
            }
        }
        let idx = Idx::new(self.next_insertion_index, key.hash);

        let key_and_idx = KeyAndIdx::new(key, idx);
        self.key_and_idx_to_value.insert(key_and_idx, v);

        self.indexes.push(idx);

        self.next_insertion_index.increment();
    }

    pub fn get(&mut self, k: &K) -> Option<&V> {
        debug_assert!(self.key_and_idx_to_value.len() <= self.max_size);
        debug_assert_eq!(self.key_and_idx_to_value.len(), self.indexes.len());

        let k_wrap: &Kwrap<K> = unsafe { mem::transmute(k) };

        if let Some((mut key_and_idx, v)) =
            self.key_and_idx_to_value.remove_entry(k_wrap /*key*/)
        {
            let old_idx_pos = self.indexes.binary_search(&key_and_idx.idx).unwrap();

            self.indexes.remove(old_idx_pos);

            //let key = Key::new_from_hasher(k, self.key_and_idx_to_value.hasher().build_hasher());
            key_and_idx.idx.idx = self.next_insertion_index;

            let idx = Idx::new(self.next_insertion_index, key_and_idx.idx.hash);
            self.indexes.push(idx);

            self.key_and_idx_to_value.insert(key_and_idx, v);
            self.next_insertion_index.increment();

            // We don't perform .get(k) here, because that would re-calculate the hash.
            self.key_and_idx_to_value.get(&idx)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
