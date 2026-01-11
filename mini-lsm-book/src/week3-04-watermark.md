<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# Watermark and Garbage Collection

In this chapter, you will implement structures to track the lowest read timestamp in use and collect unused versions from SSTs during compaction.

To run the test cases:

```
cargo x copy-test --week 3 --day 4
cargo x scheck
```

## Task 1: Implement Watermark

In this task, you will need to modify:

```
src/mvcc/watermark.rs
```

The watermark tracks the lowest `read_ts` in the system. When a new transaction is created, it should call `add_reader` to add its read timestamp for tracking. When a transaction aborts or commits, it should remove its timestamp from the watermark. The watermark structure returns the lowest `read_ts` in the system when `watermark()` is called. If there are no ongoing transactions, it returns `None`.

You may implement the watermark using a `BTreeMap` that maintains, for each `read_ts`, how many snapshots are using that timestamp. Do not keep entries with zero readers in the map.

## Task 2: Maintain Watermark in Transactions

In this task, you will need to modify:

```
src/mvcc/txn.rs
src/mvcc.rs
```

Add the `read_ts` to the watermark when a transaction starts, and remove it when `drop` is called for the transaction.

## Task 3: Garbage Collection in Compaction

In this task, you will need to modify:

```
src/compact.rs
```

Now that we have a watermark for the system, we can clean up unused versions during the compaction process.

- If a version of a key is above the watermark, keep it.
- For all versions of a key at or below the watermark, keep only the latest version.

For example, if we have watermark = 3 and the following data:

```
a@4=del <- above watermark
a@3=3   <- latest version at or below the watermark
a@2=2   <- can be removed; no reader can observe it
a@1=1   <- can be removed; no reader can observe it
b@1=1   <- latest version at or below the watermark
c@4=4   <- above watermark
d@3=del <- can be removed if compacting to bottom-most level
d@2=2   <- can be removed
```

Compacting these keys yields:

```
a@4=del
a@3=3
b@1=1
c@4=4
d@3=del (can be removed if compacting to bottom-most level)
```

Assume these are all keys in the engine. If we scan at ts = 3, we get `a=3, b=1, c=4` before/after compaction. If we scan at ts = 4, we get `b=1, c=4` before/after compaction. Compaction _will not_ and _should not_ affect transactions with a read timestamp >= the watermark.

## Test Your Understanding

- In our implementation, we manage watermarks ourselves with the lifecycle of `Transaction` (so-called unmanaged mode). If the user intends to manage key timestamps and watermarks themselves (e.g., they have their own timestamp generator), what changes are needed in the `write_batch`/`get`/`scan` APIs to validate requests? Are there architectural assumptions that might be hard to maintain in this case?
- Why do we need to store an `Arc<Transaction>` inside a transaction iterator?
- What is the condition to fully remove a key from the SST file?
- For now, we only remove a key when compacting to the bottom-most level. Is there any earlier time when we can remove the key? (Hint: you know the start/end key of each SST in all levels.)
- Consider the case where the user creates a long-running transaction and we cannot garbage-collect anything. The user keeps updating a single key. Eventually, there could be a key with thousands of versions in a single SST file. How would it affect performance, and how would you deal with it?

## Bonus Tasks

- **O(1) Watermark.** You may implement an amortized O(1) watermark structure by using a hash map or a cyclic queue.

{{#include copyright.md}}
