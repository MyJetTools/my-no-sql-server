# MyNoSqlServer


### Example of config file
```yaml
PersistenceDest: ~/.mynosqldb/data 
CompressData: true
MaxPayloadSize: 4000000
Location: M1
TableApiKey: 123
InitThreadsAmount: 1
SkipBrokenPartitions: false
SaveThreadsAmount: 2
TcpSendTimeoutSec: 30
BackupFolder: ~/.mynosqldb/backup
BackupIntervalHours: 24
MaxBackupsToKeep: 5
```

### Parameters:
* PersistenceDest - where the data is persisted. A path ending with `.sqlite`/`.sqlite3`/`.db` selects the SQLite backend; any other path is treated as a directory and selects the slotted-page files backend (see "Persistence Types" below);
* CompressData - true/false - enable/disable compression of data between nodes;
* MaxPayloadSize - max size of payload in bytes which is sent to Readers per round trip;
* Location - shows in statusbar of the UI;
* TableApiKey - API key to make irreversible operations with tables through api;
* InitThreadsAmount - amount of threads to initialize data from Storage;
* SkipBrokenPartitions - skip broken partitions during initialization;
* SaveThreadsAmount - amount of threads to save data to Storage;
* TcpSendTimeoutSec - timeout for tcp send operation, otherwise connection will be closed;
* BackupFolder - folder to store backups as ZIP Archives;
* BackupIntervalHours - interval between backups;
* MaxBackupsToKeep - max amount of backups to keep in BackupFolder;



### Persistence Types

The persistence backend is chosen by the shape of `PersistenceDest`. In both
backends the unit of persistence is a **partition**: the whole partition is
serialized to a JSON array of its rows and compressed with **zstd**. On startup
everything is read into memory, the in-memory tables are rebuilt, and the raw
persisted bytes are released.

#### SQLite — `PersistenceDest` ends with `.sqlite` / `.sqlite3` / `.db`

```yaml
PersistenceDest: ~/.mynosqldb/data.sqlite
```

One row per partition in the `partitions` table, `content = base64(zstd(rows))`,
plus a `tables_metadata` table for table attributes. In-place updates, free-page
reuse and compaction are handled by SQLite itself (`VACUUM` runs periodically).

#### Slotted page-files — `PersistenceDest` is a directory

```yaml
PersistenceDest: ~/.mynosqldb/data
```

Data is stored in a set of **size-class page-files** inside the directory
(candle-storage style). Each page-file holds fixed-size slots; the file name is
the slot size in bytes (powers of two starting at 512):

```
<dir>/tables.meta     # table attributes (YAML; legacy JSON still loads), rewritten atomically on change
<dir>/512             # page-file: array of 512-byte slots
<dir>/1024
...
```

A partition is written into the smallest size class its compressed payload fits
into. As long as it keeps fitting the same class it is **overwritten in place**
(no reallocation); if it outgrows the class it moves to a larger one and the old
slot is freed for reuse. Each slot is self-describing (carries its table +
partition key, and `body_len == 0` marks a freed slot) and carries a `crc32`,
so recovery is a plain scan of the page-files — there is no separate on-disk
key index and no persisted free-list. A slot with a failing crc (a torn write)
is skipped on recovery (honouring `SkipBrokenPartitions`).

> There is no automatic conversion between the two formats. To move data from
> one backend to another, take a backup and restore it into a server configured
> with the target `PersistenceDest` — the restore path re-persists everything in
> the new format.


