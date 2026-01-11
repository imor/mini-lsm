<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# Mini-LSM Course Overview

## Course Structure

![Course Overview](lsm-tutorial/00-full-overview.svg)

This course has three parts (weeks). In the first week, we focus on the storage structure and storage format of an LSM storage engine. In the second week, we dive deep into compactions and implement persistence support for the storage engine. In the third week, we implement multi-version concurrency control.

- [The First Week: Mini-LSM](./week1-overview.md)
- [The Second Week: Compaction and Persistence](./week2-overview.md)
- [The Third Week: Multi-Version Concurrency Control](./week3-overview.md)

See [Environment Setup](./00-get-started.md) to set up your environment.

## Overview of LSM

An LSM storage engine generally has three parts:

1. Write-ahead log to persist temporary data for recovery.
2. SSTs on disk to maintain an LSM-tree structure.
3. Mem-tables in memory for batching small writes.

The storage engine generally provides the following interfaces:

- `Put(key, value)`: store a key-value pair in the LSM tree.
- `Delete(key)`: remove a key and its corresponding value.
- `Get(key)`: get the value corresponding to a key.
- `Scan(range)`: get a range of key-value pairs.

To ensure persistence,

- `Sync()`: ensure all operations before `sync` are persisted to disk.

Some engines choose to combine `Put` and `Delete` into a single operation called `WriteBatch`, which accepts a batch of key-value pairs.

In this course, we assume the LSM tree is using a leveled compaction algorithm, which is commonly used in real-world systems.

### Write Path

![Write Path](lsm-tutorial/00-lsm-write-flow.svg)

The write path of LSM contains four steps:

1. Write the key-value pair to the write-ahead log so that it can be recovered after the storage engine crashes.
2. Write the key-value pair to the memtable. After (1) and (2) complete, we notify the user that the write operation is complete.
3. (In the background) When a mem-table is full, we freeze it into an immutable mem-table and flush it to disk as an SST file in the background.
4. (In the background) The engine compacts some files in some levels into lower levels to maintain a good shape for the LSM tree so that read amplification is low.

### Read Path

![Read Path](lsm-tutorial/00-lsm-read-flow.svg)

To read a key:

1. First probe all mem-tables from the latest to the oldest.
2. If the key is not found, search the entire LSM tree containing SSTs to find the data.

There are two types of read: lookup and scan. Lookup finds one key in the LSM tree, while scan iterates all keys within a range in the storage engine. We will cover both of them throughout the course.

{{#include copyright.md}}
