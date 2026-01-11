<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# Week 3 Overview: Multi-Version Concurrency Control

In this part, you will add MVCC to the LSM engine you built over the previous two weeks. We will add timestamp encoding to keys to maintain multiple versions and adjust parts of the engine to ensure old data is either retained or garbage-collected based on whether users are reading an older version.

The general approach to MVCC in this course is inspired by and partially based on [BadgerDB](https://github.com/dgraph-io/badger).

The key idea of MVCC is to store and access multiple versions of a key. Therefore, we will change the key format to `user_key + timestamp (u64)`. On the user interface side, we will add APIs to access historical versions. In summary, we add a monotonically increasing timestamp to each key.

In previous parts, we assumed newer keys are in upper levels of the LSM tree and older keys are in lower levels. During compaction, we kept only the latest version of a key if multiple versions appeared across levels, and merging adjacent levels/tiers preserved the “newer on top” invariant. In MVCC, the key with a larger timestamp is the newest. During compaction, we remove a key only if no user is accessing an older version. Although MVCC can work without strictly keeping the newest version in upper levels, we keep this invariant in the course: if multiple versions exist, later versions always appear in an upper level.

Generally, there are two ways to use a storage engine with MVCC support. If the engine is used as a standalone component and users do not want to assign timestamps manually, they use transaction APIs to store and retrieve data — timestamps are transparent. Alternatively, the engine can be integrated into a system where users manage timestamps themselves. Following BadgerDB’s terminology, the mode that hides timestamps is the un-managed mode, and the one that gives the user full control is the managed mode.

**Managed Mode APIs**

```
get(key, read_timestamp) -> (value, write_timestamp)
scan(key_range, read_timestamp) -> iterator<key, value, write_timestamp>
put/delete/write_batch(key, timestamp)
set_watermark(timestamp) # we will talk about watermarks soon!
```

**Un-managed/Normal Mode APIs**

```
get(key) -> value
scan(key_range) -> iterator<key, value>
start_transaction() -> txn
txn.put/delete/write_batch(key, timestamp)
```

As you can see, the managed mode APIs require the user to provide a timestamp for operations. The timestamp may come from a centralized timestamp service or from logs of other systems (e.g., Postgres logical replication). The user must also specify a watermark, below which versions can be removed.

For the un-managed APIs, behavior is similar to what we implemented before, except writes and reads happen within a transaction. A transaction observes a consistent snapshot: concurrent writes by other threads/transactions are invisible. The storage engine manages timestamps internally and does not expose them.

This week, we will first spend three days refactoring the table format and memtables. We will change the key format to a key slice plus a timestamp. After that, we will implement the APIs needed to provide consistent snapshots and transactions.

We have 7 chapters (days) in this part:

- [Day 1: Timestamp Key Refactor](./week3-01-ts-key-refactor.md). Change the `key` module to the MVCC version and refactor your system to use keys with timestamps.
- [Day 2: Snapshot Read — Memtables and Timestamps](./week3-02-snapshot-read-part-1.md). Refactor the memtable and the write path to support multi-version reads/writes.
- [Day 3: Snapshot Read — Transaction API](./week3-03-snapshot-read-part-2.md). Implement the transaction API and finish the remaining read/write path to support snapshot reads.
- [Day 4: Watermark and Garbage Collection](./week3-04-watermark.md). Implement watermark computation and perform garbage collection during compaction to remove old versions.
- [Day 5: Transaction and Optimistic Concurrency Control](./week3-05-txn-occ.md). Create a private workspace for transactions and commit them in a batch so that modifications are not visible to other transactions until commit.
- [Day 6: Serializable Snapshot Isolation](./week3-06-serializable.md). Implement OCC serializability checks to ensure modifications are serializable and abort transactions that violate serializability.
- [Day 7: Compaction Filters](./week3-07-compaction-filter.md). Generalize compaction-time garbage collection into compaction filters that remove data during compaction based on user-defined rules.

{{#include copyright.md}}
