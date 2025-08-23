#![forbid(unsafe_code)]

use core::borrow::Borrow;
use core::hash::Hash;
use core::marker::PhantomData;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::Arc;

pub trait InsertionIndex: Ord + Copy {
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
//------

#[cfg(target_pointer_width = "64")]
type UsizeAndU32 = u64;

#[cfg(target_pointer_width = "64")]
type UsizeAndU64 = u64;
// If there is an 128bit platform, add similar.
//
//------

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
    key_to_value_and_index: HashMap<CK, (V, I)>,
    insertion_index_to_key: BTreeMap<I, CK>,
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
            insertion_index_to_key: BTreeMap::new(),
            _phantom_key: PhantomData,
        }
    }

    pub fn put(&mut self, k: K, v: V) {
        debug_assert!(self.key_to_value_and_index.len() <= self.max_size);

        if let Some((_old_v, old_idx)) = self.key_to_value_and_index.remove(&k) {
            let old_key = self.insertion_index_to_key.remove(&old_idx);
            debug_assert!(old_key.is_some_and(|old| *old.borrow() == k));
        } else {
            if self.key_to_value_and_index.len() == self.max_size {
                // remove the least recently used
                let (oldest_idx, oldest_key) = self.insertion_index_to_key.pop_first().unwrap();
                let (_, oldest_idx_paired) = self
                    .key_to_value_and_index
                    .remove(oldest_key.borrow())
                    .unwrap();
                assert!(oldest_idx == oldest_idx_paired);
            }
        }
        let ck = CK::new(k);
        self.key_to_value_and_index
            .insert(ck.clone(), (v, self.next_insertion_index));
        self.insertion_index_to_key
            .insert(self.next_insertion_index, ck);

        self.next_insertion_index.increment();
    }

    pub fn get(&mut self, k: &K) -> Option<&V> {
        if let Some(value_and_index) = self.key_to_value_and_index.get_mut(k) {
            let existing_key = self
                .insertion_index_to_key
                .remove(&value_and_index.1)
                .unwrap();
            debug_assert!(*existing_key.borrow() == *k);
            self.insertion_index_to_key
                .insert(self.next_insertion_index, existing_key);

            value_and_index.1 = self.next_insertion_index;

            self.next_insertion_index.increment();
            return Some(&value_and_index.0);
        } else {
            return None;
        }
        /*
        // @TODO without k.clone()
        let mut value_and_idx = self
            .key_to_value_and_index
            .entry(k.clone());
        if let Entry::Occupied(mut occupied_entry) = value_and_idx {
            let existing_key = self.insertion_index_to_key.remove(&occupied_entry.get_mut().1).unwrap();
            debug_assert!( existing_key==*k );
            self.insertion_index_to_key.insert(self.next_insertion_index, existing_key);

            occupied_entry.get_mut().1 = self.next_insertion_index;

            self.next_insertion_index.increment();
            // "cannot return value referencing local variable `occupied_entry`":
            //
            // return Some(&(occupied_entry.get().0))
        } else {
            return None;
        }
        let value_option = self.key_to_value_and_index.get(k);
        return Some(&value_option.unwrap().0);
        return Some(&self.key_to_value_and_index.get(k).unwrap().0);
        //return Some(&self.key_to_value_and_index.get_mut(k).unwrap().0);
        */
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
