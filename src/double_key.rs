use super::InsertionIndex;
use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

struct Sealed;

pub trait CloneKey<K>: Borrow<K> + Clone {
    fn new(key: K) -> Self;
    #[allow(private_interfaces)]
    fn sealed() -> Sealed;
}
impl<K: Clone> CloneKey<K> for K {
    fn new(key: K) -> Self {
        key
    }
    #[allow(private_interfaces)]
    fn sealed() -> Sealed {
        Sealed
    }
}
impl<K> CloneKey<K> for Rc<K> {
    fn new(key: K) -> Self {
        Rc::new(key)
    }
    #[allow(private_interfaces)]
    fn sealed() -> Sealed {
        Sealed
    }
}
impl<K> CloneKey<K> for Arc<K> {
    fn new(key: K) -> Self {
        Arc::new(key)
    }
    #[allow(private_interfaces)]
    fn sealed() -> Sealed {
        Sealed
    }
}

/**
 * Like a tuple of `I` and `CK`, but using only `I` part for comparison, so that we don't need the
 * `CK` part when looking it up.
 */
struct IndexAndKey<K, I: InsertionIndex, CK: CloneKey<K>> {
    idx: I,
    ck: CK,
    _phantom_key: PhantomData<K>,
}
impl<K, I: InsertionIndex, CK: CloneKey<K>> IndexAndKey<K, I, CK> {
    fn new(idx: I, ck: CK) -> Self {
        Self {
            idx,
            ck,
            _phantom_key: PhantomData,
        }
    }
}
impl<K, I: InsertionIndex, CK: CloneKey<K>> PartialEq for IndexAndKey<K, I, CK> {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
    fn ne(&self, other: &Self) -> bool {
        self.idx != other.idx
    }
}
impl<K, I: InsertionIndex, CK: CloneKey<K>> Eq for IndexAndKey<K, I, CK> {}
impl<K, I: InsertionIndex, CK: CloneKey<K>> PartialOrd for IndexAndKey<K, I, CK> {
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
impl<K, I: InsertionIndex, CK: CloneKey<K>> Ord for IndexAndKey<K, I, CK> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.idx.cmp(&other.idx)
    }
}

pub struct LRUCache<
    K,
    V,
    I: InsertionIndex,
    CK: CloneKey<K>,
    const MOST_RECENT_FAST: bool,
    const RECYCLE: bool,
> {
    max_size: usize,
    next_insertion_index: I,
    //                      HashMap<  KandI,      V >
    //
    //                      HashMap< (K, I, u64), V >
    key_to_value_and_index: HashMap<CK, (V, I)>,
    /** Always sorted. */
    //                Vec< Idx >
    //
    //                Vec< (I, u64) >
    indexes_and_keys: Vec<IndexAndKey<K, I, CK>>,
    _phantom_key: PhantomData<K>,
}

impl<
        K: Hash + Eq,
        V,
        I: InsertionIndex,
        CK: CloneKey<K> + Hash + Eq,
        const MOST_RECENT_FAST: bool,
        const RECYCLE: bool,
    > LRUCache<K, V, I, CK, MOST_RECENT_FAST, RECYCLE>
{
    pub fn new(max_size: usize) -> Self {
        assert!(I::accommodates(max_size));
        Self {
            max_size,
            next_insertion_index: I::ZERO,
            key_to_value_and_index: HashMap::with_capacity(max_size),
            indexes_and_keys: Vec::with_capacity(max_size),
            _phantom_key: PhantomData,
        }
    }

    pub fn put(&mut self, k: K, v: V) {
        debug_assert!(self.key_to_value_and_index.len() <= self.max_size);

        if let Some((_old_v, old_idx)) = self.key_to_value_and_index.remove(&k) {
            let old_idx_and_key_pos = self
                .indexes_and_keys
                .binary_search_by_key(&old_idx, |idx_and_key| idx_and_key.idx)
                .unwrap();
            // We always remove the old entry, even if the storage is not full (to our capacity)
            // yet. We could store an Option, and set it to None, which would save the shifting of
            // the rest of items. However, that would help only while storage is not full. But, a
            // cache is beneficial/intended to use once it gets full, so we keep it simple.
            let old_key = self.indexes_and_keys.remove(old_idx_and_key_pos);
            debug_assert!(*old_key.ck.borrow() == k);
        } else {
            if self.key_to_value_and_index.len() == self.max_size {
                // remove the least recently used
                let oldest_idx_and_key = self.indexes_and_keys.remove(0);
                let (_, oldest_idx_paired) = self
                    .key_to_value_and_index
                    .remove(oldest_idx_and_key.ck.borrow())
                    .unwrap();
                assert!(oldest_idx_and_key.idx == oldest_idx_paired);
            }
        }
        let ck = CK::new(k);
        self.key_to_value_and_index
            .insert(ck.clone(), (v, self.next_insertion_index));
        self.indexes_and_keys
            .push(IndexAndKey::new(self.next_insertion_index, ck));

        self.next_insertion_index.increment();
    }

    pub fn get(&mut self, k: &K) -> Option<&V> {
        if let Some(value_and_index) = self.key_to_value_and_index.get_mut(k) {
            let old_idx_and_key_pos = self
                .indexes_and_keys
                .binary_search_by_key(&value_and_index.1, |idx_and_key| idx_and_key.idx)
                .unwrap();

            let existing_index_and_key = self.indexes_and_keys.remove(old_idx_and_key_pos);
            debug_assert!(*existing_index_and_key.ck.borrow() == *k);

            self.indexes_and_keys.push(IndexAndKey::new(
                self.next_insertion_index,
                existing_index_and_key.ck,
            ));

            value_and_index.1 = self.next_insertion_index;

            self.next_insertion_index.increment();
            return Some(&value_and_index.0);
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
