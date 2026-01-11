<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# Timestamp Key Encoding and Refactor

In this chapter, you will:

- Refactor your implementation to use a key+timestamp representation.
- Make your code compile with the new key representation.

To run the test cases:

```
cargo x copy-test --week 3 --day 1
cargo x scheck
```

**Note: The MVCC subsystem is not fully implemented until Week 3 Day 2. You only need to pass Week 3 Day 1 tests and all Week 1 tests at the end of this day. Week 2 tests may not work because of compaction.**

## Task 0: Use MVCC Key Encoding

You will need to replace the key encoding module with the MVCC one. We have removed some interfaces from the original key module and implemented new comparators for the keys. If you followed the instructions in the previous chapters and did not use `into_inner` on the key, you should pass all Day 3 test cases after the refactors. Otherwise, look carefully at places where you compare keys without considering timestamps.

Specifically, the key type definition has been changed from:

```rust,no_run
pub struct Key<T: AsRef<[u8]>>(T);
```

...to:

```rust,no_run
pub struct Key<T: AsRef<[u8]>>(T /* user key */, u64 /* timestamp */);
```

...where we have a timestamp associated with keys. We only use this key representation internally. On the user interface, users do not provide a timestamp; therefore, some structures still use `&[u8]` instead of `KeySlice` in the engine. We will cover where we need to change function signatures later. For now, run:

```
cp mini-lsm-mvcc/src/key.rs mini-lsm-starter/src/
```

There are other ways of storing the timestamp. For example, we can still use the `pub struct Key<T: AsRef<[u8]>>(T);` representation but assume the last 8 bytes of the key are the timestamp. You can also implement this as part of the bonus tasks.

```plaintext
Alternative key representation: | user_key (varlen) | ts (8 bytes) | in a single slice
Our key representation: | user_key slice | ts (u64) |
```

In the key+timestamp encoding, for the same user key, larger timestamps are ordered first. For example,

```
("a", 233) < ("a", 0) < ("b", 233) < ("b", 0)
```

## Task 1: Encode Timestamps in Blocks

After replacing the key module, your code may not compile. In this chapter, your goal is to make it compile. In this task, modify:

```
src/block.rs
src/block/builder.rs
src/block/iterator.rs
```

`raw_ref()` and `len()` have been removed from the key API. Instead, use `key_ref()` to retrieve the slice of the user key and `key_len()` to retrieve the length of the user key. Refactor your block builder and decoding implementation to use the new APIs. You also need to encode timestamps in `BlockBuilder::add`. The new block entry record format:

```
key_overlap_len (u16) | remaining_key_len (u16) | key (remaining_key_len) | timestamp (u64)
```

You may use `raw_len` to estimate the space required by a key and store the timestamp after the user key.

After changing the block encoding, update the decoding in both `block.rs` and `iterator.rs` accordingly.

## Task 2: Encoding Timestamps in SSTs

Then, you can go ahead and modify the table format,

```
src/table.rs
src/table/builder.rs
src/table/iterator.rs
```

Specifically, change your block metadata encoding to include key timestamps. All other code remains the same. Because we use `KeySlice` in function signatures (e.g., `seek`, `add`), the new key comparator will automatically order keys by user key and timestamp.

In your table builder, use `key_ref()` to build the Bloom filter. This naturally creates a prefix Bloom filter for your SSTs.

## Task 3: LSM Iterators

Because we use associated generic types to make most iterators work for different key types (i.e., `&[u8]` and `KeySlice<'_>`), you do not need to modify merge iterators and concat iterators if they are implemented correctly. The `LsmIterator` is where we strip the timestamp from the internal key representation and return the latest version of a key to the user. In this task, modify:

```
src/lsm_iterator.rs
```

For now, we do not modify `LsmIterator` to keep only the latest version of a key. Simply make it compile by appending a timestamp to the user key when passing the key to the inner iterator, and stripping the timestamp when returning to the user. The behavior of your LSM iterator for now should be returning multiple versions of the same key to the user.

## Task 4: Memtable

For now, keep the existing memtable logic. Return a key slice to the user and flush SSTs with `TS_DEFAULT`. We will make the memtable MVCC in the next chapter. In this task, modify:

```
src/mem_table.rs
```

## Task 5: Engine Read Path

In this task, modify:

```
src/lsm_storage.rs
```

Now that keys include a timestamp, create iterators by seeking a key with a timestamp instead of only the user key. Create a key slice with `TS_RANGE_BEGIN`, which is the largest timestamp.

When checking if a user key is in a table, compare the user key without the timestamp.

At this point, build your implementation and pass all Week 1 test cases. All keys stored in the system use `TS_DEFAULT` (timestamp 0). We will make the engine fully multi-version and pass all test cases in the next two chapters.

{{#include copyright.md}}
