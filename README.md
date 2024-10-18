# MY SERVICE BUS


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
* PersistenceDest - can be path of a folder and can be an Microsoft Azure Storage account;
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
If PersistenceDest is a MicrosoftAzure connection string - content is going to be persisted to Azure Table Storage.
If PersistenceDest ends with ".sqlite" - content is going to be persisted to SQLite format.
If PersistenceDest is a folder path - content is going to be persisted to files in that folder in format:
* Folder = TableName
* Each File inside each table folder - is a partition of the table


