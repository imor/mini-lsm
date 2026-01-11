<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# Transaction and Optimistic Concurrency Control

In this chapter, you will implement all `Transaction` interfaces. Your implementation maintains a private workspace for modifications within a transaction and commits them in a batch, so changes remain visible only to the transaction until commit. We check for conflicts (i.e., serializable conflicts) at commit time — this is optimistic concurrency control (OCC).

To run the test cases:

```
cargo x copy-test --week 3 --day 5
cargo x scheck
```

## Task 1: Local Workspace — Put and Delete

In this task, modify:

```
src/mvcc/txn.rs
```

Implement `put` and `delete` by inserting the corresponding key/value into `local_storage`, which is a skiplist memtable without key timestamps. For deletes, insert an empty value rather than removing an entry from the skiplist.

## Task 2: Get and Scan

In this task, modify:

```
src/mvcc/txn.rs
```

For `get`, first probe local storage. If a value is found, return the value or `None` depending on whether it is a deletion marker. For `scan`, implement a `TxnLocalIterator` for the skiplist as in Chapter 1.1 (the memtable iterator without key timestamps). Store a `TwoMergeIterator<TxnLocalIterator, FusedIterator<LsmIterator>>` inside `TxnIterator`. Because `TwoMergeIterator` preserves deletion markers from child iterators, update `TxnIterator` to handle deletions correctly.

## Task 3: Commit

In this task, modify:

```
src/mvcc/txn.rs
```

Assume a transaction is used on a single thread. Once the transaction enters the commit phase, set `self.committed = true` so users cannot perform further operations. Your `put`, `delete`, `scan`, and `get` implementations should return an error if the transaction is already committed.

The commit implementation should collect all key-value pairs from local storage and submit a write batch to the storage engine.

## Task 4: Atomic WAL

In this task, modify:

```
src/wal.rs
src/mem_table.rs
src/lsm_storage.rs
```

`commit` produces a write batch, and currently batches are not atomic. Update the WAL to include a header and a footer around each batch to ensure atomicity.

The new WAL encoding is as follows:

```
|   HEADER   |                          BODY                                      |  FOOTER  |
|     u32    |   u16   | var | u64 |    u16    |  var  |           ...            |    u32   |
| batch_size | key_len | key | ts  | value_len | value | more key-value pairs ... | checksum |
```

`batch_size` is the size in bytes of the BODY section. `checksum` is computed over the BODY.

There are no test cases to verify this change. As long as you pass existing tests and implement the WAL format above, you are good.

Implement `Wal::put_batch` and `MemTable::put_batch`. The original `put` should treat a single key-value pair as a batch; call `put_batch` from `put`.

A batch should be handled within the same memtable and the same WAL, even if it exceeds the memtable size limit.

## Test Your Understanding

- With everything implemented so far, does the system satisfy snapshot isolation? If not, what else is needed to support snapshot isolation? (Note: snapshot isolation differs from serializable snapshot isolation covered in the next chapter.)
- What if the user wants to batch-import data (e.g., 1 TB)? If they use the transaction API, what advice would you give? Are there opportunities to optimize for this case?
- What is optimistic concurrency control? What would the system look like if we implemented pessimistic concurrency control in Mini-LSM instead?
- What happens if your system crashes and leaves a corrupted WAL on disk? How do you handle this situation?
- When committing the transaction, is it necessary to insert everything into the memtable as a batch, or can you insert key-by-key? Why?

## Bonus Tasks

- **Spill to Disk.** If the private workspace of a transaction gets too large, you may flush some data to disk.

{{#include copyright.md}}
