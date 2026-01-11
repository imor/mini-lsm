<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# Snapshot Read — Memtable and Timestamps

In this chapter, you will:

- Refactor your memtable/WAL to store multiple versions of a key.
- Implement the new engine write path to assign each key a timestamp.
- Make your compaction process aware of multi-version keys.
- Implement the new engine read path to return the latest version of a key.

During the refactor, you might need to change some function signatures from `&self` to `self: &Arc<Self>` where necessary.

To run the test cases:

```
cargo x copy-test --week 3 --day 2
cargo x scheck
```

**Note: You also need to pass everything up to 2.4 after finishing this chapter.**

## Task 1: MemTable, Write-Ahead Log, and Read Path

In this task, modify:

```
src/wal.rs
src/mem_table.rs
src/lsm_storage.rs
```

We have already updated most keys in the engine to `KeySlice`, which contains a bytes key and a timestamp. However, some parts of the system still do not consider timestamps. In this task, modify your memtable and WAL implementation to take timestamps into account.

First, change the type of the `SkipMap` stored in your memtable.

```rust,no_run
pub struct MemTable {
    // map: Arc<SkipMap<Bytes, Bytes>>,
    map: Arc<SkipMap<KeyBytes, Bytes>>, // Bytes -> KeyBytes
    // ...
}
```

After that, fix remaining compiler errors to complete this task.

**MemTable::get**

We keep the `get` interface so test cases can probe a specific version of a key in the memtable. This interface should not be used in your read path after finishing this task. Given that we store `KeyBytes` (`(Bytes, u64)`) in the skiplist while the user probes a `KeySlice` (`(&[u8], u64)`), we need a way to convert the latter into a reference to the former to retrieve data from the skiplist.

To do this, you may use unsafe code to force-cast the `&[u8]` to be static and use `Bytes::from_static` to create a bytes object from a static slice. This is sound because `Bytes` does not free memory for static slices.

<details>

<summary>Spoilers: Convert u8 slice to Bytes</summary>

```rust,no_run
Bytes::from_static(unsafe { std::mem::transmute(key.key_ref()) })
```

</details>

Previously this was unnecessary because `Bytes` implements `Borrow<[u8]>`.

**MemTable::put**

Change the signature to `fn put(&self, key: KeySlice, value: &[u8])`, and convert a `KeySlice` to `KeyBytes` in your implementation.

**MemTable::scan**

Change the signature to `fn scan(&self, lower: Bound<KeySlice>, upper: Bound<KeySlice>) -> MemTableIterator`. Convert `KeySlice` to `KeyBytes` and use them as `SkipMap::range` parameters.

**MemTable::flush**

Instead of using a default timestamp, use the key’s timestamp when flushing the memtable to SSTs.

**MemTableIterator**

It should now store `(KeyBytes, Bytes)`, and the returned key type should be `KeySlice`.

**Wal::recover** and **Wal::put**

The write-ahead log should now accept a key slice instead of a user key slice. When serializing and deserializing WAL records, write the timestamp into the record and compute the checksum over the timestamp and all other fields.

The WAL format is as follows:

```
| key_len (u16, excludes ts length) | key | ts (u64) | value_len (u16) | value | checksum (u32) |
```

**LsmStorageInner::get**

Previously, `get` first probed the memtables and then scanned the SSTs. Now that the memtable uses the new key+timestamp APIs, re-implement `get`. The easiest way is to create a merge iterator over memtables, immutable memtables, L0 SSTs, and lower-level SSTs — the same as in `scan` — and apply Bloom-filter checks for SSTs.

**LsmStorageInner::scan**

Incorporate the new memtable APIs, and set the scan range to `(user_key_begin, TS_RANGE_BEGIN)` and `(user_key_end, TS_RANGE_END)`. Note: when handling exclusive boundaries, advance to the next user key, not another version of the same key.

## Task 2: Write Path

In this task, modify:

```
src/lsm_storage.rs
```

`LsmStorageInner` has an `mvcc` field that includes all data structures used for multi-version concurrency control this week. When you open a directory and initialize the storage engine, initialize that structure.

In `write_batch`, obtain a commit timestamp for all keys in the batch. Use `self.mvcc().latest_commit_ts() + 1` at the beginning, and call `self.mvcc().update_commit_ts(ts)` at the end to advance the next commit timestamp. To ensure batches get unique timestamps and new keys are placed on top of old ones, acquire the write lock `self.mvcc().write_lock.lock()` at the start so only one thread writes at a time.

## Task 3: MVCC Compaction

In this task, modify:

```
src/compact.rs
```

Previously, compaction kept only the latest version of a key and removed keys at the bottom level if they were deleted. With MVCC, keys have timestamps, and we cannot use the same logic for compaction.

In this chapter, remove the deletion logic. You may ignore `compact_to_bottom_level` for now, and you should keep ALL versions of a key during compaction.

Also, implement compaction so that all versions of the same key are placed in the same SST file, even if it exceeds the SST size limit. This ensures that if a key is found in an SST in a level, it will not appear in other SSTs in that level, simplifying many parts of the system.

## Task 4: LSM Iterator

In this task, modify:

```
src/lsm_iterator.rs
```

In the previous chapter, we implemented the LSM iterator to treat the same key with different timestamps as different keys. Now, refactor the LSM iterator to return only the latest version of a key when multiple versions are retrieved from the child iterator.

Record `prev_key` in the iterator. If you already returned the latest version of a key to the user, skip older versions and proceed to the next key.

At this point, you should pass all tests in previous chapters except the persistence tests (2.5 and 2.6).

## Test Your Understanding

- What is the difference between `get` in the MVCC engine and in the engine you built in Week 2?
- In Week 2, `get` stopped at the first memtable/level where a key was found. Can you do the same in the MVCC version?
- How do you convert `KeySlice` to `&KeyBytes`? Is it safe/sound?
- Why do we need to take a write lock in the write path?

We do not provide reference answers to these questions. Feel free to discuss them in the Discord community.

## Bonus Tasks

- **Early Stop for Memtable Gets**. Instead of creating a merge iterator over all memtables and SSTs, implement `get` as follows: if you find a version of a key in the memtable, stop searching. The same applies to SSTs.

{{#include copyright.md}}
