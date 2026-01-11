<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# Compaction Implementation

![Chapter Overview](./lsm-tutorial/week2-01-full.svg)

In this chapter, you will:

- Implement the compaction logic that combines some files and produces new files.
- Implement the logic to update the LSM state and manage SST files on the filesystem.
- Update the LSM read path to incorporate LSM levels.

To copy the test cases into the starter code and run them,

```
cargo x copy-test --week 2 --day 1
cargo x scheck
```

<div class="warning">

It might be helpful to read the [week 2 overview](./week2-overview.md) for a general understanding of compactions before this chapter.

</div>

## Task 1: Compaction Implementation

In this task, you will implement the core logic of compaction — perform a merge sort over a set of SST files into a sorted run. You will need to modify:

```
src/compact.rs
```

Specifically, the `force_full_compaction` and `compact` functions. `force_full_compaction` is the compaction trigger that decides which files to compact and updates the LSM state. `compact` does the actual compaction job that merges some SST files and returns a set of new SST files.

Your compaction implementation should take all SSTs in the storage engine, merge them using `MergeIterator`, and then use the SST builder to write the result into new files. You will need to split SST files if a file is too large. After compaction completes, update the LSM state to add the new sorted run to the first level of the LSM tree. You will also need to remove unused files in the LSM tree. In your implementation, SSTs should only be stored in two places: L0 SSTs and L1 SSTs. That is, the `levels` structure in the LSM state should contain only one vector. In `LsmStorageState`, we have already initialized the LSM to have L1 in the `levels` field.

Compaction should not block L0 flush; therefore, do not take the state lock while merging files. Only take the state lock at the end of the compaction process when you update the LSM state, and release the lock immediately after finishing modifications.

You can assume the user will ensure only one compaction is running. `force_full_compaction` will be called in only one thread at any time. SSTs placed in level 1 should be sorted by their first key and should not have overlapping key ranges.

<details>

<summary>Spoilers: Compaction Pseudo Code</summary>

```rust,no_run
fn force_full_compaction(&self) {
    let ssts_to_compact = {
        let state = self.state.read();
        state.l0_sstables + state.levels[0]
    };
    let new_ssts = self.compact(FullCompactionTask(ssts_to_compact))?;
    {
        let state_lock = self.state_lock.lock();
        let state = self.state.write();
        state.l0_sstables.remove(/* the ones being compacted */);
        state.levels[0] = new_ssts; // new SSTs added to L1
    };
    std::fs::remove(ssts_to_compact)?;
}
```

</details>

In your compaction implementation, you only need to handle `FullCompaction` for now, where the task information contains the SSTs to be compacted. You will also need to ensure the order of SSTs is correct so that the latest version of a key is written into the new SST.

Because we always compact all SSTs, if we find multiple versions of a key, we can simply retain the latest one. If the latest version is a delete marker, we do not need to keep it in the produced SST files. This does not apply to the compaction strategies in the next few chapters.

There are some things that you might need to think about.

- How does your implementation handle L0 flush in parallel with compaction? (Do not take the state lock during compaction, and consider new L0 files produced while compaction is in progress.)
- If your implementation removes the original SST files immediately after the compaction completes, will it cause problems in your system? (Generally no on macOS/Linux because the OS will not actually remove the file until no file handle is being held.)

## Task 2: Concat Iterator

In this task, you will need to modify,

```
src/iterators/concat_iterator.rs
```

Now that you have created sorted runs in your system, it is possible to do a simple optimization over the read path. You do not always need to create merge iterators for your SSTs. If SSTs belong to one sorted run, you can create a concat iterator that simply iterates the keys in each SST in order, because SSTs in one sorted run do not contain overlapping key ranges and they are sorted by their first key. We do not want to create all SST iterators in advance (because it will lead to one block read), and therefore we only store SST objects in this iterator.

## Task 3: Integrate with the Read Path

In this task, you will need to modify,

```
src/lsm_iterator.rs
src/lsm_storage.rs
src/compact.rs
```

Now that we have the two-level structure for your LSM tree, and you can change your read path to use the new concat iterator to optimize the read path.

You will need to change the inner iterator type of the `LsmStorageIterator`. After that, you can construct a two merge iterator that merges memtables and L0 SSTs, and another merge iterator that merges that iterator with the L1 concat iterator.

You can also change your compaction implementation to leverage the concat iterator.

You will need to implement `num_active_iterators` for concat iterator so that the test case can test if concat iterators are being used by your implementation, and it should always be 1.

To test your implementation interactively,

```shell
cargo run --bin mini-lsm-cli-ref -- --compaction none # reference solution
cargo run --bin mini-lsm-cli -- --compaction none # your solution
```

And then,

```
fill 1000 3000
flush
fill 1000 3000
flush
full_compaction
fill 1000 3000
flush
full_compaction
get 2333
scan 2000 2333
```

## Test Your Understanding

- What are the definitions of read/write/space amplifications? (This is covered in the overview chapter)
- What are the ways to accurately compute the read/write/space amplifications, and what are the ways to estimate them?
- Is it correct that a key will take some storage space even if a user requests to delete it?
- Given that compaction takes a lot of write bandwidth and read bandwidth and may interfere with foreground operations, it can be a good idea to postpone compaction when there is heavy write flow. It may even be beneficial to stop or pause existing compaction tasks in this situation. What do you think of this idea? (Read the [SILK: Preventing Latency Spikes in Log-Structured Merge Key-Value Stores](https://www.usenix.org/conference/atc19/presentation/balmau) paper!)
- Is it a good idea to use/fill the block cache for compactions? Or is it better to fully bypass the block cache during compaction?
- Does it make sense to have a `struct ConcatIterator<I: StorageIterator>` in the system?
- Some researchers/engineers propose to offload compaction to a remote server or a serverless lambda function. What are the benefits, and what might be the potential challenges and performance impacts of doing remote compaction? (Think of the point when a compaction completes and what happens to the block cache on the next read request...)

We do not provide reference answers to the questions; feel free to discuss them in the Discord community.

{{#include copyright.md}}
