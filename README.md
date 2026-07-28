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


### Write operations and the `TimeStamp` field

For almost every write operation the server **assigns the `TimeStamp` itself** (its
own clock at the moment of the write) and ignores whatever `TimeStamp` the client
sent. This holds for `Insert`, `InsertOrReplace`, `Bulk/InsertOrReplace`,
`Bulk/CleanAndBulkInsert*`, transactions, etc. Here `TimeStamp` means "when the
server stored the row".

#### `InsertOrReplaceIfNew` — conditional upsert by client `TimeStamp`

There is one family of operations where `TimeStamp` means something different — the
**version of the object in a distributed system**, assigned by the *client*:

| Endpoint | Shape |
|----------|-------|
| `POST /api/Row/InsertOrReplaceIfNew` | single entity |
| `POST /api/Bulk/InsertOrReplaceIfNew` | array of entities |
| `POST /api/Bulk/InsertOrReplaceIfNewByChunks` | accumulate chunks, then `...Commit` / `...Cancel` |

Semantics: a row is written **only** when it is missing, or when the incoming
`TimeStamp` is **strictly greater** than the `TimeStamp` already stored. Equal or
older timestamps are silently skipped. This is a last-writer-wins upsert keyed on a
client-owned version, used to converge replicas that may deliver the same object out
of order.

Because the timestamp *is* the version, it is **mandatory** here: every entity must
carry a valid ISO-8601 `TimeStamp`. An entity with a missing or unparseable
`TimeStamp` makes the whole request fail with **HTTP 400** naming the offender:

```
Entity with PartitionKey '<pk>' RowKey '<rk>' does not contain TimeStamp
```

(For the chunked flow this validation runs per chunk at upload time, so a bad chunk
is rejected before the commit.) The response of the successful bulk / chunked
operations is an empty `202`; the single one returns `200`.

Server-clock substitution (the default of every *other* write) and client-version
comparison (these `IfNew` operations) are two distinct request-parsing paths and
must not be confused.

#### `Bulk/InsertOrReplace?useTimestamp=true` — keep client timestamps on a plain upsert

`POST /api/Bulk/InsertOrReplace` takes an optional `useTimestamp` query flag:

- **absent / `false`** (default): the historical behaviour — the server assigns its
  own clock to every row and ignores whatever `TimeStamp` the client sent.
- **`true`**: each row keeps the `TimeStamp` that came in the entity. This is a plain
  **unconditional** upsert (unlike `InsertOrReplaceIfNew` there is *no* "is it newer"
  check — every row is written), it only controls *which* `TimeStamp` gets stored. As
  with the `IfNew` family the timestamp is then mandatory: any entity with a missing
  or unparseable `TimeStamp` fails the whole request with the same **HTTP 400** naming
  the offender.


