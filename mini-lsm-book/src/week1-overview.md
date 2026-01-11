<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# Week 1 Overview: Mini-LSM

![Chapter Overview](./lsm-tutorial/week1-overview.svg)

In the first week of the course, you will build the necessary storage formats for the storage engine, the system's read and write paths, and have a working implementation of an LSM-based key-value store. There are 7 chapters (days) in this part.

- [Day 1: Memtable](./week1-01-memtable.md). You will implement the in-memory read and write paths of the system.
- [Day 2: Merge Iterator](./week1-02-merge-iterator.md). You will extend what you have built in day 1 and implement a `scan` interface for your system.
- [Day 3: Block Encoding](./week1-03-block.md). We start the on-disk structure and implement block encoding/decoding.
- [Day 4: SST Encoding](./week1-04-sst.md). SSTs are composed of blocks. By the end of the day, you will have the basic building blocks of the LSM on-disk structure.
- [Day 5: Read Path](./week1-05-read-path.md). Now that we have both in-memory and on-disk structures, we can combine them to have a fully working read path for the storage engine.
- [Day 6: Write Path](./week1-06-write-path.md). In day 5 the test harness generates the structures; in day 6, you will control SST flushes yourself. You will implement flushing to level-0 SSTs, completing the storage engine.
- [Day 7: SST Optimizations](./week1-07-sst-optimizations.md). You will implement several SST format optimizations to improve system performance.

At the end of the week, your storage engine should handle all get/scan/put requests. The only missing parts are persisting the LSM state to disk and organizing SSTs on disk more efficiently. You will have a working **Mini-LSM** storage engine.

{{#include copyright.md}}
