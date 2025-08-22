use core::hash::Hash;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use core::borrow::Borrow;

pub trait InsertionIndex: Ord + Copy {
    const ZERO: Self;
    const MAX: Self;
    fn increment(&mut self);
}
impl InsertionIndex for u8 {
    const ZERO: Self = 0;
    const MAX: Self = u8::MAX;
    fn increment(&mut self) {
        *self += 1;
    }
}
impl InsertionIndex for u16 {
    const ZERO: Self = 0;
    const MAX: Self = u16::MAX;
    fn increment(&mut self) {
        *self += 1;
    }
}
impl InsertionIndex for u32 {
    const ZERO: Self = 0;
    const MAX: Self = u32::MAX;
    fn increment(&mut self) {
        *self += 1;
    }
}
impl InsertionIndex for u64 {
    const ZERO: Self = 0;
    const MAX: Self = u64::MAX;
    fn increment(&mut self) {
        *self += 1;
    }
}
impl InsertionIndex for u128 {
    const ZERO: Self = 0;
    const MAX: Self = u128::MAX;
    fn increment(&mut self) {
        *self += 1;
    }
}

// @TODO sealed trait
pub trait WrapKey<K> : Borrow<K> {
    fn new(key: K) -> Self;
}
impl <K> WrapKey<K> for K {
    fn new(key: K) -> Self {
        key
    }
}
impl <K> WrapKey<K> for Rc<K> {
    fn new(key: K) -> Self {
        Rc::new(key)
    }
}

pub struct LRUCache<K, V, I: InsertionIndex> {
    max_size: usize,
    next_insertion_index: I,
    key_to_value_and_index: HashMap<K, (V, I)>,
    insertion_index_to_key: BTreeMap<I, K>,
}

impl<K: Clone, V, I: InsertionIndex> Borrow<LRUCache<K, V, I>> for u8 {
    fn borrow(&self) -> &LRUCache<K, V, I> { panic!(); }
}
impl<K: Clone, V, I: InsertionIndex> Borrow<u8> for LRUCache<K, V, I> {
    fn borrow(&self) -> &u8 { panic!(); }
}
// #[repr(transparent)]

impl<K: Hash + Eq + Clone, V, I: InsertionIndex> LRUCache<K, V, I> {
    pub fn new(max_size: usize) -> Self {
        assert!(max_size > 0);
        Self {
            max_size,
            next_insertion_index: I::ZERO,
            key_to_value_and_index: HashMap::with_capacity(max_size),
            insertion_index_to_key: BTreeMap::new(),
        }
    }

    pub fn put(&mut self, k: K, v: V) {
        debug_assert!(self.key_to_value_and_index.len() <= self.max_size);

        if let Some((_old_v, old_idx)) = self.key_to_value_and_index.remove(&k) {
            let old_key = self.insertion_index_to_key.remove(&old_idx);
            debug_assert!(old_key.is_some_and(|old| old == k));
        } else {
            if self.key_to_value_and_index.len() == self.max_size {
                // remove the least recently used
                let (oldest_idx, oldest_key) = self.insertion_index_to_key.pop_first().unwrap();
                let (_, oldest_idx_paired) = self.key_to_value_and_index.remove(&oldest_key).unwrap();
                assert!(oldest_idx == oldest_idx_paired);
            }
        }
        let k_clone = k.clone();
        self.key_to_value_and_index
            .insert(k, (v, self.next_insertion_index));
        self.insertion_index_to_key
            .insert(self.next_insertion_index, k_clone);

        self.next_insertion_index.increment();
    }

    pub fn get(&mut self, k: &K) -> Option<&V> {
        if let Some(value_and_index) = self.key_to_value_and_index.get_mut(k) {
            let existing_key = self
                .insertion_index_to_key
                .remove(&value_and_index.1)
                .unwrap();
            debug_assert!(existing_key == *k);
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
