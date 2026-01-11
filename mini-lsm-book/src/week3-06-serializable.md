<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# (A Partial) Serializable Snapshot Isolation

Now we will add a conflict-detection algorithm at transaction commit time so the engine provides a degree of serializability.

To run the test cases:

```
cargo x copy-test --week 3 --day 6
cargo x scheck
```

Let’s walk through a serializability example. Consider two transactions:

```
txn1: put("key1", get("key2"))
txn2: put("key2", get("key1"))
```

The initial state is `key1=1, key2=2`. Serializability means the outcome is equivalent to executing the transactions one by one in some order. If we execute `txn1` then `txn2`, we get `key1=2, key2=2`. If we execute `txn2` then `txn1`, we get `key1=1, key2=1`.

However, with our current implementation, if the execution of these two transactions overlaps:

```
txn1: get key2 <- 2
txn2: get key1 <- 1
txn1: put key1=2, commit
txn2: put key2=1, commit
```

We get `key1=2, key2=1`. This cannot be produced by any serial execution of the two transactions. This phenomenon is called write skew.

With serializability validation, we ensure modifications correspond to some serial execution order. This allows critical workloads that require serializable execution. For example, if a user runs bank-transfer workloads on Mini-LSM, they expect the total balance to remain constant at any point in time. We cannot guarantee this invariant without serializable checks.

One technique for serializability validation is to record the read set and write set of each transaction. We validate before committing a transaction (optimistic concurrency control). If the transaction’s read set overlaps with the write set of any transaction committed after its read timestamp, validation fails and we abort.

Back to the example: suppose `txn1` and `txn2` both start at timestamp 1.

```
txn1: get key2 <- 2
txn2: get key1 <- 1
txn1: put key1=2, commit ts = 2
txn2: put key2=1, start serializable verification
```

When validating `txn2`, consider all transactions with commit timestamps in the exclusive range `(read_ts, expected_commit_ts)` (here, `1 < ts < 3`). The only matching transaction is `txn1`. `txn1`’s write set is `{key1}`, and `txn2`’s read set is `{key1}`. They overlap, so we must abort `txn2`.

## Task 1: Track Read Set in Get and Write Set

In this task, modify:

```
src/mvcc/txn.rs
src/mvcc.rs
```

When `get` is called, add the key to the transaction’s read set. In our implementation, we store key hashes to reduce memory usage and speed up probing; this may cause false positives if two keys share a hash. Use `farmhash::hash32` to hash keys. Note: even if `get` returns not found, still record the key in the read set.

In `LsmMvccInner::new_txn`, create empty read/write sets when `serializable = true`.

## Task 2: Track Read Set in Scan

In this task, modify:

```
src/mvcc/txn.rs
```

In this course, we only guarantee serializability for `get` requests. You still need to track the read set for scans, but in specific cases, scans may still yield non-serializable results.

To understand why this is hard, consider:

```
txn1: put("key1", len(scan(..)))
txn2: put("key2", len(scan(..)))
```

If the database starts with `a=1, b=2`, we should get either `a=1, b=2, key1=2, key2=3` or `a=1, b=2, key1=3, key2=2`. However, if execution proceeds as follows:

```
txn1: len(scan(..)) = 2
txn2: len(scan(..)) = 2
txn1: put key1 = 2, commit, read set = {a, b}, write set = {key1}
txn2: put key2 = 2, commit, read set = {a, b}, write set = {key2}
```

This passes our serializability validation yet does not correspond to any serial order. Fully correct serializability validation must track key ranges; using key hashes is efficient when only `get` is used. See the bonus tasks for implementing correct serializability checks.

## Task 3: Engine Interface and Serializable Validation

In this task, modify:

```
src/mvcc/txn.rs
src/lsm_storage.rs
```

Implement validation in the commit phase. Acquire `commit_lock` for every transaction commit to ensure only one transaction enters verification and commit at a time.

Iterate over all transactions with commit timestamps in `(read_ts, expected_commit_ts)` (exclusive bounds) and check if the current transaction’s read set overlaps any matching transaction’s write set. If it passes, submit a write batch and insert the transaction’s write set into `self.inner.mvcc().committed_txns` keyed by the commit timestamp.

Skip the check if `write_set` is empty. Read-only transactions can always commit.

Also modify the `put`, `delete`, and `write_batch` APIs in `LsmStorageInner`. Define a helper `write_batch_inner` to process a batch. If `options.serializable = true`, `put`, `delete`, and the user-facing `write_batch` should create a transaction instead of writing directly. The batch helper should return a `u64` commit timestamp so `Transaction::commit` can store committed data into the MVCC structure.

## Task 4: Garbage Collection

In this task, modify:

```
src/mvcc/txn.rs
```

When committing a transaction, clean up the committed-transaction map by removing all transactions below the watermark, as they will not be involved in future serializability validations.

## Test Your Understanding

- Consider building a relational database on Mini-LSM where each row is a key-value pair (key: primary key; value: serialized row) with serializability verification enabled. Does the database directly gain ANSI serializable isolation? Why or why not?
- What we implement here is write snapshot isolation (see “A critique of snapshot isolation”), which guarantees serializability. Are there cases where execution is serializable but would be rejected by write snapshot-isolation validation?
- Some databases claim serializable snapshot isolation by tracking only keys accessed in gets/scans (not key ranges). Do they really prevent write skews caused by phantoms? (For example, see BadgerDB.)

We do not provide reference answers to these questions. Feel free to discuss them in the Discord community.

## Bonus Tasks

- **Read-Only Transactions.** With serializability enabled, keep track of the read set for a transaction.
- **Precision/Predicate Locking.** Maintain the read set using ranges instead of single keys to support scans over the full key space. This enables serializability verification for scans.

{{#include copyright.md}}
