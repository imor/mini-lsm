<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# Snapshot Read — Engine Read Path and Transaction API

In this chapter, you will:

- Finish the read path based on the previous chapter to support snapshot reads.
- Implement the transaction API to support snapshot reads.
- Implement the engine recovery process to correctly restore the commit timestamp.

By the end of this day, your engine can provide a consistent view of the storage key space.

During the refactor, you might need to change some function signatures from `&self` to `self: &Arc<Self>` where necessary.

To run the test cases:

```
cargo x copy-test --week 3 --day 3
cargo x scheck
```

**Note: After finishing this chapter, you also need to pass test cases for 2.5 and 2.6.**

## Task 1: LSM Iterator with Read Timestamp

Goal example:

```rust,no_run
let snapshot1 = engine.new_txn();
// write something to the engine
let snapshot2 = engine.new_txn();
// write something to the engine
snapshot1.get(/* ... */); // we can retrieve a consistent snapshot of a previous state of the engine
```

To achieve this, record the read timestamp (the latest committed timestamp) when creating the transaction. During reads, return only versions with timestamps less than or equal to the read timestamp.

In this task, modify:

```
src/lsm_iterator.rs
```

Record a read timestamp in `LsmIterator`:

```rust,no_run
impl LsmIterator {
    pub(crate) fn new(
        iter: LsmIteratorInner,
        end_bound: Bound<Bytes>,
        read_ts: u64,
    ) -> Result<Self> {
        // ...
    }
}
```

Update `next()` to skip versions newer than `read_ts` and return the latest version at or below `read_ts` for each user key.

## Task 2: Multi-Version Scan and Get

In this task, modify:

```
src/mvcc.rs
src/mvcc/txn.rs
src/lsm_storage.rs
```

Now that `read_ts` exists in the LSM iterator, implement `scan` and `get` on the transaction so you can read data at a specific point in time.

We recommend creating helper functions such as `scan_with_ts(/* original parameters */, read_ts: u64)` and `get_with_ts` in `LsmStorageInner`. Implement the original engine `get`/`scan` by creating a transaction (snapshot) and performing the operation over that transaction. The call path would be:

```
LsmStorageInner::scan -> new_txn and Transaction::scan -> LsmStorageInner::scan_with_ts
```

To create a transaction in `LsmStorageInner::scan`, provide an `Arc<LsmStorageInner>` to the transaction constructor. Change the `scan` signature to take `self: &Arc<Self>` instead of `&self`, so you can create a transaction with `let txn = self.mvcc().new_txn(self.clone(), /* ... */)`.

Change your `scan` function to return a `TxnIterator`. To ensure the snapshot remains live during iteration, `TxnIterator` stores the snapshot object. Inside `TxnIterator`, store a `FusedIterator<LsmIterator>` for now; this will change later when we implement OCC.

You do not need to implement `Transaction::put/delete` for now; all modifications still go through the engine.

## Task 3: Store Largest Timestamp in SST

In this task, modify:

```
src/table.rs
src/table/builder.rs
```

In your SST encoding, store the largest timestamp after the block metadata and load it during SST open. This helps determine the latest commit timestamp during recovery.

## Task 4: Recover Commit Timestamp

Now that SSTs record the largest timestamp and the WAL records per-entry timestamps, compute the largest timestamp committed before engine startup and use it as the latest committed timestamp when creating the `mvcc` object.

If WAL is disabled, compute the latest committed timestamp by finding the largest timestamp among SSTs. If WAL is enabled, also iterate all recovered memtables to find the largest timestamp.

In this task, modify:

```
src/lsm_storage.rs
```

There are no test cases for this section. After finishing, you should pass all persistence tests from previous chapters (including 2.5 and 2.6).

## Test Your Understanding

- So far, we have assumed that our SST files use a monotonically increasing ID as the file name. Is it okay to use `<level>_<begin_key>_<end_key>_<max_ts>.sst` as the SST file name? What potential problems could arise?
- Consider an alternative implementation of transaction/snapshot. In our implementation, we store `read_ts` in iterators and the transaction context, so users can access a consistent version of the database based on timestamps. Is it viable to store the current LSM state directly in the transaction context to obtain a consistent snapshot (i.e., all SST IDs, their level information, and all memtables + ts)? What are the pros/cons? What if the engine does not have memtables? What if the engine runs on a distributed storage system like S3?
- Suppose you are implementing a backup utility for the MVCC Mini-LSM engine. Is it enough to simply copy all SST files without backing up the LSM state? Why or why not?

We do not provide reference answers to these questions. Feel free to discuss them in the Discord community.

{{#include copyright.md}}
