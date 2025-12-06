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

use std::cmp::{self};
use std::collections::BinaryHeap;
use std::collections::binary_heap::PeekMut;

use anyhow::Result;

use crate::key::KeySlice;

use super::StorageIterator;

/// An iterator together with its index in the memtables list
/// A lower index has higher priority for a key
struct IterWithIndex<I: StorageIterator> {
    pub iter: Box<I>,
    pub index: usize,
}

impl<I: StorageIterator> PartialEq for IterWithIndex<I> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}

impl<I: StorageIterator> Eq for IterWithIndex<I> {}

impl<I: StorageIterator> PartialOrd for IterWithIndex<I> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: StorageIterator> Ord for IterWithIndex<I> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.iter
            // compare the keys of the iterator first to make sure the lower key is returned first
            .key()
            .cmp(&other.iter.key())
            // for keys with the same value compare indices so that keys in the lower indexed iterator is return first
            .then(self.index.cmp(&other.index))
            // reverse to make the binary heap a min heap
            .reverse()
    }
}

/// Merge multiple iterators of the same type. If the same key occurs multiple times in some
/// iterators, prefer the one with smaller index.
pub struct MergeIterator<I: StorageIterator> {
    iters: BinaryHeap<IterWithIndex<I>>,
    current_iter: Option<IterWithIndex<I>>,
}

impl<I: StorageIterator> MergeIterator<I> {
    /// Create a MergeIterator from a vector of iterators
    pub fn create(iters: Vec<Box<I>>) -> Self {
        let mut heap = BinaryHeap::new();

        for (index, iter) in iters.into_iter().enumerate() {
            // Only add valid iterators to the heap because adding invalid
            // iterators to the heap will result in those invalid iterators'
            // key() method being called by the Ord::cmp method (see above)
            // This might not be valid to call on an invalid iterator.
            if iter.is_valid() {
                heap.push(IterWithIndex { iter, index });
            }
        }

        let current = heap.pop();
        MergeIterator {
            iters: heap,
            current_iter: current,
        }
    }

    /// Pops all
    fn pop_equal(&mut self, current_key: I::KeyType<'_>) -> Result<()> {
        Ok(())
    }
}

impl<I: 'static + for<'a> StorageIterator<KeyType<'a> = KeySlice<'a>>> StorageIterator
    for MergeIterator<I>
{
    type KeyType<'a> = KeySlice<'a>;

    fn key(&'_ self) -> KeySlice<'_> {
        self.current_iter
            .as_ref()
            .expect("key() called without checking is_valid()")
            .iter
            .key()
    }

    fn value(&self) -> &[u8] {
        self.current_iter
            .as_ref()
            .expect("value() called without checking is_valid()")
            .iter
            .value()
    }

    fn is_valid(&self) -> bool {
        self.current_iter
            .as_ref()
            .map(|iter| iter.iter.is_valid())
            .unwrap_or(false)
    }

    fn next(&mut self) -> Result<()> {
        if let Some(current_iter) = &mut self.current_iter {
            // Iterate over all the iterators in the binary heap that have the same key as the current iterator
            // and pop them off the heap if:
            // 1. Either calling next() returns an error
            // 2. Or the iterator has become invalid.
            // In the first case we also return an error immediately to the caller instead of continuing.
            // This is done to avoid returning duplicate keys from multiple iterators.
            while let Some(mut other_iter) = self.iters.peek_mut() {
                if current_iter.iter.key() == other_iter.iter.key() {
                    // If the peeked iterator's key is the same as the current iterator's key
                    if let e @ Err(_) = other_iter.iter.next() {
                        PeekMut::pop(other_iter);
                        return e;
                    }

                    if !other_iter.iter.is_valid() {
                        PeekMut::pop(other_iter);
                    }
                } else {
                    // break if the peeked iterator's key is not the same
                    break;
                }
            }

            // Move the current iterator to the next item
            current_iter.iter.next()?;

            // If the current iterator is invalid, pop it out of the heap and select the next one
            if !current_iter.iter.is_valid() {
                if let Some(iter) = self.iters.pop() {
                    *current_iter = iter;
                }
                return Ok(());
            }

            // Otherwise, compare with heap top and swap if necessary.
            if let Some(mut other_iter) = self.iters.peek_mut()
                && *current_iter < *other_iter
            {
                std::mem::swap(&mut *other_iter, current_iter);
            }
        }

        Ok(())
    }
}
