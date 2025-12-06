// Copyright (c) 2022-2025 Alex Chi Z
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

use std::ops::Bound;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use anyhow::Result;
use bytes::Bytes;
use crossbeam_skiplist::SkipMap;
use crossbeam_skiplist::map::Entry;
use ouroboros::self_referencing;

use crate::iterators::StorageIterator;
use crate::key::KeySlice;
use crate::table::SsTableBuilder;
use crate::wal::Wal;

/// A basic mem-table based on crossbeam-skiplist.
///
/// An initial implementation of memtable is part of week 1, day 1. It will be incrementally implemented in other
/// chapters of week 1 and week 2.
#[derive(Debug)]
pub struct MemTable {
    map: Arc<SkipMap<Bytes, Bytes>>,
    wal: Option<Wal>,
    id: usize,
    approximate_size: Arc<AtomicUsize>,
}

/// Create a bound of `Bytes` from a bound of `&[u8]`.
pub(crate) fn map_bound(bound: Bound<&[u8]>) -> Bound<Bytes> {
    match bound {
        Bound::Included(x) => Bound::Included(Bytes::copy_from_slice(x)),
        Bound::Excluded(x) => Bound::Excluded(Bytes::copy_from_slice(x)),
        Bound::Unbounded => Bound::Unbounded,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum GetResult {
    Tombstoned,
    Found(Bytes),
    Missing,
}

impl MemTable {
    /// Create a new mem-table.
    pub fn create(id: usize) -> Self {
        MemTable {
            map: Arc::new(SkipMap::new()),
            wal: None,
            id,
            approximate_size: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Create a new mem-table with WAL
    pub fn create_with_wal(_id: usize, _path: impl AsRef<Path>) -> Result<Self> {
        unimplemented!()
    }

    /// Create a memtable from WAL
    pub fn recover_from_wal(_id: usize, _path: impl AsRef<Path>) -> Result<Self> {
        unimplemented!()
    }

    pub fn for_testing_put_slice(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.put(key, value)
    }

    pub fn for_testing_get_slice(&self, key: &[u8]) -> GetResult {
        self.get(key)
    }

    pub fn for_testing_scan_slice(
        &self,
        lower: Bound<&[u8]>,
        upper: Bound<&[u8]>,
    ) -> MemTableIterator {
        // This function is only used in week 1 tests, so during the week 3 key-ts refactor, you do
        // not need to consider the bound exclude/include logic. Simply provide `DEFAULT_TS` as the
        // timestamp for the key-ts pair.
        self.scan(lower, upper)
    }

    /// Get a value by key.
    pub fn get(&self, key: &[u8]) -> GetResult {
        match self.map.get(key) {
            Some(value) if value.value().is_empty() => GetResult::Tombstoned,
            Some(value) => GetResult::Found(value.value().clone()),
            None => GetResult::Missing,
        }
    }

    /// Put a key-value pair into the mem-table.
    ///
    /// In week 1, day 1, simply put the key-value pair into the skipmap.
    /// In week 2, day 6, also flush the data to WAL.
    /// In week 3, day 5, modify the function to use the batch API.
    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let key = Bytes::copy_from_slice(key);
        let value = Bytes::copy_from_slice(value);
        let kv_size = key.len() + value.len();
        // If the key already exists in the map we double count the kv_size in the approximate_size
        // This can be refined later to be more accurate but is fine for now.
        self.approximate_size
            .fetch_add(kv_size, std::sync::atomic::Ordering::Relaxed);
        self.map.insert(key, value);

        Ok(())
    }

    /// Implement this in week 3, day 5; if you want to implement this earlier, use `&[u8]` as the key type.
    pub fn put_batch(&self, _data: &[(KeySlice, &[u8])]) -> Result<()> {
        unimplemented!()
    }

    pub fn sync_wal(&self) -> Result<()> {
        if let Some(ref wal) = self.wal {
            wal.sync()?;
        }
        Ok(())
    }

    /// Get an iterator over a range of keys.
    pub fn scan(&self, lower: Bound<&[u8]>, upper: Bound<&[u8]>) -> MemTableIterator {
        let (lower, upper) = (map_bound(lower), map_bound(upper));
        let mut iter = MemTableIteratorBuilder {
            map: self.map.clone(),
            iter_builder: |map| map.range((lower, upper)),
            item: KvPair::empty(),
        }
        .build();
        iter.next();
        iter
    }

    /// Flush the mem-table to SSTable. Implement in week 1 day 6.
    pub fn flush(&self, _builder: &mut SsTableBuilder) -> Result<()> {
        unimplemented!()
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn approximate_size(&self) -> usize {
        self.approximate_size
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Only use this function when closing the database
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

type SkipMapRangeIter<'a> =
    crossbeam_skiplist::map::Range<'a, Bytes, (Bound<Bytes>, Bound<Bytes>), Bytes, Bytes>;

/// A key-value pair
struct KvPair {
    key: Bytes,
    value: Bytes,
}

impl KvPair {
    fn empty() -> KvPair {
        KvPair {
            key: Bytes::new(),
            value: Bytes::new(),
        }
    }
}

/// An iterator over a range of `SkipMap`. This is a self-referential structure and please refer to week 1, day 2
/// chapter for more information.
///
/// This is part of week 1, day 2.
#[self_referencing]
pub struct MemTableIterator {
    /// Stores a reference to the skipmap.
    map: Arc<SkipMap<Bytes, Bytes>>,
    /// Stores a skipmap iterator that refers to the lifetime of `MemTableIterator` itself.
    /// This is an iterator over a (sub)range of items in the skip map
    #[borrows(map)]
    #[not_covariant]
    iter: SkipMapRangeIter<'this>,
    /// Stores the current key-value pair.
    item: KvPair,
}

impl MemTableIterator {
    /// Returns a valid key-value pair if entry is Some(_) and an empty KvPair otherwise
    fn entry_to_item(entry: Option<Entry<'_, Bytes, Bytes>>) -> KvPair {
        entry
            .map(|x| KvPair {
                key: x.key().clone(),
                value: x.value().clone(),
            })
            .unwrap_or_else(KvPair::empty)
    }

    /// Tries to retrieve the next item from the underlying skip map's iterator and stores it in the item field.
    /// If no item is found the item contains an empty Bytes pair.
    fn next(&mut self) {
        // The `with_iter_mut` method is generated by the ouroboros crate and give us a mutable reference to the [`MemTableIterator::iter`] field
        // Similarly the `with_item_mut` method is also generated

        // We call the `next` method on  this iterator and get the next item
        let next_item = self.with_iter_mut(|iter| MemTableIterator::entry_to_item(iter.next()));

        // We then store this next_item inside [`MemTableIterator::item`]
        self.with_item_mut(|item| *item = next_item);
    }
}

impl StorageIterator for MemTableIterator {
    type KeyType<'a> = KeySlice<'a>;

    fn value(&self) -> &[u8] {
        // Return the value from the key-value pair stored in the item
        &self.borrow_item().value
    }

    fn key(&'_ self) -> KeySlice<'_> {
        // Return the key from the key-vaue pair stored in the key
        KeySlice::from_slice(&self.borrow_item().key)
    }

    fn is_valid(&self) -> bool {
        // If the key is not empty the iterator is valid because the item stores a valid key-value pair in that case
        !self.borrow_item().key.is_empty()
    }

    fn next(&mut self) -> Result<()> {
        // Call the underlying iterator to move to the next item in the iterator
        MemTableIterator::next(self);
        Ok(())
    }
}
