<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# Snack Time: Compaction Filters

Congratulations — you made it! In the previous chapter, you made your LSM engine multi-version capable, and users can now use transaction APIs to interact with it. To wrap up the week, we will implement a few small but important features. Welcome to Mini-LSM’s Week 3 snack time!

In this chapter, we generalize compaction garbage collection into compaction filters.

Currently, compaction retains all keys above the watermark and only the latest version at or below the watermark. We can extend compaction to help users automatically clean up unused data in the background.

Consider a case where the user stores database tables in Mini-LSM. Each row key is prefixed with the table name. For example:

```
table1_key1 -> row
table1_key2 -> row
table1_key3 -> row
table2_key1 -> row
table2_key2 -> row
```

Now the user executes `DROP TABLE table1`. The engine needs to clean up all data with the `table1` prefix.

There are several ways to do this. The user could scan all keys beginning with `table1` and request deletes. However, scanning a large database is slow and generates as many tombstones as existing keys. Scan-and-delete does not immediately free space — it adds more data, and space is only reclaimed when tombstones reach the bottom level.

Alternatively, they can use column families (covered in the “rest of your life” chapter). Each table is stored in a separate column family (a standalone LSM state), and the SST files can be removed directly when the table is dropped.

In this course, we implement a third approach: compaction filters. Filters can be added dynamically at runtime. During compaction, if a key matches a filter, we silently remove it in the background. For example, attaching a filter `prefix=table1` removes all matching keys during compaction.

## Task 1: Compaction Filter

In this task, modify:

```
src/compact.rs
```

Iterate all compaction filters in `LsmStorageInner::compaction_filters`. If the first version of a key at or below the watermark matches a filter, remove it instead of keeping it in the SST file.

To run the test cases:

```
cargo x copy-test --week 3 --day 7
cargo x scheck
```

Assume users will not `get` or `scan` keys within the filtered prefix range. Returning a value for keys in that range is undefined behavior.

{{#include copyright.md}}
